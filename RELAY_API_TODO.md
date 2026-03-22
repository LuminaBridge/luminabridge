# Relay API 待办事项和已知问题

## 🔧 需要修复的问题

### 1. Database::new() 方法

**问题**: 在测试代码中使用了 `Database::new()`，但该方法需要异步初始化数据库连接。

**当前位置**: `src/relay/mod.rs` 测试代码

**解决方案**: 

在 `src/db/mod.rs` 中添加同步构造函数用于测试：

```rust
impl Database {
    /// Create a new database instance (for testing only)
    /// 创建新的数据库实例（仅用于测试）
    #[cfg(test)]
    pub fn new(config: Arc<Config>) -> Result<Self> {
        // This is a simplified version for testing
        // Real implementation requires async connection
        use sqlx::postgres::PgPoolOptions;
        
        // Would need to be async, so for tests we might need to mock
        todo!("Database connection requires async context")
    }
}
```

**或者** 修改测试使用 mock：

```rust
#[cfg(test)]
mod tests {
    use mockall::mock;
    
    mock! {
        pub Database {
            async fn get_channels_for_model(&self, tenant_id: i64, model: &str) -> Result<Vec<Channel>>;
            async fn get_active_channels(&self, tenant_id: i64) -> Result<Vec<Channel>>;
            // ... other methods
        }
    }
}
```

### 2. extract_tenant_id 实现

**问题**: `src/routes/relay.rs` 中的 `extract_tenant_id()` 返回硬编码的租户 ID。

**当前代码**:
```rust
async fn extract_tenant_id(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<i64> {
    // Simplified implementation - real implementation would:
    // 1. Validate the API key against the database
    // 2. Extract tenant ID from the token/key
    // 3. Return the tenant ID
    
    // For now, return a default tenant ID
    Ok(1)
}
```

**解决方案**: 与现有认证中间件集成

```rust
async fn extract_tenant_id(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<i64> {
    use crate::middleware::auth::extract_token_from_header;
    use crate::middleware::auth::get_claims;
    
    // Extract token from header
    let token = extract_token_from_header(headers)?;
    
    // Validate and get claims
    let claims = get_claims(&token, &state.config.oauth.jwt_secret)?;
    
    // Extract tenant ID from claims
    claims.tenant_id.ok_or_else(|| {
        Error::Auth("Tenant ID not found in token".to_string())
    })
}
```

### 3. 流式响应的完整实现

**问题**: 当前的流式实现较为基础，需要完善。

**当前代码位置**: `src/routes/relay.rs::handle_streaming_completion`

**需要改进**:
- 更好的错误处理
- 正确的 SSE 格式
- 支持 Anthropic 和 Google 的流式响应转换

**建议实现**:

```rust
async fn handle_streaming_completion(
    relay: RelayService,
    request: ChatCompletionRequest,
    channel: &Channel,
    api_key: &str,
) -> Result<Response> {
    use axum::body::Body;
    use futures_util::stream::Stream;
    use std::convert::Infallible;
    
    // Create SSE stream
    let stream = relay.stream_chat_completion(request, channel, api_key).await?;
    
    let sse_stream = stream.map(|chunk_result| {
        match chunk_result {
            Ok(chunk) => {
                let data = serde_json::to_string(&chunk).unwrap_or_default();
                Ok::<_, Infallible>(format!("data: {}\n\n", data))
            }
            Err(e) => {
                // Send error in SSE format
                let error_data = json!({
                    "error": {
                        "message": e.to_string(),
                        "type": "relay_error"
                    }
                });
                Ok(format!("data: {}\n\n", error_data))
            }
        }
    });
    
    // Chain with [DONE] marker
    let stream = sse_stream.chain(tokio_stream::once(
        async { Ok::<_, Infallible>("data: [DONE]\n\n".to_string()) }
    ));
    
    let body = Body::from_stream(stream);
    
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONNECTION, "keep-alive"),
            (header::X_ACCEL_BUFFERING, "no"), // For nginx
        ],
        body,
    ).into_response())
}
```

### 4. 成本计算逻辑

**问题**: `record_usage` 中的成本计算简化为 0。

**当前代码**: `src/relay/mod.rs::record_usage`

```rust
let cost = rust_decimal::Decimal::ZERO; // Simplified
```

**解决方案**: 添加价格表

```rust
// 在 config 中添加价格配置
pub struct PricingConfig {
    pub models: HashMap<String, ModelPricing>,
}

pub struct ModelPricing {
    pub prompt_price_per_1k: Decimal,
    pub completion_price_per_1k: Decimal,
}

// 计算成本
fn calculate_cost(model: &str, usage: &Usage, pricing: &PricingConfig) -> Decimal {
    let model_pricing = pricing.models.get(model)
        .unwrap_or(&DEFAULT_PRICING);
    
    let prompt_cost = Decimal::from(usage.prompt_tokens) 
        * model_pricing.prompt_price_per_1k 
        / Decimal::from(1000);
    
    let completion_cost = Decimal::from(usage.completion_tokens)
        * model_pricing.completion_price_per_1k
        / Decimal::from(1000);
    
    prompt_cost + completion_cost
}
```

### 5. 重试逻辑

**问题**: 缺少自动重试机制。

**建议实现**:

在 `src/relay/mod.rs` 中添加：

```rust
use tokio::time::{sleep, Duration};

/// Relay with retry logic
/// 带重试逻辑的中继
pub async fn relay_with_retry(
    &self,
    request: ChatCompletionRequest,
    channel: &Channel,
    api_key: &str,
) -> Result<ChatCompletionResponse> {
    let mut last_error = None;
    
    for attempt in 0..channel.retry_count {
        match self.relay_chat_completion(request.clone(), channel, api_key).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                warn!("Attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
                
                // Exponential backoff
                if attempt < channel.retry_count - 1 {
                    let delay = Duration::from_millis(
                        100 * (2_u64.pow(attempt as u32))
                    );
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

## 📝 文档待办

### 1. API 文档

需要添加：
- [ ] OpenAPI/Swagger 规范
- [ ] 端点详细文档
- [ ] 错误代码文档
- [ ] 速率限制文档

### 2. 使用示例

需要添加：
- [ ] cURL 示例
- [ ] Python SDK 示例
- [ ] JavaScript SDK 示例
- [ ] 流式响应示例

### 3. 部署文档

需要添加：
- [ ] Docker 部署指南
- [ ] Kubernetes 部署指南
- [ ] 环境变量配置
- [ ] 生产环境优化建议

## 🧪 测试待办

### 1. 集成测试

需要添加：
- [ ] 端到端聊天完成测试
- [ ] 流式响应测试
- [ ] 渠道故障转移测试
- [ ] 限流触发测试

### 2. 性能测试

需要添加：
- [ ] 负载测试
- [ ] 压力测试
- [ ] 并发测试
- [ ] 延迟基准测试

### 3. Mock 测试

需要添加：
- [ ] Mock 上游 API 响应
- [ ] Mock 数据库操作
- [ ] Mock 认证流程

## 🔒 安全待办

### 1. 输入验证

需要添加：
- [ ] 请求大小限制
- [ ] 消息长度限制
- [ ] 模型名称白名单验证
- [ ] API 密钥格式验证

### 2. 速率限制增强

需要添加：
- [ ] 每用户限流
- [ ] 每模型限流
- [ ] 动态限流调整
- [ ] 限流白名单

### 3. 审计日志

需要添加：
- [ ] 请求日志
- [ ] 错误日志
- [ ] 安全事件日志
- [ ] 用量审计

## 📊 监控待办

### 1. 指标收集

需要添加：
- [ ] 请求延迟直方图
- [ ] 错误率指标
- [ ] 渠道健康指标
- [ ] Token 用量指标

### 2. 告警规则

需要定义：
- [ ] 高错误率告警
- [ ] 高延迟告警
- [ ] 渠道故障告警
- [ ] 资源使用告警

### 3. 仪表板

需要创建：
- [ ] 实时请求监控
- [ ] 渠道状态仪表板
- [ ] 用量统计仪表板
- [ ] 成本分析仪表板

## 🚀 性能优化

### 1. 缓存策略

建议实现：
- [ ] 模型列表缓存（Redis）
- [ ] 渠道配置缓存
- [ ] 响应缓存（相同请求）
- [ ] DNS 缓存

### 2. 连接池

建议优化：
- [ ] HTTP 连接池调优
- [ ] 数据库连接池调优
- [ ] Redis 连接池调优

### 3. 异步优化

建议改进：
- [ ] 减少锁竞争
- [ ] 使用无锁数据结构
- [ ] 优化任务调度

## 📋 优先级排序

### P0 (必须完成)
1. ✅ 核心中继逻辑
2. ✅ 基本端点实现
3. ✅ 限流中间件
4. 🔧 认证集成（extract_tenant_id）
5. 🔧 数据库测试支持

### P1 (重要)
1. 🔧 流式响应完善
2. 🔧 成本计算
3. 🔧 重试逻辑
4. 📝 API 文档
5. 🧪 集成测试

### P2 (改进)
1. 📊 监控指标
2. 🔒 安全增强
3. 🚀 性能优化
4. 📝 使用示例
5. 🧪 性能测试

### P3 (可选)
1. 📊 高级仪表板
2. 🔒 审计日志
3. 🚀 高级缓存
4. 📝 部署文档
5. 🧪 Mock 测试

---

**创建日期**: 2026-03-22  
**最后更新**: 2026-03-22  
**状态**: 进行中
