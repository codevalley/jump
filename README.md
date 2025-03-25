# Jump - Ephemeral Share Service

Jump is a secure and efficient ephemeral sharing service built with Rust. It allows users to share content that automatically expires after a set duration, ensuring data privacy and security.

## Features

- **Ephemeral Sharing**: Content automatically expires after a configurable duration
- **Secure**: Content is encrypted and only accessible via unique hash IDs
- **Rate Limited**: Built-in protection against abuse
- **Type Safe**: Strict MIME type validation for content
- **Fast**: Built with Rust for optimal performance
- **Simple API**: RESTful API for easy integration

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Redis 6.0 or higher
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/jump.git
   cd jump
   ```

2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

3. Update the environment variables in `.env` with your configuration.

4. Build the project:
   ```bash
   cargo build --release
   ```

5. Run the server:
   ```bash
   cargo run --release
   ```

## Usage

### Creating a Share

```bash
curl -X POST http://localhost:8080/api/v1/payloads \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Your content here",
    "mime_type": "text/plain",
    "expiry_time": "2024-03-14T12:00:00Z"
  }'
```

### Retrieving a Share

```bash
curl http://localhost:8080/api/v1/payloads/{hash_id}
```

## Configuration

The service can be configured using environment variables:

- `SERVER_HOST`: Host to bind the server to (default: "127.0.0.1")
- `SERVER_PORT`: Port to listen on (default: 8080)
- `REDIS_URL`: Redis connection URL
- `RATE_LIMIT_REQUESTS`: Number of requests allowed per window
- `RATE_LIMIT_WINDOW_SECS`: Rate limit window in seconds
- `MAX_PAYLOAD_SIZE`: Maximum payload size in bytes
- `DEFAULT_EXPIRY_HOURS`: Default expiry time in hours

## Development

### Running Tests

```bash
cargo test
```

### Documentation

Generate and view the documentation:

```bash
cargo doc --open
```

## API Documentation

The API documentation is available at `/api/docs` when running the server.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
