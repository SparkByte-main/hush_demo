// ============================================================================
// 依赖导入 | Dependencies Import | 依存関係のインポート
// ============================================================================

// actix-web: 高性能的 Rust web 框架 | actix-web: High-performance Rust web framework | actix-web: 高性能なRust webフレームワーク
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, web::Bytes};

// 标准库导入 | Standard library imports | 標準ライブラリのインポート
use std::collections::HashMap;           // 哈希映射，用于存储路由 | HashMap for storing routes | ルートを格納するためのHashMap
use std::ffi::{c_char, CStr, CString};   // C 语言 FFI 类型 | C language FFI types | C言語FFI型
use std::sync::{Arc, Mutex};             // 线程安全的共享数据结构 | Thread-safe shared data structures | スレッドセーフな共有データ構造
use std::thread;                         // 线程操作 | Thread operations | スレッド操作

// ============================================================================
// 示例函数：基本的 FFI 演示 | Example Function: Basic FFI Demonstration | サンプル関数：基本的なFFIデモンストレーション
// ============================================================================

// #[unsafe(no_mangle)] 属性详解 | #[unsafe(no_mangle)] attribute explanation | #[unsafe(no_mangle)] 属性の詳細説明:
// - no_mangle: 告诉编译器不要改变函数名，保持原始名称 | no_mangle: Tell compiler not to change function name, keep original name | no_mangle: コンパイラに関数名を変更せず、元の名前を保持するよう指示
//   这样其他语言（如 Zig）可以通过确切的函数名找到这个函数 | So other languages (like Zig) can find this function by exact name | これにより他の言語（Zigなど）が正確な関数名でこの関数を見つけることができる
// - unsafe(): 在新版本 Rust 中，no_mangle 被认为是不安全的属性 | unsafe(): In newer Rust versions, no_mangle is considered unsafe attribute | unsafe(): 新しいバージョンのRustでは、no_mangleは安全でない属性とみなされる
//   因为它可能导致符号冲突，所以需要用 unsafe() 包装 | Because it may cause symbol conflicts, needs to be wrapped with unsafe() | シンボルの競合を引き起こす可能性があるため、unsafe()でラップする必要がある
//
// 函数签名解释 | Function signature explanation | 関数シグネチャの説明:
// - pub: 使函数对外部可见 | pub: Make function visible to external code | pub: 関数を外部から見えるようにする
// - extern "C": 使用 C 调用约定，确保与其他语言的兼容性 | extern "C": Use C calling convention, ensure compatibility with other languages | extern "C": C呼び出し規約を使用し、他の言語との互換性を確保
// - -> *const c_char: 返回一个指向 C 字符串的常量指针 | -> *const c_char: Return a constant pointer to C string | -> *const c_char: C文字列への定数ポインタを返す
#[unsafe(no_mangle)]
pub extern "C" fn rust_hello_world() -> *const c_char {
    // 创建一个 Rust 字符串并转换为 C 风格的字符串 | Create a Rust string and convert to C-style string | Rust文字列を作成してC形式の文字列に変換
    // CString::new() 会在字符串末尾添加空终止符 '\0' | CString::new() adds null terminator '\0' at end of string | CString::new()は文字列の末尾にnull終端文字'\0'を追加
    // unwrap() 用于处理可能的错误（如字符串中包含空字节）| unwrap() handles possible errors (like null bytes in string) | unwrap()は可能なエラー（文字列内のnullバイトなど）を処理
    let hello = CString::new("Hello, World!").unwrap();

    // into_raw() 将 CString 转换为原始指针并转移所有权 | into_raw() converts CString to raw pointer and transfers ownership | into_raw()はCStringを生ポインタに変換し、所有権を移転
    // 内存管理注意事项 | Memory management notes | メモリ管理の注意事項:
    // 1. 返回的指针可以被 C/Zig 代码安全使用 | 1. Returned pointer can be safely used by C/Zig code | 1. 返されたポインタはC/Zigコードで安全に使用可能
    // 2. 内存不会被 Rust 自动释放 | 2. Memory won't be automatically freed by Rust | 2. メモリはRustによって自動的に解放されない
    // 3. 调用方负责释放内存（在这个简单示例中我们忽略了这点）| 3. Caller is responsible for freeing memory (ignored in this simple example) | 3. 呼び出し側がメモリの解放を担当（この簡単な例では無視）
    hello.into_raw()
}

// ============================================================================
// Web 框架核心数据结构 | Web Framework Core Data Structures | Webフレームワークのコアデータ構造
// ============================================================================

// Web 服务器结构体，包含路由映射表 | Web server struct containing route mapping table | ルートマッピングテーブルを含むWebサーバー構造体
pub struct WebServer {
    // 路由存储：使用 "METHOD:PATH" 格式作为键 | Route storage: using "METHOD:PATH" format as key | ルート保存："METHOD:PATH"形式をキーとして使用
    // 例如："GET:/", "POST:/users" | Examples: "GET:/", "POST:/users" | 例："GET:/", "POST:/users"
    // Arc<Mutex<>> 确保多线程安全访问 | Arc<Mutex<>> ensures thread-safe access | Arc<Mutex<>>はマルチスレッドセーフなアクセスを保証
    // 值是指向 Zig 处理函数的函数指针，现在接收三个参数：method, path, body | Value is function pointer to Zig handler function, now receives three parameters: method, path, body | 値はZigハンドラ関数への関数ポインタ、現在3つのパラメータを受け取る：method, path, body
    routes: Arc<Mutex<HashMap<String, extern "C" fn(*const c_char, *const c_char, *const c_char) -> *const c_char>>>,
}

// 全局服务器实例指针，用于在异步处理函数中访问路由 | Global server instance pointer for accessing routes in async handlers | 非同期ハンドラ関数でルートにアクセスするためのグローバルサーバーインスタンスポインタ
// 注意：使用全局状态不是最佳实践，但简化了 FFI 接口设计 | Note: Using global state is not best practice, but simplifies FFI interface design | 注意：グローバル状態の使用はベストプラクティスではないが、FFIインターフェース設計を簡素化
static mut GLOBAL_SERVER: Option<*mut WebServer> = None;

// ============================================================================
// Web 框架 FFI 接口函数 | Web Framework FFI Interface Functions | WebフレームワークFFIインターフェース関数
// ============================================================================

// 创建新的 web 服务器实例 | Create new web server instance | 新しいWebサーバーインスタンスを作成
// 返回值：指向 WebServer 结构体的原始指针 | Return: Raw pointer to WebServer struct | 戻り値：WebServer構造体への生ポインタ
// 注意：调用方负责最终调用 web_server_free() 释放内存 | Note: Caller is responsible for calling web_server_free() to release memory | 注意：呼び出し側は最終的にweb_server_free()を呼び出してメモリを解放する責任がある
#[unsafe(no_mangle)]
pub extern "C" fn web_server_new() -> *mut WebServer {
    // 在堆上创建 WebServer 实例 | Create WebServer instance on heap | ヒープ上にWebServerインスタンスを作成
    let server = Box::new(WebServer {
        routes: Arc::new(Mutex::new(HashMap::new())), // 初始化空的路由表 | Initialize empty route table | 空のルートテーブルを初期化
    });
    
    // 将 Box 转换为原始指针，转移所有权 | Convert Box to raw pointer, transfer ownership | Boxを生ポインタに変換し、所有権を移転
    let server_ptr = Box::into_raw(server);

    // 设置全局服务器引用，供异步处理函数使用 | Set global server reference for async handler functions | 非同期ハンドラ関数で使用するためのグローバルサーバー参照を設定
    unsafe {
        GLOBAL_SERVER = Some(server_ptr);
    }

    server_ptr
}

// 添加路由到服务器 | Add route to server | サーバーにルートを追加
// 参数说明 | Parameters | パラメータ:
// - server: 服务器实例指针 | server: Server instance pointer | server: サーバーインスタンスポインタ
// - method: HTTP 方法（如 "GET", "POST"）| method: HTTP method (e.g. "GET", "POST") | method: HTTPメソッド（例："GET", "POST"）
// - path: 路由路径（如 "/", "/users"）| path: Route path (e.g. "/", "/users") | path: ルートパス（例："/", "/users"）
// - handler: Zig 处理函数指针，现在接收 method, path, body 三个参数 | handler: Zig handler function pointer, now receives method, path, body parameters | handler: Zigハンドラ関数ポインタ、現在method, path, bodyの3つのパラメータを受け取る
#[unsafe(no_mangle)]
pub extern "C" fn web_server_add_route(
    server: *mut WebServer,
    method: *const c_char,
    path: *const c_char,
    handler: extern "C" fn(*const c_char, *const c_char, *const c_char) -> *const c_char,
) {
    // 参数有效性检查 | Parameter validity check | パラメータの有効性チェック
    if server.is_null() || method.is_null() || path.is_null() {
        return;
    }

    unsafe {
        // 将 C 字符串转换为 Rust 字符串 | Convert C strings to Rust strings | C文字列をRust文字列に変換
        let method_str = CStr::from_ptr(method).to_string_lossy().to_string();
        let path_str = CStr::from_ptr(path).to_string_lossy().to_string();
        
        // 创建路由键：格式为 "METHOD:PATH" | Create route key: format "METHOD:PATH" | ルートキーを作成：形式は"METHOD:PATH"
        let route_key = format!("{}:{}", method_str, path_str);
        let server_ref = &*server;

        // 获取路由表的互斥锁并插入新路由 | Acquire route table mutex lock and insert new route | ルートテーブルのミューテックスロックを取得し、新しいルートを挿入
        if let Ok(mut routes) = server_ref.routes.lock() {
            routes.insert(route_key, handler);
        }
    }
}

// ============================================================================
// HTTP 请求处理核心逻辑 | HTTP Request Processing Core Logic | HTTPリクエスト処理のコアロジック
// ============================================================================

// 通用路由处理函数，处理所有传入的 HTTP 请求 | Generic route handler for all incoming HTTP requests | すべての受信HTTPリクエストを処理する汎用ルートハンドラ
// 此函数由 actix-web 框架调用，负责路由分发和 Zig 处理函数调用 | Called by actix-web framework, responsible for route dispatching and Zig handler invocation | この関数はactix-webフレームワークによって呼び出され、ルートディスパッチとZigハンドラ関数の呼び出しを担当
async fn handle_request(req: HttpRequest, body: Bytes) -> HttpResponse {
    // 提取 HTTP 方法和路径 | Extract HTTP method and path | HTTPメソッドとパスを抽出
    let method = req.method().as_str();  // 如 "GET", "POST" | e.g. "GET", "POST" | 例："GET", "POST"
    let path = req.path();               // 如 "/", "/users" | e.g. "/", "/users" | 例："/", "/users"
    
    // 将请求体转换为字符串 | Convert request body to string | リクエストボディを文字列に変換
    let body_str = String::from_utf8_lossy(&body);
    
    // 构造路由键用于查找处理函数 | Construct route key for handler lookup | ハンドラ検索用のルートキーを構築
    let route_key = format!("{}:{}", method, path);

    unsafe {
        // 检查全局服务器实例是否存在 | Check if global server instance exists | グローバルサーバーインスタンスが存在するかチェック
        if let Some(server_ptr) = GLOBAL_SERVER {
            let server_ref = &*server_ptr;

            // 获取路由表的读锁 | Acquire read lock on route table | ルートテーブルの読み取りロックを取得
            if let Ok(routes) = server_ref.routes.lock() {
                // 查找匹配的路由处理函数 | Look for matching route handler | 一致するルートハンドラを検索
                if let Some(handler) = routes.get(&route_key) {
                    // 创建方法、路径和请求体的 C 字符串，传递给 Zig | Create C strings for method, path and body to pass to Zig | メソッド、パス、リクエストボディのC文字列を作成してZigに渡す
                    let method_cstr = CString::new(method).unwrap();
                    let path_cstr = CString::new(path).unwrap();
                    let body_cstr = CString::new(body_str.as_ref()).unwrap();
                    
                    // 调用 Zig 处理函数，传递方法、路径和请求体参数 | Call Zig handler function with method, path and body parameters | Zigハンドラ関数を呼び出し、メソッド、パス、ボディパラメータを渡す
                    let response_ptr = handler(method_cstr.as_ptr(), path_cstr.as_ptr(), body_cstr.as_ptr());

                    // 检查 Zig 函数是否返回有效响应 | Check if Zig function returned valid response | Zig関数が有効な応答を返したかチェック
                    if !response_ptr.is_null() {
                        let response_str = CStr::from_ptr(response_ptr).to_string_lossy();
                        return HttpResponse::Ok().body(response_str.to_string());
                    }
                }
            }
        }
    }

    // 如果没有找到匹配的路由，返回 404 | Return 404 if no matching route found | 一致するルートが見つからない場合、404を返す
    HttpResponse::NotFound().body("Route not found")
}

// 启动 web 服务器 | Start web server | Webサーバーを起動
// 参数说明 | Parameters | パラメータ:
// - server: 服务器实例指针 | server: Server instance pointer | server: サーバーインスタンスポインタ
// - port: 监听端口号 | port: Port number to listen on | port: リッスンするポート番号
// 注意：此函数在新线程中启动服务器，不会阻塞调用方 | Note: This function starts server in new thread, won't block caller | 注意：この関数は新しいスレッドでサーバーを起動し、呼び出し元をブロックしない
#[unsafe(no_mangle)]
pub extern "C" fn web_server_start(server: *mut WebServer, port: u16) {
    // 参数有效性检查 | Parameter validity check | パラメータの有効性チェック
    if server.is_null() {
        return;
    }

    // 在新线程中启动服务器，避免阻塞 Zig 主线程 | Start server in new thread to avoid blocking Zig main thread | 新しいスレッドでサーバーを起動し、Zigメインスレッドのブロックを回避
    thread::spawn(move || {
        // 创建 tokio 异步运行时 | Create tokio async runtime | tokio非同期ランタイムを作成
        let rt = tokio::runtime::Runtime::new().unwrap();

        // 在异步运行时中启动 actix-web 服务器 | Start actix-web server in async runtime | 非同期ランタイム内でactix-webサーバーを起動
        rt.block_on(async {
            println!("Starting web framework server on port {}", port);

            // 创建 HTTP 服务器实例 | Create HTTP server instance | HTTPサーバーインスタンスを作成
            // default_service: 将所有请求路由到 handle_request 函数 | default_service: Route all requests to handle_request function | default_service: すべてのリクエストをhandle_request関数にルーティング
            HttpServer::new(|| App::new().default_service(web::route().to(handle_request)))
                .bind(("127.0.0.1", port))           // 绑定到本地地址和指定端口 | Bind to localhost and specified port | ローカルアドレスと指定ポートにバインド
                .expect("Failed to bind server")     // 绑定失败时 panic | Panic if binding fails | バインドに失敗した場合panic
                .run()                               // 启动服务器 | Start server | サーバーを起動
                .await                               // 等待服务器运行 | Wait for server to run | サーバーの実行を待機
                .expect("Failed to run server");     // 运行失败时 panic | Panic if server fails to run | サーバーの実行に失敗した場合panic
        });
    });
}

// 释放服务器资源 | Free server resources
#[unsafe(no_mangle)]
pub extern "C" fn web_server_free(server: *mut WebServer) {
    if !server.is_null() {
        unsafe {
            let _ = Box::from_raw(server);
            GLOBAL_SERVER = None;
        }
    }
}

// 注意：在生产代码中，应该提供一个对应的释放函数 | Note: In production code, should provide corresponding free function
// 例如 | Example:
#[unsafe(no_mangle)]
pub extern "C" fn rust_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
