//! Rate limiting middleware for actix-web.
//!
//! This middleware implements rate limiting for API endpoints using
//! the Redis-based rate limiter.

use std::task::{Context, Poll};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorBadRequest,
    Error,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::infrastructure::rate_limit::{RateLimiter, RateLimitError};

/// Rate limiting middleware
pub struct RateLimitMiddleware<T>
where
    T: RateLimiter,
{
    limiter: Rc<T>,
}

impl<T> RateLimitMiddleware<T>
where
    T: RateLimiter,
{
    pub fn new(limiter: T) -> Self {
        Self {
            limiter: Rc::new(limiter),
        }
    }
}

impl<S, B, T> Transform<S, ServiceRequest> for RateLimitMiddleware<T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    T: RateLimiter + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddlewareService {
            service,
            limiter: self.limiter.clone(),
        })
    }
}

pub struct RateLimitMiddlewareService<S, T>
where
    T: RateLimiter,
{
    service: S,
    limiter: Rc<T>,
}

impl<S, B, T> Service<ServiceRequest> for RateLimitMiddlewareService<S, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    T: RateLimiter + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let limiter = self.limiter.clone();
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            // Check rate limit before processing request
            if let Err(e) = limiter.check_rate_limit(&client_ip).await {
                match e {
                    RateLimitError::LimitExceeded(wait_time) => {
                        return Err(ErrorBadRequest(format!(
                            "Rate limit exceeded. Try again in {} seconds",
                            wait_time
                        )));
                    }
                    RateLimitError::Redis(e) => {
                        // Log the error but allow the request to proceed
                        // This ensures the API remains available even if Redis is down
                        eprintln!("Redis rate limiting error: {}", e);
                    }
                }
            }

            // Process the request
            fut.await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test,
        web,
        App, HttpResponse,
    };
    use crate::infrastructure::{
        rate_limit::{RedisRateLimiter, RateLimitConfig},
        redis::{RedisConfig, RedisRepository},
    };

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_rate_limit_middleware() {
        // Try to create Redis connection
        let redis = match RedisRepository::new(RedisConfig::default()) {
            Ok(redis) => redis,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        // Check if Redis is actually working
        if redis.get_conn().await.is_err() {
            eprintln!("Skipping test: Redis connection failed");
            return;
        }

        // Try a simple ping command to verify Redis is working properly
        let mut conn = match redis.get_conn().await {
            Ok(conn) => conn,
            Err(_) => {
                eprintln!("Skipping test: Redis connection failed");
                return;
            }
        };
        
        let ping_result: Result<String, redis::RedisError> = redis::cmd("PING")
            .query_async(&mut conn)
            .await;
            
        if ping_result.is_err() {
            eprintln!("Skipping test: Redis PING failed - {}", ping_result.unwrap_err());
            return;
        }

        // Create rate limiter with a very low limit for testing
        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 1,
        };
        let limiter = RedisRateLimiter::new(redis, config);

        // Create test application
        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(limiter))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // First request should succeed
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let _body = test::read_body(resp).await;

        // Second request should succeed
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let _body = test::read_body(resp).await;

        // Third request should fail with 400 Bad Request
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400);
        let _body = test::read_body(resp).await;
    }
}
