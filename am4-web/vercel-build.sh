#!/bin/bash
set -e

# 1. 安装 Rust WASM 目标 (Vercel 构建环境自带 Rust，但可能没加 target)
rustup target add wasm32-unknown-unknown

# 2. 安装 Trunk (由于二进制包 GLIBC 版本不匹配，改用 cargo install 确保兼容性)
if ! command -v trunk &> /dev/null; then
    echo "Trunk not found, installing..."
    cargo install --locked trunk --version 0.21.14
fi

# 3. 安装 uv (用于运行脚本中定义的 hooks，如生成 PWA sw.js)
if ! command -v uv &> /dev/null; then
    echo "uv not found, installing..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
fi
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

# 4. 执行 Trunk 构建
trunk build --release --minify
