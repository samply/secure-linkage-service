use thiserror::Error;

#[derive(Error, Debug)]
pub enum SLSError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Unable to set proxy settings: {0}")]
    InvalidProxyConfig(reqwest::Error),
}
