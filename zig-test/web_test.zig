// ============================================================================
// Zig Web 应用程序 - 使用 Rust actix-web 框架 | Zig Web Application - Using Rust actix-web Framework
// ============================================================================

// 导入 Zig 标准库 | Import Zig standard library
const std = @import("std");

// ============================================================================
// 外部函数声明 - Rust Web 框架 FFI 接口 | External Function Declarations - Rust Web Framework FFI Interface
// ============================================================================

// 创建新的 web 服务器实例 | Create new web server instance
// 返回值：不透明指针，指向 Rust 端的 WebServer 结构体 | Return: Opaque pointer to WebServer struct on Rust side
extern fn web_server_new() ?*anyopaque;

// 添加路由到服务器 | Add route to server
// 参数说明 | Parameters:
// - server: 服务器实例指针 | server: Server instance pointer
// - method: HTTP 方法字符串（如 "GET", "POST"）| method: HTTP method string (e.g. "GET", "POST")
// - path: 路由路径字符串（如 "/", "/users"）| path: Route path string (e.g. "/", "/users")
// - handler: Zig 处理函数指针，使用 C 调用约定 | handler: Zig handler function pointer with C calling convention
extern fn web_server_add_route(server: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, handler: *const fn ([*:0]const u8, [*:0]const u8) callconv(.C) [*:0]const u8) void;

// 启动 web 服务器 | Start web server
// 参数说明 | Parameters:
// - server: 服务器实例指针 | server: Server instance pointer
// - port: 监听端口号 | port: Port number to listen on
extern fn web_server_start(server: ?*anyopaque, port: u16) void;

// 释放服务器资源 | Free server resources
// 参数说明 | Parameters:
// - server: 要释放的服务器实例指针 | server: Server instance pointer to free
extern fn web_server_free(server: ?*anyopaque) void;

// 释放 Rust 分配的字符串内存 | Free string memory allocated by Rust
// 参数说明 | Parameters:
// - ptr: 要释放的字符串指针 | ptr: String pointer to free
extern fn rust_free_string(ptr: [*:0]u8) void;

// ============================================================================
// 路由处理函数 - 业务逻辑实现 | Route Handler Functions - Business Logic Implementation
// ============================================================================

// 根路径处理函数：处理 "/" 路径的 GET 和 POST 请求 | Root path handler: handles GET and POST requests for "/" path
// 函数签名说明 | Function signature explanation:
// - export: 导出函数供 Rust 通过 FFI 调用 | export: Export function for Rust to call via FFI
// - callconv(.C): 使用 C 调用约定，确保与 Rust 兼容 | callconv(.C): Use C calling convention for Rust compatibility
// - 参数 method: HTTP 方法字符串指针 | Parameter method: HTTP method string pointer
// - 参数 path: 请求路径字符串指针 | Parameter path: Request path string pointer
// - 返回值: 响应内容的字符串指针 | Return: Response content string pointer
export fn hello_handler(method: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 暂时不使用请求路径参数 | Temporarily not using request path parameter

    // 将 C 字符串转换为 Zig 字符串切片 | Convert C string to Zig string slice
    const method_str = std.mem.span(method);

    // 使用 C 分配器创建响应字符串 | Create response string using C allocator
    // 注意：使用 C 分配器确保内存可以被 Rust 端正确处理 | Note: Using C allocator ensures memory can be properly handled by Rust side
    const allocator = std.heap.c_allocator;
    const response = std.fmt.allocPrintZ(allocator, "Hello from Zig! Method: {s}", .{method_str}) catch {
        // 内存分配失败时返回错误信息 | Return error message if memory allocation fails
        return "Error: Memory allocation failed".ptr;
    };

    return response.ptr;
}

// About 页面处理函数：处理 "/about" 路径的请求 | About page handler: handles requests for "/about" path
// 功能：返回包含 HTTP 方法信息的关于页面内容 | Function: Returns about page content with HTTP method information
export fn about_handler(method: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 当前实现中不使用路径参数 | Path parameter not used in current implementation

    // 提取 HTTP 方法信息 | Extract HTTP method information
    const method_str = std.mem.span(method);

    // 分配内存并格式化响应字符串 | Allocate memory and format response string
    const allocator = std.heap.c_allocator;
    const response = std.fmt.allocPrintZ(allocator, "About page - Method: {s} - This is a Zig web application using Rust framework!", .{method_str}) catch {
        return "Error: Memory allocation failed".ptr;
    };

    return response.ptr;
}

// 测试处理函数：演示自定义路由处理 | Test handler function: demonstrates custom route handling
// 功能：返回简单的测试响应，包含 HTTP 方法信息 | Function: Returns simple test response with HTTP method information
export fn web_test_hello_handler(method: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 当前实现中不使用路径参数 | Path parameter not used in current implementation

    // 提取 HTTP 方法信息 | Extract HTTP method information
    const method_str = std.mem.span(method);

    // 分配内存并格式化响应字符串 | Allocate memory and format response string
    const allocator = std.heap.c_allocator;
    const response = std.fmt.allocPrintZ(allocator, "Hello from Zig web application! Method: {s}", .{method_str}) catch {
        return "Error: Memory allocation failed".ptr;
    };

    return response.ptr;
}

// ============================================================================
// 应用程序入口点 | Application Entry Point
// ============================================================================

// 主函数：初始化并启动 Zig web 应用程序 | Main function: Initialize and start Zig web application
// 功能流程 | Function flow:
// 1. 创建 web 服务器实例 | 1. Create web server instance
// 2. 注册路由和处理函数 | 2. Register routes and handler functions
// 3. 启动 HTTP 服务器 | 3. Start HTTP server
// 4. 保持程序运行 | 4. Keep program running
pub fn main() !void {
    // 应用程序启动信息 | Application startup information
    std.debug.print("Creating Zig web application with Rust framework...\n", .{});

    // ========================================================================
    // 第一步：创建 web 服务器实例 | Step 1: Create web server instance
    // ========================================================================
    const server = web_server_new();
    if (server == null) {
        std.debug.print("Failed to create web server\n", .{});
        return;
    }

    // ========================================================================
    // 第二步：注册路由和处理函数 | Step 2: Register routes and handler functions
    // ========================================================================
    std.debug.print("Registering routes...\n", .{});

    // 根路径路由：支持 GET 和 POST 方法 | Root path routes: support GET and POST methods
    web_server_add_route(server, "GET", "/", hello_handler);
    web_server_add_route(server, "POST", "/", hello_handler);

    // About 页面路由：支持 GET 和 POST 方法 | About page routes: support GET and POST methods
    web_server_add_route(server, "GET", "/about", about_handler);
    web_server_add_route(server, "POST", "/about", about_handler);

    // 测试路由：仅支持 GET 方法 | Test route: only support GET method
    web_server_add_route(server, "GET", "/web_test_hello", web_test_hello_handler);

    // ========================================================================
    // 第三步：启动 HTTP 服务器 | Step 3: Start HTTP server
    // ========================================================================
    std.debug.print("Starting server on http://127.0.0.1:8080\n", .{});
    std.debug.print("Routes available:\n", .{});
    std.debug.print("  GET/POST / - Hello page (shows HTTP method)\n", .{});
    std.debug.print("  GET/POST /about - About page (shows HTTP method)\n", .{});
    std.debug.print("  GET /web_test_hello - Test page (shows HTTP method)\n", .{});
    std.debug.print("Press Ctrl+C to stop\n", .{});

    // 启动服务器（在新线程中运行，不阻塞主线程）| Start server (runs in new thread, doesn't block main thread)
    web_server_start(server, 7070);

    // ========================================================================
    // 第四步：保持程序运行 | Step 4: Keep program running
    // ========================================================================
    // 无限循环保持主线程活跃，直到用户按 Ctrl+C 终止程序 | Infinite loop keeps main thread alive until user presses Ctrl+C
    while (true) {
        std.time.sleep(1000000000); // 睡眠 1 秒，减少 CPU 使用率 | Sleep for 1 second to reduce CPU usage
    }

    // 注意：在实际应用中，应该在程序退出时调用 web_server_free(server) 释放资源
    // Note: In real applications, should call web_server_free(server) to release resources on program exit
}
