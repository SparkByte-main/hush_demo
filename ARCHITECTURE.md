# Hush 框架架构文档

## 概述

本文档描述了 Hush 框架重构后的模块化架构，该架构建立了统一的 FFI 接口规范、错误处理机制和内存管理工具。

## 模块结构

### 核心模块 (src/core/)

#### 1. 错误处理 (error.rs)
- **统一错误类型**: `HushError` 枚举涵盖所有可能的错误情况
- **结果类型**: `HushResult<T>` 作为统一的返回类型
- **C 兼容错误码**: `ErrorCode` 枚举用于 FFI 接口
- **全局错误状态管理**: 线程安全的错误状态存储和访问
- **FFI 错误接口**: 
  - `hush_get_last_error()` - 获取最后错误的字符串表示
  - `hush_get_last_error_code()` - 获取最后错误的错误码
  - `hush_clear_error()` - 清除错误状态

#### 2. FFI 接口规范 (ffi.rs)
- **字符串转换工具**: 
  - `to_c_string()` - Rust 字符串转 C 字符串
  - `from_c_string()` - C 字符串转 Rust 字符串
- **结果处理**: `handle_ffi_result()` - 统一的 FFI 结果处理
- **指针验证**: 空指针检查和验证函数
- **通用 FFI 工具**:
  - `hush_free_string()` - 释放字符串内存
  - `hush_string_clone()` - 复制字符串
  - `hush_string_length()` - 获取字符串长度
  - `hush_string_compare()` - 比较字符串

#### 3. 内存管理 (memory.rs)
- **内存管理器**: `MemoryManager` 负责跨 FFI 边界的内存安全
- **RAII 包装器**: `CStringWrapper` 提供自动内存管理
- **字节数组处理**: `ByteArray` 用于二进制数据传输
- **底层内存操作**:
  - `hush_malloc()` - 分配内存
  - `hush_free()` - 释放内存
  - `hush_realloc()` - 重新分配内存
  - `hush_memcpy()` - 内存复制
  - `hush_memset()` - 内存设置

#### 4. 核心类型 (types.rs)
- **HTTP 相关类型**:
  - `HttpMethod` - HTTP 方法枚举
  - `HttpStatus` - HTTP 状态码枚举
- **上下文类型**:
  - `RequestContext` - 请求上下文，包含所有请求信息
  - `ResponseContext` - 响应上下文，包含所有响应信息
- **路由信息**: `RouteInfo` - 路由元数据
- **C 兼容结构**: `CRequestContext` 和 `CResponseContext`

### Web 模块 (src/web/)

#### 1. 服务器 (server.rs)
- **WebServer**: 重构后的 Web 服务器实现
- **WebServerConfig**: 服务器配置管理
- **异步请求处理**: 基于 actix-web 的高性能请求处理
- **路由分发**: 集成路由管理器进行请求分发

#### 2. 路由管理 (router.rs)
- **Router**: 路由管理器，支持动态路由添加/删除
- **RouteMatcher**: 路由匹配器，支持路径参数和通配符
- **路由信息管理**: 完整的路由元数据跟踪

#### 3. 请求处理 (handler.rs)
- **RequestHandler**: 请求处理器包装器
- **ResponseBuilder**: 响应构建器，提供便捷的响应创建
- **中间件支持**: `Middleware` 系统支持请求/响应拦截
- **内置处理器**: 常用的文本、JSON、HTML 处理器
- **内置中间件**: 日志记录、CORS 等中间件

## FFI 接口

### Web 服务器接口
```c
// 服务器生命周期管理
WebServer* web_server_new();
void web_server_free(WebServer* server);
void web_server_start(WebServer* server, uint16_t port);

// 路由管理
void web_server_add_route(
    WebServer* server,
    const char* method,
    const char* path,
    ZigHandlerFn handler
);
```

### 错误处理接口
```c
// 错误状态管理
const char* hush_get_last_error();
ErrorCode hush_get_last_error_code();
void hush_clear_error();
void hush_free_error_string(char* ptr);
```

### 内存管理接口
```c
// 内存操作
void* hush_malloc(size_t size);
void hush_free(void* ptr, size_t size);
void* hush_realloc(void* ptr, size_t old_size, size_t new_size);
void* hush_memcpy(void* dest, const void* src, size_t size);
void* hush_memset(void* ptr, int value, size_t size);

// 字符串操作
void hush_free_string(char* ptr);
char* hush_string_clone(const char* ptr);
int hush_string_length(const char* ptr);
int hush_string_compare(const char* ptr1, const char* ptr2);
```

## 测试验证

### 功能测试结果
1. **基础 FFI**: ✅ `rust_hello_world()` 正常工作
2. **GET 请求**: ✅ `curl http://127.0.0.1:8080/` 返回正确响应
3. **POST 请求**: ✅ 支持请求体数据传递
4. **路由分发**: ✅ 多个路由 (`/`, `/about`, `/api/users`) 正常工作
5. **错误处理**: ✅ 404 错误正确返回
6. **JSON 响应**: ✅ API 端点返回正确的 JSON 格式

### 架构优势
1. **模块化设计**: 清晰的模块分离，便于维护和扩展
2. **统一错误处理**: 一致的错误处理机制，提高调试效率
3. **内存安全**: RAII 和智能指针确保内存安全
4. **类型安全**: 强类型系统减少运行时错误
5. **FFI 友好**: 简洁的 C 接口，易于其他语言集成
6. **高性能**: 基于 actix-web 的异步处理

## 下一步扩展

基础架构已经建立，可以在此基础上实现：
- 中间件系统
- 数据库集成
- 身份验证
- 配置管理
- 模板引擎
- WebSocket 支持
- 等其他企业级功能

## 构建和运行

```bash
# 构建 Rust 库
cargo build

# 构建 Zig 应用
zig build

# 运行基础测试
./zig-out/bin/zig-rust-demo

# 运行 Web 服务器测试
zig build-exe zig-test/web_test.zig -lhush_demo -L target/debug -lc
./web_test
```