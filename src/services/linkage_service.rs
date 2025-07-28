use crate::core::compute_linkage;
use crate::domain::{
    ClientLinkageData, LinkageError, LinkageParty, LinkageResult, RecordId, ResultData,
    ServerLinkageData,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};
use tokio::time::Duration;
use tracing::warn;

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
pub struct LinkageService {
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    timeout_duration: Duration,
}

impl LinkageService {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(20))
    }

    pub fn with_timeout(timeout_duration: Duration) -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            timeout_duration,
        }
    }

    pub async fn process_linkage_request(
        &self,
        source_party: String,
        destination_party: String,
        source_party_obj: &LinkageParty,
        dest_party_obj: &LinkageParty,
        request: LinkageRequest,
    ) -> Result<LinkageResponse, LinkageServiceError> {
        let current_key = format!("{}:{}", source_party, destination_party);
        let matching_key = format!("{}:{}", destination_party, source_party);

        let mut pending_requests = self.pending_requests.write().await;

        if let Some(matching_request) = pending_requests.remove(&matching_key) {
            // Found a match! Process immediately
            drop(pending_requests);

            let result = match (&request, &matching_request.data) {
                (LinkageRequest::Client(client_req), LinkageRequest::Server(_)) => {
                    self.perform_linkage(
                        source_party_obj,
                        dest_party_obj,
                        client_req,
                        &matching_request.data,
                    )
                    .await
                }
                (LinkageRequest::Server(_), LinkageRequest::Client(client_req)) => {
                    self.perform_linkage(dest_party_obj, source_party_obj, client_req, &request)
                        .await
                }
                _ => {
                    return Err(LinkageServiceError::InvalidRequestPairing(
                        "Both requests are the same type".to_string(),
                    ));
                }
            };

            match result {
                Ok(linkage_result) => {
                    let linkage_ids: Vec<Option<String>> = linkage_result
                        .0
                        .iter()
                        .map(|opt_id| opt_id.as_ref().map(|id| id.0.0.to_string()))
                        .collect();

                    let response = LinkageResponse {
                        status: "completed".to_string(),
                        linkage_ids: Some(linkage_ids),
                        message: Some("Linkage computation completed".to_string()),
                    };

                    // Send response to the waiting request
                    let _ = matching_request.response_sender.send(response.clone());
                    Ok(response)
                }
                Err(e) => {
                    let error_response = LinkageResponse {
                        status: "error".to_string(),
                        linkage_ids: None,
                        message: Some(format!("Linkage computation failed: {}", e)),
                    };

                    let _ = matching_request
                        .response_sender
                        .send(error_response.clone());
                    Err(LinkageServiceError::LinkageComputationFailed(e))
                }
            }
        } else {
            // No match found - store and wait
            let (tx, rx) = oneshot::channel();

            let pending = PendingRequest {
                source_party: source_party.clone(),
                destination_party: destination_party.clone(),
                data: request,
                response_sender: tx,
            };

            pending_requests.insert(current_key.clone(), pending);
            drop(pending_requests);

            // Setup cleanup task
            let service_clone = self.clone();
            let key_clone = current_key.clone();
            let timeout_duration = self.timeout_duration;

            tokio::spawn(async move {
                tokio::time::sleep(timeout_duration).await;
                if service_clone
                    .pending_requests
                    .write()
                    .await
                    .remove(&key_clone)
                    .is_some()
                {
                    warn!("Request {} timed out and was removed", key_clone);
                }
            });

            // Wait for matching request
            match tokio::time::timeout(timeout_duration, rx).await {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(_)) => Err(LinkageServiceError::InternalError(
                    "Channel error occurred".to_string(),
                )),
                Err(_) => {
                    // Timeout occurred, remove if still present
                    self.pending_requests.write().await.remove(&current_key);
                    Err(LinkageServiceError::Timeout)
                }
            }
        }
    }

    async fn perform_linkage(
        &self,
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
}

impl Default for LinkageService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkageServiceError {
    #[error("Invalid request pairing: {0}")]
    InvalidRequestPairing(String),
    #[error("Linkage computation failed: {0}")]
    LinkageComputationFailed(#[from] LinkageError),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Request timed out")]
    Timeout,
}
