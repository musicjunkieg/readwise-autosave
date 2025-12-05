//! Dashboard handlers for user settings

use std::sync::Arc;

use axum::{extract::State, response::Html};

use crate::AppState;

/// User settings dashboard
pub async fn settings(State(_state): State<Arc<AppState>>) -> Html<String> {
    // TODO: Get user from session
    // TODO: Fetch user settings from database
    // TODO: Render actual settings form

    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Settings - Readwise Autosave</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 600px; margin: 2rem auto; padding: 1rem; }
        h1 { color: #1185fe; }
        .form-group { margin: 1.5rem 0; }
        label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
        input[type="text"], input[type="password"] {
            width: 100%; padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px;
        }
        .checkbox-group { display: flex; align-items: center; gap: 0.5rem; }
        .btn { background: #1185fe; color: white; padding: 0.75rem 1.5rem;
               border: none; border-radius: 6px; cursor: pointer; }
        .btn:hover { background: #0066cc; }
        .btn-danger { background: #dc3545; }
        .btn-danger:hover { background: #c82333; }
        .status { padding: 1rem; background: #e8f4fd; border-radius: 6px; margin-bottom: 1rem; }
        .nav { margin-bottom: 2rem; }
        .nav a { color: #1185fe; }
    </style>
</head>
<body>
    <div class="nav">
        <a href="/">← Back to Home</a> |
        <form action="/auth/logout" method="POST" style="display: inline;">
            <button type="submit" style="background: none; border: none; color: #dc3545; cursor: pointer;">Logout</button>
        </form>
    </div>

    <h1>⚙️ Settings</h1>

    <div class="status">
        <strong>Status:</strong> Not connected<br>
        <small>Connect with Bluesky to enable bookmark sync.</small>
    </div>

    <form action="/api/settings" method="POST">
        <div class="form-group">
            <label for="readwise_token">Readwise Access Token</label>
            <input type="password" id="readwise_token" name="readwise_token"
                   placeholder="Get from readwise.io/access_token" required>
            <small>Get your token at <a href="https://readwise.io/access_token" target="_blank">readwise.io/access_token</a></small>
        </div>

        <div class="form-group">
            <div class="checkbox-group">
                <input type="checkbox" id="bookmark_sync" name="bookmark_sync" checked>
                <label for="bookmark_sync" style="margin-bottom: 0;">Enable bookmark sync</label>
            </div>
            <small>Automatically save bookmarked posts to Readwise</small>
        </div>

        <div class="form-group">
            <div class="checkbox-group">
                <input type="checkbox" id="extract_links" name="extract_links">
                <label for="extract_links" style="margin-bottom: 0;">Extract links from posts</label>
            </div>
            <small>Also save URLs found in bookmarked posts to Readwise Reader</small>
        </div>

        <div class="form-group">
            <button type="submit" class="btn">Save Settings</button>
        </div>
    </form>
</body>
</html>"#
            .to_string(),
    )
}
