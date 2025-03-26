//! Logging infrastructure for the application.
//!
//! This module provides logging configuration and middleware for the application.
//! It uses the `tracing` crate for structured logging and provides middleware
//! for logging HTTP requests and responses.

use tracing::Level;
use tracing_subscriber::{
    fmt,
    EnvFilter, 
    prelude::*,
};

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// The log level for the application.
    pub level: Level,
    /// Whether to log in JSON format.
    pub json_format: bool,
    /// Whether to log request and response bodies.
    pub log_bodies: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json_format: false,
            log_bodies: false,
        }
    }
}

/// Initialize the logging system with the given configuration.
///
/// # Examples
///
/// ```
/// use jump::infrastructure::logging::{init_logging, LoggingConfig};
/// use tracing::Level;
///
/// let config = LoggingConfig {
///     level: Level::DEBUG,
///     json_format: false,
///     log_bodies: true,
/// };
///
/// init_logging(config);
/// ```
pub fn init_logging(config: LoggingConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(format!("jump={},actix_web=info,actix_server=info", config.level))
        });

    if config.json_format {
        // Setup JSON formatting
        let fmt_layer = fmt::layer()
            .with_target(true)
            .json();
            
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else {
        // Setup regular formatting
        let fmt_layer = fmt::layer()
            .with_target(true);
            
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }
}

mod middleware;

pub use middleware::RequestLogger;
