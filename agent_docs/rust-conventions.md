# Rust Conventions

## Error Handling

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bluesky API error: {0}")]
    BlueskyApi(String),

    #[error("Readwise API error: {0}")]
    ReadwiseApi(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

Use `anyhow` for application-level error propagation.

## Async Patterns

```rust
use tokio::spawn;

// Background task
let handle = spawn(async move {
    loop {
        poll_bookmarks().await;
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});
```

## Trait-Based Abstractions

For testability, define traits for external services:

```rust
#[async_trait]
pub trait BlueskyClient: Send + Sync {
    async fn get_bookmarks(&self, cursor: Option<&str>) -> Result<BookmarkResponse>;
    async fn get_post_thread(&self, uri: &str) -> Result<ThreadResponse>;
}
```

## No Unwrap in Production

```rust
// Bad
let value = some_option.unwrap();

// Good
let value = some_option.ok_or_else(|| AppError::MissingValue)?;

// Or with context
let value = some_option.context("Expected value to be present")?;
```

## Module Organization

```rust
// mod.rs - public interface
pub mod client;
pub mod types;

pub use client::*;
pub use types::*;
```

## Configuration

Use `config` crate with environment overrides:

```rust
#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub bluesky_bot_handle: String,
    // ...
}
```

---

## Learnings

(Append patterns here as we implement)
