use crate::config::Config;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: &'static Config,
}
