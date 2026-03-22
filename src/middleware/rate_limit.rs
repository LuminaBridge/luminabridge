//! Rate limiting middleware for LuminaBridge
//!
//! Provides token-based and IP-based rate limiting with customizable strategies.
//! 提供基于令牌和基于 IP 的速率限制，支持自定义策略。

use axum::{
    extract::{State, Request},
    http::StatusCode,
    middleware::Next,
    response::{Response, Json},
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn, info};

use crate::config::RateLimitConfig;
use crate::error::{Error, Result};

/// Rate limiter state shared across requests
/// 在请求之间共享的速率限制器状态
#[derive(Clone)]
pub struct RateLimiterState {
    /// Rate limit configuration
    /// 速率限制配置
    config: Arc<RateLimitConfig>,
    
    /// Token-based rate limiter
    /// 基于令牌的速率限制器
    token_limiter: Arc<TokenRateLimiter>,
    
    /// IP-based rate limiter
    /// 基于 IP 的速率限制器
    ip_limiter: Arc<IPRateLimiter>,
}

impl RateLimiterState {
    /// Create a new rate limiter state
    /// 创建新的速率限制器状态
    pub fn new(config: Arc<RateLimitConfig>) -> Self {
        RateLimiterState {
            config: config.clone(),
            token_limiter: Arc::new(TokenRateLimiter::new(
                config.requests_per_sec,
                config.burst_size,
            )),
            ip_limiter: Arc::new(IPRateLimiter::new(
                config.requests_per_sec,
                config.burst_size,
            )),
        }
    }
    
    /// Check if rate limiting is enabled
    /// 检查是否启用了速率限制
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Token bucket rate limiter
/// 令牌桶速率限制器
pub struct TokenRateLimiter {
    /// Requests per second
    /// 每秒请求数
    requests_per_sec: u32,
    
    /// Burst size (max tokens)
    /// 突发大小（最大令牌数）
    burst_size: u32,
    
    /// Token buckets per API key
    /// 每个 API 密钥的令牌桶
    buckets: RwLock<HashMap<String, TokenBucket>>,
}

impl TokenRateLimiter {
    /// Create a new token rate limiter
    /// 创建新的令牌速率限制器
    pub fn new(requests_per_sec: u32, burst_size: u32) -> Self {
        TokenRateLimiter {
            requests_per_sec,
            burst_size,
            buckets: RwLock::new(HashMap::new()),
        }
    }
    
    /// Check if request is allowed for the given API key
    /// 检查给定 API 密钥的请求是否允许
    pub async fn check(&self, api_key: &str) -> RateLimitResult {
        if !self.is_enabled() {
            return RateLimitResult::Allowed;
        }
        
        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets
            .entry(api_key.to_string())
            .or_insert_with(|| TokenBucket::new(self.burst_size, self.requests_per_sec));
        
        bucket.consume()
    }
    
    /// Check if rate limiting is enabled
    /// 检查是否启用了速率限制
    fn is_enabled(&self) -> bool {
        self.requests_per_sec > 0 && self.burst_size > 0
    }
}

/// IP-based rate limiter
/// 基于 IP 的速率限制器
pub struct IPRateLimiter {
    /// Requests per second
    /// 每秒请求数
    requests_per_sec: u32,
    
    /// Burst size
    /// 突发大小
    burst_size: u32,
    
    /// Token buckets per IP
    /// 每个 IP 的令牌桶
    buckets: RwLock<HashMap<String, TokenBucket>>,
}

impl IPRateLimiter {
    /// Create a new IP rate limiter
    /// 创建新的 IP 速率限制器
    pub fn new(requests_per_sec: u32, burst_size: u32) -> Self {
        IPRateLimiter {
            requests_per_sec,
            burst_size,
            buckets: RwLock::new(HashMap::new()),
        }
    }
    
    /// Check if request is allowed for the given IP
    /// 检查给定 IP 的请求是否允许
    pub async fn check(&self, ip: &str) -> RateLimitResult {
        if !self.is_enabled() {
            return RateLimitResult::Allowed;
        }
        
        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets
            .entry(ip.to_string())
            .or_insert_with(|| TokenBucket::new(self.burst_size, self.requests_per_sec));
        
        bucket.consume()
    }
    
    /// Check if rate limiting is enabled
    /// 检查是否启用了速率限制
    fn is_enabled(&self) -> bool {
        self.requests_per_sec > 0 && self.burst_size > 0
    }
}

/// Token bucket implementation
/// 令牌桶实现
struct TokenBucket {
    /// Current number of tokens
    /// 当前令牌数
    tokens: u32,
    
    /// Maximum tokens (burst size)
    /// 最大令牌数（突发大小）
    max_tokens: u32,
    
    /// Refill rate (tokens per second)
    /// 补充速率（每秒令牌数）
    refill_rate: u32,
    
    /// Last refill time
    /// 上次补充时间
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    /// 创建新的令牌桶
    pub fn new(max_tokens: u32, refill_rate: u32) -> Self {
        TokenBucket {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }
    
    /// Consume a token, returns whether the request is allowed
    /// 消耗一个令牌，返回请求是否允许
    pub fn consume(&mut self) -> RateLimitResult {
        self.refill();
        
        if self.tokens > 0 {
            self.tokens -= 1;
            RateLimitResult::Allowed
        } else {
            RateLimitResult::Limited
        }
    }
    
    /// Refill tokens based on elapsed time
    /// 根据经过的时间补充令牌
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        
        // Calculate tokens to add
        let tokens_to_add = (elapsed.as_secs_f32() * self.refill_rate as f32) as u32;
        
        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
            self.last_refill = now;
        }
    }
}

/// Rate limit check result
/// 速率限制检查结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Request is allowed
    /// 请求允许
    Allowed,
    
    /// Request is rate limited
    /// 请求被速率限制
    Limited,
}

impl RateLimitResult {
    /// Check if request is allowed
    /// 检查请求是否允许
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed)
    }
}

/// Rate limit middleware
/// 速率限制中间件
pub async fn rate_limit_middleware(
    State(state): State<RateLimiterState>,
    request: Request,
    next: Next,
) -> Result<Response> {
    if !state.is_enabled() {
        return Ok(next.run(request).await);
    }
    
    // Extract API key from Authorization header
    let api_key = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    
    // Extract client IP
    let client_ip = request
        .headers()
        .get(axum::http::header::FORWARDED)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split('=').nth(1))
        .and_then(|value| value.split(';').next())
        .unwrap_or("unknown")
        .to_string();
    
    // Check token-based rate limit
    if let Some(key) = api_key {
        match state.token_limiter.check(key).await {
            RateLimitResult::Limited => {
                warn!("Rate limit exceeded for API key");
                return Err(Error::RateLimit("Rate limit exceeded".to_string()));
            }
            RateLimitResult::Allowed => {
                debug!("API key rate limit check passed");
            }
        }
    }
    
    // Check IP-based rate limit
    match state.ip_limiter.check(&client_ip).await {
        RateLimitResult::Limited => {
            warn!("Rate limit exceeded for IP: {}", client_ip);
            return Err(Error::RateLimit("Rate limit exceeded".to_string()));
        }
        RateLimitResult::Allowed => {
            debug!("IP rate limit check passed for: {}", client_ip);
        }
    }
    
    Ok(next.run(request).await)
}

/// Create rate limit middleware layer
/// 创建速率限制中间件层
pub fn create_rate_limit_layer(
    config: Arc<RateLimitConfig>,
) -> axum::middleware::FromFn<
    fn(RateLimiterState, Request, Next) -> axum::extract::RequestPart<impl futures_util::Future<Output = std::result::Result<Response, Error>>>,
    RateLimiterState,
> {
    let state = RateLimiterState::new(config);
    axum::middleware::from_fn_with_state(state, rate_limit_middleware)
}

/// Custom rate limit strategy trait
/// 自定义速率限制策略特征
pub trait RateLimitStrategy: Send + Sync {
    /// Check if request should be allowed
    /// 检查请求是否应该允许
    fn check(&self, identifier: &str) -> RateLimitResult;
    
    /// Get strategy name
    /// 获取策略名称
    fn name(&self) -> &str;
}

/// Sliding window rate limiter strategy
/// 滑动窗口速率限制器策略
pub struct SlidingWindowLimiter {
    /// Window size in seconds
    /// 窗口大小（秒）
    window_size: u64,
    
    /// Max requests per window
    /// 每个窗口的最大请求数
    max_requests: u32,
}

impl SlidingWindowLimiter {
    /// Create a new sliding window limiter
    /// 创建新的滑动窗口限制器
    pub fn new(window_size: u64, max_requests: u32) -> Self {
        SlidingWindowLimiter {
            window_size,
            max_requests,
        }
    }
}

impl RateLimitStrategy for SlidingWindowLimiter {
    fn check(&self, _identifier: &str) -> RateLimitResult {
        // Simplified implementation
        // Real implementation would track requests in a time window
        RateLimitResult::Allowed
    }
    
    fn name(&self) -> &str {
        "sliding_window"
    }
}

/// Fixed window rate limiter strategy
/// 固定窗口速率限制器策略
pub struct FixedWindowLimiter {
    /// Window size in seconds
    /// 窗口大小（秒）
    window_size: u64,
    
    /// Max requests per window
    /// 每个窗口的最大请求数
    max_requests: u32,
}

impl FixedWindowLimiter {
    /// Create a new fixed window limiter
    /// 创建新的固定窗口限制器
    pub fn new(window_size: u64, max_requests: u32) -> Self {
        FixedWindowLimiter {
            window_size,
            max_requests,
        }
    }
}

impl RateLimitStrategy for FixedWindowLimiter {
    fn check(&self, _identifier: &str) -> RateLimitResult {
        // Simplified implementation
        RateLimitResult::Allowed
    }
    
    fn name(&self) -> &str {
        "fixed_window"
    }
}

/// Rate limit headers to add to responses
/// 要添加到响应的速率限制头
pub fn add_rate_limit_headers(
    response: &mut Response,
    limit: u32,
    remaining: u32,
    reset_secs: u64,
) {
    use axum::http::header;
    
    response.headers_mut().insert(
        header::HeaderName::from_static("x-ratelimit-limit"),
        limit.to_string().parse().unwrap(),
    );
    
    response.headers_mut().insert(
        header::HeaderName::from_static("x-ratelimit-remaining"),
        remaining.to_string().parse().unwrap(),
    );
    
    response.headers_mut().insert(
        header::HeaderName::from_static("x-ratelimit-reset"),
        reset_secs.to_string().parse().unwrap(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_creation() {
        let bucket = TokenBucket::new(10, 5);
        assert_eq!(bucket.tokens, 10);
        assert_eq!(bucket.max_tokens, 10);
        assert_eq!(bucket.refill_rate, 5);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(3, 1);
        
        assert!(bucket.consume().is_allowed());
        assert!(bucket.consume().is_allowed());
        assert!(bucket.consume().is_allowed());
        
        // Should be limited now
        assert!(!bucket.consume().is_allowed());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(5, 10);
        
        // Consume all tokens
        for _ in 0..5 {
            bucket.consume();
        }
        
        assert!(!bucket.consume().is_allowed());
        
        // Wait a bit and refill (simulated)
        bucket.last_refill = Instant::now() - Duration::from_secs(1);
        bucket.refill();
        
        // Should have tokens again
        assert!(bucket.tokens > 0);
    }

    #[test]
    fn test_rate_limit_result() {
        assert!(RateLimitResult::Allowed.is_allowed());
        assert!(!RateLimitResult::Limited.is_allowed());
    }

    #[tokio::test]
    async fn test_token_rate_limiter() {
        let limiter = TokenRateLimiter::new(10, 5);
        
        // First 5 requests should be allowed
        for _ in 0..5 {
            assert!(limiter.check("test-key").await.is_allowed());
        }
        
        // 6th request should be limited
        assert!(!limiter.check("test-key").await.is_allowed());
    }

    #[tokio::test]
    async fn test_ip_rate_limiter() {
        let limiter = IPRateLimiter::new(10, 3);
        
        // First 3 requests should be allowed
        for _ in 0..3 {
            assert!(limiter.check("127.0.0.1").await.is_allowed());
        }
        
        // 4th request should be limited
        assert!(!limiter.check("127.0.0.1").await.is_allowed());
        
        // Different IP should be allowed
        assert!(limiter.check("192.168.1.1").await.is_allowed());
    }
}
