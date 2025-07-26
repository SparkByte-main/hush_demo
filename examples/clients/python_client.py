#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
============================================================================
Hush 框架客户端 Python 示例
============================================================================

这个文件展示了如何使用 Python 与 Hush 框架后端 API 进行交互。
包含了完整的错误处理、重试机制、性能监控等功能。

依赖:
    pip install requests

使用方法:
    python client_examples.py
"""

import json
import time
import logging
from typing import Dict, Any, Optional, List
from dataclasses import dataclass
from contextlib import contextmanager

try:
    import requests
    from requests.adapters import HTTPAdapter
    from requests.packages.urllib3.util.retry import Retry
except ImportError:
    print("❌ 请安装 requests 库: pip install requests")
    exit(1)

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


@dataclass
class ApiResponse:
    """API 响应数据类"""
    status_code: int
    data: Any
    headers: Dict[str, str]
    elapsed_ms: float


class HushApiError(Exception):
    """Hush API 异常类"""
    def __init__(self, message: str, status_code: int = None, response_data: Any = None):
        super().__init__(message)
        self.status_code = status_code
        self.response_data = response_data


class HushApiClient:
    """
    Hush API 客户端类
    提供与 Hush 框架后端 API 交互的方法
    """
    
    def __init__(self, base_url: str = "http://localhost:8080", timeout: int = 30):
        """
        初始化 API 客户端
        
        Args:
            base_url: API 基础 URL
            timeout: 请求超时时间（秒）
        """
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self.token = None
        
        # 创建会话并配置重试策略
        self.session = requests.Session()
        retry_strategy = Retry(
            total=3,
            backoff_factor=1,
            status_forcelist=[429, 500, 502, 503, 504],
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        self.session.mount("http://", adapter)
        self.session.mount("https://", adapter)
    
    def set_auth_token(self, token: str) -> None:
        """设置认证令牌"""
        self.token = token
        logger.info("🔐 已设置认证令牌")
    
    def _get_headers(self, additional_headers: Dict[str, str] = None) -> Dict[str, str]:
        """获取请求头"""
        headers = {
            'Content-Type': 'application/json',
            'User-Agent': 'HushApiClient/1.0.0 (Python)'
        }
        
        if self.token:
            headers['Authorization'] = f'Bearer {self.token}'
        
        if additional_headers:
            headers.update(additional_headers)
        
        return headers
    
    @contextmanager
    def _performance_monitor(self, operation: str):
        """性能监控上下文管理器"""
        start_time = time.time()
        try:
            yield
        finally:
            elapsed = (time.time() - start_time) * 1000
            logger.info(f"⏱️ {operation} 耗时: {elapsed:.2f}ms")
    
    def _make_request(
        self, 
        method: str, 
        endpoint: str, 
        data: Any = None, 
        headers: Dict[str, str] = None,
        params: Dict[str, str] = None
    ) -> ApiResponse:
        """
        发送 HTTP 请求
        
        Args:
            method: HTTP 方法
            endpoint: API 端点
            data: 请求数据
            headers: 额外的请求头
            params: URL 参数
            
        Returns:
            ApiResponse: 响应对象
            
        Raises:
            HushApiError: API 请求异常
        """
        url = f"{self.base_url}{endpoint}"
        request_headers = self._get_headers(headers)
        
        logger.info(f"🚀 发送请求: {method} {url}")
        
        try:
            with self._performance_monitor(f"{method} {endpoint}"):
                response = self.session.request(
                    method=method,
                    url=url,
                    json=data if data and method in ['POST', 'PUT', 'PATCH'] else None,
                    headers=request_headers,
                    params=params,
                    timeout=self.timeout
                )
            
            # 解析响应数据
            try:
                response_data = response.json()
            except json.JSONDecodeError:
                response_data = response.text
            
            api_response = ApiResponse(
                status_code=response.status_code,
                data=response_data,
                headers=dict(response.headers),
                elapsed_ms=response.elapsed.total_seconds() * 1000
            )
            
            # 检查响应状态
            if response.ok:
                logger.info(f"✅ 请求成功: {response.status_code}")
                return api_response
            else:
                error_msg = f"HTTP {response.status_code}: {response.reason}"
                logger.error(f"❌ 请求失败: {error_msg}")
                raise HushApiError(error_msg, response.status_code, response_data)
                
        except requests.exceptions.RequestException as e:
            logger.error(f"❌ 网络错误: {str(e)}")
            raise HushApiError(f"网络错误: {str(e)}")
    
    # ========================================================================
    # API 方法
    # ========================================================================
    
    def health_check(self) -> Dict[str, Any]:
        """健康检查"""
        response = self._make_request('GET', '/health')
        return response.data
    
    def get_user_info(self) -> str:
        """获取用户信息"""
        response = self._make_request('GET', '/user')
        return response.data
    
    def get_users(self) -> Dict[str, Any]:
        """获取用户列表（需要认证）"""
        response = self._make_request('GET', '/api/users')
        return response.data
    
    def create_user(self, user_data: Dict[str, Any]) -> Dict[str, Any]:
        """创建新用户（需要认证）"""
        response = self._make_request('POST', '/api/users', data=user_data)
        return response.data
    
    def get_admin_dashboard(self) -> Dict[str, Any]:
        """获取管理员仪表板（需要管理员权限）"""
        response = self._make_request('GET', '/admin/dashboard')
        return response.data
    
    def cors_preflight_check(self, endpoint: str, method: str = 'GET') -> ApiResponse:
        """发送 CORS 预检请求"""
        headers = {
            'Access-Control-Request-Method': method,
            'Access-Control-Request-Headers': 'Content-Type, Authorization'
        }
        return self._make_request('OPTIONS', endpoint, headers=headers)


# ============================================================================
# 使用示例
# ============================================================================

def basic_usage_example():
    """基本使用示例"""
    print("\n🎯 基本使用示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    try:
        # 1. 健康检查
        print("\n1️⃣ 健康检查:")
        health = client.health_check()
        print(f"健康状态: {health}")
        
        # 2. 获取用户信息
        print("\n2️⃣ 用户信息:")
        user_info = client.get_user_info()
        print(f"用户信息: {user_info}")
        
    except HushApiError as e:
        print(f"❌ API 错误: {e}")
        if e.response_data:
            print(f"错误详情: {e.response_data}")


def authentication_example():
    """认证示例"""
    print("\n🔐 认证示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    try:
        # 1. 尝试无认证访问受保护端点
        print("\n1️⃣ 无认证访问受保护端点:")
        try:
            users = client.get_users()
            print(f"用户列表: {users}")
        except HushApiError as e:
            print(f"预期的认证错误: {e.status_code} - {e.response_data}")
        
        # 2. 设置认证令牌并重试
        print("\n2️⃣ 使用认证令牌:")
        sample_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiMTIzNDUiLCJ1c2VybmFtZSI6InpoYW5nc2FuIiwicm9sZSI6InVzZXIiLCJleHAiOjE2NDA5OTUyMDB9.example_signature"
        
        client.set_auth_token(sample_token)
        
        try:
            users = client.get_users()
            print(f"用户列表: {users}")
        except HushApiError as e:
            print(f"认证失败（预期的，因为使用示例令牌）: {e.status_code} - {e.response_data}")
            
    except Exception as e:
        print(f"❌ 认证示例失败: {e}")


def cors_example():
    """CORS 示例"""
    print("\n🌐 CORS 示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    try:
        # 1. CORS 预检请求
        print("\n1️⃣ CORS 预检请求:")
        try:
            response = client.cors_preflight_check('/api/users', 'GET')
            print(f"CORS 预检成功: {response.status_code}")
            
            # 显示 CORS 相关头部
            cors_headers = {k: v for k, v in response.headers.items() 
                          if k.lower().startswith('access-control')}
            if cors_headers:
                print("CORS 头部:")
                for key, value in cors_headers.items():
                    print(f"  {key}: {value}")
            
        except HushApiError as e:
            print(f"CORS 预检结果: {e.status_code} - {e.response_data}")
        
        # 2. 带 Origin 头的请求
        print("\n2️⃣ 带 Origin 头的请求:")
        try:
            response = client._make_request(
                'GET', 
                '/api/users',
                headers={'Origin': 'https://app.example.com'}
            )
            print(f"跨域请求成功: {response.status_code}")
        except HushApiError as e:
            print(f"跨域请求结果: {e.status_code} - {e.response_data}")
            
    except Exception as e:
        print(f"❌ CORS 示例失败: {e}")


def post_request_example():
    """POST 请求示例"""
    print("\n📝 POST 请求示例")
    print("=" * 50)
    
    client = HushApiClient()
    client.set_auth_token("valid_jwt_token_here")
    
    try:
        # 创建新用户
        print("\n1️⃣ 创建新用户:")
        new_user = {
            "name": "张三",
            "email": "zhangsan@example.com"
        }
        
        try:
            result = client.create_user(new_user)
            print(f"用户创建成功: {result}")
        except HushApiError as e:
            print(f"用户创建结果: {e.status_code} - {e.response_data}")
            
    except Exception as e:
        print(f"❌ POST 请求示例失败: {e}")


def error_handling_example():
    """错误处理示例"""
    print("\n⚠️ 错误处理示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    def handle_api_error(error: HushApiError):
        """处理 API 错误"""
        error_messages = {
            401: "🔒 认证错误: 请提供有效的认证令牌",
            403: "🚫 权限错误: 访问被拒绝",
            429: "🚦 限流错误: 请求过于频繁，请稍后重试",
            500: "💥 服务器错误: 内部服务器错误"
        }
        
        message = error_messages.get(error.status_code, f"❓ 未知错误: {error.status_code}")
        print(message)
        
        if error.response_data:
            print(f"错误详情: {error.response_data}")
    
    try:
        # 1. 测试认证错误
        print("\n1️⃣ 测试认证错误:")
        try:
            client.get_users()
        except HushApiError as e:
            handle_api_error(e)
        
        # 2. 测试权限错误
        print("\n2️⃣ 测试权限错误:")
        try:
            client.get_admin_dashboard()
        except HushApiError as e:
            handle_api_error(e)
            
    except Exception as e:
        print(f"❌ 错误处理示例失败: {e}")


def batch_request_example():
    """批量请求示例"""
    print("\n📦 批量请求示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    try:
        print("\n1️⃣ 并行执行多个请求:")
        
        import concurrent.futures
        import threading
        
        def safe_request(func, name):
            """安全的请求包装器"""
            try:
                return name, func()
            except Exception as e:
                return name, f"错误: {str(e)}"
        
        # 定义要执行的请求
        requests_to_make = [
            (client.health_check, "健康检查"),
            (client.get_user_info, "用户信息"),
            (client.get_users, "用户列表")  # 这个会失败
        ]
        
        # 使用线程池并行执行
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
            futures = [
                executor.submit(safe_request, func, name) 
                for func, name in requests_to_make
            ]
            
            for future in concurrent.futures.as_completed(futures):
                name, result = future.result()
                print(f"{name}: {result}")
                
    except Exception as e:
        print(f"❌ 批量请求示例失败: {e}")


def retry_example():
    """重试机制示例"""
    print("\n🔄 重试机制示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    def retry_request(func, max_retries=3, delay=1.0):
        """重试请求函数"""
        for attempt in range(1, max_retries + 1):
            try:
                print(f"尝试 {attempt}/{max_retries}...")
                return func()
            except HushApiError as e:
                print(f"尝试 {attempt} 失败: {e}")
                
                if attempt == max_retries:
                    raise e
                
                # 指数退避
                wait_time = delay * (2 ** (attempt - 1))
                print(f"等待 {wait_time:.1f}s 后重试...")
                time.sleep(wait_time)
    
    try:
        print("\n1️⃣ 重试失败的请求:")
        
        try:
            result = retry_request(
                lambda: client.get_users(),  # 这个请求会失败
                max_retries=3,
                delay=0.5
            )
            print(f"重试成功: {result}")
        except HushApiError as e:
            print(f"所有重试都失败了: {e}")
            
    except Exception as e:
        print(f"❌ 重试示例失败: {e}")


def performance_example():
    """性能监控示例"""
    print("\n📊 性能监控示例")
    print("=" * 50)
    
    client = HushApiClient()
    
    try:
        print("\n1️⃣ 单个请求性能:")
        start_time = time.time()
        client.health_check()
        elapsed = (time.time() - start_time) * 1000
        print(f"单个请求耗时: {elapsed:.2f}ms")
        
        print("\n2️⃣ 多个请求性能:")
        start_time = time.time()
        
        for i in range(5):
            try:
                client.health_check()
            except Exception as e:
                print(f"请求 {i+1} 失败: {e}")
        
        total_elapsed = (time.time() - start_time) * 1000
        print(f"5个请求总耗时: {total_elapsed:.2f}ms")
        print(f"平均每个请求: {total_elapsed/5:.2f}ms")
        
    except Exception as e:
        print(f"❌ 性能监控示例失败: {e}")


def advanced_usage_example():
    """高级使用示例"""
    print("\n🚀 高级使用示例")
    print("=" * 50)
    
    # 自定义配置的客户端
    client = HushApiClient(
        base_url="http://localhost:8080",
        timeout=10
    )
    
    try:
        print("\n1️⃣ 自定义请求头:")
        response = client._make_request(
            'GET',
            '/health',
            headers={
                'X-Custom-Header': 'CustomValue',
                'X-Request-ID': '12345'
            }
        )
        print(f"自定义请求成功: {response.status_code}")
        
        print("\n2️⃣ 响应头信息:")
        interesting_headers = ['content-type', 'server', 'date']
        for header in interesting_headers:
            value = response.headers.get(header, '未设置')
            print(f"{header}: {value}")
            
    except Exception as e:
        print(f"❌ 高级使用示例失败: {e}")


# ============================================================================
# 主执行函数
# ============================================================================

def run_all_examples():
    """运行所有示例"""
    print("🚀 Hush 框架客户端 Python 示例")
    print("=" * 60)
    
    examples = [
        basic_usage_example,
        authentication_example,
        cors_example,
        post_request_example,
        error_handling_example,
        batch_request_example,
        retry_example,
        performance_example,
        advanced_usage_example
    ]
    
    for example in examples:
        try:
            example()
        except Exception as e:
            logger.error(f"示例 {example.__name__} 执行失败: {e}")
        
        # 在示例之间稍作停顿
        time.sleep(0.5)
    
    print("\n🎉 所有示例执行完成！")


def interactive_mode():
    """交互模式"""
    print("\n🎮 交互模式")
    print("=" * 50)
    
    client = HushApiClient()
    
    while True:
        print("\n可用命令:")
        print("1. health - 健康检查")
        print("2. user - 获取用户信息")
        print("3. users - 获取用户列表")
        print("4. token <token> - 设置认证令牌")
        print("5. quit - 退出")
        
        try:
            command = input("\n请输入命令: ").strip().split()
            
            if not command:
                continue
                
            cmd = command[0].lower()
            
            if cmd == 'quit':
                break
            elif cmd == 'health':
                result = client.health_check()
                print(f"结果: {result}")
            elif cmd == 'user':
                result = client.get_user_info()
                print(f"结果: {result}")
            elif cmd == 'users':
                result = client.get_users()
                print(f"结果: {result}")
            elif cmd == 'token' and len(command) > 1:
                client.set_auth_token(command[1])
                print("认证令牌已设置")
            else:
                print("未知命令")
                
        except HushApiError as e:
            print(f"❌ API 错误: {e.status_code} - {e.response_data}")
        except KeyboardInterrupt:
            break
        except Exception as e:
            print(f"❌ 错误: {e}")
    
    print("👋 再见！")


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == '--interactive':
        interactive_mode()
    else:
        run_all_examples()


# ============================================================================
# 使用说明
# ============================================================================

"""
使用说明:

1. 基本使用:
   python client_examples.py

2. 交互模式:
   python client_examples.py --interactive

3. 作为模块导入:
   from client_examples import HushApiClient
   
   client = HushApiClient('http://your-server:port')
   client.set_auth_token('your-jwt-token')
   users = client.get_users()

4. 错误处理:
   try:
       result = client.some_method()
   except HushApiError as e:
       print(f'Status: {e.status_code}')
       print(f'Data: {e.response_data}')

5. 自定义配置:
   client = HushApiClient(
       base_url='http://localhost:8080',
       timeout=30
   )

依赖安装:
   pip install requests
"""