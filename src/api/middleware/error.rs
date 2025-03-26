//! Error handling middleware for the API.
//!
//! This middleware provides consistent error handling and logging for API errors.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    web::JsonConfig,
    Error, HttpResponse, ResponseError,
};
use futures::future::{ok, Ready};
use futures::Future;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::pin::Pin;
use std::rc::Rc;
use tracing::{error, warn};

/// Standard error response format for the API.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// HTTP status code
    pub status: u16,
    /// Error message
    pub message: String,
    /// Error code (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Request ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.status, self.message)
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }
}

/// Error handling middleware for the API.
///
/// This middleware catches errors from the service and converts them to
/// standardized JSON error responses. It also logs errors with appropriate
/// severity levels.
///
/// # Examples
///
/// ```
/// use actix_web::{web, App, HttpServer};
/// use jump::api::middleware::error::ErrorHandlerMiddleware;
///
/// let app = App::new()
///     .wrap(ErrorHandlerMiddleware::new())
///     .service(web::resource("/").to(|| async { "Hello, world!" }));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ErrorHandlerMiddleware;

impl ErrorHandlerMiddleware {
    /// Create a new error handler middleware.
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for ErrorHandlerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ErrorHandlerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorHandlerMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct ErrorHandlerMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ErrorHandlerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let result = service.call(req).await;

            match result {
                Ok(response) => {
                    // If the response is an error (4xx or 5xx), log it
                    let status = response.status();
                    if status.is_client_error() {
                        warn!(
                            status_code = status.as_u16(),
                            path = response.request().path(),
                            "Client error"
                        );
                    } else if status.is_server_error() {
                        error!(
                            status_code = status.as_u16(),
                            path = response.request().path(),
                            "Server error"
                        );
                    }
                    Ok(response)
                }
                Err(err) => {
                    // Log the error
                    error!(
                        error = %err,
                        "Request processing error"
                    );
                    Err(err)
                }
            }
        })
    }
}

/// Configure JSON error handling for the application.
///
/// This function returns a JsonConfig that will handle JSON deserialization
/// errors and convert them to standardized error responses.
pub fn configure_json_error_handling() -> JsonConfig {
    JsonConfig::default()
        .error_handler(|err, _req| {
            // Log the error
            warn!(error = %err, "JSON deserialization error");
            
            // Create a standardized error response
            let error_response = ErrorResponse {
                status: StatusCode::BAD_REQUEST.as_u16(),
                message: format!("JSON error: {}", err),
                code: Some("INVALID_JSON".to_string()),
                request_id: None,
            };
            
            actix_web::error::InternalError::from_response(
                err,
                HttpResponse::BadRequest().json(error_response),
            )
            .into()
        })
}
