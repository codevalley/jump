//! Payload module represents the core domain entity for the ephemeral sharing service.
//!
//! A Payload represents a piece of content that can be shared temporarily, with features like:
//! - Unique identification through HashId
//! - Content type validation through MimeType
//! - Automatic expiration through expiry_time
//! - Tracking of creation, update, and view times

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::hash_id::HashId;
use super::mime_type::{MimeType, MimeTypeError};

/// Errors that can occur when working with Payloads.
#[derive(Debug, Error)]
pub enum PayloadError {
    /// The payload has expired and is no longer accessible
    #[error("Payload has expired")]
    Expired,

    /// The MIME type provided is not supported
    #[error("Invalid MIME type: {0}")]
    InvalidMimeType(#[from] MimeTypeError),

    /// The content is empty
    #[error("Content cannot be empty")]
    EmptyContent,
}

/// Represents a shareable payload in the system.
/// 
/// A Payload is the core entity of the ephemeral sharing service. It contains
/// the actual content to be shared, along with metadata about the content
/// and its lifecycle.
/// 
/// # Examples
/// 
/// ```
/// use jump::domain::payload::Payload;
/// use chrono::{Utc, Duration};
/// 
/// let content = "Hello, World!";
/// let payload = Payload::new(
///     content.to_string(),
///     None, // Default to text/plain
///     None, // Default expiry time
/// ).unwrap();
/// 
/// assert_eq!(payload.content(), content);
/// assert!(!payload.is_expired());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    /// Unique identifier for the payload
    hash_id: HashId,
    
    /// The actual content being shared
    content: String,
    
    /// The MIME type of the content
    mime_type: MimeType,
    
    /// When the payload was created
    created_at: DateTime<Utc>,
    
    /// Last time the payload was updated
    updated_at: DateTime<Utc>,
    
    /// When the payload was last viewed
    viewed_at: Option<DateTime<Utc>>,
    
    /// When the payload will expire
    expiry_time: DateTime<Utc>,
}

impl Payload {
    /// Creates a new Payload with the given content and optional parameters.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The content to be shared
    /// * `mime_type` - Optional MIME type (defaults to text/plain)
    /// * `expiry_time` - Optional expiry time (defaults to 24 hours from creation)
    /// 
    /// # Errors
    /// 
    /// Returns `PayloadError::EmptyContent` if the content is empty
    /// Returns `PayloadError::InvalidMimeType` if the MIME type is not supported
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::payload::Payload;
    /// use chrono::{Utc, Duration};
    /// 
    /// // Create with defaults
    /// let payload = Payload::new(
    ///     "Hello".to_string(),
    ///     None,
    ///     None,
    /// ).unwrap();
    /// 
    /// // Create with specific MIME type and expiry
    /// let payload = Payload::new(
    ///     "Hello".to_string(),
    ///     Some("text/html".to_string()),
    ///     Some(Utc::now() + Duration::hours(1)),
    /// ).unwrap();
    /// ```
    pub fn new(
        content: String,
        mime_type: Option<String>,
        expiry_time: Option<DateTime<Utc>>,
    ) -> Result<Self, PayloadError> {
        if content.is_empty() {
            return Err(PayloadError::EmptyContent);
        }

        let now = Utc::now();
        let mime_type = match mime_type {
            Some(mime_str) => MimeType::try_from(mime_str.as_str())?,
            None => MimeType::TextPlain,
        };

        Ok(Self {
            hash_id: HashId::new(),
            content,
            mime_type,
            created_at: now,
            updated_at: now,
            viewed_at: None,
            expiry_time: expiry_time.unwrap_or_else(|| now + Duration::hours(24)),
        })
    }

    /// Returns true if the payload has expired.
    /// 
    /// A payload is considered expired if the current time is past its expiry_time.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry_time
    }

    /// Records that the payload was viewed at the current time.
    pub fn mark_viewed(&mut self) {
        self.viewed_at = Some(Utc::now());
    }

    /// Returns the unique identifier of the payload.
    pub fn hash_id(&self) -> &HashId {
        &self.hash_id
    }

    /// Returns the content of the payload.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the MIME type of the payload.
    pub fn mime_type(&self) -> &MimeType {
        &self.mime_type
    }

    /// Returns when the payload was created.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns when the payload was last updated.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns when the payload was last viewed, if ever.
    pub fn viewed_at(&self) -> Option<DateTime<Utc>> {
        self.viewed_at
    }

    /// Returns when the payload will expire.
    pub fn expiry_time(&self) -> DateTime<Utc> {
        self.expiry_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payload_with_defaults() {
        let content = "Test content".to_string();
        let payload = Payload::new(content.clone(), None, None).unwrap();

        assert_eq!(payload.content(), content);
        assert!(matches!(payload.mime_type(), MimeType::TextPlain));
        assert!(payload.viewed_at().is_none());
        assert!(!payload.is_expired());
    }

    #[test]
    fn test_create_payload_with_custom_mime_type() {
        let payload = Payload::new(
            "Test content".to_string(),
            Some("text/html".to_string()),
            None,
        )
        .unwrap();

        assert!(matches!(payload.mime_type(), MimeType::TextHtml));
    }

    #[test]
    fn test_create_payload_with_invalid_mime_type() {
        let result = Payload::new(
            "Test content".to_string(),
            Some("invalid/type".to_string()),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_create_payload_with_empty_content() {
        let result = Payload::new("".to_string(), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_mark_viewed_updates_viewed_at() {
        let mut payload = Payload::new("Test content".to_string(), None, None).unwrap();
        assert!(payload.viewed_at().is_none());

        payload.mark_viewed();
        assert!(payload.viewed_at().is_some());
    }

    #[test]
    fn test_payload_expires() {
        let payload = Payload::new(
            "Test content".to_string(),
            None,
            Some(Utc::now() - Duration::hours(1)),
        )
        .unwrap();

        assert!(payload.is_expired());
    }
}
