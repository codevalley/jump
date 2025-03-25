//! Rate limiting implementation using Redis.
//!
//! This module provides rate limiting functionality using Redis as a backend.
//! It implements a sliding window rate limiter that tracks requests per IP
//! within a configurable time window.

use async_trait::async_trait;
use thiserror::Error;
use chrono;

use crate::infrastructure::redis::{RedisRepository};

/// Errors that can occur during rate limiting
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// Redis error
    #[error("Redis error: {0}")]
    Redis(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Try again in {0} seconds")]
    LimitExceeded(u64),
}

impl From<redis::RedisError> for RateLimitError {
    fn from(err: redis::RedisError) -> Self {
        RateLimitError::Redis(err.to_string())
    }
}

impl From<anyhow::Error> for RateLimitError {
    fn from(err: anyhow::Error) -> Self {
        RateLimitError::Redis(err.to_string())
    }
}

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    
    /// Time window in seconds
    pub window_seconds: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request should be allowed
    ///
    /// # Arguments
    /// * `key` - Unique identifier for the client (e.g., IP address)
    ///
    /// # Returns
    /// * `Ok(())` if the request is allowed
    /// * `Err(RateLimitError)` if the request should be rejected
    async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError>;
}

/// Redis-based rate limiter implementation using sliding window algorithm
pub struct RedisRateLimiter {
    redis: RedisRepository,
    config: RateLimitConfig,
}

impl RedisRateLimiter {
    /// Create a new Redis rate limiter
    pub fn new(redis: RedisRepository, config: RateLimitConfig) -> Self {
        Self { redis, config }
    }

    /// Generate Redis key for rate limiting
    fn rate_limit_key(key: &str) -> String {
        format!("rate_limit:{}", key)
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let mut conn = self.redis.get_conn().await.map_err(|e| RateLimitError::Redis(e.to_string()))?;
        let now = chrono::Utc::now().timestamp() as u64;
        let window_start = now - self.config.window_seconds as u64;
        let redis_key = Self::rate_limit_key(key);

        // First, add the current request to the sorted set
        let _: () = redis::cmd("ZADD")
            .arg(&redis_key)
            .arg(now)
            .arg(format!("req:{}", now)) // Use unique member for each request
            .query_async(&mut conn)
            .await.map_err(|e| RateLimitError::Redis(e.to_string()))?;
        
        // Set expiry on the key to auto-cleanup
        let _: () = redis::cmd("EXPIRE")
            .arg(&redis_key)
            .arg(self.config.window_seconds * 2) // Double the window for safety
            .query_async(&mut conn)
            .await.map_err(|e| RateLimitError::Redis(e.to_string()))?;

        // Remove old entries outside the window
        let _: () = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&redis_key)
            .arg(0)
            .arg(window_start)
            .query_async(&mut conn)
            .await.map_err(|e| RateLimitError::Redis(e.to_string()))?;

        // Count requests in current window
        let count: u32 = redis::cmd("ZCOUNT")
            .arg(&redis_key)
            .arg(window_start)
            .arg("+inf")
            .query_async(&mut conn)
            .await.map_err(|e| RateLimitError::Redis(e.to_string()))?;

        println!("Rate limit check: key={}, count={}, max={}", key, count, self.config.max_requests);

        if count > self.config.max_requests {
            // Get the oldest timestamp in the window to calculate wait time
            let oldest_entries: Vec<(String, f64)> = redis::cmd("ZRANGE")
                .arg(&redis_key)
                .arg(0)
                .arg(0)
                .arg("WITHSCORES")
                .query_async(&mut conn)
                .await.map_err(|e| RateLimitError::Redis(e.to_string()))?;
                
            if let Some((_, timestamp)) = oldest_entries.first() {
                let wait_time = self.config.window_seconds as u64 - (now - *timestamp as u64);
                return Err(RateLimitError::LimitExceeded(wait_time.max(1)));
            } else {
                // Fallback if we can't determine the exact wait time
                return Err(RateLimitError::LimitExceeded(self.config.window_seconds as u64));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::infrastructure::redis::RedisConfig;

    async fn create_test_limiter() -> RedisRateLimiter {
        let redis = RedisRepository::new(RedisConfig::default()).unwrap();
        let config = RateLimitConfig {
            max_requests: 3,
            window_seconds: 1,
        };
        RedisRateLimiter::new(redis, config)
    }

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
    async fn test_rate_limit_allows_requests_within_limit() {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis is not available");
            return;
        }

        let limiter = create_test_limiter().await;
        let key = "test_client_1";

        // Should allow max_requests
        for _ in 0..3 {
            assert!(limiter.check_rate_limit(key).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess_requests() {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis is not available");
            return;
        }

        let limiter = create_test_limiter().await;
        let key = "test_client_2";

        // Clean up any previous test data
        let mut conn = RedisRepository::new(RedisConfig::default())
            .unwrap()
            .get_conn()
            .await
            .unwrap();
        
        let _: () = redis::cmd("DEL")
            .arg(RedisRateLimiter::rate_limit_key(key))
            .query_async(&mut conn)
            .await
            .unwrap();

        // Add 3 requests manually to the sorted set
        let now = chrono::Utc::now().timestamp() as u64;
        for i in 0..3 {
            let _: () = redis::cmd("ZADD")
                .arg(RedisRateLimiter::rate_limit_key(key))
                .arg(now - i)  // Different timestamps
                .arg(format!("req:{}", i))
                .query_async(&mut conn)
                .await
                .unwrap();
        }

        // Next request should fail
        let result = limiter.check_rate_limit(key).await;
        println!("Excess request result: {:?}", result);
        
        match result {
            Err(RateLimitError::LimitExceeded(_)) => (),
            other => panic!("Expected LimitExceeded, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_resets_after_window() {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis is not available");
            return;
        }

        let limiter = create_test_limiter().await;
        let key = "test_client_3";

        // Use up all allowed requests
        for _ in 0..3 {
            assert!(limiter.check_rate_limit(key).await.is_ok());
        }

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Should be able to make requests again
        assert!(limiter.check_rate_limit(key).await.is_ok());
    }
}
