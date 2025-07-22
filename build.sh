#!/bin/bash

# Rust + Zig 混合项目构建脚本 | Rust + Zig hybrid project build script
# 这个脚本会先构建 Rust 库，然后构建 Zig 可执行文件 | This script builds Rust library first, then builds Zig executable

echo "=== 开始构建 Rust + Zig 混合项目 | Starting Rust + Zig hybrid project build ==="

# 第一步：构建 Rust 动态库 | Step 1: Build Rust dynamic library
echo "步骤 1: 构建 Rust 动态库... | Step 1: Building Rust dynamic library..."
echo "执行命令: cargo build | Executing command: cargo build"

# 使用 Cargo 构建 Rust 项目 | Use Cargo to build Rust project
# --lib 参数确保只构建库，不构建二进制文件 | --lib parameter ensures only library is built, not binary
# 这会在 target/debug/ 目录下生成动态库文件 | This generates dynamic library files in target/debug/ directory
cargo build --lib

# 检查 Rust 构建是否成功 | Check if Rust build succeeded
if [ $? -ne 0 ]; then
    echo "❌ Rust 构建失败！| Rust build failed!"
    exit 1
fi

echo "✅ Rust 库构建成功！| Rust library build successful!"

# 显示生成的库文件 | Display generated library files
echo "生成的库文件 | Generated library files:"
ls -la target/debug/libhush_demo.*

echo ""

# 第二步：构建 Zig 可执行文件 | Step 2: Build Zig executable
echo "步骤 2: 构建 Zig 可执行文件... | Step 2: Building Zig executable..."
echo "执行命令: zig build | Executing command: zig build"

# 使用 Zig 构建系统构建可执行文件 | Use Zig build system to build executable
# 这会链接之前生成的 Rust 动态库 | This links the previously generated Rust dynamic library
zig build

# 检查 Zig 构建是否成功 | Check if Zig build succeeded
if [ $? -ne 0 ]; then
    echo "❌ Zig 构建失败！| Zig build failed!"
    exit 1
fi

echo "✅ Zig 可执行文件构建成功！| Zig executable build successful!"

# 显示生成的可执行文件 | Display generated executable files
echo "生成的可执行文件 | Generated executable files:"
ls -la zig-out/bin/

echo ""
echo "=== 构建完成 | Build completed ==="
echo "运行程序 | Run program: ./zig-out/bin/zig-rust-demo"
echo "或者使用 | Or use: zig build run"