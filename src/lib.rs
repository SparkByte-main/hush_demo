// ============================================================================
// 依赖导入 | Dependencies Import
// ============================================================================

// actix-web: 高性能的 Rust web 框架 | actix-web: High-performance Rust web framework
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

// 标准库导入 | Standard library imports
use std::collections::HashMap;           // 哈希映射，用于存储路由 | HashMap for storing routes
use std::ffi::{c_char, CStr, CString};   // C 语言 FFI 类型 | C language FFI types
use std::sync::{Arc, Mutex};             // 线程安全的共享数据结构 | Thread-safe shared data structures
use std::thread;                         // 线程操作 | Thread operations

// ============================================================================
// 示例函数：基本的 FFI 演示 | Example Function: Basic FFI Demonstration
// ============================================================================

// #[unsafe(no_mangle)] 属性详解 | #[unsafe(no_mangle)] attribute explanation:
// - no_mangle: 告诉编译器不要改变函数名，保持原始名称 | no_mangle: Tell compiler not to change function name, keep original name
//   这样其他语言（如 Zig）可以通过确切的函数名找到这个函数 | So other languages (like Zig) can find this function by exact name
// - unsafe(): 在新版本 Rust 中，no_mangle 被认为是不安全的属性 | unsafe(): In newer Rust versions, no_mangle is considered unsafe attribute
//   因为它可能导致符号冲突，所以需要用 unsafe() 包装 | Because it may cause symbol conflicts, needs to be wrapped with unsafe()
//
// 函数签名解释 | Function signature explanation:
// - pub: 使函数对外部可见 | pub: Make function visible to external code
// - extern "C": 使用 C 调用约定，确保与其他语言的兼容性 | extern "C": Use C calling convention, ensure compatibility with other languages
// - -> *const c_char: 返回一个指向 C 字符串的常量指针 | -> *const c_char: Return a constant pointer to C string
#[unsafe(no_mangle)]
pub extern "C" fn rust_hello_world() -> *const c_char {
    // 创建一个 Rust 字符串并转换为 C 风格的字符串 | Create a Rust string and convert to C-style string
    // CString::new() 会在字符串末尾添加空终止符 '\0' | CString::new() adds null terminator '\0' at end of string
    // unwrap() 用于处理可能的错误（如字符串中包含空字节）| unwrap() handles possible errors (like null bytes in string)
    let hello = CString::new("Hello, World!").unwrap();

    // into_raw() 将 CString 转换为原始指针并转移所有权 | into_raw() converts CString to raw pointer and transfers ownership
    // 内存管理注意事项 | Memory management notes:
    // 1. 返回的指针可以被 C/Zig 代码安全使用 | 1. Returned pointer can be safely used by C/Zig code
    // 2. 内存不会被 Rust 自动释放 | 2. Memory won't be automatically freed by Rust
    // 3. 调用方负责释放内存（在这个简单示例中我们忽略了这点）| 3. Caller is responsible for freeing memory (ignored in this simple example)
    hello.into_raw()
}

// ============================================================================
// Web 框架核心数据结构 | Web Framework Core Data Structures
// ============================================================================

// Web 服务器结构体，包含路由映射表 | Web server struct containing route mapping table
pub struct WebServer {
    // 路由存储：使用 "METHOD:PATH" 格式作为键 | Route storage: using "METHOD:PATH" format as key
    // 例如："GET:/", "POST:/users" | Examples: "GET:/", "POST:/users"
    // Arc<Mutex<>> 确保多线程安全访问 | Arc<Mutex<>> ensures thread-safe access
    // 值是指向 Zig 处理函数的函数指针 | Value is function pointer to Zig handler function
    routes: Arc<Mutex<HashMap<String, extern "C" fn(*const c_char, *const c_char) -> *const c_char>>>,
}

// 全局服务器实例指针，用于在异步处理函数中访问路由 | Global server instance pointer for accessing routes in async handlers
// 注意：使用全局状态不是最佳实践，但简化了 FFI 接口设计 | Note: Using global state is not best practice, but simplifies FFI interface design
static mut GLOBAL_SERVER: Option<*mut WebServer> = None;

// ============================================================================
// Web 框架 FFI 接口函数 | Web Framework FFI Interface Functions
// ============================================================================

// 创建新的 web 服务器实例 | Create new web server instance
// 返回值：指向 WebServer 结构体的原始指针 | Return: Raw pointer to WebServer struct
// 注意：调用方负责最终调用 web_server_free() 释放内存 | Note: Caller is responsible for calling web_server_free() to release memory
#[unsafe(no_mangle)]
pub extern "C" fn web_server_new() -> *mut WebServer {
    // 在堆上创建 WebServer 实例 | Create WebServer instance on heap
    let server = Box::new(WebServer {
        routes: Arc::new(Mutex::new(HashMap::new())), // 初始化空的路由表 | Initialize empty route table
    });
    
    // 将 Box 转换为原始指针，转移所有权 | Convert Box to raw pointer, transfer ownership
    let server_ptr = Box::into_raw(server);

    // 设置全局服务器引用，供异步处理函数使用 | Set global server reference for async handler functions
    unsafe {
        GLOBAL_SERVER = Some(server_ptr);
    }

    server_ptr
}

// 添加路由到服务器 | Add route to server
// 参数说明 | Parameters:
// - server: 服务器实例指针 | server: Server instance pointer
// - method: HTTP 方法（如 "GET", "POST"）| method: HTTP method (e.g. "GET", "POST")
// - path: 路由路径（如 "/", "/users"）| path: Route path (e.g. "/", "/users")
// - handler: Zig 处理函数指针，接收 method 和 path 参数 | handler: Zig handler function pointer, receives method and path parameters
#[unsafe(no_mangle)]
pub extern "C" fn web_server_add_route(
    server: *mut WebServer,
    method: *const c_char,
    path: *const c_char,
    handler: extern "C" fn(*const c_char, *const c_char) -> *const c_char,
) {
    // 参数有效性检查 | Parameter validity check
    if server.is_null() || method.is_null() || path.is_null() {
        return;
    }

    unsafe {
        // 将 C 字符串转换为 Rust 字符串 | Convert C strings to Rust strings
        let method_str = CStr::from_ptr(method).to_string_lossy().to_string();
        let path_str = CStr::from_ptr(path).to_string_lossy().to_string();
        
        // 创建路由键：格式为 "METHOD:PATH" | Create route key: format "METHOD:PATH"
        let route_key = format!("{}:{}", method_str, path_str);
        let server_ref = &*server;

        // 获取路由表的互斥锁并插入新路由 | Acquire route table mutex lock and insert new route
        if let Ok(mut routes) = server_ref.routes.lock() {
            routes.insert(route_key, handler);
        }
    }
}

// ============================================================================
// HTTP 请求处理核心逻辑 | HTTP Request Processing Core Logic
// ============================================================================

// 通用路由处理函数，处理所有传入的 HTTP 请求 | Generic route handler for all incoming HTTP requests
// 此函数由 actix-web 框架调用，负责路由分发和 Zig 处理函数调用 | Called by actix-web framework, responsible for route dispatching and Zig handler invocation
async fn handle_request(req: HttpRequest) -> HttpResponse {
    // 提取 HTTP 方法和路径 | Extract HTTP method and path
    let method = req.method().as_str();  // 如 "GET", "POST" | e.g. "GET", "POST"
    let path = req.path();               // 如 "/", "/users" | e.g. "/", "/users"
    
    // 构造路由键用于查找处理函数 | Construct route key for handler lookup
    let route_key = format!("{}:{}", method, path);

    unsafe {
        // 检查全局服务器实例是否存在 | Check if global server instance exists
        if let Some(server_ptr) = GLOBAL_SERVER {
            let server_ref = &*server_ptr;

            // 获取路由表的读锁 | Acquire read lock on route table
            if let Ok(routes) = server_ref.routes.lock() {
                // 查找匹配的路由处理函数 | Look for matching route handler
                if let Some(handler) = routes.get(&route_key) {
                    // 创建方法和路径的 C 字符串，传递给 Zig | Create C strings for method and path to pass to Zig
                    let method_cstr = CString::new(method).unwrap();
                    let path_cstr = CString::new(path).unwrap();
                    
                    // 调用 Zig 处理函数，传递方法和路径参数 | Call Zig handler function with method and path parameters
                    let response_ptr = handler(method_cstr.as_ptr(), path_cstr.as_ptr());

                    // 检查 Zig 函数是否返回有效响应 | Check if Zig function returned valid response
                    if !response_ptr.is_null() {
                        let response_str = CStr::from_ptr(response_ptr).to_string_lossy();
                        return HttpResponse::Ok().body(response_str.to_string());
                    }
                }
            }
        }
    }

    // 如果没有找到匹配的路由，返回 404 | Return 404 if no matching route found
    HttpResponse::NotFound().body("Route not found")
}

// 启动 web 服务器 | Start web server
// 参数说明 | Parameters:
// - server: 服务器实例指针 | server: Server instance pointer
// - port: 监听端口号 | port: Port number to listen on
// 注意：此函数在新线程中启动服务器，不会阻塞调用方 | Note: This function starts server in new thread, won't block caller
#[unsafe(no_mangle)]
pub extern "C" fn web_server_start(server: *mut WebServer, port: u16) {
    // 参数有效性检查 | Parameter validity check
    if server.is_null() {
        return;
    }

    // 在新线程中启动服务器，避免阻塞 Zig 主线程 | Start server in new thread to avoid blocking Zig main thread
    thread::spawn(move || {
        // 创建 tokio 异步运行时 | Create tokio async runtime
        let rt = tokio::runtime::Runtime::new().unwrap();

        // 在异步运行时中启动 actix-web 服务器 | Start actix-web server in async runtime
        rt.block_on(async {
            println!("Starting web framework server on port {}", port);

            // 创建 HTTP 服务器实例 | Create HTTP server instance
            // default_service: 将所有请求路由到 handle_request 函数 | default_service: Route all requests to handle_request function
            HttpServer::new(|| App::new().default_service(web::route().to(handle_request)))
                .bind(("127.0.0.1", port))           // 绑定到本地地址和指定端口 | Bind to localhost and specified port
                .expect("Failed to bind server")     // 绑定失败时 panic | Panic if binding fails
                .run()                               // 启动服务器 | Start server
                .await                               // 等待服务器运行 | Wait for server to run
                .expect("Failed to run server");     // 运行失败时 panic | Panic if server fails to run
        });
    });
}

// 释放服务器资源 | Free server resources
#[unsafe(no_mangle)]
pub extern "C" fn web_server_free(server: *mut WebServer) {
    if !server.is_null() {
        unsafe {
            let _ = Box::from_raw(server);
            GLOBAL_SERVER = None;
        }
    }
}

// 注意：在生产代码中，应该提供一个对应的释放函数 | Note: In production code, should provide corresponding free function
// 例如 | Example:
#[unsafe(no_mangle)]
pub extern "C" fn rust_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
