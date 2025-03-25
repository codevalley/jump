**High-Level Development Plan (Rust + Redis + Docker + DDD + Clean Architecture)**

**Phase 1: Project Setup and Core Domain**

1.  **Project Initialization:**
    *   Create a new Rust project: `cargo new ephemeral-share --bin`
    *   Set up the basic project structure (see below).
    *   Add dependencies to `Cargo.toml`: `actix-web`, `redis`, `serde`, `serde_json`, `tokio`, `uuid`, `dotenv`, `chrono`.

2.  **Define the Core Domain (Domain Layer - `domain`):**
    *   **Identify Entities:**
        *   `Payload`: This is your core entity. It represents the shared data.
            *   Attributes: `hash_id` (UUID), `content` (String), `mime_type` (String), `created_at` (DateTime), `updated_at` (DateTime), `viewed_at` (Option<DateTime>), `expiry_time` (DateTime).
    *   **Value Objects:**
        *   Consider using `NaiveDateTime` and `DateTime<Utc>` from the `chrono` crate for date/time representation. This offers better type safety and handling than raw strings.
        *   `MimeType`: Could be a simple wrapper around a String, or an enum if you want to restrict to a specific set of MIME types.
        *   `HashId`: a wrapper around `uuid`
    *   **Domain Services (if needed):**
        *  Initially, you might not need complex domain services. A `PayloadService` could encapsulate logic for generating the `hash_id` and setting the default `expiry_time`.
    *   **Create `domain` module:**  Create a `src/domain` directory.  Within it, create `mod.rs`, `payload.rs`, `mime_type.rs`(if needed), `hash_id.rs`.
        *   `domain/mod.rs`:
            ```rust
            pub mod payload;
            pub mod mime_type; // if needed
            pub mod hash_id;
            ```
        *   `domain/payload.rs`: Define the `Payload` struct and any associated methods (e.g., `new` for creating a new Payload instance).
            ```rust
            use serde::{Deserialize, Serialize};
            use uuid::Uuid;
            use chrono::{DateTime, Utc};
            use super::hash_id::HashId;

            #[derive(Debug, Serialize, Deserialize)]
            pub struct Payload {
                pub hash_id: HashId,
                pub content: String,
                pub mime_type: String,
                pub created_at: DateTime<Utc>,
                pub updated_at: DateTime<Utc>,
                pub viewed_at: Option<DateTime<Utc>>,
                pub expiry_time: DateTime<Utc>,
            }
            // Implement methods for Payload here, e.g.,:
            impl Payload{
                pub fn new(content: String, mime_type: Option<String>, expiry_time: Option<DateTime<Utc>>)-> Self{
                    // ..
                    todo!()
                }
            }
            ```
        * `domain/hash_id.rs`: Define a `HashId` struct.
           ```rust
           use serde::{Deserialize, Serialize};
            use uuid::Uuid;

            #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
            pub struct HashId(String);

            impl HashId {
                pub fn new() -> Self {
                    HashId(Uuid::new_v4().to_string_remove_dashes())
                }

                pub fn as_string(&self) -> &str {
                    &self.0
                }
                pub fn from_string(val: String) -> Self{
                    HashId(val)
                }
            }
            impl Default for HashId{
                fn default() -> Self {
                    Self::new()
                }
            }

           ```

3.  **Define the Application Layer (`application`):**
    *   **Use Cases:** Define the use cases for your application. These are the actions a user can perform.
        *   `CreatePayload`: Takes the payload content, MIME type, and optional expiry time, and creates a new `Payload`.
        *   `GetPayload`: Retrieves a `Payload` by its `hash_id`.
    *   **Create `application` module:** Create a `src/application` directory.  Within it, create `mod.rs`, `dtos.rs`, `use_cases.rs`.
    *    `application/mod.rs`
         ```rust
         pub mod dtos;
         pub mod use_cases;

         ```
    *  `application/dtos.rs`: Create Data Transfer Objects (DTOs) for requests and responses.  This helps decouple your API from your domain model.
        ```rust
        use serde::{Deserialize, Serialize};
        use chrono::{DateTime, Utc};

        #[derive(Deserialize)]
        pub struct CreatePayloadRequest {
            pub content: String,
            pub mime_type: Option<String>,
            pub expiry_time: Option<DateTime<Utc>>,
        }

        #[derive(Serialize)]
        pub struct CreatePayloadResponse {
            pub hash_id: String,
            pub content: String,
            pub mime_type: String,
            pub created_at: DateTime<Utc>,
            pub updated_at: DateTime<Utc>,
            pub viewed_at: Option<DateTime<Utc>>,
            pub expiry_time: DateTime<Utc>,
        }

        #[derive(Serialize)]
        pub struct GetPayloadResponse {
            pub hash_id: String,
            pub content: String,
            pub mime_type: String,
            pub created_at: DateTime<Utc>,
            pub updated_at: DateTime<Utc>,
            pub viewed_at: Option<DateTime<Utc>>,
            pub expiry_time: DateTime<Utc>,
        }
        ```
    *   `application/use_cases.rs`: Implement the use cases. These will use the repository (defined in the next phase) to interact with the data store.
        ```rust
        use crate::domain::payload::Payload;
        use crate::domain::hash_id::HashId;
        use crate::application::dtos::{CreatePayloadRequest, CreatePayloadResponse, GetPayloadResponse};
        use crate::infrastructure::repository::Repository; // You'll define this later
        use async_trait::async_trait;
        use std::sync::Arc;
        use chrono::Utc;


        #[async_trait]
        pub trait CreatePayloadUseCase {
            async fn execute(&self, request: CreatePayloadRequest) -> Result<CreatePayloadResponse, anyhow::Error>;
        }

        #[async_trait]
        pub trait GetPayloadUseCase {
            async fn execute(&self, hash_id: String) -> Result<Option<GetPayloadResponse>, anyhow::Error>;
        }

        pub struct CreatePayloadUseCaseImpl {
            pub repo: Arc<dyn Repository + Send + Sync>, // Dependency Injection
        }

        #[async_trait]
        impl CreatePayloadUseCase for CreatePayloadUseCaseImpl {
            async fn execute(&self, request: CreatePayloadRequest) -> Result<CreatePayloadResponse, anyhow::Error> {
                // 1. Create a new Payload entity.
                let payload = Payload::new(
                    request.content,
                    request.mime_type,
                    request.expiry_time
                );

                // 2. Save the payload using the repository.
                self.repo.save(payload).await?;

                // 3. Create and return the response DTO.

                Ok(CreatePayloadResponse{
                    hash_id: payload.hash_id.as_string().to_string(),
                    content: payload.content,
                    mime_type: payload.mime_type,
                    created_at: payload.created_at,
                    updated_at: payload.updated_at,
                    viewed_at: payload.viewed_at,
                    expiry_time: payload.expiry_time
                }) // Convert to DTO

            }
        }
        pub struct GetPayloadUseCaseImpl{
            pub repo: Arc<dyn Repository + Send + Sync>,
        }

        #[async_trait]
        impl GetPayloadUseCase for GetPayloadUseCaseImpl{
            async fn execute(&self, hash_id: String) -> Result<Option<GetPayloadResponse>, anyhow::Error>{

                let payload_result = self.repo.get(HashId::from_string(hash_id)).await?;
                // update the viewed_at
                if let Some(payload) = payload_result{
                    // 3. Create and return the response DTO.
                    Ok(Some(GetPayloadResponse{
                        hash_id: payload.hash_id.as_string().to_string(),
                        content: payload.content,
                        mime_type: payload.mime_type,
                        created_at: payload.created_at,
                        updated_at: payload.updated_at,
                        viewed_at: payload.viewed_at,
                        expiry_time: payload.expiry_time
                    })) // Convert to DTO
                }else{
                    Ok(None)
                }

            }
        }

        ```

**Phase 2: Infrastructure and Persistence**

1.  **Implement the Repository (Infrastructure Layer - `infrastructure`):**
    *   **Define the `Repository` trait:** This interface defines how the application interacts with the data store (Redis). It *abstracts* the specific database implementation.
        ```rust
        // src/infrastructure/repository.rs
        use crate::domain::payload::Payload;
        use crate::domain::hash_id::HashId;
        use async_trait::async_trait;

        #[async_trait]
        pub trait Repository {
            async fn save(&self, payload: Payload) -> Result<(), anyhow::Error>;
            async fn get(&self, hash_id: HashId) -> Result<Option<Payload>, anyhow::Error>;
        }
        ```
    *   **Create `infrastructure` module:**  Create a `src/infrastructure` directory.  Within it, create `mod.rs`, `repository.rs`, `redis_repository.rs`.
    *  `infrastructure/mod.rs`
        ```rust
        pub mod repository;
        pub mod redis_repository;
        ```
    *   **Implement `RedisRepository`:**  Create a concrete implementation of the `Repository` trait that uses the `redis` crate to interact with Redis. This is where you'll use `set_ex` and `get`.
        ```rust
        // src/infrastructure/redis_repository.rs
        use crate::domain::payload::Payload;
        use crate::domain::hash_id::HashId;
        use crate::infrastructure::repository::Repository;
        use async_trait::async_trait;
        use redis::aio::ConnectionManager;
        use redis::AsyncCommands;
        use std::sync::Arc;

        pub struct RedisRepository {
            pub client: Arc<ConnectionManager>, // Inject the Redis connection manager
        }

        #[async_trait]
        impl Repository for RedisRepository {
            async fn save(&self, payload: Payload) -> Result<(), anyhow::Error> {
                let mut conn = self.client.clone();
                let key = payload.hash_id.as_string();
                let serialized_payload = serde_json::to_string(&payload)?;
                let expiry_seconds = (payload.expiry_time - chrono::Utc::now()).num_seconds();

                conn.set_ex(key, serialized_payload, expiry_seconds as usize).await?;
                Ok(())
            }

            async fn get(&self, hash_id: HashId) -> Result<Option<Payload>, anyhow::Error> {
                let mut conn = self.client.clone();
                let key = hash_id.as_string();
                let result: Option<String> = conn.get(key).await?;

                match result {
                    Some(value) => {
                        let payload: Payload = serde_json::from_str(&value)?;
                        Ok(Some(payload))
                    },
                    None => Ok(None),
                }
            }
        }

        ```
    *   **Dependency Injection:**  The `RedisRepository` takes the Redis `ConnectionManager` as a dependency (injected in the constructor).  This makes it testable (you can mock the `ConnectionManager` in tests).

2.  **Connect Use Cases to Repository:**
      *  In `application/use_cases.rs`, make sure the implementations (e.g. `CreatePayloadUseCaseImpl`) use the `Repository` trait, *not* the concrete `RedisRepository`. This maintains the separation of concerns.  The example code above already shows this.

**Phase 3: API Interface (Presentation Layer) - Continued**

We were in the middle of configuring the main application in `src/main.rs`. Here's the completed `main.rs`, including the missing parts from the previous response:

Rust

```
use actix_web::{web, App, HttpServer};
use redis::aio::ConnectionManager;
use std::env;
use std::sync::Arc;
use dotenv::dotenv;

mod domain;
mod application;
mod infrastructure;
mod api;

use crate::infrastructure::redis_repository::RedisRepository;
use crate::infrastructure::repository::Repository;
use crate::application::use_cases::{CreatePayloadUseCase, CreatePayloadUseCaseImpl, GetPayloadUseCase, GetPayloadUseCaseImpl};
use crate::api::payload_controller::{create_payload_handler, get_payload_handler};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
    let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let redis_url = format!("redis://{}:{}/", redis_host, redis_port);

    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let manager = client.get_tokio_connection_manager().await.expect("Failed to create Redis connection manager");

    // Create the repository (Redis implementation)
    let redis_repo: Arc<dyn Repository + Send + Sync> = Arc::new(RedisRepository {
        client: Arc::new(manager),
    });
    // Create the use cases, injecting the repository
    let create_payload_use_case: Arc<dyn CreatePayloadUseCase + Send + Sync> = Arc::new(CreatePayloadUseCaseImpl { repo: redis_repo.clone() });

    let get_payload_use_case: Arc<dyn GetPayloadUseCase + Send + Sync> = Arc::new(GetPayloadUseCaseImpl{repo: redis_repo.clone()});


    println!("Server running on http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(create_payload_use_case.clone())) // Inject for create_payload
        .app_data(web::Data::new(get_payload_use_case.clone()))// Inject for get_payload
        .service(create_payload_handler) // POST /api/v1/payloads
        .service(get_payload_handler)   // GET /api/v1/payloads/{hash_id}
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

**Explanation of `main.rs`:**

- **Dependencies:** We import all the necessary modules (domain, application, infrastructure, api).
- **Environment Variables:** We use `dotenv` to load environment variables (e.g., `REDIS_HOST`, `REDIS_PORT`) from a `.env` file (for local development). In production, you'd set these environment variables directly on the server.
- **Redis Connection:** We create a Redis client and connection manager.
- **Dependency Injection (Wiring):**
    - We create an instance of `RedisRepository`, passing in the Redis connection manager.
    - We create instances of the use case implementations (`CreatePayloadUseCaseImpl`, `GetPayloadUseCaseImpl`), injecting the `RedisRepository`. We use `Arc` (atomic reference counting) to allow shared ownership of the repository across multiple threads.
    - We use `web::Data` to wrap the use case instances, making them available to the Actix-web handlers.
- **Actix-web Server:**
    - `HttpServer::new`: Creates a new HTTP server.
    - `App::new`: Creates a new Actix-web application.
    - `.app_data(...)`: Registers the use case instances as application data, making them accessible to the handlers via dependency injection.
    - `.service(...)`: Registers the handler functions (`create_payload_handler`, `get_payload_handler`) for the corresponding routes.
    - `.bind(...)`: Binds the server to the specified address and port.
    - `.run().await`: Starts the server and runs it asynchronously.

**Directory Structure (Complete):**

```
ephemeral-share/
├── Cargo.toml
├── Cargo.lock
├── .env  (optional, for local development)
└── src/
    ├── main.rs
    ├── domain/
    │   ├── mod.rs
    │   ├── payload.rs
    │   ├── hash_id.rs
    │   └── mime_type.rs (optional)
    ├── application/
    │   ├── mod.rs
    │   ├── dtos.rs
    │   └── use_cases.rs
    ├── infrastructure/
    │   ├── mod.rs
    │   ├── repository.rs
    │   └── redis_repository.rs
    └── api/
        ├── mod.rs
        └── payload_controller.rs

```

**Phase 4: Dockerization**

1. **Create `Dockerfile`:** (This is the same as in the previous response, but included here for completeness).
    
    Dockerfile
    
    ```
    # Use a Rust base image
    FROM rust:1.72-slim-buster AS builder
    
    # Create a working directory
    WORKDIR /usr/src/ephemeral-share
    
    # Copy the source code
    COPY . .
    
    # Build the application in release mode
    RUN cargo build --release
    
    # Start a new stage for a smaller final image
    FROM debian:buster-slim
    
    # Copy the compiled binary from the builder stage
    COPY --from=builder /usr/src/ephemeral-share/target/release/ephemeral-share /usr/local/bin/ephemeral-share
    
     # Copy .env file.
    COPY --from=builder /usr/src/ephemeral-share/.env /usr/local/bin/.env
    
    # Expose the port
    EXPOSE 8080
    
    # Run the application
    CMD ["ephemeral-share"]
    ```
    
2. **Create `docker-compose.yml`:** (Also the same as before, but here for completeness).
    
    YAML
    
    ```
    version: '3.8'
    services:
      web:
        build: .
        ports:
          - "8080:8080"
        depends_on:
          - redis
        environment:
             REDIS_HOST: redis
             REDIS_PORT: 6379
      redis:
        image: "redis:alpine"
        ports:
          - "6379:6379" # Optional - only if you need external access to Redis
    ```
    
3. **Build and Run:**
    
    - `docker-compose build`
    - `docker-compose up -d`

**Phase 5: Testing and Refinement**

1. **Unit Tests:**
    
    - Write unit tests for your domain logic (e.g., `Payload` methods) and use cases. You can use Rust's built-in testing framework (`#[cfg(test)] mod tests { ... }`).
    - **Mocking:** For the use case tests, you'll want to mock the `Repository` trait. You can use a crate like `mockall` to create mock implementations. This allows you to test the use cases in isolation, without needing a real Redis instance.
2. **Integration Tests:**
    
    - Write integration tests for your API endpoints. You can use `actix-web`'s testing utilities to start a test server and make requests to your API. These tests _will_ interact with a real (or test) Redis instance.
3. **Rate Limiting:**
    
    - Implement rate limiting middleware in your `api` layer. You can use a Redis-backed rate limiter (e.g., store counters in Redis). There are also crates that provide rate-limiting functionality.
4. **Error Handling:**
    
    - Ensure you have robust error handling throughout your application. Use `Result` types and propagate errors appropriately. Return meaningful error responses from your API.
5. **Logging:**
    
    - Add logging to your application (using a crate like `log` or `tracing`). This will help with debugging and monitoring.
6. **Refactoring:**
    
    - Continuously refactor your code to improve its structure, readability, and maintainability.
7. **Add a basic command line app**:
    
    - Add `clap` and create `cli` app.

**Example of Unit Testing (using `mockall`):**

Rust

```
// In application/use_cases.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::payload::Payload;
    use crate::domain::hash_id::HashId;
    use crate::infrastructure::repository::MockRepository; // Import the mock
    use mockall::predicate::*; // Import mockall predicates
    use chrono::Utc;

    #[actix_web::test]
    async fn test_create_payload_success() {
        // Create a mock repository.
        let mut mock_repo = MockRepository::new();

        // Set up expectations for the mock.
        mock_repo.expect_save()
            .withf(|p: &Payload| p.content == "test content") // Check content
            .returning(|_| Ok(())); // Return Ok(()) on save

        // Create the use case with the mocked repository.
        let use_case = CreatePayloadUseCaseImpl {
            repo: Arc::new(mock_repo),
        };

        // Create a request.
        let request = CreatePayloadRequest {
            content: "test content".to_string(),
            mime_type: Some("text/plain".to_string()),
            expiry_time: None,
        };

        // Execute the use case.
        let result = use_case.execute(request).await;

        // Assert that the result is Ok.
        assert!(result.is_ok());

         // you can do more assert, if required.

    }
}
```

This comprehensive plan covers all the major steps, from project setup to testing, using best practices like DDD, Clean Architecture, and dependency injection. Remember to iterate and adapt the plan as you go. This is a solid foundation for building a robust and maintainable Ephemeral Share service in Rust.

