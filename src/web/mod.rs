// ============================================================================
// Web 模块 - HTTP 服务器和路由管理 | Web Module - HTTP Server and Route Management
// ============================================================================

pub mod server;
pub mod router;
pub mod handler;
pub mod middleware_demo;

#[cfg(test)]
pub mod middleware_integration_tests;

#[cfg(test)]
pub mod end_to_end_middleware_tests;

// 重新导出核心类型和函数
pub use server::{WebServer, WebServerConfig};
pub use router::{Router, RouteMatcher};
pub use handler::{RequestHandler, ResponseBuilder};