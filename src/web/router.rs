// ============================================================================
// 路由管理器 | Route Manager
// ============================================================================

use std::collections::HashMap;
use crate::core::error::{HushError, HushResult};
use crate::core::types::{HttpMethod, RequestContext, ResponseContext, RouteInfo};
use super::handler::RequestHandler;

/// 路由管理器
pub struct Router {
    routes: HashMap<String, RequestHandler>,
    route_info: Vec<RouteInfo>,
}

impl Router {
    /// 创建新的路由管理器
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            route_info: Vec::new(),
        }
    }
    
    /// 添加路由
    pub fn add_route(&mut self, method: HttpMethod, path: String, handler: RequestHandler) -> HushResult<()> {
        let route_key = format!("{}:{}", method.as_str(), path);
        
        // 检查路由是否已存在
        if self.routes.contains_key(&route_key) {
            return Err(HushError::HttpError(format!("Route already exists: {}", route_key)));
        }
        
        // 添加路由信息
        let route_info = RouteInfo::new(method, path, format!("handler_{}", self.routes.len()));
        self.route_info.push(route_info);
        
        // 添加路由处理器
        self.routes.insert(route_key, handler);
        
        Ok(())
    }
    
    /// 移除路由
    pub fn remove_route(&mut self, method: HttpMethod, path: &str) -> HushResult<()> {
        let route_key = format!("{}:{}", method.as_str(), path);
        
        if self.routes.remove(&route_key).is_none() {
            return Err(HushError::RouteNotFound);
        }
        
        // 移除路由信息
        self.route_info.retain(|info| info.route_key() != route_key);
        
        Ok(())
    }
    
    /// 处理请求
    pub fn handle_request(&self, context: &RequestContext) -> HushResult<ResponseContext> {
        let route_key = format!("{}:{}", context.method.as_str(), context.path);
        
        match self.routes.get(&route_key) {
            Some(handler) => handler.handle(context),
            None => Err(HushError::RouteNotFound),
        }
    }
    
    /// 获取所有路由信息
    pub fn get_routes(&self) -> &Vec<RouteInfo> {
        &self.route_info
    }
    
    /// 检查路由是否存在
    pub fn has_route(&self, method: HttpMethod, path: &str) -> bool {
        let route_key = format!("{}:{}", method.as_str(), path);
        self.routes.contains_key(&route_key)
    }
    
    /// 获取路由数量
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
    
    /// 清空所有路由
    pub fn clear(&mut self) {
        self.routes.clear();
        self.route_info.clear();
    }
    
    /// 获取支持的 HTTP 方法列表（针对特定路径）
    pub fn get_supported_methods(&self, path: &str) -> Vec<HttpMethod> {
        let mut methods = Vec::new();
        
        for method in &[
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::DELETE,
            HttpMethod::PATCH,
            HttpMethod::HEAD,
            HttpMethod::OPTIONS,
        ] {
            if self.has_route(method.clone(), path) {
                methods.push(method.clone());
            }
        }
        
        methods
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 路由匹配器 | Route Matcher
// ============================================================================

/// 路由匹配器，支持路径参数和通配符
pub struct RouteMatcher {
    pattern: String,
    params: Vec<String>,
}

impl RouteMatcher {
    /// 创建新的路由匹配器
    pub fn new(pattern: &str) -> Self {
        let mut params = Vec::new();
        
        // 解析路径参数（如 /users/:id）
        for segment in pattern.split('/') {
            if segment.starts_with(':') {
                params.push(segment[1..].to_string());
            }
        }
        
        Self {
            pattern: pattern.to_string(),
            params,
        }
    }
    
    /// 匹配路径并提取参数
    pub fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let pattern_segments: Vec<&str> = self.pattern.split('/').collect();
        let path_segments: Vec<&str> = path.split('/').collect();
        
        // 段数必须相同
        if pattern_segments.len() != path_segments.len() {
            return None;
        }
        
        let mut params = HashMap::new();
        
        for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
            if pattern_seg.starts_with(':') {
                // 路径参数
                let param_name = &pattern_seg[1..];
                params.insert(param_name.to_string(), path_seg.to_string());
            } else if *pattern_seg != *path_seg {
                // 字面量段不匹配
                return None;
            }
        }
        
        Some(params)
    }
    
    /// 获取参数名列表
    pub fn param_names(&self) -> &Vec<String> {
        &self.params
    }
    
    /// 获取模式字符串
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_route_matcher() {
        let matcher = RouteMatcher::new("/users/:id/posts/:post_id");
        
        // 测试匹配成功
        let params = matcher.matches("/users/123/posts/456").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
        
        // 测试匹配失败
        assert!(matcher.matches("/users/123").is_none());
        assert!(matcher.matches("/users/123/posts").is_none());
        assert!(matcher.matches("/posts/123/users/456").is_none());
    }
    
    #[test]
    fn test_router_basic_operations() {
        let mut router = Router::new();
        
        // 测试添加路由
        let handler = RequestHandler::new(|_| {
            Ok(ResponseContext::with_text(crate::core::types::HttpStatus::Ok, "test"))
        });
        
        assert!(router.add_route(HttpMethod::GET, "/test".to_string(), handler).is_ok());
        assert_eq!(router.route_count(), 1);
        assert!(router.has_route(HttpMethod::GET, "/test"));
        
        // 测试重复添加路由
        let handler2 = RequestHandler::new(|_| {
            Ok(ResponseContext::with_text(crate::core::types::HttpStatus::Ok, "test2"))
        });
        assert!(router.add_route(HttpMethod::GET, "/test".to_string(), handler2).is_err());
        
        // 测试移除路由
        assert!(router.remove_route(HttpMethod::GET, "/test").is_ok());
        assert_eq!(router.route_count(), 0);
        assert!(!router.has_route(HttpMethod::GET, "/test"));
    }
}