#!/bin/bash

echo "Testing Hush Framework Web Server..."

# 启动服务器（后台运行）
./web_test &
SERVER_PID=$!

# 等待服务器启动
sleep 3

echo "Testing GET request to root path..."
curl -s http://127.0.0.1:8080/ || echo "GET / failed"

echo -e "\nTesting POST request to root path..."
curl -s -X POST -d "test data" http://127.0.0.1:8080/ || echo "POST / failed"

echo -e "\nTesting GET request to /about..."
curl -s http://127.0.0.1:8080/about || echo "GET /about failed"

echo -e "\nTesting POST request to /api/users..."
curl -s -X POST -H "Content-Type: application/json" -d '{"name":"test","email":"test@example.com"}' http://127.0.0.1:8080/api/users || echo "POST /api/users failed"

# 停止服务器
kill $SERVER_PID 2>/dev/null

echo -e "\nServer test completed."