mod app_state;
pub mod config;
pub(crate) mod errors;
pub mod routers;

pub use app_state::AppState;
pub use config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
enum Role {
    Server,
    Client,
}
