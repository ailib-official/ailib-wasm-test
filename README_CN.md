# ailib-wasm-test

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/wasm-wasm32--unknown--unknown-yellow.svg)](https://webassembly.org/)

> 一个最小化的网页聊天应用，用于验证 [ai-lib](https://github.com/ailib-official) 生态中 WASM 组件的可用性。所有 AI 协议逻辑——请求构建、响应解析、错误分类、流事件处理——均运行在由 `ai-lib-core` 编译的浏览器 WASM 模块中。

**阅读语言**：[English](README.md)

---

## 目录

- [项目动机](#项目动机)
- [架构设计](#架构设计)
- [关键发现](#关键发现)
- [项目结构](#项目结构)
- [前置条件](#前置条件)
- [构建与运行](#构建与运行)
- [配置说明](#配置说明)
- [测试](#测试)
- [WASM API 参考](#wasm-api-参考)
- [HTTP API 参考](#http-api-参考)
- [工作原理](#工作原理)
- [治理规则](#治理规则)
- [许可证](#许可证)

---

## 项目动机

ai-lib 生态提供了一个名为 `ai-lib-core` 的 Rust crate，实现了 AI 协议规范——聊天请求构建、响应解析、错误码分类等功能。本项目的核心问题是：

> **`ai-lib-core` 能否编译为浏览器 WASM 并与真实 AI 提供商端到端正常工作？**

答案是**可以的**，但需要注意若干问题，详见[关键发现](#关键发现)。本项目既是验证测试，也是任何希望在浏览器环境中使用 ai-lib-core 的开发者的参考实现。

---

## 架构设计

```
┌──────────────────────────────────────────────────────────────┐
│  浏览器                                                       │
│                                                               │
│  ┌──────────────────┐  ┌──────────────────────────────────┐  │
│  │  index.html      │  │  ailib_wasm_bg.wasm              │  │
│  │  (聊天界面)      │──│  (ai-lib-core 编译为 WASM)       │  │
│  │                  │  │                                  │  │
│  │  • 用户输入      │  │  • build_chat_request()          │  │
│  │  • SSE 展示      │  │  • parse_chat_response()         │  │
│  │  • 模型选择      │  │  • parse_stream_event()          │  │
│  │                  │  │  • classify_error()              │  │
│  │                  │  │  • is_stream_done()              │  │
│  └────────┬─────────┘  └──────────────────────────────────┘  │
│           │                                                    │
│           │  POST /api/proxy  或  POST /api/proxy/stream      │
│           │  (JSON 请求体由 WASM 构建)                        │
└───────────┼────────────────────────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────────────────────────────────┐
│  ailib-wasm-test-server  (Axum, 端口 3000)                   │
│                                                               │
│  • 从环境变量解析 API 密钥                                    │
│  • 通过 libcurl 转发请求到 AI 提供商                          │
│  • 在返回浏览器前移除 Authorization 头                        │
│  • 返回完整响应 (非流式) 或 SSE 流                            │
│                                                               │
│  路由:                                                        │
│    GET  /              → index.html                           │
│    GET  /health        → {"status":"ok","version":"0.1.0"}    │
│    POST /api/proxy     → 非流式代理                           │
│    POST /api/proxy/stream → SSE 流式代理                      │
│    *   /...            → 静态文件服务                         │
└──────────────────────────────────────────────────────────────┘
```

**设计原则**：服务端是无状态的转发管道。所有 AI 协议智能运行在 WASM 模块中。服务端的存在仅为了：

1. 绕过浏览器 CORS 限制
2. 保护 API 密钥（永不发送到浏览器）
3. 处理 TLS 指纹问题（见[关键发现](#关键发现)）

---

## 关键发现

开发过程中，对 ai-lib 生态和 AI 提供商兼容性有若干重要发现：

### 1. ai-lib-wasm 面向 WASI，而非浏览器 WASM

现有的 `ai-lib-wasm` crate 目标平台是 `wasm32-wasip1` (WASI)，专为服务端运行时设计，**无法**在浏览器中运行。要在浏览器中使用 `ai-lib-core`，我们创建了一个新的 crate (`ailib-wasm-browser`)，将 `ai-lib-core` 编译为 `wasm32-unknown-unknown` 并使用 `wasm-bindgen`。

### 2. ai-lib-core 可直接编译为浏览器 WASM

`drivers` 模块（包括 `OpenAiDriver`）和 `error_code` 模块无需任何修改即可编译到 `wasm32-unknown-unknown`。这证实了 `ai-lib-core` 天然兼容 WASM。

### 3. `uuid` 和 `getrandom` 需要启用 `js` 特性

目标为 `wasm32-unknown-unknown` 时，`uuid` 和 `getrandom` crate 需要启用其 `js` 特性：

```toml
uuid = { version = "1.6", features = ["js"] }
getrandom = { version = "0.2", features = ["js"] }
```

否则 WASM 编译会因 "getrandom: unavailable" 而失败。

### 4. 需要设置 `wasm-opt = false`

当系统未安装 `binaryen` (wasm-opt) 时，`wasm-pack` 构建会失败。在 Cargo.toml 元数据中禁用即可：

```toml
[package.metadata.wasm-pack.profile.release]
wasm-opt = false
```

### 5. Groq 屏蔽 Rust TLS 指纹

**这是最具影响力的发现。** Groq 的 Cloudflare 代理对使用 Rust `reqwest` crate 发出的请求返回 HTTP 403——无论使用 `rustls` 还是 `native-tls` 后端。屏蔽基于 TLS 指纹识别 (JA3/JA4)。

- **`reqwest` (rustls)**：403 Forbidden
- **`reqwest` (native-tls)**：403 Forbidden
- **`curl` 二进制 (子进程)**：403 Forbidden
- **`curl` crate (libcurl FFI)**：✅ 200 OK

`curl` crate 的 `Easy` handle 可以正常工作，因为 libcurl 的 TLS 指纹与 Rust 原生 TLS 栈以及 `curl` 二进制均不同。这可能是由于 HTTP/2 协商差异或 libcurl 内部实现的请求头排序不同。

### 6. BigInt / u16 互操作问题

- Rust `i64` 映射到 JavaScript `BigInt` 而非 `Number`。修复方式：将 `max_tokens` 从 `i64` 改为 `f64`。
- Rust `u16` 会导致 wasm-bindgen 返回字符串类型而非数字。修复方式：将 `code_val` 从 `u16` 改为 `u32`。

### 7. HTTP_PROXY 环境变量会影响 Groq

`curl` 二进制子进程会继承父进程的 `HTTP_PROXY` 环境变量，导致 Groq 请求失败。`curl` crate **不会**自动继承代理设置，因此无需额外处理。使用 `curl` 二进制时需添加 `--noproxy "*"`。

---

## 项目结构

```
ailib-wasm-test/
├── Cargo.toml                          # 工作空间定义
├── LICENSE                             # Apache-2.0
├── LICENSE-MIT                         # MIT
├── LICENSE-APACHE                      # Apache-2.0
├── README.md                           # 英文版
├── README_CN.md                        # 中文版（本文件）
├── crates/
│   ├── wasm-browser/                   # 浏览器 WASM crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # wasm_bindgen 包装层
│   │       └── core_logic.rs           # 纯 Rust 逻辑 + 19 个单元测试
│   └── server/                         # 后端代理服务器
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                 # Axum 服务器 + curl 代理 + 8 个测试
├── static/
│   ├── index.html                      # 聊天界面（单文件 SPA）
│   └── wasm/                           # wasm-pack 输出
│       ├── ailib_wasm_bg.wasm
│       ├── ailib_wasm.js
│       └── ailib_wasm.d.ts
└── tests/
    ├── e2e.spec.js                     # Playwright 端到端测试 (7 个测试)
    ├── playwright.config.js
    └── package.json
```

---

## 前置条件

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.75+ | 构建服务器和 WASM crate |
| wasm-pack | 最新 | 构建 WASM 浏览器目标 |
| Node.js | 18+ | Playwright 端到端测试 |
| curl (libcurl) | 任意 | 服务器使用 `curl` crate 的 `static-curl` 特性 |

### 安装 wasm-pack

```bash
cargo install wasm-pack
```

### 安装 Playwright

```bash
cd tests && npm install && npx playwright install
```

---

## 构建与运行

### 1. 构建 WASM 模块

```bash
wasm-pack build crates/wasm-browser --target web --out-dir ../../static/wasm --out-name ailib_wasm
```

此命令将 `ai-lib-core` 编译为 `wasm32-unknown-unknown`，并将 WASM 二进制 + JS 胶水代码输出到 `static/wasm/`。

### 2. 构建服务器

```bash
cargo build --release
```

### 3. 设置环境变量

```bash
export GROQ_API_KEY="gsk_..."
export DEEPSEEK_API_KEY="sk-..."
export NVIDIA_API_KEY="nvapi-..."
# 可选：OpenAI
export OPENAI_API_KEY="sk-..."
```

### 4. 启动服务器

```bash
./target/release/ailib-wasm-test-server
```

或作为后台进程：

```bash
nohup ./target/release/ailib-wasm-test-server > /tmp/ailib-wasm-test.log 2>&1 & disown
```

### 5. 在浏览器中打开

访问 [http://localhost:3000](http://localhost:3000)。你将看到：

- 页头显示 "WASM ready"（确认 WASM 模块已加载）
- 模型选择器，支持 NVIDIA、Groq 和 DeepSeek 提供商
- 支持流式输出的聊天输入框

---

## 配置说明

### 支持的提供商

| 提供商 | 模型 ID | 环境变量 | 状态 |
|--------|---------|----------|------|
| NVIDIA | `nvidia/z-ai/glm-5.1` | `NVIDIA_API_KEY` | ✅ 可用 |
| NVIDIA | `nvidia/z-ai/glm4.7` | `NVIDIA_API_KEY` | ✅ 可用 |
| Groq | `groq/llama-3.1-8b-instant` | `GROQ_API_KEY` | ✅ 可用（仅 curl crate） |
| DeepSeek | `deepseek/deepseek-chat` | `DEEPSEEK_API_KEY` | ✅ 可用 |
| OpenAI | 任意 | `OPENAI_API_KEY` | ✅ 可用 |

### 服务器配置

服务器默认监听 `0.0.0.0:3000`。没有命令行参数，配置仅通过环境变量完成。

---

## 测试

### 单元测试（27 个测试）

```bash
cargo test --release
```

包含：
- **19 个 WASM 逻辑测试** (`core_logic.rs`) — 纯 Rust，无需 WASM 运行时
- **8 个服务器测试** (`main.rs`) — 包括健康检查端点、路由处理和 Groq/DeepSeek 实际集成测试

### 端到端测试（7 个测试）

需要服务器正在运行：

```bash
# 先启动服务器
./target/release/ailib-wasm-test-server &

# 运行端到端测试
cd tests && env -i HOME=$HOME PATH=$PATH npx playwright test --reporter=list
```

测试覆盖：
1. 健康检查端点返回 OK
2. `index.html` 正确加载，标题正确
3. WASM 模块初始化成功
4. DeepSeek 流式聊天端到端正常
5. Groq 流式聊天端到端正常
6. 新建聊天清除历史记录
7. 所有 WASM 函数可从 JavaScript 调用

---

## WASM API 参考

浏览器 WASM 模块通过 `wasm-bindgen` 暴露 5 个函数：

### `build_chat_request(messages_json, model, temperature, max_tokens, stream) → BuildResult`

使用 `ai-lib-core` 的 `OpenAiDriver` 构建 OpenAI 兼容的聊天补全请求体。

| 参数 | 类型 | 说明 |
|------|------|------|
| `messages_json` | `string` | 消息数组的 JSON 字符串，格式为 `[{"role":"user\|assistant\|system","content":"..."}]` |
| `model` | `string` | 模型标识符（如 `"deepseek-chat"`） |
| `temperature` | `f64` | 采样温度 (0.0–2.0) |
| `max_tokens` | `f64` | 最大生成 token 数（使用 f64 以避免 JS BigInt 问题） |
| `stream` | `bool` | 是否启用 SSE 流式输出 |

**返回值**：`BuildResult`，包含 `.body()`（JSON 字符串）和 `.stream()`（布尔值）。

### `parse_chat_response(response_json) → ParseResult`

解析非流式聊天补全响应 JSON。

**返回值**：`ParseResult`，包含 `.content()`、`.finish_reason()`、`.prompt_tokens()`、`.completion_tokens()`、`.total_tokens()`。

### `parse_stream_event(data) → StreamEventResult`

解析单个 SSE 流事件的数据载荷。

**返回值**：`StreamEventResult`，包含：
- `.event_type()` — `"content_delta"`、`"thinking_delta"`、`"role_assign"`、`"stream_end"` 或 `"unknown"`
- `.data()` — 提取的内容字符串
- `.done()` — 流是否结束

### `classify_error(status_code) → ErrorClassResult`

使用 `ai-lib-core` 的 `StandardErrorCode` 对 HTTP 错误状态码进行分类。

**返回值**：`ErrorClassResult`，包含 `.code()`、`.name()`、`.category()`、`.retryable()`。

### `is_stream_done(data) → bool`

检查 SSE 数据载荷是否表示流结束（`[DONE]`）。

---

## HTTP API 参考

### `GET /health`

返回服务器健康状态。

```json
{"status": "ok", "version": "0.1.0"}
```

### `POST /api/proxy`

非流式代理。将请求体转发到指定的 AI 提供商 URL。

**请求体**：
```json
{
  "url": "https://api.deepseek.com/chat/completions",
  "headers": {},
  "body": { "model": "deepseek-chat", "messages": [...], "stream": false },
  "stream": false
}
```

**响应**：
```json
{
  "status": 200,
  "body": { "choices": [...], "usage": {...} }
}
```

### `POST /api/proxy/stream`

SSE 流式代理。转发请求并以 Server-Sent Events 格式返回响应。

**请求体**：与 `/api/proxy` 相同，但 `"stream": true`。

**响应**：SSE 流，包含 `data:` 事件。每个事件包含上游提供商 SSE 响应的一行数据。

---

## 工作原理

### 请求流程（流式）

1. 用户输入消息并点击 "Send"
2. JavaScript 调用 `wasm.build_chat_request(messages, model, 0.7, 4096, true)` — WASM 构建符合协议的请求体
3. JavaScript 发送 `POST /api/proxy/stream`，携带 WASM 构建的请求体
4. 服务器的 `proxy_stream_handler` 接收请求
5. 服务器根据 URL 从环境变量解析 API 密钥
6. `spawn_blocking` 线程运行 `curl` crate 的 `Easy` handle
7. `write_function` 回调按换行符分割传入数据，通过 `mpsc` 通道发送每一行
8. 异步端从 `ReceiverStream` 读取，去除 `data: ` 前缀，将每行包装为 axum SSE `Event`
9. 浏览器 JavaScript 读取 SSE 流，对每个事件调用 `wasm.parse_stream_event(data)`
10. WASM 返回事件类型（`content_delta`、`stream_end` 等）和数据
11. JavaScript 将内容追加到聊天显示区域

### 请求流程（非流式）

与上述步骤 1–4 相同，但：
- 端点为 `POST /api/proxy`
- 服务器通过 `curl_proxy()` 等待完整响应
- JavaScript 调用 `wasm.parse_chat_response(body)` 提取内容、结束原因和使用量

### 错误处理

当代理返回非 OK 状态时，JavaScript 调用 `wasm.classify_error(statusCode)`，返回：
- 错误码和名称（如 `429 RATE_LIMITED`）
- 类别（`"rate"`、`"client"`、`"server"`）
- 错误是否可重试

---

## 治理规则

本项目遵循 ai-lib 生态治理规则：

| 规则 | 说明 |
|------|------|
| **ARCH-001** | 协议驱动设计 — 所有 AI 逻辑来自 ai-lib-core |
| **ARCH-004** | 默认分支为 `main` |
| **GOV-001 v2** | 公开代码仓库必须位于 `ailib-official`，不得放在个人账户下 |
| **GOV-002** | 合并冲突解决需代码级审查，不得盲目覆盖整个文件 |
| **DOC-001** | 英文代码注释 + 中文模块头 |

---

## 许可证

本项目采用以下任一许可证授权：

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

由您自行选择。

除非您明确声明，否则您有意提交以包含在本项目中的任何贡献均应按上述双重许可授权，不附加任何额外条款或条件。
