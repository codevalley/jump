//! Test utilities for the application layer.

use std::collections::HashMap;
use std::sync::Mutex;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::hash_id::HashId;
use crate::domain::payload::Payload;
use crate::application::repository::Repository;

/// A mock repository implementation for testing.
pub struct MockRepository {
    payloads: Mutex<HashMap<String, Payload>>,
}

impl MockRepository {
    /// Create a new mock repository.
    pub fn new() -> Self {
        Self {
            payloads: Mutex::new(HashMap::new()),
        }
    }
    
    /// Add a payload to the repository for testing.
    pub fn add_payload(&self, payload: Payload) {
        let hash_id = payload.hash_id().as_string().to_string();
        self.payloads.lock().unwrap().insert(hash_id, payload);
    }
    
    /// Get the number of payloads in the repository.
    pub fn count(&self) -> usize {
        self.payloads.lock().unwrap().len()
    }
}

#[async_trait]
impl Repository for MockRepository {
    async fn save(&self, payload: &Payload) -> Result<(), anyhow::Error> {
        let hash_id = payload.hash_id().as_string().to_string();
        self.payloads.lock().unwrap().insert(hash_id, payload.clone());
        Ok(())
    }
    
    async fn get(&self, hash_id: &HashId) -> Result<Option<Payload>, anyhow::Error> {
        let hash_id_str = hash_id.as_string();
        let result = self.payloads.lock().unwrap().get(hash_id_str).cloned();
        Ok(result)
    }
    
    async fn delete(&self, hash_id: &HashId) -> Result<(), anyhow::Error> {
        let hash_id_str = hash_id.as_string();
        self.payloads.lock().unwrap().remove(hash_id_str);
        Ok(())
    }
}

/// Create a test payload with the given content and expiry time.
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
