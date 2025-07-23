// ============================================================================
// 中间件集成测试 | Middleware Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::server::{WebServer, WebServerConfig};
    use super::super::handler::RequestHandler;
    use crate::core::types::{HttpMethod, ResponseContext, HttpStatus};
    use crate::core::error::HushResult;
    use crate::middleware::core::{MiddlewareContext, MiddlewareResult};
    use std::time::Duration;
    use std::thread;

    /// 创建测试用的 Web 服务器
    fn create_test_server() -> WebServer {
        let config = WebServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // 使用随机端口
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

    #[test]
    fn test_middleware_integration_basic() {
        let server = create_test_server();
        
        // 添加一个简单的路由
        let handler = create_test_handler("Hello, World!");
        server.add_route(HttpMethod::GET, "/test", handler).unwrap();
        
        // 验证中间件数量（应该为0，因为还没有启动服务器）
        assert_eq!(server.middleware_count(), 0);
        
        // 添加一个测试中间件
        server.add_middleware("test_middleware".to_string(), |ctx, next| {
            // 在请求上下文中添加一些数据
            ctx.set_data("middleware_executed".to_string(), "true".to_string());
            next(ctx)
        }).unwrap();
        
        // 验证中间件已添加
        assert_eq!(server.middleware_count(), 1);
    }

    #[test]
    fn test_cors_middleware_integration() {
        let server = create_test_server();
        
        // 添加 CORS 中间件
        server.add_cors_middleware("*".to_string()).unwrap();
        
        // 添加一个测试路由
        let handler = create_test_handler("CORS test");
        server.add_route(HttpMethod::GET, "/cors-test", handler).unwrap();
        
        // 验证中间件已添加
        assert!(server.middleware_count() > 0);
    }

    #[test]
    fn test_logger_middleware_integration() {
        let server = create_test_server();
        
        // 添加日志中间件
        server.add_logger_middleware().unwrap();
        
        // 添加一个测试路由
        let handler = create_test_handler("Logger test");
        server.add_route(HttpMethod::GET, "/logger-test", handler).unwrap();
        
        // 验证中间件已添加
        assert!(server.middleware_count() > 0);
    }

    #[test]
    fn test_auth_middleware_integration() {
        let server = create_test_server();
        
        // 添加认证中间件
        server.add_auth_middleware("test_secret".to_string()).unwrap();
        
        // 添加一个测试路由
        let handler = create_test_handler("Auth test");
        server.add_route(HttpMethod::GET, "/auth-test", handler).unwrap();
        
        // 验证中间件已添加
        assert!(server.middleware_count() > 0);
    }

    #[test]
    fn test_multiple_middleware_integration() {
        let server = create_test_server();
        
        // 添加多个中间件
        server.add_logger_middleware().unwrap();
        server.add_cors_middleware("https://example.com".to_string()).unwrap();
        server.add_auth_middleware("secret_key".to_string()).unwrap();
        
        // 添加自定义中间件
        server.add_middleware("custom_middleware".to_string(), |ctx, next| {
            ctx.set_data("custom_data".to_string(), "processed".to_string());
            next(ctx)
        }).unwrap();
        
        // 添加一个测试路由
        let handler = create_test_handler("Multiple middleware test");
        server.add_route(HttpMethod::GET, "/multi-test", handler).unwrap();
        
        // 验证所有中间件都已添加
        assert_eq!(server.middleware_count(), 4);
    }

    #[test]
    fn test_middleware_error_handling() {
        let server = create_test_server();
        
        // 添加一个会产生错误的中间件
        server.add_middleware("error_middleware".to_string(), |_ctx, _next| {
            Ok(MiddlewareResult::Error(
                crate::core::error::HushError::InternalError("Test error".to_string())
            ))
        }).unwrap();
        
        // 添加一个测试路由
        let handler = create_test_handler("Error test");
        server.add_route(HttpMethod::GET, "/error-test", handler).unwrap();
        
        // 验证中间件已添加
        assert!(server.middleware_count() > 0);
    }

    #[test]
    fn test_middleware_early_response() {
        let server = create_test_server();
        
        // 添加一个提前返回响应的中间件
        server.add_middleware("early_response_middleware".to_string(), |_ctx, _next| {
            let response = ResponseContext::with_json(
                HttpStatus::Ok,
                r#"{"message": "Early response from middleware"}"#
            );
            Ok(MiddlewareResult::Response(response))
        }).unwrap();
        
        // 添加一个测试路由（这个路由不应该被执行）
        let handler = create_test_handler("This should not be reached");
        server.add_route(HttpMethod::GET, "/early-response-test", handler).unwrap();
        
        // 验证中间件已添加
        assert!(server.middleware_count() > 0);
    }

    #[test]
    fn test_middleware_chain_execution_order() {
        let server = create_test_server();
        
        // 添加多个中间件来测试执行顺序
        server.add_middleware("first_middleware".to_string(), |ctx, next| {
            ctx.set_data("execution_order".to_string(), "first".to_string());
            let result = next(ctx)?;
            // 在响应返回时也可以进行处理
            Ok(result)
        }).unwrap();
        
        server.add_middleware("second_middleware".to_string(), |ctx, next| {
            let current = ctx.get_data("execution_order").unwrap_or(&"".to_string()).clone();
            ctx.set_data("execution_order".to_string(), format!("{},second", current));
            next(ctx)
        }).unwrap();
        
        // 添加一个测试路由
        let handler = RequestHandler::new(|context| {
            // 验证中间件执行顺序
            let default_order = "".to_string();
            let execution_order = context.get_user_data("execution_order").unwrap_or(&default_order);
            let response_text = format!("Execution order: {}", execution_order);
            Ok(ResponseContext::with_text(HttpStatus::Ok, &response_text))
        });
        server.add_route(HttpMethod::GET, "/order-test", handler).unwrap();
        
        // 验证中间件已添加
        assert_eq!(server.middleware_count(), 2);
    }

    #[test]
    fn test_server_startup_with_middleware() {
        let server = create_test_server();
        
        // 添加中间件
        server.add_logger_middleware().unwrap();
        server.add_cors_middleware("*".to_string()).unwrap();
        
        // 添加路由
        let handler = create_test_handler("Startup test");
        server.add_route(HttpMethod::GET, "/startup-test", handler).unwrap();
        
        // 启动服务器（使用随机端口）
        let port = 18080; // 使用固定端口进行测试
        let result = server.start_with_port(port);
        
        // 验证服务器启动成功
        assert!(result.is_ok());
        
        // 等待服务器启动
        thread::sleep(Duration::from_millis(100));
        
        // 验证服务器正在运行
        assert!(server.is_running());
        
        // 停止服务器
        server.stop().unwrap();
    }

    #[test]
    fn test_middleware_with_different_http_methods() {
        let server = create_test_server();
        
        // 添加一个记录 HTTP 方法的中间件
        server.add_middleware("method_logger".to_string(), |ctx, next| {
            println!("Processing {} request to {}", 
                ctx.request.method.as_str(), 
                ctx.request.path
            );
            next(ctx)
        }).unwrap();
        
        // 添加不同 HTTP 方法的路由
        server.add_route(HttpMethod::GET, "/method-test", 
            create_test_handler("GET response")).unwrap();
        server.add_route(HttpMethod::POST, "/method-test", 
            create_test_handler("POST response")).unwrap();
        server.add_route(HttpMethod::PUT, "/method-test", 
            create_test_handler("PUT response")).unwrap();
        server.add_route(HttpMethod::DELETE, "/method-test", 
            create_test_handler("DELETE response")).unwrap();
        
        // 验证中间件已添加
        assert!(server.middleware_count() > 0);
    }

    #[test]
    fn test_middleware_context_data_sharing() {
        let server = create_test_server();
        
        // 添加一个设置数据的中间件
        server.add_middleware("data_setter".to_string(), |ctx, next| {
            ctx.set_data("user_id".to_string(), "12345".to_string());
            ctx.set_data("session_id".to_string(), "abcdef".to_string());
            next(ctx)
        }).unwrap();
        
        // 添加一个读取数据的中间件
        server.add_middleware("data_reader".to_string(), |ctx, next| {
            let user_id = ctx.get_data("user_id").cloned().unwrap_or_default();
            let session_id = ctx.get_data("session_id").cloned().unwrap_or_default();
            ctx.set_data("combined_data".to_string(), 
                format!("user:{},session:{}", user_id, session_id));
            next(ctx)
        }).unwrap();
        
        // 添加测试路由
        let handler = RequestHandler::new(|context| {
            let default_data = "no_data".to_string();
            let combined_data = context.get_user_data("combined_data")
                .unwrap_or(&default_data);
            Ok(ResponseContext::with_text(HttpStatus::Ok, combined_data))
        });
        server.add_route(HttpMethod::GET, "/data-test", handler).unwrap();
        
        // 验证中间件已添加
        assert_eq!(server.middleware_count(), 2);
    }
}