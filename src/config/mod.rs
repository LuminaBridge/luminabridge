//! Configuration module for LuminaBridge
//!
//! Handles loading and managing application configuration.
//! 处理加载和管理应用程序配置。

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{Error, Result};

/// Main configuration structure
/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    /// 服务器配置
    pub server: ServerConfig,
    
    /// Database configuration
    /// 数据库配置
    pub database: DatabaseConfig,
    
    /// OAuth configuration
    /// OAuth 配置
    pub oauth: OAuthConfig,
    
    /// Cache configuration (Redis)
    /// 缓存配置（Redis）
    pub cache: Option<CacheConfig>,
    
    /// Logging configuration
    /// 日志配置
    pub logging: LoggingConfig,
    
    /// Rate limiting configuration
    /// 速率限制配置
    pub rate_limit: RateLimitConfig,
}

/// Server configuration
/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    /// 绑定主机
    pub host: String,
    
    /// Port to listen on
    /// 监听端口
    pub port: u16,
    
    /// Worker threads (0 = auto)
    /// 工作线程数（0 = 自动）
    pub workers: usize,
    
    /// Request timeout in seconds
    /// 请求超时（秒）
    pub timeout_secs: u64,
    
    /// Maximum connection count
    /// 最大连接数
    pub max_connections: usize,
}

/// Database configuration
/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    /// 数据库 URL
    pub url: String,
    
    /// Maximum connections in pool
    /// 连接池最大连接数
    pub max_connections: u32,
    
    /// Minimum connections in pool
    /// 连接池最小连接数
    pub min_connections: u32,
    
    /// Connection timeout in seconds
    /// 连接超时（秒）
    pub timeout_secs: u64,
}

/// OAuth configuration
/// OAuth 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// GitHub OAuth configuration
    /// GitHub OAuth 配置
    pub github: Option<OAuthProviderConfig>,
    
    /// Discord OAuth configuration
    /// Discord OAuth 配置
    pub discord: Option<OAuthProviderConfig>,
    
    /// JWT secret for token signing
    /// JWT 密钥用于令牌签名
    pub jwt_secret: String,
    
    /// Token expiration in seconds
    /// 令牌过期时间（秒）
    pub token_expiration_secs: u64,
}

/// OAuth provider configuration
/// OAuth 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    /// Client ID
    /// 客户端 ID
    pub client_id: String,
    
    /// Client secret
    /// 客户端密钥
    pub client_secret: String,
    
    /// Redirect URL
    /// 重定向 URL
    pub redirect_url: String,
    
    /// OAuth scopes
    /// OAuth 作用域
    pub scopes: Vec<String>,
}

/// Cache configuration (Redis)
/// 缓存配置（Redis）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis URL
    /// Redis URL
    pub url: String,
    
    /// Maximum connections
    /// 最大连接数
    pub max_connections: u32,
}

/// Logging configuration
/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    /// 日志级别（trace, debug, info, warn, error）
    pub level: String,
    
    /// Log format (json, pretty)
    /// 日志格式（json, pretty）
    pub format: String,
    
    /// Log file path (optional)
    /// 日志文件路径（可选）
    pub file: Option<String>,
}

/// Rate limiting configuration
/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    /// 启用速率限制
    pub enabled: bool,
    
    /// Requests per second
    /// 每秒请求数
    pub requests_per_sec: u32,
    
    /// Burst size
    /// 突发大小
    pub burst_size: u32,
}

impl Config {
    /// Load configuration from file and environment
    /// 从文件和环境加载配置
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to config file (defaults to config/config.yml)
    ///
    /// # Returns
    ///
    /// * `Result<Config>` - Loaded configuration or error
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = Config::load().await?;
    /// ```
    pub fn load() -> Result<Self> {
        // Try to load from config file first
        let config_path = Self::find_config_file();
        
        let config = if let Some(path) = config_path {
            Self::load_from_file(&path)?
        } else {
            Self::load_from_env()?
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from YAML file
    /// 从 YAML 文件加载配置
    fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    }
    
    /// Load configuration from environment variables
    /// 从环境变量加载配置
    fn load_from_env() -> Result<Self> {
        // Use config crate for environment-based loading
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("LUMINABRIDGE").separator("__"))
            .build()
            .map_err(|e| Error::Config(format!("Failed to build config from env: {}", e)))?;
        
        let config: Config = config.try_deserialize()
            .map_err(|e| Error::Config(format!("Failed to deserialize config: {}", e)))?;
        
        Ok(config)
    }
    
    /// Find configuration file
    /// 查找配置文件
    fn find_config_file() -> Option<std::path::PathBuf> {
        let paths = [
            "config/config.yml",
            "config/config.yaml",
            "config.yml",
            "config.yaml",
            ".config/config.yml",
        ];
        
        for path in &paths {
            let p = Path::new(path);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        
        None
    }
    
    /// Validate configuration
    /// 验证配置
    fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(Error::Config("Server port must be greater than 0".to_string()));
        }
        
        // Validate database URL
        if self.database.url.is_empty() {
            return Err(Error::Config("Database URL is required".to_string()));
        }
        
        // Validate JWT secret
        if self.oauth.jwt_secret.len() < 32 {
            return Err(Error::Config("JWT secret must be at least 32 characters".to_string()));
        }
        
        Ok(())
    }
    
    /// Get default configuration for development
    /// 获取开发环境的默认配置
    pub fn development() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                workers: 2,
                timeout_secs: 30,
                max_connections: 100,
            },
            database: DatabaseConfig {
                url: "postgres://localhost/luminabridge_dev".to_string(),
                max_connections: 10,
                min_connections: 2,
                timeout_secs: 30,
            },
            oauth: OAuthConfig {
                github: None,
                discord: None,
                jwt_secret: "dev-secret-key-must-be-changed-in-production-12345678".to_string(),
                token_expiration_secs: 86400, // 24 hours
            },
            cache: None,
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: "pretty".to_string(),
                file: None,
            },
            rate_limit: RateLimitConfig {
                enabled: false,
                requests_per_sec: 100,
                burst_size: 50,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let config = Config::development();
        assert_eq!(config.server.port, 3000);
        assert!(!config.oauth.jwt_secret.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::development();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }
}
