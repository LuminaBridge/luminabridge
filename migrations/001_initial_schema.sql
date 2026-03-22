-- LuminaBridge Initial Database Schema
-- Migration: 001_initial_schema
-- Date: 2026-03-22
-- Description: Creates all base tables for LuminaBridge

-- =============================================================================
-- Tenants Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS tenants (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    quota_limit BIGINT DEFAULT 0,
    quota_used BIGINT DEFAULT 0,
    settings JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE tenants IS '租户表 - 存储多租户信息';
COMMENT ON COLUMN tenants.id IS '租户 ID';
COMMENT ON COLUMN tenants.name IS '租户名称';
COMMENT ON COLUMN tenants.slug IS '租户标识（URL 友好）';
COMMENT ON COLUMN tenants.status IS '租户状态：active, suspended, deleted';
COMMENT ON COLUMN tenants.quota_limit IS '配额限制';
COMMENT ON COLUMN tenants.quota_used IS '已用配额';
COMMENT ON COLUMN tenants.settings IS '租户设置（JSON）';

-- =============================================================================
-- Users Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    display_name VARCHAR(255),
    avatar_url VARCHAR(500),
    role VARCHAR(50) DEFAULT 'user',
    status VARCHAR(20) DEFAULT 'active',
    oauth_provider VARCHAR(50),
    oauth_id VARCHAR(255),
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE users IS '用户表';
COMMENT ON COLUMN users.id IS '用户 ID';
COMMENT ON COLUMN users.tenant_id IS '所属租户 ID';
COMMENT ON COLUMN users.email IS '用户邮箱（唯一）';
COMMENT ON COLUMN users.password_hash IS '密码哈希（用于密码登录）';
COMMENT ON COLUMN users.role IS '用户角色：admin, member, viewer';
COMMENT ON COLUMN users.status IS '用户状态：active, inactive, banned';

-- =============================================================================
-- OAuth Accounts Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS oauth_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_user_id)
);

COMMENT ON TABLE oauth_accounts IS 'OAuth 账户关联表';
COMMENT ON COLUMN oauth_accounts.provider IS 'OAuth 提供商：github, discord, google';
COMMENT ON COLUMN oauth_accounts.provider_user_id IS '提供商用户 ID';
COMMENT ON COLUMN oauth_accounts.access_token IS '访问令牌';
COMMENT ON COLUMN oauth_accounts.refresh_token IS '刷新令牌';

-- =============================================================================
-- Channels Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS channels (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    channel_type VARCHAR(50) NOT NULL,
    key TEXT NOT NULL,
    base_url VARCHAR(500),
    models JSONB DEFAULT '[]'::jsonb,
    weight INT DEFAULT 10,
    status VARCHAR(20) DEFAULT 'active',
    priority INT DEFAULT 0,
    timeout_ms INT DEFAULT 30000,
    retry_count INT DEFAULT 3,
    balance DECIMAL(20, 6) DEFAULT 0,
    last_test_at TIMESTAMP WITH TIME ZONE,
    last_test_status VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE channels IS '渠道表 - AI 服务提供商配置';
COMMENT ON COLUMN channels.channel_type IS '渠道类型：openai, anthropic, google, azure';
COMMENT ON COLUMN channels.key IS 'API 密钥（加密存储）';
COMMENT ON COLUMN channels.models IS '支持的模型列表（JSON 数组）';
COMMENT ON COLUMN channels.weight IS '负载均衡权重';
COMMENT ON COLUMN channels.status IS '渠道状态：active, disabled, error';

-- =============================================================================
-- Tokens Table (API Keys)
-- =============================================================================
CREATE TABLE IF NOT EXISTS tokens (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT REFERENCES tenants(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    key VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    quota_limit BIGINT DEFAULT 0,
    quota_used BIGINT DEFAULT 0,
    expire_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'active',
    allowed_ips JSONB DEFAULT '[]'::jsonb,
    allowed_models JSONB DEFAULT '[]'::jsonb,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE tokens IS 'API 令牌表';
COMMENT ON COLUMN tokens.key IS '令牌密钥（sk-xxx 格式）';
COMMENT ON COLUMN tokens.quota_limit IS '配额限制（0 表示无限制）';
COMMENT ON COLUMN tokens.allowed_ips IS '允许的 IP 列表（JSON 数组）';
COMMENT ON COLUMN tokens.allowed_models IS '允许的模型列表（JSON 数组）';

-- =============================================================================
-- Usage Stats Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS usage_stats (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT REFERENCES tenants(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    channel_id BIGINT REFERENCES channels(id) ON DELETE SET NULL,
    model VARCHAR(100),
    prompt_tokens BIGINT DEFAULT 0,
    completion_tokens BIGINT DEFAULT 0,
    total_tokens BIGINT DEFAULT 0,
    cost DECIMAL(20, 6) DEFAULT 0,
    status VARCHAR(20),
    latency_ms INT,
    request_id VARCHAR(100),
    response_status INT,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE usage_stats IS '用量统计表';
COMMENT ON COLUMN usage_stats.model IS '使用的模型名称';
COMMENT ON COLUMN usage_stats.prompt_tokens IS '输入 token 数';
COMMENT ON COLUMN usage_stats.completion_tokens IS '输出 token 数';
COMMENT ON COLUMN usage_stats.cost IS '费用';
COMMENT ON COLUMN usage_stats.status IS '请求状态：success, error, timeout';
COMMENT ON COLUMN usage_stats.latency_ms IS '延迟（毫秒）';

-- =============================================================================
-- Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

CREATE INDEX IF NOT EXISTS idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_provider ON oauth_accounts(provider);

CREATE INDEX IF NOT EXISTS idx_channels_tenant_id ON channels(tenant_id);
CREATE INDEX IF NOT EXISTS idx_channels_status ON channels(status);
CREATE INDEX IF NOT EXISTS idx_channels_type ON channels(channel_type);

CREATE INDEX IF NOT EXISTS idx_tokens_tenant_id ON tokens(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tokens_key ON tokens(key);
CREATE INDEX IF NOT EXISTS idx_tokens_status ON tokens(status);

CREATE INDEX IF NOT EXISTS idx_usage_stats_tenant_id ON usage_stats(tenant_id);
CREATE INDEX IF NOT EXISTS idx_usage_stats_user_id ON usage_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_usage_stats_channel_id ON usage_stats(channel_id);
CREATE INDEX IF NOT EXISTS idx_usage_stats_model ON usage_stats(model);
CREATE INDEX IF NOT EXISTS idx_usage_stats_created_at ON usage_stats(created_at);

-- =============================================================================
-- Default Data
-- =============================================================================
-- Insert default tenant
INSERT INTO tenants (id, name, slug, status, quota_limit, quota_used)
VALUES (1, 'Default Tenant', 'default', 'active', 0, 0)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Triggers for updated_at
-- =============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_channels_updated_at
    BEFORE UPDATE ON channels
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tokens_updated_at
    BEFORE UPDATE ON tokens
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
