#!/bin/bash

# ============================================================================
# Hush 框架清理脚本
# ============================================================================

echo "🧹 清理 Hush 框架项目..."

# 清理 Rust 构建产物
echo "清理 Rust 构建产物..."
cargo clean

# 清理 Zig 缓存
echo "清理 Zig 缓存..."
rm -rf .zig-cache/
rm -rf zig-out/

# 清理编译产物
echo "清理编译产物..."
rm -f *.o
rm -f middleware
rm -f middleware_test
rm -f web_test

# 清理旧的测试文件
echo "清理旧的测试文件..."
rm -rf zig-test/
rm -rf zig_web_demo/

# 清理临时文件
echo "清理临时文件..."
find . -name "*.tmp" -delete
find . -name "*.log" -delete
find . -name ".DS_Store" -delete

echo "✅ 清理完成！"