//! Repository trait for data persistence.

use async_trait::async_trait;

use crate::domain::{hash_id::HashId, payload::Payload};

/// Repository trait for storing and retrieving payloads.
#[async_trait]
pub trait Repository: Send + Sync {
    /// Save a payload to the repository.
    async fn save(&self, payload: &Payload) -> Result<(), anyhow::Error>;

    /// Get a payload from the repository by its hash ID.
    async fn get(&self, hash_id: &HashId) -> Result<Option<Payload>, anyhow::Error>;

    /// Delete a payload from the repository by its hash ID.
    async fn delete(&self, hash_id: &HashId) -> Result<(), anyhow::Error>;
}
