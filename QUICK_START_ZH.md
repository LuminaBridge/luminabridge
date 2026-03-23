# LuminaBridge 快速验证指南

## 前提条件
- 已安装 Rust (rustc 1.75+)
- 已安装 Docker
- PostgreSQL 和 Redis 已运行

## 快速验证步骤

### 1. 拉取最新代码
```bash
cd ~/luminabridge
git pull origin main
```

### 2. 编译（Debug 模式，更快）
```bash
# 编译
cargo build

# 预计时间：10-15 分钟
```

### 3. 运行服务
```bash
# 后台运行
./target/debug/luminabridge &

# 或者使用 nohup
nohup ./target/debug/luminabridge > app.log 2>&1 &
```

### 4. 测试 API
```bash
# 健康检查
curl http://localhost:8080/health

# 预期输出：
# {"status":"ok","timestamp":"2026-03-23T..."}
```

### 5. 查看日志
```bash
# 查看应用日志
tail -f app.log

# 或者查看实时输出
journalctl -f
```

## 常见问题

### 编译错误
```bash
# 查看详细错误
cargo build 2>&1 | tee build.log
grep "error" build.log
```

### 端口冲突
```bash
# 检查端口占用
netstat -tlnp | grep 8080

# 修改配置文件中的端口
# config/config.toml
```

### 数据库连接失败
```bash
# 检查 PostgreSQL 状态
docker ps | grep postgres

# 测试连接
psql -h localhost -U luminabridge -d luminabridge_dev
```

## 成功标志

- ✅ 编译无错误
- ✅ 服务启动成功
- ✅ `curl http://localhost:8080/health` 返回 `{"status":"ok"}`

---

**当前代码版本**: `4605a74`
**提交信息**: "Fix: Add log dependency and update code"
