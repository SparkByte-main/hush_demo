#!/bin/bash

# ============================================================================
# 构建中间件测试脚本 | Build Middleware Test Script
# ============================================================================

set -e  # 遇到错误时退出

echo "Building Hush framework with middleware support..."

# 构建 Rust 库
echo "Building Rust library..."
cargo build --release

# 获取库文件路径
RUST_LIB_PATH="target/release/libhush_demo.dylib"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    RUST_LIB_PATH="target/release/libhush_demo.so"
fi

# 检查库文件是否存在
if [ ! -f "$RUST_LIB_PATH" ]; then
    echo "Error: Rust library not found at $RUST_LIB_PATH"
    exit 1
fi

echo "Rust library built successfully: $RUST_LIB_PATH"

# 构建 Zig 中间件测试（不需要链接 Rust 库）
echo "Building Zig middleware test..."
zig build-exe zig-test/middleware_test.zig -O ReleaseFast

echo "Running Zig middleware test..."
./middleware_test

# 构建集成的 Web 服务器测试
echo "Building integrated web server with middleware..."
zig build-exe zig-test/web_test.zig -lc -L./target/release -lhush_demo

echo "Build completed successfully!"
echo ""
echo "Available executables:"
echo "  ./middleware_test - Zig middleware system test"
echo "  ./web_test - Web server with basic functionality"
echo ""
echo "To test the web server:"
echo "  ./web_test &"
echo "  curl http://127.0.0.1:8080/"
echo "  curl -X POST -d 'test data' http://127.0.0.1:8080/"
echo ""
echo "To test middleware functionality:"
echo "  ./middleware_test"