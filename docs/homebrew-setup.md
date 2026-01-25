# Homebrew Tap Setup

OpenKeyring CLI 通过独立的 tap 仓库提供 Homebrew 安装支持。

## Tap 仓库地址

https://github.com/OpenKeyring/homebrew-tap

## 用户安装方式

```bash
# 添加 tap 并安装
brew tap OpenKeyring/homebrew-tap
brew install OpenKeyring/homebrew-tap/ok

# 或者一步到位（tap 如果不存在会自动添加）
brew install OpenKeyring/homebrew-tap/ok
```

## 维护者指南

### 发布新版本时

1. 在 keyring-cli 发布新版本后，更新 tap 仓库中的 formula

2. 获取发布包的 sha256：

```bash
# 下载发布包
curl -LO https://github.com/open-keyring/keyring-cli/archive/refs/tags/v0.1.0.tar.gz

# 计算 sha256
shasum -a 256 v0.1.0.tar.gz
```

3. 更新 `Formula/ok.rb` 中的 `url` 和 `sha256`

4. 提交并推送到 tap 仓库：

```bash
cd <path-to-homebrew-tap>
vi Formula/ok.rb
git commit -am "Bump version to v0.1.1"
git push
```

### 本地测试 formula

在修改 formula 后，可以本地测试：

```bash
brew install OpenKeyring/homebrew-tap/ok --build-from-source
```
