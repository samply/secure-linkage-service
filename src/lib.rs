mod app_state;
pub mod config;
pub(crate) mod errors;
mod linkage_result;
mod party;
pub mod routers;

pub use app_state::AppState;
pub use config::Config;
pub use party::Party;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
enum Role {
    Server,
    Client,
}
