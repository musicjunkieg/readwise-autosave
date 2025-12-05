//! Route definitions

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use super::handlers;
use crate::AppState;

/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Public routes
        .route("/", get(handlers::index))
        .route("/health", get(handlers::health))
        // Auth routes
        .route(
            "/auth/login",
            get(handlers::auth::login).post(handlers::auth::login_submit),
        )
        .route("/auth/callback", get(handlers::auth::callback))
        .route("/auth/logout", post(handlers::auth::logout))
        // Dashboard routes
        .route("/dashboard", get(handlers::dashboard::settings))
        .route("/api/settings", post(handlers::api::update_settings))
        // Share state with all routes
        .with_state(state)
}
