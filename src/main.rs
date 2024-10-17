use anyhow::Context;
use tokio::net::TcpListener;
use tracing::info;

use securelinkageservice::{config::LogFormat, routers::build_main_router, AppState, Config};
use tracing_subscriber::EnvFilter;
mod banner;

/// Sets up a relevant shutdown signals. This will exit on either SIGINT or SIGTERM.
async fn shutdown_signal() {
    let sigint = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to create Ctrl+C handler");
    };

    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to create SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = sigint => {},
        () = sigterm => {},
    }

    info!("shutdown signal received")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = Config::new();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(config.log_level.tracing_level())
        .with_target(false);
    match config.log_format {
        LogFormat::Text => subscriber.with_ansi(false).init(),
        LogFormat::TextColor => subscriber.with_ansi(true).init(),
        LogFormat::Json => subscriber.json().with_span_list(false).init(),
    }
    banner::print_banner();

    let listener = TcpListener::bind(config.listen)
        .await
        .context(format!("could not listen to `{}`", config.listen))?;

    info!("starting server on `{}`", config.listen);
    let router = build_main_router(AppState {
        config: Box::leak(Box::new(config)),
    });
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("failed to start server")?;

    Ok(())
}
