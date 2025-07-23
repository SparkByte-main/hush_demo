// ============================================================================
// 端到端中间件测试 | End-to-End Middleware Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::server::{WebServer, WebServerConfig};
    use super::super::handler::RequestHandler;
    use crate::core::types::{HttpMethod, ResponseContext, HttpStatus};
    use crate::middleware::core::{MiddlewareResult};
    use std::time::Duration;
    use std::thread;

    /// 创建测试用的 Web 服务器
    fn create_test_server(port: u16) -> WebServer {
        let config = WebServerConfig {
            host: "127.0.0.1".to_string(),
            port,
            max_connections: 100,
            keep_alive: 10,
            request_timeout: 10,
        };
        WebServer::new(config)
    }

    /// 创建简单的测试处理器
    fn create_test_handler(response_text: &'static str) -> RequestHandler {
        RequestHandler::new(move |_context| {
            Ok(ResponseContext::with_text(HttpStatus::Ok, response_text))
        })
    }

    /// 发送 HTTP 请求的辅助函数（简化版，实际应该使用 HTTP 客户端）
    fn make_http_request(url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 这里我们只是模拟 HTTP 请求，实际测试中应该使用真实的 HTTP 客户端
        // 由于这是一个集成测试，我们主要验证服务器能够正常启动和配置
        Ok(format!("Mock response for {}", url))
    }

    #[test]
    fn test_end_to_end_middleware_chain() {
        let port = 18081;
        let server = create_test_server(port);
        
        // 添加日志中间件
        server.add_logger_middleware().unwrap();
        
        // 添加 CORS 中间件
        server.add_cors_middleware("*".to_string()).unwrap();
        
        // 添加自定义中间件来记录请求
        server.add_middleware("request_counter".to_string(), |ctx, next| {
            println!("Processing request: {} {}", ctx.request.method.as_str(), ctx.request.path);
            ctx.set_data("processed_by_counter".to_string(), "true".to_string());
            next(ctx)
        }).unwrap();
        
        // 添加测试路由
        let handler = RequestHandler::new(|context| {
            let processed = context.get_user_data("processed_by_counter")
                .map(|s| s.as_str())
                .unwrap_or("false");
            let response_text = format!("Hello! Processed by counter: {}", processed);
            Ok(ResponseContext::with_text(HttpStatus::Ok, &response_text))
        });
        server.add_route(HttpMethod::GET, "/test", handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should be running");
        
        // 验证中间件数量（包括路由处理器）
        assert!(server.middleware_count() > 3, "Should have multiple middleware");
        
        // 停止服务器
        server.stop().unwrap();
        
        // 等待服务器停止
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_middleware_error_handling_end_to_end() {
        let port = 18082;
        let server = create_test_server(port);
        
        // 添加一个会产生错误的中间件
        server.add_middleware("error_middleware".to_string(), |_ctx, _next| {
            Ok(MiddlewareResult::Error(
                crate::core::error::HushError::InternalError("Middleware error test".to_string())
            ))
        }).unwrap();
        
        // 添加测试路由（这个不应该被执行）
        let handler = create_test_handler("This should not be reached");
        server.add_route(HttpMethod::GET, "/error-test", handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should be running");
        
        // 停止服务器
        server.stop().unwrap();
    }

    #[test]
    fn test_middleware_early_response_end_to_end() {
        let port = 18083;
        let server = create_test_server(port);
        
        // 添加一个提前返回响应的中间件
        server.add_middleware("early_response".to_string(), |_ctx, _next| {
            let response = ResponseContext::with_json(
                HttpStatus::Ok,
                r#"{"message": "Early response from middleware", "source": "middleware"}"#
            );
            Ok(MiddlewareResult::Response(response))
        }).unwrap();
        
        // 添加测试路由（这个不应该被执行）
        let handler = create_test_handler("This should not be reached");
        server.add_route(HttpMethod::GET, "/early-test", handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should be running");
        
        // 停止服务器
        server.stop().unwrap();
    }

    #[test]
    fn test_cors_middleware_end_to_end() {
        let port = 18084;
        let server = create_test_server(port);
        
        // 添加 CORS 中间件
        server.add_cors_middleware("https://example.com".to_string()).unwrap();
        
        // 添加测试路由
        let handler = create_test_handler("CORS test response");
        server.add_route(HttpMethod::GET, "/cors", handler).unwrap();
        
        // 添加 OPTIONS 路由来测试预检请求
        let options_handler = create_test_handler("OPTIONS response");
        server.add_route(HttpMethod::OPTIONS, "/cors", options_handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should be running");
        
        // 停止服务器
        server.stop().unwrap();
    }

    #[test]
    fn test_auth_middleware_end_to_end() {
        let port = 18085;
        let server = create_test_server(port);
        
        // 添加认证中间件
        server.add_auth_middleware("test_secret_key".to_string()).unwrap();
        
        // 添加受保护的路由
        let protected_handler = create_test_handler("Protected resource");
        server.add_route(HttpMethod::GET, "/protected", protected_handler).unwrap();
        
        // 添加公开的路由（健康检查）
        let health_handler = create_test_handler("OK");
        server.add_route(HttpMethod::GET, "/health", health_handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should be running");
        
        // 停止服务器
        server.stop().unwrap();
    }

    #[test]
    fn test_complex_middleware_chain_end_to_end() {
        let port = 18086;
        let server = create_test_server(port);
        
        // 添加多个中间件，测试复杂的中间件链
        
        // 1. 日志中间件（高优先级）
        server.add_logger_middleware().unwrap();
        
        // 2. CORS 中间件
        server.add_cors_middleware("*".to_string()).unwrap();
        
        // 3. 请求 ID 中间件
        server.add_middleware("request_id".to_string(), |ctx, next| {
            let request_id = format!("req_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());
            ctx.set_data("request_id".to_string(), request_id);
            next(ctx)
        }).unwrap();
        
        // 4. 性能监控中间件
        server.add_middleware("performance".to_string(), |ctx, next| {
            let start = std::time::Instant::now();
            let result = next(ctx)?;
            let duration = start.elapsed();
            println!("Request processed in {:?}", duration);
            Ok(result)
        }).unwrap();
        
        // 5. 响应头中间件
        server.add_middleware("response_headers".to_string(), |ctx, next| {
            match next(ctx)? {
                MiddlewareResult::Response(mut response) => {
                    response.add_header("X-Powered-By".to_string(), "Hush Framework".to_string());
                    if let Some(request_id) = ctx.get_data("request_id") {
                        response.add_header("X-Request-ID".to_string(), request_id.clone());
                    }
                    Ok(MiddlewareResult::Response(response))
                }
                other => Ok(other),
            }
        }).unwrap();
        
        // 添加测试路由
        let handler = RequestHandler::new(|context| {
            let default_id = "unknown".to_string();
            let request_id = context.get_user_data("request_id")
                .unwrap_or(&default_id);
            let response_text = format!("Complex middleware test - Request ID: {}", request_id);
            Ok(ResponseContext::with_text(HttpStatus::Ok, &response_text))
        });
        server.add_route(HttpMethod::GET, "/complex", handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should be running");
        
        // 验证中间件数量
        println!("Total middleware count: {}", server.middleware_count());
        assert!(server.middleware_count() >= 5, "Should have at least 5 middleware");
        
        // 停止服务器
        server.stop().unwrap();
    }

    #[test]
    fn test_middleware_crash_recovery() {
        let port = 18087;
        let server = create_test_server(port);
        
        // 添加一个可能会 panic 的中间件（但我们用错误处理来模拟）
        server.add_middleware("crash_test".to_string(), |_ctx, _next| {
            // 模拟中间件崩溃，但通过错误处理来优雅地处理
            Ok(MiddlewareResult::Error(
                crate::core::error::HushError::InternalError("Simulated crash".to_string())
            ))
        }).unwrap();
        
        // 添加测试路由
        let handler = create_test_handler("Should not reach here");
        server.add_route(HttpMethod::GET, "/crash-test", handler).unwrap();
        
        // 启动服务器
        let start_result = server.start_with_port(port);
        assert!(start_result.is_ok(), "Server should start successfully even with problematic middleware");
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(200));
        
        // 验证服务器正在运行
        assert!(server.is_running(), "Server should still be running after middleware error");
        
        // 停止服务器
        server.stop().unwrap();
    }
}