# Jump Service

A high-performance, Redis-backed temporary payload storage service with rate limiting.

## Features

- Create, retrieve, and delete temporary payloads
- Built-in rate limiting
- Automatic payload expiration
- High performance with Redis backend
- Comprehensive logging
- Detailed error handling

## Quick Start

### Prerequisites

- Rust (latest stable)
- Redis server (6.0 or higher)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/codevalley/jump.git
cd jump
```

2. Start Redis:
```bash
# Using Docker
docker run --name jump-redis -p 6379:6379 -d redis

# Or use your system's Redis
systemctl start redis  # Linux
brew services start redis  # macOS
```

3. Build and run:
```bash
cargo run
```

The service will start on `http://localhost:8080`.

## API Overview

### Health Check
```http
GET /api/health
```

### Create Payload
```http
POST /api/v1/payloads
Content-Type: application/json

{
  "content": "Your content here",
  "mime_type": "text/plain",
  "expiry_time": "2025-03-28T00:00:00Z"  // Optional
}
```

### Get Payload
```http
GET /api/v1/payloads/{hash_id}
```

### Delete Payload
```http
DELETE /api/v1/payloads/{hash_id}
```

For detailed API documentation, see [API.md](docs/API.md).

## Configuration

The service can be configured using environment variables:

```bash
# Server configuration
PORT=8080
HOST=127.0.0.1

# Redis configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=16
REDIS_TIMEOUT=5

# Rate limiting
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60  # seconds

# Payload limits
MAX_PAYLOAD_SIZE=10485760  # 10MB
DEFAULT_EXPIRY=86400      # 24 hours
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test test_api_integration
```

### Code Style

The project follows Rust standard formatting. Format your code using:

```bash
cargo fmt
```

### Logging

The service uses `tracing` for structured logging. Log levels can be configured using the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

## Project Structure

```
src/
├── api/            # HTTP API layer
│   ├── middleware/ # Rate limiting, logging
│   └── v1/        # API version 1 endpoints
├── application/    # Business logic
│   ├── use_cases/ # Core operations
│   └── dto/       # Data transfer objects
├── domain/        # Core domain models
├── infrastructure/ # External services
│   └── redis/     # Redis implementation
└── main.rs        # Application entry point
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [actix-web](https://actix.rs/)
- Storage powered by [Redis](https://redis.io/)
- Logging with [tracing](https://docs.rs/tracing)
