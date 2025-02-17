use crate::entities::*;
use bytes::Bytes;
use http::Response;

// Linkage

#[allow(unused_variables)]
pub fn process_share_data(data: LinkageShare) -> LinkageShare {
    todo!()
}

#[allow(unused_variables)]
pub fn is_result_ready(linkage_result: LinkageResult) -> Ready {
    todo!()
}

#[allow(unused_variables)]
pub fn compute_linkage_result(linkage_result: LinkageResult) -> LinkageResult {
    todo!()
}

#[allow(unused_variables)]
pub fn create_initial_state() -> LinkageState {
    todo!()
}

#[allow(unused_variables)]
pub fn generate_linkage_ids(epilinkers: Vec<EpiLinker>, count: Counter) -> Bytes {
    todo!()
}

#[allow(unused_variables)]
pub fn update_computation_state(
    linkage_result: LinkageResult,
    linkage_data: LinkageShare,
) -> LinkageState {
    todo!()
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
    timeout: Counter,
) -> Response<Bytes> {
    todo!()
}
