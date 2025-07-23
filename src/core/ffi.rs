// ============================================================================
// FFI 接口规范和工具函数 | FFI Interface Specification and Utility Functions
// ============================================================================

use std::ffi::{c_char, CStr, CString};
use super::error::{HushError, HushResult, set_last_error, ErrorCode};

/// FFI 操作结果类型
pub type FFIResult<T> = Result<T, ErrorCode>;

/// 将 Rust 字符串转换为 C 字符串
pub fn to_c_string(s: &str) -> HushResult<CString> {
    CString::new(s).map_err(|_| HushError::FFIError("Failed to create C string".to_string()))
}

/// 从 C 字符串指针安全地创建 Rust 字符串
pub fn from_c_string(ptr: *const c_char) -> HushResult<String> {
    if ptr.is_null() {
        return Err(HushError::NullPointer);
    }
    
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| HushError::FFIError("Invalid UTF-8 in C string".to_string()))
    }
}

/// 处理 FFI 结果，设置错误状态并返回错误码
pub fn handle_ffi_result<T>(result: HushResult<T>) -> FFIResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let error_code = ErrorCode::from(error.clone());
            set_last_error(error);
            Err(error_code)
        }
    }
}

/// 安全地检查指针是否为空
pub fn check_null_ptr<T>(ptr: *const T, name: &str) -> HushResult<()> {
    if ptr.is_null() {
        Err(HushError::InvalidParameter)
    } else {
        Ok(())
    }
}

/// 安全地检查可变指针是否为空
pub fn check_null_mut_ptr<T>(ptr: *mut T, name: &str) -> HushResult<()> {
    if ptr.is_null() {
        Err(HushError::InvalidParameter)
    } else {
        Ok(())
    }
}

// ============================================================================
// 通用 FFI 工具函数 | Generic FFI Utility Functions
// ============================================================================

/// 释放 Rust 分配的字符串内存
#[unsafe(no_mangle)]
pub extern "C" fn hush_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// 复制字符串到 C 堆内存
#[unsafe(no_mangle)]
pub extern "C" fn hush_string_clone(ptr: *const c_char) -> *mut c_char {
    if ptr.is_null() {
        set_last_error(HushError::NullPointer);
        return std::ptr::null_mut();
    }
    
    match from_c_string(ptr) {
        Ok(s) => {
            match to_c_string(&s) {
                Ok(c_string) => c_string.into_raw(),
                Err(error) => {
                    set_last_error(error);
                    std::ptr::null_mut()
                }
            }
        }
        Err(error) => {
            set_last_error(error);
            std::ptr::null_mut()
        }
    }
}

/// 获取字符串长度
#[unsafe(no_mangle)]
pub extern "C" fn hush_string_length(ptr: *const c_char) -> i32 {
    if ptr.is_null() {
        set_last_error(HushError::NullPointer);
        return -1;
    }
    
    unsafe {
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => s.len() as i32,
            Err(_) => {
                set_last_error(HushError::FFIError("Invalid UTF-8 string".to_string()));
                -1
            }
        }
    }
}

/// 比较两个 C 字符串
#[unsafe(no_mangle)]
pub extern "C" fn hush_string_compare(ptr1: *const c_char, ptr2: *const c_char) -> i32 {
    if ptr1.is_null() || ptr2.is_null() {
        set_last_error(HushError::NullPointer);
        return -2; // 特殊错误码表示空指针
    }
    
    match (from_c_string(ptr1), from_c_string(ptr2)) {
        (Ok(s1), Ok(s2)) => {
            if s1 == s2 { 0 } else if s1 < s2 { -1 } else { 1 }
        }
        _ => {
            set_last_error(HushError::FFIError("String comparison failed".to_string()));
            -2
        }
    }
}

// ============================================================================
// FFI 宏定义 | FFI Macro Definitions
// ============================================================================

/// 用于简化 FFI 函数错误处理的宏
#[macro_export]
macro_rules! ffi_try {
    ($expr:expr) => {
        match $crate::core::ffi::handle_ffi_result($expr) {
            Ok(val) => val,
            Err(_) => return std::ptr::null_mut(),
        }
    };
}

/// 用于检查 FFI 函数参数的宏
#[macro_export]
macro_rules! ffi_check_ptr {
    ($ptr:expr, $name:expr) => {
        if let Err(error) = $crate::core::ffi::check_null_ptr($ptr, $name) {
            $crate::core::error::set_last_error(error);
            return std::ptr::null_mut();
        }
    };
}

/// 用于检查可变 FFI 函数参数的宏
#[macro_export]
macro_rules! ffi_check_mut_ptr {
    ($ptr:expr, $name:expr) => {
        if let Err(error) = $crate::core::ffi::check_null_mut_ptr($ptr, $name) {
            $crate::core::error::set_last_error(error);
            return std::ptr::null_mut();
        }
    };
}