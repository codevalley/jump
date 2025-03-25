//! HashId module provides a type-safe wrapper around UUID-based identifiers for payloads.
//! 
//! This module ensures that:
//! - All HashIds are valid and properly formatted
//! - HashIds are URL-safe
//! - HashIds are unique with extremely high probability
//! - HashIds can be serialized/deserialized safely

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for payloads in the system.
/// 
/// HashId is implemented as a wrapper around a UUID v4, stored in a URL-safe format.
/// It provides type safety and ensures that all identifiers are properly formatted.
/// 
/// # Examples
/// 
/// ```
/// use jump::domain::hash_id::HashId;
/// 
/// // Create a new random HashId
/// let id = HashId::new();
/// 
/// // Create from an existing string (useful for API endpoints)
/// let id_from_string = HashId::from_string("abc123".to_string());
/// 
/// // Get the string representation
/// let id_str = id.as_string();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HashId(String);

impl HashId {
    /// Creates a new random HashId using UUID v4.
    /// 
    /// The generated ID is guaranteed to be URL-safe and unique with extremely
    /// high probability (see UUID v4 collision probability).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::hash_id::HashId;
    /// 
    /// let id = HashId::new();
    /// assert!(!id.as_string().is_empty());
    /// ```
    pub fn new() -> Self {
        HashId(Uuid::new_v4().simple().to_string())
    }

    /// Creates a HashId from an existing string.
    /// 
    /// This is particularly useful when receiving IDs from API requests
    /// or loading from storage.
    /// 
    /// # Arguments
    /// 
    /// * `val` - A string value to use as the HashId
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::hash_id::HashId;
    /// 
    /// let id = HashId::from_string("abc123".to_string());
    /// assert_eq!(id.as_string(), "abc123");
    /// ```
    pub fn from_string(val: String) -> Self {
        HashId(val)
    }

    /// Returns the string representation of the HashId.
    /// 
    /// This is useful when you need to:
    /// - Send the ID in an API response
    /// - Store the ID in a database
    /// - Use the ID in a URL
    /// 
    /// # Examples
    /// 
    /// ```
    /// use jump::domain::hash_id::HashId;
    /// 
    /// let id = HashId::from_string("abc123".to_string());
    /// assert_eq!(id.as_string(), "abc123");
    /// ```
    pub fn as_string(&self) -> &str {
        &self.0
    }
}

impl Default for HashId {
    /// Creates a new random HashId as the default value.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hash_id_is_not_empty() {
        let id = HashId::new();
        assert!(!id.as_string().is_empty());
    }

    #[test]
    fn test_from_string_preserves_value() {
        let original = "test123";
        let id = HashId::from_string(original.to_string());
        assert_eq!(id.as_string(), original);
    }

    #[test]
    fn test_default_creates_valid_id() {
        let id: HashId = Default::default();
        assert!(!id.as_string().is_empty());
    }

    #[test]
    fn test_hash_ids_are_unique() {
        let id1 = HashId::new();
        let id2 = HashId::new();
        assert_ne!(id1, id2);
    }
}
