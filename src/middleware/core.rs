// ============================================================================
// 中间件系统核心实现 | Middleware System Core Implementation
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use crate::core::error::{HushError, HushResult};
use crate::core::types::{RequestContext, ResponseContext, HttpStatus};

/// 中间件执行结果
#[derive(Debug)]
pub enum MiddlewareResult {
    /// 继续执行下一个中间件
    Continue,
    /// 提前返回响应，跳过后续中间件
    Response(ResponseContext),
    /// 中间件执行出错
    Error(HushError),
}

/// 中间件上下文，包含请求信息和共享数据
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    pub request: RequestContext,
    pub shared_data: HashMap<String, String>,
}

impl MiddlewareContext {
    pub fn new(request: RequestContext) -> Self {
        Self {
            request,
            shared_data: HashMap::new(),
        }
    }
    
    pub fn set_data(&mut self, key: String, value: String) {
        self.shared_data.insert(key, value);
    }
    
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.shared_data.get(key)
    }
    
    pub fn remove_data(&mut self, key: &str) -> Option<String> {
        self.shared_data.remove(key)
    }
}

/// Next 函数类型，用于调用下一个中间件
pub type NextFunction = Box<dyn Fn(&mut MiddlewareContext) -> HushResult<MiddlewareResult> + Send + Sync>;

/// 中间件处理函数类型
pub type MiddlewareHandler = Box<dyn Fn(&mut MiddlewareContext, NextFunction) -> HushResult<MiddlewareResult> + Send + Sync>;

/// 中间件特征定义
pub trait Middleware: Send + Sync {
    /// 处理请求
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult>;
    
    /// 中间件名称
    fn name(&self) -> &str;
    
    /// 中间件优先级（数字越小优先级越高）
    fn priority(&self) -> i32 {
        100
    }
}

/// 函数式中间件包装器
pub struct FunctionMiddleware {
    name: String,
    priority: i32,
    handler: MiddlewareHandler,
}

impl FunctionMiddleware {
    pub fn new<F>(name: String, handler: F) -> Self 
    where
        F: Fn(&mut MiddlewareContext, NextFunction) -> HushResult<MiddlewareResult> + Send + Sync + 'static,
    {
        Self {
            name,
            priority: 100,
            handler: Box::new(handler),
        }
    }
    
    pub fn with_priority<F>(name: String, priority: i32, handler: F) -> Self 
    where
        F: Fn(&mut MiddlewareContext, NextFunction) -> HushResult<MiddlewareResult> + Send + Sync + 'static,
    {
        Self {
            name,
            priority,
            handler: Box::new(handler),
        }
    }
}

impl Middleware for FunctionMiddleware {
    fn process(&self, context: &mut MiddlewareContext, next: NextFunction) -> HushResult<MiddlewareResult> {
        (self.handler)(context, next)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn priority(&self) -> i32 {
        self.priority
    }
}

/// 中间件链管理器
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }
    
    /// 添加中间件
    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middlewares.push(Arc::new(middleware));
        // 按优先级排序
        self.middlewares.sort_by_key(|m| m.priority());
    }
    
    /// 添加函数式中间件
    pub fn add_function<F>(&mut self, name: String, handler: F)
    where
        F: Fn(&mut MiddlewareContext, NextFunction) -> HushResult<MiddlewareResult> + Send + Sync + 'static,
    {
        let middleware = FunctionMiddleware::new(name, handler);
        self.add(middleware);
    }
    
    /// 添加带优先级的函数式中间件
    pub fn add_function_with_priority<F>(&mut self, name: String, priority: i32, handler: F)
    where
        F: Fn(&mut MiddlewareContext, NextFunction) -> HushResult<MiddlewareResult> + Send + Sync + 'static,
    {
        let middleware = FunctionMiddleware::with_priority(name, priority, handler);
        self.add(middleware);
    }
    
    /// 执行中间件链
    pub fn execute(&self, mut context: MiddlewareContext) -> HushResult<ResponseContext> {
        if self.middlewares.is_empty() {
            return Err(HushError::InternalError("No middlewares or handler defined".to_string()));
        }
        
        self.execute_middleware_at_index(0, &mut context)
    }
    
    /// 递归执行指定索引的中间件
    fn execute_middleware_at_index(&self, index: usize, context: &mut MiddlewareContext) -> HushResult<ResponseContext> {
        if index >= self.middlewares.len() {
            // 所有中间件都执行完毕，返回默认响应
            return Ok(ResponseContext::with_text(HttpStatus::Ok, "OK"));
        }
        
        let middleware = self.middlewares[index].clone();
        let next_index = index + 1;
        let middlewares = self.middlewares.clone();
        
        // 创建 next 函数
        let next: NextFunction = Box::new(move |ctx: &mut MiddlewareContext| -> HushResult<MiddlewareResult> {
            if next_index >= middlewares.len() {
                // 没有更多中间件，返回继续
                return Ok(MiddlewareResult::Continue);
            }
            
            let next_middleware = middlewares[next_index].clone();
            let next_next_index = next_index + 1;
            let middlewares_clone = middlewares.clone();
            
            // 递归创建下一个 next 函数
            let recursive_next: NextFunction = Box::new(move |ctx: &mut MiddlewareContext| -> HushResult<MiddlewareResult> {
                if next_next_index >= middlewares_clone.len() {
                    return Ok(MiddlewareResult::Continue);
                }
                
                // 简化处理，直接返回 Continue
                Ok(MiddlewareResult::Continue)
            });
            
            next_middleware.process(ctx, recursive_next)
        });
        
        // 执行当前中间件
        match middleware.process(context, next)? {
            MiddlewareResult::Continue => {
                // 继续执行下一个中间件
                self.execute_middleware_at_index(next_index, context)
            }
            MiddlewareResult::Response(response) => {
                // 中间件返回了响应，直接返回
                Ok(response)
            }
            MiddlewareResult::Error(error) => {
                // 中间件执行出错
                Err(error)
            }
        }
    }
    
    /// 获取中间件数量
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }
    
    /// 获取中间件名称列表
    pub fn middleware_names(&self) -> Vec<String> {
        self.middlewares.iter().map(|m| m.name().to_string()).collect()
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{HttpMethod, HttpStatus};
    
    #[test]
    fn test_middleware_context() {
        let request = RequestContext::new(HttpMethod::GET, "/test".to_string());
        let mut context = MiddlewareContext::new(request);
        
        context.set_data("key1".to_string(), "value1".to_string());
        assert_eq!(context.get_data("key1"), Some(&"value1".to_string()));
        
        let removed = context.remove_data("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(context.get_data("key1"), None);
    }
    
    #[test]
    fn test_function_middleware() {
        let middleware = FunctionMiddleware::new(
            "test_middleware".to_string(),
            |_ctx, next| {
                // 调用下一个中间件
                next(_ctx)
            }
        );
        
        assert_eq!(middleware.name(), "test_middleware");
        assert_eq!(middleware.priority(), 100);
    }
    
    #[test]
    fn test_middleware_chain_basic() {
        let mut chain = MiddlewareChain::new();
        
        // 添加测试中间件
        chain.add_function("test1".to_string(), |ctx, next| {
            ctx.set_data("test1".to_string(), "executed".to_string());
            next(ctx)
        });
        
        chain.add_function("test2".to_string(), |ctx, next| {
            ctx.set_data("test2".to_string(), "executed".to_string());
            next(ctx)
        });
        
        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
        
        let names = chain.middleware_names();
        assert!(names.contains(&"test1".to_string()));
        assert!(names.contains(&"test2".to_string()));
    }
    
    #[test]
    fn test_middleware_priority() {
        let mut chain = MiddlewareChain::new();
        
        // 添加不同优先级的中间件
        chain.add_function_with_priority("low_priority".to_string(), 200, |_ctx, next| {
            next(_ctx)
        });
        
        chain.add_function_with_priority("high_priority".to_string(), 50, |_ctx, next| {
            next(_ctx)
        });
        
        let names = chain.middleware_names();
        // 高优先级的中间件应该排在前面
        assert_eq!(names[0], "high_priority");
        assert_eq!(names[1], "low_priority");
    }
}