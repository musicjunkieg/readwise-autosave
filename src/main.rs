use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod bluesky;
mod config;
mod content;
mod db;
mod readwise;
mod services;
mod web;

/// Shared application state
pub struct AppState {
    pub config: config::Config,
    // TODO: Add database pool
    // TODO: Add OAuth client
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "readwise_autosave=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting readwise-autosave");

    // Load configuration
    let config = config::Config::load()?;
    tracing::info!("Configuration loaded");

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
    });

    // Create router with state
    let app = web::routes::create_router(state).layer(TraceLayer::new_for_http());

    // Start the server
    let addr = &config.server_address;
    tracing::info!("Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
