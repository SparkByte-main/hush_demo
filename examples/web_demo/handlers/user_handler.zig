// ============================================================================
// 用户处理器 - 集成中间件演示
// ============================================================================

const std = @import("std");

extern fn web_server_add_route(server: ?*anyopaque, method: [*:0]const u8, path: [*:0]const u8, handler: *const fn ([*:0]const u8, [*:0]const u8, [*:0]const u8) callconv(.C) [*:0]const u8) void;

pub export fn user_handle(req: [*:0]const u8, res: [*:0]const u8, path: [*:0]const u8) callconv(.C) [*:0]const u8 {
    _ = req;
    _ = res;
    _ = path;
    
    // 返回用户信息的 JSON 响应
    return "{\"user\": {\"id\": 1, \"name\": \"示例用户\", \"email\": \"user@example.com\", \"status\": \"active\"}, \"message\": \"用户信息获取成功\"}";
}
