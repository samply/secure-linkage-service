use secure_linkage_service::server::start_server;
use std::env;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init();

    // Get host and port from environment or use defaults
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()
        .unwrap_or(5000);

    let address = format!("{}:{}", host, port);

    println!("Starting secure linkage service on {}", address);

    start_server(&address).await
}
