# 🚀 Hush 框架

一个高性能的 Web 框架，使用 Rust 构建核心，提供 Zig 语言接口。

## ✨ 特性

- **🚀 高性能**: Rust 核心确保最大速度
- **🛡️ 内存安全**: 利用 Rust 的内存安全保证
- **🔧 简洁 API**: 清晰的 Zig 接口，易于使用
- **🌐 跨平台**: 支持 macOS、Linux 和 Windows
- **🔌 中间件系统**: 完整的可插拔中间件架构
- **🔐 安全特性**: 内置 CORS、JWT 认证、请求限流
- **📊 监控支持**: 详细的请求日志和性能追踪

## 🏁 快速开始

### 环境要求

- **Rust**: 1.70+ (推荐使用 rustup)
- **Zig**: 0.11+ (从官网下载或使用包管理器)

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

### 运行示例

```bash
# Web 服务器演示
zig run examples/web_demo/main.zig -lc -L./target/debug -lhush_demo

# 中间件演示
zig run examples/zig/middleware_demo.zig -lc -L./target/debug -lhush_demo

# 基础使用示例
zig run examples/zig/basic_usage.zig -lc -L./target/debug -lhush_demo
```

### API 测试

```bash
# 运行完整的 API 测试套件
./scripts/test_api.sh

# 测试特定功能
./scripts/test_api.sh -t health
```

## 🏗️ 项目结构

```
hush_framework/
├── 📁 src/                    # Rust 核心源码
│   ├── core/                  # 核心功能模块
│   ├── middleware/            # 中间件系统
│   └── web/                   # Web 服务器模块
├── 📁 examples/               # 示例代码
│   ├── zig/                   # Zig 语言示例
│   ├── web_demo/              # Web 演示应用
│   └── clients/               # 客户端示例
├── 📁 docs/                   # 文档目录
├── 📁 scripts/                # 构建和测试脚本
└── 📁 tests/                  # 测试文件
```

详细的项目结构说明请查看 [项目结构文档](docs/PROJECT_STRUCTURE.md)。

## 🎯 API 概览

### Web 服务器

```zig
// 创建服务器
const server = web_server_new();

// 添加路由
web_server_add_route(server, "GET", "/hello", hello_handler);

// 启动服务器
web_server_start(server, 8080);
```

### 中间件系统

```zig
// 创建中间件链
const middleware = hush_middleware_new();

// 添加内置中间件
hush_middleware_add_logger(middleware);
hush_middleware_add_cors(middleware, "https://example.com");
hush_middleware_add_rate_limit(middleware, 100, 3600);
hush_middleware_add_auth_jwt(middleware, "secret_key");
```

### 请求处理器

```zig
pub export fn hello_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    return "{\"message\": \"Hello, World!\"}";
}
```

## 🛡️ 安全特性

- **CORS 支持**: 完整的跨域资源共享配置
- **JWT 认证**: 内置 JSON Web Token 认证中间件
- **请求限流**: 防止 API 滥用的限流机制
- **输入验证**: 安全的输入处理和验证

## 📊 性能特性

- **零拷贝**: 高效的内存管理
- **异步处理**: 支持高并发请求
- **中间件缓存**: 智能的中间件执行优化
- **连接池**: 数据库连接池支持

## 📚 文档

- [📖 API 使用指南](docs/API_USAGE_GUIDE.md) - 详细的 API 使用说明
- [🏗️ 架构文档](docs/ARCHITECTURE.md) - 系统架构设计
- [🔧 开发指南](docs/DEVELOPMENT_GUIDE.md) - 开发环境配置和贡献指南
- [📁 项目结构](docs/PROJECT_STRUCTURE.md) - 项目组织结构说明

## 🎮 在线演示

打开 `examples/clients/demo.html` 在浏览器中体验完整的 API 功能演示。

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
./scripts/test_api.sh

# 性能测试
zig run examples/zig/performance_test.zig -lc -L./target/release -lhush_demo
```

## 🤝 贡献

我们欢迎所有形式的贡献！

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

详细的贡献指南请查看 [开发指南](docs/DEVELOPMENT_GUIDE.md)。

## 📄 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](LICENSE) 文件。

## 🙏 致谢

感谢所有为这个项目做出贡献的开发者！

---

**快乐编码！** 🎉

如有问题或建议，欢迎提交 Issue 或 Pull Request。