# RustyTag 使用指南

## 目录

1. [安装](#安装)
2. [基础使用](#基础使用)
3. [高级功能](#高级功能)
4. [最佳实践](#最佳实践)

## 安装

### 使用 Cargo 安装

```bash
cargo install rustytag
```

### 从源码安装

```bash
git clone https://github.com/sichang824/rustytag.git
cd rustytag
cargo build --release
```

## 基础使用

### 初始化项目

```bash
rustytag init
```

这个命令会：

- 初始化 Git 仓库（如果尚未初始化）
- 创建 .gitignore 文件
- 设置初始版本

### 版本管理

RustyTag 支持语义化版本管理，提供三种版本升级方式：

```bash
# 升级补丁版本 (1.0.0 -> 1.0.1)
rustytag patch

# 升级次要版本 (1.0.0 -> 1.1.0)
rustytag minor

# 升级主要版本 (1.0.0 -> 2.0.0)
rustytag major
```

### 标签同步

```bash
# 查看并同步标签
rustytag sync

# 重置本地标签以匹配远程
rustytag reset
```

### 查看信息

```bash
# 显示当前版本信息
rustytag show

# 显示特定标签详情
rustytag show v1.0.0
```

## 高级功能

### 发布管理

```bash
# 创建新发布
rustytag release

# 列出所有发布
rustytag release list
```

### 配置管理

```bash
# 设置配置项
rustytag config --set KEY=VALUE

# 常用配置项：
# - CHANGELOG_TEMPLATE: 更新日志模板路径
# - COMMIT_TEMPLATE: 提交信息模板
# - DEFAULT_BRANCH: 默认分支名
```

## 最佳实践

### 版本号管理

- **补丁版本 (patch)**：用于 bug 修复和小改动
- **次要版本 (minor)**：用于新功能添加（向后兼容）
- **主要版本 (major)**：用于破坏性更改

### Git 工作流

1. 确保本地更改已提交
2. 使用适当的命令升级版本
3. 检查更新日志
4. 推送更改和标签到远程

```bash
git add .
git commit -m "feat: add new feature"
rustytag minor  # 升级次要版本
git push origin main --tags
```

### 更新日志最佳实践

- 使用清晰的分类（Added, Changed, Fixed 等）
- 每个更改都提供简短说明
- 包含相关的 issue/PR 链接

### 标签同步策略

1. 定期运行 `rustytag sync` 检查标签状态
2. 在重要操作前使用 `rustytag show` 确认版本
3. 如遇冲突，使用 `rustytag reset` 重置到远程状态

## 常见问题

### Q: 如何处理版本冲突？

A: 使用 `rustytag reset` 重置本地标签，然后重新应用更改。

### Q: 如何回滚版本？

A: 使用 Git 检出特定标签，然后创建新的版本：

```bash
git checkout v1.0.0
rustytag patch
```

### Q: 远程标签同步失败？

A: 检查：

1. Git 远程配置是否正确
2. 是否有推送权限
3. SSH 密钥是否配置正确

## 提示和技巧

1. **自动化发布**
   - 配合 CI/CD 使用
   - 利用 GitHub Actions 自动化发布流程

2. **版本号选择**
   - 新项目从 0.1.0 开始
   - 1.0.0 表示第一个稳定版本
   - 预发布版本使用 -alpha, -beta 后缀

3. **提交信息规范**
   - feat: 新功能
   - fix: 修复
   - docs: 文档更新
   - style: 代码风格
   - refactor: 重构
   - test: 测试相关
   - chore: 构建/工具相关

## 更多资源

- [语义化版本规范](https://semver.org/)
- [RustyTag 文档](https://docs.rs/rustytag)
- [GitHub 仓库](https://github.com/sichang824/rustytag)
- [问题反馈](https://github.com/sichang824/rustytag/issues)
