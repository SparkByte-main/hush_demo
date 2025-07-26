// ============================================================================
// 综合中间件示例 | Comprehensive Middleware Example
// ============================================================================

const std = @import("std");
const print = std.debug.print;

// 导入 Hush 框架的中间件 FFI 接口
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

pub fn main() !void {
    print("=== Hush 框架中间件综合示例 ===\n\n", .{});

    // 创建中间件链
    const middleware = hush_middleware_new();
    if (middleware == null) {
        print("错误：无法创建中间件链\n", .{});
        return;
    }
    defer hush_middleware_free(middleware);

    print("1. 创建中间件链...\n", .{});
    print("   初始中间件数量: {}\n\n", .{hush_middleware_count(middleware)});

    // 添加日志中间件（最高优先级）
    print("2. 添加日志中间件...\n", .{});
    hush_middleware_add_logger(middleware);
    print("   当前中间件数量: {}\n\n", .{hush_middleware_count(middleware)});

    // 添加 CORS 中间件
    print("3. 添加 CORS 中间件...\n", .{});
    hush_middleware_add_cors(middleware, "https://example.com,https://app.com");
    print("   当前中间件数量: {}\n\n", .{hush_middleware_count(middleware)});

    // 添加请求限流中间件
    print("4. 添加请求限流中间件 (100 请求/小时)...\n", .{});
    hush_middleware_add_rate_limit(middleware, 100, 3600);
    print("   当前中间件数量: {}\n\n", .{hush_middleware_count(middleware)});

    // 添加 JWT 认证中间件
    print("5. 添加 JWT 认证中间件...\n", .{});
    hush_middleware_add_auth_jwt(middleware, "my_super_secret_key_12345");
    print("   当前中间件数量: {}\n\n", .{hush_middleware_count(middleware)});

    // 添加基于用户的限流中间件
    print("6. 添加基于用户的限流中间件...\n", .{});
    hush_middleware_add_rate_limit_by_user(middleware);
    print("   当前中间件数量: {}\n\n", .{hush_middleware_count(middleware)});

    // 显示中间件列表
    print("7. 中间件执行顺序:\n", .{});
    const names = hush_middleware_names(middleware);
    print("   {s}\n\n", .{names});

    // 测试不同类型的请求
    print("8. 测试请求处理:\n\n", .{});

    // 测试 1: OPTIONS 预检请求
    print("   测试 1: CORS 预检请求\n", .{});
    print("   请求: OPTIONS /api/users\n", .{});
    const cors_response = hush_middleware_execute(middleware, "OPTIONS", "/api/users", "");
    print("   响应: {s}\n\n", .{cors_response});

    // 测试 2: 健康检查请求（应该跳过认证）
    print("   测试 2: 健康检查请求\n", .{});
    print("   请求: GET /health\n", .{});
    const health_response = hush_middleware_execute(middleware, "GET", "/health", "");
    print("   响应: {s}\n\n", .{health_response});

    // 测试 3: 需要认证的请求（无令牌）
    print("   测试 3: 受保护的请求（无认证）\n", .{});
    print("   请求: GET /api/protected\n", .{});
    const unauth_response = hush_middleware_execute(middleware, "GET", "/api/protected", "");
    print("   响应: {s}\n\n", .{unauth_response});

    // 测试 4: POST 请求带请求体
    print("   测试 4: POST 请求\n", .{});
    print("   请求: POST /api/create\n", .{});
    print("   请求体: {{\"name\": \"test\", \"value\": 123}}\n", .{});
    const post_response = hush_middleware_execute(middleware, "POST", "/api/create", "{\"name\": \"test\", \"value\": 123}");
    print("   响应: {s}\n\n", .{post_response});

    print("=== 中间件功能演示完成 ===\n", .{});
    print("\n功能特性总结:\n", .{});
    print("✓ CORS 中间件 - 支持跨域请求配置\n", .{});
    print("✓ 日志中间件 - 记录请求和响应信息\n", .{});
    print("✓ 请求限流中间件 - 防止 API 滥用\n", .{});
    print("✓ JWT 认证中间件 - 保护敏感端点\n", .{});
    print("✓ 基于用户的限流 - 细粒度访问控制\n", .{});
    print("✓ 中间件链管理 - 按优先级自动排序\n", .{});
    print("✓ FFI 接口 - 完整的 C/Zig 兼容性\n", .{});
}

// 辅助函数：演示中间件配置的灵活性
fn demonstrateMiddlewareFlexibility() void {
    print("\n=== 中间件配置灵活性演示 ===\n", .{});

    // 创建不同配置的中间件链
    const basic_middleware = hush_middleware_new();
    if (basic_middleware != null) {
        defer hush_middleware_free(basic_middleware);
        
        print("基础配置 - 仅日志和 CORS:\n", .{});
        hush_middleware_add_logger(basic_middleware);
        hush_middleware_add_cors(basic_middleware, "*");
        print("  中间件数量: {}\n", .{hush_middleware_count(basic_middleware)});
    }

    const secure_middleware = hush_middleware_new();
    if (secure_middleware != null) {
        defer hush_middleware_free(secure_middleware);
        
        print("安全配置 - 完整的安全栈:\n", .{});
        hush_middleware_add_logger(secure_middleware);
        hush_middleware_add_cors(secure_middleware, "https://trusted.com");
        hush_middleware_add_rate_limit(secure_middleware, 50, 1800);
        hush_middleware_add_auth_jwt(secure_middleware, "secure_secret");
        hush_middleware_add_rate_limit_by_user(secure_middleware);
        print("  中间件数量: {}\n", .{hush_middleware_count(secure_middleware)});
    }
}

// 性能测试函数
fn performanceTest() void {
    print("\n=== 中间件性能测试 ===\n", .{});
    
    const middleware = hush_middleware_new();
    if (middleware == null) return;
    defer hush_middleware_free(middleware);

    // 添加所有中间件
    hush_middleware_add_logger(middleware);
    hush_middleware_add_cors(middleware, "*");
    hush_middleware_add_rate_limit(middleware, 1000, 60);

    const start_time = std.time.milliTimestamp();
    
    // 执行多次请求
    var i: u32 = 0;
    while (i < 100) : (i += 1) {
        _ = hush_middleware_execute(middleware, "GET", "/test", "");
    }
    
    const end_time = std.time.milliTimestamp();
    const duration = end_time - start_time;
    
    print("处理 100 个请求耗时: {} ms\n", .{duration});
    print("平均每个请求: {d:.2} ms\n", .{@as(f64, @floatFromInt(duration)) / 100.0});
}