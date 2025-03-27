//! Tests for the Redis repository implementation.

use chrono::{Duration, Utc};

use crate::application::repository::Repository;
use crate::domain::hash_id::HashId;
use crate::infrastructure::redis::{RedisConfig, RedisRepository};
use super::{create_test_payload, is_redis_available};

/// Test saving and retrieving a payload from Redis.
#[tokio::test]
async fn test_redis_save_and_get() -> Result<(), anyhow::Error> {
    // Skip test if Redis is not available
    if !is_redis_available().await {
        println!("Skipping Redis test: Redis not available");
        return Ok(());
    }

    // Arrange
    let config = RedisConfig::default();
    let repository = RedisRepository::new(config)?;
    
    // Disable persistence error checking for tests
    let _ = repository.disable_stop_writes_on_bgsave_error().await;
    
    let content = "Test content for Redis";
    let payload = create_test_payload(content, None);
    let hash_id = payload.hash_id().clone();
    
    // Act - Save
    repository.save(&payload).await?;
    
    // Act - Get
    let retrieved = repository.get(&hash_id).await?;
    
    // Assert
    assert!(retrieved.is_some(), "Expected to retrieve the payload");
    let retrieved_payload = retrieved.unwrap();
    assert_eq!(
        retrieved_payload.content(), 
        content, 
        "Retrieved content should match original"
    );
    assert_eq!(
        retrieved_payload.hash_id().as_string(), 
        hash_id.as_string(), 
        "Retrieved hash ID should match original"
    );
    
    Ok(())
}

/// Test deleting a payload from Redis.
#[tokio::test]
async fn test_redis_delete() -> Result<(), anyhow::Error> {
    // Skip test if Redis is not available
    if !is_redis_available().await {
        println!("Skipping Redis test: Redis not available");
        return Ok(());
    }

    // Arrange
    let config = RedisConfig::default();
    let repository = RedisRepository::new(config)?;
    
    // Disable persistence error checking for tests
    let _ = repository.disable_stop_writes_on_bgsave_error().await;
    
    let payload = create_test_payload("Content to be deleted", None);
    let hash_id = payload.hash_id().clone();
    
    // Save the payload first
    repository.save(&payload).await?;
    
    // Verify it exists
    let before_delete = repository.get(&hash_id).await?;
    assert!(before_delete.is_some(), "Payload should exist before deletion");
    
    // Act - Delete
    repository.delete(&hash_id).await?;
    
    // Assert
    let after_delete = repository.get(&hash_id).await?;
    assert!(after_delete.is_none(), "Payload should not exist after deletion");
    
    Ok(())
}

/// Test handling of expired payloads.
#[tokio::test]
async fn test_expired_payload() -> Result<(), anyhow::Error> {
    // Skip test if Redis is not available
    if !is_redis_available().await {
        println!("Skipping Redis test: Redis not available");
        return Ok(());
    }

    // Arrange
    let config = RedisConfig::default();
    let repository = RedisRepository::new(config)?;
    
    // Disable persistence error checking for tests
    let _ = repository.disable_stop_writes_on_bgsave_error().await;
    
    // Create a payload that expires in 2 seconds
    let expiry_time = Utc::now() + Duration::seconds(2);
    let payload = create_test_payload("Expiring content", Some(expiry_time));
    let hash_id = payload.hash_id().clone();
    
    // Save the payload
    repository.save(&payload).await?;
    
    // Verify it exists immediately
    let before_expiry = repository.get(&hash_id).await?;
    assert!(before_expiry.is_some(), "Payload should exist before expiry");
    
    // Wait for expiration (3 seconds to be safe)
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Act - Try to get after expiry
    let after_expiry = repository.get(&hash_id).await?;
    
    // Assert
    assert!(after_expiry.is_none(), "Payload should not exist after expiry");
    
    Ok(())
}

/// Test handling of multiple payloads.
#[tokio::test]
async fn test_multiple_payloads() -> Result<(), anyhow::Error> {
    // Skip test if Redis is not available
    if !is_redis_available().await {
        println!("Skipping Redis test: Redis not available");
        return Ok(());
    }

    // Arrange
    let config = RedisConfig::default();
    let repository = RedisRepository::new(config)?;
    
    // Disable persistence error checking for tests
    let _ = repository.disable_stop_writes_on_bgsave_error().await;
    
    // Create multiple payloads
    let payload1 = create_test_payload("Content 1", None);
    let hash_id1 = payload1.hash_id().clone();
    
    let payload2 = create_test_payload("Content 2", None);
    let hash_id2 = payload2.hash_id().clone();
    
    let payload3 = create_test_payload("Content 3", None);
    let hash_id3 = payload3.hash_id().clone();
    
    // Save all payloads
    repository.save(&payload1).await?;
    repository.save(&payload2).await?;
    repository.save(&payload3).await?;
    
    // Act - Get each payload
    let retrieved1 = repository.get(&hash_id1).await?;
    let retrieved2 = repository.get(&hash_id2).await?;
    let retrieved3 = repository.get(&hash_id3).await?;
    
    // Assert
    assert!(retrieved1.is_some(), "Payload 1 should exist");
    assert!(retrieved2.is_some(), "Payload 2 should exist");
    assert!(retrieved3.is_some(), "Payload 3 should exist");
    
    assert_eq!(
        retrieved1.unwrap().content(), 
        "Content 1", 
        "Retrieved content 1 should match original"
    );
    assert_eq!(
        retrieved2.unwrap().content(), 
        "Content 2", 
        "Retrieved content 2 should match original"
    );
    assert_eq!(
        retrieved3.unwrap().content(), 
        "Content 3", 
        "Retrieved content 3 should match original"
    );
    
    Ok(())
}
