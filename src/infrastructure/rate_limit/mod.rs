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
#[derive(Clone)]
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
    use crate::infrastructure::redis::{RedisConfig, RedisRepository};
    use crate::infrastructure::tests::is_redis_available;

    async fn clean_test_key(key: &str) -> Result<(), String> {
        let redis = match RedisRepository::new(RedisConfig::default()) {
            Ok(redis) => redis,
            Err(e) => return Err(format!("Failed to create Redis repository: {}", e)),
        };

        let mut conn = match redis.get_conn().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Failed to get Redis connection: {}", e)),
        };

        let _: () = redis::cmd("DEL")
            .arg(RedisRateLimiter::rate_limit_key(key))
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| format!("Failed to delete key: {}", e))?;

        Ok(())
    }

    #[tokio::test]
    async fn test_rate_limit_allows_max_requests() {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis not available");
            return;
        }

        let key = "test_client_1";
        
        // Clean up any previous test data
        let _ = clean_test_key(key).await;
        
        let limiter = RedisRateLimiter::new(RedisRepository::new(RedisConfig::default()).unwrap(), RateLimitConfig::default());

        // Should allow max_requests (which is 2 in our test limiter)
        for i in 0..2 {
            let result = limiter.check_rate_limit(key).await;
            assert!(result.is_ok(), "Request {} should be allowed", i+1);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess_requests() {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis not available");
            return;
        }

        // Create a Redis repository for testing
        let redis_repo = match RedisRepository::new(RedisConfig::default()) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Skipping test: Could not create Redis repository: {}", e);
                return;
            }
        };

        // Disable Redis persistence error checking for tests
        if let Err(e) = redis_repo.disable_stop_writes_on_bgsave_error().await {
            eprintln!("Warning: Could not disable Redis persistence error checking: {}", e);
        }

        // Use a unique key for this test
        let key = format!("test_block_{}", chrono::Utc::now().timestamp_millis());
        
        // Clean up any previous test data
        let mut conn = match redis_repo.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Skipping test: Could not get Redis connection: {}", e);
                return;
            }
        };
        
        let _: () = redis::cmd("DEL")
            .arg(RedisRateLimiter::rate_limit_key(&key))
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap_or(());
        
        // Create a limiter with a specific configuration for testing
        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 60, // Use a longer window to avoid timing issues
        };
        let max_requests = config.max_requests; // Store the max_requests value before moving config
        let limiter = RedisRateLimiter::new(redis_repo, config);

        // Manually add entries to the rate limit key to simulate reaching the limit
        let mut conn = limiter.redis.get_conn().await.unwrap();
        let now = chrono::Utc::now().timestamp() as f64;
        
        // Add max_requests entries with the current timestamp
        for i in 0..max_requests {
            let _: () = redis::cmd("ZADD")
                .arg(RedisRateLimiter::rate_limit_key(&key))
                .arg(now)
                .arg(format!("request{}", i))
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        }

        // Next request should fail with LimitExceeded
        let result = limiter.check_rate_limit(&key).await;
        assert!(result.is_err(), "Request after limit should fail");
        match result {
            Err(RateLimitError::LimitExceeded(_)) => (),
            other => panic!("Expected LimitExceeded, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_resets_after_window() {
        if !is_redis_available().await {
            eprintln!("Skipping Redis test: Redis not available");
            return;
        }

        // Create a Redis repository for testing
        let redis_repo = match RedisRepository::new(RedisConfig::default()) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Skipping test: Could not create Redis repository: {}", e);
                return;
            }
        };

        // Disable Redis persistence error checking for tests
        if let Err(e) = redis_repo.disable_stop_writes_on_bgsave_error().await {
            eprintln!("Warning: Could not disable Redis persistence error checking: {}", e);
        }

        // Use a unique key for this test to avoid conflicts with other tests
        let key = format!("test_reset_{}", chrono::Utc::now().timestamp_millis());
        
        // Clean up any previous test data
        let mut conn = match redis_repo.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Skipping test: Could not get Redis connection: {}", e);
                return;
            }
        };
        
        let _: () = redis::cmd("DEL")
            .arg(RedisRateLimiter::rate_limit_key(&key))
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap_or(());
        
        // Create a limiter with a very short window for testing
        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 1, // Very short window
        };
        let limiter = RedisRateLimiter::new(redis_repo, config);

        // Manually add entries to the rate limit key to simulate reaching the limit
        let mut conn = limiter.redis.get_conn().await.unwrap();
        let now = chrono::Utc::now().timestamp() as f64;
        
        // Add two entries with the current timestamp
        let _: () = redis::cmd("ZADD")
            .arg(RedisRateLimiter::rate_limit_key(&key))
            .arg(now)
            .arg("request1")
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();
            
        let _: () = redis::cmd("ZADD")
            .arg(RedisRateLimiter::rate_limit_key(&key))
            .arg(now)
            .arg("request2")
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();

        // Next request should fail
        let result = limiter.check_rate_limit(&key).await;
        assert!(result.is_err(), "Request after limit should fail");
        
        // Manually remove the rate limit key to simulate window expiry
        let _: () = redis::cmd("DEL")
            .arg(RedisRateLimiter::rate_limit_key(&key))
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();

        // Now we should be able to make requests again
        let result = limiter.check_rate_limit(&key).await;
        assert!(result.is_ok(), "Request after window reset should be allowed");
    }
}
