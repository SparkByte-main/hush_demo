# Hush Demo | Hush 演示项目 | Hushデモプロジェクト

**Hush** is a revolutionary web framework that combines Zig's performance with Rust's safety through FFI. Build high-performance web applications with Zig while leveraging Rust's mature ecosystem.

**Hush** 是一个革命性的 Web 框架，通过 FFI 将 Zig 的性能与 Rust 的安全性相结合。使用 Zig 构建高性能 Web 应用程序，同时利用 Rust 成熟的生态系统。

**Hush** は、FFIを通じてZigのパフォーマンスとRustの安全性を組み合わせた革新的なWebフレームワークです。Rustの成熟したエコシステムを活用しながら、Zigで高性能なWebアプリケーションを構築できます。

## ✨ Features | 特性 | 機能

- 🚀 **HTTP Methods Support** | **HTTP 方法支持** | **HTTPメソッドサポート**: GET, POST, PUT, DELETE and more | GET、POST、PUT、DELETE 等 | GET、POST、PUT、DELETE など
- 📦 **Request Body Processing** | **请求体处理** | **リクエストボディ処理**: Full POST data and JSON support | 完整的 POST 数据和 JSON 支持 | 完全なPOSTデータとJSONサポート
- 🛣️ **Dynamic Routing** | **动态路由** | **動的ルーティング**: Flexible route registration system | 灵活的路由注册系统 | 柔軟なルート登録システム
- ⚡ **High Performance** | **高性能** | **高性能**: Powered by Rust's actix-web framework | 基于 Rust 的 actix-web 框架 | RustのActix-webフレームワークを使用
- 🔒 **Memory Safe** | **内存安全** | **メモリ安全**: Rust's ownership model ensures safety | Rust 的所有权模型确保安全性 | Rustの所有権モデルが安全性を保証
- 🌐 **Cross-Platform** | **跨平台** | **クロスプラットフォーム**: Works on Windows, macOS, and Linux | 支持 Windows、macOS 和 Linux | Windows、macOS、Linuxで動作

## 🎯 Current Status | 当前状态 | 現在の状況

✅ **Fully Functional Web Framework** | **完全功能的 Web 框架** | **完全に機能するWebフレームワーク**

This is a **complete web framework** that demonstrates how to build modern web applications using Zig for business logic and Rust for the underlying web infrastructure.

这是一个**完整的 Web 框架**，演示如何使用 Zig 编写业务逻辑，使用 Rust 构建底层 Web 基础设施来构建现代 Web 应用程序。

これは**完全なWebフレームワーク**で、ビジネスロジックにZig、基盤となるWebインフラストラクチャにRustを使用して、モダンなWebアプリケーションを構築する方法を実演しています。

## 📁 Project Structure | 项目结构 | プロジェクト構造

```
hush_demo/
├── src/
│   ├── lib.rs              # Rust web framework core | Rust Web 框架核心 | Rust Webフレームワークコア
│   └── main.rs             # Rust main (unused) | Rust 主程序（未使用） | Rustメイン（未使用）
├── zig-test/
│   ├── main.zig            # Basic FFI demo | 基本 FFI 演示 | 基本FFIデモ
│   └── web_test.zig        # Web application | Web 应用程序 | Webアプリケーション
├── build.zig               # Zig build configuration | Zig 构建配置 | Zigビルド設定
├── build.sh                # One-click build script | 一键构建脚本 | ワンクリックビルドスクリプト
├── Cargo.toml              # Rust dependencies | Rust 依赖配置 | Rust依存関係
└── README.md               # This documentation | 本文档 | このドキュメント
```

## Quick Start | 快速开始

### Prerequisites | 前置要求

- **Rust** (latest stable) | **Rust**（最新稳定版）
- **Zig** (0.11.0 or later) | **Zig**（0.11.0 或更高版本）
- **Cargo** (comes with Rust) | **Cargo**（随 Rust 安装）

### Installation | 安装

1. **Clone the repository | 克隆仓库**
   ```bash
   git clone <repository-url>
   cd hush_demo
   ```

2. **Make build script executable | 使构建脚本可执行**
   ```bash
   chmod +x build.sh
   ```

### Building and Running | 构建和运行 | ビルドと実行

#### One-Click Build | 一键构建 | ワンクリックビルド

```bash
./build.sh
./web_test
```

#### Manual Build | 手动构建 | 手動ビルド

```bash
# Build Rust web framework | 构建 Rust Web 框架 | Rust Webフレームワークをビルド
cargo build --lib

# Build Zig web application | 构建 Zig Web 应用 | Zig Webアプリケーションをビルド
zig build-exe zig-test/web_test.zig -lhush_demo -L./target/debug

# Start the web server | 启动 Web 服务器 | Webサーバーを起動
./web_test
```

### Expected Output | 预期输出 | 期待される出力

```
Creating Zig web application with Rust framework...
Registering routes...
Starting server on http://127.0.0.1:8080
Routes available:
  GET/POST / - Hello page (shows HTTP method and POST data)
  GET/POST /about - About page (shows HTTP method and POST data)
  GET /web_test_hello - Test page (shows HTTP method)
  POST /api/users - API endpoint (accepts JSON data)
Press Ctrl+C to stop
Starting web framework server on port 8080
```

## 🧪 Testing the API | 测试 API | APIのテスト

Once the server is running, you can test all endpoints:

服务器运行后，您可以测试所有端点：

サーバーが実行されたら、すべてのエンドポイントをテストできます：

### Browser Testing | 浏览器测试 | ブラウザテスト

Visit these URLs in your browser:

在浏览器中访问这些 URL：

ブラウザでこれらのURLにアクセス：

- `http://127.0.0.1:8080/` - Hello page | 主页 | ホームページ
- `http://127.0.0.1:8080/about` - About page | 关于页面 | Aboutページ
- `http://127.0.0.1:8080/web_test_hello` - Test page | 测试页面 | テストページ

### cURL Testing | cURL 测试 | cURLテスト

```bash
# GET requests | GET 请求 | GETリクエスト
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/about

# POST requests with data | 带数据的 POST 请求 | データ付きPOSTリクエスト
curl -X POST http://127.0.0.1:8080/ -d "Hello World"
curl -X POST http://127.0.0.1:8080/about -d "username=admin&password=123"

# JSON API endpoint | JSON API 端点 | JSON APIエンドポイント
curl -X POST http://127.0.0.1:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com","age":25}'
```

### Expected Responses | 预期响应 | 期待されるレスポンス

**GET /** response:
```
Hello from Zig! Method: GET
```

**POST /** with data response:
```
Hello from Zig! Method: POST, POST Data: Hello World
```

**POST /api/users** with JSON response:
```json
{"status":"success","message":"User created","method":"POST","received_data":"{\"name\":\"John\",\"email\":\"john@example.com\",\"age\":25}"}
```

## 🏗️ Architecture | 架构 | アーキテクチャ

This framework demonstrates a **layered architecture** where:

该框架展示了一个**分层架构**，其中：

このフレームワークは**レイヤードアーキテクチャ**を実演しています：

1. **Rust Framework Layer | Rust 框架层 | Rustフレームワーク層**:
   - HTTP server management (actix-web) | HTTP 服务器管理 (actix-web) | HTTPサーバー管理 (actix-web)
   - Request/response processing | 请求/响应处理 | リクエスト/レスポンス処理
   - Route dispatching | 路由分发 | ルートディスパッチ
   - Memory management | 内存管理 | メモリ管理

2. **FFI Interface Layer | FFI 接口层 | FFIインターフェース層**:
   - C-compatible function signatures | C 兼容函数签名 | C互換関数シグネチャ
   - Data marshalling between languages | 语言间数据编组 | 言語間データマーシャリング
   - Safe memory handling | 安全内存处理 | 安全なメモリ処理

3. **Zig Application Layer | Zig 应用层 | Zigアプリケーション層**:
   - Business logic implementation | 业务逻辑实现 | ビジネスロジック実装
   - Route handler functions | 路由处理函数 | ルートハンドラ関数
   - JSON processing and responses | JSON 处理和响应 | JSON処理とレスポンス

## 🔧 Adding New Routes | 添加新路由 | 新しいルートの追加

To add new web endpoints to your application:

要为您的应用程序添加新的 Web 端点：

アプリケーションに新しいWebエンドポイントを追加するには：

1. **Create a handler function in Zig | 在 Zig 中创建处理函数 | Zigでハンドラ関数を作成**:
   ```zig
   export fn my_api_handler(method: [*:0]const u8, path: [*:0]const u8, body: [*:0]const u8) callconv(.C) [*:0]const u8 {
       const method_str = std.mem.span(method);
       const body_str = std.mem.span(body);
       
       const allocator = std.heap.c_allocator;
       const response = std.fmt.allocPrintZ(allocator, 
           "{{\"endpoint\":\"my-api\",\"method\":\"{s}\",\"data\":\"{s}\"}}", 
           .{method_str, body_str}
       ) catch {
           return "Error: Memory allocation failed".ptr;
       };
       return response.ptr;
   }
   ```

2. **Register the route | 注册路由 | ルートを登録**:
   ```zig
   web_server_add_route(server, "POST", "/api/my-endpoint", my_api_handler);
   ```

3. **Rebuild and test | 重新构建和测试 | 再ビルドとテスト**:
   ```bash
   ./build.sh
   ./web_test
   curl -X POST http://127.0.0.1:8080/api/my-endpoint -d '{"test":"data"}'
   ```

## 🔬 Technical Details | 技术细节 | 技術詳細

- **🔗 FFI Safety | FFI 安全性 | FFI安全性**: Uses C ABI for seamless cross-language compatibility | 使用 C ABI 实现无缝跨语言兼容性 | シームレスなクロス言語互換性のためにC ABIを使用
- **🧠 Memory Management | 内存管理 | メモリ管理**: Safe memory handling with Rust's ownership model | 使用 Rust 所有权模型进行安全内存处理 | Rustの所有権モデルによる安全なメモリ処理
- **⚙️ Build System | 构建系统 | ビルドシステム**: Automated linking and cross-platform support | 自动链接和跨平台支持 | 自動リンクとクロスプラットフォームサポート
- **🌍 Cross-Platform | 跨平台 | クロスプラットフォーム**: Tested on Windows, macOS, and Linux | 在 Windows、macOS 和 Linux 上测试 | Windows、macOS、Linuxでテスト済み
- **📊 Performance | 性能 | パフォーマンス**: Leverages actix-web's async runtime for high throughput | 利用 actix-web 的异步运行时实现高吞吐量 | 高スループットのためにactix-webの非同期ランタイムを活用
- **🔒 Type Safety | 类型安全 | 型安全性**: Compile-time guarantees from both Rust and Zig | Rust 和 Zig 的编译时保证 | RustとZigの両方からのコンパイル時保証

## 🤝 Contributing | 贡献 | 貢献

We welcome contributions to make this framework even better! Here's how you can help:

我们欢迎贡献，让这个框架变得更好！您可以这样帮助：

このフレームワークをより良くするための貢献を歓迎します！以下の方法で協力できます：

- 🐛 **Report bugs** | **报告错误** | **バグ報告**: Found an issue? Let us know! | 发现问题？告诉我们！ | 問題を発見しましたか？お知らせください！
- 💡 **Suggest features** | **建议功能** | **機能提案**: Have ideas for improvements? | 有改进想法？ | 改善のアイデアはありますか？
- 📝 **Improve documentation** | **改进文档** | **ドキュメント改善**: Help make our docs clearer | 帮助让我们的文档更清晰 | ドキュメントをより明確にするのを手伝ってください
- 🔧 **Submit pull requests** | **提交拉取请求** | **プルリクエスト提出**: Code contributions welcome! | 欢迎代码贡献！ | コード貢献を歓迎します！

## 📄 License | 许可证 | ライセンス

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

このプロジェクトはMITライセンスの下でライセンスされています - 詳細は[LICENSE](LICENSE)ファイルをご覧ください。

## 🗺️ Roadmap | 路线图 | ロードマップ

### ✅ Completed | 已完成 | 完了

- ✅ **HTTP Methods Support** | **HTTP 方法支持** | **HTTPメソッドサポート**: GET, POST, PUT, DELETE | GET、POST、PUT、DELETE | GET、POST、PUT、DELETE
- ✅ **Request Body Processing** | **请求体处理** | **リクエストボディ処理**: Full POST data support | 完整的 POST 数据支持 | 完全なPOSTデータサポート
- ✅ **Dynamic Routing System** | **动态路由系统** | **動的ルーティングシステム**: Flexible route registration | 灵活的路由注册 | 柔軟なルート登録
- ✅ **JSON API Support** | **JSON API 支持** | **JSON APIサポート**: Modern API endpoints | 现代化 API 端点 | モダンなAPIエンドポイント
- ✅ **Cross-Platform Build** | **跨平台构建** | **クロスプラットフォームビルド**: Windows, macOS, Linux | Windows、macOS、Linux | Windows、macOS、Linux
- ✅ **Comprehensive Documentation** | **完整文档** | **包括的なドキュメント**: Multi-language comments | 多语言注释 | 多言語コメント

### 🚧 In Progress | 进行中 | 進行中

- 🔄 **Performance Optimization** | **性能优化** | **パフォーマンス最適化**: Benchmarking and tuning | 基准测试和调优 | ベンチマークと調整
- 🔄 **Error Handling** | **错误处理** | **エラーハンドリング**: Better error responses | 更好的错误响应 | より良いエラーレスポンス

### 📋 Planned | 计划中 | 計画中

- 🎯 **Middleware Support** | **中间件支持** | **ミドルウェアサポート**: Authentication, logging, CORS | 认证、日志、CORS | 認証、ログ、CORS
- 🎯 **Database Integration** | **数据库集成** | **データベース統合**: SQL and NoSQL support | SQL 和 NoSQL 支持 | SQLとNoSQLサポート
- 🎯 **WebSocket Support** | **WebSocket 支持** | **WebSocketサポート**: Real-time communication | 实时通信 | リアルタイム通信
- 🎯 **Template Engine** | **模板引擎** | **テンプレートエンジン**: HTML rendering | HTML 渲染 | HTMLレンダリング
- 🎯 **Static File Serving** | **静态文件服务** | **静的ファイル配信**: CSS, JS, images | CSS、JS、图片 | CSS、JS、画像
- 🎯 **Testing Framework** | **测试框架** | **テストフレームワーク**: Unit and integration tests | 单元和集成测试 | ユニットと統合テスト

---

**Happy coding! | 编程愉快！ | ハッピーコーディング！** 🦀⚡

*Built with ❤️ using Zig + Rust | 使用 Zig + Rust 用心构建 | Zig + Rustで❤️を込めて構築*