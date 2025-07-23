// ============================================================================
// 中间件系统模块 | Middleware System Module
// ============================================================================

pub mod core;
pub mod ffi;
pub mod builtin;

// 重新导出核心类型和函数
pub use core::{
    Middleware, MiddlewareChain, MiddlewareResult, MiddlewareContext,
    MiddlewareHandler, NextFunction
};
pub use ffi::{
    HushMiddleware, HushMiddlewareHandler, HushRequestContext,
    hush_middleware_new, hush_middleware_add, hush_middleware_free,
    hush_middleware_add_cors, hush_middleware_add_auth_jwt, hush_middleware_add_logger
};
pub use builtin::{CorsMiddleware, LoggerMiddleware, AuthMiddleware};