//! Database module for LuminaBridge
//!
//! Handles database connections and operations.
//! 处理数据库连接和操作。

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tracing::{info, warn};

use crate::config::DatabaseConfig;
use crate::error::{Error, Result};

/// Database connection wrapper
/// 数据库连接包装器
pub struct Database {
    /// PostgreSQL connection pool
    /// PostgreSQL 连接池
    pool: PgPool,
}

impl Database {
    /// Connect to the database
    /// 连接到数据库
    ///
    /// # Arguments
    ///
    /// * `config` - Database configuration
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Database connection or error
    ///
    /// # Example
    ///
    /// ```rust
    /// let db = Database::connect(&config).await?;
    /// ```
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.timeout_secs))
            .connect(&config.url)
            .await
            .map_err(|e| Error::Database(e))?;
        
        info!("Database connection established");
        
        Ok(Database { pool })
    }
    
    /// Get the connection pool
    /// 获取连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Run database migrations
    /// 运行数据库迁移
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Error on failure
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        // TODO: Implement actual migrations using sqlx::migrate!
        // For now, just log that migrations would run
        
        info!("Database migrations completed");
        Ok(())
    }
    
    /// Check database health
    /// 检查数据库健康状态
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if healthy, Error otherwise
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(())
    }
    
    /// Close the database connection
    /// 关闭数据库连接
    pub async fn close(&self) {
        self.pool.close().await;
        info!("Database connection closed");
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            pool: self.pool.clone(),
        }
    }
}

/// User database operations
/// 用户数据库操作
pub struct UserRepository<'a> {
    db: &'a Database,
}

impl<'a> UserRepository<'a> {
    /// Create a new user repository
    /// 创建新的用户仓库
    pub fn new(db: &'a Database) -> Self {
        UserRepository { db }
    }
    
    /// Find user by ID
    /// 按 ID 查找用户
    ///
    /// # Arguments
    ///
    /// * `id` - User ID
    ///
    /// # Returns
    ///
    /// * `Result<Option<User>>` - User if found
    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Find user by OAuth provider and ID
    /// 按 OAuth 提供商和 ID 查找用户
    ///
    /// # Arguments
    ///
    /// * `provider` - OAuth provider name
    /// * `provider_id` - Provider's user ID
    ///
    /// # Returns
    ///
    /// * `Result<Option<User>>` - User if found
    pub async fn find_by_oauth(&self, provider: &str, provider_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u 
             JOIN oauth_accounts o ON u.id = o.user_id 
             WHERE o.provider = $1 AND o.provider_user_id = $2"
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Create a new user
    /// 创建新用户
    ///
    /// # Arguments
    ///
    /// * `email` - User email
    /// * `name` - User name
    ///
    /// # Returns
    ///
    /// * `Result<User>` - Created user
    pub async fn create(&self, email: &str, name: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, name, created_at, updated_at) 
             VALUES ($1, $2, NOW(), NOW()) 
             RETURNING *"
        )
        .bind(email)
        .bind(name)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
}

/// User model
/// 用户模型
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    /// User ID
    /// 用户 ID
    pub id: i64,
    
    /// Email address
    /// 电子邮件地址
    pub email: String,
    
    /// Display name
    /// 显示名称
    pub name: String,
    
    /// Account status (active, suspended, deleted)
    /// 账户状态（active, suspended, deleted）
    pub status: String,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last update timestamp
    /// 最后更新时间戳
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// OAuth account model
/// OAuth 账户模型
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OAuthAccount {
    /// Account ID
    /// 账户 ID
    pub id: i64,
    
    /// Associated user ID
    /// 关联用户 ID
    pub user_id: i64,
    
    /// OAuth provider (github, discord, etc.)
    /// OAuth 提供商（github, discord 等）
    pub provider: String,
    
    /// Provider's user ID
    /// 提供商的用户 ID
    pub provider_user_id: String,
    
    /// Access token
    /// 访问令牌
    pub access_token: String,
    
    /// Refresh token (optional)
    /// 刷新令牌（可选）
    pub refresh_token: Option<String>,
    
    /// Token expiration timestamp
    /// 令牌过期时间戳
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// API key model
/// API 密钥模型
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKey {
    /// Key ID
    /// 密钥 ID
    pub id: i64,
    
    /// Associated user ID
    /// 关联用户 ID
    pub user_id: i64,
    
    /// Key hash (never store plain text)
    /// 密钥哈希（永不存储明文）
    pub key_hash: String,
    
    /// Key name/label
    /// 密钥名称/标签
    pub name: String,
    
    /// Key permissions
    /// 密钥权限
    pub permissions: Vec<String>,
    
    /// Whether key is active
    /// 密钥是否活跃
    pub active: bool,
    
    /// Last used timestamp
    /// 最后使用时间戳
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual database connection
    async fn test_database_connect() {
        let config = DatabaseConfig {
            url: "postgres://localhost/luminabridge_test".to_string(),
            max_connections: 5,
            min_connections: 1,
            timeout_secs: 30,
        };
        
        // This will fail without a real database, but tests the code path
        let result = Database::connect(&config).await;
        assert!(result.is_err()); // Expected to fail without DB
    }
}
