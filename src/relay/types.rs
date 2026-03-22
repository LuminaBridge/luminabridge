//! Relay types for OpenAI-compatible API
//!
//! Defines request/response types for chat completions and other relay operations.
//! 为聊天完成和其他中继操作定义请求/响应类型。

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Chat Completion Types (OpenAI Compatible)
// ============================================================================

/// Chat completion request (OpenAI format)
/// 聊天完成请求（OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model to use
    /// 要使用的模型
    pub model: String,
    
    /// Messages array
    /// 消息数组
    pub messages: Vec<Message>,
    
    /// Temperature (0.0 - 2.0)
    /// 温度（0.0 - 2.0）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Top p (0.0 - 1.0)
    /// Top p（0.0 - 1.0）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Number of choices
    /// 选择数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    
    /// Stream mode
    /// 流式模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Stop sequences
    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopSequence>,
    
    /// Max tokens
    /// 最大令牌数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Presence penalty
    /// 存在惩罚
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// Frequency penalty
    /// 频率惩罚
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// Logit bias
    /// Logit 偏置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<std::collections::HashMap<String, f32>>,
    
    /// User identifier
    /// 用户标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    
    /// Response format
    /// 响应格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    
    /// Tools array
    /// 工具数组
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    
    /// Tool choice
    /// 工具选择
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

/// Stop sequence can be a string or array of strings
/// 停止序列可以是字符串或字符串数组
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StopSequence {
    /// Single stop sequence
    /// 单个停止序列
    String(String),
    /// Multiple stop sequences
    /// 多个停止序列
    Array(Vec<String>),
}

/// Message in a chat conversation
/// 聊天对话中的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    /// 消息角色
    pub role: MessageRole,
    
    /// Message content
    /// 消息内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    
    /// Message name (optional)
    /// 消息名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Tool calls (for assistant messages)
    /// 工具调用（用于助手消息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    
    /// Tool call ID (for tool messages)
    /// 工具调用 ID（用于工具消息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Message role
/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message
    /// 系统消息
    System,
    /// User message
    /// 用户消息
    User,
    /// Assistant message
    /// 助手消息
    Assistant,
    /// Tool message
    /// 工具消息
    Tool,
}

/// Message content can be text or array of content parts
/// 消息内容可以是文本或内容部分数组
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    /// 简单文本内容
    Text(String),
    /// Array of content parts (for multimodal)
    /// 内容部分数组（用于多模态）
    Parts(Vec<ContentPart>),
}

/// Content part for multimodal messages
/// 多模态消息的内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    /// Content type
    /// 内容类型
    #[serde(rename = "type")]
    pub content_type: String,
    
    /// Text content (if type is text)
    /// 文本内容（如果类型是 text）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    
    /// Image URL (if type is image_url)
    /// 图片 URL（如果类型是 image_url）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

/// Image URL for multimodal content
/// 多模态内容的图片 URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL
    /// 图片 URL
    pub url: String,
    
    /// Detail level
    /// 详细程度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Tool definition
/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool type
    /// 工具类型
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Function definition
    /// 函数定义
    pub function: FunctionDefinition,
}

/// Function definition for tools
/// 工具函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    /// 函数名称
    pub name: String,
    
    /// Function description
    /// 函数描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Function parameters (JSON Schema)
    /// 函数参数（JSON Schema）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// Tool choice
/// 工具选择
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// String choice (none, auto, required)
    /// 字符串选择（none, auto, required）
    String(String),
    /// Object choice for specific function
    /// 特定函数的对象选择
    Object {
        /// Tool type
        /// 工具类型
        #[serde(rename = "type")]
        tool_type: String,
        /// Function to call
        /// 要调用的函数
        function: FunctionChoice,
    },
}

/// Function choice
/// 函数选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    /// Function name
    /// 函数名称
    pub name: String,
}

/// Tool call
/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    /// 工具调用 ID
    pub id: String,
    
    /// Tool type
    /// 工具类型
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Function call
    /// 函数调用
    pub function: FunctionCall,
}

/// Function call
/// 函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    /// 函数名称
    pub name: String,
    
    /// Function arguments (JSON string)
    /// 函数参数（JSON 字符串）
    pub arguments: String,
}

/// Response format
/// 响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Format type (text, json_object)
    /// 格式类型（text, json_object）
    #[serde(rename = "type")]
    pub format_type: String,
}

// ============================================================================
// Chat Completion Response Types
// ============================================================================

/// Chat completion response (OpenAI format)
/// 聊天完成响应（OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Response ID
    /// 响应 ID
    pub id: String,
    
    /// Object type
    /// 对象类型
    pub object: String,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created: i64,
    
    /// Model used
    /// 使用的模型
    pub model: String,
    
    /// Choices array
    /// 选择数组
    pub choices: Vec<Choice>,
    
    /// Usage statistics
    /// 用量统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    
    /// System fingerprint
    /// 系统指纹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Choice in a chat completion response
/// 聊天完成响应中的选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Choice index
    /// 选择索引
    pub index: u32,
    
    /// Message
    /// 消息
    pub message: Message,
    
    /// Finish reason
    /// 结束原因
    pub finish_reason: Option<String>,
    
    /// Log probabilities
    /// 对数概率
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Value>,
}

/// Usage statistics
/// 用量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Prompt tokens
    /// 提示词令牌数
    pub prompt_tokens: u32,
    
    /// Completion tokens
    /// 完成令牌数
    pub completion_tokens: u32,
    
    /// Total tokens
    /// 总令牌数
    pub total_tokens: u32,
}

/// Stream chunk for SSE responses
/// SSE 响应的流式块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Chunk ID
    /// 块 ID
    pub id: String,
    
    /// Object type
    /// 对象类型
    pub object: String,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created: i64,
    
    /// Model used
    /// 使用的模型
    pub model: String,
    
    /// Choices array
    /// 选择数组
    pub choices: Vec<ChunkChoice>,
    
    /// System fingerprint
    /// 系统指纹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Choice in a stream chunk
/// 流式块中的选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    /// Choice index
    /// 选择索引
    pub index: u32,
    
    /// Delta (incremental message)
    /// 增量（增量消息）
    pub delta: Delta,
    
    /// Finish reason
    /// 结束原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Delta for streaming responses
/// 流式响应的增量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    /// Role
    /// 角色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    
    /// Content
    /// 内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    
    /// Tool calls
    /// 工具调用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

// ============================================================================
// Model Types
// ============================================================================

/// Model list response
/// 模型列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelList {
    /// Object type
    /// 对象类型
    pub object: String,
    
    /// Data array
    /// 数据数组
    pub data: Vec<Model>,
}

/// Model information
/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model ID
    /// 模型 ID
    pub id: String,
    
    /// Object type
    /// 对象类型
    pub object: String,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created: i64,
    
    /// Owned by
    /// 所有者
    pub owned_by: String,
}

// ============================================================================
// Text Completion Types (Legacy)
// ============================================================================

/// Text completion request (legacy OpenAI format)
/// 文本完成请求（传统 OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use
    /// 要使用的模型
    pub model: String,
    
    /// Prompt
    /// 提示词
    pub prompt: Prompt,
    
    /// Temperature
    /// 温度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Max tokens
    /// 最大令牌数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Stream mode
    /// 流式模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Stop sequences
    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopSequence>,
    
    /// Number of choices
    /// 选择数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
}

/// Prompt can be a string or array of strings
/// 提示词可以是字符串或字符串数组
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prompt {
    /// Single prompt
    /// 单个提示词
    String(String),
    /// Multiple prompts
    /// 多个提示词
    Array(Vec<String>),
}

/// Text completion response
/// 文本完成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response ID
    /// 响应 ID
    pub id: String,
    
    /// Object type
    /// 对象类型
    pub object: String,
    
    /// Creation timestamp
    /// 创建时间戳
    pub created: i64,
    
    /// Model used
    /// 使用的模型
    pub model: String,
    
    /// Choices array
    /// 选择数组
    pub choices: Vec<CompletionChoice>,
    
    /// Usage statistics
    /// 用量统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Choice in a completion response
/// 完成响应中的选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// Choice index
    /// 选择索引
    pub index: u32,
    
    /// Generated text
    /// 生成的文本
    pub text: String,
    
    /// Finish reason
    /// 结束原因
    pub finish_reason: Option<String>,
    
    /// Log probabilities
    /// 对数概率
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: Some(MessageContent::Text("Hello".to_string())),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }
            ],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            response_format: None,
            tools: None,
            tool_choice: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::System;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"system\"");
    }

    #[test]
    fn test_usage_serialization() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("30"));
    }
}
