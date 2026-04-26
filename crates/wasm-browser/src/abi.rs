//! Unified versioned entry point for the browser WASM module (WASM-001).
//!
//! Hosts pass a single JSON object with `"op"` and op-specific fields. This
//! mirrors `ai-lib-wasm::ailib_invoke` on the WASI side so policy-free protocol
//! helpers stay consistent across runtimes (ARCH-003).

use serde_json::{json, Value};

/// ABI version of the wasm-bindgen surface (independent from WASI `ai-lib-wasm`).
pub const BROWSER_ABI_VERSION: u32 = 1;

fn require_op(v: &Value) -> Result<&str, String> {
    v.get("op")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing \"op\"".to_string())
}

fn require_u16(v: &Value, key: &str) -> Result<u16, String> {
    v.get(key)
        .and_then(|x| x.as_u64())
        .filter(|&n| n <= u16::MAX as u64)
        .map(|n| n as u16)
        .ok_or_else(|| format!("missing or invalid \"{}\"", key))
}

/// Dispatch JSON `{ "op": "...", ... }` to core_logic. Returns a JSON string payload.
pub fn invoke(input_json: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(input_json).map_err(|e| format!("input json: {}", e))?;
    let ctx_version = v.get("version").and_then(|x| x.as_u64()).unwrap_or(1) as u32;
    if ctx_version > BROWSER_ABI_VERSION {
        return Err(format!(
            "ctx version {} newer than BROWSER_ABI_VERSION {}",
            ctx_version, BROWSER_ABI_VERSION
        ));
    }

    match require_op(&v)? {
        "abi_version" => Ok(json!({ "version": BROWSER_ABI_VERSION }).to_string()),

        "capabilities" => Ok(json!({
            "version": BROWSER_ABI_VERSION,
            "ops": [
                "abi_version",
                "capabilities",
                "build_request",
                "parse_response",
                "parse_stream_event",
                "classify_error",
                "is_stream_done",
            ],
            "legacy_exports": [
                "build_chat_request",
                "parse_chat_response",
                "parse_stream_event",
                "classify_error",
                "is_stream_done",
            ],
        })
        .to_string()),

        "build_request" => {
            let messages = v
                .get("messages")
                .ok_or_else(|| "build_request: missing \"messages\"".to_string())?;
            let model = v
                .get("model")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "build_request: missing \"model\"".to_string())?;
            let temperature = v.get("temperature").and_then(|x| x.as_f64()).unwrap_or(0.7);
            let max_tokens = v
                .get("max_tokens")
                .and_then(|x| x.as_f64())
                .unwrap_or(4096.0);
            let stream = v.get("stream").and_then(|x| x.as_bool()).unwrap_or(false);

            let messages_json = serde_json::to_string(messages)
                .map_err(|e| format!("messages serialize: {}", e))?;
            let (body, stream_flag) = crate::core_logic::build_chat_request(
                &messages_json,
                model,
                temperature,
                max_tokens,
                stream,
            )?;
            Ok(json!({ "body": body, "stream": stream_flag }).to_string())
        }

        "parse_response" => {
            let response_json = v
                .get("response_json")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "parse_response: missing \"response_json\" string".to_string())?;
            let r = crate::core_logic::parse_chat_response(response_json)?;
            serde_json::to_string(&json!({
                "content": r.content,
                "finish_reason": r.finish_reason,
                "prompt_tokens": r.prompt_tokens,
                "completion_tokens": r.completion_tokens,
                "total_tokens": r.total_tokens,
                "reasoning_tokens": r.reasoning_tokens,
                "cache_read_tokens": r.cache_read_tokens,
                "cache_creation_tokens": r.cache_creation_tokens,
            }))
            .map_err(|e| format!("parse_response out: {}", e))
        }

        "parse_stream_event" => {
            let data = v
                .get("data")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "parse_stream_event: missing \"data\"".to_string())?;
            let (event_type, data_str, done) = crate::core_logic::parse_stream_event(data)?;
            Ok(json!({
                "event_type": event_type,
                "data": data_str,
                "done": done,
            })
            .to_string())
        }

        "classify_error" => {
            let status = require_u16(&v, "status_code")?;
            let (code, name, category, retryable) = crate::core_logic::classify_error(status);
            Ok(json!({
                "code": code,
                "name": name,
                "category": category,
                "retryable": retryable,
            })
            .to_string())
        }

        "is_stream_done" => {
            let data = v
                .get("data")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "is_stream_done: missing \"data\"".to_string())?;
            Ok(json!({ "done": crate::core_logic::is_stream_done(data) }).to_string())
        }

        other => Err(format!("unknown op: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoke_abi_version() {
        let out = invoke(r#"{"op":"abi_version"}"#).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["version"], BROWSER_ABI_VERSION);
    }

    #[test]
    fn invoke_capabilities() {
        let out = invoke(r#"{"op":"capabilities","version":1}"#).unwrap();
        assert!(out.contains("build_request"));
    }

    #[test]
    fn invoke_build_request() {
        let j = r#"{"op":"build_request","messages":[{"role":"user","content":"hi"}],"model":"m","stream":false}"#;
        let out = invoke(j).unwrap();
        assert!(out.contains("\"body\""));
    }

    #[test]
    fn invoke_v1_host_compat() {
        let j = r#"{"op":"abi_version","version":1}"#;
        assert!(invoke(j).is_ok());
    }

    #[test]
    fn invoke_rejects_future_ctx_version() {
        let j = format!(
            r#"{{"op":"abi_version","version":{}}}"#,
            BROWSER_ABI_VERSION + 99
        );
        assert!(invoke(&j).is_err());
    }
}
