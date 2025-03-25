//! Use cases for the application layer.
//!
//! This module contains the business logic for:
//! - Creating new payloads
//! - Retrieving existing payloads
//! - Handling payload expiry
//! 
//! Use cases are the primary way that the API layer interacts with the domain model.
//! They encapsulate all business rules and coordinate between different parts of the system.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use thiserror::Error;
use validator::Validate;

use crate::domain::{
    payload::{Payload, PayloadError},
    hash_id::HashId,
};
use super::dtos::{CreatePayloadRequest, CreatePayloadResponse, GetPayloadResponse};

/// Errors that can occur in use cases.
#[derive(Debug, Error)]
pub enum UseCaseError {
    /// Validation error for input data
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Payload was not found
    #[error("Payload not found")]
    NotFound,

    /// Payload has expired
    #[error("Payload has expired")]
    Expired,

    /// Repository error
    #[error("Repository error: {0}")]
    RepositoryError(#[from] anyhow::Error),

    /// Domain error
    #[error("Domain error: {0}")]
    DomainError(#[from] PayloadError),
}

/// Repository trait defining storage operations needed by use cases.
#[async_trait]
pub trait Repository: Send + Sync {
    /// Save a payload to storage
    async fn save(&self, payload: &Payload) -> Result<(), anyhow::Error>;
    
    /// Get a payload by its hash_id
    async fn get(&self, hash_id: &HashId) -> Result<Option<Payload>, anyhow::Error>;
    
    /// Delete a payload by its hash_id
    async fn delete(&self, hash_id: &HashId) -> Result<(), anyhow::Error>;
}

/// Use case for creating a new payload.
#[async_trait]
pub trait CreatePayloadUseCase {
    /// Execute the create payload use case.
    /// 
    /// # Arguments
    /// 
    /// * `request` - The create payload request DTO
    /// 
    /// # Returns
    /// 
    /// Returns the created payload response or an error if creation failed.
    /// 
    /// # Errors
    /// 
    /// Returns `UseCaseError` if:
    /// - Input validation fails
    /// - Repository operations fail
    /// - Domain rules are violated
    async fn execute(
        &self,
        request: CreatePayloadRequest,
    ) -> Result<CreatePayloadResponse, UseCaseError>;
}

/// Use case for retrieving an existing payload.
#[async_trait]
pub trait GetPayloadUseCase {
    /// Execute the get payload use case.
    /// 
    /// # Arguments
    /// 
    /// * `hash_id` - The unique identifier of the payload to retrieve
    /// 
    /// # Returns
    /// 
    /// Returns the payload response if found, or an error if retrieval failed.
    /// 
    /// # Errors
    /// 
    /// Returns `UseCaseError` if:
    /// - Payload is not found
    /// - Payload has expired
    /// - Repository operations fail
    async fn execute(&self, hash_id: String) -> Result<GetPayloadResponse, UseCaseError>;
}

/// Implementation of the create payload use case.
pub struct CreatePayloadUseCaseImpl {
    repository: Arc<dyn Repository>,
}

impl CreatePayloadUseCaseImpl {
    /// Create a new instance of the use case implementation.
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CreatePayloadUseCase for CreatePayloadUseCaseImpl {
    async fn execute(
        &self,
        request: CreatePayloadRequest,
    ) -> Result<CreatePayloadResponse, UseCaseError> {
        // Validate request
        if let Err(errors) = request.validate() {
            return Err(UseCaseError::ValidationError(errors.to_string()));
        }

        // Create domain entity
        let payload = Payload::new(
            request.content,
            request.mime_type,
            request.expiry_time,
        )?;

        // Save to repository
        self.repository.save(&payload).await?;

        // Create response
        Ok(CreatePayloadResponse {
            hash_id: payload.hash_id().as_string().to_string(),
            content: payload.content().to_string(),
            mime_type: payload.mime_type().to_string(),
            created_at: payload.created_at(),
            updated_at: payload.updated_at(),
            viewed_at: payload.viewed_at(),
            expiry_time: payload.expiry_time(),
        })
    }
}

/// Implementation of the get payload use case.
pub struct GetPayloadUseCaseImpl {
    repository: Arc<dyn Repository>,
}

impl GetPayloadUseCaseImpl {
    /// Create a new instance of the use case implementation.
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl GetPayloadUseCase for GetPayloadUseCaseImpl {
    async fn execute(&self, hash_id: String) -> Result<GetPayloadResponse, UseCaseError> {
        // Get payload from repository
        let mut payload = self
            .repository
            .get(&HashId::from_string(hash_id))
            .await?
            .ok_or(UseCaseError::NotFound)?;

        // Check if expired
        if payload.is_expired() {
            // Delete expired payload
            self.repository
                .delete(payload.hash_id())
                .await
                .map_err(UseCaseError::RepositoryError)?;
            return Err(UseCaseError::Expired);
        }

        // Mark as viewed
        payload.mark_viewed();
        self.repository.save(&payload).await?;

        // Create response
        Ok(GetPayloadResponse {
            hash_id: payload.hash_id().as_string().to_string(),
            content: payload.content().to_string(),
            mime_type: payload.mime_type().to_string(),
            created_at: payload.created_at(),
            updated_at: payload.updated_at(),
            viewed_at: payload.viewed_at(),
            expiry_time: payload.expiry_time(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        Repository {}

        #[async_trait]
        impl Repository for Repository {
            async fn save(&self, payload: &Payload) -> Result<(), anyhow::Error>;
            async fn get(&self, hash_id: &HashId) -> Result<Option<Payload>, anyhow::Error>;
            async fn delete(&self, hash_id: &HashId) -> Result<(), anyhow::Error>;
        }
    }

    #[tokio::test]
    async fn test_create_payload_success() {
        let repository = Arc::new(MockRepository::new());
        let use_case = CreatePayloadUseCaseImpl::new(repository);

        let request = CreatePayloadRequest {
            content: "Test content".to_string(),
            mime_type: Some("text/plain".to_string()),
            expiry_time: None,
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_payload_validation_error() {
        let repository = Arc::new(MockRepository::new());
        let use_case = CreatePayloadUseCaseImpl::new(repository);

        let request = CreatePayloadRequest {
            content: "".to_string(), // Empty content should fail validation
            mime_type: None,
            expiry_time: None,
        };

        let result = use_case.execute(request).await;
        assert!(matches!(result, Err(UseCaseError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_get_payload_success() {
        let mut repository = MockRepository::new();
        
        // Setup mock repository
        repository
            .expect_get()
            .returning(|_| Ok(Some(Payload::new(
                "Test content".to_string(),
                None,
                None,
            ).unwrap())));
        
        repository
            .expect_save()
            .returning(|_| Ok(()));

        let use_case = GetPayloadUseCaseImpl::new(Arc::new(repository));
        let result = use_case.execute("test-hash".to_string()).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_payload_not_found() {
        let mut repository = MockRepository::new();
        
        repository
            .expect_get()
            .returning(|_| Ok(None));

        let use_case = GetPayloadUseCaseImpl::new(Arc::new(repository));
        let result = use_case.execute("nonexistent".to_string()).await;
        
        assert!(matches!(result, Err(UseCaseError::NotFound)));
    }
}
