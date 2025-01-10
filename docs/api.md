# API 文档

## 1. 用户注册

### 请求格式

```json
{
  "action": "register",
  "username": "newUser",
  "password": "securePassword123"
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "generic_response",
  "status": "success",
  "message": "Registration successful."
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "Username already exists."
}
```

---

## 2. 用户登录

### 请求格式

```json
{
  "action": "login",
  "user_id": 123,
  "password": "userPassword"
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "generic_response",
  "status": "success",
  "message": "Login successful."
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "Invalid user credentials."
}
```

---

## 3. 获取用户信息

### 请求格式

```json
{
  "action": "userinfo",
  "user_id": 123
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "username",
  "user_id": 123,
  "username": "exampleUser"
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "User not found."
}
```

---

## 4. 发送消息

### 请求格式

```json
{
  "action": "send_message",
  "receiver": 456,
  "message": "Hello, this is a message!"
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "generic_response",
  "status": "success",
  "message": "Message sent successfully."
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "User is offline."
}
```

---

## 5. 接收消息（服务器主动推送）

### 回复格式

```json
{
  "action": "receive_message",
  "sender": 123,
  "message": "Hello, this is a received message!",
  "timestamp": "2025-01-10T10:05:00Z"
}
```

---

## 6. 获取在线用户列表

### 请求格式

```json
{
  "action": "request",
  "request": "online_users"
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "online_users",
  "flag": "success",
  "user_ids": [123, 456, 789]
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "Failed to retrieve online users."
}
```

---

## 7. 获取好友列表

### 请求格式

```json
{
  "action": "request",
  "request": "get_friends"
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "friend_list",
  "friend_ids": [123, 456, 789]
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "Failed to retrieve friend list."
}
```

---

## 8. 获取群组列表

### 请求格式

```json
{
  "action": "request",
  "request": "get_groups"
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "group_list",
  "friend_ids": [1, 2, 3]
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "Failed to retrieve group list."
}
```

---

## 9. 获取群组成员

### 请求格式

```json
{
  "action": "objrequest",
  "request": "get_group_members",
  "id": 1
}
```

### 回复格式

#### 成功响应

```json
{
  "action": "group_members",
  "group_id": 1,
  "member_ids": [123, 456, 789]
}
```

#### 失败响应

```json
{
  "action": "error",
  "message": "Group not found."
}
```

---

## 错误响应格式

在所有请求中，当服务器无法处理请求时，都会返回以下格式的错误响应：

```json
{
  "action": "error",
  "message": "Detailed error description."
}
```

---
