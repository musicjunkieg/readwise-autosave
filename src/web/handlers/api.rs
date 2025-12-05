//! API handlers for settings and other operations

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;

use crate::AppState;

/// Form data for updating settings
#[derive(Debug, Deserialize)]
pub struct SettingsForm {
    pub readwise_token: String,
    #[serde(default)]
    pub bookmark_sync: bool,
    #[serde(default)]
    pub extract_links: bool,
}

/// Update user settings
pub async fn update_settings(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<SettingsForm>,
) -> Response {
    // TODO: Get user from session
    // TODO: Validate Readwise token by making a test API call
    // TODO: Update settings in database

    tracing::info!(
        "Settings update requested: bookmark_sync={}, extract_links={}",
        form.bookmark_sync,
        form.extract_links
    );

    // Validate that token is not empty
    if form.readwise_token.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Readwise token is required").into_response();
    }

    // TODO: Actually save settings

    // Redirect back to dashboard with success message
    Redirect::to("/dashboard?saved=true").into_response()
}
