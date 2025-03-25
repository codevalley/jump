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
use std::sync::Arc;
use thiserror::Error;
use validator::Validate;

use crate::domain::{
    payload::{Payload, PayloadError},
    hash_id::HashId,
};
use super::{
    dtos::{CreatePayloadRequest, CreatePayloadResponse, GetPayloadResponse},
    repository::Repository,
};

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

/// Use case for creating a new payload.
#[async_trait]
pub trait CreatePayloadUseCase: Send + Sync {
    /// Execute the use case.
    async fn execute(
        &self,
        request: CreatePayloadRequest,
    ) -> Result<CreatePayloadResponse, UseCaseError>;
}

/// Use case for retrieving an existing payload.
#[async_trait]
pub trait GetPayloadUseCase: Send + Sync {
    /// Execute the use case.
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
        request.validate().map_err(|e| UseCaseError::ValidationError(e.to_string()))?;

        // Create payload
        let payload = Payload::new(
            request.content,
            request.mime_type,
            request.expiry_time,
        ).map_err(UseCaseError::DomainError)?;

        // Save payload
        self.repository
            .save(&payload)
            .await
            .map_err(UseCaseError::RepositoryError)?;

        // Return response
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
        // Create HashId from string
        let hash_id = HashId::from_string(hash_id);

        // Get payload from repository
        let mut payload = self.repository
            .get(&hash_id)
            .await
            .map_err(UseCaseError::RepositoryError)?
            .ok_or(UseCaseError::NotFound)?;

        // Check if payload has expired
        if payload.is_expired() {
            // Delete expired payload
            self.repository
                .delete(&hash_id)
                .await
                .map_err(UseCaseError::RepositoryError)?;
            return Err(UseCaseError::Expired);
        }

        // Mark payload as viewed
        payload.mark_viewed();
        self.repository
            .save(&payload)
            .await
            .map_err(UseCaseError::RepositoryError)?;

        // Return response
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
    use chrono::{Duration, Utc};
    use mockall::mock;
    use mockall::predicate::*;

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
        let mut mock = MockRepository::new();
        mock.expect_save()
            .with(always())
            .times(1)
            .returning(|_| Ok(()));

        let use_case = CreatePayloadUseCaseImpl::new(Arc::new(mock));
        let request = CreatePayloadRequest {
            content: "test".to_string(),
            mime_type: Some("text/plain".to_string()),
            expiry_time: Some(Utc::now() + Duration::hours(1)),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_payload_validation_error() {
        let mock = MockRepository::new();
        let use_case = CreatePayloadUseCaseImpl::new(Arc::new(mock));
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
        let mut mock = MockRepository::new();
        let payload = Payload::new(
            "test".to_string(),
            Some("text/plain".to_string()),
            Some(Utc::now() + Duration::hours(1)),
        ).unwrap();
        let hash_id = payload.hash_id().clone();

        mock.expect_get()
            .with(eq(hash_id.clone()))
            .times(1)
            .returning(move |_| Ok(Some(payload.clone())));
        
        // We also need to expect save since the use case marks the payload as viewed
        mock.expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let use_case = GetPayloadUseCaseImpl::new(Arc::new(mock));
        let result = use_case.execute(hash_id.as_string().to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_payload_not_found() {
        let mut mock = MockRepository::new();
        let hash_id = HashId::new();

        mock.expect_get()
            .with(eq(hash_id.clone()))
            .times(1)
            .returning(|_| Ok(None));

        let use_case = GetPayloadUseCaseImpl::new(Arc::new(mock));
        let result = use_case.execute(hash_id.as_string().to_string()).await;
        assert!(matches!(result, Err(UseCaseError::NotFound)));
    }
}
