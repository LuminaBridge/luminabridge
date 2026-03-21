//! LuminaBridge - Illuminating AI Connections
//!
//! 下一代高性能 AI 网关，基于 Rust 构建
//! Next-generation high-performance AI gateway built with Rust

mod auth;
mod config;
mod db;
mod error;
mod relay;
mod server;

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::Config;
use crate::db::Database;
use crate::server::Server;

/// Application version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
const APP_NAME: &str = "luminabridge";

/// Main entry point | 主入口点
///
/// # Panics
///
/// Panics if the server fails to start or if configuration is invalid
///
/// # Example
///
/// ```bash
/// cargo run
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging | 初始化日志
    init_logging();

    info!("🌉 {} v{} - Illuminating AI Connections", APP_NAME, VERSION);
    info!("Starting {}...", APP_NAME);

    // Load configuration | 加载配置
    let config = Arc::new(Config::load()?);
    info!("Configuration loaded successfully");

    // Initialize database | 初始化数据库
    let db = Arc::new(Database::connect(&config.database).await?);
    info!("Database connection established");

    // Run database migrations | 运行数据库迁移
    db.migrate().await?;
    info!("Database migrations completed");

    // Create and start server | 创建并启动服务器
    let server = Server::new(config, db);
    
    info!("🚀 Server starting on {}:{}", 
          server.config.server.host, 
          server.config.server.port);
    
    server.run().await?;

    Ok(())
}

/// Initialize logging system
/// 初始化日志系统
///
/// Configures tracing with environment-based log level filtering
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,luminabridge=debug"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_line_number(true))
        .with(filter)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_app_name() {
        assert_eq!(APP_NAME, "luminabridge");
    }
}
