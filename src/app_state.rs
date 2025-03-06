use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    config::Config,
    party::Party,
    types::{CompleteComputation, Computation},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub parties: Arc<RwLock<HashMap<&'static str, Party>>>,
    pub computations: Arc<RwLock<HashMap<(String, String), Computation>>>,
    pub results: Arc<RwLock<HashMap<(String, String), CompleteComputation>>>,
}
