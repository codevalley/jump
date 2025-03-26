//! API layer implementation.
//!
//! This module contains the API implementation including:
//! - Route definitions
//! - Request handlers
//! - Middleware
//! - Response types

use actix_web::web;

pub mod v1;
pub mod middleware;
pub mod health;

/// Configure all API routes.
///
/// This function sets up all the routes for the API, including all versions.
pub fn configure() -> impl Fn(&mut web::ServiceConfig) {
    |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::scope("/api")
                .configure(v1::configure())
                .configure(health::configure())
        );
    }
}
