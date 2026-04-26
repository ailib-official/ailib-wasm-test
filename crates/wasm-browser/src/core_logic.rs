//! # core_logic — Pure Rust protocol logic, testable on native target.
//!
//! 核心协议逻辑 — 纯 Rust 实现，不依赖 wasm-bindgen，可在原生目标上测试。
//!
//! Each function returns `Result<..., String>` so the wasm_bindgen wrappers
//! only need to convert `String` → `JsValue`.

use ai_lib_core::drivers::{AnthropicDriver, OpenAiDriver, ProviderDriver};
use ai_lib_core::error_code::StandardErrorCode;
use ai_lib_core::protocol::v2::capabilities::Capability;
use ai_lib_core::types::events::StreamingEvent;
use ai_lib_core::types::message::Message;
use serde_json::{json, Value};

fn wasm_browser_caps() -> Vec<Capability> {
    vec![Capability::Text, Capability::Streaming]
}

/// Build a chat completion request body using ai-lib-core's OpenAiDriver.
pub fn build_chat_request(
    messages_json: &str,
    model: &str,
    temperature: f64,
    max_tokens: f64,
    stream: bool,
) -> Result<(String, bool), String> {
    let ai_messages: Vec<Message> =
        serde_json::from_str(messages_json).map_err(|e| format!("Invalid messages JSON: {}", e))?;

    let driver = OpenAiDriver::new("wasm-browser", wasm_browser_caps());
    let request = driver
        .build_request(
            &ai_messages,
            model,
            Some(temperature),
            Some(max_tokens as u32),
            stream,
            None,
        )
        .map_err(|e| format!("build_request failed: {:?}", e))?;

    let body = serde_json::to_string(&request.body)
        .map_err(|e| format!("Serialize request failed: {}", e))?;

    Ok((body, stream))
}

/// Flattened non-streaming parse result. Fields ordered to keep the first five
/// positional values backward-compatible with the original 5-tuple; extra
/// extended-token fields are appended so the wasm-bindgen layer can grow
/// without breaking existing JS callers.
pub struct ParsedResponse {
    pub content: String,
    pub finish_reason: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
}

/// Parse a non-streaming chat completion response.
///
/// Uses `ai-lib-core` `OpenAiDriver::parse_response` so **usage and content**
/// match the Rust runtime. Token extraction in `usage` is **ARCH-003**
/// (unified `parse_openai_usage_value` in `ai-lib-core`).
pub fn parse_chat_response(response_json: &str) -> Result<ParsedResponse, String> {
    let body: Value =
        serde_json::from_str(response_json).map_err(|e| format!("Invalid response JSON: {}", e))?;
    let driver = OpenAiDriver::new("wasm-browser", wasm_browser_caps());
    let dr = driver
        .parse_response(&body)
        .map_err(|e| format!("parse_response: {:?}", e))?;

    let u64_to_i = |n: u64| n as i64;

    let (reasoning_tokens, cache_read_tokens, cache_creation_tokens) = match &dr.usage {
        Some(u) => (
            u.reasoning_tokens.map(u64_to_i).unwrap_or(0),
            u.cache_read_tokens.map(u64_to_i).unwrap_or(0),
            u.cache_creation_tokens.map(u64_to_i).unwrap_or(0),
        ),
        None => (0, 0, 0),
    };
    let (prompt_tokens, completion_tokens, total_tokens) = match &dr.usage {
        Some(u) => (
            u64_to_i(u.prompt_tokens),
            u64_to_i(u.completion_tokens),
            u64_to_i(u.total_tokens),
        ),
        None => (0, 0, 0),
    };

    Ok(ParsedResponse {
        content: dr.content.unwrap_or_default(),
        finish_reason: dr
            .finish_reason
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        prompt_tokens,
        completion_tokens,
        total_tokens,
        reasoning_tokens,
        cache_read_tokens,
        cache_creation_tokens,
    })
}

/// Map `StreamingEvent` to the browser WASM (`event_type`, `payload`, `is_terminal`) triple.
fn map_streaming_to_wasm_tuple(ev: &StreamingEvent) -> Option<(String, String, bool)> {
    use StreamingEvent as E;
    match ev {
        E::PartialContentDelta { content, .. } if !content.is_empty() => {
            Some(("content_delta".into(), content.clone(), false))
        }
        E::ThinkingDelta { thinking, .. } if !thinking.is_empty() => {
            Some(("thinking_delta".into(), thinking.clone(), false))
        }
        E::StreamEnd { finish_reason } => Some((
            "stream_end".into(),
            finish_reason
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "stop".to_string()),
            true,
        )),
        E::StreamError { error, .. } => Some(("stream_error".into(), error.to_string(), false)),
        E::PartialToolCall {
            tool_call_id,
            arguments,
            index,
            ..
        } => {
            let mut m = serde_json::Map::new();
            m.insert("index".into(), json!(index.unwrap_or(0)));
            if !tool_call_id.is_empty() {
                m.insert("id".into(), json!(tool_call_id));
            }
            m.insert("arguments".into(), json!(arguments));
            Some((
                "tool_call_delta".into(),
                serde_json::Value::Object(m).to_string(),
                false,
            ))
        }
        _ => None,
    }
}

fn try_parse_with_drivers(data: &str) -> Option<(String, String, bool)> {
    let caps = wasm_browser_caps();
    let d_anth = AnthropicDriver::new("wasm-browser", caps.clone());
    if let Ok(Some(ev)) = d_anth.parse_stream_event(data) {
        if let Some(t) = map_streaming_to_wasm_tuple(&ev) {
            return Some(t);
        }
    }
    let d_open = OpenAiDriver::new("wasm-browser", caps);
    if let Ok(Some(ev)) = d_open.parse_stream_event(data) {
        if let Some(t) = map_streaming_to_wasm_tuple(&ev) {
            return Some(t);
        }
    }
    None
}

/// OpenAI-style `choices[0].*` path not fully covered by `OpenAiDriver` (e.g. streaming
/// `tool_calls` array fragments, `role` delta, or nonstandard `finish_reason` values).
fn parse_stream_openai_choices_residue(event: &Value) -> Option<(String, String, bool)> {
    let ch0 = event.get("choices")?.get(0)?;
    if let Some(fr) = ch0.get("finish_reason").and_then(|f| f.as_str()) {
        if ["stop", "length", "content_filter", "tool_calls"].contains(&fr) {
            return Some(("stream_end".into(), fr.to_string(), true));
        }
    }
    let delta = ch0.get("delta")?;
    if let Some(calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
        if let Some(first) = calls.first() {
            let index = first.get("index").and_then(|v| v.as_i64()).unwrap_or(0);
            let id = first.get("id").and_then(|v| v.as_str());
            let func = first.get("function");
            let name = func.and_then(|f| f.get("name")).and_then(|v| v.as_str());
            let args = func
                .and_then(|f| f.get("arguments"))
                .and_then(|v| v.as_str());
            let mut payload = serde_json::Map::new();
            payload.insert("index".into(), serde_json::Value::from(index));
            if let Some(id) = id {
                payload.insert("id".into(), serde_json::Value::from(id));
            }
            if let Some(name) = name {
                payload.insert("name".into(), serde_json::Value::from(name));
            }
            if let Some(args) = args {
                payload.insert("arguments".into(), serde_json::Value::from(args));
            }
            return Some((
                "tool_call_delta".into(),
                serde_json::Value::Object(payload).to_string(),
                false,
            ));
        }
    }
    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
        return Some(("content_delta".into(), content.to_string(), false));
    }
    if let Some(t) = delta.get("reasoning_content").and_then(|c| c.as_str()) {
        if !t.is_empty() {
            return Some(("thinking_delta".into(), t.to_string(), false));
        }
    }
    if let Some(t) = delta.get("reasoning").and_then(|c| c.as_str()) {
        if !t.is_empty() {
            return Some(("thinking_delta".into(), t.to_string(), false));
        }
    }
    if delta.get("role").is_some() {
        return Some(("role_assign".into(), String::new(), false));
    }
    None
}

/// Parse a single SSE stream event data payload.
///
/// First delegates to `ai-lib-core` drivers (Anthropic then OpenAI), then falls
/// back to a small **OpenAI streaming residue** for `tool_calls` / `role` deltas.
/// Returns `(event_type, payload, is_terminal)`.
pub fn parse_stream_event(data: &str) -> Result<(String, String, bool), String> {
    if is_stream_done(data) {
        return Ok(("stream_end".to_string(), String::new(), true));
    }
    if let Some(t) = try_parse_with_drivers(data) {
        return Ok(t);
    }
    let event: Value = serde_json::from_str(data.trim())
        .map_err(|e| format!("Invalid stream event JSON: {}", e))?;
    if let Some(t) = parse_stream_openai_choices_residue(&event) {
        return Ok(t);
    }
    Ok(("unknown".to_string(), String::new(), false))
}

/// Classify an HTTP error status code using ai-lib-core's StandardErrorCode.
pub fn classify_error(status_code: u16) -> (u32, String, String, bool) {
    let ec = StandardErrorCode::from_http_status(status_code);

    let name = ec.name().to_string();
    let category = ec.category().to_string();
    let retryable = ec.retryable();

    (status_code as u32, name, category, retryable)
}

/// Check if an SSE data payload signals stream completion.
pub fn is_stream_done(data: &str) -> bool {
    data.trim() == "[DONE]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_chat_request_basic() {
        let messages = r#"[{"role":"user","content":"Hello"}]"#;
        let result = build_chat_request(messages, "gpt-4", 0.7, 1024.0, false);
        assert!(result.is_ok());
        let (body, stream) = result.unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["model"], "gpt-4");
        assert!(!stream);
    }

    #[test]
    fn test_build_chat_request_streaming() {
        let messages = r#"[{"role":"user","content":"Hi"}]"#;
        let result = build_chat_request(messages, "llama-3", 0.5, 2048.0, true);
        assert!(result.is_ok());
        let (_, stream) = result.unwrap();
        assert!(stream);
    }

    #[test]
    fn test_build_chat_request_invalid_json() {
        let result = build_chat_request("not json", "model", 0.7, 100.0, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_chat_request_with_system() {
        let messages =
            r#"[{"role":"system","content":"You are helpful"},{"role":"user","content":"Hi"}]"#;
        let result = build_chat_request(messages, "test-model", 0.7, 100.0, false);
        assert!(result.is_ok());
        let (body, _) = result.unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let msgs = parsed["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn test_build_chat_request_tool_message_includes_tool_call_id() {
        let messages = r#"[
            {"role":"user","content":"q"},
            {"role":"tool","content":"r","tool_call_id":"call_1"}
        ]"#;
        let result = build_chat_request(messages, "m", 0.7, 100.0, false);
        assert!(result.is_ok());
        let (body, _) = result.unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let msgs = parsed["messages"].as_array().unwrap();
        assert_eq!(msgs[1]["role"], "tool");
        assert_eq!(msgs[1]["tool_call_id"], "call_1");
    }

    #[test]
    fn test_parse_chat_response_basic() {
        let response = r#"{
            "choices": [{"message":{"content":"Hello!"},"finish_reason":"stop"}],
            "usage": {"prompt_tokens":5,"completion_tokens":2,"total_tokens":7}
        }"#;
        let result = parse_chat_response(response).unwrap();
        assert_eq!(result.content, "Hello!");
        assert_eq!(result.finish_reason, "stop");
        assert_eq!(result.prompt_tokens, 5);
        assert_eq!(result.completion_tokens, 2);
        assert_eq!(result.total_tokens, 7);
    }

    #[test]
    fn test_parse_chat_response_empty() {
        let response = r#"{"choices":[]}"#;
        let result = parse_chat_response(response).unwrap();
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_parse_chat_response_openai_reasoning_tokens() {
        let response = r#"{
            "choices": [{"message":{"content":"ok"},"finish_reason":"stop"}],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30,
                "completion_tokens_details": {"reasoning_tokens": 7},
                "prompt_tokens_details": {"cached_tokens": 4}
            }
        }"#;
        let r = parse_chat_response(response).unwrap();
        assert_eq!(r.reasoning_tokens, 7);
        assert_eq!(r.cache_read_tokens, 4);
        assert_eq!(r.cache_creation_tokens, 0);
    }

    #[test]
    fn test_parse_chat_response_anthropic_usage() {
        let response = r#"{
            "choices": [{"message":{"content":"hi"},"finish_reason":"stop"}],
            "usage": {
                "input_tokens": 12,
                "output_tokens": 3,
                "cache_creation_input_tokens": 5,
                "cache_read_input_tokens": 2
            }
        }"#;
        let r = parse_chat_response(response).unwrap();
        assert_eq!(r.prompt_tokens, 12);
        assert_eq!(r.completion_tokens, 3);
        assert_eq!(r.total_tokens, 15);
        assert_eq!(r.cache_creation_tokens, 5);
        assert_eq!(r.cache_read_tokens, 2);
    }

    #[test]
    fn test_parse_stream_event_anthropic_thinking() {
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"let me..."}}"#;
        let r = parse_stream_event(data).unwrap();
        assert_eq!(r.0, "thinking_delta");
        assert_eq!(r.1, "let me...");
        assert!(!r.2);
    }

    #[test]
    fn test_parse_stream_event_anthropic_text() {
        let data =
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hi"}}"#;
        let r = parse_stream_event(data).unwrap();
        assert_eq!(r.0, "content_delta");
        assert_eq!(r.1, "hi");
    }

    #[test]
    fn test_parse_stream_event_anthropic_message_stop() {
        let data = r#"{"type":"message_stop"}"#;
        let r = parse_stream_event(data).unwrap();
        assert_eq!(r.0, "stream_end");
        assert!(r.2);
    }

    #[test]
    fn test_parse_stream_event_openai_tool_call() {
        let data = r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"get_weather","arguments":"{\"city\":"}}]}}]}"#;
        let r = parse_stream_event(data).unwrap();
        assert_eq!(r.0, "tool_call_delta");
        let v: serde_json::Value = serde_json::from_str(&r.1).unwrap();
        assert_eq!(v["index"], 0);
        assert_eq!(v["id"], "call_1");
        assert_eq!(v["name"], "get_weather");
    }

    #[test]
    fn test_parse_chat_response_invalid() {
        let result = parse_chat_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_stream_event_content() {
        let data = r#"{"choices":[{"delta":{"content":"Hi"}}]}"#;
        let result = parse_stream_event(data).unwrap();
        assert_eq!(result.0, "content_delta");
        assert_eq!(result.1, "Hi");
        assert!(!result.2);
    }

    #[test]
    fn test_parse_stream_event_done() {
        let result = parse_stream_event("[DONE]").unwrap();
        assert_eq!(result.0, "stream_end");
        assert!(result.2);
    }

    #[test]
    fn test_parse_stream_event_finish_reason() {
        let data = r#"{"choices":[{"delta":{},"finish_reason":"stop"}]}"#;
        let result = parse_stream_event(data).unwrap();
        assert_eq!(result.0, "stream_end");
        assert!(result.2);
    }

    #[test]
    fn test_parse_stream_event_role() {
        let data = r#"{"choices":[{"delta":{"role":"assistant"}}]}"#;
        let result = parse_stream_event(data).unwrap();
        assert_eq!(result.0, "role_assign");
        assert!(!result.2);
    }

    #[test]
    fn test_parse_stream_event_thinking() {
        let data = r#"{"choices":[{"delta":{"reasoning_content":"Let me think..."}}]}"#;
        let result = parse_stream_event(data).unwrap();
        assert_eq!(result.0, "thinking_delta");
        assert_eq!(result.1, "Let me think...");
    }

    #[test]
    fn test_parse_stream_event_invalid() {
        let result = parse_stream_event("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_classify_error_rate_limit() {
        let result = classify_error(429);
        assert_eq!(result.0, 429);
        assert_eq!(result.2, "rate");
        assert!(result.3);
    }

    #[test]
    fn test_classify_error_auth() {
        let result = classify_error(401);
        assert_eq!(result.2, "client");
        assert!(!result.3);
    }

    #[test]
    fn test_classify_error_server() {
        let result = classify_error(500);
        assert_eq!(result.2, "server");
        assert!(result.3);
    }

    #[test]
    fn test_classify_error_not_found() {
        let result = classify_error(404);
        assert_eq!(result.2, "client");
        assert!(!result.3);
    }

    #[test]
    fn test_is_stream_done_true() {
        assert!(is_stream_done("[DONE]"));
        assert!(is_stream_done(" [DONE] "));
    }

    #[test]
    fn test_is_stream_done_false() {
        assert!(!is_stream_done("{\"choices\":[]}"));
        assert!(!is_stream_done("data: [DONE]"));
    }
}
