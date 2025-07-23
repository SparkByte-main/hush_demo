// ============================================================================
// 内存管理和类型转换工具 | Memory Management and Type Conversion Tools
// ============================================================================

use std::ffi::{c_char, CString};
use std::sync::Arc;
use super::error::{HushError, HushResult};

/// 内存管理器，负责跨 FFI 边界的内存安全
pub struct MemoryManager {
    // 可以在这里添加内存池、统计信息等
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {}
    }
    
    /// 安全地分配 C 字符串
    pub fn allocate_c_string(&self, s: &str) -> HushResult<*mut c_char> {
        let c_string = CString::new(s)
            .map_err(|_| HushError::FFIError("Failed to create C string".to_string()))?;
        Ok(c_string.into_raw())
    }
    
    /// 安全地释放 C 字符串
    pub fn free_c_string(&self, ptr: *mut c_char) {
        if !ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(ptr);
            }
        }
    }
    
    /// 复制 C 字符串
    pub fn clone_c_string(&self, ptr: *const c_char) -> HushResult<*mut c_char> {
        if ptr.is_null() {
            return Err(HushError::NullPointer);
        }
        
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(ptr);
            let rust_str = c_str.to_str()
                .map_err(|_| HushError::FFIError("Invalid UTF-8 in C string".to_string()))?;
            self.allocate_c_string(rust_str)
        }
    }
}

/// C 字符串包装器，提供 RAII 内存管理
pub struct CStringWrapper {
    ptr: *mut c_char,
    manager: Arc<MemoryManager>,
}

impl CStringWrapper {
    pub fn new(s: &str, manager: Arc<MemoryManager>) -> HushResult<Self> {
        let ptr = manager.allocate_c_string(s)?;
        Ok(Self { ptr, manager })
    }
    
    pub fn as_ptr(&self) -> *const c_char {
        self.ptr
    }
    
    pub fn into_raw(self) -> *mut c_char {
        let ptr = self.ptr;
        std::mem::forget(self); // 防止 Drop 被调用
        ptr
    }
}

impl Drop for CStringWrapper {
    fn drop(&mut self) {
        self.manager.free_c_string(self.ptr);
    }
}

/// 全局内存管理器实例
static GLOBAL_MEMORY_MANAGER: std::sync::OnceLock<Arc<MemoryManager>> = std::sync::OnceLock::new();

/// 获取全局内存管理器
pub fn get_memory_manager() -> Arc<MemoryManager> {
    GLOBAL_MEMORY_MANAGER
        .get_or_init(|| Arc::new(MemoryManager::new()))
        .clone()
}

// ============================================================================
// 类型转换工具 | Type Conversion Tools
// ============================================================================

/// 将 Rust Vec<u8> 转换为 C 兼容的字节数组
pub struct ByteArray {
    data: *mut u8,
    length: usize,
    capacity: usize,
}

impl ByteArray {
    pub fn from_vec(mut vec: Vec<u8>) -> Self {
        let data = vec.as_mut_ptr();
        let length = vec.len();
        let capacity = vec.capacity();
        std::mem::forget(vec); // 防止 Vec 被释放
        
        Self { data, length, capacity }
    }
    
    pub fn as_ptr(&self) -> *const u8 {
        self.data
    }
    
    pub fn len(&self) -> usize {
        self.length
    }
    
    pub fn into_vec(self) -> Vec<u8> {
        unsafe {
            Vec::from_raw_parts(self.data, self.length, self.capacity)
        }
    }
}

impl Drop for ByteArray {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                let _ = Vec::from_raw_parts(self.data, self.length, self.capacity);
            }
        }
    }
}

// ============================================================================
// FFI 内存管理接口 | FFI Memory Management Interface
// ============================================================================

/// 分配指定大小的内存块
#[unsafe(no_mangle)]
pub extern "C" fn hush_malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    
    let layout = std::alloc::Layout::from_size_align(size, 1);
    match layout {
        Ok(layout) => unsafe { std::alloc::alloc(layout) },
        Err(_) => std::ptr::null_mut(),
    }
}

/// 释放内存块
#[unsafe(no_mangle)]
pub extern "C" fn hush_free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 {
        let layout = std::alloc::Layout::from_size_align(size, 1);
        if let Ok(layout) = layout {
            unsafe {
                std::alloc::dealloc(ptr, layout);
            }
        }
    }
}

/// 重新分配内存块
#[unsafe(no_mangle)]
pub extern "C" fn hush_realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if new_size == 0 {
        hush_free(ptr, old_size);
        return std::ptr::null_mut();
    }
    
    if ptr.is_null() {
        return hush_malloc(new_size);
    }
    
    let old_layout = std::alloc::Layout::from_size_align(old_size, 1);
    let new_layout = std::alloc::Layout::from_size_align(new_size, 1);
    
    match (old_layout, new_layout) {
        (Ok(old_layout), Ok(new_layout)) => unsafe {
            std::alloc::realloc(ptr, old_layout, new_size)
        },
        _ => std::ptr::null_mut(),
    }
}

/// 复制内存块
#[unsafe(no_mangle)]
pub extern "C" fn hush_memcpy(dest: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    if dest.is_null() || src.is_null() || size == 0 {
        return std::ptr::null_mut();
    }
    
    unsafe {
        std::ptr::copy_nonoverlapping(src, dest, size);
    }
    dest
}

/// 设置内存块
#[unsafe(no_mangle)]
pub extern "C" fn hush_memset(ptr: *mut u8, value: i32, size: usize) -> *mut u8 {
    if ptr.is_null() || size == 0 {
        return std::ptr::null_mut();
    }
    
    unsafe {
        std::ptr::write_bytes(ptr, value as u8, size);
    }
    ptr
}