//! Streaming module for LuminaBridge
//!
//! Handles streaming responses with real-time token tracking.
//! 处理带实时令牌追踪的流式响应。

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;
use futures_util::stream::StreamExt;
use tracing::{warn, debug, info};

use crate::error::{Error, Result};
use crate::relay::types::{ChatCompletionChunk, Usage};

/// Error indicating quota has been exceeded
/// 表示配额已超出的错误
#[derive(Debug, Clone)]
pub struct QuotaExceeded {
    /// Tokens used before exceeding
    /// 超出前使用的令牌数
    pub tokens_used: i64,
    /// Quota limit
    /// 配额限制
    pub quota_limit: i64,
    /// Additional tokens that would have been used
    /// 本将使用的额外令牌数
    pub tokens_requested: i64,
}

impl std::fmt::Display for QuotaExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Quota exceeded: used {} of {}, requested {} more",
            self.tokens_used, self.quota_limit, self.tokens_requested
        )
    }
}

impl std::error::Error for QuotaExceeded {}

/// Token tracker for monitoring usage during streaming
/// 用于监控流式传输期间用量的令牌追踪器
pub struct TokenTracker {
    /// Prompt tokens (known at start)
    /// Prompt tokens（开始时已知）
    pub prompt_tokens: i64,
    
    /// Completion tokens accumulated during streaming
    /// 流式传输期间累积的 completion tokens
    pub completion_tokens: i64,
    
    /// Total quota limit for this request
    /// 此请求的总配额限制
    pub quota_limit: i64,
    
    /// Whether quota has been exceeded
    /// 配额是否已超出
    pub exceeded: bool,
}

impl TokenTracker {
    /// Create a new token tracker
    /// 创建新的令牌追踪器
    ///
    /// # Arguments
    ///
    /// * `prompt_tokens` - Number of prompt tokens
    /// * `quota_limit` - Maximum allowed total tokens (0 = unlimited)
    pub fn new(prompt_tokens: i64, quota_limit: i64) -> Self {
        TokenTracker {
            prompt_tokens,
            completion_tokens: 0,
            quota_limit,
            exceeded: false,
        }
    }
    
    /// Add completion tokens and check if quota is exceeded
    /// 添加 completion tokens 并检查配额是否超出
    ///
    /// # Arguments
    ///
    /// * `tokens` - Number of completion tokens to add
    ///
    /// # Returns
    ///
    /// Ok(()) if within quota, Err(QuotaExceeded) if exceeded
    pub fn add_completion_tokens(&mut self, tokens: i64) -> std::result::Result<(), QuotaExceeded> {
        if self.exceeded {
            return Err(QuotaExceeded {
                tokens_used: self.get_total_usage(),
                quota_limit: self.quota_limit,
                tokens_requested: tokens,
            });
        }
        
        let new_total = self.prompt_tokens + self.completion_tokens + tokens;
        
        // Check quota if limit is set (> 0)
        if self.quota_limit > 0 && new_total > self.quota_limit {
            self.exceeded = true;
            return Err(QuotaExceeded {
                tokens_used: self.get_total_usage(),
                quota_limit: self.quota_limit,
                tokens_requested: tokens,
            });
        }
        
        self.completion_tokens += tokens;
        Ok(())
    }
    
    /// Get total token usage (prompt + completion)
    /// 获取总令牌用量（prompt + completion）
    pub fn get_total_usage(&self) -> i64 {
        self.prompt_tokens + self.completion_tokens
    }
    
    /// Get remaining quota
    /// 获取剩余配额
    pub fn get_remaining(&self) -> i64 {
        if self.quota_limit <= 0 {
            i64::MAX // Unlimited
        } else {
            self.quota_limit - self.get_total_usage()
        }
    }
    
    /// Check if quota has been exceeded
    /// 检查配额是否已超出
    pub fn is_exceeded(&self) -> bool {
        self.exceeded
    }
    
    /// Reset the tracker
    /// 重置追踪器
    pub fn reset(&mut self) {
        self.completion_tokens = 0;
        self.exceeded = false;
    }
}

/// Wrapper for streaming responses with token tracking
/// 带令牌追踪的流式响应包装器
pub struct StreamingResponse<S>
where
    S: Stream<Item = Result<ChatCompletionChunk>> + Unpin,
{
    /// The underlying SSE stream
    /// 底层 SSE 流
    pub stream: S,
    
    /// Token tracker for monitoring usage
    /// 用于监控用量的令牌追踪器
    pub token_tracker: TokenTracker,
    
    /// Whether the stream has completed
    /// 流是否已完成
    pub completed: bool,
    
    /// Final usage statistics (available after completion)
    /// 最终用量统计（完成后可用）
    pub final_usage: Option<UsageStats>,
}

/// Usage statistics from a streaming response
/// 来自流式响应的用量统计
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Prompt tokens
    /// Prompt tokens
    pub prompt_tokens: i64,
    /// Completion tokens
    /// Completion tokens
    pub completion_tokens: i64,
    /// Total tokens
    /// Total tokens
    pub total_tokens: i64,
}

impl<S> StreamingResponse<S>
where
    S: Stream<Item = Result<ChatCompletionChunk>> + Unpin,
{
    /// Create a new streaming response wrapper
    /// 创建新的流式响应包装器
    ///
    /// # Arguments
    ///
    /// * `stream` - The underlying SSE stream
    /// * `prompt_tokens` - Number of prompt tokens
    /// * `quota_limit` - Maximum allowed total tokens (0 = unlimited)
    pub fn new(stream: S, prompt_tokens: i64, quota_limit: i64) -> Self {
        StreamingResponse {
            stream,
            token_tracker: TokenTracker::new(prompt_tokens, quota_limit),
            completed: false,
            final_usage: None,
        }
    }
    
    /// Get current token usage
    /// 获取当前令牌用量
    pub fn current_usage(&self) -> i64 {
        self.token_tracker.get_total_usage()
    }
    
    /// Get final usage stats (only available after completion)
    /// 获取最终用量统计（仅在完成后可用）
    pub fn get_final_usage(&self) -> Option<&UsageStats> {
        self.final_usage.as_ref()
    }
    
    /// Check if quota has been exceeded
    /// 检查配额是否已超出
    pub fn is_quota_exceeded(&self) -> bool {
        self.token_tracker.is_exceeded()
    }
}

impl<S> Stream for StreamingResponse<S>
where
    S: Stream<Item = Result<ChatCompletionChunk>> + Unpin,
{
    type Item = Result<ChatCompletionChunk>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // If already exceeded or completed, return None
        if self.token_tracker.is_exceeded() || self.completed {
            return Poll::Ready(None);
        }
        
        // Poll the underlying stream
        let stream_pin = Pin::new(&mut self.stream);
        match stream_pin.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                // Estimate tokens from chunk content
                // This is an approximation - real token counting would require a tokenizer
                let content_length = chunk
                    .choices
                    .first()
                    .and_then(|c| c.delta.content.as_ref())
                    .map(|t| t.len())
                    .unwrap_or(0);
                
                // Rough estimation: ~4 characters per token (English average)
                let estimated_tokens = (content_length / 4) as i64;
                
                // If this is the final chunk with usage info, use actual values
                // Note: ChatCompletionChunk may have usage in a separate field if provided by provider
                if let Some(ref usage) = get_chunk_usage(&chunk) {
                    let actual_completion = usage.completion_tokens as i64;
                    let actual_total = usage.total_tokens as i64;
                    let actual_prompt = usage.prompt_tokens as i64;
                    
                    // Update tracker with actual values
                    self.token_tracker.completion_tokens = actual_completion;
                    self.token_tracker.prompt_tokens = actual_prompt;
                    
                    // Store final usage
                    self.final_usage = Some(UsageStats {
                        prompt_tokens: actual_prompt,
                        completion_tokens: actual_completion,
                        total_tokens: actual_total,
                    });
                    
                    self.completed = true;
                    
                    debug!(
                        "Stream completed: prompt={}, completion={}, total={}",
                        actual_prompt, actual_completion, actual_total
                    );
                    
                    Poll::Ready(Some(Ok(chunk)))
                } else {
                    // Add estimated tokens to tracker
                    if estimated_tokens > 0 {
                        if let Err(e) = self.token_tracker.add_completion_tokens(estimated_tokens) {
                            warn!("Quota exceeded during streaming: {}", e);
                            self.completed = true;
                            
                            // Return error chunk to signal quota exceeded
                            return Poll::Ready(Some(Err(Error::QuotaExceeded(
                                e.tokens_used,
                                e.quota_limit,
                            ))));
                        }
                    }
                    
                    Poll::Ready(Some(Ok(chunk)))
                }
            }
            Poll::Ready(Some(Err(e))) => {
                // Stream error - mark as completed
                self.completed = true;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                // Stream ended normally
                self.completed = true;
                
                // If we don't have usage info yet, estimate from tracked tokens
                if self.final_usage.is_none() {
                    self.final_usage = Some(UsageStats {
                        prompt_tokens: self.token_tracker.prompt_tokens,
                        completion_tokens: self.token_tracker.completion_tokens,
                        total_tokens: self.token_tracker.get_total_usage(),
                    });
                    
                    info!(
                        "Stream ended (no usage info): total_tokens={}",
                        self.token_tracker.get_total_usage()
                    );
                }
                
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Extract usage from chunk if available
/// 从块中提取用量（如果可用）
fn get_chunk_usage(chunk: &ChatCompletionChunk) -> Option<&Usage> {
    // ChatCompletionChunk doesn't have usage field by default
    // Some providers may include it in the final chunk as an extension
    // For now, return None and rely on tracked estimates
    None
}

/// Estimate token count from text content
/// 从文本内容估算令牌数
///
/// This is a rough estimation. For accurate counting, use a proper tokenizer.
/// 这是粗略估算。要准确计数，请使用适当的分词器。
pub fn estimate_tokens(text: &str) -> i64 {
    // Simple estimation: ~4 characters per token for English
    // This varies by language and content type
    (text.len() as f64 / 4.0).ceil() as i64
}

/// Count tokens in a message using character-based estimation
/// 使用基于字符的估算计算消息中的令牌数
pub fn count_message_tokens(content: &Option<crate::relay::types::MessageContent>) -> i64 {
    match content {
        None => 0,
        Some(crate::relay::types::MessageContent::Text(text)) => estimate_tokens(text),
        Some(crate::relay::types::MessageContent::Parts(parts)) => {
            parts.iter()
                .filter_map(|p| p.text.as_ref())
                .map(|t| estimate_tokens(t))
                .sum()
        }
    }
}

/// Count tokens in a delta content (streaming)
/// 计算增量内容中的令牌数（流式）
pub fn count_delta_tokens(content: &Option<String>) -> i64 {
    match content {
        None => 0,
        Some(text) => estimate_tokens(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::wrappers::ReceiverStream;
    use futures_util::stream;

    #[test]
    fn test_token_tracker_basic() {
        let mut tracker = TokenTracker::new(100, 1000);
        
        assert_eq!(tracker.get_total_usage(), 100);
        assert_eq!(tracker.get_remaining(), 900);
        assert!(!tracker.is_exceeded());
    }

    #[test]
    fn test_token_tracker_add_tokens() {
        let mut tracker = TokenTracker::new(100, 1000);
        
        tracker.add_completion_tokens(200).unwrap();
        assert_eq!(tracker.get_total_usage(), 300);
        assert_eq!(tracker.get_remaining(), 700);
        
        tracker.add_completion_tokens(500).unwrap();
        assert_eq!(tracker.get_total_usage(), 800);
        assert_eq!(tracker.get_remaining(), 200);
    }

    #[test]
    fn test_token_tracker_exceed_quota() {
        let mut tracker = TokenTracker::new(100, 1000);
        
        tracker.add_completion_tokens(800).unwrap();
        assert_eq!(tracker.get_total_usage(), 900);
        
        // This should exceed quota
        let result = tracker.add_completion_tokens(200);
        assert!(result.is_err());
        assert!(tracker.is_exceeded());
    }

    #[test]
    fn test_token_tracker_unlimited() {
        let mut tracker = TokenTracker::new(100, 0); // 0 = unlimited
        
        tracker.add_completion_tokens(10000).unwrap();
        assert!(!tracker.is_exceeded());
        assert_eq!(tracker.get_remaining(), i64::MAX);
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("hello world"), 3); // 11 chars / 4 = 2.75 -> 3
        assert_eq!(estimate_tokens("test"), 1); // 4 chars / 4 = 1
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_quota_exceeded_error() {
        let quota_err = QuotaExceeded {
            tokens_used: 900,
            quota_limit: 1000,
            tokens_requested: 200,
        };
        
        let msg = quota_err.to_string();
        assert!(msg.contains("900"));
        assert!(msg.contains("1000"));
        assert!(msg.contains("200"));
    }
}
