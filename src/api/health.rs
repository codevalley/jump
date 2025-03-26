//! Health check endpoints.
//!
//! This module provides health check endpoints for the API.

use actix_web::{web, HttpResponse, Responder};
use tracing::info;

/// Health check endpoint.
///
/// Returns a simple OK response to indicate that the service is running.
#[tracing::instrument(name = "Health check")]
pub async fn health_check() -> impl Responder {
    info!("Health check request received");
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Configure health check routes.
pub fn configure() -> impl Fn(&mut web::ServiceConfig) {
    |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::scope("/health")
                .route("", web::get().to(health_check))
        );
    }
}
