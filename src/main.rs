// Allow unused code during scaffolding phase
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use atproto_identity::key::{generate_key, KeyType};
use atproto_identity::resolve::{
    HickoryDnsResolver, InnerIdentityResolver, SharedIdentityResolver,
};
use atproto_oauth::workflow::OAuthClient;

mod bluesky;
mod config;
mod content;
mod db;
mod readwise;
mod services;
mod web;

use bluesky::oauth::{OAuthService, OAuthStateStore};

/// Shared application state
pub struct AppState {
    pub config: config::Config,
    pub oauth_service: OAuthService,
    // TODO: Add database pool
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

    // Create HTTP client for OAuth operations
    let http_client = reqwest::Client::new();

    // Create identity resolver
    let dns_resolver = Arc::new(HickoryDnsResolver::create_resolver(&[]));
    let identity_resolver = SharedIdentityResolver(Arc::new(InnerIdentityResolver {
        dns_resolver,
        http_client: http_client.clone(),
        plc_hostname: "plc.directory".to_string(),
    }));

    // Generate signing key for OAuth client (should be persisted in production)
    let signing_key =
        generate_key(KeyType::P256Private).context("Failed to generate OAuth signing key")?;

    // Create OAuth client configuration
    let oauth_client = OAuthClient {
        client_id: config
            .oauth_client_id
            .clone()
            .unwrap_or_else(|| "https://example.com/oauth/client-metadata.json".to_string()),
        redirect_uri: config
            .oauth_redirect_uri
            .clone()
            .unwrap_or_else(|| format!("http://{}/auth/callback", config.server_address)),
        private_signing_key_data: signing_key,
    };

    // Create OAuth state store
    let oauth_state_store = Arc::new(OAuthStateStore::new());

    // Create OAuth service
    let oauth_service = OAuthService::new(
        http_client,
        oauth_client,
        identity_resolver,
        oauth_state_store,
    );

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        oauth_service,
    });

    // Create session store
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    // Create router with state and session layer
    let app = web::routes::create_router(state)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http());

    // Start the server
    let addr = &config.server_address;
    tracing::info!("Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
