// 导入 Zig 标准库 | Import Zig standard library
const std = @import("std");

// 声明外部函数（来自 Rust 动态库）| Declare external function (from Rust dynamic library)
// extern: 告诉 Zig 这个函数在外部定义（不在当前文件中）| extern: Tell Zig this function is defined externally (not in current file)
// fn rust_hello_world(): 函数名必须与 Rust 中的函数名完全匹配 | fn rust_hello_world(): Function name must exactly match the one in Rust
// [*:0]const u8: Zig 中表示 C 风格字符串的类型 | [*:0]const u8: Type representing C-style strings in Zig
//   - [*:0]: 表示一个以 null 终止的指针 | [*:0]: Represents a null-terminated pointer
//   - const u8: 指向常量字节的指针 | const u8: Pointer to constant bytes
//   - 这等价于 C 中的 const char* | This is equivalent to const char* in C
extern fn rust_hello_world() [*:0]const u8;

// 主函数 | Main function
// !void: 表示函数可能返回错误或 void | !void: Indicates function may return error or void
pub fn main() !void {
    // 调用 Rust 函数获取字符串 | Call Rust function to get string
    // 这里发生的过程 | Process that happens here:
    // 1. Zig 通过动态链接调用 Rust 函数 | 1. Zig calls Rust function through dynamic linking
    // 2. Rust 函数返回一个指向 C 字符串的指针 | 2. Rust function returns a pointer to C string
    // 3. 该指针被存储在 hello_msg 变量中 | 3. The pointer is stored in hello_msg variable
    const hello_msg = rust_hello_world();
    
    // 打印从 Rust 获取的字符串 | Print the string obtained from Rust
    // std.debug.print: Zig 的调试打印函数 | std.debug.print: Zig's debug print function
    // "Rust says: {s}\n": 格式字符串 | "Rust says: {s}\n": Format string
    //   - {s}: 字符串占位符 | {s}: String placeholder
    //   - \n: 换行符 | \n: Newline character
    // .{hello_msg}: 参数元组，包含要打印的值 | .{hello_msg}: Parameter tuple containing values to print
    std.debug.print("Rust says: {s}\n", .{hello_msg});
    
    // 注意：在这个简单示例中，我们没有释放 Rust 分配的内存 | Note: In this simple example, we don't free the memory allocated by Rust
    // 在生产代码中，应该调用相应的释放函数来避免内存泄漏 | In production code, should call corresponding free function to avoid memory leaks
}