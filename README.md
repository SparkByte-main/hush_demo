# Hush Demo | Hush 演示项目

**Hush** is a lightweight, high‑performance web framework for the Zig programming language, implemented in Rust. Currently in demo stage, hush aims to bring Rust's safety and speed to Zig web development.

**Hush** 是一个为 Zig 编程语言设计的轻量级、高性能 Web 框架，使用 Rust 实现。目前处于演示阶段，hush 旨在将 Rust 的安全性和速度带到 Zig Web 开发中。

## Features | 特性

- **Zero‑cost abstractions** | **零成本抽象**: Leveraging Rust's ownership model to eliminate runtime overhead | 利用 Rust 的所有权模型消除运行时开销
- **Modular middleware** | **模块化中间件**: Compose request handlers, routers, and filters with ease | 轻松组合请求处理器、路由器和过滤器
- **Async-first design** | **异步优先设计**: Built on Rust's async ecosystem for maximum throughput | 基于 Rust 异步生态系统构建，实现最大吞吐量
- **Simple API** | **简单 API**: Get up and running with minimal boilerplate | 以最少的样板代码快速启动和运行

## Current Status | 当前状态

⚠️ **Note**: hush is experimental. API and internals are subject to rapid change.

⚠️ **注意**: hush 是实验性的。API 和内部实现可能会快速变化。

This repository contains a **demo project** showcasing Rust-Zig interoperability through FFI (Foreign Function Interface).

此仓库包含一个**演示项目**，展示通过 FFI（外部函数接口）实现 Rust-Zig 互操作性。

## Project Structure | 项目结构

```
hush_demo/
├── src/
│   ├── lib.rs          # Rust library with C-compatible functions | 包含 C 兼容函数的 Rust 库
│   └── main.rs         # Rust main (unused in demo) | Rust 主程序（演示中未使用）
├── zig-test/
│   └── main.zig        # Zig program calling Rust functions | 调用 Rust 函数的 Zig 程序
├── build.zig           # Zig build configuration | Zig 构建配置
├── build.sh            # Automated build script | 自动化构建脚本
├── Cargo.toml          # Rust project configuration | Rust 项目配置
└── README.md           # This file | 本文件
```

## Quick Start | 快速开始

### Prerequisites | 前置要求

- **Rust** (latest stable) | **Rust**（最新稳定版）
- **Zig** (0.11.0 or later) | **Zig**（0.11.0 或更高版本）
- **Cargo** (comes with Rust) | **Cargo**（随 Rust 安装）

### Installation | 安装

1. **Clone the repository | 克隆仓库**
   ```bash
   git clone <repository-url>
   cd hush_demo
   ```

2. **Make build script executable | 使构建脚本可执行**
   ```bash
   chmod +x build.sh
   ```

### Building and Running | 构建和运行

#### Method 1: Automated Build | 方法1：自动化构建

```bash
./build.sh
```

This script will:
- Build the Rust library | 构建 Rust 库
- Build the Zig executable | 构建 Zig 可执行文件
- Show you how to run the program | 显示如何运行程序

#### Method 2: Manual Build | 方法2：手动构建

```bash
# Step 1: Build Rust library | 步骤1：构建 Rust 库
cargo build --lib

# Step 2: Build and run Zig program | 步骤2：构建并运行 Zig 程序
zig build run
```

#### Method 3: Step by Step | 方法3：分步执行

```bash
# Build Rust library | 构建 Rust 库
cargo build --lib

# Build Zig executable | 构建 Zig 可执行文件
zig build

# Run the program | 运行程序
./zig-out/bin/zig-rust-demo
```

### Expected Output | 预期输出

```
Rust says: Hello, World!
```

## How It Works | 工作原理

This demo showcases **Foreign Function Interface (FFI)** between Rust and Zig:

此演示展示了 Rust 和 Zig 之间的**外部函数接口（FFI）**：

1. **Rust Side | Rust 端**:
   - Exports a C-compatible function using `#[unsafe(no_mangle)]` | 使用 `#[unsafe(no_mangle)]` 导出 C 兼容函数
   - Returns a C-style string pointer | 返回 C 风格字符串指针
   - Compiles to a dynamic library (`.so`, `.dylib`, or `.dll`) | 编译为动态库（`.so`、`.dylib` 或 `.dll`）

2. **Zig Side | Zig 端**:
   - Declares the external function with `extern` | 使用 `extern` 声明外部函数
   - Links to the Rust-generated library | 链接到 Rust 生成的库
   - Calls the function and prints the result | 调用函数并打印结果

3. **Build System | 构建系统**:
   - `build.zig` configures library paths and linking | `build.zig` 配置库路径和链接
   - Handles cross-platform library naming | 处理跨平台库命名
   - Manages the build dependencies | 管理构建依赖

## Adding New Functions | 添加新函数

To add more Rust functions callable from Zig:

要添加更多可从 Zig 调用的 Rust 函数：

1. **Add to Rust (`src/lib.rs`) | 添加到 Rust (`src/lib.rs`)**:
   ```rust
   #[unsafe(no_mangle)]
   pub extern "C" fn rust_add_numbers(a: i32, b: i32) -> i32 {
       a + b
   }
   ```

2. **Declare in Zig (`zig-test/main.zig`) | 在 Zig 中声明 (`zig-test/main.zig`)**:
   ```zig
   extern fn rust_add_numbers(a: i32, b: i32) i32;
   ```

3. **Rebuild | 重新构建**:
   ```bash
   ./build.sh
   ```

No changes to `build.zig` are needed! | 无需修改 `build.zig`！

## Technical Details | 技术细节

- **FFI Safety | FFI 安全性**: Uses C ABI for cross-language compatibility | 使用 C ABI 实现跨语言兼容性
- **Memory Management | 内存管理**: Current demo has potential memory leaks (see code comments) | 当前演示存在潜在内存泄漏（见代码注释）
- **Build System | 构建系统**: Zig's build system handles library linking automatically | Zig 构建系统自动处理库链接
- **Cross-Platform | 跨平台**: Supports Windows, macOS, and Linux | 支持 Windows、macOS 和 Linux

## Contributing | 贡献

This is a demo project for the experimental hush framework. Contributions, suggestions, and feedback are welcome!

这是实验性 hush 框架的演示项目。欢迎贡献、建议和反馈！

## License | 许可证

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

## Roadmap | 路线图

- [ ] Complete web framework implementation | 完整的 Web 框架实现
- [ ] HTTP request/response handling | HTTP 请求/响应处理
- [ ] Routing system | 路由系统
- [ ] Middleware support | 中间件支持
- [ ] Async/await integration | 异步/等待集成
- [ ] Performance benchmarks | 性能基准测试
- [ ] Documentation and examples | 文档和示例

---

**Happy coding! | 编程愉快！** 🦀⚡