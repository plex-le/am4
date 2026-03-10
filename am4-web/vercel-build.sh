#!/bin/bash
set -e

# 1. 安装 Rust WASM 目标 (Vercel 构建环境自带 Rust，但可能没加 target)
rustup target add wasm32-unknown-unknown

# 2. 安装 Trunk (使用 curl 下载二进制文件，避免 cargo install 编译过慢)
TRUNK_VERSION="v0.21.14"
curl -L "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xz

# 3. 安装 uv (用于运行脚本中定义的 hooks，如生成 PWA sw.js)
curl -LsSf https://astral.sh/uv/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"

# 4. 执行 Trunk 构建
./trunk build --release --minify
