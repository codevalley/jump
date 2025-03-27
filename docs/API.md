# Jump Service API Documentation

## API Version 1

Base URL: `/api/v1`

## Endpoints

### Create Payload

Creates a new temporary payload.

```http
POST /payloads
Content-Type: application/json
```

#### Request Body

```json
{
  "content": "string",
  "mime_type": "string",
  "expiry_time": "2025-03-28T00:00:00Z"  // Optional, ISO 8601 format
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| content | string | Yes | The content to store |
| mime_type | string | Yes | MIME type of the content |
| expiry_time | string | No | When the content should expire (ISO 8601) |

#### Response

##### Success (201 Created)
```json
{
  "hash_id": "string",
  "expiry_time": "2025-03-28T00:00:00Z"
}
```

##### Errors
- 400 Bad Request: Invalid request body or MIME type
- 413 Payload Too Large: Content exceeds size limit
- 429 Too Many Requests: Rate limit exceeded
- 500 Internal Server Error: Server error

### Get Payload

Retrieves a payload by its hash ID.

```http
GET /payloads/{hash_id}
```

#### Parameters

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| hash_id | path | string | Yes | The unique identifier of the payload |

#### Response

##### Success (200 OK)
```json
{
  "content": "string",
  "mime_type": "string",
  "expiry_time": "2025-03-28T00:00:00Z"
}
```

##### Errors
- 404 Not Found: Payload not found or expired
- 429 Too Many Requests: Rate limit exceeded
- 500 Internal Server Error: Server error

### Delete Payload

Deletes a payload by its hash ID.

```http
DELETE /payloads/{hash_id}
```

#### Parameters

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| hash_id | path | string | Yes | The unique identifier of the payload |

#### Response

##### Success (204 No Content)
No response body

##### Errors
- 404 Not Found: Payload not found
- 429 Too Many Requests: Rate limit exceeded
- 500 Internal Server Error: Server error

## Rate Limiting

The API implements rate limiting based on client IP address:

- Default limit: 100 requests per 60 seconds
- Rate limit headers:
  - `X-RateLimit-Limit`: Maximum requests per window
  - `X-RateLimit-Remaining`: Remaining requests in current window
  - `X-RateLimit-Reset`: Time when the rate limit resets (Unix timestamp)

## Error Responses

All error responses follow this format:

```json
{
  "error": "string"  // Human-readable error message
}
```

## Examples

### Creating a Payload

Request:
```bash
curl -X POST http://localhost:8080/api/v1/payloads \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello, World!",
    "mime_type": "text/plain",
    "expiry_time": "2025-03-28T00:00:00Z"
  }'
```

Response:
```json
{
  "hash_id": "feb61cd1b3d34efebab6d6a8490071b2",
  "expiry_time": "2025-03-28T00:00:00Z"
}
```

### Retrieving a Payload

Request:
```bash
curl http://localhost:8080/api/v1/payloads/feb61cd1b3d34efebab6d6a8490071b2
```

Response:
```json
{
  "content": "Hello, World!",
  "mime_type": "text/plain",
  "expiry_time": "2025-03-28T00:00:00Z"
}
```

### Deleting a Payload

Request:
```bash
curl -X DELETE http://localhost:8080/api/v1/payloads/feb61cd1b3d34efebab6d6a8490071b2
```

Response:
```
204 No Content
```
