# Contributing to LuminaBridge

🌉 **Illuminating AI Connections | 照亮 AI 连接**

---

## 📋 Table of Contents | 目录

- [Reporting Bugs | 报告 Bug](#reporting-bugs--报告-bug)
- [Suggesting Features | 建议新功能](#suggesting-features--建议新功能)
- [Submitting Pull Requests | 提交 PR](#submitting-pull-requests--提交-pr)
- [Code Style | 代码风格](#code-style--代码风格)
- [Development Setup | 开发环境设置](#development-setup--开发环境设置)
- [Testing | 测试](#testing--测试)

---

## 🐛 Reporting Bugs | 报告 Bug

**English:**

Before creating bug reports, please check existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

**Use the following template:**

```markdown
**Description**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Environment:**
- OS: [e.g. Windows 11, macOS 14, Ubuntu 22.04]
- Rust version: [e.g. 1.75.0]
- LuminaBridge version: [e.g. 0.1.0]

**Additional context**
Add any other context about the problem here.
```

**中文:**

在创建 Bug 报告之前，请检查现有问题，因为您可能会发现不需要创建新问题。创建 Bug 报告时，请尽可能包含更多细节：

**使用以下模板：**

```markdown
**描述**
清晰简洁地描述问题是什么。

**复现步骤**
重现行为的步骤：
1. 前往 '...'
2. 点击 '....'
3. 滚动到 '....'
4. 看到错误

**预期行为**
清晰简洁地描述您期望发生的事情。

**截图**
如果适用，添加截图来帮助解释您的问题。

**环境：**
- 操作系统：[例如 Windows 11, macOS 14, Ubuntu 22.04]
- Rust 版本：[例如 1.75.0]
- LuminaBridge 版本：[例如 0.1.0]

**其他上下文**
在此添加关于问题的任何其他上下文。
```

---

## 💡 Suggesting Features | 建议新功能

**English:**

We welcome feature suggestions! Please use the following template:

```markdown
**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Additional context**
Add any other context or screenshots about the feature request here.
```

**中文:**

我们欢迎功能建议！请使用以下模板：

```markdown
**您的功能请求是否与问题相关？请描述。**
清晰简洁地描述问题是什么。例如：我总是很困扰当 [...]

**描述您想要的解决方案**
清晰简洁地描述您希望发生的事情。

**描述您考虑过的替代方案**
清晰简洁地描述您考虑过的任何替代解决方案或功能。

**其他上下文**
在此添加关于功能请求的任何其他上下文或截图。
```

---

## 🔄 Submitting Pull Requests | 提交 PR

**English:**

### Before Submitting

1. **Fork the repository** and create your branch from `main`
2. **Set up development environment** (see [Development Setup](#development-setup--开发环境设置))
3. **Make your changes** following our code style guidelines
4. **Test your changes** thoroughly
5. **Ensure the test suite passes**
6. **Update documentation** if necessary

### Submitting Process

1. **Create a Pull Request** on GitHub
2. **Use a clear title** following the format: `[Type] Description`
   - Examples: `[Feature] Add Discord OAuth provider`, `[Bugfix] Fix rate limiter panic`
3. **Write a detailed description** of your changes
4. **Link related issues** using `Fixes #123` or `Related to #456`
5. **Wait for review** from maintainers

### PR Checklist | PR 检查清单

- [ ] Code follows style guidelines | 代码遵循风格指南
- [ ] Tests added/updated | 已添加/更新测试
- [ ] Documentation updated | 已更新文档
- [ ] No breaking changes (or clearly marked) | 无破坏性变更（或明确标记）
- [ ] Commit messages are clear | 提交信息清晰

**中文:**

### 提交前

1. **Fork 仓库** 并从 `main` 创建您的分支
2. **设置开发环境**（参见 [开发环境设置](#development-setup--开发环境设置)）
3. **进行更改** 遵循我们的代码风格指南
4. **彻底测试** 您的更改
5. **确保测试套件通过**
6. **更新文档**（如必要）

### 提交流程

1. **在 GitHub 上创建 Pull Request**
2. **使用清晰的标题** 遵循格式：`[类型] 描述`
   - 示例：`[Feature] 添加 Discord OAuth 提供商`，`[Bugfix] 修复速率限制器 panic`
3. **编写详细的变更描述**
4. **链接相关问题** 使用 `Fixes #123` 或 `Related to #456`
5. **等待维护者审查**

---

## 📝 Code Style | 代码风格

**English:**

### Rust Code Style

We follow standard Rust conventions and use `rustfmt` for formatting:

```bash
# Format code
cargo fmt

# Or using Make
make fmt
```

**Key Guidelines:**

- **Naming Conventions:**
  - Types and structs: `PascalCase` (e.g., `OAuthProvider`, `UserInfo`)
  - Functions and methods: `snake_case` (e.g., `get_user_info`, `exchange_code`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_CONNECTIONS`, `API_VERSION`)
  - Private fields: prefix with `_` if intentionally unused (e.g., `_phantom`)

- **Error Handling:**
  - Use `Result<T, E>` for recoverable errors
  - Use `panic!` only for unrecoverable errors
  - Implement `std::error::Error` for custom error types

- **Documentation:**
  - Use `///` for public API documentation
  - Use `//` for internal comments
  - Include examples in doc comments when applicable

```rust
/// OAuth Provider trait defines the interface for OAuth authentication.
/// 
/// # Example
/// 
/// ```
/// let provider = GitHubProvider::new(config);
/// let url = provider.get_login_url("state");
/// ```
pub trait OAuthProvider {
    /// Returns the provider name
    fn name(&self) -> &str;
    
    /// Exchange authorization code for access token
    async fn exchange_code(&self, code: &str) -> Result<TokenResponse, OAuthError>;
}
```

**中文:**

### Rust 代码风格

我们遵循标准 Rust 约定并使用 `rustfmt` 进行格式化：

```bash
# 格式化代码
cargo fmt

# 或使用 Make
make fmt
```

**关键指南：**

- **命名约定：**
  - 类型和结构体：`PascalCase`（例如 `OAuthProvider`, `UserInfo`）
  - 函数和方法：`snake_case`（例如 `get_user_info`, `exchange_code`）
  - 常量：`SCREAMING_SNAKE_CASE`（例如 `MAX_CONNECTIONS`, `API_VERSION`）
  - 私有字段：如果有意未使用，前缀加 `_`（例如 `_phantom`）

- **错误处理：**
  - 对可恢复错误使用 `Result<T, E>`
  - 仅对不可恢复错误使用 `panic!`
  - 为自定义错误类型实现 `std::error::Error`

- **文档：**
  - 使用 `///` 进行公共 API 文档
  - 使用 `//` 进行内部注释
  - 在文档注释中包含示例（如适用）

---

## 🛠️ Development Setup | 开发环境设置

**English:**

### Prerequisites | 前置要求

- **Rust** 1.75.0 or later ([installation](https://www.rust-lang.org/tools/install))
- **PostgreSQL** 14.0 or later (for database)
- **Git** for version control
- **Make** (optional, for convenience commands)

### Step-by-Step Setup | 分步设置

```bash
# 1. Clone your fork
git clone https://github.com/YOUR_USERNAME/luminabridge.git
cd luminabridge

# 2. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install development tools
rustup component add rustfmt clippy

# 4. Set up PostgreSQL
# macOS
brew install postgresql@14
brew services start postgresql@14

# Ubuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# Create database
createdb luminabridge_dev

# 5. Copy configuration
cp config/config.example.yml config/config.yml

# 6. Update database URL in config
# Edit config/config.yml with your PostgreSQL credentials

# 7. Build the project
cargo build

# 8. Run tests
cargo test

# 9. Run the server
cargo run
# or
make run
```

### IDE Setup | IDE 设置

**VS Code:**
- Install `rust-analyzer` extension
- Install `Even Better TOML` for configuration files

**IntelliJ IDEA:**
- Install `Rust` plugin via JetBrains Marketplace

**中文:**

### 前置要求

- **Rust** 1.75.0 或更高版本（[安装指南](https://www.rust-lang.org/tools/install)）
- **PostgreSQL** 14.0 或更高版本（用于数据库）
- **Git** 用于版本控制
- **Make**（可选，用于便捷命令）

### 分步设置

```bash
# 1. 克隆您的 fork
git clone https://github.com/YOUR_USERNAME/luminabridge.git
cd luminabridge

# 2. 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 安装开发工具
rustup component add rustfmt clippy

# 4. 设置 PostgreSQL
# macOS
brew install postgresql@14
brew services start postgresql@14

# Ubuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# 创建数据库
createdb luminabridge_dev

# 5. 复制配置
cp config/config.example.yml config/config.yml

# 6. 在配置中更新数据库 URL
# 在 config/config.yml 中编辑您的 PostgreSQL 凭据

# 7. 构建项目
cargo build

# 8. 运行测试
cargo test

# 9. 运行服务器
cargo run
# 或
make run
```

### IDE 设置

**VS Code:**
- 安装 `rust-analyzer` 扩展
- 安装 `Even Better TOML` 用于配置文件

**IntelliJ IDEA:**
- 通过 JetBrains Marketplace 安装 `Rust` 插件

---

## ✅ Testing | 测试

**English:**

### Running Tests | 运行测试

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_oauth_github

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Using Make
make test          # Run short tests
make test-coverage # Run with coverage
```

### Writing Tests | 编写测试

**Unit Tests | 单元测试:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_name() {
        let config = ProviderConfig {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_url: "http://localhost".to_string(),
            scopes: vec!["user:email".to_string()],
            extra: None,
        };
        
        let provider = GitHubProvider::new(config);
        assert_eq!(provider.name(), "github");
    }

    #[tokio::test]
    async fn test_exchange_code() {
        // Async test example
        let result = provider.exchange_code("test_code").await;
        assert!(result.is_ok());
    }
}
```

### Test Requirements | 测试要求

- **Unit tests** for all public functions
- **Integration tests** for API endpoints
- **Documentation tests** for public APIs
- **Minimum 80% code coverage** for new features

**中文:**

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行测试并输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_oauth_github

# 运行覆盖率测试（需要 cargo-tarpaulin）
cargo tarpaulin --out Html

# 使用 Make
make test          # 运行简短测试
make test-coverage # 运行覆盖率测试
```

### 编写测试

**单元测试:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_name() {
        let config = ProviderConfig {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_url: "http://localhost".to_string(),
            scopes: vec!["user:email".to_string()],
            extra: None,
        };
        
        let provider = GitHubProvider::new(config);
        assert_eq!(provider.name(), "github");
    }

    #[tokio::test]
    async fn test_exchange_code() {
        // 异步测试示例
        let result = provider.exchange_code("test_code").await;
        assert!(result.is_ok());
    }
}
```

### 测试要求

- 所有公共函数都需要**单元测试**
- API 端点需要**集成测试**
- 公共 API 需要**文档测试**
- 新功能的**最低代码覆盖率 80%**

---

## 📚 Additional Resources | 其他资源

**English:**

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

**中文:**

- [Rust 编程指南](https://kaisery.github.io/trpl-zh-cn/)
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- [Rust 示例](https://doc.rust-lang.org/rust-by-example/)

---

## 🙏 Code of Conduct | 行为准则

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

请注意，本项目采用 [贡献者行为准则](CODE_OF_CONDUCT.md)。参与本项目即表示您同意遵守其条款。

---

## 📞 Getting Help | 获取帮助

**English:**

- Join our [Discord community](https://discord.gg/luminabridge) for real-time help
- Open an [issue](https://github.com/LuminaBridge/luminabridge/issues) for bugs or questions
- Check existing [documentation](https://luminabridge.github.io/docs)

**中文:**

- 加入我们的 [Discord 社区](https://discord.gg/luminabridge) 获取实时帮助
- 为 Bug 或问题提交 [issue](https://github.com/LuminaBridge/luminabridge/issues)
- 查看现有 [文档](https://luminabridge.github.io/docs)

---

<div align="center">

**Thank you for contributing to LuminaBridge!**

**感谢您为 LuminaBridge 做出贡献！**

🌉 Illuminating AI Connections

</div>
