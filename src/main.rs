use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;
use tracing::info;

use jump::{
    api::{self, middleware::{ErrorHandlerMiddleware, RateLimitMiddleware, configure_json_error_handling}},
    application::{
        repository::Repository,
        use_cases::{
            CreatePayloadUseCaseImpl,
            GetPayloadUseCaseImpl,
            DeletePayloadUseCaseImpl,
        },
    },
    infrastructure::{
        redis::{RedisConfig, RedisRepository},
        rate_limit::{RateLimitConfig, RedisRateLimiter},
        logging::{LoggingConfig, init_logging, RequestLogger},
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    init_logging(LoggingConfig::default());
    
    info!("Starting Jump service");
    
    // Configure Redis
    let redis_config = RedisConfig::default();
    let redis_repo = match RedisRepository::new(redis_config.clone()) {
        Ok(repo) => {
            info!("Connected to Redis");
            repo
        },
        Err(e) => {
            panic!("Failed to connect to Redis: {}", e);
        }
    };
    
    // Disable Redis persistence error checking for development
    if let Err(e) = redis_repo.disable_stop_writes_on_bgsave_error().await {
        info!("Warning: Could not disable Redis persistence error checking: {}", e);
        info!("Some write operations may fail if Redis cannot persist to disk");
    } else {
        info!("Disabled Redis persistence error checking for development");
    }
    
    // Create repository
    let repository: Arc<dyn Repository> = Arc::new(redis_repo.clone());
    
    // Create use cases
    let create_payload_use_case = Arc::new(CreatePayloadUseCaseImpl::new(repository.clone()));
    let get_payload_use_case = Arc::new(GetPayloadUseCaseImpl::new(repository.clone()));
    let delete_payload_use_case = Arc::new(DeletePayloadUseCaseImpl::new(repository.clone()));
    
    // Configure rate limiter
    let rate_limit_config = RateLimitConfig {
        max_requests: 100,
        window_seconds: 60,
    };
    let rate_limiter = RedisRateLimiter::new(redis_repo, rate_limit_config);
    
    // Start HTTP server
    info!("Starting HTTP server on 127.0.0.1:8080");
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            // Add middlewares
            .wrap(ErrorHandlerMiddleware::new())
            .wrap(RequestLogger::new(false)) // Don't log request bodies
            .wrap(RateLimitMiddleware::new(rate_limiter.clone()))
            .wrap(cors)
            // Configure JSON handling
            .app_data(configure_json_error_handling())
            // Add application state
            .app_data(web::Data::new(create_payload_use_case.clone()))
            .app_data(web::Data::new(get_payload_use_case.clone()))
            .app_data(web::Data::new(delete_payload_use_case.clone()))
            // Add API routes
            .configure(api::configure())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
