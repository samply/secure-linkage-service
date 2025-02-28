#![allow(unused)]
mod app_state;
pub mod config;
pub(crate) mod errors;
mod party;
pub mod routers;
pub mod types;


use std::fmt::Display;

pub use app_state::AppState;
pub use config::Config;
pub use party::Party;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all="lowercase")]
enum Role {
    Server,
    Client,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
