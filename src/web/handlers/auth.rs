//! Authentication handlers for AT Protocol OAuth

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::AppState;

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub iss: Option<String>,
}

/// Form data for login
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub handle: String,
}

/// Session keys
const SESSION_USER_DID: &str = "user_did";
const SESSION_ACCESS_TOKEN: &str = "access_token";
const SESSION_REFRESH_TOKEN: &str = "refresh_token";

/// Show login form (GET /auth/login)
pub async fn login(State(_state): State<Arc<AppState>>) -> Response {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Login - Readwise Autosave</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 400px; margin: 2rem auto; padding: 1rem; }
        h1 { color: #1185fe; }
        form { margin-top: 1rem; }
        label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
        input[type="text"] {
            width: 100%; padding: 0.75rem; font-size: 1rem;
            border: 1px solid #ccc; border-radius: 6px; box-sizing: border-box;
        }
        input[type="text"]:focus { outline: 2px solid #1185fe; border-color: #1185fe; }
        .btn {
            display: block; width: 100%; background: #1185fe; color: white;
            padding: 0.75rem 1.5rem; font-size: 1rem; border: none;
            border-radius: 6px; margin-top: 1rem; cursor: pointer;
        }
        .btn:hover { background: #0066cc; }
        .help { color: #666; font-size: 0.875rem; margin-top: 0.5rem; }
        .error { color: #d32f2f; background: #ffebee; padding: 0.75rem; border-radius: 6px; margin-bottom: 1rem; }
    </style>
</head>
<body>
    <h1>🦋 Login with Bluesky</h1>
    <form method="POST" action="/auth/login">
        <label for="handle">Your Bluesky Handle</label>
        <input type="text" id="handle" name="handle" placeholder="username.bsky.social" required>
        <p class="help">Enter your full handle (e.g., alice.bsky.social)</p>
        <button type="submit" class="btn">Continue with Bluesky</button>
    </form>
    <p><a href="/">← Back to home</a></p>
</body>
</html>"#,
    )
    .into_response()
}

/// Handle login form submission (POST /auth/login)
pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> Response {
    let handle = form.handle.trim();

    // Validate handle format
    if handle.is_empty() {
        return show_login_error("Please enter your Bluesky handle").into_response();
    }

    // Initiate OAuth flow
    match state.oauth_service.initiate_login(handle).await {
        Ok(auth_url) => {
            tracing::info!("Redirecting user to authorization URL");
            Redirect::to(&auth_url).into_response()
        }
        Err(e) => {
            tracing::error!("OAuth initiation failed: {}", e);
            show_login_error(&format!("Failed to start login: {}", e)).into_response()
        }
    }
}

/// Handle OAuth callback (GET /auth/callback)
pub async fn callback(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> Response {
    // Check for errors from the OAuth provider
    if let Some(error) = params.error {
        let description = params.error_description.unwrap_or_default();
        tracing::error!("OAuth error: {} - {}", error, description);
        return show_callback_error(&error, &description).into_response();
    }

    // Get required parameters
    let code = match params.code {
        Some(c) => c,
        None => {
            return show_callback_error("Missing Code", "No authorization code received")
                .into_response()
        }
    };

    let oauth_state = match params.state {
        Some(s) => s,
        None => {
            return show_callback_error("Missing State", "No state parameter received")
                .into_response()
        }
    };

    // Complete the OAuth flow
    match state
        .oauth_service
        .complete_login(&oauth_state, &code)
        .await
    {
        Ok(token_response) => {
            // Get user DID from token response
            let user_did = token_response.sub.unwrap_or_default();

            if user_did.is_empty() {
                return show_callback_error("Invalid Response", "No user DID in token response")
                    .into_response();
            }

            // Store tokens in session
            if let Err(e) = session.insert(SESSION_USER_DID, &user_did).await {
                tracing::error!("Failed to store user DID in session: {}", e);
            }
            if let Err(e) = session
                .insert(SESSION_ACCESS_TOKEN, &token_response.access_token)
                .await
            {
                tracing::error!("Failed to store access token in session: {}", e);
            }
            if let Some(ref refresh_token) = token_response.refresh_token {
                if let Err(e) = session.insert(SESSION_REFRESH_TOKEN, refresh_token).await {
                    tracing::error!("Failed to store refresh token in session: {}", e);
                }
            }

            tracing::info!("User {} logged in successfully", user_did);

            // Redirect to dashboard
            Redirect::to("/dashboard").into_response()
        }
        Err(e) => {
            tracing::error!("OAuth completion failed: {}", e);
            show_callback_error("Login Failed", &e.to_string()).into_response()
        }
    }
}

/// Handle logout (POST /auth/logout)
pub async fn logout(session: Session) -> Redirect {
    // Clear session
    if let Err(e) = session.delete().await {
        tracing::error!("Failed to delete session: {}", e);
    }
    Redirect::to("/")
}

/// Show login error page
fn show_login_error(message: &str) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Login Error - Readwise Autosave</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 400px; margin: 2rem auto; padding: 1rem; }}
        h1 {{ color: #d32f2f; }}
        .error {{ color: #d32f2f; background: #ffebee; padding: 0.75rem; border-radius: 6px; }}
        .btn {{ display: inline-block; background: #1185fe; color: white; padding: 0.75rem 1.5rem;
                text-decoration: none; border-radius: 6px; margin-top: 1rem; }}
    </style>
</head>
<body>
    <h1>Login Error</h1>
    <div class="error">{}</div>
    <a href="/auth/login" class="btn">Try Again</a>
</body>
</html>"#,
        message
    ))
}

/// Show callback error page
fn show_callback_error(error: &str, description: &str) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Login Failed - Readwise Autosave</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 400px; margin: 2rem auto; padding: 1rem; }}
        h1 {{ color: #d32f2f; }}
        .error {{ background: #ffebee; padding: 1rem; border-radius: 6px; margin: 1rem 0; }}
        .error-title {{ color: #d32f2f; font-weight: bold; margin: 0 0 0.5rem 0; }}
        .error-desc {{ color: #666; margin: 0; }}
        .btn {{ display: inline-block; background: #1185fe; color: white; padding: 0.75rem 1.5rem;
                text-decoration: none; border-radius: 6px; }}
    </style>
</head>
<body>
    <h1>Login Failed</h1>
    <div class="error">
        <p class="error-title">{}</p>
        <p class="error-desc">{}</p>
    </div>
    <a href="/auth/login" class="btn">Try Again</a>
</body>
</html>"#,
        error, description
    ))
}
