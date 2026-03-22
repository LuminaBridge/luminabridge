//! Database module for LuminaBridge
//!
//! Handles database connections, models, and operations.
//! 处理数据库连接、模型和操作。

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};
use serde::Deserialize;

use crate::config::DatabaseConfig;
use crate::error::{Error, Result};
use crate::types::{RealtimeStats, PaginationParams, CreateChannelRequest, UpdateUserRequest, UpdateTenantRequest, CreateTokenRequest};
use chrono::{DateTime, Utc};

pub use models::*;

/// Database models
/// 数据库模型
pub mod models;

/// Database connection wrapper
/// 数据库连接包装器
pub struct Database {
    /// PostgreSQL connection pool
    /// PostgreSQL 连接池
    pool: PgPool,
    
    /// Broadcast sender for realtime stats
    /// 实时统计广播发送器
    pub stats_sender: broadcast::Sender<RealtimeStats>,
}

impl Database {
    /// Connect to the database
    /// 连接到数据库
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.timeout_secs))
            .connect(&config.url)
            .await
            .map_err(|e| Error::Database(e))?;
        
        // Create broadcast channel for stats (capacity: 1000)
        let (stats_sender, _) = broadcast::channel(1000);
        
        info!("Database connection established");
        
        Ok(Database { pool, stats_sender })
    }
    
    /// Get the connection pool
    /// 获取连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Run database migrations
    /// 运行数据库迁移
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        // Create tables if not exists (for development)
        self.create_tables().await?;
        
        info!("Database migrations completed");
        Ok(())
    }
    
    /// Create tables (for development/SQLite fallback)
    /// 创建表（用于开发/SQLite 回退）
    async fn create_tables(&self) -> Result<()> {
        // Tenants table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tenants (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                slug VARCHAR(100) UNIQUE NOT NULL,
                status VARCHAR(20) DEFAULT 'active',
                quota_limit BIGINT DEFAULT 0,
                quota_used BIGINT DEFAULT 0,
                settings JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;
        
        // Users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id BIGSERIAL PRIMARY KEY,
                tenant_id BIGINT REFERENCES tenants(id),
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255),
                display_name VARCHAR(255),
                avatar_url VARCHAR(500),
                role VARCHAR(50) DEFAULT 'user',
                status VARCHAR(20) DEFAULT 'active',
                oauth_provider VARCHAR(50),
                oauth_id VARCHAR(255),
                last_login_at TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;
        
        // Channels table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS channels (
                id BIGSERIAL PRIMARY KEY,
                tenant_id BIGINT REFERENCES tenants(id),
                name VARCHAR(255) NOT NULL,
                channel_type VARCHAR(50) NOT NULL,
                key TEXT NOT NULL,
                base_url VARCHAR(500),
                models JSONB DEFAULT '[]',
                weight INT DEFAULT 10,
                status VARCHAR(20) DEFAULT 'active',
                priority INT DEFAULT 0,
                timeout_ms INT DEFAULT 30000,
                retry_count INT DEFAULT 3,
                balance DECIMAL(20, 6) DEFAULT 0,
                last_test_at TIMESTAMP,
                last_test_status VARCHAR(50),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;
        
        // Tokens table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tokens (
                id BIGSERIAL PRIMARY KEY,
                tenant_id BIGINT REFERENCES tenants(id),
                user_id BIGINT REFERENCES users(id),
                key VARCHAR(255) UNIQUE NOT NULL,
                name VARCHAR(255),
                quota_limit BIGINT DEFAULT 0,
                quota_used BIGINT DEFAULT 0,
                expire_at TIMESTAMP,
                status VARCHAR(20) DEFAULT 'active',
                allowed_ips JSONB DEFAULT '[]',
                allowed_models JSONB DEFAULT '[]',
                last_used_at TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;
        
        // Usage stats table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS usage_stats (
                id BIGSERIAL PRIMARY KEY,
                tenant_id BIGINT REFERENCES tenants(id),
                user_id BIGINT REFERENCES users(id),
                channel_id BIGINT REFERENCES channels(id),
                model VARCHAR(100),
                prompt_tokens BIGINT DEFAULT 0,
                completion_tokens BIGINT DEFAULT 0,
                total_tokens BIGINT DEFAULT 0,
                cost DECIMAL(20, 6) DEFAULT 0,
                status VARCHAR(20),
                latency_ms INT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;
        
        // OAuth accounts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS oauth_accounts (
                id BIGSERIAL PRIMARY KEY,
                user_id BIGINT REFERENCES users(id),
                provider VARCHAR(50) NOT NULL,
                provider_user_id VARCHAR(255) NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                expires_at TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(provider, provider_user_id)
            )
            "#
        ).execute(&self.pool).await?;
        
        // Alerts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alerts (
                id BIGSERIAL PRIMARY KEY,
                tenant_id BIGINT REFERENCES tenants(id),
                level VARCHAR(20) NOT NULL,
                alert_type VARCHAR(50) NOT NULL,
                message TEXT NOT NULL,
                entity_id BIGINT,
                entity_type VARCHAR(50),
                is_resolved BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                resolved_at TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_tenant_id ON alerts(tenant_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_is_resolved ON alerts(is_resolved)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_created_at ON alerts(created_at)").execute(&self.pool).await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_channels_tenant_id ON channels(tenant_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tokens_tenant_id ON tokens(tenant_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_stats_tenant_id ON usage_stats(tenant_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_stats_created_at ON usage_stats(created_at)").execute(&self.pool).await?;
        
        // Insert default tenant if not exists
        sqlx::query(
            r#"
            INSERT INTO tenants (id, name, slug, status, quota_limit, quota_used)
            VALUES (1, 'Default Tenant', 'default', 'active', 0, 0)
            ON CONFLICT (id) DO NOTHING
            "#
        ).execute(&self.pool).await?;
        
        Ok(())
    }
    
    /// Check database health
    /// 检查数据库健康状态
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
            stats_sender: self.stats_sender.clone(),
        }
    }
}

// ============================================================================
// User Repository Methods
// ============================================================================

impl Database {
    /// Find user by ID
    /// 按 ID 查找用户
    pub async fn find_user(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Find user by email
    /// 按电子邮件查找用户
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Find user by OAuth provider and ID
    /// 按 OAuth 提供商和 ID 查找用户
    pub async fn find_user_by_oauth(&self, provider: &str, provider_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT u.* FROM users u 
               JOIN oauth_accounts o ON u.id = o.user_id 
               WHERE o.provider = $1 AND o.provider_user_id = $2"#
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Find users by tenant
    /// 按租户查找用户
    pub async fn find_users_by_tenant(&self, tenant_id: i64, params: &UserListParams) -> Result<Vec<User>> {
        let offset = params.pagination.offset();
        let limit = params.pagination.limit();
        
        let mut query = String::from("SELECT * FROM users WHERE tenant_id = $1");
        let mut query_args = sqlx::query::QueryArguments::default();
        query_args.push(tenant_id);
        
        if let Some(ref status) = params.status {
            query.push_str(&format!(" AND status = ${}", query_args.len() + 1));
            query_args.push(status.clone());
        }
        
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", 
            query_args.len() + 1, query_args.len() + 2));
        query_args.push(limit);
        query_args.push(offset);
        
        let users = sqlx::query_as_with::<_, User, _>(&query, query_args)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(users)
    }
    
    /// Count users by tenant
    /// 按租户计数用户
    pub async fn count_users(&self, tenant_id: i64, params: &UserListParams) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM users WHERE tenant_id = $1");
        
        if let Some(ref status) = params.status {
            query.push_str(&format!(" AND status = $2"));
            let count: i64 = sqlx::query_scalar(&query)
                .bind(tenant_id)
                .bind(status)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::Database(e))?;
            return Ok(count);
        }
        
        let count: i64 = sqlx::query_scalar(&query)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(count)
    }
    
    /// Create user with password
    /// 创建带密码的用户
    pub async fn create_with_password(&self, email: &str, display_name: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (tenant_id, email, password_hash, display_name, role, status, created_at, updated_at)
               VALUES (1, $1, $2, $3, 'user', 'active', NOW(), NOW())
               RETURNING *"#
        )
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Create user with OAuth
    /// 使用 OAuth 创建用户
    pub async fn create_with_oauth(
        &self,
        email: &str,
        display_name: &str,
        provider: &str,
        provider_id: &str,
        access_token: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (tenant_id, email, display_name, oauth_provider, oauth_id, status, created_at, updated_at)
               VALUES (1, $1, $2, $3, $4, 'active', NOW(), NOW())
               RETURNING *"#
        )
        .bind(email)
        .bind(display_name)
        .bind(provider)
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        // Link OAuth account
        sqlx::query(
            r#"INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token, created_at)
               VALUES ($1, $2, $3, $4, NOW())"#
        )
        .bind(user.id)
        .bind(provider)
        .bind(provider_id)
        .bind(access_token)
        .execute(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Link OAuth account to existing user
    /// 将 OAuth 账户链接到现有用户
    pub async fn link_oauth_account(
        &self,
        user_id: i64,
        provider: &str,
        provider_id: &str,
        access_token: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token, created_at)
               VALUES ($1, $2, $3, $4, NOW())
               ON CONFLICT (provider, provider_user_id) DO UPDATE SET access_token = $4"#
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_id)
        .bind(access_token)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Update user
    /// 更新用户
    pub async fn update_user(&self, user_id: i64, payload: &UpdateUserRequest) -> Result<User> {
        let mut updates = Vec::new();
        if payload.display_name.is_some() {
            updates.push("display_name = COALESCE($2, display_name)");
        }
        if payload.avatar_url.is_some() {
            updates.push("avatar_url = COALESCE($3, avatar_url)");
        }
        if payload.role.is_some() {
            updates.push("role = COALESCE($4, role)");
        }
        if payload.status.is_some() {
            updates.push("status = COALESCE($5, status)");
        }
        updates.push("updated_at = NOW()");
        
        let query = format!(
            "UPDATE users SET {} WHERE id = $1 RETURNING *",
            updates.join(", ")
        );
        
        let user = sqlx::query_as::<_, User>(&query)
            .bind(user_id)
            .bind(&payload.display_name)
            .bind(&payload.avatar_url)
            .bind(&payload.role)
            .bind(&payload.status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(user)
    }
    
    /// Delete user
    /// 删除用户
    pub async fn delete_user(&self, user_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Get user usage stats
    /// 获取用户用量统计
    pub async fn get_user_usage_stats(&self, user_id: i64) -> Result<crate::routes::users::UserUsageStats> {
        let total_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM usage_stats WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let total_tokens: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM usage_stats WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let total_cost: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(cost), 0) FROM usage_stats WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(crate::routes::users::UserUsageStats {
            total_requests,
            total_tokens,
            total_cost,
            requests_by_model: vec![],
        })
    }
}

// User list params (moved from routes to avoid circular dependency)
#[derive(Debug, Deserialize)]
pub struct UserListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub status: Option<String>,
    pub role: Option<String>,
}

// ============================================================================
// Tenant Repository Methods
// ============================================================================

impl Database {
    /// Find tenant by ID
    /// 按 ID 查找租户
    pub async fn find_tenant(&self, id: i64) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(tenant)
    }
    
    /// Update tenant
    /// 更新租户
    pub async fn update_tenant(&self, tenant_id: i64, payload: &UpdateTenantRequest) -> Result<Tenant> {
        let mut updates = Vec::new();
        if payload.name.is_some() {
            updates.push("name = COALESCE($2, name)");
        }
        if payload.settings.is_some() {
            updates.push("settings = COALESCE($3, settings)");
        }
        updates.push("updated_at = NOW()");
        
        let query = format!(
            "UPDATE tenants SET {} WHERE id = $1 RETURNING *",
            updates.join(", ")
        );
        
        let tenant = sqlx::query_as::<_, Tenant>(&query)
            .bind(tenant_id)
            .bind(&payload.name)
            .bind(&payload.settings)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(tenant)
    }
    
    /// Get tenant usage stats
    /// 获取租户用量统计
    pub async fn get_tenant_usage_stats(&self, tenant_id: i64) -> Result<crate::routes::tenant::TenantUsageStats> {
        let quota_limit: Option<i64> = sqlx::query_scalar(
            "SELECT quota_limit FROM tenants WHERE id = $1"
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten();
        
        let quota_used: i64 = sqlx::query_scalar(
            "SELECT quota_used FROM tenants WHERE id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let total_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM usage_stats WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let total_tokens: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM usage_stats WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let total_cost: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(cost), 0) FROM usage_stats WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let quota_remaining = quota_limit.map(|l| l - quota_used);
        let usage_percentage = if let Some(limit) = quota_limit {
            if limit > 0 { (quota_used as f64 / limit as f64) * 100.0 } else { 0.0 }
        } else {
            0.0
        };
        
        Ok(crate::routes::tenant::TenantUsageStats {
            period_start: Utc::now(),
            period_end: Utc::now(),
            quota_limit,
            quota_used,
            quota_remaining,
            usage_percentage,
            total_requests,
            total_tokens,
            total_cost,
        })
    }
    
    /// Find tenant members
    /// 查找租户成员
    pub async fn find_tenant_members(&self, tenant_id: i64) -> Result<Vec<crate::routes::tenant::TenantMember>> {
        let members = sqlx::query_as::<_, crate::routes::tenant::TenantMember>(
            "SELECT id as user_id, email, display_name, role, status, created_at as joined_at 
             FROM users WHERE tenant_id = $1 ORDER BY created_at"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(members)
    }
}

// ============================================================================
// Channel Repository Methods
// ============================================================================

impl Database {
    /// Find channel by ID
    /// 按 ID 查找渠道
    pub async fn find_channel(&self, id: i64) -> Result<Option<Channel>> {
        let channel = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(channel)
    }
    
    /// Find channels by tenant
    /// 按租户查找渠道
    pub async fn find_channels_by_tenant(&self, tenant_id: i64, params: &crate::routes::channels::ChannelListParams) -> Result<Vec<Channel>> {
        let offset = params.pagination.offset();
        let limit = params.pagination.limit();
        
        let mut query = String::from("SELECT * FROM channels WHERE tenant_id = $1");
        let mut args: Vec<sqlx::postgres::PgArgument> = vec![];
        
        // Build dynamic query (simplified version)
        let channels = sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(channels)
    }
    
    /// Count channels by tenant
    /// 按租户计数渠道
    pub async fn count_channels(&self, tenant_id: i64, _params: &crate::routes::channels::ChannelListParams) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM channels WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(count)
    }
    
    /// Check if channel exists
    /// 检查渠道是否存在
    pub async fn channel_exists(&self, tenant_id: i64, name: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channels WHERE tenant_id = $1 AND name = $2)"
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(exists)
    }
    
    /// Create channel
    /// 创建渠道
    pub async fn create_channel(&self, tenant_id: i64, payload: &CreateChannelRequest) -> Result<Channel> {
        let models_json = serde_json::to_value(&payload.models).unwrap_or(serde_json::json!([]));
        
        let channel = sqlx::query_as::<_, Channel>(
            r#"INSERT INTO channels 
               (tenant_id, name, channel_type, key, base_url, models, weight, status, priority, timeout_ms, retry_count, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, $9, $10, NOW(), NOW())
               RETURNING *"#
        )
        .bind(tenant_id)
        .bind(&payload.name)
        .bind(&payload.channel_type)
        .bind(&payload.key)
        .bind(&payload.base_url)
        .bind(models_json)
        .bind(payload.weight)
        .bind(payload.priority)
        .bind(payload.timeout_ms)
        .bind(payload.retry_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(channel)
    }
    
    /// Update channel
    /// 更新渠道
    pub async fn update_channel(&self, channel_id: i64, payload: &crate::routes::channels::UpdateChannelRequest) -> Result<Channel> {
        let mut updates = Vec::new();
        let mut idx = 2;
        
        if payload.name.is_some() {
            updates.push(format!("name = COALESCE(${idx}, name)"));
            idx += 1;
        }
        if payload.key.is_some() {
            updates.push(format!("key = COALESCE(${idx}, key)"));
            idx += 1;
        }
        if payload.base_url.is_some() {
            updates.push(format!("base_url = COALESCE(${idx}, base_url)"));
            idx += 1;
        }
        if payload.models.is_some() {
            updates.push(format!("models = COALESCE(${idx}, models)"));
            idx += 1;
        }
        if payload.weight.is_some() {
            updates.push(format!("weight = COALESCE(${idx}, weight)"));
            idx += 1;
        }
        if payload.priority.is_some() {
            updates.push(format!("priority = COALESCE(${idx}, priority)"));
            idx += 1;
        }
        if payload.timeout_ms.is_some() {
            updates.push(format!("timeout_ms = COALESCE(${idx}, timeout_ms)"));
            idx += 1;
        }
        if payload.retry_count.is_some() {
            updates.push(format!("retry_count = COALESCE(${idx}, retry_count)"));
            idx += 1;
        }
        if payload.status.is_some() {
            updates.push(format!("status = COALESCE(${idx}, status)"));
            idx += 1;
        }
        updates.push("updated_at = NOW()".to_string());
        
        let query = format!(
            "UPDATE channels SET {} WHERE id = $1 RETURNING *",
            updates.join(", ")
        );
        
        let mut q = sqlx::query_as::<_, Channel>(&query).bind(channel_id);
        
        if let Some(ref name) = payload.name { q = q.bind(name); }
        if let Some(ref key) = payload.key { q = q.bind(key); }
        if let Some(ref base_url) = payload.base_url { q = q.bind(base_url); }
        if let Some(ref models) = payload.models { q = q.bind(serde_json::to_value(models).unwrap()); }
        if let Some(weight) = payload.weight { q = q.bind(weight); }
        if let Some(priority) = payload.priority { q = q.bind(priority); }
        if let Some(timeout_ms) = payload.timeout_ms { q = q.bind(timeout_ms); }
        if let Some(retry_count) = payload.retry_count { q = q.bind(retry_count); }
        if let Some(ref status) = payload.status { q = q.bind(status); }
        
        let channel = q.fetch_one(&self.pool).await.map_err(|e| Error::Database(e))?;
        
        Ok(channel)
    }
    
    /// Delete channel
    /// 删除渠道
    pub async fn delete_channel(&self, channel_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM channels WHERE id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Set channel status
    /// 设置渠道状态
    pub async fn set_channel_status(&self, channel_id: i64, status: &str) -> Result<Channel> {
        let channel = sqlx::query_as::<_, Channel>(
            "UPDATE channels SET status = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(channel_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(channel)
    }
    
    /// Update channel test info
    /// 更新渠道测试信息
    pub async fn update_channel_test_info(&self, channel_id: i64, response: &crate::routes::channels::TestChannelResponse) -> Result<()> {
        sqlx::query(
            "UPDATE channels SET last_test_at = NOW(), last_test_status = $2 WHERE id = $1"
        )
        .bind(channel_id)
        .bind(&response.message)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Get channels for a specific model (for relay channel selection)
    /// 获取特定模型的渠道（用于中继渠道选择）
    pub async fn get_channels_for_model(&self, tenant_id: i64, model: &str) -> Result<Vec<Channel>> {
        let channels = sqlx::query_as::<_, Channel>(
            r#"SELECT * FROM channels 
               WHERE tenant_id = $1 
               AND status = 'active'
               AND models @> $2
               ORDER BY priority DESC, weight DESC, created_at ASC"#
        )
        .bind(tenant_id)
        .bind(serde_json::json!([model]))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(channels)
    }
    
    /// Get all active channels for a tenant
    /// 获取租户的所有活跃渠道
    pub async fn get_active_channels(&self, tenant_id: i64) -> Result<Vec<Channel>> {
        let channels = sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE tenant_id = $1 AND status = 'active' ORDER BY created_at"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(channels)
    }
    
    /// Create usage stat record
    /// 创建用量统计记录
    pub async fn create_usage_stat(
        &self,
        tenant_id: i64,
        user_id: Option<i64>,
        channel_id: Option<i64>,
        model: Option<String>,
        prompt_tokens: i64,
        completion_tokens: i64,
        total_tokens: i64,
        cost: rust_decimal::Decimal,
        status: Option<String>,
        latency_ms: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO usage_stats 
               (tenant_id, user_id, channel_id, model, prompt_tokens, completion_tokens, total_tokens, cost, status, latency_ms, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"#
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(channel_id)
        .bind(model)
        .bind(prompt_tokens)
        .bind(completion_tokens)
        .bind(total_tokens)
        .bind(cost)
        .bind(status)
        .bind(latency_ms)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Token Repository Methods
// ============================================================================

impl Database {
    /// Find token by key
    /// 按密钥查找令牌
    pub async fn find_token_by_key(&self, key: &str) -> Result<Option<Token>> {
        let token = sqlx::query_as::<_, Token>("SELECT * FROM tokens WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(token)
    }
    
    /// Find token by ID
    /// 按 ID 查找令牌
    pub async fn find_token(&self, id: i64) -> Result<Option<Token>> {
        let token = sqlx::query_as::<_, Token>("SELECT * FROM tokens WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(token)
    }
    
    /// Find tokens by tenant
    /// 按租户查找令牌
    pub async fn find_tokens_by_tenant(&self, tenant_id: i64, _params: &crate::routes::tokens::TokenListParams) -> Result<Vec<Token>> {
        let tokens = sqlx::query_as::<_, Token>(
            "SELECT * FROM tokens WHERE tenant_id = $1 ORDER BY created_at DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(tokens)
    }
    
    /// Count tokens by tenant
    /// 按租户计数令牌
    pub async fn count_tokens(&self, tenant_id: i64, _params: &crate::routes::tokens::TokenListParams) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tokens WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(count)
    }
    
    /// Create token
    /// 创建令牌
    pub async fn create_token(&self, tenant_id: i64, user_id: Option<i64>, key: &str, payload: &CreateTokenRequest) -> Result<Token> {
        let expire_at = payload.expire_at.map(|ts| DateTime::from_timestamp(ts, 0).unwrap());
        let allowed_models_json = payload.allowed_models.as_ref()
            .and_then(|m| serde_json::to_value(m).ok());
        let allowed_ips_json = payload.allowed_ips.as_ref()
            .and_then(|ips| serde_json::to_value(ips).ok());
        
        let token = sqlx::query_as::<_, Token>(
            r#"INSERT INTO tokens 
               (tenant_id, user_id, key, name, quota_limit, expire_at, status, allowed_models, allowed_ips, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, $8, NOW(), NOW())
               RETURNING *"#
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(key)
        .bind(&payload.name)
        .bind(if payload.quota_limit > 0 { Some(payload.quota_limit) } else { None })
        .bind(expire_at)
        .bind(allowed_models_json)
        .bind(allowed_ips_json)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(token)
    }
    
    /// Update token quota
    /// 更新令牌配额
    pub async fn update_token_quota(&self, token_id: i64, quota_limit: i64) -> Result<Token> {
        let token = sqlx::query_as::<_, Token>(
            "UPDATE tokens SET quota_limit = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(token_id)
        .bind(quota_limit)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(token)
    }
    
    /// Update token key
    /// 更新令牌密钥
    pub async fn update_token_key(&self, token_id: i64, new_key: &str) -> Result<()> {
        sqlx::query("UPDATE tokens SET key = $2, updated_at = NOW() WHERE id = $1")
            .bind(token_id)
            .bind(new_key)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Delete token
    /// 删除令牌
    pub async fn delete_token(&self, token_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM tokens WHERE id = $1")
            .bind(token_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Update token usage
    /// 更新令牌用量
    pub async fn update_token_usage(
        &self,
        token_id: i64,
        tokens_used: i64,
    ) -> Result<Token> {
        let token = sqlx::query_as::<_, Token>(
            r#"UPDATE tokens 
               SET quota_used = quota_used + $2, 
                   last_used_at = NOW(),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#
        )
        .bind(token_id)
        .bind(tokens_used)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(token)
    }
    
    /// Check token quota
    /// 检查令牌配额
    pub async fn check_token_quota(
        &self,
        token_id: i64,
        tokens_needed: i64,
    ) -> Result<bool> {
        let token = self.find_token(token_id).await?
            .ok_or(Error::TokenNotFound)?;
        
        // If no quota limit, allow
        let quota_limit = match token.quota_limit {
            Some(limit) => limit,
            None => return Ok(true),
        };
        
        // Check if quota_used + tokens_needed <= quota_limit
        Ok(token.quota_used + tokens_needed <= quota_limit)
    }
    
    /// Validate token access to a model
    /// 验证令牌对模型的访问权限
    pub async fn validate_token_access(
        &self,
        token_id: i64,
        model: &str,
    ) -> Result<bool> {
        let token = self.find_token(token_id).await?
            .ok_or(Error::TokenNotFound)?;
        
        // If no model restrictions, allow
        let allowed_models = match &token.allowed_models {
            Some(models) => models,
            None => return Ok(true),
        };
        
        if let Some(model_array) = allowed_models.as_array() {
            for model_value in model_array {
                if let Some(allowed_model) = model_value.as_str() {
                    // Exact match
                    if allowed_model == model {
                        return Ok(true);
                    }
                    
                    // Wildcard match (e.g., "gpt-*" matches "gpt-3.5-turbo")
                    if allowed_model.ends_with('*') {
                        let prefix = &allowed_model[..allowed_model.len() - 1];
                        if model.starts_with(prefix) {
                            return Ok(true);
                        }
                    }
                }
            }
            return Ok(false);
        }
        
        // If allowed_models is null or empty, allow all
        Ok(true)
    }
}

// ============================================================================
// Stats Repository Methods
// ============================================================================

impl Database {
    /// Get realtime stats
    /// 获取实时统计
    pub async fn get_realtime_stats(&self, tenant_id: i64) -> Result<RealtimeStats> {
        // Calculate from recent data (last minute)
        let tps: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM usage_stats WHERE tenant_id = $1 AND created_at > NOW() - INTERVAL '1 minute'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let rpm: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM usage_stats WHERE tenant_id = $1 AND created_at > NOW() - INTERVAL '1 hour'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let avg_latency: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(latency_ms), 0) FROM usage_stats WHERE tenant_id = $1 AND created_at > NOW() - INTERVAL '1 hour'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        let active_channels: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM channels WHERE tenant_id = $1 AND status = 'active'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(RealtimeStats {
            tps,
            rpm: rpm / 60,
            latency_ms: avg_latency,
            error_rate: 0.0, // Would calculate from error status
            active_channels,
            timestamp: Utc::now(),
        })
    }
    
    /// Get usage stats
    /// 获取用量统计
    pub async fn get_usage_stats(&self, tenant_id: i64, start: Option<&str>, end: Option<&str>, group_by: &str) -> Result<Vec<crate::routes::stats::UsageStatEntry>> {
        // Build date format based on group_by
        let date_format = match group_by {
            "hour" => "YYYY-MM-DD HH24:MI",
            "minute" => "YYYY-MM-DD HH24:MI:SS",
            "week" => "IYYY-IW",
            _ => "YYYY-MM-DD", // default: day
        };
        
        // Build date range filter
        let mut date_filter = String::new();
        if let Some(start_date) = start {
            date_filter.push_str(&format!(" AND us.created_at >= '{}'", start_date));
        }
        if let Some(end_date) = end {
            date_filter.push_str(&format!(" AND us.created_at <= '{}'", end_date));
        }
        
        let query = format!(
            r#"SELECT 
                   TO_CHAR(us.created_at, '{}') as timestamp,
                   COUNT(*) as requests,
                   COALESCE(SUM(us.total_tokens), 0) as total_tokens,
                   COALESCE(SUM(us.prompt_tokens), 0) as prompt_tokens,
                   COALESCE(SUM(us.completion_tokens), 0) as completion_tokens,
                   COALESCE(SUM(us.cost), 0) as cost
               FROM usage_stats us
               WHERE us.tenant_id = $1{}
               GROUP BY TO_CHAR(us.created_at, '{}')
               ORDER BY timestamp"#,
            date_format, date_filter, date_format
        );
        
        let stats = sqlx::query_as::<_, crate::routes::stats::UsageStatEntry>(&query)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(stats)
    }
    
    /// Get request trend for the last N days
    /// 获取最近 N 天的请求趋势
    pub async fn get_request_trend(&self, tenant_id: i64, days: i32) -> Result<Vec<crate::routes::stats::UsageTrendEntry>> {
        let query = r#"SELECT 
                           TO_CHAR(DATE(created_at), 'YYYY-MM-DD') as date,
                           COUNT(*) as requests,
                           COALESCE(SUM(us.total_tokens), 0) as tokens,
                           COALESCE(SUM(us.cost), 0) as cost
                       FROM usage_stats us
                       WHERE us.tenant_id = $1 
                       AND created_at >= NOW() - INTERVAL '1 day' * $2
                       GROUP BY DATE(created_at)
                       ORDER BY date"#;
        
        let trend = sqlx::query_as::<_, crate::routes::stats::UsageTrendEntry>(query)
            .bind(tenant_id)
            .bind(days)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(trend)
    }
    
    /// Get channel stats
    /// 获取渠道统计
    pub async fn get_channel_stats(&self, tenant_id: i64, start: Option<&str>, end: Option<&str>) -> Result<Vec<crate::routes::stats::ChannelStats>> {
        // Build date range filter
        let mut date_filter = String::new();
        if let Some(start_date) = start {
            date_filter.push_str(&format!(" AND us.created_at >= '{}'", start_date));
        }
        if let Some(end_date) = end {
            date_filter.push_str(&format!(" AND us.created_at <= '{}'", end_date));
        }
        
        let query = format!(
            r#"SELECT 
                   c.id as channel_id,
                   c.name as channel_name,
                   COUNT(us.id) as requests,
                   COUNT(CASE WHEN us.status != 'error' THEN 1 END) as success_count,
                   COUNT(CASE WHEN us.status = 'error' THEN 1 END) as error_count,
                   COALESCE(AVG(us.latency_ms), 0) as avg_latency_ms,
                   COALESCE(SUM(us.total_tokens), 0) as total_tokens,
                   COALESCE(SUM(us.cost), 0) as cost
               FROM channels c
               LEFT JOIN usage_stats us ON c.id = us.channel_id{}
               WHERE c.tenant_id = $1
               GROUP BY c.id, c.name
               ORDER BY requests DESC"#,
            date_filter
        );
        
        let stats = sqlx::query_as::<_, crate::routes::stats::ChannelStats>(&query)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(stats)
    }
    
    /// Get model stats
    /// 获取模型统计
    pub async fn get_model_stats(&self, tenant_id: i64, start: Option<&str>, end: Option<&str>) -> Result<Vec<crate::routes::stats::ModelStats>> {
        // Build date range filter
        let mut date_filter = String::new();
        if let Some(start_date) = start {
            date_filter.push_str(&format!(" AND created_at >= '{}'", start_date));
        }
        if let Some(end_date) = end {
            date_filter.push_str(&format!(" AND created_at <= '{}'", end_date));
        }
        
        let query = format!(
            r#"SELECT 
                   model,
                   COUNT(*) as requests,
                   COALESCE(SUM(total_tokens), 0) as total_tokens,
                   COALESCE(SUM(prompt_tokens), 0) as prompt_tokens,
                   COALESCE(SUM(completion_tokens), 0) as completion_tokens,
                   COALESCE(SUM(cost), 0) as cost
               FROM usage_stats
               WHERE tenant_id = $1 AND model IS NOT NULL{}
               GROUP BY model
               ORDER BY requests DESC"#,
            date_filter
        );
        
        let stats = sqlx::query_as::<_, crate::routes::stats::ModelStats>(&query)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(stats)
    }
    
    /// Get billing stats
    /// 获取计费统计
    pub async fn get_billing_stats(&self, tenant_id: i64) -> Result<crate::routes::stats::BillingStats> {
        let total_cost: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(cost), 0) FROM usage_stats WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(crate::routes::stats::BillingStats {
            total_cost,
            period_start: Utc::now(),
            period_end: Utc::now(),
            previous_period_cost: 0.0,
            cost_by_category: vec![],
        })
    }
    
    /// Get total requests count
    /// 获取总请求数
    pub async fn get_total_requests(&self, tenant_id: i64) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM usage_stats WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        Ok(count)
    }
    
    /// Get total tokens count
    /// 获取总 token 数
    pub async fn get_total_tokens(&self, tenant_id: i64) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM usage_stats WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        Ok(count)
    }
    
    /// Get active channels count
    /// 获取活跃渠道数
    pub async fn get_active_channels_count(&self, tenant_id: i64) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM channels WHERE tenant_id = $1 AND status = 'active'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        Ok(count)
    }
    
    /// Get today's revenue
    /// 获取今日收入
    pub async fn get_today_revenue(&self, tenant_id: i64) -> Result<f64> {
        let revenue: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(cost), 0) FROM usage_stats WHERE tenant_id = $1 AND DATE(created_at) = CURRENT_DATE"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        Ok(revenue)
    }
    
    /// List all channels for a tenant
    /// 列出租户的所有渠道
    pub async fn list_channels(&self, tenant_id: i64) -> Result<Vec<crate::db::Channel>> {
        let channels = sqlx::query_as::<_, crate::db::Channel>(
            "SELECT * FROM channels WHERE tenant_id = $1 ORDER BY created_at"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        Ok(channels)
    }
}

// ============================================================================
// Alert Repository Methods
// ============================================================================

impl Database {
    /// Get active alerts for dashboard
    /// 获取仪表盘的活跃告警
    pub async fn get_active_alerts(&self, tenant_id: i64, limit: i64) -> Result<Vec<crate::db::Alert>> {
        let alerts = sqlx::query_as::<_, crate::db::Alert>(
            r#"SELECT * FROM alerts 
               WHERE tenant_id = $1 AND is_resolved = FALSE 
               ORDER BY 
                   CASE level 
                       WHEN 'critical' THEN 1 
                       WHEN 'warning' THEN 2 
                       WHEN 'info' THEN 3 
                       ELSE 4 
                   END,
                   created_at DESC 
               LIMIT $2"#
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(alerts)
    }
    
    /// Create a new alert
    /// 创建新告警
    pub async fn create_alert(
        &self,
        tenant_id: i64,
        level: &str,
        alert_type: &str,
        message: &str,
        entity_id: Option<i64>,
        entity_type: Option<&str>,
    ) -> Result<crate::db::Alert> {
        let alert = sqlx::query_as::<_, crate::db::Alert>(
            r#"INSERT INTO alerts (tenant_id, level, alert_type, message, entity_id, entity_type, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW())
               RETURNING *"#
        )
        .bind(tenant_id)
        .bind(level)
        .bind(alert_type)
        .bind(message)
        .bind(entity_id)
        .bind(entity_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e))?;
        
        Ok(alert)
    }
    
    /// Resolve an alert
    /// 解决告警
    pub async fn resolve_alert(&self, alert_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE alerts SET is_resolved = TRUE, resolved_at = NOW() WHERE id = $1"
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Generate alerts based on channel metrics
    /// 根据渠道指标生成告警
    pub async fn generate_channel_alerts(&self, tenant_id: i64) -> Result<Vec<crate::db::Alert>> {
        let mut new_alerts = Vec::new();
        
        // Check for high error rate channels (>10% in last hour)
        let high_error_channels = sqlx::query_as::<_, (i64, String, f64)>(
            r#"SELECT c.id, c.name, 
                      COALESCE(
                          (SELECT COUNT(*)::float FROM usage_stats us 
                           WHERE us.channel_id = c.id AND us.status = 'error' 
                           AND us.created_at > NOW() - INTERVAL '1 hour') * 100.0 /
                          NULLIF((SELECT COUNT(*)::float FROM usage_stats us 
                                  WHERE us.channel_id = c.id 
                                  AND us.created_at > NOW() - INTERVAL '1 hour'), 0),
                          0
                      ) as error_rate
               FROM channels c
               WHERE c.tenant_id = $1 AND c.status = 'active'
               HAVING error_rate > 10"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        
        for (channel_id, channel_name, error_rate) in high_error_channels {
            // Check if alert already exists
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM alerts WHERE entity_id = $1 AND entity_type = 'channel' AND alert_type = 'high_error_rate' AND is_resolved = FALSE)"
            )
            .bind(channel_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);
            
            if !exists {
                if let Ok(alert) = self.create_alert(
                    tenant_id,
                    "critical",
                    "high_error_rate",
                    &format!("渠道 '{}' 的错误率超过阈值 ({:.1}%)", channel_name, error_rate),
                    Some(channel_id),
                    Some("channel"),
                ).await {
                    new_alerts.push(alert);
                }
            }
        }
        
        // Check for high latency channels (>5000ms average in last hour)
        let high_latency_channels = sqlx::query_as::<_, (i64, String, f64)>(
            r#"SELECT c.id, c.name, 
                      COALESCE(
                          (SELECT AVG(us.latency_ms) FROM usage_stats us 
                           WHERE us.channel_id = c.id 
                           AND us.created_at > NOW() - INTERVAL '1 hour'),
                          0
                      ) as avg_latency
               FROM channels c
               WHERE c.tenant_id = $1 AND c.status = 'active'
               HAVING avg_latency > 5000"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        
        for (channel_id, channel_name, avg_latency) in high_latency_channels {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM alerts WHERE entity_id = $1 AND entity_type = 'channel' AND alert_type = 'high_latency' AND is_resolved = FALSE)"
            )
            .bind(channel_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);
            
            if !exists {
                if let Ok(alert) = self.create_alert(
                    tenant_id,
                    "warning",
                    "high_latency",
                    &format!("渠道 '{}' 的平均响应时间过长 ({:.0}ms)", channel_name, avg_latency),
                    Some(channel_id),
                    Some("channel"),
                ).await {
                    new_alerts.push(alert);
                }
            }
        }
        
        // Check for low balance channels (<10)
        let low_balance_channels = sqlx::query_as::<_, (i64, String, rust_decimal::Decimal)>(
            r#"SELECT id, name, balance FROM channels 
               WHERE tenant_id = $1 AND status = 'active' AND balance IS NOT NULL AND balance < 10"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        
        for (channel_id, channel_name, balance) in low_balance_channels {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM alerts WHERE entity_id = $1 AND entity_type = 'channel' AND alert_type = 'low_balance' AND is_resolved = FALSE)"
            )
            .bind(channel_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);
            
            if !exists {
                if let Ok(alert) = self.create_alert(
                    tenant_id,
                    "warning",
                    "low_balance",
                    &format!("渠道 '{}' 余额不足 ({:.2})", channel_name, balance),
                    Some(channel_id),
                    Some("channel"),
                ).await {
                    new_alerts.push(alert);
                }
            }
        }
        
        Ok(new_alerts)
    }
    
    /// Generate alerts for token quota
    /// 生成令牌配额告警
    pub async fn generate_token_alerts(&self, tenant_id: i64) -> Result<Vec<crate::db::Alert>> {
        let mut new_alerts = Vec::new();
        
        // Check for tokens nearing quota limit (>80% used)
        let quota_tokens = sqlx::query_as::<_, (i64, String, i64, i64)>(
            r#"SELECT id, name, quota_limit, quota_used FROM tokens 
               WHERE tenant_id = $1 AND status = 'active' 
               AND quota_limit > 0 
               AND quota_used::float / quota_limit::float > 0.8"#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        
        for (token_id, token_name, quota_limit, quota_used) in quota_tokens {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM alerts WHERE entity_id = $1 AND entity_type = 'token' AND alert_type = 'quota_exhaustion' AND is_resolved = FALSE)"
            )
            .bind(token_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);
            
            if !exists {
                let usage_pct = (quota_used as f64 / quota_limit as f64) * 100.0;
                let level = if usage_pct > 95.0 { "critical" } else { "warning" };
                
                if let Ok(alert) = self.create_alert(
                    tenant_id,
                    level,
                    "quota_exhaustion",
                    &format!("令牌 '{}' 的配额即将用尽 (已使用 {:.1}%)", token_name, usage_pct),
                    Some(token_id),
                    Some("token"),
                ).await {
                    new_alerts.push(alert);
                }
            }
        }
        
        Ok(new_alerts)
    }
}
