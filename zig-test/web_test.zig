// ============================================================================
// Zig Web 应用程序 - 使用 Rust actix-web 框架 | Zig Web Application - Using Rust actix-web Framework | Zig Webアプリケーション - Rust actix-webフレームワークを使用
// ============================================================================

// 导入 Zig 标准库 | Import Zig standard library | Zig標準ライブラリをインポート
const std = @import("std");

// ============================================================================
// 外部函数声明 - Rust Web 框架 FFI 接口 | External Function Declarations - Rust Web Framework FFI Interface | 外部関数宣言 - Rust WebフレームワークFFIインターフェース
// ============================================================================

// 创建新的 web 服务器实例 | Create new web server instance | 新しいWebサーバーインスタンスを作成
// 返回值：不透明指针，指向 Rust 端的 WebServer 结构体 | Return: Opaque pointer to WebServer struct on Rust side | 戻り値：Rust側のWebServer構造体を指す不透明ポインタ
extern fn web_server_new() ?*anyopaque;

// 添加路由到服务器 | Add route to server | サーバーにルートを追加
// 参数说明 | Parameters | パラメータ:
// - server: 服务器实例指针 | server: Server instance pointer | server: サーバーインスタンスポインタ
// - method: HTTP 方法字符串（如 "GET", "POST"）| method: HTTP method string (e.g. "GET", "POST") | method: HTTPメソッド文字列（例："GET", "POST"）
// - path: 路由路径字符串（如 "/", "/users"）| path: Route path string (e.g. "/", "/users") | path: ルートパス文字列（例："/", "/users"）
// - handler: Zig 处理函数指针，现在接收 method, path, body 三个参数 | handler: Zig handler function pointer, now receives method, path, body parameters | handler: Zigハンドラ関数ポインタ、現在method, path, bodyの3つのパラメータを受け取る
extern fn web_server_add_route(server: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, handler: *const fn ([*:0]const u8, [*:0]const u8, [*:0]const u8) callconv(.C) [*:0]const u8) void;

// 启动 web 服务器 | Start web server | Webサーバーを起動
// 参数说明 | Parameters | パラメータ:
// - server: 服务器实例指针 | server: Server instance pointer | server: サーバーインスタンスポインタ
// - port: 监听端口号 | port: Port number to listen on | port: リッスンするポート番号
extern fn web_server_start(server: ?*anyopaque, port: u16) void;

// 释放服务器资源 | Free server resources | サーバーリソースを解放
// 参数说明 | Parameters | パラメータ:
// - server: 要释放的服务器实例指针 | server: Server instance pointer to free | server: 解放するサーバーインスタンスポインタ
extern fn web_server_free(server: ?*anyopaque) void;

// 释放 Rust 分配的字符串内存 | Free string memory allocated by Rust | Rustによって割り当てられた文字列メモリを解放
// 参数说明 | Parameters | パラメータ:
// - ptr: 要释放的字符串指针 | ptr: String pointer to free | ptr: 解放する文字列ポインタ
extern fn rust_free_string(ptr: [*:0]u8) void;

// ============================================================================
// 路由处理函数 - 业务逻辑实现 | Route Handler Functions - Business Logic Implementation | ルートハンドラ関数 - ビジネスロジック実装
// ============================================================================

// 根路径处理函数：处理 "/" 路径的 GET 和 POST 请求 | Root path handler: handles GET and POST requests for "/" path | ルートパスハンドラ："/"パスのGETおよびPOSTリクエストを処理
// 函数签名说明 | Function signature explanation | 関数シグネチャの説明:
// - export: 导出函数供 Rust 通过 FFI 调用 | export: Export function for Rust to call via FFI | export: RustがFFI経由で呼び出すために関数をエクスポート
// - callconv(.C): 使用 C 调用约定，确保与 Rust 兼容 | callconv(.C): Use C calling convention for Rust compatibility | callconv(.C): Rustとの互換性を確保するためC呼び出し規約を使用
// - 参数 method: HTTP 方法字符串指针 | Parameter method: HTTP method string pointer | パラメータ method: HTTPメソッド文字列ポインタ
// - 参数 path: 请求路径字符串指针 | Parameter path: Request path string pointer | パラメータ path: リクエストパス文字列ポインタ
// - 参数 body: 请求体字符串指针（POST 数据）| Parameter body: Request body string pointer (POST data) | パラメータ body: リクエストボディ文字列ポインタ（POSTデータ）
// - 返回值: 响应内容的字符串指针 | Return: Response content string pointer | 戻り値: レスポンス内容の文字列ポインタ
export fn hello_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 暂时不使用请求路径参数 | Temporarily not using request path parameter | 一時的にリクエストパスパラメータを使用しない

    // 将 C 字符串转换为 Zig 字符串切片 | Convert C strings to Zig string slices | C文字列をZig文字列スライスに変換
    const method_str = std.mem.span(method);
    const body_str = std.mem.span(body);

    // 使用 C 分配器创建响应字符串 | Create response string using C allocator | Cアロケータを使用してレスポンス文字列を作成
    // 注意：使用 C 分配器确保内存可以被 Rust 端正确处理 | Note: Using C allocator ensures memory can be properly handled by Rust side | 注意：Cアロケータを使用することで、Rust側でメモリが適切に処理されることを保証
    const allocator = std.heap.c_allocator;

    // 根据 HTTP 方法返回不同的响应 | Return different responses based on HTTP method | HTTPメソッドに基づいて異なるレスポンスを返す
    const response = if (std.mem.eql(u8, method_str, "POST"))
        std.fmt.allocPrintZ(allocator, "Hello from Zig! Method: {s}, POST Data: {s}", .{ method_str, body_str })
    else
        std.fmt.allocPrintZ(allocator, "Hello from Zig! Method: {s}", .{method_str});

    return (response catch {
        // 内存分配失败时返回错误信息 | Return error message if memory allocation fails | メモリ割り当てに失敗した場合のエラーメッセージを返す
        return "Error: Memory allocation failed".ptr;
    }).ptr;
}

// About 页面处理函数：处理 "/about" 路径的请求 | About page handler: handles requests for "/about" path | Aboutページハンドラ："/about"パスのリクエストを処理
// 功能：返回包含 HTTP 方法和请求体信息的关于页面内容 | Function: Returns about page content with HTTP method and body information | 機能：HTTPメソッドとリクエストボディ情報を含むAboutページ内容を返す
export fn about_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 当前实现中不使用路径参数 | Path parameter not used in current implementation | 現在の実装ではパスパラメータを使用しない

    // 提取 HTTP 方法和请求体信息 | Extract HTTP method and body information | HTTPメソッドとリクエストボディ情報を抽出
    const method_str = std.mem.span(method);
    const body_str = std.mem.span(body);

    // 分配内存并格式化响应字符串 | Allocate memory and format response string | メモリを割り当ててレスポンス文字列をフォーマット
    const allocator = std.heap.c_allocator;
    const response = if (std.mem.eql(u8, method_str, "POST") and body_str.len > 0)
        std.fmt.allocPrintZ(allocator, "About page - Method: {s}, POST Data: {s} - This is a Zig web application using Rust framework!", .{ method_str, body_str })
    else
        std.fmt.allocPrintZ(allocator, "About page - Method: {s} - This is a Zig web application using Rust framework!", .{method_str});

    return (response catch {
        return "Error: Memory allocation failed".ptr;
    }).ptr;
}

// 测试处理函数：演示自定义路由处理 | Test handler function: demonstrates custom route handling | テストハンドラ関数：カスタムルート処理のデモンストレーション
// 功能：返回简单的测试响应，包含 HTTP 方法和请求体信息 | Function: Returns simple test response with HTTP method and body information | 機能：HTTPメソッドとリクエストボディ情報を含む簡単なテストレスポンスを返す
export fn web_test_hello_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 当前实现中不使用路径参数 | Path parameter not used in current implementation | 現在の実装ではパスパラメータを使用しない

    // 提取 HTTP 方法和请求体信息 | Extract HTTP method and body information | HTTPメソッドとリクエストボディ情報を抽出
    const method_str = std.mem.span(method);
    const body_str = std.mem.span(body);

    // 分配内存并格式化响应字符串 | Allocate memory and format response string | メモリを割り当ててレスポンス文字列をフォーマット
    const allocator = std.heap.c_allocator;
    const response = if (std.mem.eql(u8, method_str, "POST") and body_str.len > 0)
        std.fmt.allocPrintZ(allocator, "Hello from Zig web application! Method: {s}, POST Data: {s}", .{ method_str, body_str })
    else
        std.fmt.allocPrintZ(allocator, "Hello from Zig web application! Method: {s}", .{method_str});

    return (response catch {
        return "Error: Memory allocation failed".ptr;
    }).ptr;
}

// ============================================================================
// 应用程序入口点 | Application Entry Point | アプリケーションエントリーポイント
// ============================================================================

// 主函数：初始化并启动 Zig web 应用程序 | Main function: Initialize and start Zig web application | メイン関数：Zig webアプリケーションを初期化して起動
// 功能流程 | Function flow | 機能フロー:
// 1. 创建 web 服务器实例 | 1. Create web server instance | 1. Webサーバーインスタンスを作成
// 2. 注册路由和处理函数 | 2. Register routes and handler functions | 2. ルートとハンドラ関数を登録
// 3. 启动 HTTP 服务器 | 3. Start HTTP server | 3. HTTPサーバーを起動
// 4. 保持程序运行 | 4. Keep program running | 4. プログラムの実行を維持
pub fn main() !void {
    // 应用程序启动信息 | Application startup information | アプリケーション起動情報
    std.debug.print("Creating Zig web application with Rust framework...\n", .{});

    // ========================================================================
    // 第一步：创建 web 服务器实例 | Step 1: Create web server instance | ステップ1：Webサーバーインスタンスを作成
    // ========================================================================
    const server = web_server_new();
    if (server == null) {
        std.debug.print("Failed to create web server\n", .{});
        return;
    }

    // ========================================================================
    // 第二步：注册路由和处理函数 | Step 2: Register routes and handler functions | ステップ2：ルートとハンドラ関数を登録
    // ========================================================================
    std.debug.print("Registering routes...\n", .{});

    // 根路径路由：支持 GET 和 POST 方法 | Root path routes: support GET and POST methods | ルートパスルート：GETとPOSTメソッドをサポート
    web_server_add_route(server, "GET", "/", hello_handler);
    web_server_add_route(server, "POST", "/", hello_handler);

    // About 页面路由：支持 GET 和 POST 方法 | About page routes: support GET and POST methods | Aboutページルート：GETとPOSTメソッドをサポート
    web_server_add_route(server, "GET", "/about", about_handler);
    web_server_add_route(server, "POST", "/about", about_handler);

    // 测试路由：仅支持 GET 方法 | Test route: only support GET method | テストルート：GETメソッドのみサポート
    web_server_add_route(server, "GET", "/web_test_hello", web_test_hello_handler);

    // API 路由：专门处理 POST 请求，演示 JSON 数据处理 | API route: specifically handles POST requests, demonstrates JSON data processing | APIルート：POSTリクエストを専門的に処理、JSONデータ処理のデモンストレーション
    web_server_add_route(server, "POST", "/api/users", create_user_handler);

    // ========================================================================
    // 第三步：启动 HTTP 服务器 | Step 3: Start HTTP server | ステップ3：HTTPサーバーを起動
    // ========================================================================
    std.debug.print("Starting server on http://127.0.0.1:8080\n", .{});
    std.debug.print("Routes available:\n", .{});
    std.debug.print("  GET/POST / - Hello page (shows HTTP method and POST data)\n", .{});
    std.debug.print("  GET/POST /about - About page (shows HTTP method and POST data)\n", .{});
    std.debug.print("  GET /web_test_hello - Test page (shows HTTP method)\n", .{});
    std.debug.print("  POST /api/users - API endpoint (accepts JSON data)\n", .{});
    std.debug.print("Press Ctrl+C to stop\n", .{});

    // 启动服务器（在新线程中运行，不阻塞主线程）| Start server (runs in new thread, doesn't block main thread) | サーバーを起動（新しいスレッドで実行、メインスレッドをブロックしない）
    web_server_start(server, 8080);

    // ========================================================================
    // 第四步：保持程序运行 | Step 4: Keep program running | ステップ4：プログラムの実行を維持
    // ========================================================================
    // 无限循环保持主线程活跃，直到用户按 Ctrl+C 终止程序 | Infinite loop keeps main thread alive until user presses Ctrl+C | 無限ループでメインスレッドを活性化し、ユーザーがCtrl+Cを押すまでプログラムを終了させない
    while (true) {
        std.time.sleep(1000000000); // 睡眠 1 秒，减少 CPU 使用率 | Sleep for 1 second to reduce CPU usage | 1秒間スリープしてCPU使用率を削減
    }

    // 注意：在实际应用中，应该在程序退出时调用 web_server_free(server) 释放资源 | Note: In real applications, should call web_server_free(server) to release resources on program exit | 注意：実際のアプリケーションでは、プログラム終了時にweb_server_free(server)を呼び出してリソースを解放すべき
}

// API 处理函数：专门处理 POST 请求，演示 JSON 数据处理 | API handler function: specifically handles POST requests, demonstrates JSON data processing | APIハンドラ関数：POSTリクエストを専門的に処理、JSONデータ処理のデモンストレーション
// 功能：模拟创建用户的 API 接口，解析 JSON 数据并返回响应 | Function: Simulates user creation API endpoint, parses JSON data and returns response | 機能：ユーザー作成APIエンドポイントをシミュレート、JSONデータを解析してレスポンスを返す
export fn create_user_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = path; // 当前实现中不使用路径参数 | Path parameter not used in current implementation | 現在の実装ではパスパラメータを使用しない

    // 提取 HTTP 方法和请求体信息 | Extract HTTP method and body information | HTTPメソッドとリクエストボディ情報を抽出
    const method_str = std.mem.span(method);
    const body_str = std.mem.span(body);

    // 分配内存并格式化响应字符串 | Allocate memory and format response string | メモリを割り当ててレスポンス文字列をフォーマット
    const allocator = std.heap.c_allocator;

    // 检查是否为 POST 请求且有请求体数据 | Check if it's a POST request with body data | POSTリクエストでリクエストボディデータがあるかチェック
    if (std.mem.eql(u8, method_str, "POST") and body_str.len > 0) {
        // 模拟 JSON 响应：创建用户成功 | Simulate JSON response: user creation successful | JSONレスポンスをシミュレート：ユーザー作成成功
        const response = std.fmt.allocPrintZ(allocator, "{{\"status\":\"success\",\"message\":\"User created\",\"method\":\"{s}\",\"received_data\":\"{s}\"}}", .{ method_str, body_str }) catch {
            return "Error: Memory allocation failed".ptr;
        };
        return response.ptr;
    } else {
        // 返回错误响应：方法不允许或缺少数据 | Return error response: method not allowed or missing data | エラーレスポンスを返す：メソッドが許可されていないかデータが不足
        const response = std.fmt.allocPrintZ(allocator, "{{\"status\":\"error\",\"message\":\"POST method required with JSON data\",\"method\":\"{s}\"}}", .{method_str}) catch {
            return "Error: Memory allocation failed".ptr;
        };
        return response.ptr;
    }
}
