// ============================================================================
// Zig 中间件系统测试 | Zig Middleware System Test
// ============================================================================

const std = @import("std");

// 模拟的中间件接口，用于测试 Zig 代码结构
const MockMiddleware = struct {
    /// 请求上下文结构体
    pub const RequestContext = struct {
        method: []const u8,
        path: []const u8,
        body: []const u8,
        headers: std.StringHashMap([]const u8),
        user_data: std.StringHashMap([]const u8),
        allocator: std.mem.Allocator,

        const Self = @This();

        /// 创建新的请求上下文
        pub fn init(allocator: std.mem.Allocator, method: []const u8, path: []const u8, body: []const u8) Self {
            return Self{
                .method = method,
                .path = path,
                .body = body,
                .headers = std.StringHashMap([]const u8).init(allocator),
                .user_data = std.StringHashMap([]const u8).init(allocator),
                .allocator = allocator,
            };
        }

        /// 释放资源
        pub fn deinit(self: *Self) void {
            self.headers.deinit();
            self.user_data.deinit();
        }

        /// 添加头部信息
        pub fn addHeader(self: *Self, key: []const u8, value: []const u8) !void {
            try self.headers.put(key, value);
        }

        /// 获取头部信息
        pub fn getHeader(self: *Self, key: []const u8) ?[]const u8 {
            return self.headers.get(key);
        }

        /// 设置用户数据
        pub fn setUserData(self: *Self, key: []const u8, value: []const u8) !void {
            try self.user_data.put(key, value);
        }

        /// 获取用户数据
        pub fn getUserData(self: *Self, key: []const u8) ?[]const u8 {
            return self.user_data.get(key);
        }

        /// 检查是否已认证
        pub fn isAuthenticated(self: *Self) bool {
            return self.getUserData("authenticated") != null;
        }

        /// 获取认证令牌
        pub fn getAuthToken(self: *Self) ?[]const u8 {
            return self.getUserData("token");
        }
    };

    /// 中间件处理结果
    pub const MiddlewareResult = enum {
        Continue,    // 继续执行下一个中间件
        Response,    // 提前返回响应
        Error,       // 发生错误
    };

    /// 中间件处理函数类型定义
    pub const MiddlewareHandlerFn = *const fn (*RequestContext, ?[]const u8) MiddlewareResult;

    /// 创建日志中间件处理函数
    pub fn createLoggerMiddleware() MiddlewareHandlerFn {
        const impl = struct {
            fn handler(ctx: *RequestContext, user_data: ?[]const u8) MiddlewareResult {
                _ = user_data;
                std.debug.print("[MIDDLEWARE] {s} {s} - Processing request\n", .{ ctx.method, ctx.path });
                return MiddlewareResult.Continue;
            }
        };
        return impl.handler;
    }

    /// 创建 CORS 中间件处理函数
    pub fn createCorsMiddleware(allowed_origins: []const u8) MiddlewareHandlerFn {
        _ = allowed_origins;
        const impl = struct {
            fn handler(ctx: *RequestContext, user_data: ?[]const u8) MiddlewareResult {
                _ = user_data;
                // 检查是否是 OPTIONS 预检请求
                if (std.mem.eql(u8, ctx.method, "OPTIONS")) {
                    std.debug.print("[CORS] Handling OPTIONS preflight request for {s}\n", .{ctx.path});
                    return MiddlewareResult.Response;
                }
                return MiddlewareResult.Continue;
            }
        };
        return impl.handler;
    }

    /// 创建认证中间件处理函数
    pub fn createAuthMiddleware(secret: []const u8) MiddlewareHandlerFn {
        _ = secret;
        const impl = struct {
            fn handler(ctx: *RequestContext, user_data: ?[]const u8) MiddlewareResult {
                _ = user_data;
                
                // 检查是否需要跳过认证的路径
                const skip_paths = [_][]const u8{ "/health", "/login", "/public" };
                for (skip_paths) |skip_path| {
                    if (std.mem.startsWith(u8, ctx.path, skip_path)) {
                        return MiddlewareResult.Continue;
                    }
                }

                // 检查 Authorization 头
                const auth_header = ctx.getHeader("Authorization");
                if (auth_header == null) {
                    std.debug.print("[AUTH] Missing Authorization header for {s}\n", .{ctx.path});
                    return MiddlewareResult.Response;
                }

                // 简化的令牌验证（实际应该使用 JWT 库）
                const token = auth_header.?;
                if (token.len < 10) {
                    std.debug.print("[AUTH] Invalid token for {s}\n", .{ctx.path});
                    return MiddlewareResult.Response;
                }

                // 设置认证状态
                ctx.setUserData("authenticated", "true") catch {
                    return MiddlewareResult.Error;
                };
                ctx.setUserData("token", token) catch {
                    return MiddlewareResult.Error;
                };

                std.debug.print("[AUTH] Authentication successful for {s}\n", .{ctx.path});
                return MiddlewareResult.Continue;
            }
        };
        return impl.handler;
    }

    /// 创建限流中间件处理函数
    pub fn createRateLimitMiddleware(max_requests: u32, window_seconds: u64) MiddlewareHandlerFn {
        _ = max_requests;
        _ = window_seconds;
        const impl = struct {
            fn handler(ctx: *RequestContext, user_data: ?[]const u8) MiddlewareResult {
                _ = user_data;
                
                // 简化的限流逻辑（实际应该使用更复杂的算法）
                const rate_limited = ctx.getUserData("rate_limited");
                if (rate_limited != null) {
                    std.debug.print("[RATE_LIMIT] Rate limit exceeded for {s}\n", .{ctx.path});
                    return MiddlewareResult.Response;
                }

                std.debug.print("[RATE_LIMIT] Request allowed for {s}\n", .{ctx.path});
                return MiddlewareResult.Continue;
            }
        };
        return impl.handler;
    }
};

/// 演示如何使用自定义中间件
pub fn demonstrateCustomMiddleware(allocator: std.mem.Allocator) !void {
    std.debug.print("Demonstrating custom middleware usage...\n", .{});

    // 创建请求上下文
    var ctx = MockMiddleware.RequestContext.init(allocator, "GET", "/api/users", "");
    defer ctx.deinit();

    // 添加一些头部信息
    try ctx.addHeader("Content-Type", "application/json");
    try ctx.addHeader("Authorization", "Bearer valid_token_12345");

    // 创建并测试不同的中间件
    const logger = MockMiddleware.createLoggerMiddleware();
    const cors = MockMiddleware.createCorsMiddleware("*");
    const auth = MockMiddleware.createAuthMiddleware("secret");
    const rate_limit = MockMiddleware.createRateLimitMiddleware(100, 60);

    // 依次执行中间件
    std.debug.print("Executing logger middleware...\n", .{});
    _ = logger(&ctx, null);

    std.debug.print("Executing CORS middleware...\n", .{});
    _ = cors(&ctx, null);

    std.debug.print("Executing auth middleware...\n", .{});
    const auth_result = auth(&ctx, null);
    if (auth_result == MockMiddleware.MiddlewareResult.Continue) {
        std.debug.print("Authentication successful!\n", .{});
        std.debug.print("Is authenticated: {}\n", .{ctx.isAuthenticated()});
        if (ctx.getAuthToken()) |token| {
            std.debug.print("Auth token: {s}\n", .{token});
        }
    }

    std.debug.print("Executing rate limit middleware...\n", .{});
    _ = rate_limit(&ctx, null);

    std.debug.print("Custom middleware demonstration completed!\n", .{});
}

/// 测试不同的请求场景
pub fn testMiddlewareScenarios(allocator: std.mem.Allocator) !void {
    std.debug.print("\nTesting different middleware scenarios...\n", .{});

    const test_cases = [_]struct {
        method: []const u8,
        path: []const u8,
        body: []const u8,
        auth_header: ?[]const u8,
        description: []const u8,
    }{
        .{ .method = "GET", .path = "/", .body = "", .auth_header = null, .description = "Public GET request" },
        .{ .method = "POST", .path = "/api/protected", .body = "{\"data\":\"test\"}", .auth_header = "Bearer valid_token_12345", .description = "Protected POST request with valid token" },
        .{ .method = "POST", .path = "/api/protected", .body = "{\"data\":\"test\"}", .auth_header = null, .description = "Protected POST request without token" },
        .{ .method = "OPTIONS", .path = "/api/users", .body = "", .auth_header = null, .description = "CORS preflight request" },
        .{ .method = "GET", .path = "/health", .body = "", .auth_header = null, .description = "Health check request (should skip auth)" },
    };

    const auth = MockMiddleware.createAuthMiddleware("secret");
    const cors = MockMiddleware.createCorsMiddleware("*");
    const logger = MockMiddleware.createLoggerMiddleware();

    for (test_cases) |test_case| {
        std.debug.print("\n--- Testing: {s} ---\n", .{test_case.description});
        std.debug.print("Request: {s} {s}\n", .{ test_case.method, test_case.path });
        
        var ctx = MockMiddleware.RequestContext.init(allocator, test_case.method, test_case.path, test_case.body);
        defer ctx.deinit();

        // 添加认证头（如果有）
        if (test_case.auth_header) |header| {
            try ctx.addHeader("Authorization", header);
        }

        // 执行中间件链
        _ = logger(&ctx, null);
        const cors_result = cors(&ctx, null);
        
        if (cors_result == MockMiddleware.MiddlewareResult.Response) {
            std.debug.print("Result: CORS preflight handled\n", .{});
            continue;
        }

        const auth_result = auth(&ctx, null);
        switch (auth_result) {
            MockMiddleware.MiddlewareResult.Continue => {
                std.debug.print("Result: Request allowed, proceeding to handler\n", .{});
                if (ctx.isAuthenticated()) {
                    std.debug.print("User authenticated with token: {s}\n", .{ctx.getAuthToken().?});
                }
            },
            MockMiddleware.MiddlewareResult.Response => {
                std.debug.print("Result: Request blocked by authentication middleware\n", .{});
            },
            MockMiddleware.MiddlewareResult.Error => {
                std.debug.print("Result: Error in authentication middleware\n", .{});
            },
        }
    }

    std.debug.print("\nMiddleware scenario testing completed!\n", .{});
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    std.debug.print("=== Zig Middleware System Test ===\n", .{});
    
    try demonstrateCustomMiddleware(allocator);
    try testMiddlewareScenarios(allocator);
    
    std.debug.print("\n=== All tests completed successfully! ===\n", .{});
}