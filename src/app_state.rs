use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{config::Config, party::Party};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub parties: Arc<RwLock<HashMap<&'static str, Party>>>,
}
