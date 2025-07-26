#!/bin/bash

# ============================================================================
# Hush 框架 API 测试脚本
# ============================================================================

echo "🚀 Hush 框架 API 测试脚本"
echo "=================================="

# 服务器配置
SERVER_URL="http://localhost:8080"
SLEEP_TIME=1

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 辅助函数
print_test() {
    echo -e "\n${BLUE}🧪 测试: $1${NC}"
    echo "----------------------------------------"
}

print_success() {
    echo -e "${GREEN}✅ 成功: $1${NC}"
}

print_error() {
    echo -e "${RED}❌ 错误: $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  信息: $1${NC}"
}

# 检查服务器是否运行
check_server() {
    print_test "检查服务器状态"
    
    if curl -s --connect-timeout 5 "$SERVER_URL/health" > /dev/null; then
        print_success "服务器正在运行"
        return 0
    else
        print_error "服务器未运行，请先启动服务器："
        echo "  zig run zig_web_demo/main.zig -lc -L./target/debug -lhush_demo"
        return 1
    fi
}

# 测试 1: 健康检查端点
test_health_check() {
    print_test "健康检查端点"
    
    echo "请求: GET $SERVER_URL/health"
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" "$SERVER_URL/health")
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "200" ]; then
        print_success "健康检查通过"
    else
        print_error "健康检查失败"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 2: 用户信息端点
test_user_info() {
    print_test "用户信息端点"
    
    echo "请求: GET $SERVER_URL/user"
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" "$SERVER_URL/user")
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "200" ]; then
        print_success "用户信息获取成功"
    else
        print_error "用户信息获取失败"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 3: CORS 预检请求
test_cors_preflight() {
    print_test "CORS 预检请求"
    
    echo "请求: OPTIONS $SERVER_URL/api/users"
    echo "Origin: https://app.example.com"
    
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
        -X OPTIONS \
        -H "Origin: https://app.example.com" \
        -H "Access-Control-Request-Method: GET" \
        -H "Access-Control-Request-Headers: Content-Type, Authorization" \
        "$SERVER_URL/api/users")
    
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    # 获取 CORS 头部
    headers=$(curl -s -I \
        -X OPTIONS \
        -H "Origin: https://app.example.com" \
        -H "Access-Control-Request-Method: GET" \
        "$SERVER_URL/api/users")
    
    echo "CORS 响应头:"
    echo "$headers" | grep -i "access-control" || echo "  无 CORS 头部"
    
    if [ "$http_code" = "204" ] || [ "$http_code" = "200" ]; then
        print_success "CORS 预检请求成功"
    else
        print_error "CORS 预检请求失败"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 4: 无认证的受保护端点
test_protected_without_auth() {
    print_test "受保护端点（无认证）"
    
    echo "请求: GET $SERVER_URL/api/users"
    echo "认证: 无"
    
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" "$SERVER_URL/api/users")
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "401" ] || [ "$http_code" = "403" ]; then
        print_success "正确拒绝了无认证请求"
    else
        print_error "应该拒绝无认证请求"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 5: 带认证的受保护端点
test_protected_with_auth() {
    print_test "受保护端点（带认证）"
    
    # 使用示例 JWT 令牌（在实际应用中应该是有效的）
    jwt_token="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiMTIzNDUiLCJ1c2VybmFtZSI6InpoYW5nc2FuIiwicm9sZSI6InVzZXIiLCJleHAiOjE2NDA5OTUyMDB9.example_signature"
    
    echo "请求: GET $SERVER_URL/api/users"
    echo "认证: Bearer $jwt_token"
    
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
        -H "Authorization: Bearer $jwt_token" \
        "$SERVER_URL/api/users")
    
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "200" ]; then
        print_success "认证请求成功"
    elif [ "$http_code" = "401" ]; then
        print_info "JWT 令牌无效（这是预期的，因为使用的是示例令牌）"
    else
        print_error "意外的响应状态码"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 6: POST 请求创建用户
test_create_user() {
    print_test "创建用户 POST 请求"
    
    jwt_token="valid_jwt_token_here"
    user_data='{"name": "测试用户", "email": "test@example.com"}'
    
    echo "请求: POST $SERVER_URL/api/users"
    echo "认证: Bearer $jwt_token"
    echo "数据: $user_data"
    
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $jwt_token" \
        -d "$user_data" \
        "$SERVER_URL/api/users")
    
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        print_success "用户创建请求成功"
    elif [ "$http_code" = "401" ]; then
        print_info "需要有效的认证令牌"
    else
        print_error "用户创建请求失败"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 7: 管理员端点
test_admin_endpoint() {
    print_test "管理员端点"
    
    admin_token="admin_jwt_token_here"
    
    echo "请求: GET $SERVER_URL/admin/dashboard"
    echo "认证: Bearer $admin_token"
    
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
        -H "Authorization: Bearer $admin_token" \
        "$SERVER_URL/admin/dashboard")
    
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "200" ]; then
        print_success "管理员端点访问成功"
    elif [ "$http_code" = "401" ] || [ "$http_code" = "403" ]; then
        print_info "需要管理员权限（这是预期的）"
    else
        print_error "管理员端点访问失败"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 8: 限流测试
test_rate_limiting() {
    print_test "请求限流测试"
    
    echo "发送多个快速请求以触发限流..."
    
    success_count=0
    rate_limited_count=0
    
    for i in {1..10}; do
        response=$(curl -s -w "\nHTTP_CODE:%{http_code}" "$SERVER_URL/health")
        http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
        
        if [ "$http_code" = "200" ]; then
            success_count=$((success_count + 1))
        elif [ "$http_code" = "429" ]; then
            rate_limited_count=$((rate_limited_count + 1))
        fi
        
        echo -n "."
    done
    
    echo ""
    echo "成功请求: $success_count"
    echo "被限流请求: $rate_limited_count"
    
    if [ $success_count -gt 0 ]; then
        print_success "基本请求功能正常"
    fi
    
    if [ $rate_limited_count -gt 0 ]; then
        print_success "限流机制正常工作"
    else
        print_info "未触发限流（可能需要更多请求）"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 9: 错误的 CORS 源
test_cors_rejection() {
    print_test "CORS 源拒绝测试"
    
    echo "请求: OPTIONS $SERVER_URL/api/users"
    echo "Origin: https://malicious-site.com"
    
    response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
        -X OPTIONS \
        -H "Origin: https://malicious-site.com" \
        -H "Access-Control-Request-Method: GET" \
        "$SERVER_URL/api/users")
    
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    body=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    echo "响应状态码: $http_code"
    echo "响应内容: $body"
    
    if [ "$http_code" = "403" ]; then
        print_success "正确拒绝了不允许的源"
    else
        print_error "应该拒绝不允许的源"
    fi
    
    sleep $SLEEP_TIME
}

# 测试 10: 性能测试
test_performance() {
    print_test "基本性能测试"
    
    echo "测试 100 个并发请求的响应时间..."
    
    start_time=$(date +%s%N)
    
    # 使用 xargs 进行并发请求
    seq 1 100 | xargs -n1 -P10 -I{} curl -s "$SERVER_URL/health" > /dev/null
    
    end_time=$(date +%s%N)
    duration=$(( (end_time - start_time) / 1000000 )) # 转换为毫秒
    
    echo "100 个请求总耗时: ${duration}ms"
    echo "平均每个请求: $((duration / 100))ms"
    
    if [ $duration -lt 5000 ]; then # 5秒内完成
        print_success "性能测试通过"
    else
        print_error "性能可能需要优化"
    fi
}

# 主测试函数
run_all_tests() {
    echo -e "\n${BLUE}开始执行所有测试...${NC}"
    
    # 检查服务器状态
    if ! check_server; then
        exit 1
    fi
    
    # 执行所有测试
    test_health_check
    test_user_info
    test_cors_preflight
    test_protected_without_auth
    test_protected_with_auth
    test_create_user
    test_admin_endpoint
    test_rate_limiting
    test_cors_rejection
    test_performance
    
    echo -e "\n${GREEN}🎉 所有测试完成！${NC}"
}

# 显示使用帮助
show_help() {
    echo "Hush 框架 API 测试脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help          显示此帮助信息"
    echo "  -s, --server URL    设置服务器 URL (默认: http://localhost:8080)"
    echo "  -t, --test NAME     运行特定测试"
    echo ""
    echo "可用测试:"
    echo "  health              健康检查测试"
    echo "  user                用户信息测试"
    echo "  cors                CORS 测试"
    echo "  auth                认证测试"
    echo "  post                POST 请求测试"
    echo "  admin               管理员端点测试"
    echo "  rate                限流测试"
    echo "  performance         性能测试"
    echo "  all                 运行所有测试 (默认)"
    echo ""
    echo "示例:"
    echo "  $0                  # 运行所有测试"
    echo "  $0 -t health        # 只运行健康检查测试"
    echo "  $0 -s http://localhost:9000 -t all  # 使用自定义服务器地址"
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -s|--server)
            SERVER_URL="$2"
            shift 2
            ;;
        -t|--test)
            TEST_NAME="$2"
            shift 2
            ;;
        *)
            echo "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# 执行指定的测试
case "${TEST_NAME:-all}" in
    health)
        check_server && test_health_check
        ;;
    user)
        check_server && test_user_info
        ;;
    cors)
        check_server && test_cors_preflight && test_cors_rejection
        ;;
    auth)
        check_server && test_protected_without_auth && test_protected_with_auth
        ;;
    post)
        check_server && test_create_user
        ;;
    admin)
        check_server && test_admin_endpoint
        ;;
    rate)
        check_server && test_rate_limiting
        ;;
    performance)
        check_server && test_performance
        ;;
    all)
        run_all_tests
        ;;
    *)
        echo "未知测试: $TEST_NAME"
        show_help
        exit 1
        ;;
esac