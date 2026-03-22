# LuminaBridge Relay API 快速开始指南

## 🚀 5 分钟快速开始

### 1. 配置渠道

首先，在数据库中配置至少一个 AI 渠道：

```sql
-- 配置 OpenAI 渠道
INSERT INTO channels (
  tenant_id, name, channel_type, key, base_url,
  models, weight, status, priority, timeout_ms, retry_count
) VALUES (
  1,
  'OpenAI Primary',
  'openai',
  'sk-your-openai-api-key-here',
  'https://api.openai.com/v1',
  '["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"]',
  10,
  'active',
  0,
  30000,
  3
);

-- 配置 Anthropic 渠道（可选）
INSERT INTO channels (
  tenant_id, name, channel_type, key, base_url,
  models, weight, status
) VALUES (
  1,
  'Anthropic Claude',
  'anthropic',
  'sk-ant-your-anthropic-key-here',
  'https://api.anthropic.com/v1',
  '["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"]',
  5,
  'active'
);
```

### 2. 启动服务器

```bash
# 开发模式
cargo run

# 或者设置环境变量
export LUMINABRIDGE__DATABASE__URL=postgres://localhost/luminabridge
export LUMINABRIDGE__SERVER__PORT=3000
cargo run
```

### 3. 测试端点

#### 健康检查
```bash
curl http://localhost:3000/health
```

#### 获取模型列表
```bash
curl -X GET http://localhost:3000/v1/models \
  -H "Authorization: Bearer any-token"
```

#### 聊天完成
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer any-token" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ],
    "temperature": 0.7
  }'
```

#### 流式聊天完成
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer any-token" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Tell me a short story"}
    ],
    "stream": true
  }'
```

## 📝 API 参考

### 端点列表

| 端点 | 方法 | 描述 | 认证 |
|------|------|------|------|
| `/v1/chat/completions` | POST | 聊天完成 | Bearer Token |
| `/v1/completions` | POST | 文本完成 | Bearer Token |
| `/v1/models` | GET | 模型列表 | Bearer Token |
| `/v1/models/:id` | GET | 模型详情 | Bearer Token |

### 请求格式

#### Chat Completions

```json
{
  "model": "gpt-3.5-turbo",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 100,
  "stream": false
}
```

#### 支持的参数

- `model` (必需): 模型名称
- `messages` (必需): 消息数组
- `temperature`: 温度 (0.0 - 2.0)
- `max_tokens`: 最大令牌数
- `top_p`: Top P 采样
- `stream`: 是否流式响应
- `stop`: 停止序列
- `presence_penalty`: 存在惩罚
- `frequency_penalty`: 频率惩罚

### 响应格式

#### 成功响应

```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-3.5-turbo",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! How can I help you?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

#### 错误响应

```json
{
  "error": {
    "code": "PROVIDER_ERROR",
    "message": "Provider error: 401 - Invalid API key",
    "type": "luminabridge_error"
  }
}
```

### 错误代码

| 错误代码 | HTTP 状态码 | 描述 |
|----------|------------|------|
| `PROVIDER_ERROR` | 502 | 上游提供商错误 |
| `RATE_LIMIT_EXCEEDED` | 429 | 超过速率限制 |
| `AUTH_ERROR` | 401 | 认证失败 |
| `INVALID_REQUEST` | 400 | 请求格式错误 |

## 🔧 配置选项

### 环境变量

```bash
# 服务器配置
LUMINABRIDGE__SERVER__HOST=0.0.0.0
LUMINABRIDGE__SERVER__PORT=3000
LUMINABRIDGE__SERVER__TIMEOUT_SECS=30

# 数据库配置
LUMINABRIDGE__DATABASE__URL=postgres://user:pass@localhost/dbname
LUMINABRIDGE__DATABASE__MAX_CONNECTIONS=10

# 速率限制
LUMINABRIDGE__RATE_LIMIT__ENABLED=true
LUMINABRIDGE__RATE_LIMIT__REQUESTS_PER_SEC=100
LUMINABRIDGE__RATE_LIMIT__BURST_SIZE=50

# OAuth/JWT
LUMINABRIDGE__OAUTH__JWT_SECRET=your-secret-key-min-32-chars
```

### YAML 配置

创建 `config/config.yml`:

```yaml
server:
  host: "0.0.0.0"
  port: 3000
  timeout_secs: 30

database:
  url: "postgres://localhost/luminabridge"
  max_connections: 10

rate_limit:
  enabled: true
  requests_per_sec: 100
  burst_size: 50

oauth:
  jwt_secret: "your-secret-key"
  token_expiration_secs: 86400

logging:
  level: "info"
  format: "json"
```

## 🎯 高级用法

### 负载均衡

配置多个相同模型的渠道，自动负载均衡：

```sql
-- 主渠道（权重 10）
INSERT INTO channels VALUES (..., 'OpenAI Primary', ..., '["gpt-3.5-turbo"]', 10, ...);

-- 备用渠道（权重 5）
INSERT INTO channels VALUES (..., 'OpenAI Backup', ..., '["gpt-3.5-turbo"]', 5, ...);
```

### 渠道优先级

使用 `priority` 字段设置优先级：

```sql
-- 高优先级渠道
UPDATE channels SET priority = 10 WHERE name = 'Premium Channel';

-- 低优先级渠道
UPDATE channels SET priority = 0 WHERE name = 'Standard Channel';
```

### 流式响应处理

JavaScript 示例：

```javascript
const response = await fetch('http://localhost:3000/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer your-token',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'gpt-3.5-turbo',
    messages: [{ role: 'user', content: 'Hello' }],
    stream: true,
  }),
});

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  
  const chunk = decoder.decode(value);
  // 解析 SSE 数据
  for (const line of chunk.split('\n')) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') continue;
      
      const parsed = JSON.parse(data);
      console.log(parsed.choices[0].delta.content);
    }
  }
}
```

## 🧪 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行中继测试
cargo test relay

# 运行限流测试
cargo test rate_limit

# 运行单个测试
cargo test test_chat_completion_request_serialization
```

### 性能测试

使用 `wrk` 或 `ab` 进行压力测试：

```bash
# 使用 wrk
wrk -t12 -c400 -d30s \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  --data '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"test"}]}' \
  http://localhost:3000/v1/chat/completions
```

## 📊 监控

### 健康检查

```bash
# 基本健康检查
curl http://localhost:3000/health

# 就绪检查（包含数据库）
curl http://localhost:3000/ready
```

### 日志

查看应用日志：

```bash
# 开发模式（详细日志）
RUST_LOG=debug cargo run

# 生产模式（仅错误）
RUST_LOG=error cargo run
```

## ❓ 常见问题

### Q: 如何添加新的 AI 提供商？

A: 在数据库中插入新渠道记录：

```sql
INSERT INTO channels (
  tenant_id, name, channel_type, key, base_url,
  models, weight, status
) VALUES (
  1,
  'Custom Provider',
  'openai',  -- 使用 OpenAI 兼容格式
  'your-api-key',
  'https://api.custom-provider.com/v1',
  '["model-1", "model-2"]',
  5,
  'active'
);
```

### Q: 如何禁用速率限制？

A: 在配置中设置：

```yaml
rate_limit:
  enabled: false
```

或环境变量：
```bash
LUMINABRIDGE__RATE_LIMIT__ENABLED=false
```

### Q: 如何查看用量统计？

A: 查询数据库：

```sql
SELECT 
  model,
  SUM(prompt_tokens) as total_prompt,
  SUM(completion_tokens) as total_completion,
  SUM(total_tokens) as total_tokens,
  COUNT(*) as request_count
FROM usage_stats
WHERE tenant_id = 1
GROUP BY model
ORDER BY total_tokens DESC;
```

### Q: 流式响应不工作？

A: 确保：
1. 请求中包含 `"stream": true`
2. 客户端支持 SSE 格式
3. 中间件没有缓冲响应（如 nginx 需要特殊配置）

Nginx 配置示例：
```nginx
location /v1/ {
    proxy_buffering off;
    proxy_cache off;
    proxy_pass http://localhost:3000;
}
```

## 📚 更多资源

- [实现报告](RELAY_IMPLEMENTATION_REPORT.md) - 详细实现文档
- [待办事项](RELAY_API_TODO.md) - 已知问题和改进计划
- [API 总结](RELAY_API_SUMMARY.md) - 功能总结

---

**最后更新**: 2026-03-22  
**版本**: 0.1.0  
**状态**: Beta
