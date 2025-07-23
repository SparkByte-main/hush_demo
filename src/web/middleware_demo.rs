// ============================================================================
// 中间件系统演示 | Middleware System Demo
// ============================================================================

use super::server::{WebServer, WebServerConfig};
use super::handler::RequestHandler;
use crate::core::types::{HttpMethod, ResponseContext, HttpStatus};
use crate::middleware::core::MiddlewareResult;
use std::time::Duration;
use std::thread;

/// 演示中间件系统的集成使用
pub fn demo_middleware_integration() {
    println!("=== Hush 框架中间件系统演示 ===");
    
    // 创建 Web 服务器
    let config = WebServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        max_connections: 1000,
        keep_alive: 30,
        request_timeout: 30,
    };
    let server = WebServer::new(config);
    
    println!("1. 添加日志中间件...");
    server.add_logger_middleware().unwrap();
    
    println!("2. 添加 CORS 中间件...");
    server.add_cors_middleware("*".to_string()).unwrap();
    
    println!("3. 添加自定义请求计数中间件...");
    server.add_middleware("request_counter".to_string(), |ctx, next| {
        // 模拟请求计数
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            ctx.set_data("request_count".to_string(), COUNTER.to_string());
            println!("  -> 处理第 {} 个请求: {} {}", COUNTER, ctx.request.method.as_str(), ctx.request.path);
        }
        next(ctx)
    }).unwrap();
    
    println!("4. 添加性能监控中间件...");
    server.add_middleware("performance_monitor".to_string(), |ctx, next| {
        let start = std::time::Instant::now();
        let result = next(ctx)?;
        let duration = start.elapsed();
        println!("  -> 请求处理耗时: {:?}", duration);
        Ok(result)
    }).unwrap();
    
    println!("5. 添加响应头中间件...");
    server.add_middleware("response_headers".to_string(), |ctx, next| {
        match next(ctx)? {
            MiddlewareResult::Response(mut response) => {
                response.add_header("X-Powered-By".to_string(), "Hush Framework".to_string());
                response.add_header("X-Framework-Version".to_string(), "1.0.0".to_string());
                if let Some(count) = ctx.get_data("request_count") {
                    response.add_header("X-Request-Count".to_string(), count.clone());
                }
                Ok(MiddlewareResult::Response(response))
            }
            other => Ok(other),
        }
    }).unwrap();
    
    println!("6. 添加路由处理器...");
    
    // 主页路由
    let home_handler = RequestHandler::new(|context| {
        let default_count = "0".to_string();
        let count = context.get_user_data("request_count").unwrap_or(&default_count);
        let response_html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Hush 框架中间件演示</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ color: #333; border-bottom: 2px solid #007acc; padding-bottom: 10px; }}
        .info {{ background: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0; }}
        .middleware {{ background: #e8f4fd; padding: 10px; margin: 10px 0; border-left: 4px solid #007acc; }}
    </style>
</head>
<body>
    <h1 class="header">🚀 Hush 框架中间件系统演示</h1>
    
    <div class="info">
        <h2>请求信息</h2>
        <p><strong>请求方法:</strong> {}</p>
        <p><strong>请求路径:</strong> {}</p>
        <p><strong>请求计数:</strong> {}</p>
        <p><strong>追踪 ID:</strong> {}</p>
    </div>
    
    <div class="middleware">
        <h3>中间件执行链</h3>
        <ol>
            <li>📝 日志中间件 - 记录请求信息</li>
            <li>🌐 CORS 中间件 - 处理跨域请求</li>
            <li>📊 请求计数中间件 - 统计请求数量</li>
            <li>⏱️ 性能监控中间件 - 监控处理时间</li>
            <li>📋 响应头中间件 - 添加自定义响应头</li>
            <li>🎯 路由处理器 - 生成最终响应</li>
        </ol>
    </div>
    
    <div class="info">
        <h3>测试链接</h3>
        <ul>
            <li><a href="/api/users">API 用户列表</a></li>
            <li><a href="/api/health">健康检查</a></li>
            <li><a href="/protected">受保护资源</a></li>
        </ul>
    </div>
</body>
</html>"#,
            context.method.as_str(),
            context.path,
            count,
            context.trace_id
        );
        
        let mut response = ResponseContext::with_text(HttpStatus::Ok, &response_html);
        response.add_header("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        Ok(response)
    });
    server.add_route(HttpMethod::GET, "/", home_handler).unwrap();
    
    // API 路由
    let api_handler = RequestHandler::new(|context| {
        let default_count = "0".to_string();
        let count = context.get_user_data("request_count").unwrap_or(&default_count);
        let response_json = format!(
            r#"{{
    "message": "API 响应成功",
    "method": "{}",
    "path": "{}",
    "request_count": {},
    "trace_id": "{}",
    "middleware_chain": [
        "logger",
        "cors", 
        "request_counter",
        "performance_monitor",
        "response_headers",
        "router"
    ],
    "timestamp": "{:?}"
}}"#,
            context.method.as_str(),
            context.path,
            count,
            context.trace_id,
            std::time::SystemTime::now()
        );
        
        Ok(ResponseContext::with_json(HttpStatus::Ok, &response_json))
    });
    server.add_route(HttpMethod::GET, "/api/users", api_handler).unwrap();
    
    // 健康检查路由
    let health_handler = RequestHandler::new(|_context| {
        let health_json = r#"{
    "status": "healthy",
    "service": "hush-framework",
    "middleware_system": "active",
    "timestamp": "2024-01-01T00:00:00Z"
}"#;
        Ok(ResponseContext::with_json(HttpStatus::Ok, health_json))
    });
    server.add_route(HttpMethod::GET, "/api/health", health_handler).unwrap();
    
    // 受保护资源路由（演示认证中间件）
    let protected_handler = RequestHandler::new(|context| {
        // 检查是否有认证信息
        let auth_header = context.get_header("Authorization");
        if auth_header.is_none() {
            return Ok(ResponseContext::with_json(
                HttpStatus::Unauthorized,
                r#"{"error": "Missing Authorization header", "message": "请提供认证令牌"}"#
            ));
        }
        
        let response_json = r#"{
    "message": "成功访问受保护资源",
    "resource": "protected_data",
    "access_granted": true
}"#;
        Ok(ResponseContext::with_json(HttpStatus::Ok, response_json))
    });
    server.add_route(HttpMethod::GET, "/protected", protected_handler).unwrap();
    
    println!("7. 中间件配置完成，当前中间件数量: {}", server.middleware_count());
    
    println!("8. 启动服务器...");
    match server.start() {
        Ok(_) => {
            println!("✅ 服务器启动成功！");
            println!("🌐 访问 http://127.0.0.1:8080 查看演示");
            println!("📊 中间件链将按以下顺序执行:");
            println!("   1. 日志中间件 (优先级: 5)");
            println!("   2. CORS 中间件 (优先级: 10)");
            println!("   3. 请求计数中间件 (优先级: 100)");
            println!("   4. 性能监控中间件 (优先级: 100)");
            println!("   5. 响应头中间件 (优先级: 100)");
            println!("   6. 路由处理器 (最后执行)");
            println!();
            println!("🔧 测试命令:");
            println!("   curl http://127.0.0.1:8080/");
            println!("   curl http://127.0.0.1:8080/api/users");
            println!("   curl http://127.0.0.1:8080/api/health");
            println!("   curl -H 'Authorization: Bearer token123' http://127.0.0.1:8080/protected");
            println!();
            println!("按 Ctrl+C 停止服务器");
            
            // 保持服务器运行
            loop {
                thread::sleep(Duration::from_secs(1));
                if !server.is_running() {
                    break;
                }
            }
        }
        Err(e) => {
            println!("❌ 服务器启动失败: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_setup() {
        // 测试演示设置是否正常工作
        let config = WebServerConfig::default();
        let server = WebServer::new(config);
        
        // 添加中间件
        server.add_logger_middleware().unwrap();
        server.add_cors_middleware("*".to_string()).unwrap();
        
        // 添加路由
        let handler = RequestHandler::new(|_| {
            Ok(ResponseContext::with_text(HttpStatus::Ok, "Demo test"))
        });
        server.add_route(HttpMethod::GET, "/demo", handler).unwrap();
        
        // 验证配置
        assert!(server.middleware_count() > 0);
    }
}