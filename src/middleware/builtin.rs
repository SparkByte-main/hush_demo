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
            allowed_headers: "Content-Type, Authorization".to_string(),
            max_age: 86400, // 24 hours
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
}

impl Middleware for CorsMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        // 检查是否是 OPTIONS 预检请求
        if context.request.method.as_str() == "OPTIONS" {
            let mut response = ResponseContext::new(HttpStatus::NoContent);
            response.add_header("Access-Control-Allow-Origin".to_string(), self.allowed_origins.clone());
            response.add_header("Access-Control-Allow-Methods".to_string(), self.allowed_methods.clone());
            response.add_header("Access-Control-Allow-Headers".to_string(), self.allowed_headers.clone());
            response.add_header("Access-Control-Max-Age".to_string(), self.max_age.to_string());
            
            return Ok(MiddlewareResult::Response(response));
        }
        
        // 对于其他请求，继续执行并添加 CORS 头
        match next(context)? {
            MiddlewareResult::Response(mut response) => {
                response.add_header("Access-Control-Allow-Origin".to_string(), self.allowed_origins.clone());
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
}

impl LoggerMiddleware {
    pub fn new() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
        }
    }
    
    pub fn requests_only() -> Self {
        Self {
            log_requests: true,
            log_responses: false,
        }
    }
    
    pub fn responses_only() -> Self {
        Self {
            log_requests: false,
            log_responses: true,
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
        }
        
        // 执行下一个中间件
        let result = next(context)?;
        
        if self.log_responses {
            let duration = start_time.elapsed().unwrap_or_default();
            match &result {
                MiddlewareResult::Response(response) => {
                    println!("[{}] {} {} - {} ({:.2}ms)", 
                        format_time(SystemTime::now()),
                        context.request.method.as_str(),
                        context.request.path,
                        response.status.as_u16(),
                        duration.as_millis() as f64
                    );
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
    // 简化实现，实际应该使用更复杂的数据结构
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        // 简化的限流逻辑
        // 实际实现应该使用 Redis 或内存存储来跟踪请求计数
        
        // 这里我们简单地检查一个标志
        if context.get_data("rate_limited").is_some() {
            let response = ResponseContext::with_json(
                HttpStatus::ServiceUnavailable,
                r#"{"error": "Rate limit exceeded"}"#
            );
            return Ok(MiddlewareResult::Response(response));
        }
        
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
        let request = RequestContext::new(HttpMethod::OPTIONS, "/test".to_string());
        let mut context = MiddlewareContext::new(request);
        
        let next = Box::new(|_ctx: &mut MiddlewareContext| {
            Ok(MiddlewareResult::Continue)
        });
        
        let result = middleware.process(&mut context, next).unwrap();
        match result {
            MiddlewareResult::Response(response) => {
                assert_eq!(response.status.as_u16(), 204);
                assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
            }
            _ => panic!("Expected response for OPTIONS request"),
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
}