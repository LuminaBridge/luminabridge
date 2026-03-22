//! LuminaBridge Library
//!
//! Core library for the LuminaBridge AI Gateway.
//! 用于 LuminaBridge AI 网关的核心库。
//!
//! # Example
//!
//! ```rust
//! use luminabridge::config::Config;
//!
//! let config = Config::load().await?;
//! ```

pub mod alerts;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod relay;
pub mod routes;
pub mod server;
pub mod types;

// Re-export commonly used types
pub use error::{Error, Result};
pub use config::Config;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = "luminabridge";
