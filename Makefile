.PHONY: build test check doc clean release

# 默认目标
all: check build test

# 构建项目
build:
	cargo build --release

run:
	cargo run -- $(word 2,$(MAKECMDGOALS))

# 运行测试
test:
	cargo test --all-features

# 代码检查
check:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings
	cargo doc --no-deps --document-private-items

# 生成文档
doc:
	cargo doc --no-deps --document-private-items --open

# 清理构建文件
clean:
	cargo clean

# 发布版本
release:
	cargo publish

# 格式化代码
fmt:
	cargo fmt --all

# 安装
install:
	cargo install --path .

# 开发模式构建
dev:
	cargo build

.PHONY: run
DEFAULT_GOAL: help

%:
	@: