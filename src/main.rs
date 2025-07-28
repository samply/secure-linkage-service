use secure_linkage_service::server::start_server;
use std::env;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init();

    // Get port from environment or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()
        .unwrap_or(5000);

    let address = format!("127.0.0.1:{}", port);

    println!("Starting secure linkage service on {}", address);

    start_server(&address).await
}
