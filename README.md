# LuminaBridge

🌉 **Illuminating AI Connections | 照亮 AI 连接**

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://github.com/LuminaBridge/luminabridge/actions/workflows/ci.yml/badge.svg)](https://github.com/LuminaBridge/luminabridge/actions)

---

## 📖 Overview | 项目介绍

**English:**

LuminaBridge is a next-generation high-performance AI gateway built with Rust. It unifies access to 50+ large language models (LLMs) through a single, OpenAI-compatible API endpoint. Designed for scalability, reliability, and developer experience, LuminaBridge serves as the bridge between your applications and the diverse AI ecosystem.

**中文:**

LuminaBridge 是一个基于 Rust 构建的下一代高性能 AI 网关。它通过单一的 OpenAI 兼容 API 端点，统一访问 50+ 个大语言模型（LLM）。LuminaBridge 专为可扩展性、可靠性和开发者体验而设计，是您应用程序与多样化 AI 生态系统之间的桥梁。

---

## ✨ Core Features | 核心特性

- **🚀 High Performance | 高性能**
  - Built with Rust for blazing-fast response times and minimal resource consumption
  - 采用 Rust 构建，响应速度极快，资源消耗极低

- **🔗 Unified API | 统一 API**
  - OpenAI-compatible interface supporting 50+ LLM providers
  - OpenAI 兼容接口，支持 50+ LLM 提供商

- **🔐 Enterprise Security | 企业级安全**
  - OAuth 2.0 authentication (GitHub, Discord, and more)
  - Rate limiting, quota management, and access control
  - OAuth 2.0 认证（GitHub、Discord 等）
  - 速率限制、配额管理和访问控制

- **📊 Real-time Analytics | 实时分析**
  - Comprehensive usage metrics and monitoring dashboard
  - 全面的使用指标和监控仪表板

- **⚖️ Intelligent Load Balancing | 智能负载均衡**
  - Weighted random, round-robin, and least-connection strategies
  - 加权随机、轮询和最少连接策略

- **🛡️ High Availability | 高可用性**
  - Automatic failover and health checking
  - 自动故障转移和健康检查

- **🔌 Extensible Architecture | 可扩展架构**
  - Plugin system for custom providers and middleware
  - 用于自定义提供商和中间件的插件系统

---

## 🚀 Quick Start | 快速开始

### Docker Deployment | Docker 部署

**English:**

The fastest way to get started with LuminaBridge is using Docker:

```bash
# Pull the latest image
docker pull ghcr.io/luminabridge/luminabridge:latest

# Run with default configuration
docker run -d \
  --name luminabridge \
  -p 3000:3000 \
  -v $(pwd)/config:/app/config \
  ghcr.io/luminabridge/luminabridge:latest

# Access the API
curl http://localhost:3000/v1/models
```

**中文:**

使用 Docker 是快速启动 LuminaBridge 的最快方式：

```bash
# 拉取最新镜像
docker pull ghcr.io/luminabridge/luminabridge:latest

# 使用默认配置运行
docker run -d \
  --name luminabridge \
  -p 3000:3000 \
  -v $(pwd)/config:/app/config \
  ghcr.io/luminabridge/luminabridge:latest

# 访问 API
curl http://localhost:3000/v1/models
```

### Building from Source | 源码构建

**English:**

```bash
# Clone the repository
git clone https://github.com/LuminaBridge/luminabridge.git
cd luminabridge

# Ensure Rust is installed (requires Rust 1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build in release mode
cargo build --release

# Run the server
./target/release/luminabridge

# Or use Make (if available)
make build
make run
```

**中文:**

```bash
# 克隆仓库
git clone https://github.com/LuminaBridge/luminabridge.git
cd luminabridge

# 确保已安装 Rust（需要 Rust 1.75+）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 以 release 模式构建
cargo build --release

# 运行服务器
./target/release/luminabridge

# 或使用 Make（如果可用）
make build
make run
```

---

## ⚙️ Configuration | 配置说明

### Basic Configuration | 基础配置

**English:**

LuminaBridge uses a YAML configuration file located at `config/config.yml`:

```yaml
# Server configuration
server:
  host: 0.0.0.0
  port: 3000
  log_level: info

# Database configuration
database:
  url: postgresql://user:password@localhost:5432/luminabridge
  max_connections: 100

# OAuth configuration
oauth:
  github:
    enabled: true
    client_id: your_github_client_id
    client_secret: your_github_client_secret
    redirect_uri: http://localhost:3000/oauth/github/callback
  discord:
    enabled: true
    client_id: your_discord_client_id
    client_secret: your_discord_client_secret
    redirect_uri: http://localhost:3000/oauth/discord/callback

# Rate limiting
rate_limit:
  requests_per_minute: 60
  tokens_per_minute: 100000

# AI Provider channels
channels:
  - name: openai
    type: openai
    api_key: sk-your-openai-key
    base_url: https://api.openai.com/v1
    models: [gpt-4, gpt-3.5-turbo]
    weight: 10
  
  - name: anthropic
    type: anthropic
    api_key: your_anthropic_key
    base_url: https://api.anthropic.com
    models: [claude-3-opus, claude-3-sonnet]
    weight: 5
```

**中文:**

LuminaBridge 使用位于 `config/config.yml` 的 YAML 配置文件：

```yaml
# 服务器配置
server:
  host: 0.0.0.0
  port: 3000
  log_level: info

# 数据库配置
database:
  url: postgresql://user:password@localhost:5432/luminabridge
  max_connections: 100

# OAuth 配置
oauth:
  github:
    enabled: true
    client_id: your_github_client_id
    client_secret: your_github_client_secret
    redirect_uri: http://localhost:3000/oauth/github/callback
  discord:
    enabled: true
    client_id: your_discord_client_id
    client_secret: your_discord_client_secret
    redirect_uri: http://localhost:3000/oauth/discord/callback

# 速率限制
rate_limit:
  requests_per_minute: 60
  tokens_per_minute: 100000

# AI 提供商渠道
channels:
  - name: openai
    type: openai
    api_key: sk-your-openai-key
    base_url: https://api.openai.com/v1
    models: [gpt-4, gpt-3.5-turbo]
    weight: 10
  
  - name: anthropic
    type: anthropic
    api_key: your_anthropic_key
    base_url: https://api.anthropic.com
    models: [claude-3-opus, claude-3-sonnet]
    weight: 5
```

### Environment Variables | 环境变量

**English:**

You can also configure LuminaBridge using environment variables:

```bash
LUMINABRIDGE_HOST=0.0.0.0
LUMINABRIDGE_PORT=3000
LUMINABRIDGE_LOG_LEVEL=info
LUMINABRIDGE_DATABASE_URL=postgresql://user:password@localhost:5432/luminabridge
LUMINABRIDGE_OAUTH_GITHUB_CLIENT_ID=your_client_id
LUMINABRIDGE_OAUTH_GITHUB_CLIENT_SECRET=your_client_secret
```

**中文:**

您也可以使用环境变量配置 LuminaBridge：

```bash
LUMINABRIDGE_HOST=0.0.0.0
LUMINABRIDGE_PORT=3000
LUMINABRIDGE_LOG_LEVEL=info
LUMINABRIDGE_DATABASE_URL=postgresql://user:password@localhost:5432/luminabridge
LUMINABRIDGE_OAUTH_GITHUB_CLIENT_ID=your_client_id
LUMINABRIDGE_OAUTH_GITHUB_CLIENT_SECRET=your_client_secret
```

---

## 📡 API Usage | API 使用示例

### Chat Completion | 聊天补全

**English:**

```bash
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "max_tokens": 1024,
    "temperature": 0.7
  }'
```

**中文:**

```bash
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {
        "role": "user",
        "content": "你好，最近怎么样？"
      }
    ],
    "max_tokens": 1024,
    "temperature": 0.7
  }'
```

### List Models | 列出模型

**English:**

```bash
curl http://localhost:3000/v1/models \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**中文:**

```bash
curl http://localhost:3000/v1/models \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Python SDK Example | Python SDK 示例

**English:**

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:3000/v1",
    api_key="YOUR_API_KEY"
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "Hello!"}
    ]
)

print(response.choices[0].message.content)
```

**中文:**

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:3000/v1",
    api_key="YOUR_API_KEY"
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "你好！"}
    ]
)

print(response.choices[0].message.content)
```

---

## 🤝 Contributing | 贡献指南

**English:**

We welcome contributions from the community! Please read our [Contributing Guide](CONTRIBUTING.md) for details on:

- How to report bugs
- How to suggest new features
- How to submit pull requests
- Code style requirements
- Development environment setup
- Testing requirements

**中文:**

我们欢迎来自社区的贡献！请阅读我们的 [贡献指南](CONTRIBUTING.md)，了解：

- 如何报告 Bug
- 如何建议新功能
- 如何提交 PR
- 代码风格要求
- 开发环境设置
- 测试要求

---

## 📄 License | 许可证

**English:**

LuminaBridge is licensed under the [Apache License 2.0](LICENSE).

**中文:**

LuminaBridge 采用 [Apache License 2.0](LICENSE) 许可证。

---

## 🔗 Links | 相关链接

- **Documentation** | 文档：https://luminabridge.github.io/docs
- **GitHub Organization** | GitHub 组织：https://github.com/LuminaBridge
- **Issue Tracker** | 问题跟踪：https://github.com/LuminaBridge/luminabridge/issues
- **Discord Community** | Discord 社区：https://discord.gg/luminabridge

---

## 📞 Support | 支持

**English:**

- For general questions and discussions, join our [Discord community](https://discord.gg/luminabridge)
- For bug reports and feature requests, please open an [issue](https://github.com/LuminaBridge/luminabridge/issues)
- For security vulnerabilities, please refer to our [Security Policy](SECURITY.md)

**中文:**

- 一般问题和讨论，请加入我们的 [Discord 社区](https://discord.gg/luminabridge)
- Bug 报告和功能请求，请提交 [issue](https://github.com/LuminaBridge/luminabridge/issues)
- 安全漏洞，请参阅我们的 [安全政策](SECURITY.md)

---

<div align="center">

**🌉 LuminaBridge - Illuminating AI Connections**

**照亮 AI 连接**

Made with ❤️ by the LuminaBridge Team

</div>
