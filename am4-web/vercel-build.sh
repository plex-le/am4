#!/bin/bash
set -e

echo "--- Bootstraping Build Environment ---"

# 1. 确保 Rust 环境齐全
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi
rustup target add wasm32-unknown-unknown

# 2. 安装并编译 Trunk (确保针对当前 OS 构建)
echo "Installing Trunk..."
cargo install --locked trunk --version 0.21.14

# 3. 安装 uv (用于处理 PWA sw.js 等脚本)
echo "Installing uv..."
curl -LsSf https://astral.sh/uv/install.sh | sh
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

# 4. 执行 Trunk 构建
echo "Starting Trunk Build..."
trunk build --release --minify
