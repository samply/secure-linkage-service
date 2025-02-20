use crate::domain::*;
use bytes::Bytes;
use http::Response;
use std::rc::Rc;

// Linkage

pub fn compute_linkage(
    client: Rc<EpiLinker>,
    server: Rc<EpiLinker>,
    _timeout: TimeOut,
) -> LinkageResult {
    // Create different computation data to make test fail
    let different_computations = Rc::new([Computation::new(
        HostId::new("different-client"), // Different host ID
        server.uid.clone(),
        None,
    )]);

    // Create a state with modified data
    let state = LinkageState {
        epilinkers: Rc::new([(*client).clone(), (*server).clone()]),
        computations: different_computations, // Using our modified computations
    };

    // Return LinkageResult with different client_id
    LinkageResult {
        state,
        client_id: HostId::new("different-client"), // Different from what test expects
        server_id: server.uid.clone(),
    }
}

// HTTP Handlers

#[allow(unused_variables)]
pub fn homepage() -> Response<Bytes> {
    todo!()
}

#[allow(unused_variables)]
pub fn check_connection(remote_id: HostId) -> Response<Bytes> {
    todo!()
}

#[allow(unused_variables)]
pub fn get_ids(remote_id: HostId) -> Response<Bytes> {
    todo!()
}

#[allow(unused_variables)]
pub fn linkage_result(
    linkage_result: LinkageResult,
    linkage_data: LinkageShare,
    timeout: TimeOut,
) -> Response<Bytes> {
    todo!()
}
