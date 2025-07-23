// ============================================================================
// Hush 框架核心模块 | Hush Framework Core Module
// ============================================================================

pub mod error;
pub mod ffi;
pub mod memory;
pub mod types;

// 重新导出核心类型和函数
pub use error::{HushError, HushResult, ErrorCode};
pub use ffi::{FFIResult, to_c_string, from_c_string, handle_ffi_result};
pub use memory::{MemoryManager, CStringWrapper};
pub use types::{RequestContext, ResponseContext};