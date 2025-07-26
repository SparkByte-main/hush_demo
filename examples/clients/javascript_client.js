// ============================================================================
// Hush 框架客户端 JavaScript 示例
// ============================================================================

/**
 * Hush API 客户端类
 * 提供与 Hush 框架后端 API 交互的方法
 */
class HushApiClient {
    constructor(baseUrl = 'http://localhost:8080', options = {}) {
        this.baseUrl = baseUrl;
        this.defaultHeaders = {
            'Content-Type': 'application/json',
            ...options.headers
        };
        this.token = options.token || null;
    }

    /**
     * 设置认证令牌
     * @param {string} token - JWT 令牌
     */
    setAuthToken(token) {
        this.token = token;
    }

    /**
     * 获取请求头
     * @param {Object} additionalHeaders - 额外的请求头
     * @returns {Object} 完整的请求头
     */
    getHeaders(additionalHeaders = {}) {
        const headers = { ...this.defaultHeaders, ...additionalHeaders };
        
        if (this.token) {
            headers['Authorization'] = `Bearer ${this.token}`;
        }
        
        return headers;
    }

    /**
     * 发送 HTTP 请求
     * @param {string} endpoint - API 端点
     * @param {Object} options - 请求选项
     * @returns {Promise} 请求结果
     */
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const config = {
            method: 'GET',
            headers: this.getHeaders(options.headers),
            ...options
        };

        try {
            console.log(`🚀 发送请求: ${config.method} ${url}`);
            
            const response = await fetch(url, config);
            const responseData = await this.handleResponse(response);
            
            console.log(`✅ 请求成功: ${response.status}`, responseData);
            return responseData;
            
        } catch (error) {
            console.error(`❌ 请求失败: ${error.message}`);
            throw error;
        }
    }

    /**
     * 处理响应
     * @param {Response} response - Fetch 响应对象
     * @returns {Promise} 处理后的响应数据
     */
    async handleResponse(response) {
        const contentType = response.headers.get('content-type');
        
        let data;
        if (contentType && contentType.includes('application/json')) {
            data = await response.json();
        } else {
            data = await response.text();
        }

        if (!response.ok) {
            const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
            error.status = response.status;
            error.data = data;
            throw error;
        }

        return data;
    }

    // ========================================================================
    // API 方法
    // ========================================================================

    /**
     * 健康检查
     * @returns {Promise} 健康状态
     */
    async healthCheck() {
        return this.request('/health');
    }

    /**
     * 获取用户信息
     * @returns {Promise} 用户信息
     */
    async getUserInfo() {
        return this.request('/user');
    }

    /**
     * 获取用户列表（需要认证）
     * @returns {Promise} 用户列表
     */
    async getUsers() {
        return this.request('/api/users');
    }

    /**
     * 创建新用户（需要认证）
     * @param {Object} userData - 用户数据
     * @returns {Promise} 创建结果
     */
    async createUser(userData) {
        return this.request('/api/users', {
            method: 'POST',
            body: JSON.stringify(userData)
        });
    }

    /**
     * 获取管理员仪表板（需要管理员权限）
     * @returns {Promise} 仪表板数据
     */
    async getAdminDashboard() {
        return this.request('/admin/dashboard');
    }

    /**
     * 发送 CORS 预检请求
     * @param {string} endpoint - 目标端点
     * @param {string} method - 请求方法
     * @returns {Promise} 预检结果
     */
    async corsPreflightCheck(endpoint, method = 'GET') {
        return this.request(endpoint, {
            method: 'OPTIONS',
            headers: {
                'Access-Control-Request-Method': method,
                'Access-Control-Request-Headers': 'Content-Type, Authorization'
            }
        });
    }
}

// ============================================================================
// 使用示例
// ============================================================================

/**
 * 基本使用示例
 */
async function basicUsageExample() {
    console.log('🎯 基本使用示例');
    console.log('================');

    // 创建 API 客户端
    const client = new HushApiClient();

    try {
        // 1. 健康检查
        console.log('\n1️⃣ 健康检查:');
        const health = await client.healthCheck();
        console.log('健康状态:', health);

        // 2. 获取用户信息
        console.log('\n2️⃣ 用户信息:');
        const userInfo = await client.getUserInfo();
        console.log('用户信息:', userInfo);

    } catch (error) {
        console.error('基本使用示例失败:', error.message);
    }
}

/**
 * 认证示例
 */
async function authenticationExample() {
    console.log('\n🔐 认证示例');
    console.log('============');

    const client = new HushApiClient();

    try {
        // 1. 尝试无认证访问受保护端点
        console.log('\n1️⃣ 无认证访问受保护端点:');
        try {
            await client.getUsers();
        } catch (error) {
            console.log('预期的认证错误:', error.status, error.data);
        }

        // 2. 设置认证令牌并重试
        console.log('\n2️⃣ 使用认证令牌:');
        const sampleToken = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiMTIzNDUiLCJ1c2VybmFtZSI6InpoYW5nc2FuIiwicm9sZSI6InVzZXIiLCJleHAiOjE2NDA5OTUyMDB9.example_signature';
        
        client.setAuthToken(sampleToken);
        
        try {
            const users = await client.getUsers();
            console.log('用户列表:', users);
        } catch (error) {
            console.log('认证失败（预期的，因为使用示例令牌）:', error.status, error.data);
        }

    } catch (error) {
        console.error('认证示例失败:', error.message);
    }
}

/**
 * CORS 示例
 */
async function corsExample() {
    console.log('\n🌐 CORS 示例');
    console.log('=============');

    const client = new HushApiClient();

    try {
        // 1. CORS 预检请求
        console.log('\n1️⃣ CORS 预检请求:');
        try {
            await client.corsPreflightCheck('/api/users', 'GET');
            console.log('CORS 预检成功');
        } catch (error) {
            console.log('CORS 预检结果:', error.status, error.data);
        }

        // 2. 实际的跨域请求
        console.log('\n2️⃣ 跨域请求:');
        try {
            const response = await client.request('/api/users', {
                headers: {
                    'Origin': 'https://app.example.com'
                }
            });
            console.log('跨域请求成功:', response);
        } catch (error) {
            console.log('跨域请求结果:', error.status, error.data);
        }

    } catch (error) {
        console.error('CORS 示例失败:', error.message);
    }
}

/**
 * POST 请求示例
 */
async function postRequestExample() {
    console.log('\n📝 POST 请求示例');
    console.log('==================');

    const client = new HushApiClient();
    client.setAuthToken('valid_jwt_token_here');

    try {
        // 创建新用户
        console.log('\n1️⃣ 创建新用户:');
        const newUser = {
            name: '张三',
            email: 'zhangsan@example.com'
        };

        try {
            const result = await client.createUser(newUser);
            console.log('用户创建成功:', result);
        } catch (error) {
            console.log('用户创建结果:', error.status, error.data);
        }

    } catch (error) {
        console.error('POST 请求示例失败:', error.message);
    }
}

/**
 * 错误处理示例
 */
async function errorHandlingExample() {
    console.log('\n⚠️ 错误处理示例');
    console.log('=================');

    const client = new HushApiClient();

    // 定义错误处理函数
    const handleApiError = (error) => {
        switch (error.status) {
            case 401:
                console.log('🔒 认证错误: 请提供有效的认证令牌');
                break;
            case 403:
                console.log('🚫 权限错误: 访问被拒绝');
                break;
            case 429:
                console.log('🚦 限流错误: 请求过于频繁，请稍后重试');
                break;
            case 500:
                console.log('💥 服务器错误: 内部服务器错误');
                break;
            default:
                console.log(`❓ 未知错误: ${error.status} - ${error.message}`);
        }
        
        if (error.data) {
            console.log('错误详情:', error.data);
        }
    };

    try {
        // 1. 测试认证错误
        console.log('\n1️⃣ 测试认证错误:');
        try {
            await client.getUsers();
        } catch (error) {
            handleApiError(error);
        }

        // 2. 测试权限错误
        console.log('\n2️⃣ 测试权限错误:');
        try {
            await client.getAdminDashboard();
        } catch (error) {
            handleApiError(error);
        }

    } catch (error) {
        console.error('错误处理示例失败:', error.message);
    }
}

/**
 * 批量请求示例
 */
async function batchRequestExample() {
    console.log('\n📦 批量请求示例');
    console.log('=================');

    const client = new HushApiClient();

    try {
        console.log('\n1️⃣ 并行执行多个请求:');
        
        const requests = [
            client.healthCheck(),
            client.getUserInfo(),
            // 这个会失败，但不会影响其他请求
            client.getUsers().catch(error => ({ error: error.message }))
        ];

        const results = await Promise.allSettled(requests);
        
        results.forEach((result, index) => {
            const requestNames = ['健康检查', '用户信息', '用户列表'];
            console.log(`${requestNames[index]}:`, 
                result.status === 'fulfilled' ? result.value : result.reason.message
            );
        });

    } catch (error) {
        console.error('批量请求示例失败:', error.message);
    }
}

/**
 * 重试机制示例
 */
async function retryExample() {
    console.log('\n🔄 重试机制示例');
    console.log('=================');

    const client = new HushApiClient();

    // 重试函数
    const retryRequest = async (requestFn, maxRetries = 3, delay = 1000) => {
        for (let attempt = 1; attempt <= maxRetries; attempt++) {
            try {
                console.log(`尝试 ${attempt}/${maxRetries}...`);
                return await requestFn();
            } catch (error) {
                console.log(`尝试 ${attempt} 失败:`, error.message);
                
                if (attempt === maxRetries) {
                    throw error;
                }
                
                // 指数退避
                const waitTime = delay * Math.pow(2, attempt - 1);
                console.log(`等待 ${waitTime}ms 后重试...`);
                await new Promise(resolve => setTimeout(resolve, waitTime));
            }
        }
    };

    try {
        console.log('\n1️⃣ 重试失败的请求:');
        
        const result = await retryRequest(
            () => client.getUsers(), // 这个请求会失败
            3,
            500
        );
        
        console.log('重试成功:', result);
        
    } catch (error) {
        console.log('所有重试都失败了:', error.message);
    }
}

/**
 * 性能监控示例
 */
async function performanceExample() {
    console.log('\n📊 性能监控示例');
    console.log('=================');

    const client = new HushApiClient();

    // 性能监控装饰器
    const withPerformanceMonitoring = (fn) => {
        return async (...args) => {
            const startTime = performance.now();
            try {
                const result = await fn(...args);
                const endTime = performance.now();
                console.log(`⏱️ 请求耗时: ${(endTime - startTime).toFixed(2)}ms`);
                return result;
            } catch (error) {
                const endTime = performance.now();
                console.log(`⏱️ 失败请求耗时: ${(endTime - startTime).toFixed(2)}ms`);
                throw error;
            }
        };
    };

    try {
        console.log('\n1️⃣ 监控单个请求性能:');
        const monitoredHealthCheck = withPerformanceMonitoring(client.healthCheck.bind(client));
        await monitoredHealthCheck();

        console.log('\n2️⃣ 监控多个请求性能:');
        const requests = Array(5).fill().map(() => monitoredHealthCheck());
        await Promise.all(requests);

    } catch (error) {
        console.error('性能监控示例失败:', error.message);
    }
}

// ============================================================================
// 主执行函数
// ============================================================================

/**
 * 运行所有示例
 */
async function runAllExamples() {
    console.log('🚀 Hush 框架客户端示例');
    console.log('========================');
    
    try {
        await basicUsageExample();
        await authenticationExample();
        await corsExample();
        await postRequestExample();
        await errorHandlingExample();
        await batchRequestExample();
        await retryExample();
        await performanceExample();
        
        console.log('\n🎉 所有示例执行完成！');
        
    } catch (error) {
        console.error('示例执行失败:', error);
    }
}

// ============================================================================
// 浏览器环境检测和执行
// ============================================================================

if (typeof window !== 'undefined') {
    // 浏览器环境
    console.log('🌐 在浏览器环境中运行');
    
    // 将客户端类添加到全局作用域
    window.HushApiClient = HushApiClient;
    window.runAllExamples = runAllExamples;
    
    // 自动运行示例（可选）
    // runAllExamples();
    
} else if (typeof module !== 'undefined' && module.exports) {
    // Node.js 环境
    console.log('🖥️ 在 Node.js 环境中运行');
    
    // 需要 node-fetch 或类似的 polyfill
    if (typeof fetch === 'undefined') {
        console.log('⚠️ 需要 fetch polyfill，请安装 node-fetch:');
        console.log('npm install node-fetch');
        console.log('然后在文件顶部添加: global.fetch = require("node-fetch");');
    } else {
        runAllExamples();
    }
    
    module.exports = { HushApiClient, runAllExamples };
}

// ============================================================================
// 使用说明
// ============================================================================

/*
使用说明:

1. 浏览器中使用:
   - 直接在 HTML 中引入此文件
   - 使用 HushApiClient 类创建客户端实例
   - 调用相应的 API 方法

2. Node.js 中使用:
   - 安装 node-fetch: npm install node-fetch
   - 在文件顶部添加: global.fetch = require('node-fetch');
   - 然后运行: node client_examples.js

3. 自定义使用:
   const client = new HushApiClient('http://your-server:port');
   client.setAuthToken('your-jwt-token');
   const users = await client.getUsers();

4. 错误处理:
   try {
     const result = await client.someMethod();
   } catch (error) {
     console.log('Status:', error.status);
     console.log('Data:', error.data);
   }
*/