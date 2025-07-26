# Hush 框架开发指南

## 🚀 快速开始

### 环境要求

- **Rust**: 1.70+ (推荐使用 rustup 安装)
- **Zig**: 0.11+ (从官网下载或使用包管理器)
- **系统**: macOS, Linux, Windows

### 安装依赖

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Zig (macOS)
brew install zig

# 或者从官网下载: https://ziglang.org/download/
```

### 构建项目

```bash
# 克隆项目
git clone <repository-url>
cd hush_framework

# 构建 Rust 核心库
./scripts/build.sh

# 或者手动构建
cargo build --release
```

## 🏗️ 项目架构

### 核心组件

1. **Rust 核心** (`src/`):
   - 高性能的核心功能实现
   - FFI 接口提供给其他语言
   - 完整的中间件系统

2. **Zig 接口** (`examples/zig/`):
   - 简洁的 Zig 语言绑定
   - 类型安全的 API 封装
   - 丰富的使用示例

3. **Web 演示** (`examples/web_demo/`):
   - 完整的 Web 应用示例
   - 展示所有功能特性
   - 实际使用场景演示

### 模块说明

```
src/
├── core/           # 核心功能模块
│   ├── error.rs    # 统一错误处理
│   ├── ffi.rs      # FFI 接口定义
│   ├── memory.rs   # 内存管理
│   └── types.rs    # 核心类型定义
├── middleware/     # 中间件系统
│   ├── builtin.rs  # 内置中间件实现
│   ├── core.rs     # 中间件核心逻辑
│   └── ffi.rs      # 中间件 FFI 接口
└── web/           # Web 服务器模块
    ├── handler.rs  # 请求处理器
    ├── router.rs   # 路由系统
    └── server.rs   # HTTP 服务器
```

## 🔧 开发工作流

### 1. 添加新功能

```bash
# 1. 创建功能分支
git checkout -b feature/new-feature

# 2. 在 Rust 中实现核心功能
# 编辑 src/ 下的相关文件

# 3. 添加 FFI 接口 (如果需要)
# 编辑相关的 ffi.rs 文件

# 4. 创建 Zig 示例
# 在 examples/zig/ 中添加示例

# 5. 编写测试
cargo test

# 6. 更新文档
# 编辑 docs/ 下的相关文档
```

### 2. 测试流程

```bash
# 单元测试
cargo test

# 集成测试
./scripts/test_api.sh

# 性能测试
zig run examples/zig/performance_test.zig -lc -L./target/release -lhush_demo

# 手动测试
zig run examples/web_demo/main.zig -lc -L./target/debug -lhush_demo
```

### 3. 代码规范

#### Rust 代码规范

```rust
// 使用标准的 Rust 命名规范
pub struct MyStruct {
    field_name: String,
}

// 添加详细的文档注释
/// 这是一个示例函数
/// 
/// # 参数
/// - `param`: 参数说明
/// 
/// # 返回值
/// 返回值说明
/// 
/// # 示例
/// ```rust
/// let result = my_function("example");
/// ```
pub fn my_function(param: &str) -> String {
    // 实现逻辑
}

// 错误处理使用 Result 类型
pub fn fallible_function() -> HushResult<String> {
    // 可能失败的操作
}
```

#### Zig 代码规范

```zig
// 使用 snake_case 命名
const my_constant = 42;

// 函数命名使用 camelCase
pub fn myFunction(param: []const u8) void {
    // 实现逻辑
}

// 添加注释说明
/// 这是一个示例函数
/// 参数: param - 输入参数
/// 返回: 无返回值
pub export fn example_handler(req: [*:0]const u8) callconv(.C) [*:0]const u8 {
    // FFI 函数实现
}
```

## 🧪 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // 测试基本功能
        let result = my_function("test");
        assert_eq!(result, "expected");
    }
    
    #[test]
    fn test_error_handling() {
        // 测试错误处理
        let result = fallible_function();
        assert!(result.is_err());
    }
}
```

### 集成测试

```bash
# 运行完整的 API 测试套件
./scripts/test_api.sh

# 测试特定功能
./scripts/test_api.sh -t health

# 性能测试
./scripts/test_api.sh -t performance
```

### 手动测试

```bash
# 启动测试服务器
zig run examples/web_demo/main.zig -lc -L./target/debug -lhush_demo

# 在另一个终端测试 API
curl http://localhost:8080/health
curl http://localhost:8080/api/users
```

## 📚 文档编写

### API 文档

- 所有公共 API 必须有详细的文档注释
- 包含参数说明、返回值、示例代码
- 使用 `cargo doc` 生成文档

### 使用指南

- 在 `docs/` 目录下维护使用指南
- 包含完整的示例代码
- 定期更新以反映最新功能

### 示例代码

- 每个新功能都应该有对应的示例
- 示例应该完整且可运行
- 包含错误处理和最佳实践

## 🔍 调试技巧

### Rust 调试

```bash
# 使用 debug 构建
cargo build

# 启用详细日志
RUST_LOG=debug cargo run

# 使用 GDB/LLDB 调试
rust-gdb target/debug/hush_demo
```

### Zig 调试

```bash
# 编译时启用调试信息
zig run -O Debug examples/web_demo/main.zig -lc -L./target/debug -lhush_demo

# 使用 GDB 调试
gdb ./zig-out/bin/web_demo
```

### FFI 调试

```rust
// 在 FFI 函数中添加日志
#[no_mangle]
pub extern "C" fn my_ffi_function(param: *const c_char) -> *const c_char {
    eprintln!("FFI function called with: {:?}", param);
    // 函数实现
}
```

## 🚀 性能优化

### Rust 优化

```bash
# 使用 release 构建
cargo build --release

# 启用 LTO (Link Time Optimization)
# 在 Cargo.toml 中配置:
[profile.release]
lto = true
codegen-units = 1
```

### 内存管理

```rust
// 使用 Arc 和 Mutex 进行线程安全的共享
use std::sync::{Arc, Mutex};

// 避免不必要的克隆
fn efficient_function(data: &str) -> &str {
    // 直接返回引用而不是克隆
}
```

### FFI 优化

```rust
// 减少字符串转换
// 使用 CString 缓存常用字符串
lazy_static! {
    static ref CACHED_STRINGS: HashMap<&'static str, CString> = {
        // 初始化缓存
    };
}
```

## 🔧 工具推荐

### 开发工具

- **IDE**: VS Code + Rust Analyzer + Zig 扩展
- **调试**: GDB/LLDB, rust-gdb
- **性能分析**: perf, valgrind, cargo-flamegraph
- **代码格式化**: rustfmt, zig fmt

### 有用的 Cargo 命令

```bash
# 检查代码
cargo check

# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 生成文档
cargo doc --open

# 运行基准测试
cargo bench
```

## 📋 发布流程

### 版本管理

1. 更新 `Cargo.toml` 中的版本号
2. 更新 `CHANGELOG.md`
3. 创建 git tag
4. 发布到 crates.io (如果适用)

### 发布检查清单

- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] 示例代码可运行
- [ ] 性能测试通过
- [ ] 版本号已更新
- [ ] CHANGELOG 已更新

## 🤝 贡献指南

### 提交代码

1. Fork 项目
2. 创建功能分支
3. 编写代码和测试
4. 提交 Pull Request
5. 代码审查
6. 合并到主分支

### 代码审查标准

- 代码风格符合项目规范
- 包含充分的测试
- 文档完整且准确
- 性能影响可接受
- 向后兼容性考虑

## 🆘 常见问题

### 构建问题

**Q: 编译时找不到 Rust 库**
```bash
# 确保库路径正确
export LD_LIBRARY_PATH=./target/debug:$LD_LIBRARY_PATH
```

**Q: Zig 编译错误**
```bash
# 检查 Zig 版本
zig version

# 清理缓存
rm -rf .zig-cache/
```

### 运行时问题

**Q: FFI 调用崩溃**
- 检查指针是否为空
- 确保字符串以 null 结尾
- 验证内存管理正确性

**Q: 性能问题**
- 使用 release 构建
- 检查是否有内存泄漏
- 分析热点函数

## 📞 获取帮助

- **文档**: 查看 `docs/` 目录下的文档
- **示例**: 参考 `examples/` 目录下的示例代码
- **测试**: 运行 `./scripts/test_api.sh` 验证环境
- **Issues**: 在 GitHub 上提交问题

---

Happy coding! 🎉