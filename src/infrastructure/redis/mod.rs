//! Redis infrastructure implementation.
//!
//! This module provides Redis connection pooling and basic operations
//! for storing and retrieving payloads.

use deadpool_redis::{Config, Pool, Runtime};
use redis;
use serde_json;
use thiserror::Error;
use async_trait::async_trait;

use crate::{
    application::repository::Repository,
    domain::{hash_id::HashId, payload::Payload},
};

/// Redis configuration
#[derive(Clone, Debug)]
pub struct RedisConfig {
    /// Redis URL (e.g. "redis://localhost:6379")
    pub url: String,
    /// Maximum number of connections in the pool
    pub pool_max_size: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_max_size: 16,
            connection_timeout: 5,
        }
    }
}

/// Redis errors
#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Failed to create Redis pool: {0}")]
    PoolCreation(String),
    #[error("Failed to get Redis connection: {0}")]
    Connection(String),
    #[error("Redis operation failed: {0}")]
    Operation(String),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Redis repository for storing and retrieving payloads
#[derive(Clone)]
pub struct RedisRepository {
    pool: Pool,
}

impl RedisRepository {
    /// Create a new Redis repository
    pub fn new(config: RedisConfig) -> Result<Self, RedisError> {
        let cfg = Config::from_url(config.url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| RedisError::PoolCreation(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get a Redis connection from the pool
    pub async fn get_conn(&self) -> Result<deadpool_redis::Connection, RedisError> {
        self.pool
            .get()
            .await
            .map_err(|e| RedisError::Connection(e.to_string()))
    }

    /// Generate Redis key for a payload
    fn payload_key(hash_id: &HashId) -> String {
        format!("payload:{}", hash_id.as_string())
    }
}

#[async_trait]
impl Repository for RedisRepository {
    async fn save(&self, payload: &Payload) -> Result<(), anyhow::Error> {
        let mut conn = self.get_conn().await?;
        let key = Self::payload_key(payload.hash_id());
        let json = serde_json::to_string(payload)?;

        // Calculate expiry duration
        let expiry = payload.expiry_time().timestamp() - chrono::Utc::now().timestamp();
        
        // Only save if not expired
        if expiry > 0 {
            let _: () = redis::cmd("SET")
                .arg(&key)
                .arg(&json)
                .arg("EX")
                .arg(expiry as usize)
                .query_async(&mut conn)
                .await?;
        }

        Ok(())
    }

    async fn get(&self, hash_id: &HashId) -> Result<Option<Payload>, anyhow::Error> {
        let mut conn = self.get_conn().await?;
        let key = Self::payload_key(hash_id);

        let json: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;

        match json {
            Some(json) => {
                let payload: Payload = serde_json::from_str(&json)?;
                Ok(Some(payload))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, hash_id: &HashId) -> Result<(), anyhow::Error> {
        let mut conn = self.get_conn().await?;
        let key = Self::payload_key(hash_id);
        
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    async fn is_redis_available() -> bool {
        match RedisRepository::new(RedisConfig::default()) {
            Ok(repo) => {
                let conn_result = repo.get_conn().await;
                if conn_result.is_err() {
                    return false;
                }
                
                // Try a simple ping command to verify Redis is working properly
                let mut conn = conn_result.unwrap();
                let ping_result: Result<String, redis::RedisError> = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await;
                
                ping_result.is_ok()
            },
            Err(_) => false,
        }
    }

    #[tokio::test]
    async fn test_redis_save_and_get() -> Result<(), anyhow::Error> {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis is not available");
            return Ok(());
        }

        let repo = RedisRepository::new(RedisConfig::default())?;
        let payload = Payload::new(
            "test content".to_string(),
            Some("text/plain".to_string()),
            Some(Utc::now() + Duration::hours(1)),
        )?;

        // Save payload
        repo.save(&payload).await?;

        // Get payload
        let retrieved = repo.get(payload.hash_id()).await?.unwrap();
        assert_eq!(retrieved.content(), payload.content());
        assert_eq!(retrieved.mime_type().to_string(), "text/plain");

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_delete() -> Result<(), anyhow::Error> {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis is not available");
            return Ok(());
        }

        let repo = RedisRepository::new(RedisConfig::default())?;
        let payload = Payload::new("test content".to_string(), None, None)?;

        // Save and verify
        repo.save(&payload).await?;
        assert!(repo.get(payload.hash_id()).await?.is_some());

        // Delete and verify
        repo.delete(payload.hash_id()).await?;
        assert!(repo.get(payload.hash_id()).await?.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_expired_payload() -> Result<(), anyhow::Error> {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis is not available");
            return Ok(());
        }

        let repo = RedisRepository::new(RedisConfig::default())?;
        let payload = Payload::new(
            "test content".to_string(),
            None,
            Some(Utc::now() + Duration::seconds(1)),
        )?;

        // Save payload
        repo.save(&payload).await?;

        // Wait for expiry
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Should be gone
        assert!(repo.get(payload.hash_id()).await?.is_none());

        Ok(())
    }
}
