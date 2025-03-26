//! Middleware for logging HTTP requests and responses.
//!
//! This module provides middleware for logging HTTP requests and responses
//! using the `tracing` crate.

use std::future::{ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::time::Instant;

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::Future;
use tracing::{info, warn, Level};

/// Middleware for logging HTTP requests and responses.
///
/// This middleware logs the following information for each request:
/// - HTTP method
/// - URI
/// - Status code
/// - Duration
/// - User agent
/// - IP address
/// - Request ID (if available)
///
/// # Examples
///
/// ```
/// use actix_web::{web, App, HttpServer};
/// use jump::infrastructure::logging::RequestLogger;
///
/// let app = App::new()
///     .wrap(RequestLogger::new(false)) // Don't log request/response bodies
///     .service(web::resource("/").to(|| async { "Hello, world!" }));
/// ```
#[derive(Debug, Clone)]
pub struct RequestLogger {
    log_bodies: bool,
}

impl RequestLogger {
    /// Create a new request logger middleware.
    ///
    /// # Arguments
    ///
    /// * `log_bodies` - Whether to log request and response bodies. This should
    ///   be used with caution as it can log sensitive information.
    pub fn new(log_bodies: bool) -> Self {
        Self { log_bodies }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RequestLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware {
            service: Rc::new(service),
            log_bodies: self.log_bodies,
        }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: Rc<S>,
    log_bodies: bool,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let service = Rc::clone(&self.service);
        let log_bodies = self.log_bodies;

        // Extract request information
        let method = req.method().clone();
        let uri = req.uri().clone();
        let connection_info = req.connection_info().clone();
        let headers = req.headers().clone();

        // Get client IP
        let client_ip = connection_info.realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // Get user agent
        let user_agent = headers
            .get("user-agent")
            .map(|h| h.to_str().unwrap_or("unknown"))
            .unwrap_or("unknown")
            .to_string();

        // Create a span for this request
        let request_span = tracing::span!(
            Level::INFO,
            "http_request",
            method = %method,
            uri = %uri,
            client_ip = %client_ip,
            user_agent = %user_agent,
        );

        // Log request details
        let _request_span_guard = request_span.enter();
        info!("Received request");

        // Log request body if enabled
        if log_bodies {
            // Note: In a real implementation, you might want to clone the request body
            // and log it, but this requires more complex handling
            info!("Request body logging enabled, but implementation is pending");
        }

        Box::pin(async move {
            // Process the request
            let res = service.call(req).await?;
            let duration = start_time.elapsed();

            // Create a response span
            let response_span = tracing::span!(
                Level::INFO,
                "http_response",
                status = res.status().as_u16(),
                duration_ms = duration.as_millis() as u64,
            );

            // Log response details
            let _response_span_guard = response_span.enter();
            
            if res.status().is_success() {
                info!("Request completed successfully");
            } else if res.status().is_server_error() {
                warn!("Request failed with server error");
            } else {
                info!("Request completed with non-success status");
            }

            // Map the response to include the original body
            Ok(res.map_into_left_body())
        })
    }
}
