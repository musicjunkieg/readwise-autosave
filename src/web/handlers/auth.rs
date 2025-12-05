//! Authentication handlers for AT Protocol OAuth

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::AppState;

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Initiate OAuth login flow
pub async fn login(State(_state): State<Arc<AppState>>) -> Response {
    // TODO: Generate PKCE verifier and state
    // TODO: Build authorization URL using atproto-oauth
    // TODO: Store state in session
    // TODO: Redirect to Bluesky authorization endpoint

    // For now, return a placeholder
    Html(
        r#"<!DOCTYPE html>
<html>
<head><title>Login</title></head>
<body>
<h1>OAuth Login</h1>
<p>OAuth flow not yet implemented. Coming soon!</p>
<p><a href="/">Back to home</a></p>
</body>
</html>"#,
    )
    .into_response()
}

/// Handle OAuth callback
pub async fn callback(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<CallbackParams>,
) -> Response {
    // Check for errors from the OAuth provider
    if let Some(error) = params.error {
        let description = params.error_description.unwrap_or_default();
        return Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Login Error</title></head>
<body>
<h1>Login Failed</h1>
<p>Error: {} - {}</p>
<p><a href="/">Try again</a></p>
</body>
</html>"#,
            error, description
        ))
        .into_response();
    }

    // TODO: Verify state parameter
    // TODO: Exchange code for tokens using PKCE
    // TODO: Get user info (DID, handle)
    // TODO: Create or update user in database
    // TODO: Create session
    // TODO: Redirect to dashboard

    if let Some(_code) = params.code {
        // Placeholder success response
        Redirect::to("/dashboard").into_response()
    } else {
        Html(
            r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
<h1>Missing Authorization Code</h1>
<p><a href="/">Try again</a></p>
</body>
</html>"#,
        )
        .into_response()
    }
}

/// Handle logout
pub async fn logout(State(_state): State<Arc<AppState>>) -> Redirect {
    // TODO: Clear session
    // TODO: Revoke tokens if needed
    Redirect::to("/")
}
