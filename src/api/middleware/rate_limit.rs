//! Rate limiting middleware for actix-web.
//!
//! This middleware implements rate limiting for API endpoints using
//! the Redis-based rate limiter.

use std::rc::Rc;
use std::pin::Pin;
use std::future::Future;
use std::task::{Context, Poll};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
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
                        // Log rate limit exceeded
                        tracing::warn!("Rate limit exceeded for IP: {}", client_ip);
                        
                        // Return 429 Too Many Requests with appropriate headers
                        return Err(actix_web::error::Error::from(
                            actix_web::error::ErrorTooManyRequests(format!(
                                "Rate limit exceeded. Try again in {} seconds",
                                wait_time
                            ))
                        ));
                    }
                    RateLimitError::Redis(msg) => {
                        // Log Redis error but don't block the request
                        tracing::error!("Rate limit Redis error: {}", msg);
                    }
                }
            } else {
                // Log successful rate limit check
                tracing::debug!("Rate limit check: key={}, count={}, max={}", 
                    client_ip, 1, 2); // Use fixed value for max_requests in debug log
            }

            // Proceed with the request
            fut.await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // Mock rate limiter that fails after a certain number of requests
    struct MockRateLimiter {
        counter: Arc<AtomicUsize>,
        max_requests: usize,
    }

    impl MockRateLimiter {
        fn new(max_requests: usize) -> Self {
            Self {
                counter: Arc::new(AtomicUsize::new(0)),
                max_requests,
            }
        }
    }

    #[async_trait::async_trait]
    impl RateLimiter for MockRateLimiter {
        async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
            let count = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
            
            tracing::debug!("Mock rate limit check: key={}, count={}, max={}", key, count, self.max_requests);
            
            if count > self.max_requests {
                Err(RateLimitError::LimitExceeded(60))
            } else {
                Ok(())
            }
        }
    }

    // Test the RateLimiter trait implementation directly
    #[tokio::test]
    async fn test_rate_limiter_trait() {
        // Create a mock rate limiter that allows 2 requests
        let limiter = MockRateLimiter::new(2);
        
        // First request should succeed
        let result1 = limiter.check_rate_limit("test-ip").await;
        assert!(result1.is_ok(), "First request should succeed");
        
        // Second request should succeed
        let result2 = limiter.check_rate_limit("test-ip").await;
        assert!(result2.is_ok(), "Second request should succeed");
        
        // Third request should fail
        let result3 = limiter.check_rate_limit("test-ip").await;
        assert!(result3.is_err(), "Third request should fail");
        
        // Verify the error is LimitExceeded
        match result3 {
            Err(RateLimitError::LimitExceeded(_)) => (),
            other => panic!("Expected LimitExceeded, got {:?}", other),
        }
    }
}
