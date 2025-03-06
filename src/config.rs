use clap::Parser;
use reqwest::{header::HeaderValue, Certificate, Client};
use tracing::{debug, info, warn};

use crate::errors::SLSError;
use std::{net::SocketAddr, path::PathBuf};

/// Specifies the log's output format
#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum LogFormat {
    Text,
    TextColor,
    Json,
}

/// Specifies how much log output the app generates
#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn tracing_level(&self) -> tracing::Level {
        use tracing::Level;

        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

const CLAP_FOOTER: &str = "For proxy support, environment variables HTTP_PROXY, HTTPS_PROXY, ALL_PROXY and NO_PROXY (and their lower-case variants) are supported. Usually, you want to set HTTP_PROXY *and* HTTPS_PROXY or set ALL_PROXY if both values are the same.\n\nFor updates and detailed usage instructions, visit https://github.com/samply/securelinkageservice";

#[derive(Clone, Debug, clap::Parser)]
#[clap(
    name("🔗 Secure LinkageService"),
    about,
    version,
    propagate_version = true,
    after_help(CLAP_FOOTER)
)]
struct CliArgs {
    /// The Socket address the server should listen on
    #[clap(long, env = "LISTEN", default_value = "[::]:9081")]
    listen: SocketAddr,

    /// Defines how the log output will be formatted
    #[clap(value_enum, long, env = "LOG_FORMAT", default_value_t = LogFormat::TextColor)]
    log_format: LogFormat,

    /// Defines how noisy the server should be
    #[clap(value_enum, long, env = "LOG_LEVEL", default_value_t = LogLevel::Info)]
    log_level: LogLevel,

    /// Number of retries for reaching downstream services
    #[clap(long, env, value_parser, default_value = "32")]
    retry_count: usize,

    /// Outgoing HTTP proxy: Directory with CA certificates to trust for TLS connections (e.g. /etc/samply/cacerts/)
    #[clap(long, env, value_parser)]
    tls_ca_certificates_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub log_format: LogFormat,
    pub log_level: LogLevel,
    pub retry_count: usize,
    pub client: Client,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        debug!("Loading config");
        Config::load().unwrap_or_else(|e| {
            eprintln!("Unable to start as there was an error reading the config:\n{}\n\nTerminating -- please double-check your startup parameters with --help and refer to the documentation.", e);
            std::process::exit(1);
        })
    }

    fn load() -> Result<Self, SLSError> {
        let cli_args = CliArgs::parse();
        info!("Successfully read config and API keys from CLI and secrets files.");
        let tls_ca_certificates_dir = cli_args.tls_ca_certificates_dir;
        let tls_ca_certificates = load_certificates_from_dir(tls_ca_certificates_dir.clone())
            .map_err(|e| {
                SLSError::ConfigurationError(format!("Unable to read from TLS CA directory: {}", e))
            })?;
        let client = prepare_reqwest_client(&tls_ca_certificates)?;
        let config = Config {
            listen: cli_args.listen,
            log_format: cli_args.log_format,
            log_level: cli_args.log_level,
            retry_count: cli_args.retry_count,
            client,
        };
        Ok(config)
    }
}

pub fn load_certificates_from_dir(
    ca_dir: Option<PathBuf>,
) -> Result<Vec<Certificate>, std::io::Error> {
    let mut result = Vec::new();
    if let Some(ca_dir) = ca_dir {
        for file in ca_dir.read_dir()? {
            //.map_err(|e| SamplyBeamError::ConfigurationFailed(format!("Unable to read from TLS CA directory {}: {}", ca_dir.to_string_lossy(), e)))
            let path = file?.path();
            let content = std::fs::read(&path)?;
            let cert = Certificate::from_pem(&content);
            if let Err(e) = cert {
                warn!(
                    "Unable to read certificate from file {}: {}",
                    path.to_string_lossy(),
                    e
                );
                continue;
            }
            result.push(cert.unwrap());
        }
    }
    Ok(result)
}

pub fn prepare_reqwest_client(certs: &Vec<Certificate>) -> Result<reqwest::Client, SLSError> {
    let mut client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .user_agent(HeaderValue::from_static(env!("SAMPLY_USER_AGENT")));
    for cert in certs {
        client = client.add_root_certificate(cert.to_owned());
    }
    client
        .build()
        .map_err(|e| SLSError::ConfigurationError(format!("Cannot create http client: {}", e)))
}
