use anyhow::{Context, Result};
use serde::Deserialize;

/// Application configuration loaded from environment and config files
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server address to bind to (e.g., "0.0.0.0:3000")
    #[serde(default = "default_server_address")]
    pub server_address: String,

    /// PostgreSQL database URL
    pub database_url: String,

    /// Bluesky bot account handle for DM functionality
    pub bluesky_bot_handle: Option<String>,

    /// Bluesky bot account app password
    pub bluesky_bot_password: Option<String>,

    /// OAuth client ID (HTTPS URL pointing to client metadata)
    pub oauth_client_id: Option<String>,

    /// OAuth redirect URI
    pub oauth_redirect_uri: Option<String>,

    /// Bookmark polling interval in seconds
    #[serde(default = "default_bookmark_poll_interval")]
    pub bookmark_poll_interval_secs: u64,

    /// DM polling interval in seconds
    #[serde(default = "default_dm_poll_interval")]
    pub dm_poll_interval_secs: u64,
}

fn default_server_address() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_bookmark_poll_interval() -> u64 {
    30
}

fn default_dm_poll_interval() -> u64 {
    10
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            // Start with default values
            .set_default("server_address", "0.0.0.0:3000")?
            .set_default("bookmark_poll_interval_secs", 30)?
            .set_default("dm_poll_interval_secs", 10)?
            // Add config file if it exists
            .add_source(config::File::with_name("config").required(false))
            // Override with environment variables (prefixed with APP_)
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("_")
                    .try_parsing(true),
            )
            // Also check for DATABASE_URL directly (common convention)
            .add_source(config::Environment::default().try_parsing(true))
            .build()
            .context("Failed to build configuration")?;

        config
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_server_address(), "0.0.0.0:3000");
        assert_eq!(default_bookmark_poll_interval(), 30);
        assert_eq!(default_dm_poll_interval(), 10);
    }
}
