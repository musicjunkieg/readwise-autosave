//! Database queries
//!
//! TODO: Implement database operations

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;

/// Database operations
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get a user by their Bluesky DID
    pub async fn get_user_by_did(&self, did: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE bluesky_did = $1")
            .bind(did)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// Get user settings
    pub async fn get_user_settings(&self, user_id: Uuid) -> Result<Option<UserSettings>> {
        let settings =
            sqlx::query_as::<_, UserSettings>("SELECT * FROM user_settings WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(settings)
    }

    /// Check if a bookmark has been processed
    pub async fn is_bookmark_processed(&self, user_id: Uuid, post_uri: &str) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM processed_bookmarks WHERE user_id = $1 AND post_uri = $2)",
        )
        .bind(user_id)
        .bind(post_uri)
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }
}
