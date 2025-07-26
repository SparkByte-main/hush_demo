#!/bin/bash

# Zig Web 应用构建脚本 | Zig Web Application Build Script

set -e  # 遇到错误立即退出 | Exit on error

echo "🚀 构建 Zig Web 应用..."

# 1. 构建 Rust 库 | Build Rust library
echo "📦 构建 Rust 库..."
cargo build --lib

# 2. 构建 Zig 应用 | Build Zig application  
echo "⚡ 构建 Zig 应用..."
zig build-exe zig-test/web_test.zig -lhush_demo -L./target/debug

echo "✅ 构建完成！"
echo ""
echo "🌐 启动服务器："
echo "   ./web_test"
echo ""
echo "📡 测试接口："
echo "   curl http://127.0.0.1:8080/"
echo "   curl http://127.0.0.1:8080/about"