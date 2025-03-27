//! Integration tests for the API endpoints.

use actix_web::{test, web, App, http::StatusCode};
use std::sync::Arc;
use serde_json::json;

use jump::{
    api::{self, middleware::configure_json_error_handling},
    application::{
        use_cases::{CreatePayloadUseCaseImpl, GetPayloadUseCaseImpl, DeletePayloadUseCaseImpl},
        repository::Repository,
    },
};

// Import the mock repository from our application tests
mod test_utils {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;
    
    use jump::domain::hash_id::HashId;
    use jump::domain::payload::Payload;
    use jump::application::repository::Repository;

    #[derive(Clone)]
    pub struct MockRepository {
        payloads: Arc<Mutex<HashMap<String, Payload>>>,
    }

    impl MockRepository {
        pub fn new() -> Self {
            Self {
                payloads: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn save(&self, payload: &Payload) -> Result<(), anyhow::Error> {
            let mut payloads = self.payloads.lock().unwrap();
            payloads.insert(payload.hash_id().as_string().to_string(), payload.clone());
            Ok(())
        }

        async fn get(&self, id: &HashId) -> Result<Option<Payload>, anyhow::Error> {
            let payloads = self.payloads.lock().unwrap();
            Ok(payloads.get(id.as_string()).cloned())
        }

        async fn delete(&self, id: &HashId) -> Result<(), anyhow::Error> {
            let mut payloads = self.payloads.lock().unwrap();
            payloads.remove(id.as_string());
            Ok(())
        }
    }

    /// Create a test payload with the given content and expiry time.
    pub fn create_test_payload(
        content: &str, 
        expiry_seconds: Option<i64>
    ) -> Payload {
        use chrono::{Duration, Utc};
        
        let expiry = expiry_seconds.map(|secs| Utc::now() + Duration::seconds(secs));
        Payload::new(content.to_string(), Some("text/plain".to_string()), expiry).unwrap()
    }
}

use test_utils::{MockRepository, create_test_payload};

/// Test the health check endpoint.
#[actix_web::test]
async fn test_health_endpoint() {
    // Arrange
    let app = test::init_service(
        App::new()
            .app_data(configure_json_error_handling())
            .configure(api::configure())
    ).await;
    
    // Act
    let req = test::TestRequest::get().uri("/api/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    // Assert
    assert!(resp.status().is_success(), "Health endpoint should return success");
    
    // Parse response body
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "ok", "Health status should be 'ok'");
    assert!(json["version"].is_string(), "Version should be a string");
}

/// Test creating a payload.
#[actix_web::test]
async fn test_create_payload_endpoint() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let create_use_case = Arc::new(CreatePayloadUseCaseImpl::new(repository.clone()));
    let get_use_case = Arc::new(GetPayloadUseCaseImpl::new(repository.clone()));
    let delete_use_case = Arc::new(DeletePayloadUseCaseImpl::new(repository.clone()));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_use_case.clone()))
            .app_data(web::Data::new(get_use_case.clone()))
            .app_data(web::Data::new(delete_use_case.clone()))
            .app_data(configure_json_error_handling())
            .configure(api::configure())
    )
    .await;

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/payloads")
        .set_json(json!({
            "content": "Test content",
            "mime_type": "text/plain",
            "expiry_seconds": 3600
        }))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_object(), "Response body should be a JSON object");
    assert!(body.get("hash_id").is_some(), "Response should contain a hash_id field");
    assert!(body["hash_id"].is_string(), "hash_id should be a string");
}

/// Test getting a payload.
#[actix_web::test]
async fn test_get_payload_endpoint() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let create_use_case = Arc::new(CreatePayloadUseCaseImpl::new(repository.clone()));
    let get_use_case = Arc::new(GetPayloadUseCaseImpl::new(repository.clone()));
    let delete_use_case = Arc::new(DeletePayloadUseCaseImpl::new(repository.clone()));
    
    // Create a test payload and add it to the repository
    let content = "Test content for API get";
    let payload = create_test_payload(content, None);
    let hash_id = payload.hash_id().as_string().to_string();
    repository.save(&payload).await.unwrap();
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_use_case.clone()))
            .app_data(web::Data::new(get_use_case.clone()))
            .app_data(web::Data::new(delete_use_case.clone()))
            .app_data(configure_json_error_handling())
            .configure(api::configure())
    )
    .await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/payloads/{}", hash_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["hash_id"].as_str().unwrap(), hash_id);
    assert_eq!(body["content"].as_str().unwrap(), content);
    assert_eq!(body["mime_type"].as_str().unwrap(), "text/plain");
}

/// Test getting a non-existent payload.
#[actix_web::test]
async fn test_get_nonexistent_payload() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let create_use_case = Arc::new(CreatePayloadUseCaseImpl::new(repository.clone()));
    let get_use_case = Arc::new(GetPayloadUseCaseImpl::new(repository.clone()));
    let delete_use_case = Arc::new(DeletePayloadUseCaseImpl::new(repository.clone()));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_use_case.clone()))
            .app_data(web::Data::new(get_use_case.clone()))
            .app_data(web::Data::new(delete_use_case.clone()))
            .app_data(configure_json_error_handling())
            .configure(api::configure())
    )
    .await;

    // Act - use a random hash ID that doesn't exist
    let req = test::TestRequest::get()
        .uri("/api/v1/payloads/nonexistent")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// Test payload size validation.
#[actix_web::test]
async fn test_payload_size_validation() {
    // Arrange
    let repository = Arc::new(MockRepository::new());
    let create_use_case = Arc::new(CreatePayloadUseCaseImpl::new(repository.clone()));
    let get_use_case = Arc::new(GetPayloadUseCaseImpl::new(repository.clone()));
    let delete_use_case = Arc::new(DeletePayloadUseCaseImpl::new(repository.clone()));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_use_case.clone()))
            .app_data(web::Data::new(get_use_case.clone()))
            .app_data(web::Data::new(delete_use_case.clone()))
            .app_data(configure_json_error_handling())
            .configure(api::configure())
    )
    .await;

    // Create a large payload that exceeds the size limit (10MB + 1 byte)
    let large_content = "a".repeat(10 * 1024 * 1024 + 1);
    
    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/payloads")
        .set_json(json!({
            "content": large_content,
            "mime_type": "text/plain",
            "expiry_seconds": 3600
        }))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Response body: {:?}", body);
    assert!(body.is_object(), "Response body should be a JSON object");
    assert!(
        body.get("error").is_some() || body.get("message").is_some(),
        "Response should contain an error message (either 'error' or 'message' field)"
    );
}

/// Test the delete payload endpoint.
#[actix_web::test]
async fn test_delete_payload_endpoint() {
    // Create a repository and use cases
    let repository = Arc::new(MockRepository::new());
    let create_use_case = Arc::new(CreatePayloadUseCaseImpl::new(repository.clone()));
    let get_use_case = Arc::new(GetPayloadUseCaseImpl::new(repository.clone()));
    let delete_use_case = Arc::new(DeletePayloadUseCaseImpl::new(repository.clone()));
    
    // Create a test payload and add it to the repository
    let payload = create_test_payload("Content to delete", None);
    let hash_id = payload.hash_id().as_string().to_string();
    repository.save(&payload).await.unwrap();
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_use_case.clone()))
            .app_data(web::Data::new(get_use_case.clone()))
            .app_data(web::Data::new(delete_use_case.clone()))
            .app_data(configure_json_error_handling())
            .configure(api::configure())
    )
    .await;
    
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/payloads/{}", hash_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
