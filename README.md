# Jump Service

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/rust-lang/rust/workflows/CI/badge.svg)](https://github.com/rust-lang/rust/actions)

A high-performance, Redis-backed temporary payload storage service with built-in rate limiting and automatic expiration. Built with Rust for optimal performance and reliability.

<p align="center">
  <img src="docs/assets/jump-logo.png" alt="Jump Service Logo" width="200"/>
</p>

## What is Jump?

Jump is a modern solution to ephemeral data storage and sharing. It provides a secure, fast, and reliable way to store temporary data that automatically expires after a set duration. Whether you need to share sensitive information, temporary tokens, or any other short-lived data, Jump makes it simple and secure.

### Why Jump?

- **High Performance**: Built with Rust for optimal speed and efficiency
- **Secure**: Data automatically expires and is never permanently stored
- **Lightning Fast**: Redis-backed storage for sub-millisecond access
- **Rate Limited**: Built-in protection against abuse
- **Type Safe**: Strict MIME type validation for content
- **RESTful API**: Simple and intuitive API design

## How Does It Work?

1. **Create**: Send your data with an optional expiry time
2. **Share**: Get a unique hash ID for your data
3. **Access**: Use the hash ID to retrieve the data
4. **Auto-Expire**: Data is automatically deleted after expiry

Jump is:
- **Ephemeral**: All data automatically expires
- **Fast**: Redis-backed for high performance
- **Efficient**: Optimized for minimal resource usage

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

[View Full API Documentation](docs/API.md)

## Configuration

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
