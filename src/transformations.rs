mod entities;

use bytes::Bytes;
use http::Response;

// Linkage

fn process_share_data(data: LinkageShare) -> LinkageShare;

fn is_result_ready(linkageResult: LinkageResult) -> bool;

fn compute_linkage_result(linkageResult: LinkageResult) -> LinkageResult;

fn create_initial_state() -> LinkageState;

fn generate_linkage_ids(epilinkers: Vect<EpiLinker>, count: uint) -> Bytes;

fn update_computation_state(
    linkageResult: LinkageResult,
    linkageData: LinkageShare,
) -> LinkageState;

// Handlers

fn homepage() -> Response;

fn check_connection(remote_id: str) -> Response;

fn get_ids(remote_id: str) -> Response;

fn linkage_result(
    linkageResult: LinkageResult,
    linkageData: LinkageShare,
    timeout: uint,
) -> Response;
