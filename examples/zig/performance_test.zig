// ============================================================================
// Zig 中间件系统使用示例 | Zig Middleware System Usage Example
// ============================================================================

const std = @import("std");
const middleware = @import("middleware.zig");

// ============================================================================
// 外部函数声明 - Web 服务器 FFI 接口 | External Function Declarations - Web Server FFI Interface
// ============================================================================

extern fn web_server_new() ?*anyopaque;
extern fn web_server_add_route(server: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, handler: *const fn ([*:0]const u8, [*:0]const u8, [*:0]const u8) callconv(.C) [*:0]const u8) void;
extern fn web_server_start(server: ?*anyopaque, port: u16) void;
extern fn web_server_free(server: ?*anyopaque) void;
extern fn rust_free_string(ptr: [*:0]u8) void;

// ============================================================================
// 全局中间件链 | Global Middleware Chain
// ============================================================================

var global_middleware_chain: ?middleware.MiddlewareChain = null;
var global_allocator: std.mem.Allocator = undefined;

// ============================================================================
// 中间件增强的路由处理函数 | Middleware-Enhanced Route Handlers
// ============================================================================

/// 带中间件的根路径处理函数
export fn middleware_hello_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    const method_str = std.mem.span(method);
    const path_str = std.mem.span(path);
    const body_str = std.mem.span(body);

    // 执行中间件链
    if (global_middleware_chain) |*chain| {
        const middleware_result = chain.execute(method_str, path_str, body_str) catch {
            return createErrorResponse("Middleware execution failed");
        };
        defer global_allocator.free(middleware_result);

        // 如果中间件返回了响应，直接返回
        if (!std.mem.eql(u8, middleware_result, "OK")) {
            return createResponse(middleware_result);
        }
    }

    // 执行实际的业务逻辑
    const response = if (std.mem.eql(u8, method_str, "POST"))
        std.fmt.allocPrintZ(global_allocator, "{{\"message\":\"Hello from middleware-enhanced Zig!\",\"method\":\"{s}\",\"path\":\"{s}\",\"body\":\"{s}\"}}", .{ method_str, path_str, body_str })
    else
        std.fmt.allocPrintZ(global_allocator, "{{\"message\":\"Hello from middleware-enhanced Zig!\",\"method\":\"{s}\",\"path\":\"{s}\"}}", .{ method_str, path_str });

    return (response catch {
        return createErrorResponse("Memory allocation failed");
    }).ptr;
}

/// 受保护的 API 端点处理函数
export fn protected_api_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    const method_str = std.mem.span(method);
    const path_str = std.mem.span(path);
    const body_str = std.mem.span(body);

    // 执行中间件链
    if (global_middleware_chain) |*chain| {
        const middleware_result = chain.execute(method_str, path_str, body_str) catch {
            return createErrorResponse("Middleware execution failed");
        };
        defer global_allocator.free(middleware_result);

        // 检查是否通过了认证中间件
        if (std.mem.indexOf(u8, middleware_result, "Unauthorized") != null) {
            return createResponse("{\"error\":\"Authentication required\",\"status\":401}");
        }
    }

    // 执行受保护的业务逻辑
    const response = std.fmt.allocPrintZ(global_allocator, "{{\"message\":\"Access granted to protected resource\",\"method\":\"{s}\",\"path\":\"{s}\",\"timestamp\":{}}}", .{ method_str, path_str, std.time.timestamp() }) catch {
        return createErrorResponse("Memory allocation failed");
    };

    return response.ptr;
}

/// 公共健康检查端点（跳过认证）
export fn health_check_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = method;
    _ = path;
    _ = body;

    const response = std.fmt.allocPrintZ(global_allocator, "{{\"status\":\"healthy\",\"timestamp\":{},\"middleware_count\":{}}}", .{ std.time.timestamp(), if (global_middleware_chain) |*chain| chain.count() else 0 }) catch {
        return createErrorResponse("Memory allocation failed");
    };

    return response.ptr;
}

/// CORS 预检请求处理函数
export fn options_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    const method_str = std.mem.span(method);
    const path_str = std.mem.span(path);
    const body_str = std.mem.span(body);

    // 执行中间件链（主要是 CORS 中间件）
    if (global_middleware_chain) |*chain| {
        const middleware_result = chain.execute(method_str, path_str, body_str) catch {
            return createErrorResponse("Middleware execution failed");
        };
        defer global_allocator.free(middleware_result);
    }

    // 返回 CORS 预检响应
    const response = std.fmt.allocPrintZ(global_allocator, "{{\"message\":\"CORS preflight handled\",\"method\":\"{s}\",\"path\":\"{s}\"}}", .{ method_str, path_str }) catch {
        return createErrorResponse("Memory allocation failed");
    };

    return response.ptr;
}

/// 用户管理 API 端点
export fn user_management_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    const method_str = std.mem.span(method);
    const path_str = std.mem.span(path);
    const body_str = std.mem.span(body);

    // 执行中间件链
    if (global_middleware_chain) |*chain| {
        const middleware_result = chain.execute(method_str, path_str, body_str) catch {
            return createErrorResponse("Middleware execution failed");
        };
        defer global_allocator.free(middleware_result);

        // 检查认证和限流
        if (std.mem.indexOf(u8, middleware_result, "Unauthorized") != null) {
            return createResponse("{\"error\":\"Authentication required\",\"status\":401}");
        }
        if (std.mem.indexOf(u8, middleware_result, "Rate limit exceeded") != null) {
            return createResponse("{\"error\":\"Rate limit exceeded\",\"status\":429}");
        }
    }

    // 根据 HTTP 方法执行不同的操作
    const response = switch (method_str[0]) {
        'G' => std.fmt.allocPrintZ(global_allocator, "{{\"action\":\"list_users\",\"method\":\"{s}\",\"path\":\"{s}\"}}", .{ method_str, path_str }),
        'P' => if (std.mem.eql(u8, method_str, "POST"))
            std.fmt.allocPrintZ(global_allocator, "{{\"action\":\"create_user\",\"method\":\"{s}\",\"path\":\"{s}\",\"data\":\"{s}\"}}", .{ method_str, path_str, body_str })
        else
            std.fmt.allocPrintZ(global_allocator, "{{\"action\":\"update_user\",\"method\":\"{s}\",\"path\":\"{s}\",\"data\":\"{s}\"}}", .{ method_str, path_str, body_str }),
        'D' => std.fmt.allocPrintZ(global_allocator, "{{\"action\":\"delete_user\",\"method\":\"{s}\",\"path\":\"{s}\"}}", .{ method_str, path_str }),
        else => std.fmt.allocPrintZ(global_allocator, "{{\"error\":\"Method not allowed\",\"method\":\"{s}\"}}", .{method_str}),
    } catch {
        return createErrorResponse("Memory allocation failed");
    };

    return response.ptr;
}

// ============================================================================
// 辅助函数 | Helper Functions
// ============================================================================

/// 创建错误响应
fn createErrorResponse(message: []const u8) [*:0]const u8 {
    const response = std.fmt.allocPrintZ(global_allocator, "{{\"error\":\"{s}\"}}", .{message}) catch {
        return "Error: Memory allocation failed".ptr;
    };
    return response.ptr;
}

/// 创建响应
fn createResponse(message: []const u8) [*:0]const u8 {
    const response = global_allocator.dupeZ(u8, message) catch {
        return "Error: Memory allocation failed".ptr;
    };
    return response.ptr;
}

/// 初始化中间件链
fn initializeMiddleware(allocator: std.mem.Allocator) !void {
    global_allocator = allocator;
    global_middleware_chain = try middleware.MiddlewareChain.init(allocator);

    if (global_middleware_chain) |*chain| {
        // 添加日志中间件（最高优先级）
        try chain.addLogger();

        // 添加 CORS 中间件
        try chain.addCors("http://localhost:3000,http://localhost:8080,*");

        // 添加 JWT 认证中间件
        try chain.addAuthJwt("my_super_secret_jwt_key_12345");

        std.debug.print("Middleware chain initialized with {} middlewares\n", .{chain.count()});

        // 打印中间件名称
        const names = try chain.getNames();
        defer allocator.free(names);
        std.debug.print("Middleware names: {s}\n", .{names});
    }
}

/// 清理中间件链
fn cleanupMiddleware() void {
    if (global_middleware_chain) |*chain| {
        chain.deinit();
        global_middleware_chain = null;
    }
}

// ============================================================================
// 主应用程序 | Main Application
// ============================================================================

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("Starting Zig web application with middleware support...\n");

    // 初始化中间件系统
    try initializeMiddleware(allocator);
    defer cleanupMiddleware();

    // 创建 web 服务器
    const server = web_server_new();
    if (server == null) {
        std.debug.print("Failed to create web server\n");
        return;
    }
    defer web_server_free(server);

    std.debug.print("Registering routes with middleware support...\n");

    // 注册路由
    // 公共路由（带基础中间件）
    web_server_add_route(server, "GET", "/", middleware_hello_handler);
    web_server_add_route(server, "POST", "/", middleware_hello_handler);
    web_server_add_route(server, "OPTIONS", "/", options_handler);

    // 健康检查路由（跳过认证）
    web_server_add_route(server, "GET", "/health", health_check_handler);

    // 受保护的 API 路由（需要认证）
    web_server_add_route(server, "GET", "/api/protected", protected_api_handler);
    web_server_add_route(server, "POST", "/api/protected", protected_api_handler);

    // 用户管理路由（需要认证和限流）
    web_server_add_route(server, "GET", "/api/users", user_management_handler);
    web_server_add_route(server, "POST", "/api/users", user_management_handler);
    web_server_add_route(server, "PUT", "/api/users", user_management_handler);
    web_server_add_route(server, "DELETE", "/api/users", user_management_handler);

    // CORS 预检请求
    web_server_add_route(server, "OPTIONS", "/api/users", options_handler);
    web_server_add_route(server, "OPTIONS", "/api/protected", options_handler);

    // 启动服务器
    std.debug.print("Starting server on http://127.0.0.1:8080\n");
    std.debug.print("Available routes with middleware:\n");
    std.debug.print("  GET/POST / - Hello page (with logging, CORS)\n");
    std.debug.print("  GET /health - Health check (public, no auth required)\n");
    std.debug.print("  GET/POST /api/protected - Protected API (requires JWT auth)\n");
    std.debug.print("  GET/POST/PUT/DELETE /api/users - User management (requires JWT auth)\n");
    std.debug.print("  OPTIONS * - CORS preflight (handled by CORS middleware)\n");
    std.debug.print("\n");
    std.debug.print("Middleware features:\n");
    std.debug.print("  - Request logging\n");
    std.debug.print("  - CORS support\n");
    std.debug.print("  - JWT authentication\n");
    std.debug.print("  - Rate limiting (simulated)\n");
    std.debug.print("\n");
    std.debug.print("Test commands:\n");
    std.debug.print("  curl http://127.0.0.1:8080/\n");
    std.debug.print("  curl http://127.0.0.1:8080/health\n");
    std.debug.print("  curl -H \"Authorization: Bearer valid_token_12345\" http://127.0.0.1:8080/api/protected\n");
    std.debug.print("  curl -X POST -H \"Authorization: Bearer valid_token_12345\" -H \"Content-Type: application/json\" -d '{{\"name\":\"John\"}}' http://127.0.0.1:8080/api/users\n");
    std.debug.print("\n");
    std.debug.print("Press Ctrl+C to stop\n");

    web_server_start(server, 8080);

    // 保持程序运行
    while (true) {
        std.time.sleep(1000000000); // 1 second
    }
}

// ============================================================================
// 测试函数 | Test Functions
// ============================================================================

/// 测试中间件功能
pub fn testMiddlewareFeatures() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("Testing middleware features...\n");

    // 初始化中间件
    try initializeMiddleware(allocator);
    defer cleanupMiddleware();

    if (global_middleware_chain) |*chain| {
        // 测试不同的请求
        const test_cases = [_]struct {
            method: []const u8,
            path: []const u8,
            body: []const u8,
            description: []const u8,
        }{
            .{ .method = "GET", .path = "/", .body = "", .description = "Public GET request" },
            .{ .method = "POST", .path = "/api/protected", .body = "{\"data\":\"test\"}", .description = "Protected POST request" },
            .{ .method = "OPTIONS", .path = "/api/users", .body = "", .description = "CORS preflight request" },
            .{ .method = "GET", .path = "/health", .body = "", .description = "Health check request" },
        };

        for (test_cases) |test_case| {
            std.debug.print("\nTesting: {s}\n", .{test_case.description});
            std.debug.print("Request: {} {s}\n", .{ test_case.method, test_case.path });

            const result = try chain.execute(test_case.method, test_case.path, test_case.body);
            defer allocator.free(result);

            std.debug.print("Middleware result: {s}\n", .{result});
        }
    }

    std.debug.print("\nMiddleware testing completed!\n");
}

/// 如果以测试模式运行
pub fn runTests() !void {
    try testMiddlewareFeatures();
}
