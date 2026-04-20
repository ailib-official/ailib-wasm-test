# ailib-wasm-test

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-blue.svg)](LICENSE) [![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/) [![WASM](https://img.shields.io/badge/wasm-wasm32--unknown--unknown-yellow.svg)](https://webassembly.org/)

> 一个最小化的网页聊天应用，证明 **ai-lib-core 可以在浏览器中运行**。所有 AI 协议逻辑——请求构建、响应解析、错误分类、流事件处理——均运行在由 `ai-lib-core` 编译的 WASM 模块中。

**阅读语言**：[English](README.md)

---

## 这是什么？

一个概念验证聊天应用，展示 [ai-lib](https://github.com/ailib-official) 协议栈通过 WebAssembly 可以在浏览器中端到端工作。

浏览器加载一个由 `ai-lib-core`（Rust）编译的 WASM 模块。当你发送消息时，WASM 模块——而非服务端——负责构建 AI 协议请求、解析响应、分类错误、处理流事件。服务端仅是一个薄代理，负责 CORS 和密钥安全。

### 一句话总结

> **协议智能运行在浏览器中，由 Rust 编译而来。**

---

## 快速体验

```bash
# 构建 WASM
wasm-pack build crates/wasm-browser --target web --out-dir ../../static/wasm --out-name ailib_wasm

# 构建服务端
cargo build --release

# 设置 API 密钥
export DEEPSEEK_API_KEY="sk-..."
export NVIDIA_API_KEY="nvapi-..."

# 启动
./target/release/ailib-wasm-test-server
```

打开 **http://localhost:3000** — 与 AI 模型对话，全部由 WASM 协议逻辑驱动。

---

## 工作原理

```
┌──────────────────────────────────────────────┐
│  浏览器                                       │
│                                               │
│  ┌─────────┐     ┌────────────────────────┐  │
│  │ 聊天界面 │────▶│ ailib_wasm (WASM)      │  │
│  │         │     │                        │  │
│  │ 显示    │◀────│ • 构建请求             │  │
│  │ 流式    │     │ • 解析响应             │  │
│  │         │     │ • 分类错误             │  │
│  └─────────┘     └────────────────────────┘  │
│        │                      │               │
└────────┼──────────────────────┼───────────────┘
         │  WASM 构建的请求体    │
         ▼                      │
┌──────────────────────────────────────────────┐
│  薄代理 (Axum)                               │
│                                              │
│  • 对浏览器隐藏 API 密钥                      │
│  • 绕过 CORS                                 │
│  • 转发请求到 AI 提供商                       │
└──────────────────────────────────────────────┘
```

**设计原则**：服务端是无状态的转发管道。所有 AI 协议智能运行在 WASM 模块中。

---

## 支持的提供商

| 提供商 | 模型 | 状态 |
|--------|------|------|
| **DeepSeek** | deepseek-chat | ✅ |
| **NVIDIA** | glm-5.1, glm4.7 | ✅ |
| **Groq** | llama-3.1-8b-instant | ✅ |
| **OpenAI** | 任意 | ✅ |

设置对应的环境变量（`DEEPSEEK_API_KEY`、`NVIDIA_API_KEY`、`GROQ_API_KEY`、`OPENAI_API_KEY`），在界面中选择模型即可。

---

## WASM API

WASM 模块通过 `wasm-bindgen` 暴露 5 个函数：

| 函数 | 用途 |
|------|------|
| `build_chat_request()` | 构建 OpenAI 兼容请求体 |
| `parse_chat_response()` | 解析非流式响应 |
| `parse_stream_event()` | 解析单个 SSE 流事件 |
| `classify_error()` | 按标准错误码分类 HTTP 错误 |
| `is_stream_done()` | 检查流事件是否表示结束 |

所有函数均在 `ai-lib-core` 中实现并编译为 WASM — 零 JavaScript 协议逻辑。

---

## 构建与运行

### 前置条件

- **Rust** 1.75+ — [rustup.rs](https://rustup.rs)
- **wasm-pack** — `cargo install wasm-pack`

### 步骤

```bash
# 1. 构建 WASM 模块
wasm-pack build crates/wasm-browser --target web --out-dir ../../static/wasm --out-name ailib_wasm

# 2. 构建服务端
cargo build --release

# 3. 设置 API 密钥
export DEEPSEEK_API_KEY="sk-..."
export NVIDIA_API_KEY="nvapi-..."

# 4. 启动
./target/release/ailib-wasm-test-server
```

### 测试

```bash
# 单元测试
cargo test --release

# 端到端测试（需要运行中的服务端 + Playwright）
cd tests && npm install && npx playwright install
npx playwright test
```

---

## 项目结构

```
ailib-wasm-test/
├── crates/
│   ├── wasm-browser/    # WASM crate（wasm-bindgen + ai-lib-core）
│   └── server/          # Axum 薄代理
├── static/
│   ├── index.html       # 单文件聊天界面
│   └── wasm/            # 编译输出的 WASM
└── tests/
    └── e2e.spec.js      # Playwright 端到端测试
```

---

## 为什么这很重要？

本项目验证了 ai-lib 生态的一个关键属性：

> **同一份 Rust 代码，既能在服务端运行，也能在浏览器中运行。**

不需要在 JavaScript 中重新实现协议。服务端和客户端的 AI 逻辑不会产生漂移。一个协议栈，多种运行时。

这是 ai-lib 愿景的基础：一个开放的 AI 协议，让任何运行时——服务端、边缘节点、浏览器、嵌入式设备——都能说同一种语言。

---

## 许可证

本项目采用以下任一许可证授权：

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

由您自行选择。
