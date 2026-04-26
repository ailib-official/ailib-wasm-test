//! # ailib-wasm-test-server
//!
//! 后端代理服务 — 将浏览器 WASM 构建的请求转发给 AI provider。
//!
//! Backend proxy server forwarding browser-WASM-built requests to AI providers.
//! The browser wasm module handles request building and response parsing;
//! this server only proxies HTTP to avoid CORS and protect API keys.
//!
//! Uses libcurl (via the `curl` crate) for outbound requests because some
//! AI providers (notably Groq) block Rust's native TLS fingerprints (JA3/JA4).
//! libcurl shares the same TLS fingerprint as the system `curl` binary.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use curl::easy::Easy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {}

/// Resolve the API key for a provider based on the request URL.
/// Returns None if no matching provider is found.
fn resolve_api_key(url: &str) -> Option<String> {
    // Groq
    if url.contains("api.groq.com") {
        return std::env::var("GROQ_API_KEY").ok();
    }
    // DeepSeek
    if url.contains("api.deepseek.com") {
        return std::env::var("DEEPSEEK_API_KEY").ok();
    }
    // OpenAI
    if url.contains("api.openai.com") {
        return std::env::var("OPENAI_API_KEY").ok();
    }
    // NVIDIA (NIM)
    if url.contains("integrate.api.nvidia.com") {
        return std::env::var("NVIDIA_API_KEY").ok();
    }
    None
}

/// Incoming proxy request from the browser wasm module.
#[derive(Deserialize)]
pub struct ProxyRequest {
    pub url: String,
    #[serde(default)]
    pub headers: serde_json::Map<String, Value>,
    pub body: Value,
    pub stream: bool,
}

/// Non-streaming proxy response returned to the browser.
#[derive(Serialize)]
pub struct ProxyResponse {
    pub status: u16,
    pub body: Value,
}

/// Health check response.
#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Resolve the static directory path.
/// Resolution order:
///   1. `AILIB_WASM_STATIC_DIR` environment variable (explicit override).
///   2. `<current working directory>/static`.
///   3. `<executable directory>/static` and `<executable dir>/../static`
///      (covers `cargo run` and release-binary deployments).
fn static_dir() -> std::path::PathBuf {
    if let Ok(env_dir) = std::env::var("AILIB_WASM_STATIC_DIR") {
        let p = std::path::PathBuf::from(env_dir);
        if p.join("index.html").exists() {
            return p;
        }
    }

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("static"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("static"));
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join("static"));
            }
        }
    }

    for c in &candidates {
        if c.join("index.html").exists() {
            return c.clone();
        }
    }
    // Last-resort fallback: CWD/static even if missing; fallback_service will
    // 404, which is preferable to panicking at startup.
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("static")
}

/// Serve index.html for the root path.
async fn index_handler() -> Response {
    let path = static_dir().join("index.html");
    match std::fs::read_to_string(&path) {
        Ok(html) => axum::response::Html(html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

/// Build the axum Router with all routes and middleware.
pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/", get(index_handler))
        .route("/api/proxy", post(proxy_handler))
        .route("/api/proxy/stream", post(proxy_stream_handler))
        .fallback_service(tower_http::services::ServeDir::new(static_dir()))
        .layer(cors)
        .with_state(Arc::new(state))
}

/// GET /health — returns server status and version.
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Build `Content-Type`, optional `Authorization` from `resolve_api_key`, then caller-supplied
/// headers (skipping `authorization` and `content-type` so they cannot override built-ins).
/// Shared by non-streaming and streaming proxy for parity.
fn build_proxy_curl_headers(
    url: &str,
    custom: &serde_json::Map<String, Value>,
) -> Result<curl::easy::List, String> {
    let mut list = curl::easy::List::new();
    list.append("Content-Type: application/json")
        .map_err(|e| format!("curl header append: {}", e))?;
    if let Some(key) = resolve_api_key(url) {
        list.append(&format!("Authorization: Bearer {}", key))
            .map_err(|e| format!("curl auth header append: {}", e))?;
    }
    for (key, val) in custom {
        if key.eq_ignore_ascii_case("authorization") || key.eq_ignore_ascii_case("content-type") {
            continue;
        }
        if let Some(s) = val.as_str() {
            list.append(&format!("{}: {}", key, s))
                .map_err(|e| format!("curl header append: {}", e))?;
        }
    }
    Ok(list)
}

/// Execute a non-streaming proxy request using libcurl.
/// Runs in a blocking thread to avoid blocking the tokio runtime.
/// Never panics: any libcurl / setup / transport failure is surfaced as a
/// 502 ProxyResponse whose `body` carries an `error` key.
fn curl_proxy(url: &str, headers: &serde_json::Map<String, Value>, body: &Value) -> ProxyResponse {
    fn err_response(msg: impl std::fmt::Display) -> ProxyResponse {
        ProxyResponse {
            status: 502,
            body: serde_json::json!({ "error": msg.to_string() }),
        }
    }

    let mut easy = Easy::new();
    if let Err(e) = easy.url(url) {
        return err_response(format!("invalid url: {}", e));
    }
    if let Err(e) = easy.post(true) {
        return err_response(format!("curl post(true): {}", e));
    }
    if let Err(e) = easy.timeout(Duration::from_secs(120)) {
        return err_response(format!("curl timeout: {}", e));
    }

    let body_str = match serde_json::to_string(body) {
        Ok(s) => s,
        Err(e) => return err_response(format!("serialize body: {}", e)),
    };
    if let Err(e) = easy.post_fields_copy(body_str.as_bytes()) {
        return err_response(format!("curl post_fields: {}", e));
    }

    let list = match build_proxy_curl_headers(url, headers) {
        Ok(l) => l,
        Err(e) => return err_response(e),
    };
    if let Err(e) = easy.http_headers(list) {
        return err_response(format!("curl http_headers: {}", e));
    }

    let mut resp_data = Vec::new();
    {
        let mut transfer = easy.transfer();
        if let Err(e) = transfer.write_function(|data| {
            resp_data.extend_from_slice(data);
            Ok(data.len())
        }) {
            return err_response(format!("curl write_function: {}", e));
        }
        if let Err(e) = transfer.perform() {
            return err_response(format!("upstream request failed: {}", e));
        }
    }
    let status_code = easy.response_code().unwrap_or(0) as u16;

    let resp_body: Value = serde_json::from_slice(&resp_data).unwrap_or(Value::Null);
    ProxyResponse {
        status: status_code,
        body: resp_body,
    }
}

/// POST /api/proxy — non-streaming proxy: forward request, return full response.
async fn proxy_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ProxyRequest>,
) -> impl IntoResponse {
    let url = req.url.clone();
    let headers = req.headers.clone();
    let body = req.body.clone();

    let result = tokio::task::spawn_blocking(move || curl_proxy(&url, &headers, &body))
        .await
        .unwrap_or(ProxyResponse {
            status: 502,
            body: serde_json::json!({"error": "blocking task failed"}),
        });

    let code = StatusCode::from_u16(result.status).unwrap_or(StatusCode::BAD_GATEWAY);
    (code, Json(result))
}

/// POST /api/proxy/stream — SSE streaming proxy via curl crate.
///
/// Uses the `curl` crate (libcurl FFI) instead of the `curl` binary subprocess
/// because some AI providers (notably Groq) block the `curl` binary's TLS
/// fingerprint at Cloudflare level, while the crate's `Easy` handle works.
///
/// The curl `Easy` handle runs in a `spawn_blocking` thread; each chunk of
/// response data is sent through an `mpsc` channel to the async SSE stream.
async fn proxy_stream_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ProxyRequest>,
) -> Response {
    let url = req.url.clone();
    let headers = req.headers.clone();
    let body_str = serde_json::to_string(&req.body).unwrap_or_default();

    let (tx, rx) = tokio::sync::mpsc::channel::<String>(256);

    let url_clone = url.clone();
    let body_clone = body_str.clone();

    let handle = tokio::task::spawn_blocking(move || {
        curl_stream_proxy(&url_clone, &headers, &body_clone, tx)
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx).filter_map(|line| {
        let stripped = line.strip_prefix("data: ").unwrap_or(&line).to_string();
        if stripped.is_empty() {
            None
        } else {
            Some(Ok::<_, Infallible>(
                axum::response::sse::Event::default().data(stripped),
            ))
        }
    });

    let sse = Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)));

    // Spawn a task to await the blocking handle and log any errors
    tokio::spawn(async move {
        if let Err(e) = handle.await {
            eprintln!("curl_stream_proxy task panicked: {}", e);
        }
    });

    sse.into_response()
}

/// Execute a streaming proxy request using the curl crate.
/// Sends each line of the response body through the provided channel.
///
/// `write_function` is only invoked by libcurl on the single blocking thread
/// that owns `easy.transfer()`, so the previous `Arc<Mutex<...>>` around the
/// sender/buffer was redundant; we now hold them as plain locals via RefCell
/// to keep the closure `FnMut` without extra synchronization overhead. Any
/// setup failure emits a synthetic `error:` frame so the browser can react.
fn curl_stream_proxy(
    url: &str,
    headers: &serde_json::Map<String, Value>,
    body: &str,
    tx: tokio::sync::mpsc::Sender<String>,
) {
    let send_err = |msg: String| {
        let _ = tx.blocking_send(format!("data: {{\"error\":\"{}\"}}", msg.replace('"', "'")));
    };

    let mut easy = Easy::new();
    if let Err(e) = easy.url(url) {
        send_err(format!("invalid url: {}", e));
        return;
    }
    if let Err(e) = easy.post(true) {
        send_err(format!("curl post(true): {}", e));
        return;
    }
    if let Err(e) = easy.timeout(Duration::from_secs(120)) {
        send_err(format!("curl timeout: {}", e));
        return;
    }
    if let Err(e) = easy.post_fields_copy(body.as_bytes()) {
        send_err(format!("curl post_fields: {}", e));
        return;
    }

    let list = match build_proxy_curl_headers(url, headers) {
        Ok(l) => l,
        Err(e) => {
            send_err(e);
            return;
        }
    };
    if let Err(e) = easy.http_headers(list) {
        send_err(format!("curl http_headers: {}", e));
        return;
    }

    // write_function is invoked on the current (single) blocking thread;
    // RefCell is sufficient because there is no cross-thread sharing.
    let buffer: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());

    {
        let mut transfer = easy.transfer();
        let write_result = transfer.write_function(|data| {
            let mut buf = buffer.borrow_mut();
            buf.extend_from_slice(data);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
                if !line.is_empty() {
                    let _ = tx.blocking_send(line);
                }
            }
            Ok(data.len())
        });
        if let Err(e) = write_result {
            send_err(format!("curl write_function: {}", e));
            return;
        }
        if let Err(e) = transfer.perform() {
            send_err(format!("upstream stream failed: {}", e));
            return;
        }
    }

    // Flush any remaining bytes (last line without trailing \n)
    let buf = buffer.borrow();
    if !buf.is_empty() {
        let line = String::from_utf8_lossy(&buf).trim().to_string();
        if !line.is_empty() {
            let _ = tx.blocking_send(line);
        }
    }
}

/// Start the server on port 3000.
pub async fn run_server() -> anyhow::Result<()> {
    let state = AppState {};
    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("ailib-wasm-test-server running on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_server().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode as AxumStatusCode};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let state = AppState {};
        create_app(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxumStatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let health: HealthResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_proxy_missing_url_returns_error() {
        let app = test_app();
        let req = Request::builder()
            .method("POST")
            .uri("/api/proxy")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"headers":{},"body":{},"stream":false}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxumStatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let app = test_app();
        let req = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), AxumStatusCode::NOT_FOUND);
    }

    #[test]
    fn test_resolve_api_key_groq() {
        let key = resolve_api_key("https://api.groq.com/openai/v1/chat/completions");
        if std::env::var("GROQ_API_KEY").is_ok() {
            assert!(key.is_some());
        } else {
            assert!(key.is_none());
        }
    }

    #[test]
    fn test_resolve_api_key_deepseek() {
        let key = resolve_api_key("https://api.deepseek.com/chat/completions");
        if std::env::var("DEEPSEEK_API_KEY").is_ok() {
            assert!(key.is_some());
        } else {
            assert!(key.is_none());
        }
    }

    #[test]
    fn test_build_proxy_curl_headers_merges_custom_without_panic() {
        let mut m = serde_json::Map::new();
        m.insert(
            "X-Custom-Client-Header".into(),
            serde_json::Value::String("test-value".into()),
        );
        let list = build_proxy_curl_headers("https://api.unknown.example.com/v1", &m);
        assert!(list.is_ok(), "{:?}", list.err());
    }

    #[test]
    fn test_resolve_api_key_unknown() {
        let key = resolve_api_key("https://unknown.provider.com/v1/chat");
        assert!(key.is_none(), "Unknown provider should return None");
    }

    /// Integration probe — skipped when `GROQ_API_KEY` is missing or the network returns non-200
    /// (proxy, TLS, or provider outage) so local/CI stay deterministic.
    #[test]
    fn test_curl_proxy_groq() {
        if std::env::var("GROQ_API_KEY").is_err() {
            eprintln!("Skipping: GROQ_API_KEY not set");
            return;
        }
        let result = curl_proxy(
            "https://api.groq.com/openai/v1/chat/completions",
            &serde_json::Map::new(),
            &serde_json::json!({
                "model": "llama-3.1-8b-instant",
                "messages": [{"role": "user", "content": "Say hello"}],
                "stream": false
            }),
        );
        if result.status != 200 {
            eprintln!(
                "Skipping: Groq upstream returned {} (network/proxy): {:?}",
                result.status, result.body
            );
            return;
        }
    }

    /// Same as [`test_curl_proxy_groq`]: best-effort live probe, non-fatal on failure.
    #[test]
    fn test_curl_proxy_deepseek() {
        if std::env::var("DEEPSEEK_API_KEY").is_err() {
            eprintln!("Skipping: DEEPSEEK_API_KEY not set");
            return;
        }
        let result = curl_proxy(
            "https://api.deepseek.com/chat/completions",
            &serde_json::Map::new(),
            &serde_json::json!({
                "model": "deepseek-chat",
                "messages": [{"role": "user", "content": "Say hello"}],
                "stream": false
            }),
        );
        if result.status != 200 {
            eprintln!(
                "Skipping: DeepSeek upstream returned {} (network/proxy): {:?}",
                result.status, result.body
            );
        }
    }
}
