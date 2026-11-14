//! Streamable HTTP JSON-RPC 2.0 MCP endpoint (`GET /mcp` & `POST /mcp`)
//! exposing all 8 `lnc-styles-gallery` tools (`styles__*` compatible).

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::server::AppState;
use crate::store::StyleLibraryStore;

pub const MCP_TOOL_NAMES: &[&str] = &[
    "list_styles",
    "search",
    "get_design",
    "list_collections",
    "get_template",
    "list_components",
    "get_component",
    "reindex",
];

pub fn mcp_tools_schema() -> Vec<Value> {
    vec![
        json!({
            "name": "list_styles",
            "description": "List Refero design styles with optional theme ('light'|'dark') and category/collection filters.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "theme": { "type": "string", "description": "Filter by color theme: 'light' or 'dark'." },
                    "category": { "type": "string", "description": "Filter by collection/category slug (e.g. 'editorial', 'terminal-dev', 'luxury')." },
                    "limit": { "type": "integer", "default": 48, "description": "Max items to return (1-500)." },
                    "offset": { "type": "integer", "default": 0, "description": "Pagination offset." }
                }
            }
        }),
        json!({
            "name": "search",
            "description": "Search Refero design styles (title, tagline, source_url, DESIGN.md tokens/rules) and HTML templates.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search keywords (e.g. 'monospace terminal', 'serif editorial', 'fintech dark')." },
                    "theme": { "type": "string", "description": "Optional theme filter ('light' or 'dark')." },
                    "category": { "type": "string", "description": "Optional category/collection slug." },
                    "include_content": { "type": "boolean", "default": true, "description": "Whether to search inside DESIGN.md content." },
                    "limit": { "type": "integer", "default": 25, "description": "Maximum results to return." }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "get_design",
            "description": "Retrieve full DESIGN.md markdown style guide and metadata for a style ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "style_id": { "type": "string", "description": "UUID or ID of the Refero style." }
                },
                "required": ["style_id"]
            }
        }),
        json!({
            "name": "list_collections",
            "description": "List all browsable style collections (by mood/category, theme, and sitemap) with counts and sample styles.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "get_template",
            "description": "Retrieve a prompt-driven-ui HTML template's metadata, generation prompt, and optionally its full HTML code.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "template_id": { "type": "string", "description": "Template ID (e.g. 'en-01-saas-landing' or slug/number)." },
                    "include_html": { "type": "boolean", "default": false, "description": "Include full raw HTML string in response." }
                },
                "required": ["template_id"]
            }
        }),
        json!({
            "name": "list_components",
            "description": "List UI effect/component recipes: LNC self-contained HTML demos (buttons, cards, loaders, backgrounds, AI-agent components) and Knowns UI TSX sources.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Filter by category (buttons, cards, loaders, inputs-toggles, backgrounds, text-effects, ai-agent, marketing, knowns-ui)." },
                    "kind": { "type": "string", "description": "Filter by kind: 'html' (live demo) or 'tsx' (React source)." },
                    "q": { "type": "string", "description": "Search name, description, or tags." },
                    "limit": { "type": "integer", "default": 100 },
                    "offset": { "type": "integer", "default": 0 }
                }
            }
        }),
        json!({
            "name": "get_component",
            "description": "Get a UI component's metadata; pass include_source=true to receive the full HTML demo source or all TSX file contents.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "component_id": { "type": "string", "description": "Component ID (e.g. 'lnc-aurora-bg' or 'knowns-button')." },
                    "include_source": { "type": "boolean", "default": false }
                },
                "required": ["component_id"]
            }
        }),
        json!({
            "name": "reindex",
            "description": "Re-scan the Refero Styles and Prompt-Driven UI corpus directories on disk without restarting the server.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
    ]
}

fn arg_usize(args: &Value, key: &str, default: usize) -> usize {
    match args.get(key) {
        Some(Value::Number(n)) => n.as_u64().map(|v| v as usize).unwrap_or(default),
        Some(Value::String(s)) => s.trim().parse::<usize>().unwrap_or(default),
        _ => default,
    }
}

fn arg_bool(args: &Value, key: &str, default: bool) -> bool {
    match args.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => default,
        },
        _ => default,
    }
}

fn arg_str<'a>(args: &'a Value, keys: &[&str]) -> Option<&'a str> {
    for k in keys {
        if let Some(s) = args.get(*k).and_then(Value::as_str) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

pub fn dispatch_tool_call(
    store: &StyleLibraryStore,
    raw_name: &str,
    arguments: &Value,
) -> Result<Value, String> {
    let mut name = raw_name.trim();
    while let Some(rest) = name.strip_prefix("styles__") {
        name = rest;
    }
    match name {
        "list_styles" => Ok(store.list_styles(
            arg_str(arguments, &["q"]),
            arg_str(arguments, &["theme"]),
            arg_str(arguments, &["category", "collection"]),
            arguments.get("has_design").and_then(Value::as_bool),
            arg_usize(arguments, "limit", 48),
            arg_usize(arguments, "offset", 0),
        )),
        "search" => {
            let q = arg_str(arguments, &["query", "q"]).unwrap_or("");
            Ok(store.search(
                q,
                arg_str(arguments, &["theme"]),
                arg_str(arguments, &["category"]),
                arg_bool(arguments, "include_content", true),
                arg_usize(arguments, "limit", 25),
            ))
        }
        "get_design" => {
            let sid = arg_str(arguments, &["style_id", "id"])
                .ok_or_else(|| "Missing required parameter: style_id".to_string())?;
            store
                .get_style(sid, true)
                .ok_or_else(|| format!("Style not found: {sid}"))
        }
        "list_collections" => {
            let cols = store.list_collections();
            Ok(json!({
                "total": cols.len(),
                "collections": cols,
            }))
        }
        "get_template" => {
            let tid = arg_str(arguments, &["template_id", "id"])
                .ok_or_else(|| "Missing required parameter: template_id".to_string())?;
            let include_html = arg_bool(arguments, "include_html", false);
            store
                .get_template(tid, include_html)
                .ok_or_else(|| format!("Template not found: {tid}"))
        }
        "list_components" => Ok(store.list_components(
            arg_str(arguments, &["category"]),
            arg_str(arguments, &["kind"]),
            arg_str(arguments, &["q"]),
            arg_usize(arguments, "limit", 100),
            arg_usize(arguments, "offset", 0),
        )),
        "get_component" => {
            let cid = arg_str(arguments, &["component_id", "id"])
                .ok_or_else(|| "Missing required parameter: component_id".to_string())?;
            let include_source = arg_bool(arguments, "include_source", false);
            store
                .get_component(cid, include_source)
                .ok_or_else(|| format!("Component not found: {cid}"))
        }
        "reindex" => Ok(store.reindex()),
        other => Err(format!("Unknown MCP tool: {other}")),
    }
}

pub fn handle_jsonrpc_message(
    store: &StyleLibraryStore,
    service_name: &str,
    service_version: &str,
    msg: &Value,
) -> Option<Value> {
    let Some(obj) = msg.as_object() else {
        return Some(json!({
            "jsonrpc": "2.0",
            "id": Value::Null,
            "error": { "code": -32600, "message": "Invalid Request" }
        }));
    };

    let method = obj.get("method").and_then(Value::as_str).unwrap_or("");
    let has_id = obj.contains_key("id") && !obj.get("id").is_some_and(Value::is_null);
    let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
    let params = obj.get("params").cloned().unwrap_or_else(|| json!({}));

    // JSON-RPC notifications (no `id` or `notifications/*` with null id) produce HTTP 202.
    if !has_id
        && (!obj.contains_key("id")
            || method.starts_with("notifications/")
            || method == "initialized")
    {
        return None;
    }

    if method.is_empty() {
        return Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": { "code": -32600, "message": "Invalid Request" }
        }));
    }

    match method {
        "initialize" => {
            let protocol_version = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("2024-11-05");
            Some(json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": protocol_version,
                    "capabilities": {
                        "tools": { "listChanged": false }
                    },
                    "serverInfo": {
                        "name": service_name,
                        "version": service_version
                    }
                }
            }))
        }
        "ping" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {}
        })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": mcp_tools_schema()
            }
        })),
        "tools/call" => {
            let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let tool_args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match dispatch_tool_call(store, tool_name, &tool_args) {
                Ok(payload) => {
                    let text = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into());
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "result": {
                            "content": [{ "type": "text", "text": text }],
                            "structuredContent": payload,
                            "isError": false
                        }
                    }))
                }
                Err(err_msg) => {
                    let text =
                        serde_json::to_string(&json!({ "error": err_msg })).unwrap_or_default();
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "result": {
                            "content": [{ "type": "text", "text": text }],
                            "isError": true
                        }
                    }))
                }
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {method}")
            }
        })),
    }
}

#[allow(clippy::result_large_err)]
fn check_mcp_auth(
    state: &AppState,
    headers: &HeaderMap,
    query: &HashMap<String, String>,
) -> Result<(), Response> {
    if !state.settings.mcp_auth_required {
        return Ok(());
    }
    let mut token = String::new();
    if let Some(auth) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if auth.len() > 7 && auth[..7].eq_ignore_ascii_case("bearer ") {
            token = auth[7..].trim().to_string();
        }
    }
    if token.is_empty() {
        if let Some(q_tok) = query.get("token") {
            token = q_tok.trim().to_string();
        }
    }
    if !state.runtime.validate_api_key(&token) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "detail": "Bearer token required for /mcp. Provide Authorization: Bearer <styles_...> token."
            })),
        )
            .into_response());
    }
    Ok(())
}

pub async fn mcp_get_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if let Err(resp) = check_mcp_auth(&state, &headers, &query) {
        return resp;
    }
    Json(json!({
        "protocol": "mcp-streamable-http",
        "service": state.settings.service_name,
        "version": state.settings.version,
        "tools": MCP_TOOL_NAMES,
    }))
    .into_response()
}

pub async fn mcp_post_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Response {
    if let Err(resp) = check_mcp_auth(&state, &headers, &query) {
        return resp;
    }

    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty() && s.len() <= 128)
        .unwrap_or("lnc-styles-session")
        .to_string();

    let parsed: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": "Parse error" }
                })),
            )
                .into_response();
        }
    };

    let mut resp = match parsed {
        Value::Array(items) => {
            if items.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32600, "message": "Invalid Request" }
                    })),
                )
                    .into_response();
            }
            let mut responses = Vec::new();
            for item in items {
                if let Some(r) = handle_jsonrpc_message(
                    &state.corpus,
                    &state.settings.service_name,
                    &state.settings.version,
                    &item,
                ) {
                    responses.push(r);
                }
            }
            if responses.is_empty() {
                StatusCode::ACCEPTED.into_response()
            } else {
                Json(Value::Array(responses)).into_response()
            }
        }
        Value::Object(_) => match handle_jsonrpc_message(
            &state.corpus,
            &state.settings.service_name,
            &state.settings.version,
            &parsed,
        ) {
            None => StatusCode::ACCEPTED.into_response(),
            Some(r) => Json(r).into_response(),
        },
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32600, "message": "Invalid Request" }
            })),
        )
            .into_response(),
    };

    if let Ok(hv) = HeaderValue::from_str(&session_id) {
        resp.headers_mut().insert("Mcp-Session-Id", hv);
    }
    resp
}
