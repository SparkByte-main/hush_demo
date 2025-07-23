// ============================================================================
// Web 服务器实现 | Web Server Implementation
// ============================================================================


use std::sync::{Arc, Mutex};
use std::thread;
use actix_web::{web, App, HttpServer};
use crate::core::error::{HushError, HushResult, set_last_error};
use crate::core::types::{HttpMethod, RequestContext};
use crate::middleware::core::{MiddlewareChain, MiddlewareContext, MiddlewareResult};
use super::router::Router;
use super::handler::RequestHandler;

/// Web 服务器配置
#[derive(Debug, Clone)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub keep_alive: u64,
    pub request_timeout: u64,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            keep_alive: 30,
            request_timeout: 30,
        }
    }
}

/// 重构后的 Web 服务器结构体
pub struct WebServer {
    config: WebServerConfig,
    router: Arc<Mutex<Router>>,
    middleware_chain: Arc<Mutex<MiddlewareChain>>,
    is_running: Arc<Mutex<bool>>,
}

impl WebServer {
    /// 创建新的 Web 服务器实例
    pub fn new(config: WebServerConfig) -> Self {
        Self {
            config,
            router: Arc::new(Mutex::new(Router::new())),
            middleware_chain: Arc::new(Mutex::new(MiddlewareChain::new())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 使用默认配置创建服务器
    pub fn with_default_config() -> Self {
        Self::new(WebServerConfig::default())
    }
    
    /// 添加路由
    pub fn add_route(&self, method: HttpMethod, path: &str, handler: RequestHandler) -> HushResult<()> {
        let mut router = self.router.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire router lock".to_string()))?;
        router.add_route(method, path.to_string(), handler)
    }
    
    /// 添加中间件
    pub fn add_middleware<F>(&self, name: String, handler: F) -> HushResult<()>
    where
        F: Fn(&mut MiddlewareContext, Box<dyn Fn(&mut MiddlewareContext) -> HushResult<MiddlewareResult> + Send + Sync>) -> HushResult<MiddlewareResult> + Send + Sync + 'static,
    {
        let mut chain = self.middleware_chain.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire middleware lock".to_string()))?;
        chain.add_function(name, handler);
        Ok(())
    }
    
    /// 初始化默认中间件（包括路由处理器）
    fn initialize_default_middleware(&self) -> HushResult<()> {
        let mut chain = self.middleware_chain.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire middleware lock".to_string()))?;
        
        // 添加路由处理作为最后一个中间件
        let router_clone = Arc::clone(&self.router);
        chain.add_function("router_handler".to_string(), move |ctx, _next| {
            match router_clone.lock() {
                Ok(router) => {
                    match router.handle_request(&ctx.request) {
                        Ok(response) => Ok(MiddlewareResult::Response(response)),
                        Err(error) => Ok(MiddlewareResult::Error(error)),
                    }
                }
                Err(_) => Ok(MiddlewareResult::Error(
                    HushError::InternalError("Failed to acquire router lock".to_string())
                )),
            }
        });
        
        Ok(())
    }
    
    /// 获取中间件数量
    pub fn middleware_count(&self) -> usize {
        self.middleware_chain.lock()
            .map(|chain| chain.len())
            .unwrap_or(0)
    }
    
    /// 添加 CORS 中间件
    pub fn add_cors_middleware(&self, allowed_origins: String) -> HushResult<()> {
        use crate::middleware::builtin::CorsMiddleware;
        let mut chain = self.middleware_chain.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire middleware lock".to_string()))?;
        let cors_middleware = CorsMiddleware::new(allowed_origins);
        chain.add(cors_middleware);
        Ok(())
    }
    
    /// 添加日志中间件
    pub fn add_logger_middleware(&self) -> HushResult<()> {
        use crate::middleware::builtin::LoggerMiddleware;
        let mut chain = self.middleware_chain.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire middleware lock".to_string()))?;
        let logger_middleware = LoggerMiddleware::new();
        chain.add(logger_middleware);
        Ok(())
    }
    
    /// 添加 JWT 认证中间件
    pub fn add_auth_middleware(&self, secret: String) -> HushResult<()> {
        use crate::middleware::builtin::AuthMiddleware;
        let mut chain = self.middleware_chain.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire middleware lock".to_string()))?;
        let auth_middleware = AuthMiddleware::new(secret);
        chain.add(auth_middleware);
        Ok(())
    }
    
    /// 启动服务器
    pub fn start(&self) -> HushResult<()> {
        self.start_with_port(self.config.port)
    }
    
    /// 使用指定端口启动服务器
    pub fn start_with_port(&self, port: u16) -> HushResult<()> {
        // 检查服务器是否已经在运行
        {
            let mut running = self.is_running.lock()
                .map_err(|_| HushError::InternalError("Failed to acquire running lock".to_string()))?;
            if *running {
                return Err(HushError::HttpError("Server is already running".to_string()));
            }
            *running = true;
        }
        
        // 初始化默认中间件（包括路由处理器）
        self.initialize_default_middleware()?;
        
        let mut config = self.config.clone();
        config.port = port; // 使用传入的端口参数
        let router = Arc::clone(&self.router);
        let middleware_chain = Arc::clone(&self.middleware_chain);
        let is_running = Arc::clone(&self.is_running);
        
        // 在新线程中启动服务器
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                println!("Starting Hush web server on {}:{}", config.host, config.port);
                
                let server_result = HttpServer::new(move || {
                    let router_clone = Arc::clone(&router);
                    let middleware_clone = Arc::clone(&middleware_chain);
                    App::new()
                        .app_data(web::Data::new(router_clone))
                        .app_data(web::Data::new(middleware_clone))
                        .default_service(web::route().to(Self::handle_request))
                })
                .bind((config.host.as_str(), config.port));
                
                match server_result {
                    Ok(server) => {
                        if let Err(e) = server.run().await {
                            set_last_error(HushError::HttpError(format!("Server runtime error: {}", e)));
                            // 标记服务器为未运行状态
                            if let Ok(mut running) = is_running.lock() {
                                *running = false;
                            }
                        }
                    }
                    Err(e) => {
                        set_last_error(HushError::HttpError(format!("Failed to bind server: {}", e)));
                        // 标记服务器为未运行状态
                        if let Ok(mut running) = is_running.lock() {
                            *running = false;
                        }
                    }
                }
            });
        });
        
        Ok(())
    }
    
    /// 停止服务器
    pub fn stop(&self) -> HushResult<()> {
        let mut running = self.is_running.lock()
            .map_err(|_| HushError::InternalError("Failed to acquire running lock".to_string()))?;
        *running = false;
        Ok(())
    }
    
    /// 检查服务器是否在运行
    pub fn is_running(&self) -> bool {
        self.is_running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
    
    /// 获取服务器配置
    pub fn config(&self) -> &WebServerConfig {
        &self.config
    }
    
    /// 处理 HTTP 请求的核心函数
    async fn handle_request(
        req: actix_web::HttpRequest,
        body: actix_web::web::Bytes,
        router_data: web::Data<Arc<Mutex<Router>>>,
        middleware_data: web::Data<Arc<Mutex<MiddlewareChain>>>,
    ) -> actix_web::HttpResponse {
        // 解析 HTTP 方法
        let method = match HttpMethod::from_str(req.method().as_str()) {
            Ok(m) => m,
            Err(_) => {
                return actix_web::HttpResponse::MethodNotAllowed()
                    .body("Unsupported HTTP method");
            }
        };
        
        // 创建请求上下文
        let mut context = RequestContext::new(method.clone(), req.path().to_string());
        context.set_body(body.to_vec());
        
        // 添加请求头
        for (name, value) in req.headers() {
            if let Ok(value_str) = value.to_str() {
                context.add_header(name.to_string(), value_str.to_string());
            }
        }
        
        // 解析查询参数
        for (key, value) in req.query_string().split('&').filter_map(|pair| {
            let mut parts = pair.split('=');
            match (parts.next(), parts.next()) {
                (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                _ => None,
            }
        }) {
            context.add_query_param(key, value);
        }
        
        // 创建中间件上下文
        let middleware_context = MiddlewareContext::new(context);
        
        // 获取中间件链并执行
        match middleware_data.lock() {
            Ok(chain) => {
                // 执行中间件链（路由处理器已经在初始化时添加）
                match chain.execute(middleware_context) {
                    Ok(response) => {
                        let mut http_response = actix_web::HttpResponse::build(
                            actix_web::http::StatusCode::from_u16(response.status.as_u16())
                                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
                        );
                        
                        // 添加响应头
                        for (key, value) in &response.headers {
                            http_response.insert_header((key.as_str(), value.as_str()));
                        }
                        
                        http_response.body(response.body.clone())
                    }
                    Err(HushError::RouteNotFound) => {
                        actix_web::HttpResponse::NotFound().body("Route not found")
                    }
                    Err(HushError::MethodNotAllowed) => {
                        actix_web::HttpResponse::MethodNotAllowed().body("Method not allowed")
                    }
                    Err(error) => {
                        set_last_error(error);
                        actix_web::HttpResponse::InternalServerError()
                            .body("Internal server error")
                    }
                }
            }
            Err(_) => {
                actix_web::HttpResponse::InternalServerError()
                    .body("Failed to acquire middleware lock")
            }
        }
    }
}