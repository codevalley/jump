# Ephemeral Share - v0 Product Requirements Document

Version: 1.0

Date: August 30, 2024

## 1. Introduction

Ephemeral Share is a simple API service that allows users to temporarily share text or binary data (like images) via a generated URL. The service focuses on ease of use, disposability, and minimal friction. It's designed for quick sharing of information that doesn't require permanent storage. This PRD outlines the requirements for the initial v0 release.

## 2. Goals

- Provide a simple, functional API for sharing ephemeral data.
- Ensure data privacy and security (within the constraints of ephemerality).
- Deliver a fast and reliable service.
- Establish a foundation for future features (like authentication, more granular control over expiry, etc.).
- Keep the v0 scope _extremely_ limited to ensure a quick launch.

## 3. Target Audience

- Developers who need a quick way to share data snippets, logs, or debug information.
- Individuals who want to share temporary information (like one-time passwords, configuration details, or images) without relying on email or messaging platforms that retain data.
- Users who need a simple, no-frills data-sharing solution without account creation.

## 4. Release Criteria (Definition of Done)

- All core features (listed below) are implemented and tested.
- Basic API documentation is complete and publicly available.
- The service is deployed to a stable environment (e.g., a cloud provider like AWS, Google Cloud, or Heroku).
- Basic monitoring and logging are in place to track API usage and errors.
- Rate limiting is implemented and tested.

## 5. Core Features (v0)

- **Data Upload:**
    
    - Users can upload a payload (text or binary data) via a POST request.
    - The API accepts a `content` field (the data itself) and an optional `mime_type` field.
    - The API accepts an optional `expiry_time` field (ISO 8601 format). If not provided, a default expiry time (e.g., 24 hours) is used.
    - The API returns a unique `hash_id` (a short, URL-safe string) that represents the shared data.
    - Maximum payload size: 1MB.
    - Supported MIME types (initial list):
        - `text/plain`
        - `text/html`
        - `application/json`
        - `image/jpeg`
        - `image/png`
        - `image/gif`
    - The API returns standard HTTP status code `201 Created` in case the resource is created successfully.
- **Data Retrieval:**
    
    - Users can retrieve the payload via a GET request using the `hash_id`.
    - The API returns the payload with the correct `Content-Type` header based on the stored `mime_type`.
    - The API updates the `viewed_at` timestamp.
    - The API returns a 404 Not Found error if the `hash_id` is invalid or the data has expired.
- **Data Expiry:**
    
    - Data is automatically deleted after the `expiry_time` is reached.
    - A background process (e.g., a scheduled task or cron job) is responsible for cleaning up expired data.
- **Rate Limiting:**
    
    - Implement basic rate limiting based on the `X-Client-Token` header.
    - Limits:
        - Create: 100 requests/hour
        - Show: 1000 requests/hour
        - Default: 50 requests/hour
    - The API returns a 429 Too Many Requests error when the rate limit is exceeded. The response should include a `Retry-After` header indicating when the client can retry.
- **API Versioning:**
    
    - Use URL-based versioning (e.g., `/api/v1`).

## 6. API Endpoints (v0)

- **Base URL:** `/api/v1`
    
- **1. Add Payload**
    
    - **Method:** `POST`
    - **Endpoint:** `/api/v1/payloads`
    - **Request Header:** `X-Client-Token: <token>` (for rate limiting)
    - **Request Body:**
        
        JSON
        
        ```
        {
          "payload": {
            "content": "Your payload content here",
            "mime_type": "text/plain",
            "expiry_time": "2024-03-14T12:00:00Z"
          }
        }
        ```
        
    - **Response (Success - 201 Created):**
        
        JSON
        
        ```
        {
          "hash_id": "a1b2c3d4e5f6",
          "content": "Your payload content here",
          "mime_type": "text/plain",
          "created_at": "2024-08-30T06:39:26.692Z",
          "updated_at": "2024-08-30T06:39:26.692Z",
          "viewed_at": null,
          "expiry_time": "2024-03-14T12:00:00.000Z"
        }
        ```
        
    - **Response (Error - 422 Unprocessable Entity):**
        
        JSON
        
        ```
        {
          "error": "Invalid payload: content is required"
        }
        ```
        
    - **Response (Error - 429 Too Many Requests):**
        
        JSON
        
        ```
        {
          "error": "Rate limit exceeded",
          "retry_after": 60
        }
        ```
        
        With the response header `Retry-After: 60`
- **2. Get Payload**
    
    - **Method:** `GET`
    - **Endpoint:** `/api/v1/payloads/:hash_id`
    - **Request Header:** `X-Client-Token: <token>` (for rate limiting)
    - **Response (Success - 200 OK):**
        
        JSON
        
        ```
        {
          "hash_id": "a1b2c3d4e5f6",
          "content": "Your payload content here",
          "mime_type": "text/plain",
          "created_at": "2024-08-30T06:39:26.692Z",
          "updated_at": "2024-08-30T06:39:26.692Z",
          "viewed_at": "2024-08-30T07:15:00.000Z",
          "expiry_time": "2024-03-14T12:00:00.000Z"
        }
        ```
        
    - **Response (Error - 404 Not Found):**
        
        JSON
        
        ```
        {
          "error": "Payload not found or expired"
        }
        ```
        
    - **Response (Error - 429 Too Many Requests):** Similar to the POST request.

## 7. Non-Functional Requirements

- **Performance:** The API should respond quickly (target: < 200ms for most requests).
- **Scalability:** The architecture should be designed to handle a reasonable load (initial target: 1000 requests/second).
- **Availability:** The service should aim for high availability (target: 99.9% uptime).
- **Security:**
    - The `hash_id` should be sufficiently random and long to prevent guessing.
    - HTTPS should be enforced for all API communication.
    - Consider a Web Application Firewall (WAF).
- **Maintainability:** The codebase should be well-documented and easy to understand.

## 8. Technology Stack (Suggestions)

- **Language:** Python (with Flask or FastAPI), Node.js (with Express), Go
- **Database:** Redis, PostgreSQL, or a managed NoSQL database (like DynamoDB). _Redis is strongly recommended for v0._
- **Deployment:** AWS (Lambda + API Gateway + DynamoDB/Redis), Google Cloud (Cloud Run + Cloud SQL/Memorystore), Heroku.
- **Monitoring:** Basic logging and a monitoring service (Datadog, New Relic, Prometheus).

## 9. Future Considerations (Out of Scope for v0)

- Authentication (API keys or OAuth 2.0).
- User Accounts.
- Custom Expiry Times.
- One-Time Links.
- Password Protection.
- Web UI.
- File Uploads via Web UI.
- Statistics.
- Webhooks.
- Data encryption at rest.

## 10. Open Issues

- Finalize the technology stack.
- Determine the specific Redis/database schema.
- Define the exact algorithm for generating the `hash_id`.

---