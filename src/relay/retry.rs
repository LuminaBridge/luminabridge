//! Retry module for LuminaBridge
//!
//! Handles automatic retry with exponential backoff for failed requests.
//! 处理失败请求的自动重试和指数退避。

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, info, debug};

/// Retry condition that triggers a retry
/// 触发重试的条件
#[derive(Debug, Clone)]
pub enum RetryCondition {
    /// Network error (connection failed, timeout, etc.)
    /// 网络错误（连接失败、超时等）
    NetworkError,
    
    /// Request timeout
    /// 请求超时
    Timeout,
    
    /// Server error with status code (5xx errors)
    /// 服务器错误及状态码（5xx 错误）
    ServerError(u16),
    
    /// Rate limit exceeded (429)
    /// 超出速率限制（429）
    RateLimit,
}

impl RetryCondition {
    /// Check if a status code should trigger retry
    /// 检查状态码是否应触发重试
    pub fn should_retry_status(status: u16) -> Option<RetryCondition> {
        if status == 429 {
            Some(RetryCondition::RateLimit)
        } else if status >= 500 && status < 600 {
            Some(RetryCondition::ServerError(status))
        } else {
            None
        }
    }
}

/// Retry configuration
/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    /// 最大重试次数
    pub max_retries: u32,
    
    /// Base delay in milliseconds for exponential backoff
    /// 指数退避的基础延迟（毫秒）
    pub base_delay_ms: u64,
    
    /// Maximum delay in milliseconds
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    
    /// Conditions that should trigger a retry
    /// 应触发重试的条件
    pub retry_on: Vec<RetryCondition>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            base_delay_ms: 1000, // 1 second
            max_delay_ms: 30000, // 30 seconds
            retry_on: vec![
                RetryCondition::NetworkError,
                RetryCondition::Timeout,
                RetryCondition::RateLimit,
                RetryCondition::ServerError(500),
                RetryCondition::ServerError(502),
                RetryCondition::ServerError(503),
                RetryCondition::ServerError(504),
            ],
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with custom settings
    /// 创建自定义设置的重试配置
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        RetryConfig {
            max_retries,
            base_delay_ms,
            max_delay_ms,
            retry_on: vec![],
        }
    }
    
    /// Add a retry condition
    /// 添加重试条件
    pub fn with_condition(mut self, condition: RetryCondition) -> Self {
        self.retry_on.push(condition);
        self
    }
    
    /// Check if a condition should trigger retry
    /// 检查条件是否应触发重试
    pub fn should_retry(&self, condition: &RetryCondition) -> bool {
        self.retry_on.iter().any(|c| {
            matches!(
                (c, condition),
                (RetryCondition::NetworkError, RetryCondition::NetworkError)
                | (RetryCondition::Timeout, RetryCondition::Timeout)
                | (RetryCondition::RateLimit, RetryCondition::RateLimit)
                | (RetryCondition::ServerError(_), RetryCondition::ServerError(_))
            )
        })
    }
    
    /// Calculate delay for a given attempt using exponential backoff with jitter
    /// 使用带抖动的指数退避计算给定尝试的延迟
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        // Exponential backoff: base_delay * 2^attempt
        let exponential_delay = self.base_delay_ms * 2u64.pow(attempt);
        
        // Add jitter: random value between 0 and 10% of delay
        let jitter = (exponential_delay as f64 * 0.1 * rand::random::<f64>()) as u64;
        
        let delay_ms = (exponential_delay + jitter).min(self.max_delay_ms);
        
        Duration::from_millis(delay_ms)
    }
}

/// Result of a retry operation
/// 重试操作的结果
#[derive(Debug)]
pub struct RetryResult<T, E> {
    /// The final result (Ok or Err)
    /// 最终结果（Ok 或 Err）
    pub result: Result<T, E>,
    
    /// Number of attempts made
    /// 尝试次数
    pub attempts: u32,
    
    /// Whether a retry occurred
    /// 是否发生了重试
    pub retried: bool,
}

/// Retry with exponential backoff
/// 带指数退避的重试
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - Async operation to retry
/// * `should_retry` - Function to determine if an error should trigger retry
///
/// # Returns
///
/// RetryResult containing the final result and metadata
pub async fn retry_with_backoff<F, Fut, T, E, S>(
    config: &RetryConfig,
    operation: F,
    should_retry: S,
) -> RetryResult<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>> + Send,
    S: Fn(&E) -> bool,
{
    let mut attempts = 0;
    let mut last_error: Option<E> = None;
    
    loop {
        attempts += 1;
        debug!("Attempt {} of {}", attempts, config.max_retries + 1);
        
        match operation().await {
            Ok(result) => {
                if attempts > 1 {
                    info!("Operation succeeded after {} attempts", attempts);
                }
                return RetryResult {
                    result: Ok(result),
                    attempts,
                    retried: attempts > 1,
                };
            }
            Err(error) => {
                // Check if we should retry this error
                if !should_retry(&error) {
                    debug!("Error not retryable, failing immediately");
                    return RetryResult {
                        result: Err(error),
                        attempts,
                        retried: attempts > 1,
                    };
                }
                
                // Check if we've exhausted retries
                if attempts > config.max_retries {
                    warn!("Max retries ({}) exceeded, failing", config.max_retries);
                    return RetryResult {
                        result: Err(error),
                        attempts,
                        retried: attempts > 1,
                    };
                }
                
                last_error = Some(error);
                
                // Calculate and wait for backoff delay
                let delay = config.calculate_delay(attempts - 1);
                warn!(
                    "Attempt {} failed, retrying in {:?} (max: {})",
                    attempts, delay, config.max_retries
                );
                sleep(delay).await;
            }
        }
    }
}

/// Simple retry wrapper for operations that always retry on error
/// 简单重试包装器，出错时总是重试
pub async fn retry_simple<F, Fut, T, E>(
    config: &RetryConfig,
    operation: F,
) -> RetryResult<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>> + Send,
    E: std::fmt::Debug,
{
    retry_with_backoff(config, operation, |_| true).await
}

/// Check if an HTTP status code should trigger retry based on config
/// 根据配置检查 HTTP 状态码是否应触发重试
pub fn should_retry_status(config: &RetryConfig, status: u16) -> bool {
    if let Some(condition) = RetryCondition::should_retry_status(status) {
        config.should_retry(&condition)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig::default();
        
        // Attempt 0: ~1000ms
        let delay0 = config.calculate_delay(0);
        assert!(delay0.as_millis() >= 1000);
        assert!(delay0.as_millis() <= 1100); // With jitter
        
        // Attempt 1: ~2000ms
        let delay1 = config.calculate_delay(1);
        assert!(delay1.as_millis() >= 2000);
        assert!(delay1.as_millis() <= 2200);
        
        // Attempt 2: ~4000ms
        let delay2 = config.calculate_delay(2);
        assert!(delay2.as_millis() >= 4000);
        assert!(delay2.as_millis() <= 4400);
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            max_retries: 10,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
            retry_on: vec![],
        };
        
        // Attempt 10 would be 1000 * 2^10 = 1024000ms, but should be capped
        let delay = config.calculate_delay(10);
        assert!(delay.as_millis() <= 5000);
    }

    #[test]
    fn test_should_retry_status() {
        let config = RetryConfig::default();
        
        assert!(should_retry_status(&config, 429));
        assert!(should_retry_status(&config, 500));
        assert!(should_retry_status(&config, 503));
        assert!(!should_retry_status(&config, 400));
        assert!(!should_retry_status(&config, 404));
    }

    #[tokio::test]
    async fn test_retry_success_on_first_try() {
        let config = RetryConfig::default();
        let mut call_count = 0;
        
        let result = retry_simple(&config, || async {
            call_count += 1;
            Ok::<_, String>("success".to_string())
        }).await;
        
        assert!(result.result.is_ok());
        assert_eq!(result.attempts, 1);
        assert!(!result.retried);
        assert_eq!(call_count, 1);
    }

    #[tokio::test]
    async fn test_retry_until_success() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10, // Fast for testing
            max_delay_ms: 100,
            retry_on: vec![],
        };
        
        let mut call_count = 0;
        
        let result = retry_simple(&config, || async {
            call_count += 1;
            if call_count < 3 {
                Err("temporary error".to_string())
            } else {
                Ok::<_, String>("success".to_string())
            }
        }).await;
        
        assert!(result.result.is_ok());
        assert_eq!(result.attempts, 3);
        assert!(result.retried);
        assert_eq!(call_count, 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
            max_delay_ms: 100,
            retry_on: vec![],
        };
        
        let mut call_count = 0;
        
        let result = retry_simple(&config, || async {
            call_count += 1;
            Err::<String, _>("persistent error".to_string())
        }).await;
        
        assert!(result.result.is_err());
        assert_eq!(result.attempts, 3); // Initial + 2 retries
        assert!(result.retried);
        assert_eq!(call_count, 3);
    }
}
