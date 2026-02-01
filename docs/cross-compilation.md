# Cross 编译使用指南

本文档说明如何使用 `cross` 工具为 keyring-cli 进行跨平台编译。

## 前置要求

1. **Docker**: 需要安装 Docker 或 OrbStack
   - macOS: 推荐 OrbStack (更快) 或 Docker Desktop
   - 验证: `docker ps`

2. **cross 工具**:
   ```bash
   cargo install cross --git https://github.com/cross-rs/cross
   ```
   - 安装后验证: `cross --version`

## 快速开始

### 使用 Makefile (推荐)

```bash
# 构建 Linux x86_64
make cross-linux

# 构建 Linux ARM64
make cross-linux-arm

# 构建 Windows x86_64
make cross-windows

# 构建所有目标平台
make cross-all

# 运行交叉编译测试
make cross-test
```

### 使用 cross 命令

```bash
# 直接使用 cross
cross build --target x86_64-unknown-linux-gnu --release
cross build --target aarch64-unknown-linux-gnu --release
cross build --target x86_64-pc-windows-msvc --release

# 使用 cargo 别名 (在 .cargo/config.toml 中定义)
cargo linux-x64
cargo linux-arm
cargo windows-x64
```

### 使用构建脚本

```bash
# Debug 构建
./scripts/cross-build.sh debug

# Release 构建 (默认)
./scripts/cross-build.sh release
```

输出位置: `dist/debug/` 或 `dist/release/`

## 目标平台

| 目标三元组 | 平台 | 输出文件名 | 状态 |
|-----------|------|-----------|------|
| `x86_64-unknown-linux-gnu` | Linux x86_64 | `ok-linux-x64` | ✅ 支持 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | `ok-linux-arm64` | ✅ 支持 |
| `x86_64-pc-windows-msvc` | Windows x86_64 | `ok-windows-x64.exe` | ⚠️ 使用 CI/CD |

**注意**: Windows 跨平台编译在 macOS 上有已知问题（cross 工具限制）。请使用 GitHub Actions CI/CD 或 Windows 机器进行 Windows 构建。

## 常见问题

### Docker 权限问题

```bash
# macOS: 确保 OrbStack 正在运行
orb

# 验证 Docker 可用
docker ps
```

### 镜像拉取失败

首次运行会自动拉取 Docker 镜像 (约 500MB-1GB)，需要较长时间。

如遇网络问题，可手动预拉取：
```bash
docker pull ghcr.io/cross/x86_64-unknown-linux-gnu:main
docker pull ghcr.io/cross/aarch64-unknown-linux-gnu:main
docker pull ghcr.io/cross/x86_64-pc-windows-msvc:main
```

### 编译错误

如果遇到链接错误，请检查 `Cargo.toml` 中的依赖是否使用了静态链接特性。本项目已使用 `native-tls-vendored`，应该不会有 OpenSSL 链接问题。

## 验证构建

构建完成后，可以在对应平台上运行二进制文件验证：

```bash
# 在 Docker 中验证 Linux 构建
docker run --rm -v "$(pwd)/dist/release:/mnt" ubuntu:latest /mnt/ok-linux-x64 --version

# 在 Windows 上直接运行
ok-windows-x64.exe --version
```

## 与 CI/CD 的关系

- **本地开发**: 使用 cross 进行跨平台编译验证
- **CI/CD**: GitHub Actions 继续使用原生构建 (更快)

两者互不影响，cross 主要用于本地快速验证。
