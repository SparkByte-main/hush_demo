// ============================================================================
// 核心数据类型定义 | Core Data Type Definitions
// ============================================================================

use std::collections::HashMap;
use std::ffi::{c_char, CString};
use std::time::SystemTime;
use super::error::{HushError, HushResult};
use super::memory::get_memory_manager;

/// HTTP 方法枚举
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> HushResult<Self> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(HushError::InvalidInput(format!("Unknown HTTP method: {}", method))),
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// HTTP 状态码
#[derive(Debug, Clone, Copy)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
    NotImplemented = 501,
    ServiceUnavailable = 503,
}

impl HttpStatus {
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
    
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::NoContent => "No Content",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::NotImplemented => "Not Implemented",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
        }
    }
}

/// 请求上下文，包含所有请求相关信息
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: HttpMethod,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub user_data: HashMap<String, String>,
    pub start_time: SystemTime,
    pub trace_id: String,
}

impl RequestContext {
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self {
            method,
            path,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            user_data: HashMap::new(),
            start_time: SystemTime::now(),
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }
    
    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
    
    pub fn add_query_param(&mut self, key: String, value: String) {
        self.query_params.insert(key, value);
    }
    
    pub fn set_user_data(&mut self, key: String, value: String) {
        self.user_data.insert(key, value);
    }
    
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn get_user_data(&self, key: &str) -> Option<&String> {
        self.user_data.get(key)
    }
    
    pub fn body_as_string(&self) -> HushResult<String> {
        String::from_utf8(self.body.clone())
            .map_err(|_| HushError::InvalidInput("Invalid UTF-8 in request body".to_string()))
    }
}

/// 响应上下文，包含所有响应相关信息
#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl ResponseContext {
    pub fn new(status: HttpStatus) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
    
    pub fn with_body(status: HttpStatus, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body,
        }
    }
    
    pub fn with_text(status: HttpStatus, text: &str) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: text.as_bytes().to_vec(),
        }
    }
    
    pub fn with_json(status: HttpStatus, json: &str) -> Self {
        let mut response = Self::with_text(status, json);
        response.add_header("Content-Type".to_string(), "application/json".to_string());
        response
    }
    
    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
    
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }
    
    pub fn body_as_string(&self) -> HushResult<String> {
        String::from_utf8(self.body.clone())
            .map_err(|_| HushError::InvalidInput("Invalid UTF-8 in response body".to_string()))
    }
}

/// 路由信息
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub method: HttpMethod,
    pub path: String,
    pub handler_name: String,
}

impl RouteInfo {
    pub fn new(method: HttpMethod, path: String, handler_name: String) -> Self {
        Self {
            method,
            path,
            handler_name,
        }
    }
    
    pub fn route_key(&self) -> String {
        format!("{}:{}", self.method.as_str(), self.path)
    }
}

// ============================================================================
// C 兼容的数据结构 | C-Compatible Data Structures
// ============================================================================

/// C 兼容的请求上下文
#[repr(C)]
pub struct CRequestContext {
    pub method: *const c_char,
    pub path: *const c_char,
    pub body: *const c_char,
    pub body_length: usize,
    pub headers_count: usize,
    pub headers_keys: *const *const c_char,
    pub headers_values: *const *const c_char,
}

/// C 兼容的响应上下文
#[repr(C)]
pub struct CResponseContext {
    pub status_code: u16,
    pub body: *const c_char,
    pub body_length: usize,
    pub headers_count: usize,
    pub headers_keys: *const *const c_char,
    pub headers_values: *const *const c_char,
}

impl RequestContext {
    /// 转换为 C 兼容的结构体
    pub fn to_c_context(&self) -> HushResult<CRequestContext> {
        let manager = get_memory_manager();
        
        let method_ptr = manager.allocate_c_string(self.method.as_str())?;
        let path_ptr = manager.allocate_c_string(&self.path)?;
        let body_str = self.body_as_string().unwrap_or_default();
        let body_ptr = manager.allocate_c_string(&body_str)?;
        
        // TODO: 实现 headers 的转换
        // 这里需要分配 headers 数组，暂时简化处理
        
        Ok(CRequestContext {
            method: method_ptr,
            path: path_ptr,
            body: body_ptr,
            body_length: self.body.len(),
            headers_count: 0,
            headers_keys: std::ptr::null(),
            headers_values: std::ptr::null(),
        })
    }
}

impl ResponseContext {
    /// 从 C 兼容的结构体创建
    pub fn from_c_context(c_ctx: &CResponseContext) -> HushResult<Self> {
        let status = match c_ctx.status_code {
            200 => HttpStatus::Ok,
            201 => HttpStatus::Created,
            204 => HttpStatus::NoContent,
            400 => HttpStatus::BadRequest,
            401 => HttpStatus::Unauthorized,
            403 => HttpStatus::Forbidden,
            404 => HttpStatus::NotFound,
            405 => HttpStatus::MethodNotAllowed,
            500 => HttpStatus::InternalServerError,
            501 => HttpStatus::NotImplemented,
            503 => HttpStatus::ServiceUnavailable,
            _ => HttpStatus::InternalServerError,
        };
        
        let body = if c_ctx.body.is_null() {
            Vec::new()
        } else {
            unsafe {
                let body_str = std::ffi::CStr::from_ptr(c_ctx.body)
                    .to_str()
                    .map_err(|_| HushError::FFIError("Invalid UTF-8 in response body".to_string()))?;
                body_str.as_bytes().to_vec()
            }
        };
        
        Ok(Self {
            status,
            headers: HashMap::new(), // TODO: 实现 headers 的转换
            body,
        })
    }
}