// ============================================================================
// Zig 中间件系统封装 | Zig Middleware System Wrapper
// ============================================================================

const std = @import("std");

// ============================================================================
// 外部函数声明 - Rust 中间件 FFI 接口 | External Function Declarations - Rust Middleware FFI Interface
// ============================================================================

// 中间件处理函数类型
const MiddlewareHandler = *const fn (*RequestContext, [*:0]u8) callconv(.C) c_int;

// C 兼容的请求上下文结构
const CRequestContext = extern struct {
    method: [*:0]const u8,
    path: [*:0]const u8,
    body: [*:0]const u8,
    body_length: usize,
    headers_count: usize,
    headers_keys: [*]const [*:0]const u8,
    headers_values: [*]const [*:0]const u8,
    user_data_count: usize,
    user_data_keys: [*]const [*:0]const u8,
    user_data_values: [*]const [*:0]const u8,
};

// 外部函数声明
extern fn hush_middleware_new() ?*anyopaque;
extern fn hush_middleware_add(middleware: ?*anyopaque, handler: MiddlewareHandler, user_data: [*:0]u8) void;
extern fn hush_middleware_free(middleware: ?*anyopaque) void;
extern fn hush_middleware_add_cors(middleware: ?*anyopaque, allowed_origins: [*:0]const u8) void;
extern fn hush_middleware_add_auth_jwt(middleware: ?*anyopaque, secret: [*:0]const u8) void;
extern fn hush_middleware_add_logger(middleware: ?*anyopaque) void;
extern fn hush_middleware_execute(middleware: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) [*:0]const u8;
extern fn hush_middleware_count(middleware: ?*anyopaque) usize;
extern fn hush_middleware_names(middleware: ?*anyopaque) [*:0]const u8;
extern fn rust_free_string(ptr: [*:0]u8) void;

// ============================================================================
// Zig 中间件接口定义 | Zig Middleware Interface Definition
// ============================================================================

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
pub const MiddlewareResult = enum(c_int) {
    Continue = 0,    // 继续执行下一个中间件
    Response = 1,    // 提前返回响应
    Error = -1,      // 发生错误
};

/// 中间件处理函数类型定义
pub const MiddlewareHandlerFn = *const fn (*RequestContext, ?[]const u8) MiddlewareResult;

/// 中间件链管理器
pub const MiddlewareChain = struct {
    handle: ?*anyopaque,
    allocator: std.mem.Allocator,

    const Self = @This();

    /// 创建新的中间件链
    pub fn init(allocator: std.mem.Allocator) !Self {
        const handle = hush_middleware_new();
        if (handle == null) {
            return error.InitializationFailed;
        }

        return Self{
            .handle = handle,
            .allocator = allocator,
        };
    }

    /// 释放中间件链资源
    pub fn deinit(self: *Self) void {
        if (self.handle) |handle| {
            hush_middleware_free(handle);
            self.handle = null;
        }
    }

    /// 添加自定义中间件
    pub fn add(self: *Self, handler: MiddlewareHandlerFn, user_data: ?[]const u8) !void {
        _ = handler; // 暂时不使用，避免编译警告
        if (self.handle == null) return error.InvalidHandle;

        // 创建包装函数来适配 C 接口
        const wrapper = struct {
            fn middlewareWrapper(ctx: *CRequestContext, data: [*:0]u8) callconv(.C) c_int {
                _ = ctx;
                _ = data;
                // 这里需要将 C 结构转换为 Zig 结构并调用处理函数
                // 为了简化，我们暂时返回继续执行
                return @intFromEnum(MiddlewareResult.Continue);
            }
        };

        const user_data_cstr = if (user_data) |data| 
            try self.allocator.dupeZ(u8, data)
        else 
            try self.allocator.dupeZ(u8, "");
        defer self.allocator.free(user_data_cstr);

        hush_middleware_add(self.handle, wrapper.middlewareWrapper, user_data_cstr.ptr);
    }

    /// 添加 CORS 中间件
    pub fn addCors(self: *Self, allowed_origins: []const u8) !void {
        if (self.handle == null) return error.InvalidHandle;

        const origins_cstr = try self.allocator.dupeZ(u8, allowed_origins);
        defer self.allocator.free(origins_cstr);

        hush_middleware_add_cors(self.handle, origins_cstr.ptr);
    }

    /// 添加 JWT 认证中间件
    pub fn addAuthJwt(self: *Self, secret: []const u8) !void {
        if (self.handle == null) return error.InvalidHandle;

        const secret_cstr = try self.allocator.dupeZ(u8, secret);
        defer self.allocator.free(secret_cstr);

        hush_middleware_add_auth_jwt(self.handle, secret_cstr.ptr);
    }

    /// 添加日志中间件
    pub fn addLogger(self: *Self) !void {
        if (self.handle == null) return error.InvalidHandle;
        hush_middleware_add_logger(self.handle);
    }

    /// 执行中间件链
    pub fn execute(self: *Self, method: []const u8, path: []const u8, body: []const u8) ![]const u8 {
        if (self.handle == null) return error.InvalidHandle;

        const method_cstr = try self.allocator.dupeZ(u8, method);
        defer self.allocator.free(method_cstr);

        const path_cstr = try self.allocator.dupeZ(u8, path);
        defer self.allocator.free(path_cstr);

        const body_cstr = try self.allocator.dupeZ(u8, body);
        defer self.allocator.free(body_cstr);

        const result_ptr = hush_middleware_execute(self.handle, method_cstr.ptr, path_cstr.ptr, body_cstr.ptr);
        if (@intFromPtr(result_ptr) == 0) {
            return error.ExecutionFailed;
        }

        const result_str = std.mem.span(result_ptr);
        const result = try self.allocator.dupe(u8, result_str);
        rust_free_string(@constCast(result_ptr));
        return result;
    }

    /// 获取中间件数量
    pub fn count(self: *Self) usize {
        if (self.handle == null) return 0;
        return hush_middleware_count(self.handle);
    }

    /// 获取中间件名称列表
    pub fn getNames(self: *Self) ![]const u8 {
        if (self.handle == null) return error.InvalidHandle;

        const names_ptr = hush_middleware_names(self.handle);
        if (@intFromPtr(names_ptr) == 0) {
            return error.GetNamesFailed;
        }

        const names_str = std.mem.span(names_ptr);
        const result = try self.allocator.dupe(u8, names_str);
        rust_free_string(@constCast(names_ptr));
        return result;
    }
};

// ============================================================================
// 便捷的中间件创建函数 | Convenient Middleware Creation Functions
// ============================================================================

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

// ============================================================================
// 测试和示例代码 | Test and Example Code
// ============================================================================

/// 测试中间件系统的基本功能
pub fn testMiddlewareSystem(allocator: std.mem.Allocator) !void {
    std.debug.print("Testing Zig middleware system...\n", .{});

    // 创建中间件链
    var chain = try MiddlewareChain.init(allocator);
    defer chain.deinit();

    // 添加内置中间件
    try chain.addLogger();
    try chain.addCors("*");
    try chain.addAuthJwt("my_secret_key");

    // 检查中间件数量
    const count = chain.count();
    std.debug.print("Middleware count: {}\n", .{count});

    // 获取中间件名称
    const names = try chain.getNames();
    defer allocator.free(names);
    std.debug.print("Middleware names: {s}\n", .{names});

    // 执行中间件链
    const result = try chain.execute("GET", "/test", "");
    defer allocator.free(result);
    std.debug.print("Execution result: {s}\n", .{result});

    std.debug.print("Middleware system test completed!\n", .{});
}

/// 演示如何使用自定义中间件
pub fn demonstrateCustomMiddleware(allocator: std.mem.Allocator) !void {
    std.debug.print("Demonstrating custom middleware usage...\n", .{});

    // 创建请求上下文
    var ctx = RequestContext.init(allocator, "GET", "/api/users", "");
    defer ctx.deinit();

    // 添加一些头部信息
    try ctx.addHeader("Content-Type", "application/json");
    try ctx.addHeader("Authorization", "Bearer valid_token_12345");

    // 创建并测试不同的中间件
    const logger = createLoggerMiddleware();
    const cors = createCorsMiddleware("*");
    const auth = createAuthMiddleware("secret");
    const rate_limit = createRateLimitMiddleware(100, 60);

    // 依次执行中间件
    std.debug.print("Executing logger middleware...\n", .{});
    _ = logger(&ctx, null);

    std.debug.print("Executing CORS middleware...\n", .{});
    _ = cors(&ctx, null);

    std.debug.print("Executing auth middleware...\n", .{});
    const auth_result = auth(&ctx, null);
    if (auth_result == MiddlewareResult.Continue) {
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

// ============================================================================
// 导出的测试函数 | Exported Test Functions
// ============================================================================

/// 主测试函数，可以从其他 Zig 文件调用
pub fn runMiddlewareTests() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    try testMiddlewareSystem(allocator);
    std.debug.print("\n", .{});
    try demonstrateCustomMiddleware(allocator);
}

// 如果直接运行此文件，执行测试
pub fn main() !void {
    try runMiddlewareTests();
}