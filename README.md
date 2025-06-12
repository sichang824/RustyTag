<div align="center">

# RustyTag

### 一个基于 Git 标签的语义化版本管理工具

[English](README_EN.md) | 简体中文

[![Crates.io](https://img.shields.io/crates/v/rustytag.svg)](https://crates.io/crates/rustytag)
[![Downloads](https://img.shields.io/crates/d/rustytag.svg)](https://crates.io/crates/rustytag)
[![License](https://img.shields.io/crates/l/rustytag.svg)](LICENSE)

</div>

## 特性

- 基于 Git 标签的语义化版本管理
- 完整的语义化版本支持
- 自动版本升级（patch/minor/major）
- 本地标签与远程仓库同步
- 版本对比功能
- 轻量快速
- 跨平台支持

## 安装

### 使用 Cargo 安装

```sh
cargo install rustytag
```

### 从源码安装

1. 克隆仓库：

```sh
git clone https://github.com/yourusername/rustytag.git
```

2. 构建项目：

```sh
cd rustytag
cargo build --release
```

## 使用方法

### 基础命令

```sh
# 初始化语义化版本
rustytag init

# 版本升级命令
rustytag patch                    # 升级补丁版本 (例如: 1.0.0 -> 1.0.1)
rustytag patch -V 1.2.3          # 直接设置为指定版本 1.2.3
rustytag minor                    # 升级次要版本 (例如: 1.0.0 -> 1.1.0)
rustytag minor --version 2.0.0   # 直接设置为指定版本 2.0.0
rustytag major                    # 升级主要版本 (例如: 1.0.0 -> 2.0.0)
rustytag major -V 3.0.0          # 直接设置为指定版本 3.0.0

# 标签同步命令
rustytag sync   # 同步本地标签与远程仓库
rustytag reset  # 重置本地标签以匹配远程仓库

# 信息查看命令
rustytag show   # 显示当前版本信息

# 发布管理
rustytag release                  # 创建发布
rustytag release -l               # 列出所有发布
rustytag release --list           # 列出所有发布
rustytag release -t v1.0.0        # 为指定版本创建发布
rustytag release --tag v1.0.0     # 为指定版本创建发布

# 配置管理
rustytag config                   # 显示当前配置信息
rustytag config --set KEY=VALUE  # 设置配置项
```

### 命令详解

#### 版本管理命令

- `init`: 初始化新的 Git 仓库并设置语义化版本
- `patch/minor/major`: 按照语义化版本规范升级版本号
  - 不带参数：自动递增版本号
  - 带 `-V` 或 `--version` 参数：直接设置为指定版本

#### 标签同步命令

- `sync`: 将本地标签与远程仓库同步
- `reset`: 将本地标签重置为与远程仓库一致

#### 信息查看命令

- `show`: 显示当前项目和工具的详细信息

#### 发布管理命令

- `release`: 管理 GitHub 发布
  - 不带参数：为当前版本创建发布
  - `-l` 或 `--list`：列出所有发布
  - `-t` 或 `--tag`：为指定版本创建发布

#### 配置管理命令

- `config`: 配置 RustyTag 设置
  - 不带参数：显示当前配置信息
  - `--set KEY=VALUE`：设置配置项
  - `--global`：设置全局配置
  - `--local`：设置本地配置

## 使用示例

### 基本工作流程

```sh
# 1. 初始化项目
rustytag init

# 2. 开发完成后，升级版本
rustytag patch              # 修复 bug，升级补丁版本
rustytag minor              # 新增功能，升级次要版本  
rustytag major              # 破坏性更改，升级主要版本

# 3. 推送到远程仓库
git push --follow-tags origin main

# 4. 创建 GitHub 发布
rustytag release
```

### 高级用法

```sh
# 直接设置特定版本
rustytag patch -V 1.2.3
rustytag minor --version 2.0.0

# 查看项目信息
rustytag show

# 同步远程标签
rustytag sync

# 配置 GitHub Token（用于发布管理）
rustytag config --set GITHUB_TOKEN=your_token_here
```

## 贡献

欢迎贡献！详情请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- 灵感来源于 Git 的原生标签功能
- 使用 Rust 构建以确保性能和安全性
