//! HTTP handlers

pub mod api;
pub mod auth;
pub mod dashboard;

use axum::response::Html;

/// Landing page
pub async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Readwise Autosave</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 600px; margin: 2rem auto; padding: 1rem; }
        h1 { color: #1185fe; }
        .btn { display: inline-block; background: #1185fe; color: white; padding: 0.75rem 1.5rem;
               text-decoration: none; border-radius: 6px; margin-top: 1rem; }
        .btn:hover { background: #0066cc; }
    </style>
</head>
<body>
    <h1>📚 Readwise Autosave</h1>
    <p>Automatically save your Bluesky bookmarks to Readwise.</p>
    <ul>
        <li>Bookmark a post → saves to Readwise Highlights</li>
        <li>Bookmark a thread → saves to Readwise Reader</li>
        <li>DM posts to the bot for quick saving with notes</li>
    </ul>
    <a href="/auth/login" class="btn">Connect with Bluesky</a>
</body>
</html>"#,
    )
}

/// Health check endpoint
pub async fn health() -> &'static str {
    "ok"
}
