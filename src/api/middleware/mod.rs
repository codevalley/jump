//! Middleware for the API.
//!
//! This module contains middleware components for the API, including:
//! - Rate limiting middleware
//! - Error handling middleware

pub mod rate_limit;
pub mod error;

pub use rate_limit::RateLimitMiddleware;
pub use error::{ErrorHandlerMiddleware, configure_json_error_handling};
