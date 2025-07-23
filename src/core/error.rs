// ============================================================================
// 统一错误处理机制 | Unified Error Handling Mechanism
// ============================================================================

use std::ffi::{c_char, CString};
use std::sync::Mutex;

/// 统一的错误类型定义
#[derive(Debug, Clone)]
pub enum HushError {
    // HTTP 相关错误
    HttpError(String),
    RouteNotFound,
    MethodNotAllowed,
    
    // 数据库相关错误
    DatabaseError(String),
    ConnectionFailed,
    QueryFailed(String),
    TransactionFailed,
    
    // 认证相关错误
    AuthenticationFailed,
    AuthorizationFailed,
    InvalidToken,
    TokenExpired,
    
    // 配置相关错误
    ConfigError(String),
    ConfigNotFound(String),
    ConfigParseError(String),
    
    // 文件相关错误
    FileError(String),
    FileNotFound,
    PermissionDenied,
    
    // 验证相关错误
    ValidationError(String),
    InvalidInput(String),
    
    // 系统相关错误
    InternalError(String),
    OutOfMemory,
    Timeout,
    
    // FFI 相关错误
    FFIError(String),
    NullPointer,
    InvalidParameter,
}

/// 结果类型别名
pub type HushResult<T> = Result<T, HushError>;

/// C 兼容的错误码
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Success = 0,
    HttpError = 1000,
    DatabaseError = 2000,
    AuthError = 3000,
    ConfigError = 4000,
    FileError = 5000,
    ValidationError = 6000,
    InternalError = 9000,
}

impl From<HushError> for ErrorCode {
    fn from(error: HushError) -> Self {
        match error {
            HushError::HttpError(_) | HushError::RouteNotFound | HushError::MethodNotAllowed => ErrorCode::HttpError,
            HushError::DatabaseError(_) | HushError::ConnectionFailed | HushError::QueryFailed(_) | HushError::TransactionFailed => ErrorCode::DatabaseError,
            HushError::AuthenticationFailed | HushError::AuthorizationFailed | HushError::InvalidToken | HushError::TokenExpired => ErrorCode::AuthError,
            HushError::ConfigError(_) | HushError::ConfigNotFound(_) | HushError::ConfigParseError(_) => ErrorCode::ConfigError,
            HushError::FileError(_) | HushError::FileNotFound | HushError::PermissionDenied => ErrorCode::FileError,
            HushError::ValidationError(_) | HushError::InvalidInput(_) => ErrorCode::ValidationError,
            _ => ErrorCode::InternalError,
        }
    }
}

impl std::fmt::Display for HushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HushError::HttpError(msg) => write!(f, "HTTP Error: {}", msg),
            HushError::RouteNotFound => write!(f, "Route not found"),
            HushError::MethodNotAllowed => write!(f, "Method not allowed"),
            HushError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            HushError::ConnectionFailed => write!(f, "Database connection failed"),
            HushError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            HushError::TransactionFailed => write!(f, "Transaction failed"),
            HushError::AuthenticationFailed => write!(f, "Authentication failed"),
            HushError::AuthorizationFailed => write!(f, "Authorization failed"),
            HushError::InvalidToken => write!(f, "Invalid token"),
            HushError::TokenExpired => write!(f, "Token expired"),
            HushError::ConfigError(msg) => write!(f, "Configuration Error: {}", msg),
            HushError::ConfigNotFound(msg) => write!(f, "Configuration not found: {}", msg),
            HushError::ConfigParseError(msg) => write!(f, "Configuration parse error: {}", msg),
            HushError::FileError(msg) => write!(f, "File Error: {}", msg),
            HushError::FileNotFound => write!(f, "File not found"),
            HushError::PermissionDenied => write!(f, "Permission denied"),
            HushError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            HushError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            HushError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            HushError::OutOfMemory => write!(f, "Out of memory"),
            HushError::Timeout => write!(f, "Operation timeout"),
            HushError::FFIError(msg) => write!(f, "FFI Error: {}", msg),
            HushError::NullPointer => write!(f, "Null pointer"),
            HushError::InvalidParameter => write!(f, "Invalid parameter"),
        }
    }
}

impl std::error::Error for HushError {}

/// 全局错误状态管理
static LAST_ERROR: Mutex<Option<HushError>> = Mutex::new(None);

/// 设置最后的错误
pub fn set_last_error(error: HushError) {
    if let Ok(mut last_error) = LAST_ERROR.lock() {
        *last_error = Some(error);
    }
}

/// 获取最后的错误
pub fn get_last_error() -> Option<HushError> {
    if let Ok(last_error) = LAST_ERROR.lock() {
        last_error.clone()
    } else {
        None
    }
}

/// 清除最后的错误
pub fn clear_last_error() {
    if let Ok(mut last_error) = LAST_ERROR.lock() {
        *last_error = None;
    }
}

// ============================================================================
// FFI 错误处理接口 | FFI Error Handling Interface
// ============================================================================

/// 获取最后错误的 C 字符串表示
#[unsafe(no_mangle)]
pub extern "C" fn hush_get_last_error() -> *const c_char {
    if let Some(error) = get_last_error() {
        if let Ok(c_string) = CString::new(error.to_string()) {
            return c_string.into_raw();
        }
    }
    std::ptr::null()
}

/// 获取最后错误的错误码
#[unsafe(no_mangle)]
pub extern "C" fn hush_get_last_error_code() -> ErrorCode {
    if let Some(error) = get_last_error() {
        ErrorCode::from(error)
    } else {
        ErrorCode::Success
    }
}

/// 清除最后的错误
#[unsafe(no_mangle)]
pub extern "C" fn hush_clear_error() {
    clear_last_error();
}

/// 释放错误字符串内存
#[unsafe(no_mangle)]
pub extern "C" fn hush_free_error_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}