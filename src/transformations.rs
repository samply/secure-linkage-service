use crate::domain::*;
use bytes::Bytes;
use http::Response;

// Linkage

#[allow(unused_variables)]
pub fn compute_linkage(client: EpiLinker, server: EpiLinker, timeout: TimeOut) -> LinkageResult {
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
    timeout: TimeOut,
) -> Response<Bytes> {
    todo!()
}
