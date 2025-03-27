//! Payload API endpoints.
//!
//! This module contains the API endpoints for creating and retrieving payloads.

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use tracing::{info, warn, error};
use std::sync::Arc;

use crate::{
    application::{
        dtos::CreatePayloadRequest,
        use_cases::{
            CreatePayloadUseCaseImpl, GetPayloadUseCaseImpl, DeletePayloadUseCaseImpl,
            CreatePayloadUseCase, GetPayloadUseCase, DeletePayloadUseCase,
            UseCaseError,
        },
    },
};

/// Maximum payload size in bytes (10MB)
const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Create a new payload.
///
/// # Request
///
/// ```json
/// {
///     "content": "Your payload content here",
///     "mime_type": "text/plain",
///     "expiry_time": "2024-03-14T12:00:00Z"
/// }
/// ```
///
/// # Response
///
/// ```json
/// {
///     "hash_id": "unique-hash-id",
///     "expires_at": "2023-01-01T00:00:00Z"
/// }
/// ```
#[tracing::instrument(
    name = "Create payload",
    skip(create_payload_use_case, payload),
    fields(
        payload_size = %payload.content.len(),
        mime_type = ?payload.mime_type,
        expiry_time = ?payload.expiry_time
    )
)]
pub async fn create_payload(
    create_payload_use_case: Data<Arc<CreatePayloadUseCaseImpl>>,
    payload: Json<CreatePayloadRequest>,
) -> impl Responder {
    // Validate payload size
    if payload.content.len() > MAX_PAYLOAD_SIZE {
        warn!(
            payload_size = %payload.content.len(),
            "Payload too large, maximum size is {} bytes", MAX_PAYLOAD_SIZE
        );
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Payload too large, maximum size is {} bytes", MAX_PAYLOAD_SIZE)
        }));
    }

    // Create payload
    match create_payload_use_case.execute(payload.into_inner()).await {
        Ok(response) => {
            info!(
                hash_id = %response.hash_id,
                "Payload created successfully"
            );
            HttpResponse::Created().json(serde_json::json!({
                "hash_id": response.hash_id,
                "expires_at": response.expiry_time
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to create payload");
            match e {
                UseCaseError::ValidationError(msg) => {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": msg
                    }))
                }
                UseCaseError::RepositoryError(_) => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Internal server error"
                    }))
                }
                UseCaseError::DomainError(err) => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": err.to_string()
                    }))
                }
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "An unexpected error occurred"
                    }))
                }
            }
        }
    }
}

/// Get a payload by ID.
///
/// # Response
///
/// ```json
/// {
///     "hash_id": "unique-hash-id",
///     "content": "Your payload content here",
///     "mime_type": "text/plain",
///     "created_at": "2023-01-01T00:00:00Z",
///     "updated_at": "2023-01-01T00:00:00Z",
///     "viewed_at": "2023-01-01T00:00:00Z",
///     "expiry_time": "2023-01-01T01:00:00Z"
/// }
/// ```
#[tracing::instrument(
    name = "Get payload",
    skip(get_payload_use_case),
    fields(hash_id = %id)
)]
pub async fn get_payload(
    get_payload_use_case: Data<Arc<GetPayloadUseCaseImpl>>,
    id: Path<String>,
) -> impl Responder {
    info!("Processing get payload request");
    
    // Get the ID as a string
    let id_string = id.into_inner();
    
    // Get payload
    match get_payload_use_case.execute(id_string).await {
        Ok(response) => {
            info!(
                hash_id = %response.hash_id,
                "Payload retrieved successfully"
            );
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!(error = %e, "Failed to retrieve payload");
            match e {
                UseCaseError::NotFound => {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Payload not found"
                    }))
                }
                UseCaseError::Expired => {
                    HttpResponse::Gone().json(serde_json::json!({
                        "error": "Payload has expired"
                    }))
                }
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "An unexpected error occurred"
                    }))
                }
            }
        }
    }
}

/// Delete a payload by ID.
///
/// This endpoint deletes a payload by its ID.
#[tracing::instrument(
    name = "Delete payload",
    skip(delete_payload_use_case),
    fields(hash_id = %id)
)]
pub async fn delete_payload(
    delete_payload_use_case: Data<Arc<DeletePayloadUseCaseImpl>>,
    id: Path<String>,
) -> impl Responder {
    info!("Processing delete payload request");
    
    // Delete the payload
    match delete_payload_use_case.delete(&id).await {
        Ok(_) => {
            info!("Payload deleted successfully");
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            error!(error = %e, "Failed to delete payload");
            match e {
                UseCaseError::RepositoryError(_) => {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Payload not found"
                    }))
                }
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "An unexpected error occurred"
                    }))
                }
            }
        }
    }
}
