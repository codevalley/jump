//! Tests for the application layer use cases.

use std::sync::Arc;
use chrono::{Duration, Utc};

use crate::application::{
    dtos::CreatePayloadRequest,
    use_cases::{
        CreatePayloadUseCase, GetPayloadUseCase,
        CreatePayloadUseCaseImpl, GetPayloadUseCaseImpl,
        UseCaseError,
    },
};
use crate::domain::hash_id::HashId;
use super::{MockRepository, create_test_payload};

#[tokio::test]
async fn test_create_payload_with_valid_data() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = CreatePayloadUseCaseImpl::new(repository.clone());
    
    let request = CreatePayloadRequest {
        content: "Test content".to_string(),
        mime_type: Some("text/plain".to_string()),
        expiry_time: None,
    };
    
    // Act
    let result = use_case.execute(request).await;
    
    // Assert
    assert!(result.is_ok(), "Expected successful payload creation");
    let response = result.unwrap();
    assert!(!response.hash_id.is_empty(), "Expected non-empty hash ID");
    assert!(response.expiry_time > Utc::now(), "Expected future expiry time");
    assert_eq!(repository.count(), 1, "Expected one payload in repository");
}

#[tokio::test]
async fn test_create_payload_with_custom_expiry() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = CreatePayloadUseCaseImpl::new(repository.clone());
    
    let expiry_time = Utc::now() + Duration::hours(2);
    let request = CreatePayloadRequest {
        content: "Test content".to_string(),
        mime_type: Some("text/plain".to_string()),
        expiry_time: Some(expiry_time),
    };
    
    // Act
    let result = use_case.execute(request).await;
    
    // Assert
    assert!(result.is_ok(), "Expected successful payload creation");
    let response = result.unwrap();
    assert_eq!(
        response.expiry_time.timestamp(), 
        expiry_time.timestamp(), 
        "Expected custom expiry time"
    );
}

#[tokio::test]
async fn test_create_payload_with_empty_content() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = CreatePayloadUseCaseImpl::new(repository.clone());
    
    let request = CreatePayloadRequest {
        content: "".to_string(),
        mime_type: Some("text/plain".to_string()),
        expiry_time: None,
    };
    
    // Act
    let result = use_case.execute(request).await;
    
    // Assert
    assert!(result.is_err(), "Expected error for empty content");
    match result {
        Err(UseCaseError::DomainError(_)) => {
            // This is the expected error type
        }
        _ => panic!("Expected DomainError for empty content"),
    }
    assert_eq!(repository.count(), 0, "Expected no payloads in repository");
}

#[tokio::test]
async fn test_create_payload_with_invalid_mime_type() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = CreatePayloadUseCaseImpl::new(repository.clone());
    
    let request = CreatePayloadRequest {
        content: "Test content".to_string(),
        mime_type: Some("invalid/type".to_string()),
        expiry_time: None,
    };
    
    // Act
    let result = use_case.execute(request).await;
    
    // Assert
    assert!(result.is_err(), "Expected error for invalid MIME type");
    match result {
        Err(UseCaseError::DomainError(_)) => {
            // This is the expected error type
        }
        _ => panic!("Expected DomainError for invalid MIME type"),
    }
    assert_eq!(repository.count(), 0, "Expected no payloads in repository");
}

#[tokio::test]
async fn test_get_payload_with_valid_id() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = GetPayloadUseCaseImpl::new(repository.clone());
    
    // Create a test payload and add it to the repository
    let payload = create_test_payload("Test content", None);
    let hash_id = payload.hash_id().as_string().to_string();
    repository.add_payload(payload);
    
    // Act
    let result = use_case.execute(hash_id).await;
    
    // Assert
    assert!(result.is_ok(), "Expected successful payload retrieval");
    let response = result.unwrap();
    assert_eq!(response.content, "Test content", "Expected matching content");
    assert_eq!(response.mime_type, "text/plain", "Expected matching MIME type");
}

#[tokio::test]
async fn test_get_payload_with_nonexistent_id() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = GetPayloadUseCaseImpl::new(repository.clone());
    
    // Act
    let result = use_case.execute("nonexistent-id".to_string()).await;
    
    // Assert
    assert!(result.is_err(), "Expected error for nonexistent ID");
    match result {
        Err(UseCaseError::NotFound) => {
            // This is the expected error type
        }
        _ => panic!("Expected NotFound error for nonexistent ID"),
    }
}

#[tokio::test]
async fn test_get_expired_payload() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let use_case = GetPayloadUseCaseImpl::new(repository.clone());
    
    // Create an expired payload and add it to the repository
    let expired_time = Utc::now() - Duration::hours(1);
    let payload = create_test_payload("Expired content", Some(expired_time));
    let hash_id = payload.hash_id().as_string().to_string();
    repository.add_payload(payload);
    
    // Act
    let result = use_case.execute(hash_id).await;
    
    // Assert
    assert!(result.is_err(), "Expected error for expired payload");
    match result {
        Err(UseCaseError::Expired) => {
            // This is the expected error type
        }
        _ => panic!("Expected Expired error for expired payload"),
    }
}
