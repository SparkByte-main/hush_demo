// 导入 FFI (Foreign Function Interface) 相关类型 | Import FFI (Foreign Function Interface) related types
// c_char: C 语言的 char 类型 | c_char: C language char type
// CString: Rust 中表示 C 风格字符串的类型 | CString: Type representing C-style strings in Rust
use std::ffi::{c_char, CString};

// #[unsafe(no_mangle)] 属性的作用 | Purpose of #[unsafe(no_mangle)] attribute:
// - no_mangle: 告诉编译器不要改变函数名，保持原始名称 | no_mangle: Tell compiler not to change function name, keep original name
//   这样其他语言（如 Zig）可以通过确切的函数名找到这个函数 | So other languages (like Zig) can find this function by exact name
// - unsafe(): 在新版本 Rust 中，no_mangle 被认为是不安全的属性 | unsafe(): In newer Rust versions, no_mangle is considered unsafe attribute
//   因为它可能导致符号冲突，所以需要用 unsafe() 包装 | Because it may cause symbol conflicts, needs to be wrapped with unsafe()
//
// pub: 使函数对外部可见 | pub: Make function visible to external code
// extern "C": 使用 C 调用约定，确保与其他语言的兼容性 | extern "C": Use C calling convention, ensure compatibility with other languages
// -> *const c_char: 返回一个指向 C 字符串的常量指针 | -> *const c_char: Return a constant pointer to C string
#[unsafe(no_mangle)]
pub extern "C" fn rust_hello_world() -> *const c_char {
    // 创建一个 Rust 字符串并转换为 C 风格的字符串 | Create a Rust string and convert to C-style string
    // CString::new() 会在字符串末尾添加空终止符 '\0' | CString::new() adds null terminator '\0' at end of string
    // unwrap() 用于处理可能的错误（如字符串中包含空字节）| unwrap() handles possible errors (like null bytes in string)
    let hello = CString::new("Hello, World!").unwrap();

    // into_raw() 将 CString 转换为原始指针并转移所有权 | into_raw() converts CString to raw pointer and transfers ownership
    // 这意味着 | This means:
    // 1. 返回的指针可以被 C/Zig 代码安全使用 | 1. Returned pointer can be safely used by C/Zig code
    // 2. 内存不会被 Rust 自动释放 | 2. Memory won't be automatically freed by Rust
    // 3. 调用方负责释放内存（在这个简单示例中我们忽略了这点）| 3. Caller is responsible for freeing memory (ignored in this simple example)
    hello.into_raw()
}

// 注意：在生产代码中，应该提供一个对应的释放函数 | Note: In production code, should provide corresponding free function
// 例如 | Example:
// #[unsafe(no_mangle)]
// pub extern "C" fn rust_free_string(ptr: *mut c_char) {
//     if !ptr.is_null() {
//         unsafe {
//             let _ = CString::from_raw(ptr);
//         }
//     }
// }
