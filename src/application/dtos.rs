//! Data Transfer Objects (DTOs) for the application layer.
//!
//! This module contains all the DTOs used for:
//! - API request/response objects
//! - Data validation
//! - Serialization/deserialization of payloads
//! 
//! DTOs are designed to be API-version specific and decouple the domain model
//! from the external interface.

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

lazy_static! {
    /// Regular expression for validating MIME types
    static ref MIME_TYPE_REGEX: Regex = Regex::new(r"^[a-z]+/[a-z0-9.+-]+$").unwrap();
}

/// Request DTO for creating a new payload.
/// 
/// This struct represents the expected JSON structure for POST /api/v1/payloads
/// 
/// # Example JSON
/// ```json
/// {
///   "content": "Your payload content here",
///   "mime_type": "text/plain",
///   "expiry_time": "2024-03-14T12:00:00Z"
/// }
/// ```
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePayloadRequest {
    /// The content to be shared. Must not be empty.
    #[validate(length(min = 1, message = "Content cannot be empty"))]
    #[validate(length(max = 1048576, message = "Content cannot exceed 1MB"))]
    pub content: String,

    /// Optional MIME type of the content. If not provided, defaults to "text/plain".
    #[validate(regex(
        path = "MIME_TYPE_REGEX",
        message = "Invalid MIME type format"
    ))]
    pub mime_type: Option<String>,

    /// Optional expiry time. If not provided, defaults to 24 hours from creation.
    pub expiry_time: Option<DateTime<Utc>>,
}

/// Response DTO for successful payload creation.
/// 
/// This struct represents the JSON structure returned after successfully
/// creating a new payload.
#[derive(Debug, Serialize)]
pub struct CreatePayloadResponse {
    /// The unique identifier for accessing the payload
    pub hash_id: String,
    
    /// The stored content
    pub content: String,
    
    /// The MIME type of the content
    pub mime_type: String,
    
    /// When the payload was created
    pub created_at: DateTime<Utc>,
    
    /// When the payload was last updated
    pub updated_at: DateTime<Utc>,
    
    /// When the payload was last viewed (if ever)
    pub viewed_at: Option<DateTime<Utc>>,
    
    /// When the payload will expire
    pub expiry_time: DateTime<Utc>,
}

/// Response DTO for retrieving a payload.
/// 
/// This struct represents the JSON structure returned when retrieving
/// an existing payload.
#[derive(Debug, Serialize)]
pub struct GetPayloadResponse {
    /// The unique identifier of the payload
    pub hash_id: String,
    
    /// The stored content
    pub content: String,
    
    /// The MIME type of the content
    pub mime_type: String,
    
    /// When the payload was created
    pub created_at: DateTime<Utc>,
    
    /// When the payload was last updated
    pub updated_at: DateTime<Utc>,
    
    /// When the payload was last viewed (if ever)
    pub viewed_at: Option<DateTime<Utc>>,
    
    /// When the payload will expire
    pub expiry_time: DateTime<Utc>,
}

/// Error response DTO.
/// 
/// This struct represents the JSON structure returned when an error occurs.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// The error message
    pub error: String,
    
    /// Optional field for rate limit errors, indicating when to retry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_payload_request_validation() {
        // Valid request
        let valid_request = CreatePayloadRequest {
            content: "Test content".to_string(),
            mime_type: Some("text/plain".to_string()),
            expiry_time: None,
        };
        assert!(valid_request.validate().is_ok());

        // Empty content
        let empty_content = CreatePayloadRequest {
            content: "".to_string(),
            mime_type: None,
            expiry_time: None,
        };
        assert!(empty_content.validate().is_err());

        // Invalid MIME type
        let invalid_mime = CreatePayloadRequest {
            content: "Test content".to_string(),
            mime_type: Some("invalid-mime-type".to_string()),
            expiry_time: None,
        };
        assert!(invalid_mime.validate().is_err());
    }
}
