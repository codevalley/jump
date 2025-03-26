//! API v1 implementation.
//!
//! This module contains the v1 API implementation including:
//! - Route definitions
//! - Request handlers
//! - Middleware
//! - Response types
//! - Payload endpoints

use actix_web::web;

mod payload;

/// Configure v1 API routes.
///
/// This function sets up all the routes for the v1 API.
pub fn configure() -> impl Fn(&mut web::ServiceConfig) {
    |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::scope("/v1")
                // Payload routes
                .route("/payloads", web::post().to(payload::create_payload))
                .route("/payloads/{id}", web::get().to(payload::get_payload))
                .route("/payloads/{id}", web::delete().to(payload::delete_payload))
        );
    }
}

// Re-export handlers for testing
pub use payload::{create_payload, get_payload, delete_payload};
