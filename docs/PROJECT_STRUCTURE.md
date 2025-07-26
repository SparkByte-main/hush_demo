# Hush 框架项目结构

## 📁 项目目录结构

```
hush_framework/
├── 📁 src/                          # Rust 核心源码
│   ├── 📁 core/                     # 核心功能模块
│   │   ├── error.rs                 # 错误处理
│   │   ├── ffi.rs                   # FFI 接口
│   │   ├── memory.rs                # 内存管理
│   │   ├── types.rs                 # 核心类型定义
│   │   └── mod.rs                   # 模块导出
│   ├── 📁 middleware/               # 中间件系统
│   │   ├── builtin.rs               # 内置中间件
│   │   ├── core.rs                  # 中间件核心
│   │   ├── ffi.rs                   # 中间件 FFI
│   │   ├── integration_tests.rs     # 集成测试
│   │   └── mod.rs                   # 模块导出
│   ├── 📁 web/                      # Web 服务器模块
│   │   ├── handler.rs               # 请求处理器
│   │   ├── router.rs                # 路由系统
│   │   ├── server.rs                # Web 服务器
│   │   └── mod.rs                   # 模块导出
│   ├── lib.rs                       # 库入口
│   └── main.rs                      # 主程序入口
├── 📁 examples/                     # 示例代码
│   ├── 📁 zig/                      # Zig 语言示例
│   │   ├── basic_usage.zig          # 基础使用示例
│   │   ├── middleware_demo.zig      # 中间件演示
│   │   ├── web_server.zig           # Web 服务器示例
│   │   └── performance_test.zig     # 性能测试
│   ├── 📁 web_demo/                 # Web 演示应用
│   │   ├── 📁 handlers/             # 请求处理器
│   │   │   └── user_handler.zig     # 用户处理器
│   │   └── main.zig                 # 主程序
│   └── 📁 clients/                  # 客户端示例
│       ├── javascript_client.js     # JavaScript 客户端
│       ├── python_client.py         # Python 客户端
│       └── demo.html                # Web 演示页面
├── 📁 docs/                         # 文档目录
│   ├── API_USAGE_GUIDE.md           # API 使用指南
│   ├── ARCHITECTURE.md              # 架构文档
│   ├── PROJECT_STRUCTURE.md         # 项目结构说明
│   └── DEVELOPMENT_GUIDE.md         # 开发指南
├── 📁 scripts/                      # 构建和测试脚本
│   ├── build.sh                     # 构建脚本
│   ├── test_api.sh                  # API 测试脚本
│   └── clean.sh                     # 清理脚本
├── 📁 tests/                        # 测试文件
│   ├── integration_tests.rs         # 集成测试
│   └── unit_tests.rs                # 单元测试
├── 📁 .kiro/                        # Kiro IDE 配置
│   └── 📁 specs/                    # 规格说明
├── Cargo.toml                       # Rust 项目配置
├── build.zig                        # Zig 构建配置
├── README.md                        # 项目说明
├── LICENSE                          # 许可证
└── .gitignore                       # Git 忽略文件
```

## 📋 目录说明

### 🦀 src/ - Rust 核心源码
- **core/**: 框架核心功能，包括错误处理、FFI 接口、内存管理等
- **middleware/**: 完整的中间件系统，包括内置中间件和 FFI 接口
- **web/**: Web 服务器相关功能，包括路由、处理器等

### 🎯 examples/ - 示例代码
- **zig/**: Zig 语言使用示例，展示各种功能
- **web_demo/**: 完整的 Web 应用演示
- **clients/**: 各种语言的客户端示例

### 📚 docs/ - 文档
- 完整的 API 文档和使用指南
- 架构设计文档
- 开发指南和最佳实践

### 🔧 scripts/ - 工具脚本
- 构建、测试、清理等自动化脚本
- 开发环境配置脚本

### 🧪 tests/ - 测试
- 单元测试和集成测试
- 性能测试和基准测试

## 🚀 快速开始

1. **构建项目**:
   ```bash
   ./scripts/build.sh
   ```

2. **运行示例**:
   ```bash
   # Web 服务器演示
   zig run examples/web_demo/main.zig -lc -L./target/debug -lhush_demo
   
   # 中间件演示
   zig run examples/zig/middleware_demo.zig -lc -L./target/debug -lhush_demo
   ```

3. **运行测试**:
   ```bash
   ./scripts/test_api.sh
   ```

4. **查看文档**:
   - [API 使用指南](./API_USAGE_GUIDE.md)
   - [架构文档](./ARCHITECTURE.md)

## 🔄 迁移说明

从旧结构迁移到新结构的主要变化：

1. **zig-test/** → **examples/zig/**
2. **zig_web_demo/** → **examples/web_demo/**
3. **client_examples.*** → **examples/clients/**
4. **API_USAGE_GUIDE.md** → **docs/API_USAGE_GUIDE.md**
5. **build.sh** → **scripts/build.sh**
6. **test_api_examples.sh** → **scripts/test_api.sh**

## 📝 开发规范

- **代码组织**: 按功能模块组织，保持清晰的层次结构
- **命名规范**: 使用描述性的文件和目录名称
- **文档**: 每个模块都应有相应的文档说明
- **测试**: 新功能必须包含相应的测试用例
- **示例**: 重要功能应提供使用示例

## 🎯 未来规划

- [ ] 添加更多语言的客户端示例
- [ ] 完善性能测试套件
- [ ] 添加 Docker 支持
- [ ] 创建 CI/CD 流水线
- [ ] 添加更多中间件组件