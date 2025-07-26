// ============================================================================
// 中间件集成测试 | Middleware Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::core::{MiddlewareChain, MiddlewareContext, Middleware};
    use super::super::builtin::{CorsMiddleware, LoggerMiddleware, AuthMiddleware, RateLimitMiddleware};
    use crate::core::types::{RequestContext, HttpMethod};

    /// 测试中间件链的基本功能
    #[test]
    fn test_middleware_chain_execution() {
        let mut chain = MiddlewareChain::new();
        
        // 添加多个中间件
        chain.add(LoggerMiddleware::new());
        chain.add(CorsMiddleware::new("*".to_string()));
        chain.add(RateLimitMiddleware::new(10, 60));
        
        assert_eq!(chain.len(), 3);
        
        // 创建测试请求
        let mut request = RequestContext::new(HttpMethod::GET, "/api/test".to_string());
        request.add_header("Origin".to_string(), "https://example.com".to_string());
        
        let context = MiddlewareContext::new(request);
        
        // 执行中间件链
        let result = chain.execute(context);
        assert!(result.is_ok());
    }

    /// 测试 CORS 中间件的完整功能
    #[test]
    fn test_cors_middleware_comprehensive() {
        let mut chain = MiddlewareChain::new();
        
        // 添加 CORS 中间件
        let cors = CorsMiddleware::new("https://example.com,https://test.com".to_string())
            .with_methods("GET, POST, PUT".to_string())
            .with_headers("Content-Type, Authorization, X-Custom-Header".to_string())
            .with_max_age(3600);
        
        chain.add(cors);
        
        // 测试 OPTIONS 预检请求
        let mut request = RequestContext::new(HttpMethod::OPTIONS, "/api/test".to_string());
        request.add_header("Origin".to_string(), "https://example.com".to_string());
        request.add_header("Access-Control-Request-Method".to_string(), "POST".to_string());
        request.add_header("Access-Control-Request-Headers".to_string(), "Content-Type".to_string());
        
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context).unwrap();
        
        assert_eq!(result.status.as_u16(), 204);
        assert_eq!(result.get_header("Access-Control-Allow-Origin"), Some(&"https://example.com".to_string()));
        assert_eq!(result.get_header("Access-Control-Allow-Methods"), Some(&"GET, POST, PUT".to_string()));
        assert!(result.headers.contains_key("Access-Control-Max-Age"));
    }

    /// 测试认证中间件的功能
    #[test]
    fn test_auth_middleware_comprehensive() {
        let mut chain = MiddlewareChain::new();
        
        // 添加认证中间件
        let auth = AuthMiddleware::new("secret_key".to_string())
            .with_skip_paths(vec!["/health".to_string(), "/public".to_string()])
            .with_header_name("X-Auth-Token".to_string());
        
        chain.add(auth);
        
        // 测试跳过路径
        let request = RequestContext::new(HttpMethod::GET, "/health".to_string());
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context);
        assert!(result.is_ok());
        
        // 测试需要认证的路径（无令牌）
        let request = RequestContext::new(HttpMethod::GET, "/protected".to_string());
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context).unwrap();
        assert_eq!(result.status.as_u16(), 401);
        
        // 测试需要认证的路径（有效令牌）
        let mut request = RequestContext::new(HttpMethod::GET, "/protected".to_string());
        request.add_header("X-Auth-Token".to_string(), "valid_token_12345".to_string());
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context);
        assert!(result.is_ok());
    }

    /// 测试基于用户ID的限流
    #[test]
    fn test_rate_limiting_by_user_id() {
        let rate_limit = RateLimitMiddleware::by_user_id();
        
        let mut chain = MiddlewareChain::new();
        chain.add(rate_limit);
        
        // 模拟用户请求
        let request = RequestContext::new(HttpMethod::GET, "/api/data".to_string());
        let mut context = MiddlewareContext::new(request);
        context.set_data("user_id".to_string(), "user123".to_string());
        
        let result = chain.execute(context);
        assert!(result.is_ok());
    }

    /// 测试日志中间件的详细模式
    #[test]
    fn test_logger_middleware_detailed() {
        let mut chain = MiddlewareChain::new();
        
        // 添加详细日志中间件
        let logger = LoggerMiddleware::detailed();
        chain.add(logger);
        
        // 创建带有请求体和头部的请求
        let mut request = RequestContext::new(HttpMethod::POST, "/api/create".to_string());
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.add_header("User-Agent".to_string(), "TestClient/1.0".to_string());
        request.set_body(r#"{"name": "test", "value": 123}"#.as_bytes().to_vec());
        
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context);
        assert!(result.is_ok());
    }

    /// 测试完整的中间件栈
    #[test]
    fn test_complete_middleware_stack() {
        let mut chain = MiddlewareChain::new();
        
        // 按优先级添加中间件
        chain.add(LoggerMiddleware::new());                                    // 优先级 5
        chain.add(CorsMiddleware::permissive());                              // 优先级 10
        chain.add(RateLimitMiddleware::new(100, 3600));                       // 优先级 15
        chain.add(AuthMiddleware::new("secret".to_string())                   // 优先级 20
            .with_skip_paths(vec!["/health".to_string()]));
        
        assert_eq!(chain.len(), 4);
        
        // 验证中间件按优先级排序
        let names = chain.middleware_names();
        assert_eq!(names[0], "logger");      // 优先级最高
        assert_eq!(names[1], "cors");
        assert_eq!(names[2], "rate_limit");
        assert_eq!(names[3], "auth_jwt");    // 优先级最低
        
        // 测试健康检查请求（应该跳过认证）
        let mut request = RequestContext::new(HttpMethod::GET, "/health".to_string());
        request.add_header("Origin".to_string(), "https://example.com".to_string());
        
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context);
        assert!(result.is_ok());
        
        // 测试需要认证的请求
        let mut request = RequestContext::new(HttpMethod::POST, "/api/secure".to_string());
        request.add_header("Origin".to_string(), "https://example.com".to_string());
        request.add_header("Authorization".to_string(), "Bearer valid_token_12345".to_string());
        request.set_body(r#"{"action": "create"}"#.as_bytes().to_vec());
        
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context);
        assert!(result.is_ok());
    }

    /// 测试中间件错误处理
    #[test]
    fn test_middleware_error_handling() {
        let mut chain = MiddlewareChain::new();
        
        // 添加会产生错误的中间件配置
        chain.add(CorsMiddleware::new("https://allowed.com".to_string()));
        
        // 测试不允许的 Origin
        let mut request = RequestContext::new(HttpMethod::GET, "/api/test".to_string());
        request.add_header("Origin".to_string(), "https://malicious.com".to_string());
        
        let context = MiddlewareContext::new(request);
        let result = chain.execute(context).unwrap();
        
        // 应该返回 403 Forbidden
        assert_eq!(result.status.as_u16(), 403);
    }

    /// 测试中间件性能
    #[test]
    fn test_middleware_performance() {
        let mut chain = MiddlewareChain::new();
        
        // 添加所有中间件
        chain.add(LoggerMiddleware::requests_only());
        chain.add(CorsMiddleware::permissive());
        chain.add(RateLimitMiddleware::new(1000, 60));
        
        let start = std::time::Instant::now();
        
        // 执行多次请求
        for i in 0..100 {
            let request = RequestContext::new(HttpMethod::GET, format!("/test/{}", i));
            let context = MiddlewareContext::new(request);
            let _ = chain.execute(context);
        }
        
        let duration = start.elapsed();
        println!("100 requests processed in {:?}", duration);
        
        // 确保性能在合理范围内（每个请求不超过1ms）
        assert!(duration.as_millis() < 100);
    }

    /// 测试中间件配置的灵活性
    #[test]
    fn test_middleware_configuration_flexibility() {
        // 测试不同的 CORS 配置
        let cors1 = CorsMiddleware::new("*".to_string());
        let cors2 = CorsMiddleware::new("https://example.com".to_string())
            .with_methods("GET, POST".to_string())
            .with_headers("Content-Type".to_string())
            .with_max_age(1800);
        
        assert_eq!(cors1.name(), "cors");
        assert_eq!(cors2.name(), "cors");
        
        // 测试不同的日志配置
        let logger1 = LoggerMiddleware::new();
        let logger2 = LoggerMiddleware::requests_only();
        let logger3 = LoggerMiddleware::detailed().with_headers().with_body();
        
        assert_eq!(logger1.name(), "logger");
        assert_eq!(logger2.name(), "logger");
        assert_eq!(logger3.name(), "logger");
        
        // 测试不同的限流配置
        let rate1 = RateLimitMiddleware::new(10, 60);
        let rate2 = RateLimitMiddleware::by_user_id();
        let rate3 = RateLimitMiddleware::with_user_limits(50, 300);
        
        assert_eq!(rate1.name(), "rate_limit");
        assert_eq!(rate2.name(), "rate_limit");
        assert_eq!(rate3.name(), "rate_limit");
    }
}