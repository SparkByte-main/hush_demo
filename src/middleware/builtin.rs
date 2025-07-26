// ============================================================================
// 内置中间件实现 | Built-in Middleware Implementations
// ============================================================================

use std::time::SystemTime;
use crate::core::error::HushResult;
use crate::core::types::{ResponseContext, HttpStatus};
use super::core::{Middleware, MiddlewareContext, MiddlewareResult, NextFunction};

/// CORS 中间件
pub struct CorsMiddleware {
    allowed_origins: String,
    allowed_methods: String,
    allowed_headers: String,
    max_age: u32,
}

impl CorsMiddleware {
    pub fn new(allowed_origins: String) -> Self {
        Self {
            allowed_origins,
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allowed_headers: "Content-Type, Authorization, X-Requested-With".to_string(),
            max_age: 86400, // 24 hours
        }
    }
    
    pub fn permissive() -> Self {
        Self {
            allowed_origins: "*".to_string(),
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, PATCH, HEAD".to_string(),
            allowed_headers: "*".to_string(),
            max_age: 86400,
        }
    }
    
    pub fn with_methods(mut self, methods: String) -> Self {
        self.allowed_methods = methods;
        self
    }
    
    pub fn with_headers(mut self, headers: String) -> Self {
        self.allowed_headers = headers;
        self
    }
    
    pub fn with_max_age(mut self, max_age: u32) -> Self {
        self.max_age = max_age;
        self
    }
    
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.allowed_origins == "*" {
            return true;
        }
        
        self.allowed_origins
            .split(',')
            .map(|s| s.trim())
            .any(|allowed| allowed == origin)
    }
}

impl Middleware for CorsMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        // 获取请求的 Origin 头
        let request_origin = context.request.get_header("Origin").cloned().unwrap_or_else(|| "".to_string());
        
        // 检查 Origin 是否被允许
        let allowed_origin = if self.allowed_origins == "*" {
            "*".to_string()
        } else if self.is_origin_allowed(&request_origin) {
            request_origin.clone()
        } else {
            // Origin 不被允许，返回错误
            let response = ResponseContext::with_json(
                HttpStatus::Forbidden,
                r#"{"error": "CORS: Origin not allowed"}"#
            );
            return Ok(MiddlewareResult::Response(response));
        };
        
        // 检查是否是 OPTIONS 预检请求
        if context.request.method.as_str() == "OPTIONS" {
            let mut response = ResponseContext::new(HttpStatus::NoContent);
            response.add_header("Access-Control-Allow-Origin".to_string(), allowed_origin);
            response.add_header("Access-Control-Allow-Methods".to_string(), self.allowed_methods.clone());
            response.add_header("Access-Control-Allow-Headers".to_string(), self.allowed_headers.clone());
            response.add_header("Access-Control-Max-Age".to_string(), self.max_age.to_string());
            response.add_header("Access-Control-Allow-Credentials".to_string(), "true".to_string());
            
            return Ok(MiddlewareResult::Response(response));
        }
        
        // 对于其他请求，继续执行并添加 CORS 头
        match next(context)? {
            MiddlewareResult::Response(mut response) => {
                response.add_header("Access-Control-Allow-Origin".to_string(), allowed_origin);
                response.add_header("Access-Control-Allow-Credentials".to_string(), "true".to_string());
                response.add_header("Access-Control-Expose-Headers".to_string(), 
                    "Content-Length, Content-Type, Date, Server".to_string());
                Ok(MiddlewareResult::Response(response))
            }
            other => Ok(other),
        }
    }
    
    fn name(&self) -> &str {
        "cors"
    }
    
    fn priority(&self) -> i32 {
        10 // 高优先级，应该早执行
    }
}

/// 日志中间件
pub struct LoggerMiddleware {
    log_requests: bool,
    log_responses: bool,
    log_headers: bool,
    log_body: bool,
}

impl LoggerMiddleware {
    pub fn new() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_headers: false,
            log_body: false,
        }
    }
    
    pub fn requests_only() -> Self {
        Self {
            log_requests: true,
            log_responses: false,
            log_headers: false,
            log_body: false,
        }
    }
    
    pub fn responses_only() -> Self {
        Self {
            log_requests: false,
            log_responses: true,
            log_headers: false,
            log_body: false,
        }
    }
    
    pub fn detailed() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_headers: true,
            log_body: true,
        }
    }
    
    pub fn with_headers(mut self) -> Self {
        self.log_headers = true;
        self
    }
    
    pub fn with_body(mut self) -> Self {
        self.log_body = true;
        self
    }
    
    fn log_request_details(&self, context: &MiddlewareContext) {
        if self.log_headers && !context.request.headers.is_empty() {
            println!("  Headers:");
            for (key, value) in &context.request.headers {
                println!("    {}: {}", key, value);
            }
        }
        
        if self.log_body && !context.request.body.is_empty() {
            let body_str = context.request.body_as_string().unwrap_or_default();
            if !body_str.is_empty() {
                println!("  Body: {}", 
                    if body_str.len() > 200 { 
                        format!("{}...", &body_str[..200]) 
                    } else { 
                        body_str 
                    }
                );
            }
        }
    }
    
    fn log_response_details(&self, response: &ResponseContext) {
        if self.log_headers && !response.headers.is_empty() {
            println!("  Response Headers:");
            for (key, value) in &response.headers {
                println!("    {}: {}", key, value);
            }
        }
        
        if self.log_body {
            let body_str = response.body_as_string().unwrap_or_default();
            if !body_str.is_empty() {
                println!("  Response Body: {}", 
                    if body_str.len() > 200 { 
                        format!("{}...", &body_str[..200]) 
                    } else { 
                        body_str 
                    }
                );
            }
        }
    }
}

impl Default for LoggerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for LoggerMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        let start_time = SystemTime::now();
        
        if self.log_requests {
            println!("[{}] {} {} - Request started", 
                format_time(start_time),
                context.request.method.as_str(),
                context.request.path
            );
            self.log_request_details(context);
        }
        
        // 执行下一个中间件
        let result = next(context)?;
        
        if self.log_responses {
            let duration = start_time.elapsed().unwrap_or_default();
            match &result {
                MiddlewareResult::Response(response) => {
                    println!("[{}] {} {} - {} {} ({:.2}ms)", 
                        format_time(SystemTime::now()),
                        context.request.method.as_str(),
                        context.request.path,
                        response.status.as_u16(),
                        response.status.reason_phrase(),
                        duration.as_millis() as f64
                    );
                    self.log_response_details(response);
                }
                MiddlewareResult::Error(error) => {
                    println!("[{}] {} {} - Error: {} ({:.2}ms)", 
                        format_time(SystemTime::now()),
                        context.request.method.as_str(),
                        context.request.path,
                        error,
                        duration.as_millis() as f64
                    );
                }
                MiddlewareResult::Continue => {
                    println!("[{}] {} {} - Continue ({:.2}ms)", 
                        format_time(SystemTime::now()),
                        context.request.method.as_str(),
                        context.request.path,
                        duration.as_millis() as f64
                    );
                }
            }
        }
        
        Ok(result)
    }
    
    fn name(&self) -> &str {
        "logger"
    }
    
    fn priority(&self) -> i32 {
        5 // 很高优先级，应该最早执行
    }
}

/// JWT 认证中间件
pub struct AuthMiddleware {
    secret: String,
    skip_paths: Vec<String>,
    header_name: String,
}

impl AuthMiddleware {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            skip_paths: vec!["/health".to_string(), "/login".to_string()],
            header_name: "Authorization".to_string(),
        }
    }
    
    pub fn with_skip_paths(mut self, paths: Vec<String>) -> Self {
        self.skip_paths = paths;
        self
    }
    
    pub fn with_header_name(mut self, header_name: String) -> Self {
        self.header_name = header_name;
        self
    }
    
    fn should_skip(&self, path: &str) -> bool {
        self.skip_paths.iter().any(|skip_path| path.starts_with(skip_path))
    }
    
    fn extract_token(&self, context: &MiddlewareContext) -> Option<String> {
        context.request.get_header(&self.header_name)
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_string())
                } else {
                    Some(header.clone())
                }
            })
    }
    
    fn validate_token(&self, token: &str) -> bool {
        // 简化的 JWT 验证逻辑
        // 实际实现应该使用 JWT 库进行完整验证
        !token.is_empty() && token.len() > 10
    }
}

impl Middleware for AuthMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        // 检查是否需要跳过认证
        if self.should_skip(&context.request.path) {
            return next(context);
        }
        
        // 提取令牌
        let token = match self.extract_token(context) {
            Some(token) => token,
            None => {
                let response = ResponseContext::with_json(
                    HttpStatus::Unauthorized,
                    r#"{"error": "Missing authorization token"}"#
                );
                return Ok(MiddlewareResult::Response(response));
            }
        };
        
        // 验证令牌
        if !self.validate_token(&token) {
            let response = ResponseContext::with_json(
                HttpStatus::Unauthorized,
                r#"{"error": "Invalid authorization token"}"#
            );
            return Ok(MiddlewareResult::Response(response));
        }
        
        // 将用户信息添加到上下文中
        context.set_data("authenticated".to_string(), "true".to_string());
        context.set_data("token".to_string(), token);
        
        // 继续执行下一个中间件
        next(context)
    }
    
    fn name(&self) -> &str {
        "auth_jwt"
    }
    
    fn priority(&self) -> i32 {
        20 // 中等优先级，在 CORS 和日志之后执行
    }
}

/// 请求限流中间件
pub struct RateLimitMiddleware {
    max_requests: u32,
    window_seconds: u64,
    limit_by_user: bool,
    // 简化实现，实际应该使用更复杂的数据结构如 Redis 或内存存储
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            limit_by_user: false,
        }
    }
    
    pub fn by_user_id() -> Self {
        Self {
            max_requests: 100, // 默认每用户100请求
            window_seconds: 3600, // 1小时窗口
            limit_by_user: true,
        }
    }
    
    pub fn with_user_limits(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            limit_by_user: true,
        }
    }
    
    fn get_rate_limit_key(&self, context: &MiddlewareContext) -> String {
        if self.limit_by_user {
            // 尝试从认证信息中获取用户ID
            if let Some(user_id) = context.get_data("user_id") {
                format!("rate_limit_user_{}", user_id)
            } else if let Some(token) = context.get_data("token") {
                format!("rate_limit_token_{}", token)
            } else {
                // 如果没有用户信息，回退到IP限流
                format!("rate_limit_ip_unknown")
            }
        } else {
            // 基于IP的限流（简化实现）
            format!("rate_limit_ip_default")
        }
    }
    
    fn check_rate_limit(&self, key: &str, context: &mut MiddlewareContext) -> bool {
        // 简化的限流检查逻辑
        // 实际实现应该使用滑动窗口或令牌桶算法
        
        // 检查是否已经被标记为限流
        if context.get_data(&format!("{}_limited", key)).is_some() {
            return false;
        }
        
        // 模拟请求计数检查
        let count_key = format!("{}_count", key);
        let current_count = context.get_data(&count_key)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        if current_count >= self.max_requests {
            // 标记为限流
            context.set_data(format!("{}_limited", key), "true".to_string());
            false
        } else {
            // 增加计数
            context.set_data(count_key, (current_count + 1).to_string());
            true
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        // 获取限流键
        let rate_limit_key = self.get_rate_limit_key(context);
        
        // 检查是否超过限流
        if !self.check_rate_limit(&rate_limit_key, context) {
            let error_message = if self.limit_by_user {
                format!(r#"{{"error": "Rate limit exceeded for user", "max_requests": {}, "window_seconds": {}}}"#, 
                    self.max_requests, self.window_seconds)
            } else {
                format!(r#"{{"error": "Rate limit exceeded", "max_requests": {}, "window_seconds": {}}}"#, 
                    self.max_requests, self.window_seconds)
            };
            
            let mut response = ResponseContext::with_json(
                HttpStatus::TooManyRequests,
                &error_message
            );
            
            // 添加限流相关的响应头
            response.add_header("X-RateLimit-Limit".to_string(), self.max_requests.to_string());
            response.add_header("X-RateLimit-Window".to_string(), self.window_seconds.to_string());
            response.add_header("Retry-After".to_string(), self.window_seconds.to_string());
            
            return Ok(MiddlewareResult::Response(response));
        }
        
        // 继续执行下一个中间件
        next(context)
    }
    
    fn name(&self) -> &str {
        "rate_limit"
    }
    
    fn priority(&self) -> i32 {
        15 // 在认证之前执行
    }
}

// 辅助函数
fn format_time(time: SystemTime) -> String {
    // 简化的时间格式化
    format!("{:?}", time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{HttpMethod, RequestContext};
    
    #[test]
    fn test_cors_middleware() {
        let middleware = CorsMiddleware::new("*".to_string());
        assert_eq!(middleware.name(), "cors");
        assert_eq!(middleware.priority(), 10);
        
        // 测试 OPTIONS 请求
        let mut request = RequestContext::new(HttpMethod::OPTIONS, "/test".to_string());
        request.add_header("Origin".to_string(), "https://example.com".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Response(response) => {
                assert_eq!(response.status.as_u16(), 204);
                assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
                assert!(response.headers.contains_key("Access-Control-Allow-Methods"));
                assert!(response.headers.contains_key("Access-Control-Allow-Headers"));
                assert!(response.headers.contains_key("Access-Control-Max-Age"));
            }
            _ => panic!("Expected response for OPTIONS request"),
        }
    }
    
    #[test]
    fn test_cors_middleware_origin_validation() {
        let middleware = CorsMiddleware::new("https://allowed.com".to_string());
        
        // 测试不允许的 Origin
        let mut request = RequestContext::new(HttpMethod::GET, "/test".to_string());
        request.add_header("Origin".to_string(), "https://notallowed.com".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Response(response) => {
                assert_eq!(response.status.as_u16(), 403);
            }
            _ => panic!("Expected forbidden response for disallowed origin"),
        }
    }
    
    #[test]
    fn test_logger_middleware() {
        let middleware = LoggerMiddleware::new();
        assert_eq!(middleware.name(), "logger");
        assert_eq!(middleware.priority(), 5);
        
        let request = RequestContext::new(HttpMethod::GET, "/test".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Continue => {
                // 日志中间件应该继续执行
            }
            _ => panic!("Logger middleware should continue"),
        }
    }
    
    #[test]
    fn test_auth_middleware() {
        let middleware = AuthMiddleware::new("secret".to_string());
        assert_eq!(middleware.name(), "auth_jwt");
        assert_eq!(middleware.priority(), 20);
        
        // 测试跳过路径
        let request = RequestContext::new(HttpMethod::GET, "/health".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Continue => {
                // 健康检查路径应该跳过认证
            }
            _ => panic!("Health check should skip auth"),
        }
        
        // 测试缺少令牌
        let request = RequestContext::new(HttpMethod::GET, "/protected".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Response(response) => {
                assert_eq!(response.status.as_u16(), 401);
            }
            _ => panic!("Should return unauthorized for missing token"),
        }
    }
    
    #[test]
    fn test_rate_limit_middleware() {
        let middleware = RateLimitMiddleware::new(2, 60);
        assert_eq!(middleware.name(), "rate_limit");
        assert_eq!(middleware.priority(), 15);
        
        let request = RequestContext::new(HttpMethod::GET, "/test".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        // 第一次请求应该成功
        let result = middleware.process(&mut context, next.clone()).unwrap();
        match result {
            MiddlewareResult::Continue => {
                // 应该继续执行
            }
            _ => panic!("First request should continue"),
        }
        
        // 第二次请求应该成功
        let result = middleware.process(&mut context, next.clone()).unwrap();
        match result {
            MiddlewareResult::Continue => {
                // 应该继续执行
            }
            _ => panic!("Second request should continue"),
        }
        
        // 第三次请求应该被限流
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Response(response) => {
                assert_eq!(response.status.as_u16(), 429);
                assert!(response.headers.contains_key("X-RateLimit-Limit"));
                assert!(response.headers.contains_key("Retry-After"));
            }
            _ => panic!("Third request should be rate limited"),
        }
    }
    
    #[test]
    fn test_rate_limit_by_user_middleware() {
        let middleware = RateLimitMiddleware::by_user_id();
        assert_eq!(middleware.name(), "rate_limit");
        
        let request = RequestContext::new(HttpMethod::GET, "/test".to_string());
        let mut context = MiddlewareContext::new(request);
        
        // 设置用户ID
        context.set_data("user_id".to_string(), "user123".to_string());
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Continue => {
                // 用户限流应该继续执行
            }
            _ => panic!("User rate limit should continue for first request"),
        }
    }
    
    #[test]
    fn test_logger_middleware_detailed() {
        let middleware = LoggerMiddleware::detailed();
        assert_eq!(middleware.name(), "logger");
        assert_eq!(middleware.priority(), 5);
        
        let mut request = RequestContext::new(HttpMethod::POST, "/api/test".to_string());
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"test": "data"}"#.as_bytes().to_vec());
        
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Response(
                ResponseContext::with_json(HttpStatus::Ok, r#"{"result": "success"}"#)
            ))
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Response(response) => {
                assert_eq!(response.status.as_u16(), 200);
            }
            _ => panic!("Logger middleware should return response"),
        }
    }
}