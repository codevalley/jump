//! MimeType module provides a type-safe way to handle MIME types for payloads.
//!
//! This module ensures that:
//! - Only supported MIME types are allowed
//! - MIME types are properly formatted
//! - MIME types are validated before use

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors that can occur when working with MIME types.
#[derive(Debug, Error)]
pub enum MimeTypeError {
    /// The MIME type is not supported by the system.
    #[error("Unsupported MIME type: {0}")]
    UnsupportedMimeType(String),
}

/// Represents a supported MIME type for payloads.
/// 
/// This enum ensures that only supported MIME types can be used in the system.
/// The supported types are based on the requirements specified in the PRD.
/// 
/// # Examples
/// 
/// ```
/// use jump::domain::mime_type::MimeType;
/// 
/// let mime = MimeType::TextPlain;
/// assert_eq!(mime.to_string(), "text/plain");
/// 
/// let from_str = MimeType::try_from("text/plain").unwrap();
/// assert_eq!(from_str, MimeType::TextPlain);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MimeType {
    /// Plain text content (text/plain)
    TextPlain,
    /// HTML content (text/html)
    TextHtml,
    /// JSON content (application/json)
    ApplicationJson,
    /// JPEG image (image/jpeg)
    ImageJpeg,
    /// PNG image (image/png)
    ImagePng,
    /// GIF image (image/gif)
    ImageGif,
}

impl MimeType {
    /// Returns all supported MIME types.
    /// 
    /// This is useful for validation and documentation purposes.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::mime_type::MimeType;
    /// 
    /// let supported = MimeType::supported_types();
    /// assert!(supported.contains(&"text/plain"));
    /// ```
    pub fn supported_types() -> Vec<&'static str> {
        vec![
            "text/plain",
            "text/html",
            "application/json",
            "image/jpeg",
            "image/png",
            "image/gif",
        ]
    }

    /// Checks if a MIME type string is supported.
    /// 
    /// # Arguments
    /// 
    /// * `mime_type` - The MIME type string to check
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::mime_type::MimeType;
    /// 
    /// assert!(MimeType::is_supported("text/plain"));
    /// assert!(!MimeType::is_supported("application/pdf"));
    /// ```
    pub fn is_supported(mime_type: &str) -> bool {
        Self::supported_types().contains(&mime_type)
    }
}

impl TryFrom<&str> for MimeType {
    type Error = MimeTypeError;

    /// Tries to create a MimeType from a string.
    /// 
    /// # Arguments
    /// 
    /// * `s` - The MIME type string
    /// 
    /// # Errors
    /// 
    /// Returns `MimeTypeError::UnsupportedMimeType` if the MIME type is not supported.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::mime_type::MimeType;
    /// 
    /// let mime = MimeType::try_from("text/plain").unwrap();
    /// assert_eq!(mime, MimeType::TextPlain);
    /// 
    /// let error = MimeType::try_from("application/pdf").unwrap_err();
    /// assert!(error.to_string().contains("Unsupported MIME type"));
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "text/plain" => Ok(Self::TextPlain),
            "text/html" => Ok(Self::TextHtml),
            "application/json" => Ok(Self::ApplicationJson),
            "image/jpeg" => Ok(Self::ImageJpeg),
            "image/png" => Ok(Self::ImagePng),
            "image/gif" => Ok(Self::ImageGif),
            _ => Err(MimeTypeError::UnsupportedMimeType(s.to_string())),
        }
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::TextPlain => "text/plain",
            Self::TextHtml => "text/html",
            Self::ApplicationJson => "application/json",
            Self::ImageJpeg => "image/jpeg",
            Self::ImagePng => "image/png",
            Self::ImageGif => "image/gif",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_types_contains_required_types() {
        let types = MimeType::supported_types();
        assert!(types.contains(&"text/plain"));
        assert!(types.contains(&"text/html"));
        assert!(types.contains(&"application/json"));
        assert!(types.contains(&"image/jpeg"));
        assert!(types.contains(&"image/png"));
        assert!(types.contains(&"image/gif"));
    }

    #[test]
    fn test_valid_mime_type_conversion() {
        assert_eq!(MimeType::try_from("text/plain").unwrap(), MimeType::TextPlain);
        assert_eq!(MimeType::try_from("text/html").unwrap(), MimeType::TextHtml);
        assert_eq!(
            MimeType::try_from("application/json").unwrap(),
            MimeType::ApplicationJson
        );
    }

    #[test]
    fn test_invalid_mime_type_conversion() {
        assert!(MimeType::try_from("application/pdf").is_err());
        assert!(MimeType::try_from("invalid").is_err());
    }

    #[test]
    fn test_mime_type_display() {
        assert_eq!(MimeType::TextPlain.to_string(), "text/plain");
        assert_eq!(MimeType::ApplicationJson.to_string(), "application/json");
    }

    #[test]
    fn test_is_supported() {
        assert!(MimeType::is_supported("text/plain"));
        assert!(MimeType::is_supported("application/json"));
        assert!(!MimeType::is_supported("application/pdf"));
    }
}
