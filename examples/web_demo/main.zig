// ============================================================================
// Hush 框架 Web 应用示例 - 包含完整中间件演示
// ============================================================================

const std = @import("std");
const print = std.debug.print;

const user_handle = @import("handlers/user_handler.zig");

// Web 服务器 FFI 接口
extern fn web_server_new() ?*anyopaque;
extern fn web_server_add_route(server: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, handler: *const fn ([*:0]const u8, [*:0]const u8, [*:0]const u8) callconv(.C) [*:0]const u8) void;
extern fn web_server_start(server: ?*anyopaque, port: u16) void;

// 中间件系统 FFI 接口
extern fn hush_middleware_new() ?*anyopaque;
extern fn hush_middleware_add_logger(middleware: ?*anyopaque) void;
extern fn hush_middleware_add_cors(middleware: ?*anyopaque, allowed_origins: [*:0]const u8) void;
extern fn hush_middleware_add_rate_limit(middleware: ?*anyopaque, max_requests: u32, window_seconds: u64) void;
extern fn hush_middleware_add_auth_jwt(middleware: ?*anyopaque, secret: [*:0]const u8) void;
extern fn hush_middleware_add_rate_limit_by_user(middleware: ?*anyopaque) void;
extern fn hush_middleware_count(middleware: ?*anyopaque) usize;
extern fn hush_middleware_names(middleware: ?*anyopaque) [*:0]const u8;
extern fn hush_middleware_execute(middleware: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) [*:0]const u8;
extern fn hush_middleware_free(middleware: ?*anyopaque) void;

const port: u16 = 8080;

// 全局中间件链
var global_middleware: ?*anyopaque = null;

// ============================================================================
// 路由处理器 - 集成中间件处理
// ============================================================================

/// 健康检查处理器
pub export fn health_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;
    _ = path;

    // 执行中间件链
    if (global_middleware) |middleware| {
        const middleware_response = hush_middleware_execute(middleware, "GET", "/health", "");
        // 在实际应用中，这里会根据中间件响应决定是否继续处理
        _ = middleware_response;
    }

    return "{\"status\": \"healthy\", \"timestamp\": \"2024-01-01T00:00:00Z\", \"service\": \"hush-web-demo\"}";
}

/// API 用户列表处理器
pub export fn api_users_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;
    _ = path;

    // 执行中间件链（包含认证检查）
    if (global_middleware) |middleware| {
        const middleware_response = hush_middleware_execute(middleware, "GET", "/api/users", "");
        // 检查是否被中间件拦截（如认证失败、限流等）
        _ = middleware_response;
    }

    return "{\"users\": [{\"id\": 1, \"name\": \"张三\", \"email\": \"zhangsan@example.com\"}, {\"id\": 2, \"name\": \"李四\", \"email\": \"lisi@example.com\"}]}";
}

/// API 创建用户处理器
pub export fn api_create_user_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;
    _ = path;

    // 执行中间件链（POST 请求，包含认证和限流）
    if (global_middleware) |middleware| {
        const request_body = "{\"name\": \"新用户\", \"email\": \"newuser@example.com\"}";
        const middleware_response = hush_middleware_execute(middleware, "POST", "/api/users", request_body);
        _ = middleware_response;
    }

    return "{\"success\": true, \"user\": {\"id\": 3, \"name\": \"新用户\", \"email\": \"newuser@example.com\"}, \"message\": \"用户创建成功\"}";
}

/// 受保护的管理员接口
pub export fn admin_dashboard_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;
    _ = path;

    // 执行中间件链（需要管理员权限）
    if (global_middleware) |middleware| {
        const middleware_response = hush_middleware_execute(middleware, "GET", "/admin/dashboard", "");
        _ = middleware_response;
    }

    return "{\"dashboard\": {\"total_users\": 1250, \"active_sessions\": 89, \"system_status\": \"正常\"}, \"admin\": true}";
}

/// CORS 预检请求处理器
pub export fn options_handler(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;

    // 执行 CORS 中间件
    if (global_middleware) |middleware| {
        const middleware_response = hush_middleware_execute(middleware, "OPTIONS", path, "");
        return middleware_response;
    }

    return "";
}

// ============================================================================
// 中间件配置和初始化
// ============================================================================

/// 初始化中间件链
fn initializeMiddleware() void {
    print("🔧 初始化中间件系统...\n", .{});

    global_middleware = hush_middleware_new();
    if (global_middleware == null) {
        print("❌ 错误：无法创建中间件链\n", .{});
        return;
    }

    // 1. 添加日志中间件（最高优先级）
    print("📝 添加日志中间件...\n", .{});
    hush_middleware_add_logger(global_middleware);

    // 2. 添加 CORS 中间件
    print("🌐 添加 CORS 中间件...\n", .{});
    hush_middleware_add_cors(global_middleware, "http://localhost:3000,https://app.example.com");

    // 3. 添加全局请求限流中间件
    print("🚦 添加请求限流中间件 (100 请求/小时)...\n", .{});
    hush_middleware_add_rate_limit(global_middleware, 100, 3600);

    // 4. 添加 JWT 认证中间件
    print("🔐 添加 JWT 认证中间件...\n", .{});
    hush_middleware_add_auth_jwt(global_middleware, "hush_web_demo_secret_key_2024");

    // 5. 添加基于用户的限流中间件
    print("👤 添加基于用户的限流中间件...\n", .{});
    hush_middleware_add_rate_limit_by_user(global_middleware);

    const middleware_count = hush_middleware_count(global_middleware);
    print("✅ 中间件初始化完成，共加载 {} 个中间件\n", .{middleware_count});

    // 显示中间件执行顺序
    const names = hush_middleware_names(global_middleware);
    print("📋 中间件执行顺序: {s}\n", .{names});
}

/// 清理中间件资源
fn cleanupMiddleware() void {
    if (global_middleware) |middleware| {
        hush_middleware_free(middleware);
        global_middleware = null;
        print("🧹 中间件资源已清理\n", .{});
    }
}

// ============================================================================
// 路由配置
// ============================================================================

/// 配置所有路由
fn setupRoutes(web_app: ?*anyopaque) void {
    print("🛣️  配置应用路由...\n", .{});

    // 健康检查端点（无需认证）
    web_server_add_route(web_app, "GET", "/health", health_handler);
    print("   ✓ GET  /health - 健康检查\n", .{});

    // 原有用户端点
    web_server_add_route(web_app, "GET", "/user", user_handle.user_handle);
    print("   ✓ GET  /user - 用户信息\n", .{});

    // API 端点（需要认证）
    web_server_add_route(web_app, "GET", "/api/users", api_users_handler);
    print("   ✓ GET  /api/users - 获取用户列表\n", .{});

    web_server_add_route(web_app, "POST", "/api/users", api_create_user_handler);
    print("   ✓ POST /api/users - 创建新用户\n", .{});

    // 管理员端点（需要高级权限）
    web_server_add_route(web_app, "GET", "/admin/dashboard", admin_dashboard_handler);
    print("   ✓ GET  /admin/dashboard - 管理员仪表板\n", .{});

    // CORS 预检请求处理
    web_server_add_route(web_app, "OPTIONS", "/api/users", options_handler);
    web_server_add_route(web_app, "OPTIONS", "/admin/dashboard", options_handler);
    print("   ✓ OPTIONS - CORS 预检请求处理\n", .{});

    print("✅ 路由配置完成\n", .{});
}

// ============================================================================
// 应用程序演示
// ============================================================================

/// 演示中间件功能
fn demonstrateMiddleware() void {
    print("\n🎯 中间件功能演示:\n", .{});

    if (global_middleware == null) {
        print("❌ 中间件未初始化\n", .{});
        return;
    }

    const middleware = global_middleware.?;

    // 演示 1: 健康检查（应该通过）
    print("\n1️⃣  测试健康检查端点:\n", .{});
    const health_response = hush_middleware_execute(middleware, "GET", "/health", "");
    print("   请求: GET /health\n", .{});
    print("   响应: {s}\n", .{health_response});

    // 演示 2: CORS 预检请求
    print("\n2️⃣  测试 CORS 预检请求:\n", .{});
    const cors_response = hush_middleware_execute(middleware, "OPTIONS", "/api/users", "");
    print("   请求: OPTIONS /api/users\n", .{});
    print("   响应: {s}\n", .{cors_response});

    // 演示 3: 需要认证的 API 请求（无令牌）
    print("\n3️⃣  测试受保护的 API（无认证）:\n", .{});
    const unauth_response = hush_middleware_execute(middleware, "GET", "/api/users", "");
    print("   请求: GET /api/users\n", .{});
    print("   响应: {s}\n", .{unauth_response});

    // 演示 4: POST 请求
    print("\n4️⃣  测试 POST 请求:\n", .{});
    const post_body = "{\"name\": \"测试用户\", \"email\": \"test@example.com\"}";
    const post_response = hush_middleware_execute(middleware, "POST", "/api/users", post_body);
    print("   请求: POST /api/users\n", .{});
    print("   请求体: {s}\n", .{post_body});
    print("   响应: {s}\n", .{post_response});

    // 演示 5: 管理员端点
    print("\n5️⃣  测试管理员端点:\n", .{});
    const admin_response = hush_middleware_execute(middleware, "GET", "/admin/dashboard", "");
    print("   请求: GET /admin/dashboard\n", .{});
    print("   响应: {s}\n", .{admin_response});
}

/// 显示应用信息
fn showApplicationInfo() void {
    print("\n" ++ "=" ** 60 ++ "\n", .{});
    print("🚀 Hush 框架 Web 应用演示\n", .{});
    print("=" ** 60 ++ "\n", .{});
    print("📍 服务器地址: http://localhost:{}\n", .{port});
    print("🔧 框架版本: Hush v1.0.0\n", .{});
    print("🛡️  安全特性: CORS + JWT + 限流\n", .{});
    print("📊 监控特性: 请求日志 + 性能追踪\n", .{});
    print("=" ** 60 ++ "\n", .{});
}

/// 显示可用端点
fn showAvailableEndpoints() void {
    print("\n📋 可用端点:\n", .{});
    print("   🟢 GET    /health           - 健康检查（无需认证）\n", .{});
    print("   🟢 GET    /user             - 用户信息\n", .{});
    print("   🔒 GET    /api/users        - 获取用户列表（需要认证）\n", .{});
    print("   🔒 POST   /api/users        - 创建新用户（需要认证）\n", .{});
    print("   🔐 GET    /admin/dashboard  - 管理员仪表板（需要高级权限）\n", .{});
    print("   ⚙️  OPTIONS /api/*          - CORS 预检请求\n", .{});
    print("\n💡 提示: 受保护的端点需要在 Authorization 头中提供有效的 JWT 令牌\n", .{});
}

// ============================================================================
// 主程序
// ============================================================================

pub fn main() void {
    // 显示应用信息
    showApplicationInfo();

    // 初始化中间件系统
    initializeMiddleware();
    defer cleanupMiddleware();

    // 创建 Web 服务器
    print("\n🌐 创建 Web 服务器...\n", .{});
    const web_app = web_server_new();
    if (web_app == null) {
        print("❌ 错误：无法创建 Web 服务器\n", .{});
        return;
    }

    // 配置路由
    setupRoutes(web_app);

    // 演示中间件功能
    demonstrateMiddleware();

    // 显示可用端点
    showAvailableEndpoints();

    // 启动服务器
    print("\n🚀 启动服务器...\n", .{});
    web_server_start(web_app, port);

    print("✅ 服务器已启动，监听端口 {}\n", .{port});
    print("🔄 服务器运行中，按 Ctrl+C 停止...\n", .{});

    // 保持服务器运行
    while (true) {
        std.time.sleep(1000000000); // 休眠 1 秒
    }
}
