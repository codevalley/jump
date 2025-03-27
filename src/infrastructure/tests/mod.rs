//! Test utilities for the infrastructure layer.

use chrono::{DateTime, Utc};

use crate::domain::payload::Payload;

/// A test utility to create a payload with specific content and expiry time.
pub fn create_test_payload(
    content: &str, 
    expiry_time: Option<DateTime<Utc>>
) -> Payload {
    Payload::new(
        content.to_string(),
        Some("text/plain".to_string()),
        expiry_time,
    ).unwrap()
}

/// Check if Redis is available for testing.
pub async fn is_redis_available() -> bool {
    use redis::{Client, RedisResult};
    
    match Client::open("redis://localhost:6379") {
        Ok(client) => {
            match client.get_async_connection().await {
                Ok(mut conn) => {
                    // Try a simple PING command
                    let ping_result: RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
                    ping_result.is_ok()
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
