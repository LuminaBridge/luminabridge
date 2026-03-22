# 🔌 LuminaBridge 后端 API 开发计划

**版本**: v1.0  
**创建时间**: 2026-03-22  
**目标**: 确保前后端能正常交互

---

## 📋 API 设计原则

1. **RESTful 风格** - 资源导向、语义化 URL
2. **版本控制** - `/api/v1/` 前缀
3. **统一响应格式** - 成功/错误格式一致
4. **认证授权** - JWT + OAuth 2.0
5. **多租户隔离** - 每个请求自动关联租户
6. **限流保护** - 防止滥用
7. **文档完善** - OpenAPI/Swagger 文档

---

## 🏗️ API 路由结构

```
/api/v1/
├── auth/                    # 认证相关
│   ├── POST /login         # 邮箱密码登录
│   ├── POST /logout        # 登出
│   ├── POST /refresh       # 刷新 Token
│   ├── POST /register      # 注册
│   ├── POST /forgot-password  # 忘记密码
│   └── /oauth/             # OAuth 登录
│       ├── GET /github     # GitHub OAuth
│       ├── GET /discord    # Discord OAuth
│       └── GET /google     # Google OAuth
│
├── tenant/                  # 租户相关
│   ├── GET /               # 获取当前租户信息
│   ├── PUT /               # 更新租户配置
│   ├── GET /usage          # 租户用量统计
│   └── GET /members        # 租户成员列表
│
├── channels/                # 渠道管理
│   ├── GET /               # 获取渠道列表
│   ├── POST /              # 创建渠道
│   ├── GET /:id            # 获取渠道详情
│   ├── PUT /:id            # 更新渠道
│   ├── DELETE /:id         # 删除渠道
│   ├── POST /:id/test      # 测试渠道
│   ├── POST /:id/enable    # 启用渠道
│   ├── POST /:id/disable   # 禁用渠道
│   └── POST /batch         # 批量操作
│
├── tokens/                  # 令牌管理
│   ├── GET /               # 获取令牌列表
│   ├── POST /              # 创建令牌
│   ├── GET /:id            # 获取令牌详情
│   ├── DELETE /:id         # 删除令牌
│   ├── PATCH /:id/quota    # 更新令牌额度
│   └── POST /:id/regenerate # 重新生成令牌
│
├── users/                   # 用户管理
│   ├── GET /               # 获取用户列表
│   ├── GET /:id            # 获取用户详情
│   ├── PUT /:id            # 更新用户
│   ├── DELETE /:id         # 删除用户
│   ├── POST /invite        # 邀请用户
│   └── GET /:id/usage      # 用户用量统计
│
├── stats/                   # 统计分析
│   ├── GET /realtime       # 实时统计
│   ├── GET /usage          # 用量统计
│   ├── GET /channels       # 渠道统计
│   ├── GET /models         # 模型统计
│   └── GET /billing        # 计费统计
│
├── models/                  # 模型管理
│   ├── GET /               # 获取模型列表
│   ├── GET /:id            # 获取模型详情
│   └── GET /:id/pricing    # 模型定价
│
├── billing/                 # 计费管理
│   ├── GET /invoices       # 获取账单
│   ├── GET /invoices/:id   # 账单详情
│   ├── POST /recharge      # 充值
│   └── GET /payment-methods # 支付方式
│
├── logs/                    # 日志管理
│   ├── GET /api            # API 日志
│   ├── GET /audit          # 审计日志
│   └── GET /error          # 错误日志
│
└── settings/                # 系统设置
    ├── GET /               # 获取设置
    ├── PUT /               # 更新设置
    ├── GET /notification   # 通知设置
    └── GET /security       # 安全设置
```

---

## 📝 统一响应格式

### 成功响应

```typescript
interface SuccessResponse<T> {
  success: true;
  data: T;
  message?: string;
  meta?: {
    page?: number;
    page_size?: number;
    total?: number;
    total_pages?: number;
  };
}

// 示例
{
  "success": true,
  "data": {
    "id": 1,
    "name": "OpenAI-1",
    "type": "openai",
    "status": "active"
  },
  "message": "操作成功"
}
```

### 错误响应

```typescript
interface ErrorResponse {
  success: false;
  error: {
    code: string;
    message: string;
    details?: any;
  };
}

// 示例
{
  "success": false,
  "error": {
    "code": "CHANNEL_NOT_FOUND",
    "message": "渠道不存在",
    "details": {
      "channel_id": 123
    }
  }
}
```

### 错误码定义

```typescript
enum ErrorCode {
  // 通用错误
  INTERNAL_ERROR = "INTERNAL_ERROR",
  INVALID_REQUEST = "INVALID_REQUEST",
  UNAUTHORIZED = "UNAUTHORIZED",
  FORBIDDEN = "FORBIDDEN",
  NOT_FOUND = "NOT_FOUND",
  
  // 认证相关
  INVALID_CREDENTIALS = "INVALID_CREDENTIALS",
  TOKEN_EXPIRED = "TOKEN_EXPIRED",
  TOKEN_INVALID = "TOKEN_INVALID",
  OAUTH_FAILED = "OAUTH_FAILED",
  
  // 渠道相关
  CHANNEL_NOT_FOUND = "CHANNEL_NOT_FOUND",
  CHANNEL_ALREADY_EXISTS = "CHANNEL_ALREADY_EXISTS",
  CHANNEL_TEST_FAILED = "CHANNEL_TEST_FAILED",
  
  // 令牌相关
  TOKEN_NOT_FOUND = "TOKEN_NOT_FOUND",
  TOKEN_QUOTA_EXCEEDED = "TOKEN_QUOTA_EXCEEDED",
  TOKEN_EXPIRED = "TOKEN_EXPIRED",
  
  // 租户相关
  TENANT_NOT_FOUND = "TENANT_NOT_FOUND",
  TENANT_QUOTA_EXCEEDED = "TENANT_QUOTA_EXCEEDED",
  
  // 用户相关
  USER_NOT_FOUND = "USER_NOT_FOUND",
  USER_ALREADY_EXISTS = "USER_ALREADY_EXISTS",
  USER_INACTIVE = "USER_INACTIVE",
}
```

---

## 🔐 认证中间件

### JWT Token 结构

```typescript
interface JWTPayload {
  sub: string;        // 用户 ID
  email: string;      // 邮箱
  tenant_id: string;  // 租户 ID
  role: string;       // 角色
  iat: number;        // 签发时间
  exp: number;        // 过期时间
}
```

### 中间件实现

```rust
// src/middleware/auth.rs

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_http::auth::AsyncRequireAuthorizationLayer;

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub async fn verify_token<B>(
        State(state): State<AppState>,
        mut req: Request<B>,
        next: Next<B>,
    ) -> Result<Response, StatusCode> {
        // 从 Header 获取 Token
        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());
        
        // 验证 Token
        let token = Self::extract_token(auth_header)?;
        let claims = Self::verify_jwt(&token, &state.jwt_secret)?;
        
        // 将用户信息注入请求
        req.extensions_mut().insert(claims);
        
        Ok(next.run(req).await)
    }
    
    fn extract_token(&self, auth_header: Option<&str>) -> Result<&str, StatusCode> {
        // Bearer token 提取逻辑
    }
    
    fn verify_jwt(&self, token: &str, secret: &str) -> Result<JWTPayload, StatusCode> {
        // JWT 验证逻辑
    }
}
```

---

## 📊 核心 API 实现

### 1. 认证 API

```rust
// src/routes/auth.rs

use axum::{
    routing::{get, post},
    Router,
};

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/register", post(register))
        .route("/oauth/github", get(github_oauth))
        .route("/oauth/github/callback", get(github_callback))
        .route("/oauth/discord", get(discord_oauth))
        .route("/oauth/discord/callback", get(discord_callback))
        .with_state(state)
}

// 登录处理
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<SuccessResponse<LoginResponse>>, AppError> {
    // 1. 验证邮箱密码
    let user = state.db.find_user_by_email(&payload.email).await?;
    
    // 2. 验证密码
    let valid = verify_password(&payload.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }
    
    // 3. 生成 JWT Token
    let token = generate_jwt(&user, &state.jwt_secret)?;
    
    // 4. 生成刷新 Token
    let refresh_token = generate_refresh_token(&user.id)?;
    
    Ok(Json(SuccessResponse {
        success: true,
        data: LoginResponse {
            token,
            refresh_token,
            user: UserDTO::from(user),
        },
        message: Some("登录成功".to_string()),
        meta: None,
    }))
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    refresh_token: String,
    user: UserDTO,
}
```

### 2. 渠道管理 API

```rust
// src/routes/channels.rs

use axum::{
    routing::{get, post, put, delete},
    Router,
};

pub fn channel_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_channels))
        .route("/", post(create_channel))
        .route("/:id", get(get_channel))
        .route("/:id", put(update_channel))
        .route("/:id", delete(delete_channel))
        .route("/:id/test", post(test_channel))
        .route("/:id/enable", post(enable_channel))
        .route("/:id/disable", post(disable_channel))
        .route("/batch", post(batch_operation))
        .layer(AuthMiddleware::layer())
        .with_state(state)
}

// 获取渠道列表
async fn list_channels(
    State(state): State<AppState>,
    Extension(claims): Extension<JWTPayload>,
    Query(params): Query<ChannelListParams>,
) -> Result<Json<SuccessResponse<Vec<ChannelDTO>>>, AppError> {
    // 1. 获取租户 ID
    let tenant_id = claims.tenant_id;
    
    // 2. 查询渠道列表
    let channels = state.db
        .find_channels_by_tenant(tenant_id, &params)
        .await?;
    
    // 3. 获取总数
    let total = state.db.count_channels(tenant_id, &params).await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        data: channels.into_iter().map(ChannelDTO::from).collect(),
        message: None,
        meta: Some(ResponseMeta {
            page: params.page,
            page_size: params.page_size,
            total,
            total_pages: (total as f64 / params.page_size as f64).ceil() as i64,
        }),
    }))
}

// 创建渠道
async fn create_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<JWTPayload>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Json<SuccessResponse<ChannelDTO>>, AppError> {
    let tenant_id = claims.tenant_id;
    
    // 1. 验证渠道名称唯一性
    let exists = state.db.channel_exists(tenant_id, &payload.name).await?;
    if exists {
        return Err(AppError::ChannelAlreadyExists);
    }
    
    // 2. 创建渠道
    let channel = Channel {
        tenant_id,
        name: payload.name,
        channel_type: payload.channel_type,
        key: payload.key,
        base_url: payload.base_url,
        models: payload.models,
        weight: payload.weight.unwrap_or(10),
        status: ChannelStatus::Active,
        ..Default::default()
    };
    
    let channel = state.db.create_channel(channel).await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        data: ChannelDTO::from(channel),
        message: Some("渠道创建成功".to_string()),
        meta: None,
    }))
}

// 测试渠道
async fn test_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<JWTPayload>,
    Path(channel_id): Path<i64>,
) -> Result<Json<SuccessResponse<TestChannelResponse>>, AppError> {
    // 1. 获取渠道
    let channel = state.db.find_channel(channel_id).await?;
    
    // 2. 验证渠道属于当前租户
    if channel.tenant_id != claims.tenant_id {
        return Err(AppError::Forbidden);
    }
    
    // 3. 发送测试请求
    let start = Instant::now();
    let result = test_channel_connection(&channel).await;
    let latency = start.elapsed().as_millis();
    
    let response = match result {
        Ok(_) => TestChannelResponse {
            success: true,
            latency_ms: latency as i64,
            message: "测试成功".to_string(),
        },
        Err(e) => TestChannelResponse {
            success: false,
            latency_ms: latency as i64,
            message: format!("测试失败：{}", e),
        },
    };
    
    Ok(Json(SuccessResponse {
        success: true,
        data: response,
        message: None,
        meta: None,
    }))
}

#[derive(Debug, Deserialize)]
struct ChannelListParams {
    page: Option<i64>,
    page_size: Option<i64>,
    group: Option<String>,
    status: Option<String>,
    channel_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateChannelRequest {
    name: String,
    channel_type: String,
    key: String,
    base_url: Option<String>,
    models: Vec<String>,
    weight: Option<i32>,
}

#[derive(Debug, Serialize)]
struct TestChannelResponse {
    success: bool,
    latency_ms: i64,
    message: String,
}
```

### 3. 统计 API

```rust
// src/routes/stats.rs

async fn get_realtime_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<JWTPayload>,
) -> Result<Json<SuccessResponse<RealtimeStats>>, AppError> {
    let tenant_id = claims.tenant_id;
    
    // 从 Redis 获取实时数据
    let stats = state.redis
        .get_realtime_stats(tenant_id)
        .await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        data: stats,
        message: None,
        meta: None,
    }))
}

async fn get_usage_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<JWTPayload>,
    Query(params): Query<UsageStatsParams>,
) -> Result<Json<SuccessResponse<Vec<UsageStat>>>, AppError> {
    let tenant_id = claims.tenant_id;
    
    let stats = state.db
        .get_usage_stats(tenant_id, &params.start, &params.end, &params.group_by)
        .await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        data: stats,
        message: None,
        meta: None,
    }))
}

#[derive(Debug, Serialize)]
struct RealtimeStats {
    tps: i64,          // 每秒请求数
    rpm: i64,          // 每分钟请求数
    latency_ms: f64,   // 平均延迟
    error_rate: f64,   // 错误率
    active_channels: i64,  // 活跃渠道数
}

#[derive(Debug, Deserialize)]
struct UsageStatsParams {
    start: String,     // YYYY-MM-DD
    end: String,
    group_by: String,  // day, hour, minute
}
```

---

## 🔌 WebSocket 实现

```rust
// src/routes/ws.rs

use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message, CloseCode},
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(claims): Extension<JWTPayload>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, claims, state))
}

async fn handle_socket(
    socket: WebSocket,
    claims: JWTPayload,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();
    let tenant_id = claims.tenant_id;
    
    // 订阅实时数据
    let mut rx = state.stats_sender.subscribe();
    
    // 发送任务
    let send_task = tokio::spawn(async move {
        while let Ok(stats) = rx.recv().await {
            // 只发送当前租户的数据
            if stats.tenant_id == tenant_id {
                let msg = serde_json::to_string(&stats).unwrap();
                if sender.send(Message::Text(msg)).await.is_err()) {
                    break;
                }
            }
        }
    });
    
    // 接收任务（处理客户端消息）
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // 处理客户端消息（如心跳）
                    if text == "ping" {
                        let _ = sender.send(Message::Text("pong".to_string())).await;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });
    
    // 等待任一任务结束
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

#[derive(Debug, Serialize, Clone)]
struct RealtimeStatsMessage {
    tenant_id: String,
    tps: i64,
    rpm: i64,
    latency_ms: f64,
    error_rate: f64,
    timestamp: i64,
}
```

---

## 🗄️ 数据库模型

### 租户表

```sql
CREATE TABLE tenants (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    quota_limit BIGINT DEFAULT 0,
    quota_used BIGINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 用户表

```sql
CREATE TABLE users (
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
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
```

### 渠道表

```sql
CREATE TABLE channels (
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
);

CREATE INDEX idx_channels_tenant_id ON channels(tenant_id);
CREATE INDEX idx_channels_status ON channels(status);
```

### 令牌表

```sql
CREATE TABLE tokens (
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
);

CREATE INDEX idx_tokens_tenant_id ON tokens(tenant_id);
CREATE INDEX idx_tokens_key ON tokens(key);
```

### 用量统计表

```sql
CREATE TABLE usage_stats (
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
);

CREATE INDEX idx_usage_stats_tenant_id ON usage_stats(tenant_id);
CREATE INDEX idx_usage_stats_created_at ON usage_stats(created_at);
CREATE INDEX idx_usage_stats_channel_id ON usage_stats(channel_id);
```

---

## 📦 项目结构

```
luminabridge/
├── src/
│   ├── main.rs              # 主入口
│   ├── lib.rs               # 库导出
│   ├── config/
│   │   └── mod.rs           # 配置管理
│   ├── db/
│   │   ├── mod.rs           # 数据库连接
│   │   ├── models.rs        # 数据模型
│   │   └── repositories.rs  # 数据访问层
│   ├── auth/
│   │   ├── mod.rs           # 认证模块
│   │   ├── jwt.rs           # JWT 处理
│   │   ├── oauth/
│   │   │   ├── mod.rs
│   │   │   ├── github.rs    # GitHub OAuth
│   │   │   └── discord.rs   # Discord OAuth
│   │   └── middleware.rs    # 认证中间件
│   ├── routes/
│   │   ├── mod.rs           # 路由注册
│   │   ├── auth.rs          # 认证路由
│   │   ├── channels.rs      # 渠道路由
│   │   ├── tokens.rs        # 令牌路由
│   │   ├── users.rs         # 用户路由
│   │   ├── stats.rs         # 统计路由
│   │   └── ws.rs            # WebSocket 路由
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs          # 认证处理器
│   │   ├── channels.rs      # 渠道处理器
│   │   └── ...
│   ├── services/
│   │   ├── mod.rs
│   │   ├── channel_service.rs
│   │   └── ...
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs          # 认证中间件
│   │   ├── tenant.rs        # 租户中间件
│   │   └── rate_limit.rs    # 限流中间件
│   ├── error.rs             # 错误处理
│   └── types.rs             # 类型定义
├── Cargo.toml
└── ...
```

---

## 🚀 开发优先级

### P0 - 必须完成（前后端对接基础）

1. ✅ 项目结构搭建
2. ✅ 数据库连接和模型
3. ✅ 认证 API（登录、JWT）
4. ✅ 渠道管理 API（CRUD）
5. ✅ 认证中间件
6. ✅ 错误处理

### P1 - 重要功能

1. OAuth 登录（GitHub/Discord）
2. 令牌管理 API
3. 用户管理 API
4. 统计 API
5. WebSocket 实时推送

### P2 - 增强功能

1. 多租户完整支持
2. 计费系统
3. 批量操作
4. 日志系统
5. 限流保护

---

**文档版本**: v1.0  
**创建时间**: 2026-03-22  
**维护者**: LuminaBridge Team
