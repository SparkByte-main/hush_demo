# Hush 框架 API 使用指南

## 📋 目录
- [项目概述](#项目概述)
- [快速开始](#快速开始)
- [API 端点详解](#api-端点详解)
- [中间件系统](#中间件系统)
- [认证与授权](#认证与授权)
- [错误处理](#错误处理)
- [性能与限流](#性能与限流)
- [开发示例](#开发示例)

---

## 🚀 项目概述

Hush 是一个高性能的 Web 框架，使用 Rust 构建核心，通过 FFI 提供 Zig 语言接口。框架提供了完整的中间件系统，包括 CORS、认证、限流、日志等功能。

### 核心特性
- ✅ **高性能**: Rust 核心 + Zig 接口
- ✅ **中间件系统**: 可插拔的请求处理管道
- ✅ **安全性**: JWT 认证 + CORS + 请求限流
- ✅ **监控**: 详细的请求日志和性能追踪
- ✅ **易用性**: 简洁的 API 设计

---

## 🏁 快速开始

### 1. 编译项目
```bash
# 编译 Rust 核心库
cargo build

# 运行 Zig Web 演示
zig run zig_web_demo/main.zig -lc -L./target/debug -lhush_demo
```

### 2. 启动服务器
```bash
# 服务器将在 http://localhost:8080 启动
# 控制台会显示详细的启动信息和中间件配置
```

### 3. 测试基本功能
```bash
# 健康检查
curl http://localhost:8080/health

# 用户信息
curl http://localhost:8080/user
```

---

## 🔌 API 端点详解

### 1. 健康检查端点

**端点**: `GET /health`  
**认证**: 无需认证  
**描述**: 检查服务器运行状态

#### 请求示例
```bash
curl -X GET http://localhost:8080/health
```

#### 响应示例
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "service": "hush-web-demo"
}
```

#### 中间件处理流程
1. 📝 **日志中间件**: 记录请求开始
2. 🌐 **CORS 中间件**: 处理跨域请求
3. 🚦 **限流中间件**: 检查请求频率
4. ✅ **跳过认证**: 健康检查无需认证
5. 📝 **日志中间件**: 记录响应完成

---

### 2. 用户信息端点

**端点**: `GET /user`  
**认证**: 无需认证  
**描述**: 获取基本用户信息

#### 请求示例
```bash
curl -X GET http://localhost:8080/user
```

#### 响应示例
```json
"Hello, World!"
```

---

### 3. 用户列表 API

**端点**: `GET /api/users`  
**认证**: 需要 JWT 令牌  
**描述**: 获取系统中的用户列表

#### 请求示例
```bash
# 无认证请求（将被拒绝）
curl -X GET http://localhost:8080/api/users

# 带认证的请求
curl -X GET http://localhost:8080/api/users \
  -H "Authorization: Bearer your_jwt_token_here"
```

#### 成功响应示例
```json
{
  "users": [
    {
      "id": 1,
      "name": "张三",
      "email": "zhangsan@example.com"
    },
    {
      "id": 2,
      "name": "李四",
      "email": "lisi@example.com"
    }
  ]
}
```

#### 认证失败响应
```json
{
  "error": "Missing authorization token"
}
```

#### 中间件处理流程
1. 📝 **日志中间件**: 记录请求详情
2. 🌐 **CORS 中间件**: 验证请求来源
3. 🚦 **全局限流**: 检查 IP 请求频率
4. 🔐 **JWT 认证**: 验证 Authorization 头
5. 👤 **用户限流**: 检查用户请求频率
6. ✅ **业务逻辑**: 返回用户列表

---

### 4. 创建用户 API

**端点**: `POST /api/users`  
**认证**: 需要 JWT 令牌  
**描述**: 创建新用户

#### 请求示例
```bash
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your_jwt_token_here" \
  -d '{
    "name": "新用户",
    "email": "newuser@example.com"
  }'
```

#### 成功响应示例
```json
{
  "success": true,
  "user": {
    "id": 3,
    "name": "新用户",
    "email": "newuser@example.com"
  },
  "message": "用户创建成功"
}
```

#### 中间件处理流程
1. 📝 **日志中间件**: 记录 POST 请求和请求体
2. 🌐 **CORS 中间件**: 处理跨域 POST 请求
3. 🚦 **限流检查**: POST 请求通常有更严格的限制
4. 🔐 **JWT 认证**: 验证创建权限
5. 👤 **用户限流**: 防止用户频繁创建
6. ✅ **业务逻辑**: 创建用户并返回结果

---

### 5. 管理员仪表板

**端点**: `GET /admin/dashboard`  
**认证**: 需要管理员级别的 JWT 令牌  
**描述**: 获取管理员仪表板数据

#### 请求示例
```bash
curl -X GET http://localhost:8080/admin/dashboard \
  -H "Authorization: Bearer admin_jwt_token_here"
```

#### 成功响应示例
```json
{
  "dashboard": {
    "total_users": 1250,
    "active_sessions": 89,
    "system_status": "正常"
  },
  "admin": true
}
```

#### 权限不足响应
```json
{
  "error": "Invalid authorization token"
}
```

---

### 6. CORS 预检请求

**端点**: `OPTIONS /api/*`  
**认证**: 无需认证  
**描述**: 处理浏览器的 CORS 预检请求

#### 请求示例
```bash
curl -X OPTIONS http://localhost:8080/api/users \
  -H "Origin: https://app.example.com" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type, Authorization"
```

#### 成功响应头
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With
Access-Control-Max-Age: 86400
Access-Control-Allow-Credentials: true
```

---

## 🛡️ 中间件系统

### 中间件执行顺序
```
请求 → 日志 → CORS → 全局限流 → JWT认证 → 用户限流 → 业务逻辑
```

### 1. 日志中间件
- **优先级**: 5 (最高)
- **功能**: 记录所有请求和响应
- **配置**: 自动启用

```zig
// 添加日志中间件
hush_middleware_add_logger(middleware);
```

#### 日志输出示例
```
[SystemTime { tv_sec: 1753536685, tv_nsec: 67851000 }] GET /api/users - Request started
[SystemTime { tv_sec: 1753536685, tv_nsec: 68025000 }] GET /api/users - 200 OK (0.17ms)
```

### 2. CORS 中间件
- **优先级**: 10
- **功能**: 处理跨域请求
- **配置**: 可自定义允许的源

```zig
// 配置 CORS 中间件
hush_middleware_add_cors(middleware, "http://localhost:3000,https://app.example.com");
```

#### CORS 配置说明
- **允许的源**: `http://localhost:3000,https://app.example.com`
- **允许的方法**: `GET, POST, PUT, DELETE, OPTIONS`
- **允许的头部**: `Content-Type, Authorization, X-Requested-With`
- **缓存时间**: 24小时

### 3. 请求限流中间件
- **优先级**: 15
- **功能**: 防止 API 滥用
- **配置**: 100 请求/小时

```zig
// 添加全局限流
hush_middleware_add_rate_limit(middleware, 100, 3600);
```

#### 限流响应示例
```json
{
  "error": "Rate limit exceeded",
  "max_requests": 100,
  "window_seconds": 3600
}
```

### 4. JWT 认证中间件
- **优先级**: 20
- **功能**: 验证用户身份
- **跳过路径**: `/health`, `/login`

```zig
// 添加 JWT 认证
hush_middleware_add_auth_jwt(middleware, "hush_web_demo_secret_key_2024");
```

#### JWT 令牌格式
```
Authorization: Bearer <jwt_token>
```

### 5. 用户限流中间件
- **优先级**: 25
- **功能**: 基于用户的精细化限流
- **配置**: 自动从认证信息获取用户ID

```zig
// 添加用户限流
hush_middleware_add_rate_limit_by_user(middleware);
```

---

## 🔐 认证与授权

### JWT 令牌结构
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "user_id": "12345",
    "username": "zhangsan",
    "role": "user",
    "exp": 1640995200
  },
  "signature": "..."
}
```

### 认证流程
1. **客户端**: 在 `Authorization` 头中发送 JWT 令牌
2. **中间件**: 提取并验证令牌
3. **验证**: 检查签名、过期时间等
4. **上下文**: 将用户信息添加到请求上下文
5. **继续**: 传递给下一个中间件或业务逻辑

### 权限级别
- **公开端点**: `/health`, `/user` - 无需认证
- **用户端点**: `/api/users` - 需要有效 JWT
- **管理员端点**: `/admin/*` - 需要管理员权限

---

## ⚠️ 错误处理

### 常见错误响应

#### 1. 认证错误 (401)
```json
{
  "error": "Missing authorization token"
}
```

#### 2. 权限不足 (401)
```json
{
  "error": "Invalid authorization token"
}
```

#### 3. CORS 错误 (403)
```json
{
  "error": "CORS: Origin not allowed"
}
```

#### 4. 限流错误 (429)
```json
{
  "error": "Rate limit exceeded",
  "max_requests": 100,
  "window_seconds": 3600
}
```

### 错误处理最佳实践
1. **检查响应状态码**: 根据状态码判断错误类型
2. **解析错误消息**: 获取详细的错误信息
3. **重试机制**: 对于限流错误，实现指数退避重试
4. **用户友好**: 将技术错误转换为用户可理解的消息

---

## 📊 性能与限流

### 限流策略

#### 1. 全局限流
- **限制**: 100 请求/小时/IP
- **目的**: 防止单个 IP 滥用
- **响应**: 429 Too Many Requests

#### 2. 用户限流
- **限制**: 基于用户ID的个性化限制
- **目的**: 公平的资源分配
- **配置**: 动态调整

### 性能监控
```
处理 100 个请求耗时: 15 ms
平均每个请求: 0.15 ms
```

### 性能优化建议
1. **缓存**: 对频繁访问的数据使用缓存
2. **连接池**: 复用数据库连接
3. **异步处理**: 使用异步 I/O
4. **负载均衡**: 多实例部署

---

## 💻 开发示例

### 1. 创建新的路由处理器

```zig
/// 自定义 API 处理器
pub export fn custom_api_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;
    _ = path;
    
    // 执行中间件链
    if (global_middleware) |middleware| {
        const middleware_response = hush_middleware_execute(middleware, "GET", "/api/custom", "");
        // 检查中间件响应，决定是否继续处理
        _ = middleware_response;
    }
    
    return "{\"message\": \"自定义 API 响应\", \"data\": {}}";
}
```

### 2. 注册新路由

```zig
// 在 setupRoutes 函数中添加
web_server_add_route(web_app, "GET", "/api/custom", custom_api_handler);
```

### 3. 自定义中间件配置

```zig
// 创建专用的中间件链
const api_middleware = hush_middleware_new();
hush_middleware_add_logger(api_middleware);
hush_middleware_add_cors(api_middleware, "https://trusted-domain.com");
hush_middleware_add_rate_limit(api_middleware, 50, 1800); // 更严格的限制
```

### 4. 错误处理示例

```zig
pub export fn error_handling_example(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    if (global_middleware) |middleware| {
        const middleware_response = hush_middleware_execute(middleware, "GET", path, "");
        
        // 检查中间件是否返回错误
        if (std.mem.startsWith(u8, std.mem.span(middleware_response), "{\"error\"")) {
            // 中间件返回了错误，直接返回
            return middleware_response;
        }
    }
    
    // 继续正常处理
    return "{\"success\": true}";
}
```

---

## 🧪 测试示例

### 1. 使用 curl 测试

```bash
# 测试健康检查
curl -v http://localhost:8080/health

# 测试 CORS 预检
curl -X OPTIONS http://localhost:8080/api/users \
  -H "Origin: https://app.example.com" \
  -H "Access-Control-Request-Method: GET"

# 测试认证失败
curl -X GET http://localhost:8080/api/users

# 测试限流
for i in {1..105}; do
  curl http://localhost:8080/health
done
```

### 2. 使用 JavaScript 测试

```javascript
// 测试 CORS 请求
fetch('http://localhost:8080/api/users', {
  method: 'GET',
  headers: {
    'Authorization': 'Bearer your_jwt_token',
    'Content-Type': 'application/json'
  }
})
.then(response => response.json())
.then(data => console.log(data))
.catch(error => console.error('Error:', error));
```

### 3. 性能测试

```bash
# 使用 ab (Apache Bench) 进行压力测试
ab -n 1000 -c 10 http://localhost:8080/health

# 使用 wrk 进行性能测试
wrk -t12 -c400 -d30s http://localhost:8080/health
```

---

## 📚 总结

Hush 框架提供了一个完整的 Web 开发解决方案，具有以下优势：

1. **高性能**: Rust 核心确保了出色的性能
2. **安全性**: 完整的认证、授权和 CORS 支持
3. **可扩展**: 灵活的中间件系统
4. **易用性**: 简洁的 Zig API 接口
5. **监控**: 详细的日志和性能追踪

通过本指南，您可以快速上手 Hush 框架，构建安全、高性能的 Web 应用程序。

---

## 🔗 相关资源

- [Hush 框架源码](./src/)
- [Zig 示例代码](./zig_web_demo/)
- [中间件测试](./zig-test/)
- [构建脚本](./build.sh)

如有问题或建议，欢迎提交 Issue 或 Pull Request！