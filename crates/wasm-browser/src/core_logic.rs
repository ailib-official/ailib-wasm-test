//! # core_logic — Pure Rust protocol logic, testable on native target.
//!
//! 核心协议逻辑 — 纯 Rust 实现，不依赖 wasm-bindgen，可在原生目标上测试。
//!
//! Each function returns `Result<..., String>` so the wasm_bindgen wrappers
//! only need to convert `String` → `JsValue`.

use ai_lib_core::drivers::{OpenAiDriver, ProviderDriver};
use ai_lib_core::error_code::StandardErrorCode;
use ai_lib_core::protocol::v2::capabilities::Capability;
use ai_lib_core::types::message::Message;
use serde_json::Value;

/// Build a chat completion request body using ai-lib-core's OpenAiDriver.
pub fn build_chat_request(
    messages_json: &str,
    model: &str,
    temperature: f64,
    max_tokens: f64,
    stream: bool,
) -> Result<(String, bool), String> {
    let messages: Vec<Value> =
        serde_json::from_str(messages_json).map_err(|e| format!("Invalid messages JSON: {}", e))?;

    let ai_messages: Vec<Message> = messages
        .iter()
        .map(|m| {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
            match role {
                "system" => Message::system(content),
                "assistant" => Message::assistant(content),
                _ => Message::user(content),
            }
        })
        .collect();

    let driver = OpenAiDriver::new(
        "wasm-browser",
        vec![Capability::Text, Capability::Streaming],
    );
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
/// Token extraction is **ARCH-003 aligned with ai-lib-ts/ai-lib-go**: reasoning
/// tokens are pulled from either the flat `reasoning_tokens` or the OpenAI
/// nested `usage.completion_tokens_details.reasoning_tokens`; cache-read tokens
/// accept `cache_read_tokens`, the OpenAI `usage.prompt_tokens_details.cached_tokens`,
/// and Anthropic `cache_read_input_tokens`; cache-creation tokens accept
/// `cache_creation_tokens`, Anthropic `cache_creation_input_tokens`, and the
/// legacy `cache_write_tokens` alias.
pub fn parse_chat_response(response_json: &str) -> Result<ParsedResponse, String> {
    let resp: Value =
        serde_json::from_str(response_json).map_err(|e| format!("Invalid response JSON: {}", e))?;

    let content = resp
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let finish_reason = resp
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("finish_reason"))
        .and_then(|f| f.as_str())
        .unwrap_or("unknown")
        .to_string();

    let usage = resp.get("usage");
    let flat = |key: &str| -> i64 {
        usage
            .and_then(|u| u.get(key))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    };
    let nested = |outer: &str, inner: &str| -> i64 {
        usage
            .and_then(|u| u.get(outer))
            .and_then(|d| d.get(inner))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    };
    let first_nonzero = |vals: &[i64]| -> i64 { *vals.iter().find(|&&v| v != 0).unwrap_or(&0) };

    // Accept OpenAI (prompt_tokens/completion_tokens) and Anthropic (input_tokens/output_tokens)
    let prompt_tokens = first_nonzero(&[flat("prompt_tokens"), flat("input_tokens")]);
    let completion_tokens = first_nonzero(&[flat("completion_tokens"), flat("output_tokens")]);
    let mut total_tokens = flat("total_tokens");
    if total_tokens == 0 && (prompt_tokens > 0 || completion_tokens > 0) {
        total_tokens = prompt_tokens + completion_tokens;
    }

    let reasoning_tokens = first_nonzero(&[
        flat("reasoning_tokens"),
        nested("completion_tokens_details", "reasoning_tokens"),
    ]);
    let cache_read_tokens = first_nonzero(&[
        flat("cache_read_tokens"),
        nested("prompt_tokens_details", "cached_tokens"),
        flat("cache_read_input_tokens"),
    ]);
    let cache_creation_tokens = first_nonzero(&[
        flat("cache_creation_tokens"),
        flat("cache_creation_input_tokens"),
        flat("cache_write_tokens"),
    ]);

    Ok(ParsedResponse {
        content,
        finish_reason,
        prompt_tokens,
        completion_tokens,
        total_tokens,
        reasoning_tokens,
        cache_read_tokens,
        cache_creation_tokens,
    })
}

/// Parse a single SSE stream event data payload.
///
/// Supports both the OpenAI-compatible format (`choices[0].delta.*`) and the
/// Anthropic Messages streaming format (`type: content_block_delta` with
/// `delta.type = text_delta | thinking_delta`, plus `message_stop`). Returns
/// `(event_type, payload, is_terminal)`. `event_type` values:
/// - `content_delta` — regular assistant text delta
/// - `thinking_delta` — reasoning / thinking stream (gen-002)
/// - `tool_call_delta` — JSON fragment of a streaming tool call (gen-004);
///   `payload` is a compact JSON object `{"index":i,"id":?,"name":?,"arguments":?}`
/// - `role_assign` — initial role assignment
/// - `stream_end` — terminal `[DONE]` / `stop` / Anthropic `message_stop`
/// - `unknown` — no recognized delta content
pub fn parse_stream_event(data: &str) -> Result<(String, String, bool), String> {
    if is_stream_done(data) {
        return Ok(("stream_end".to_string(), "".to_string(), true));
    }

    let event: Value =
        serde_json::from_str(data).map_err(|e| format!("Invalid stream event JSON: {}", e))?;

    // --- Anthropic Messages streaming ---
    if let Some(kind) = event.get("type").and_then(|t| t.as_str()) {
        match kind {
            "message_stop" => return Ok(("stream_end".to_string(), "stop".to_string(), true)),
            "content_block_delta" => {
                if let Some(delta) = event.get("delta") {
                    match delta.get("type").and_then(|t| t.as_str()) {
                        Some("text_delta") => {
                            let text = delta.get("text").and_then(|t| t.as_str()).unwrap_or("");
                            return Ok(("content_delta".to_string(), text.to_string(), false));
                        }
                        Some("thinking_delta") => {
                            let text = delta
                                .get("thinking")
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            return Ok(("thinking_delta".to_string(), text.to_string(), false));
                        }
                        Some("input_json_delta") => {
                            let partial = delta
                                .get("partial_json")
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            let payload = serde_json::json!({
                                "index": event.get("index").and_then(|v| v.as_i64()).unwrap_or(0),
                                "arguments": partial,
                            });
                            return Ok((
                                "tool_call_delta".to_string(),
                                payload.to_string(),
                                false,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            "message_delta" => {
                // Anthropic stop_reason lives here
                if let Some(reason) = event
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|r| r.as_str())
                {
                    return Ok(("stream_end".to_string(), reason.to_string(), true));
                }
            }
            _ => {}
        }
    }

    // --- OpenAI-compatible streaming ---
    let delta = event
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("delta"));

    let finish_reason = event
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("finish_reason"))
        .and_then(|f| f.as_str());

    if let Some(reason) = finish_reason {
        if reason == "stop"
            || reason == "length"
            || reason == "content_filter"
            || reason == "tool_calls"
        {
            return Ok(("stream_end".to_string(), reason.to_string(), true));
        }
    }

    if let Some(delta) = delta {
        // OpenAI tool_calls delta — surface the first fragment as JSON so the
        // JS-side ToolCallAccumulator can concatenate arguments across frames.
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
                return Ok((
                    "tool_call_delta".to_string(),
                    serde_json::Value::Object(payload).to_string(),
                    false,
                ));
            }
        }
        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
            return Ok(("content_delta".to_string(), content.to_string(), false));
        }
        // reasoning_content (DeepSeek/Qwen style) or OpenAI reasoning (o1) via `reasoning`
        if let Some(thinking) = delta.get("reasoning_content").and_then(|t| t.as_str()) {
            return Ok(("thinking_delta".to_string(), thinking.to_string(), false));
        }
        if let Some(thinking) = delta.get("reasoning").and_then(|t| t.as_str()) {
            return Ok(("thinking_delta".to_string(), thinking.to_string(), false));
        }
        if delta.get("role").is_some() {
            return Ok(("role_assign".to_string(), String::new(), false));
        }
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
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hi"}}"#;
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
