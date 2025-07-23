// ============================================================================
// 请求处理器 | Request Handler
// ============================================================================

use std::sync::Arc;
use crate::core::error::HushResult;
use crate::core::types::{RequestContext, ResponseContext, HttpStatus};

/// 请求处理器函数类型
pub type HandlerFn = dyn Fn(&RequestContext) -> HushResult<ResponseContext> + Send + Sync;

/// 请求处理器包装器
pub struct RequestHandler {
    handler: Arc<HandlerFn>,
}

impl RequestHandler {
    /// 创建新的请求处理器
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&RequestContext) -> HushResult<ResponseContext> + Send + Sync + 'static,
    {
        Self {
            handler: Arc::new(handler),
        }
    }
    
    /// 处理请求
    pub fn handle(&self, context: &RequestContext) -> HushResult<ResponseContext> {
        (self.handler)(context)
    }
}

impl Clone for RequestHandler {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
        }
    }
}

/// 响应构建器，提供便捷的响应创建方法
pub struct ResponseBuilder {
    response: ResponseContext,
}

impl ResponseBuilder {
    /// 创建新的响应构建器
    pub fn new(status: HttpStatus) -> Self {
        Self {
            response: ResponseContext::new(status),
        }
    }
    
    /// 设置响应体
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.response.set_body(body);
        self
    }
    
    /// 设置文本响应体
    pub fn text(mut self, text: &str) -> Self {
        self.response.set_body(text.as_bytes().to_vec());
        self
    }
    
    /// 设置 JSON 响应体
    pub fn json(mut self, json: &str) -> Self {
        self.response.set_body(json.as_bytes().to_vec());
        self.response.add_header("Content-Type".to_string(), "application/json".to_string());
        self
    }
    
    /// 设置 HTML 响应体
    pub fn html(mut self, html: &str) -> Self {
        self.response.set_body(html.as_bytes().to_vec());
        self.response.add_header("Content-Type".to_string(), "text/html".to_string());
        self
    }
    
    /// 添加响应头
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.response.add_header(key.to_string(), value.to_string());
        self
    }
    
    /// 构建响应
    pub fn build(self) -> ResponseContext {
        self.response
    }
}

// ============================================================================
// 常用响应处理器 | Common Response Handlers
// ============================================================================

/// 创建简单的文本响应处理器
pub fn text_handler(text: &'static str) -> RequestHandler {
    RequestHandler::new(move |_context| {
        Ok(ResponseBuilder::new(HttpStatus::Ok)
            .text(text)
            .build())
    })
}

/// 创建简单的 JSON 响应处理器
pub fn json_handler(json: &'static str) -> RequestHandler {
    RequestHandler::new(move |_context| {
        Ok(ResponseBuilder::new(HttpStatus::Ok)
            .json(json)
            .build())
    })
}

/// 创建简单的 HTML 响应处理器
pub fn html_handler(html: &'static str) -> RequestHandler {
    RequestHandler::new(move |_context| {
        Ok(ResponseBuilder::new(HttpStatus::Ok)
            .html(html)
            .build())
    })
}

/// 创建 404 错误处理器
pub fn not_found_handler() -> RequestHandler {
    RequestHandler::new(|_context| {
        Ok(ResponseBuilder::new(HttpStatus::NotFound)
            .text("Not Found")
            .build())
    })
}

/// 创建 500 错误处理器
pub fn internal_error_handler() -> RequestHandler {
    RequestHandler::new(|_context| {
        Ok(ResponseBuilder::new(HttpStatus::InternalServerError)
            .text("Internal Server Error")
            .build())
    })
}

/// 创建回显请求信息的处理器
pub fn echo_handler() -> RequestHandler {
    RequestHandler::new(|context| {
        let body = context.body_as_string().unwrap_or_default();
        let response_text = format!(
            "Method: {}\nPath: {}\nBody: {}\nHeaders: {:?}",
            context.method.as_str(),
            context.path,
            body,
            context.headers
        );
        
        Ok(ResponseBuilder::new(HttpStatus::Ok)
            .text(&response_text)
            .build())
    })
}

/// 创建健康检查处理器
pub fn health_check_handler() -> RequestHandler {
    RequestHandler::new(|_context| {
        let health_info = r#"{"status":"ok","timestamp":"2024-01-01T00:00:00Z"}"#;
        Ok(ResponseBuilder::new(HttpStatus::Ok)
            .json(health_info)
            .build())
    })
}

// ============================================================================
// 中间件支持 | Middleware Support
// ============================================================================

/// 中间件函数类型
pub type MiddlewareFn = dyn Fn(&mut RequestContext, &dyn Fn(&RequestContext) -> HushResult<ResponseContext>) -> HushResult<ResponseContext> + Send + Sync;

/// 中间件包装器
pub struct Middleware {
    middleware: Arc<MiddlewareFn>,
}

impl Middleware {
    /// 创建新的中间件
    pub fn new<F>(middleware: F) -> Self
    where
        F: Fn(&mut RequestContext, &dyn Fn(&RequestContext) -> HushResult<ResponseContext>) -> HushResult<ResponseContext> + Send + Sync + 'static,
    {
        Self {
            middleware: Arc::new(middleware),
        }
    }
    
    /// 应用中间件到处理器
    pub fn apply(&self, handler: RequestHandler) -> RequestHandler {
        let middleware = Arc::clone(&self.middleware);
        RequestHandler::new(move |context| {
            let mut ctx = context.clone();
            let handler_fn = |ctx: &RequestContext| handler.handle(ctx);
            (middleware)(&mut ctx, &handler_fn)
        })
    }
}

/// 日志中间件
pub fn logging_middleware() -> Middleware {
    Middleware::new(|context, next| {
        let start_time = std::time::Instant::now();
        println!("Request: {} {}", context.method.as_str(), context.path);
        
        let result = next(context);
        
        let duration = start_time.elapsed();
        match &result {
            Ok(response) => {
                println!("Response: {} - {}ms", response.status.as_u16(), duration.as_millis());
            }
            Err(error) => {
                println!("Error: {} - {}ms", error, duration.as_millis());
            }
        }
        
        result
    })
}

/// CORS 中间件
pub fn cors_middleware(allowed_origins: &'static str) -> Middleware {
    Middleware::new(move |context, next| {
        let mut response = next(context)?;
        response.add_header("Access-Control-Allow-Origin".to_string(), allowed_origins.to_string());
        response.add_header("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string());
        response.add_header("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization".to_string());
        Ok(response)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::HttpMethod;
    
    #[test]
    fn test_response_builder() {
        let response = ResponseBuilder::new(HttpStatus::Ok)
            .text("Hello, World!")
            .header("Content-Type", "text/plain")
            .build();
        
        assert_eq!(response.status.as_u16(), 200);
        assert_eq!(response.body_as_string().unwrap(), "Hello, World!");
        assert_eq!(response.headers.get("Content-Type"), Some(&"text/plain".to_string()));
    }
    
    #[test]
    fn test_text_handler() {
        let handler = text_handler("Test response");
        let context = RequestContext::new(HttpMethod::GET, "/test".to_string());
        
        let response = handler.handle(&context).unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert_eq!(response.body_as_string().unwrap(), "Test response");
    }
    
    #[test]
    fn test_json_handler() {
        let handler = json_handler(r#"{"message":"test"}"#);
        let context = RequestContext::new(HttpMethod::GET, "/api/test".to_string());
        
        let response = handler.handle(&context).unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert_eq!(response.body_as_string().unwrap(), r#"{"message":"test"}"#);
        assert_eq!(response.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
}