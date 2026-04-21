//! # ailib-wasm-browser
//!
//! 浏览器 WASM 模块 — 将 ai-lib-core 的协议逻辑暴露给 JavaScript。
//!
//! Browser WASM module exposing ai-lib-core protocol logic via wasm-bindgen.
//! Core logic lives in `core_logic` (testable on native), wasm_bindgen wrappers
//! are thin conversions to `Result<T, JsValue>`.

use wasm_bindgen::prelude::*;

mod abi;
mod core_logic;

pub use abi::BROWSER_ABI_VERSION;

/// Result of `build_chat_request`: contains the serialized request body and stream flag.
#[wasm_bindgen]
pub struct BuildResult {
    body_str: String,
    stream_flag: bool,
}

#[wasm_bindgen]
impl BuildResult {
    /// The JSON request body as a string.
    pub fn body(&self) -> String {
        self.body_str.clone()
    }

    /// Whether streaming is enabled.
    pub fn stream(&self) -> bool {
        self.stream_flag
    }
}

/// Result of `parse_chat_response`: contains content, finish reason, and normalized usage.
///
/// Exposes ARCH-003 extended token counters (`reasoning_tokens`,
/// `cache_read_tokens`, `cache_creation_tokens`) that parity with
/// ai-lib-ts/ai-lib-go `UnifiedResponse.usage`.
#[wasm_bindgen]
pub struct ParseResult {
    content_str: String,
    finish_reason_str: String,
    prompt_t: i64,
    completion_t: i64,
    total_t: i64,
    reasoning_t: i64,
    cache_read_t: i64,
    cache_creation_t: i64,
}

#[wasm_bindgen]
impl ParseResult {
    pub fn content(&self) -> String {
        self.content_str.clone()
    }
    pub fn finish_reason(&self) -> String {
        self.finish_reason_str.clone()
    }
    pub fn prompt_tokens(&self) -> i64 {
        self.prompt_t
    }
    pub fn completion_tokens(&self) -> i64 {
        self.completion_t
    }
    pub fn total_tokens(&self) -> i64 {
        self.total_t
    }
    pub fn reasoning_tokens(&self) -> i64 {
        self.reasoning_t
    }
    pub fn cache_read_tokens(&self) -> i64 {
        self.cache_read_t
    }
    pub fn cache_creation_tokens(&self) -> i64 {
        self.cache_creation_t
    }
}

/// Result of `parse_stream_event`: contains event type, data, and done flag.
#[wasm_bindgen]
pub struct StreamEventResult {
    event_type_str: String,
    data_str: String,
    done_flag: bool,
}

#[wasm_bindgen]
impl StreamEventResult {
    pub fn event_type(&self) -> String {
        self.event_type_str.clone()
    }
    pub fn data(&self) -> String {
        self.data_str.clone()
    }
    pub fn done(&self) -> bool {
        self.done_flag
    }
}

/// Result of `classify_error`: contains error code, name, category, retryable.
#[wasm_bindgen]
pub struct ErrorClassResult {
    code_val: u32,
    name_str: String,
    category_str: String,
    retryable_val: bool,
}

#[wasm_bindgen]
impl ErrorClassResult {
    pub fn code(&self) -> u32 {
        self.code_val
    }
    pub fn name(&self) -> String {
        self.name_str.clone()
    }
    pub fn category(&self) -> String {
        self.category_str.clone()
    }
    pub fn retryable(&self) -> bool {
        self.retryable_val
    }
}

/// Build a chat completion request body using ai-lib-core protocol logic.
#[wasm_bindgen]
pub fn build_chat_request(
    messages_json: &str,
    model: &str,
    temperature: f64,
    max_tokens: f64,
    stream: bool,
) -> Result<BuildResult, JsValue> {
    core_logic::build_chat_request(messages_json, model, temperature, max_tokens, stream)
        .map(|(body, stream_flag)| BuildResult {
            body_str: body,
            stream_flag,
        })
        .map_err(|e| JsValue::from_str(&e))
}

/// Parse a non-streaming chat completion response.
#[wasm_bindgen]
pub fn parse_chat_response(response_json: &str) -> Result<ParseResult, JsValue> {
    core_logic::parse_chat_response(response_json)
        .map(|r| ParseResult {
            content_str: r.content,
            finish_reason_str: r.finish_reason,
            prompt_t: r.prompt_tokens,
            completion_t: r.completion_tokens,
            total_t: r.total_tokens,
            reasoning_t: r.reasoning_tokens,
            cache_read_t: r.cache_read_tokens,
            cache_creation_t: r.cache_creation_tokens,
        })
        .map_err(|e| JsValue::from_str(&e))
}

/// Parse a single SSE stream event data payload.
#[wasm_bindgen]
pub fn parse_stream_event(data: &str) -> Result<StreamEventResult, JsValue> {
    core_logic::parse_stream_event(data)
        .map(|(event_type, data_str, done)| StreamEventResult {
            event_type_str: event_type,
            data_str,
            done_flag: done,
        })
        .map_err(|e| JsValue::from_str(&e))
}

/// Classify an HTTP error status code.
#[wasm_bindgen]
pub fn classify_error(status_code: u16) -> ErrorClassResult {
    let (code, name, category, retryable) = core_logic::classify_error(status_code);
    ErrorClassResult {
        code_val: code,
        name_str: name,
        category_str: category,
        retryable_val: retryable,
    }
}

/// Check if an SSE data payload signals stream completion (`[DONE]`).
#[wasm_bindgen]
pub fn is_stream_done(data: &str) -> bool {
    core_logic::is_stream_done(data)
}

/// Unified versioned entry point (WASM-001). Pass a JSON object with `"op"` and fields
/// for that operation; returns a JSON string on success.
///
/// Operations: `abi_version`, `capabilities`, `build_request`, `parse_response`,
/// `parse_stream_event`, `classify_error`, `is_stream_done`. Optional `"version"`
/// field defaults to `1` and must be `<= BROWSER_ABI_VERSION`.
#[wasm_bindgen(js_name = ailib_invoke)]
pub fn ailib_invoke_json(input_json: &str) -> Result<String, JsValue> {
    abi::invoke(input_json).map_err(|e| JsValue::from_str(&e))
}
