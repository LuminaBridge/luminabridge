# 修复说明 - home crate edition2024 问题

## 问题
编译时遇到错误：
```
error: failed to parse manifest at `/home/user/.cargo/registry/src/index.crates.io-6f17d22bba15001f/home-0.5.12/Cargo.toml`
Caused by:
  feature `edition2024` is required
  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.84.1).
```

## 原因
`home` crate 的 0.5.12 版本需要 Rust edition 2024，但当前 Cargo 版本 (1.84.1) 不支持该特性。

## 已完成的修复
已在 `Cargo.toml` 中添加 patch 覆盖，将 `home` crate 锁定到 0.5.9 版本：

```toml
[patch.crates-io]
home = "=0.5.9"
```

## 下一步操作
在远程节点上重新编译：

```bash
cd luminabridge
cargo build --release
```

Cargo 会自动使用 patch 覆盖，拉取 home 0.5.9 版本而不是 0.5.12。

## 备选方案
如果上述方法不起作用，可以：
1. 更新 Rust 到最新版本：`rustup update`
2. 或者手动删除 Cargo.lock 后重新生成
