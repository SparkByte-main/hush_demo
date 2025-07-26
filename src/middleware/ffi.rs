// ============================================================================
// 中间件系统 FFI 接口 | Middleware System FFI Interface
// ============================================================================


use std::ffi::{c_char, c_int, CString};
use std::sync::{Arc, Mutex};
use crate::core::error::{HushError, set_last_error};
use crate::core::ffi::{from_c_string, to_c_string};
use crate::core::types::{RequestContext, ResponseContext, HttpMethod, HttpStatus};
use super::core::{MiddlewareChain, MiddlewareContext, MiddlewareResult};
use super::builtin::{CorsMiddleware, LoggerMiddleware, AuthMiddleware, RateLimitMiddleware};

/// C 兼容的中间件处理函数类型
pub type HushMiddlewareHandler = extern "C" fn(*mut HushRequestContext, *mut c_char) -> c_int;

/// C 兼容的请求上下文结构
#[repr(C)]
pub struct HushRequestContext {
    pub method: *const c_char,
    pub path: *const c_char,
    pub body: *const c_char,
    pub body_length: usize,
    pub headers_count: usize,
    pub headers_keys: *const *const c_char,
    pub headers_values: *const *const c_char,
    pub user_data_count: usize,
    pub user_data_keys: *const *const c_char,
    pub user_data_values: *const *const c_char,
}

/// C 兼容的中间件链结构
pub struct HushMiddleware {
    chain: Arc<Mutex<MiddlewareChain>>,
}

impl HushMiddleware {
    fn new() -> Self {
        Self {
            chain: Arc::new(Mutex::new(MiddlewareChain::new())),
        }
    }
}

/// 将 Rust RequestContext 转换为 C 兼容的结构
fn request_context_to_c(context: &RequestContext) -> Result<(HushRequestContext, Vec<CString>), HushError> {
    let mut c_strings = Vec::new();
    
    // 转换方法
    let method_cstr = to_c_string(context.method.as_str())?;
    c_strings.push(method_cstr);
    let method_ptr = c_strings.last().unwrap().as_ptr();
    
    // 转换路径
    let path_cstr = to_c_string(&context.path)?;
    c_strings.push(path_cstr);
    let path_ptr = c_strings.last().unwrap().as_ptr();
    
    // 转换请求体
    let body_str = context.body_as_string().unwrap_or_default();
    let body_cstr = to_c_string(&body_str)?;
    c_strings.push(body_cstr);
    let body_ptr = c_strings.last().unwrap().as_ptr();
    
    // 转换 headers（简化处理，实际应该分配数组）
    let headers_count = context.headers.len();
    
    // 转换 user_data（简化处理）
    let user_data_count = context.user_data.len();
    
    let c_context = HushRequestContext {
        method: method_ptr,
        path: path_ptr,
        body: body_ptr,
        body_length: context.body.len(),
        headers_count,
        headers_keys: std::ptr::null(),
        headers_values: std::ptr::null(),
        user_data_count,
        user_data_keys: std::ptr::null(),
        user_data_values: std::ptr::null(),
    };
    
    Ok((c_context, c_strings))
}

/// 从 C 兼容的结构更新 Rust RequestContext
fn update_request_context_from_c(_context: &mut RequestContext, _c_context: &HushRequestContext) -> Result<(), HushError> {
    // 这里可以从 C 结构中读取修改后的数据并更新 Rust 结构
    // 为了简化，我们暂时跳过这个实现
    Ok(())
}

// ============================================================================
// FFI 导出函数 | FFI Exported Functions
// ============================================================================

/// 创建新的中间件链
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_new() -> *mut HushMiddleware {
    let middleware = Box::new(HushMiddleware::new());
    Box::into_raw(middleware)
}

/// 添加自定义中间件处理函数
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_add(
    middleware: *mut HushMiddleware,
    handler: HushMiddlewareHandler,
    user_data: *mut c_char,
) {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        if let Ok(mut chain) = middleware_ref.chain.lock() {
            // 创建中间件名称
            let middleware_name = format!("custom_middleware_{}", chain.len());
            
            // 将 user_data 转换为安全的字符串（如果不为空）
            let user_data_string = if user_data.is_null() {
                None
            } else {
                match from_c_string(user_data) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                }
            };
            
            // 添加函数式中间件
            chain.add_function(middleware_name, move |ctx, next| {
                // 将 Rust 上下文转换为 C 结构
                match request_context_to_c(&ctx.request) {
                    Ok((mut c_context, _c_strings)) => {
                        // 准备 user_data 参数
                        let user_data_ptr = if let Some(ref data) = user_data_string {
                            match to_c_string(data) {
                                Ok(c_str) => c_str.into_raw(),
                                Err(_) => std::ptr::null_mut(),
                            }
                        } else {
                            std::ptr::null_mut()
                        };
                        
                        // 调用 C 处理函数
                        let result = handler(&mut c_context, user_data_ptr);
                        
                        // 清理 user_data_ptr
                        if !user_data_ptr.is_null() {
                            let _ = std::ffi::CString::from_raw(user_data_ptr);
                        }
                        
                        // 根据返回值决定下一步操作
                        match result {
                            0 => {
                                // 继续执行下一个中间件
                                if let Err(e) = update_request_context_from_c(&mut ctx.request, &c_context) {
                                    return Ok(MiddlewareResult::Error(e));
                                }
                                next(ctx)
                            }
                            1 => {
                                // 提前返回成功响应
                                Ok(MiddlewareResult::Response(
                                    ResponseContext::with_text(HttpStatus::Ok, "Middleware response")
                                ))
                            }
                            _ => {
                                // 返回错误
                                Ok(MiddlewareResult::Error(
                                    HushError::InternalError("Middleware handler failed".to_string())
                                ))
                            }
                        }
                    }
                    Err(e) => Ok(MiddlewareResult::Error(e))
                }
            });
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
        }
    }
}

/// 添加 CORS 中间件
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_add_cors(
    middleware: *mut HushMiddleware,
    allowed_origins: *const c_char,
) {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        let origins = if allowed_origins.is_null() {
            "*".to_string()
        } else {
            match from_c_string(allowed_origins) {
                Ok(s) => s,
                Err(e) => {
                    set_last_error(e);
                    return;
                }
            }
        };
        
        if let Ok(mut chain) = middleware_ref.chain.lock() {
            let cors_middleware = CorsMiddleware::new(origins);
            chain.add(cors_middleware);
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
        }
    }
}

/// 添加 JWT 认证中间件
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_add_auth_jwt(
    middleware: *mut HushMiddleware,
    secret: *const c_char,
) {
    if middleware.is_null() || secret.is_null() {
        set_last_error(HushError::NullPointer);
        return;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        let secret_str = match from_c_string(secret) {
            Ok(s) => s,
            Err(e) => {
                set_last_error(e);
                return;
            }
        };
        
        if let Ok(mut chain) = middleware_ref.chain.lock() {
            let auth_middleware = AuthMiddleware::new(secret_str);
            chain.add(auth_middleware);
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
        }
    }
}

/// 添加日志中间件
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_add_logger(middleware: *mut HushMiddleware) {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        if let Ok(mut chain) = middleware_ref.chain.lock() {
            let logger_middleware = LoggerMiddleware::new();
            chain.add(logger_middleware);
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
        }
    }
}

/// 添加基于IP的请求限流中间件
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_add_rate_limit(
    middleware: *mut HushMiddleware,
    max_requests: u32,
    window_seconds: u64,
) {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        if let Ok(mut chain) = middleware_ref.chain.lock() {
            let rate_limit_middleware = RateLimitMiddleware::new(max_requests, window_seconds);
            chain.add(rate_limit_middleware);
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
        }
    }
}

/// 添加基于用户ID的请求限流中间件
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_add_rate_limit_by_user(middleware: *mut HushMiddleware) {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        if let Ok(mut chain) = middleware_ref.chain.lock() {
            let rate_limit_middleware = RateLimitMiddleware::by_user_id();
            chain.add(rate_limit_middleware);
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
        }
    }
}

/// 执行中间件链
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_execute(
    middleware: *mut HushMiddleware,
    method: *const c_char,
    path: *const c_char,
    body: *const c_char,
) -> *const c_char {
    if middleware.is_null() || method.is_null() || path.is_null() {
        set_last_error(HushError::NullPointer);
        return std::ptr::null();
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        // 解析参数
        let method_str = match from_c_string(method) {
            Ok(s) => s,
            Err(e) => {
                set_last_error(e);
                return std::ptr::null();
            }
        };
        
        let path_str = match from_c_string(path) {
            Ok(s) => s,
            Err(e) => {
                set_last_error(e);
                return std::ptr::null();
            }
        };
        
        let body_str = if body.is_null() {
            String::new()
        } else {
            match from_c_string(body) {
                Ok(s) => s,
                Err(e) => {
                    set_last_error(e);
                    return std::ptr::null();
                }
            }
        };
        
        // 创建请求上下文
        let http_method = match HttpMethod::from_str(&method_str) {
            Ok(m) => m,
            Err(e) => {
                set_last_error(e);
                return std::ptr::null();
            }
        };
        
        let mut request = RequestContext::new(http_method, path_str);
        request.set_body(body_str.into_bytes());
        
        let context = MiddlewareContext::new(request);
        
        // 执行中间件链
        if let Ok(chain) = middleware_ref.chain.lock() {
            match chain.execute(context) {
                Ok(response) => {
                    match response.body_as_string() {
                        Ok(response_str) => {
                            match to_c_string(&response_str) {
                                Ok(c_str) => c_str.into_raw(),
                                Err(e) => {
                                    set_last_error(e);
                                    std::ptr::null()
                                }
                            }
                        }
                        Err(e) => {
                            set_last_error(e);
                            std::ptr::null()
                        }
                    }
                }
                Err(e) => {
                    set_last_error(e);
                    std::ptr::null()
                }
            }
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
            std::ptr::null()
        }
    }
}

/// 释放中间件链资源
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_free(middleware: *mut HushMiddleware) {
    if !middleware.is_null() {
        unsafe {
            let _ = Box::from_raw(middleware);
        }
    }
}

/// 获取中间件链中的中间件数量
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_count(middleware: *mut HushMiddleware) -> usize {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return 0;
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        if let Ok(chain) = middleware_ref.chain.lock() {
            chain.len()
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
            0
        }
    }
}

/// 获取中间件名称列表（返回 JSON 格式的字符串）
#[unsafe(no_mangle)]
pub extern "C" fn hush_middleware_names(middleware: *mut HushMiddleware) -> *const c_char {
    if middleware.is_null() {
        set_last_error(HushError::NullPointer);
        return std::ptr::null();
    }
    
    unsafe {
        let middleware_ref = &*middleware;
        
        if let Ok(chain) = middleware_ref.chain.lock() {
            let names = chain.middleware_names();
            let json_str = format!("{:?}", names); // 简化的 JSON 格式
            
            match to_c_string(&json_str) {
                Ok(c_str) => c_str.into_raw(),
                Err(e) => {
                    set_last_error(e);
                    std::ptr::null()
                }
            }
        } else {
            set_last_error(HushError::InternalError("Failed to lock middleware chain".to_string()));
            std::ptr::null()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_middleware_ffi_basic() {
        // 创建中间件链
        let middleware = hush_middleware_new();
        assert!(!middleware.is_null());
        
        // 检查初始状态
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 0);
        
        // 添加日志中间件
        hush_middleware_add_logger(middleware);
        
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 1);
        
        // 清理资源
        hush_middleware_free(middleware);
    }
    
    #[test]
    fn test_cors_middleware_ffi() {
        let middleware = hush_middleware_new();
        assert!(!middleware.is_null());
        
        let origins = CString::new("https://example.com").unwrap();
        hush_middleware_add_cors(middleware, origins.as_ptr());
        
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 1);
        
        hush_middleware_free(middleware);
    }
    
    #[test]
    fn test_auth_middleware_ffi() {
        let middleware = hush_middleware_new();
        assert!(!middleware.is_null());
        
        let secret = CString::new("my_secret_key").unwrap();
        hush_middleware_add_auth_jwt(middleware, secret.as_ptr());
        
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 1);
        
        hush_middleware_free(middleware);
    }
    
    #[test]
    fn test_rate_limit_middleware_ffi() {
        let middleware = hush_middleware_new();
        assert!(!middleware.is_null());
        
        // 添加基于IP的限流中间件
        hush_middleware_add_rate_limit(middleware, 10, 60);
        
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 1);
        
        hush_middleware_free(middleware);
    }
    
    #[test]
    fn test_rate_limit_by_user_middleware_ffi() {
        let middleware = hush_middleware_new();
        assert!(!middleware.is_null());
        
        // 添加基于用户ID的限流中间件
        hush_middleware_add_rate_limit_by_user(middleware);
        
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 1);
        
        hush_middleware_free(middleware);
    }
    
    #[test]
    fn test_multiple_middleware_ffi() {
        let middleware = hush_middleware_new();
        assert!(!middleware.is_null());
        
        // 添加多个中间件
        hush_middleware_add_logger(middleware);
        
        let origins = CString::new("*").unwrap();
        hush_middleware_add_cors(middleware, origins.as_ptr());
        
        hush_middleware_add_rate_limit(middleware, 100, 3600);
        
        let secret = CString::new("test_secret").unwrap();
        hush_middleware_add_auth_jwt(middleware, secret.as_ptr());
        
        let count = hush_middleware_count(middleware);
        assert_eq!(count, 4);
        
        // 获取中间件名称列表
        let names_ptr = hush_middleware_names(middleware);
        assert!(!names_ptr.is_null());
        
        // 清理资源
        if !names_ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(names_ptr as *mut c_char);
            }
        }
        
        hush_middleware_free(middleware);
    }
}