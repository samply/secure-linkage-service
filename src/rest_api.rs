use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};
use tokio::time::{Duration, timeout};
use tracing::{error, info, warn};

use crate::domain::*;
use crate::transformations::{compute_linkage, crypto::*};
use ring::rand::SystemRandom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkageRequestData {
    #[serde(rename = "match")]
    pub match_bit: u64,
    #[serde(rename = "tentativeMatch")]
    pub tentative_match: u64,
    #[serde(rename = "bestIndex")]
    pub best_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLinkageRequest {
    pub role: String,
    pub result: Vec<LinkageRequestData>,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientLinkageRequest {
    pub role: String,
    pub result: Vec<LinkageRequestData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LinkageRequest {
    Server(ServerLinkageRequest),
    Client(ClientLinkageRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkageResponse {
    pub status: String,
    pub linkage_ids: Option<Vec<Option<String>>>,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct PendingRequest {
    pub source_party: String,
    pub destination_party: String,
    pub data: LinkageRequest,
    pub response_sender: oneshot::Sender<LinkageResponse>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    pub parties: Arc<RwLock<HashMap<String, LinkageParty>>>,
    pub timeout_duration: Duration,
}

impl AppState {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(20))
    }

    pub fn with_timeout(timeout_duration: Duration) -> Self {
        let mut parties = HashMap::new();
        let rand = SystemRandom::new();

        // Pre-register some test parties
        let test_parties = vec!["dkfz", "tuda", "test_client", "test_server"];

        for party_name in test_parties {
            let key_material = generate_random_key_material(&rand).unwrap();
            let key = generate_key(key_material).unwrap();
            let party = LinkageParty::new(HostName(party_name.to_string()), key);
            parties.insert(party_name.to_string(), party);
        }

        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            parties: Arc::new(RwLock::new(parties)),
            timeout_duration,
        }
    }
}

pub async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new();

    let app = Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            post(handle_linkage_request),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(address).await?;
    info!("Server listening on {}", address);

    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn handle_linkage_request(
    Path((source_party, destination_party)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(request): Json<LinkageRequest>,
) -> Result<(StatusCode, Json<LinkageResponse>), StatusCode> {
    info!(
        "Received linkage request from {} to {}",
        source_party, destination_party
    );

    let parties = state.parties.read().await;
    let source_party_obj = parties.get(&source_party).ok_or(StatusCode::BAD_REQUEST)?;
    let dest_party_obj = parties
        .get(&destination_party)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let source_party_clone = source_party_obj.clone();
    let dest_party_clone = dest_party_obj.clone();
    drop(parties);

    match request {
        LinkageRequest::Server(_) => {
            // Server role request - immediate 200 OK, store for later matching
            let request_id = format!("{}:{}", source_party, destination_party);
            let (tx, rx) = oneshot::channel();

            let pending = PendingRequest {
                source_party: source_party.clone(),
                destination_party: destination_party.clone(),
                data: request,
                response_sender: tx,
            };

            state
                .pending_requests
                .write()
                .await
                .insert(request_id.clone(), pending);

            // Spawn a task to handle timeout
            let state_clone = state.clone();
            let request_id_clone = request_id.clone();
            let timeout_duration = state.timeout_duration;
            tokio::spawn(async move {
                if let Err(_) = timeout(timeout_duration, rx).await {
                    // Timeout occurred, remove the pending request
                    state_clone
                        .pending_requests
                        .write()
                        .await
                        .remove(&request_id_clone);
                    warn!("Request {} timed out", request_id_clone);
                }
            });

            Ok((
                StatusCode::OK,
                Json(LinkageResponse {
                    status: "accepted".to_string(),
                    linkage_ids: None,
                    message: Some("Server data received, waiting for client data".to_string()),
                }),
            ))
        }
        LinkageRequest::Client(client_req) => {
            // Client role request - look for matching server request
            let lookup_key = format!("{}:{}", destination_party, source_party);

            let mut pending_requests = state.pending_requests.write().await;
            if let Some(server_pending) = pending_requests.remove(&lookup_key) {
                drop(pending_requests);

                // We have a matching server request, perform linkage
                match perform_linkage(
                    &source_party_clone,
                    &dest_party_clone,
                    &client_req,
                    &server_pending.data,
                )
                .await
                {
                    Ok(linkage_result) => {
                        let linkage_ids: Vec<Option<String>> = linkage_result
                            .0
                            .iter()
                            .map(|opt_id| opt_id.as_ref().map(|id| id.0.0.to_string()))
                            .collect();

                        let response = LinkageResponse {
                            status: "completed".to_string(),
                            linkage_ids: Some(linkage_ids.clone()),
                            message: Some("Linkage computation completed".to_string()),
                        };

                        // Send response to the waiting server request
                        let _ = server_pending.response_sender.send(response.clone());

                        Ok((StatusCode::OK, Json(response)))
                    }
                    Err(e) => {
                        let error_response = LinkageResponse {
                            status: "error".to_string(),
                            linkage_ids: None,
                            message: Some(format!("Linkage computation failed: {}", e)),
                        };

                        let _ = server_pending.response_sender.send(error_response.clone());

                        error!("Linkage computation failed: {}", e);
                        Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                    }
                }
            } else {
                // No matching server request found - return 202 and wait
                let request_id = format!("{}:{}", source_party, destination_party);
                let (tx, rx) = oneshot::channel();

                let pending = PendingRequest {
                    source_party: source_party.clone(),
                    destination_party: destination_party.clone(),
                    data: LinkageRequest::Client(client_req),
                    response_sender: tx,
                };

                pending_requests.insert(request_id.clone(), pending);
                drop(pending_requests);

                // Wait for matching server request with timeout
                let timeout_duration = state.timeout_duration;
                match timeout(timeout_duration, rx).await {
                    Ok(Ok(response)) => Ok((StatusCode::OK, Json(response))),
                    Ok(Err(_)) => {
                        state.pending_requests.write().await.remove(&request_id);
                        Ok((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(LinkageResponse {
                                status: "error".to_string(),
                                linkage_ids: None,
                                message: Some("Internal error occurred".to_string()),
                            }),
                        ))
                    }
                    Err(_) => {
                        // Timeout
                        state.pending_requests.write().await.remove(&request_id);
                        Ok((
                            StatusCode::REQUEST_TIMEOUT,
                            Json(LinkageResponse {
                                status: "timeout".to_string(),
                                linkage_ids: None,
                                message: Some(
                                    "No matching server request received within timeout"
                                        .to_string(),
                                ),
                            }),
                        ))
                    }
                }
            }
        }
    }
}

async fn perform_linkage(
    client_party: &LinkageParty,
    server_party: &LinkageParty,
    client_request: &ClientLinkageRequest,
    server_request: &LinkageRequest,
) -> Result<LinkageResult, LinkageError> {
    let server_data = match server_request {
        LinkageRequest::Server(server_req) => {
            let result_data: Result<Vec<ResultData>, LinkageError> = server_req
                .result
                .iter()
                .map(|item| {
                    ResultData::from_strings(
                        &item.tentative_match.to_string(),
                        &item.match_bit.to_string(),
                        &item.best_index.to_string(),
                    )
                })
                .collect();

            let record_ids: Result<Vec<RecordId>, LinkageError> =
                server_req.ids.iter().map(|id| RecordId::new(id)).collect();

            ServerLinkageData::new(result_data?, record_ids?)
        }
        _ => return Err(LinkageError::EmptyInput),
    };

    let client_data = {
        let result_data: Result<Vec<ResultData>, LinkageError> = client_request
            .result
            .iter()
            .map(|item| {
                ResultData::from_strings(
                    &item.tentative_match.to_string(),
                    &item.match_bit.to_string(),
                    &item.best_index.to_string(),
                )
            })
            .collect();

        ClientLinkageData::new(result_data?)
    };

    compute_linkage(client_party, server_party, &client_data, &server_data)
}
