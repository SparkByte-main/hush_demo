// ============================================================================
// Hush 框架主入口文件 | Hush Framework Main Entry File
// ============================================================================

// Remove unused actix-web imports since we're using the new modular structure
use std::collections::HashMap;
use std::ffi::{c_char, CString};
use std::sync::{Arc, Mutex};


// 导入核心模块
mod core;
mod web;
mod middleware;

// 使用核心模块
use core::error::{HushError, set_last_error};
use core::ffi::{from_c_string, to_c_string};
use core::types::{HttpMethod, RequestContext, HttpStatus};
use web::server::WebServer;
use web::handler::{RequestHandler, ResponseBuilder};

// ============================================================================
// 示例函数：基本的 FFI 演示 | Example Function: Basic FFI Demonstration
// ============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn rust_hello_world() -> *const c_char {
    let hello = CString::new("Hello, World!").unwrap();
    hello.into_raw()
}

// ============================================================================
// Zig 处理函数映射和全局状态 | Zig Handler Mapping and Global State
// ============================================================================

// Zig 处理函数类型定义
type ZigHandlerFn = extern "C" fn(*const c_char, *const c_char, *const c_char) -> *const c_char;

// 存储 Zig 处理函数的映射表
static mut ZIG_HANDLERS: Option<Arc<Mutex<HashMap<String, ZigHandlerFn>>>> = None;

// 全局服务器实例指针，用于 FFI 接口
static mut GLOBAL_SERVER: Option<*mut WebServer> = None;

// 初始化 Zig 处理函数映射表
fn init_zig_handlers() {
    unsafe {
        if ZIG_HANDLERS.is_none() {
            ZIG_HANDLERS = Some(Arc::new(Mutex::new(HashMap::new())));
        }
    }
}

// 获取 Zig 处理函数映射表
fn get_zig_handlers() -> Option<Arc<Mutex<HashMap<String, ZigHandlerFn>>>> {
    unsafe { ZIG_HANDLERS.clone() }
}

// ============================================================================
// Web 框架 FFI 接口函数 | Web Framework FFI Interface Functions
// ============================================================================

/// 创建新的 web 服务器实例
#[unsafe(no_mangle)]
pub extern "C" fn web_server_new() -> *mut WebServer {
    // 初始化 Zig 处理函数映射表
    init_zig_handlers();
    
    // 创建 WebServer 实例
    let server = Box::new(WebServer::with_default_config());
    let server_ptr = Box::into_raw(server);

    // 设置全局服务器引用
    unsafe {
        GLOBAL_SERVER = Some(server_ptr);
    }

    server_ptr
}

/// 添加路由到服务器
#[unsafe(no_mangle)]
pub extern "C" fn web_server_add_route(
    server: *mut WebServer,
    method: *const c_char,
    path: *const c_char,
    handler: ZigHandlerFn,
) {
    // 参数有效性检查
    if server.is_null() || method.is_null() || path.is_null() {
        set_last_error(HushError::InvalidParameter);
        return;
    }

    unsafe {
        let server_ref = &*server;
        
        // 将 C 字符串转换为 Rust 字符串
        let method_result = from_c_string(method);
        let path_result = from_c_string(path);
        
        match (method_result, path_result) {
            (Ok(method_str), Ok(path_str)) => {
                // 解析 HTTP 方法
                match HttpMethod::from_str(&method_str) {
                    Ok(http_method) => {
                        // 创建路由键
                        let route_key = format!("{}:{}", method_str, path_str);
                        
                        // 存储 Zig 处理函数
                        if let Some(handlers) = get_zig_handlers() {
                            if let Ok(mut handlers_map) = handlers.lock() {
                                handlers_map.insert(route_key.clone(), handler);
                            }
                        }
                        
                        // 创建 Rust 处理函数包装器
                        let rust_handler = RequestHandler::new(move |context: &RequestContext| {
                            // 获取 Zig 处理函数
                            if let Some(handlers) = get_zig_handlers() {
                                if let Ok(handlers_map) = handlers.lock() {
                                    if let Some(zig_handler) = handlers_map.get(&route_key) {
                                        // 准备参数
                                        let method_cstr = to_c_string(context.method.as_str())?;
                                        let path_cstr = to_c_string(&context.path)?;
                                        let body_str = context.body_as_string().unwrap_or_default();
                                        let body_cstr = to_c_string(&body_str)?;
                                        
                                        // 调用 Zig 处理函数
                                        let response_ptr = zig_handler(
                                            method_cstr.as_ptr(),
                                            path_cstr.as_ptr(),
                                            body_cstr.as_ptr()
                                        );
                                        
                                        // 处理响应
                                        if !response_ptr.is_null() {
                                            let response_str = from_c_string(response_ptr)?;
                                            return Ok(ResponseBuilder::new(HttpStatus::Ok)
                                                .text(&response_str)
                                                .build());
                                        }
                                    }
                                }
                            }
                            
                            Err(HushError::InternalError("Handler not found".to_string()))
                        });
                        
                        // 添加路由到服务器
                        if let Err(error) = server_ref.add_route(http_method, &path_str, rust_handler) {
                            set_last_error(error);
                        }
                    }
                    Err(error) => {
                        set_last_error(error);
                    }
                }
            }
            _ => {
                set_last_error(HushError::FFIError("Invalid string parameters".to_string()));
            }
        }
    }
}

/// 启动 web 服务器
#[unsafe(no_mangle)]
pub extern "C" fn web_server_start(server: *mut WebServer, port: u16) {
    if server.is_null() {
        set_last_error(HushError::InvalidParameter);
        return;
    }

    unsafe {
        let server_ref = &*server;
        if let Err(error) = server_ref.start_with_port(port) {
            set_last_error(error);
        }
    }
}

/// 释放服务器资源
#[unsafe(no_mangle)]
pub extern "C" fn web_server_free(server: *mut WebServer) {
    if !server.is_null() {
        unsafe {
            let _ = Box::from_raw(server);
            GLOBAL_SERVER = None;
        }
    }
}

/// 释放 Rust 分配的字符串内存
#[unsafe(no_mangle)]
pub extern "C" fn rust_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// ============================================================================
// 导出核心 FFI 接口 | Export Core FFI Interfaces
// ============================================================================

// 重新导出核心错误处理函数
pub use core::error::{hush_get_last_error, hush_get_last_error_code, hush_clear_error, hush_free_error_string};

// 重新导出核心 FFI 工具函数
pub use core::ffi::{hush_free_string, hush_string_clone, hush_string_length, hush_string_compare};

// 重新导出内存管理函数
pub use core::memory::{hush_malloc, hush_free, hush_realloc, hush_memcpy, hush_memset};

// 重新导出中间件 FFI 接口
pub use middleware::ffi::{
    hush_middleware_new, hush_middleware_add, hush_middleware_free,
    hush_middleware_add_cors, hush_middleware_add_auth_jwt, hush_middleware_add_logger,
    hush_middleware_execute, hush_middleware_count, hush_middleware_names
};