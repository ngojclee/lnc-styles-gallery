//! HTTP server (`axum`): REST `/api/*` corpus routes, hardened `/api/keys*`
//! (Zero-F12-Leak + `410 Gone` on `/copy`), `/api/auth/*` + Keycloak OIDC SSO +
//! `lnc_styles_session` v2, `/mcp`, `/login.html`, static SPA serving, CSRF
//! Origin defense, and strict security headers & CSP.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{ConnectInfo, DefaultBodyLimit, Path as AxumPath, Query, State};
use axum::http::{header, Extensions, HeaderMap, HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use url::Url;

use crate::config::{
    now_secs, AppSettings, ConnectorMutationError, CopySecretOutcome, RuntimeStore,
};
use crate::mcp;
use crate::oidc::{self, LoginLimiter, OidcClient, SessionClaims};
use crate::store::{self, StyleLibraryStore};

pub struct AppState {
    pub settings: AppSettings,
    pub runtime: Arc<RuntimeStore>,
    pub corpus: Arc<StyleLibraryStore>,
    pub oidc: Arc<OidcClient>,
    pub limiter: Arc<LoginLimiter>,
}

const APP_CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self'; frame-src 'self'; frame-ancestors 'self'; base-uri 'self'; form-action 'self'; object-src 'none'";
const DEMO_CSP: &str = "default-src 'self' https: data: blob:; script-src 'self' 'unsafe-inline' https:; style-src 'self' 'unsafe-inline' https:; img-src 'self' data: blob: https:; font-src 'self' data: https:; frame-ancestors 'self'; base-uri 'self'; object-src 'none'";

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health
        .route("/health", get(api_health))
        .route("/api/health", get(api_health))
        // MCP Streamable HTTP
        .route("/mcp", get(mcp::mcp_get_handler).post(mcp::mcp_post_handler))
        .route("/api/mcp/info", get(api_mcp_info))
        // Corpus: Styles, Search, Collections
        .route("/api/styles", get(api_list_styles))
        .route("/api/styles/{style_id}", get(api_get_style))
        .route("/api/styles/{style_id}/preview", get(api_get_style_preview))
        .route("/api/styles/{style_id}/shots", get(api_get_style_shots))
        .route("/api/styles/{style_id}/shots/{n}", get(api_get_style_shot))
        .route("/api/search", get(api_search))
        .route("/api/collections", get(api_list_collections))
        .route("/api/collections/{collection_id}", get(api_get_collection))
        // Corpus: Templates
        .route("/api/templates", get(api_list_templates))
        .route("/api/templates/{template_id}", get(api_get_template))
        .route("/api/templates/{template_id}/{sub}", get(api_get_template_sub))
        // Corpus: Components
        .route("/api/components", get(api_list_components))
        .route("/api/components/{component_id}", get(api_get_component))
        .route(
            "/api/components/{component_id}/html",
            get(api_get_component_html),
        )
        .route(
            "/api/components/{component_id}/preview",
            get(api_get_component_preview),
        )
        .route(
            "/api/components/{component_id}/files/{*filename}",
            get(api_get_component_file),
        )
        // Licenses
        .route("/api/licenses", get(api_licenses))
        // API Keys (Zero-F12-Leak GET + HKDF AES-256-GCM Audited POST /copy)
        .route("/api/keys", get(api_list_keys).post(api_create_key))
        .route(
            "/api/keys/{key_id}",
            patch(api_update_key).delete(api_delete_key),
        )
        .route("/api/keys/{key_id}/revoke", post(api_revoke_key))
        .route("/api/keys/{key_id}/rotate", post(api_rotate_key))
        .route("/api/keys/{key_id}/copy", post(api_copy_key))
        .route("/api/admin/keys", get(api_list_keys).post(api_create_key))
        .route(
            "/api/admin/keys/{key_id}",
            patch(api_update_key).delete(api_delete_key),
        )
        .route("/api/admin/keys/{key_id}/revoke", post(api_revoke_key))
        .route("/api/admin/keys/{key_id}/rotate", post(api_rotate_key))
        .route("/api/admin/keys/{key_id}/copy", post(api_copy_key))
        .route("/api/admin/api-keys", get(api_list_keys).post(api_create_key))
        .route(
            "/api/admin/api-keys/{key_id}",
            patch(api_update_key).delete(api_delete_key),
        )
        .route("/api/admin/api-keys/{key_id}/revoke", post(api_revoke_key))
        .route("/api/admin/api-keys/{key_id}/rotate", post(api_rotate_key))
        .route("/api/admin/api-keys/{key_id}/copy", post(api_copy_key))
        // MCP Connectors (Inbound /mcp governance: Copy · Rotate · Revoke · Remove + Test)
        .route(
            "/api/admin/mcp-connectors",
            get(api_list_connectors).post(api_create_connector),
        )
        .route(
            "/api/admin/mcp-connectors/{connector_id}",
            patch(api_update_connector)
                .put(api_update_connector)
                .delete(api_delete_connector),
        )
        .route(
            "/api/admin/mcp-connectors/{connector_id}/copy",
            post(api_copy_connector),
        )
        .route(
            "/api/admin/mcp-connectors/{connector_id}/rotate",
            post(api_rotate_connector),
        )
        .route(
            "/api/admin/mcp-connectors/{connector_id}/revoke",
            post(api_revoke_connector),
        )
        .route(
            "/api/admin/mcp-connectors/{connector_id}/test",
            post(api_test_connector),
        )
        .route(
            "/api/mcp-connectors",
            get(api_list_connectors).post(api_create_connector),
        )
        .route(
            "/api/mcp-connectors/{connector_id}",
            patch(api_update_connector)
                .put(api_update_connector)
                .delete(api_delete_connector),
        )
        .route(
            "/api/mcp-connectors/{connector_id}/copy",
            post(api_copy_connector),
        )
        .route(
            "/api/mcp-connectors/{connector_id}/rotate",
            post(api_rotate_connector),
        )
        .route(
            "/api/mcp-connectors/{connector_id}/revoke",
            post(api_revoke_connector),
        )
        .route(
            "/api/mcp-connectors/{connector_id}/test",
            post(api_test_connector),
        )
        // Admin
        .route(
            "/api/admin/reindex",
            get(api_admin_reindex).post(api_admin_reindex),
        )
        .route("/api/admin/status", get(api_admin_status))
        .route(
            "/api/admin/settings",
            get(api_admin_settings)
                .post(api_update_admin_settings)
                .put(api_update_admin_settings),
        )
        // Auth & Keycloak OIDC SSO
        .route("/login.html", get(serve_login_html))
        .route("/login.js", get(serve_login_js))
        .route("/api/auth/status", get(api_auth_session))
        .route("/api/auth/session", get(api_auth_session))
        .route("/api/auth/login", post(api_auth_login))
        .route("/api/auth/logout", post(api_auth_logout))
        .route("/api/auth/revoke-all", post(api_auth_revoke_all))
        .route("/api/auth/oidc/login", get(api_oidc_login))
        .route("/api/auth/oidc/callback", get(api_oidc_callback))
        // SPA entry routes (Note: `/mcp` is reserved for Streamable HTTP MCP)
        .route("/", get(serve_public_spa))
        .route("/styles", get(serve_public_spa))
        .route("/collections", get(serve_public_spa))
        .route("/templates", get(serve_public_spa))
        .route("/components", get(serve_public_spa))
        .route("/platforms", get(serve_public_spa))
        .route("/licenses", get(serve_public_spa))
        .route("/keys", get(serve_protected_keys_spa))
        .route("/api-keys", get(serve_protected_api_keys_spa))
        .route("/mcp-connectors", get(serve_protected_mcp_connectors_spa))
        .route("/settings", get(serve_protected_settings_spa))
        .route("/admin", get(serve_protected_admin_spa))
        .route("/{*path}", get(serve_spa_fallback))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers_middleware,
        ))
        .with_state(state)
}

async fn security_headers_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let path_lower = path.to_ascii_lowercase();

    // Block raw or percent-encoded path traversal / null-byte / backslash probes globally
    if path.contains("..")
        || path.contains('\\')
        || path.contains('\0')
        || path_lower.contains("%2e%2e")
        || path_lower.contains("/%2e")
        || path_lower.contains("%2e/")
        || path_lower.contains("%5c")
        || path_lower.contains("%00")
    {
        let mut resp = (
            StatusCode::BAD_REQUEST,
            Json(json!({ "detail": "Invalid request path" })),
        )
            .into_response();
        apply_common_security_headers(&mut resp, &path);
        return resp;
    }

    // Block `?token=`, `?key=`, `?api_key=` secret-bearing query params globally (HTTP 400)
    if let Some(q) = req.uri().query() {
        let has_query_secret = url::form_urlencoded::parse(q.as_bytes()).any(|(k, _)| {
            matches!(
                k.to_ascii_lowercase().as_str(),
                "token" | "key" | "api_key"
            )
        });
        if has_query_secret {
            let mut resp = (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "query_secret_forbidden",
                    "detail": "Credentials must not be passed in URL query parameters (?token, ?key, ?api_key); use Authorization: Bearer or X-API-Key headers."
                })),
            )
                .into_response();
            apply_common_security_headers(&mut resp, &path);
            return resp;
        }
    }

    // Allowed-hosts guard when ALLOWED_HOSTS != "*"
    if state.settings.allowed_hosts != "*" && !state.settings.allowed_hosts.is_empty() {
        if let Some(host_hdr) = req
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
        {
            let req_host = extract_host_only(host_hdr);
            let allowed = state
                .settings
                .allowed_hosts
                .split(',')
                .map(|s| s.trim().to_ascii_lowercase())
                .any(|h| h == "*" || h == req_host);
            if !allowed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "detail": "Invalid Host header" })),
                )
                    .into_response();
            }
        }
    }

    if method == Method::OPTIONS {
        let mut resp = StatusCode::NO_CONTENT.into_response();
        apply_common_security_headers(&mut resp, &path);
        return resp;
    }

    let mut resp = next.run(req).await;
    apply_common_security_headers(&mut resp, &path);
    resp
}

const LOGIN_CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'; object-src 'none'";

fn is_no_store_path(path: &str) -> bool {
    path == "/login.html"
        || path == "/login.js"
        || path.starts_with("/api/auth/")
        || path == "/api/keys"
        || path.starts_with("/api/keys/")
        || path.starts_with("/api/admin/")
        || path == "/api/mcp-connectors"
        || path.starts_with("/api/mcp-connectors/")
}

fn apply_common_security_headers(resp: &mut Response, path: &str) {
    let h = resp.headers_mut();
    h.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        "X-Frame-Options",
        HeaderValue::from_static(if path == "/login.html" {
            "DENY"
        } else {
            "SAMEORIGIN"
        }),
    );
    h.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    h.insert(
        "Permissions-Policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    h.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    if is_no_store_path(path) {
        h.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, private, max-age=0"),
        );
        h.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    }
    if !h.contains_key(header::CONTENT_SECURITY_POLICY) {
        let is_demo_html = (path.starts_with("/api/templates/") && path.ends_with("/html"))
            || (path.starts_with("/api/components/") && path.ends_with("/html"));
        let csp = if path == "/login.html" {
            LOGIN_CSP
        } else if is_demo_html {
            DEMO_CSP
        } else {
            APP_CSP
        };
        if let Ok(v) = HeaderValue::from_str(csp) {
            h.insert(header::CONTENT_SECURITY_POLICY, v);
        }
    }
}

// ---------------------------------------------------------------------------
// Authentication, Session Renewal & CSRF Helpers
// ---------------------------------------------------------------------------

enum AuthPrincipal {
    Session(SessionClaims, Option<String>),
    AdminHeader,
    DevOpen,
}

fn auth_is_active(state: &AppState) -> bool {
    state.runtime.has_mgmt_key() || state.oidc.enabled()
}

fn extract_header_token(headers: &HeaderMap) -> Option<String> {
    if let Some(x) = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some(x.to_string());
    }
    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if auth.len() > 7 && auth[..7].eq_ignore_ascii_case("bearer ") {
            let tok = auth[7..].trim();
            if !tok.is_empty() {
                return Some(tok.to_string());
            }
        }
    }
    None
}

fn resolve_session(state: &AppState, headers: &HeaderMap) -> Option<(SessionClaims, Option<String>)> {
    let raw_cookie = oidc::cookie_value(headers, oidc::SESSION_COOKIE)?;
    let now = now_secs();
    let epoch = state.runtime.session_epoch();
    let claims = oidc::verify_session_with_ttl(
        state.runtime.master(),
        &raw_cookie,
        epoch,
        now,
        state.settings.session_ttl,
    )?;
    let renewed_set_cookie = if oidc::needs_renewal(&claims, now) {
        let updated = oidc::renew(claims.clone(), now);
        let signed = oidc::sign_session(state.runtime.master(), &updated);
        let secure = oidc::request_is_secure(headers, state.settings.cookie_secure);
        Some(oidc::session_cookie_with_ttl(
            &signed,
            secure,
            state.settings.session_ttl.max_ttl_secs,
        ))
    } else {
        None
    };
    Some((claims, renewed_set_cookie))
}

fn extract_principal(state: &AppState, headers: &HeaderMap) -> Option<AuthPrincipal> {
    if let Some(tok) = extract_header_token(headers) {
        if state.runtime.verify_mgmt_header(&tok) {
            return Some(AuthPrincipal::AdminHeader);
        }
    }
    if let Some((claims, renew_cookie)) = resolve_session(state, headers) {
        return Some(AuthPrincipal::Session(claims, renew_cookie));
    }
    None
}

fn extract_host_only(host_port: &str) -> String {
    let trimmed = host_port.trim();
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            return format!("[{}]", &rest[..end]).to_ascii_lowercase();
        }
    }
    trimmed
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

#[allow(clippy::result_large_err)]
fn check_csrf_origin(
    state: &AppState,
    headers: &HeaderMap,
    require_origin_or_same_site: bool,
) -> Result<(), Response> {
    let sfs_opt = headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_ascii_lowercase());
    if let Some(ref sfs) = sfs_opt {
        if sfs != "same-origin" && sfs != "same-site" && sfs != "none" {
            return Err(csrf_forbidden());
        }
    }

    let origin_or_ref = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get(header::REFERER).and_then(|v| v.to_str().ok()));

    let Some(raw_origin) = origin_or_ref.map(str::trim).filter(|s| !s.is_empty()) else {
        if require_origin_or_same_site && sfs_opt.as_deref() != Some("same-origin") {
            return Err(csrf_forbidden());
        }
        return Ok(());
    };
    if raw_origin.eq_ignore_ascii_case("null") {
        return Err(csrf_forbidden());
    }
    let Ok(parsed) = Url::parse(raw_origin) else {
        return Err(csrf_forbidden());
    };
    let Some(origin_host) = parsed.host_str().map(|h| h.to_ascii_lowercase()) else {
        return Err(csrf_forbidden());
    };

    // Ignore X-Forwarded-Host: only trust real Host header or configured effective_public_url.
    let mut allowed_hosts = Vec::new();
    if let Some(h) = headers.get(header::HOST).and_then(|v| v.to_str().ok()) {
        allowed_hosts.push(extract_host_only(h));
    }
    if let Ok(pub_u) = Url::parse(&state.runtime.effective_public_url(&state.settings.public_url)) {
        if let Some(ph) = pub_u.host_str() {
            allowed_hosts.push(ph.to_ascii_lowercase());
        }
    }

    if allowed_hosts.iter().any(|h| !h.is_empty() && *h == origin_host) {
        Ok(())
    } else {
        Err(csrf_forbidden())
    }
}

#[allow(clippy::result_large_err)]
fn verify_session_csrf_header(
    state: &AppState,
    claims: &SessionClaims,
    headers: &HeaderMap,
    required: bool,
) -> Result<(), Response> {
    let provided = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    match provided {
        Some(tok) => {
            if oidc::verify_session_csrf_token(state.runtime.master(), claims, tok) {
                Ok(())
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "ok": false,
                        "error": "csrf_token_invalid",
                        "detail": "Invalid X-CSRF-Token header"
                    })),
                )
                    .into_response())
            }
        }
        None if required => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "ok": false,
                "error": "csrf_token_missing",
                "detail": "Missing X-CSRF-Token header"
            })),
        )
            .into_response()),
        None => Ok(()),
    }
}

fn csrf_forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "ok": false,
            "error": "csrf_origin_mismatch",
            "detail": "Cross-origin state-changing request rejected by CSRF guard"
        })),
    )
        .into_response()
}

fn unauthorized_json(detail: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "ok": false,
            "error": "unauthorized",
            "detail": detail,
        })),
    )
        .into_response()
}

/// Require authenticated session or admin key for key/admin mutations.
#[allow(clippy::result_large_err)]
fn require_mutation_auth(
    state: &AppState,
    headers: &HeaderMap,
    is_state_changing: bool,
) -> Result<AuthPrincipal, Response> {
    match extract_principal(state, headers) {
        Some(AuthPrincipal::AdminHeader) => Ok(AuthPrincipal::AdminHeader),
        Some(AuthPrincipal::Session(c, renew_cookie)) => {
            if is_state_changing {
                check_csrf_origin(state, headers, false)?;
                verify_session_csrf_header(state, &c, headers, false)?;
            }
            Ok(AuthPrincipal::Session(c, renew_cookie))
        }
        Some(AuthPrincipal::DevOpen) => Ok(AuthPrincipal::DevOpen),
        None => {
            if !auth_is_active(state) && state.settings.dev_open_admin {
                if is_state_changing {
                    check_csrf_origin(state, headers, false)?;
                }
                Ok(AuthPrincipal::DevOpen)
            } else {
                Err(unauthorized_json(
                    "Authentication required (sign in via LNC SSO / management key or provide admin X-API-Key).",
                ))
            }
        }
    }
}

/// Require authenticated session or admin key on read admin/key endpoints whenever auth is configured.
#[allow(clippy::result_large_err)]
fn require_read_auth_when_active(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<String>, Response> {
    if !auth_is_active(state) {
        return Ok(None);
    }
    match extract_principal(state, headers) {
        Some(AuthPrincipal::Session(_, renew_cookie)) => Ok(renew_cookie),
        Some(AuthPrincipal::AdminHeader | AuthPrincipal::DevOpen) => Ok(None),
        None => Err(unauthorized_json(
            "Authentication required to view keys or admin diagnostics.",
        )),
    }
}

fn attach_optional_cookie(mut resp: Response, set_cookie: Option<String>) -> Response {
    if let Some(sc) = set_cookie {
        if let Ok(hv) = HeaderValue::from_str(&sc) {
            resp.headers_mut().append(header::SET_COOKIE, hv);
        }
    }
    resp
}

fn client_ip(headers: &HeaderMap, addr: Option<SocketAddr>) -> String {
    if let Some(xff) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return xff.to_string();
    }
    if let Some(xri) = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return xri.to_string();
    }
    addr.map(|a| a.ip().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

// ---------------------------------------------------------------------------
// Corpus & Health Handlers
// ---------------------------------------------------------------------------

async fn api_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.corpus.status())
}

#[derive(Debug, Deserialize, Default)]
struct ListStylesQuery {
    q: Option<String>,
    theme: Option<String>,
    category: Option<String>,
    collection: Option<String>,
    has_design: Option<bool>,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn api_list_styles(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListStylesQuery>,
) -> impl IntoResponse {
    let eff_cat = q.category.as_deref().or(q.collection.as_deref());
    Json(state.corpus.list_styles(
        q.q.as_deref(),
        q.theme.as_deref(),
        eff_cat,
        q.has_design,
        q.limit.unwrap_or(48),
        q.offset.unwrap_or(0),
    ))
}

async fn api_get_style(
    State(state): State<Arc<AppState>>,
    AxumPath(style_id): AxumPath<String>,
) -> Response {
    match state.corpus.get_style(&style_id, true) {
        Some(item) => Json(item).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Style not found: {style_id}") })),
        )
            .into_response(),
    }
}

async fn api_get_style_preview(
    State(state): State<Arc<AppState>>,
    AxumPath(style_id): AxumPath<String>,
) -> Response {
    if let Some(path) = state.corpus.get_style_preview_path(&style_id) {
        if let Some(resp) = serve_disk_file(&path).await {
            return resp;
        }
    }
    let Some(style) = state.corpus.get_style(&style_id, false) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Style not found: {style_id}") })),
        )
            .into_response();
    };
    let svg = store::render_style_fallback_svg(&style, &style_id);
    let mut resp = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        svg,
    )
        .into_response();
    resp.headers_mut()
        .insert("X-Preview-Fallback", HeaderValue::from_static("true"));
    resp
}

async fn api_get_style_shots(
    State(state): State<Arc<AppState>>,
    AxumPath(style_id): AxumPath<String>,
) -> Response {
    if state.corpus.get_style(&style_id, false).is_none()
        && !state.corpus.refero_style_dir_exists(&style_id)
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Style not found: {style_id}") })),
        )
            .into_response();
    }
    Json(Value::Array(state.corpus.list_style_shots(&style_id))).into_response()
}

async fn api_get_style_shot(
    State(state): State<Arc<AppState>>,
    AxumPath((style_id, n)): AxumPath<(String, String)>,
) -> Response {
    if let Some(p) = state.corpus.get_style_shot_path(&style_id, &n) {
        if let Some(resp) = serve_disk_file(&p).await {
            return resp;
        }
    }
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "detail": format!("Shot {n} not found for style: {style_id}") })),
    )
        .into_response()
}

#[derive(Debug, Deserialize, Default)]
struct SearchQuery {
    q: Option<String>,
    theme: Option<String>,
    category: Option<String>,
    include_content: Option<bool>,
    include_body: Option<bool>,
    limit: Option<usize>,
}

async fn api_search(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
) -> impl IntoResponse {
    let include = q.include_body.or(q.include_content).unwrap_or(true);
    Json(state.corpus.search(
        q.q.as_deref().unwrap_or(""),
        q.theme.as_deref(),
        q.category.as_deref(),
        include,
        q.limit.unwrap_or(30),
    ))
}

async fn api_list_collections(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cols = state.corpus.list_collections();
    Json(json!({
        "total": cols.len(),
        "items": cols,
    }))
}

#[derive(Debug, Deserialize, Default)]
struct PaginationQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn api_get_collection(
    State(state): State<Arc<AppState>>,
    AxumPath(collection_id): AxumPath<String>,
    Query(p): Query<PaginationQuery>,
) -> Response {
    match state
        .corpus
        .get_collection(&collection_id, p.limit.unwrap_or(100), p.offset.unwrap_or(0))
    {
        Some(col) => Json(col).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Collection not found: {collection_id}") })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize, Default)]
struct ListTemplatesQuery {
    lang: Option<String>,
    q: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn api_list_templates(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListTemplatesQuery>,
) -> impl IntoResponse {
    Json(state.corpus.list_templates(
        q.lang.as_deref(),
        q.q.as_deref(),
        q.limit.unwrap_or(100),
        q.offset.unwrap_or(0),
    ))
}

#[derive(Debug, Deserialize, Default)]
struct GetTemplateQuery {
    include_html: Option<bool>,
}

async fn api_get_template(
    State(state): State<Arc<AppState>>,
    AxumPath(template_id): AxumPath<String>,
    Query(q): Query<GetTemplateQuery>,
) -> Response {
    match state
        .corpus
        .get_template(&template_id, q.include_html.unwrap_or(false))
    {
        Some(tpl) => Json(tpl).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Template not found: {template_id}") })),
        )
            .into_response(),
    }
}

async fn api_get_template_sub(
    State(state): State<Arc<AppState>>,
    AxumPath((template_id, sub)): AxumPath<(String, String)>,
) -> Response {
    match sub.as_str() {
        "html" => match state.corpus.get_template_html_content(&template_id) {
            Some(html) => Html(html).into_response(),
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({ "detail": format!("HTML file missing for template: {template_id}") })),
            )
                .into_response(),
        },
        "prompt" => match state.corpus.get_template(&template_id, false) {
            Some(tpl) => {
                let prompt = tpl
                    .get("prompt")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if prompt.is_empty() {
                    (
                        StatusCode::NOT_FOUND,
                        Json(json!({ "detail": format!("Prompt missing for template: {template_id}") })),
                    )
                        .into_response()
                } else {
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
                        prompt,
                    )
                        .into_response()
                }
            }
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({ "detail": format!("Template not found: {template_id}") })),
            )
                .into_response(),
        },
        "thumb" => match state.corpus.get_template_thumb_or_meta(&template_id) {
            Some((Some(thumb_path), _)) => {
                if let Some(resp) = serve_disk_file(&thumb_path).await {
                    return resp;
                }
                StatusCode::NOT_FOUND.into_response()
            }
            Some((None, meta)) => {
                let svg = store::render_template_fallback_svg(&meta, &template_id);
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                    svg,
                )
                    .into_response()
            }
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({ "detail": format!("Template not found: {template_id}") })),
            )
                .into_response(),
        },
        filename => {
            if let Some(asset_path) = state.corpus.get_template_asset_path(&template_id, filename) {
                if let Some(resp) = serve_disk_file(&asset_path).await {
                    return resp;
                }
            }
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "detail": format!("Asset not found: {filename}") })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ListComponentsQuery {
    category: Option<String>,
    kind: Option<String>,
    q: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn api_list_components(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListComponentsQuery>,
) -> impl IntoResponse {
    Json(state.corpus.list_components(
        q.category.as_deref(),
        q.kind.as_deref(),
        q.q.as_deref(),
        q.limit.unwrap_or(200),
        q.offset.unwrap_or(0),
    ))
}

#[derive(Debug, Deserialize, Default)]
struct GetComponentQuery {
    include_source: Option<bool>,
}

async fn api_get_component(
    State(state): State<Arc<AppState>>,
    AxumPath(component_id): AxumPath<String>,
    Query(q): Query<GetComponentQuery>,
) -> Response {
    match state
        .corpus
        .get_component(&component_id, q.include_source.unwrap_or(false))
    {
        Some(c) => Json(c).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Component not found: {component_id}") })),
        )
            .into_response(),
    }
}

async fn api_get_component_html(
    State(state): State<Arc<AppState>>,
    AxumPath(component_id): AxumPath<String>,
) -> Response {
    let Some(path) = state.corpus.get_component_file_path(&component_id, "html") else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Component has no HTML demo: {component_id}") })),
        )
            .into_response();
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) => Html(String::from_utf8_lossy(&bytes).to_string()).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_get_component_preview(
    State(state): State<Arc<AppState>>,
    AxumPath(component_id): AxumPath<String>,
) -> Response {
    if let Some(p) = state.corpus.get_component_preview_path(&component_id) {
        if let Some(resp) = serve_disk_file(&p).await {
            return resp;
        }
    }
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "detail": format!("Component has no preview: {component_id}") })),
    )
        .into_response()
}

async fn api_get_component_file(
    State(state): State<Arc<AppState>>,
    AxumPath((component_id, filename)): AxumPath<(String, String)>,
) -> Response {
    if let Some(p) = state.corpus.get_component_file_path(&component_id, &filename) {
        if let Some(resp) = serve_disk_file(&p).await {
            return resp;
        }
    }
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "detail": format!("File not found: {filename}") })),
    )
        .into_response()
}

async fn api_licenses(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.corpus.get_licenses())
}

async fn api_mcp_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tools: Vec<Value> = mcp::mcp_tools_schema()
        .into_iter()
        .map(|t| {
            json!({
                "name": t["name"],
                "description": t["description"],
            })
        })
        .collect();
    let pub_url = state
        .runtime
        .effective_public_url(&state.settings.public_url);
    Json(json!({
        "endpoint": format!("{}/mcp", pub_url.trim_end_matches('/')),
        "namespace": "styles__*",
        "transport": "Streamable HTTP JSON-RPC 2.0 & SSE",
        "auth_required": state.settings.mcp_auth_required,
        "auth_mode": if state.settings.mcp_auth_required { "bearer" } else { "open" },
        "tools_count": tools.len(),
        "tools": tools,
        "keys_count": state.runtime.keys_count(),
        "connectors_count": state.runtime.connectors_count(),
    }))
}

// ---------------------------------------------------------------------------
// Hardened API Keys (`/api/keys*`, `/api/admin/api-keys*`, Audited Vault Copy)
// ---------------------------------------------------------------------------

fn principal_actor(principal: &AuthPrincipal) -> String {
    match principal {
        AuthPrincipal::Session(c, _) => format!("{}:{}", c.mode, c.sub),
        AuthPrincipal::AdminHeader => "admin_header".to_string(),
        AuthPrincipal::DevOpen => "dev_open".to_string(),
    }
}

fn rate_limited_response(retry_secs: u64, msg: &str) -> Response {
    let mut resp = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "ok": false,
            "error": "rate_limited",
            "detail": format!("{msg} Retry after {retry_secs}s."),
            "retry_after": retry_secs,
        })),
    )
        .into_response();
    if let Ok(hv) = HeaderValue::from_str(&retry_secs.to_string()) {
        resp.headers_mut().insert(header::RETRY_AFTER, hv);
    }
    resp
}

async fn api_list_keys(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let renew = match require_read_auth_when_active(&state, &headers) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let resp = Json(json!({
        "ok": true,
        "keys": state.runtime.list_keys(),
        "api_key_configured": state.runtime.has_mgmt_key(),
        "auth_required": auth_is_active(&state),
    }))
    .into_response();
    attach_optional_cookie(resp, renew)
}

#[derive(Debug, Deserialize)]
struct CreateKeyPayload {
    label: String,
    #[serde(default)]
    custom_secret: Option<String>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
}

async fn api_create_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateKeyPayload>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let label = payload.label.trim();
    if label.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "detail": "Key label cannot be empty" })),
        )
            .into_response();
    }
    match state.runtime.create_key_with_scopes(
        label,
        payload.custom_secret.as_deref(),
        payload.scopes,
    ) {
        Ok((raw_once, item)) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            let resp = Json(json!({
                "ok": true,
                "key": raw_once.as_str(),
                "item": item,
            }))
            .into_response();
            attach_optional_cookie(resp, renew)
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize, Default)]
struct UpdateKeyPayload {
    label: Option<String>,
    scopes: Option<Vec<String>>,
}

async fn api_update_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(key_id): AxumPath<String>,
    Json(payload): Json<UpdateKeyPayload>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    if let Some(ref lbl) = payload.label {
        if lbl.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "detail": "Key label cannot be empty" })),
            )
                .into_response();
        }
    }
    match state
        .runtime
        .update_key(&key_id, payload.label.as_deref(), payload.scopes)
    {
        Ok(Some(item)) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({ "ok": true, "item": item })).into_response(),
                renew,
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Key not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_copy_key(
    State(state): State<Arc<AppState>>,
    extensions: Extensions,
    headers: HeaderMap,
    AxumPath(key_id): AxumPath<String>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let remote_addr = extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(a)| *a);
    let ip = client_ip(&headers, remote_addr);
    if let Err(retry_secs) = state.runtime.check_copy_rate_limit(&ip) {
        return rate_limited_response(retry_secs, "Too many secret copy requests.");
    }
    match state.runtime.copy_key(&key_id) {
        Ok(CopySecretOutcome::Secret(raw_secret)) => {
            let actor = principal_actor(&principal);
            state
                .runtime
                .record_audit_event(&actor, "secret.copy", "api_key", &key_id, &ip);
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            let resp = Json(json!({
                "ok": true,
                "id": key_id,
                "secret": raw_secret.as_str(),
                "copied_field": "token",
            }))
            .into_response();
            attach_optional_cookie(resp, renew)
        }
        Ok(CopySecretOutcome::LegacyHashOnly) => (
            StatusCode::GONE,
            Json(json!({
                "ok": false,
                "error": "legacy_hash_only",
                "detail": "Legacy API key has no encrypted vault envelope. Rotate the key to enable Copy."
            })),
        )
            .into_response(),
        Ok(CopySecretOutcome::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Key not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize, Default)]
struct RevokeKeyPayload {
    revoke: Option<bool>,
}

async fn api_revoke_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(key_id): AxumPath<String>,
    payload: Option<Json<RevokeKeyPayload>>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let revoke_val = payload.and_then(|Json(p)| p.revoke).unwrap_or(true);
    match state.runtime.revoke_key(&key_id, revoke_val) {
        Ok(Some(item)) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            let resp = Json(json!({
                "ok": true,
                "message": if revoke_val { "Key revoked" } else { "Key reactivated" },
                "item": item,
            }))
            .into_response();
            attach_optional_cookie(resp, renew)
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Key not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize, Default)]
struct RotateKeyPayload {
    custom_secret: Option<String>,
}

async fn api_rotate_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(key_id): AxumPath<String>,
    payload: Option<Json<RotateKeyPayload>>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let custom = payload.and_then(|Json(p)| p.custom_secret);
    match state.runtime.rotate_key(&key_id, custom.as_deref()) {
        Ok(Some((raw_once, item))) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            let resp = Json(json!({
                "ok": true,
                "key": raw_once.as_str(),
                "item": item,
                "message": "Key rotated successfully",
            }))
            .into_response();
            attach_optional_cookie(resp, renew)
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Key not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_delete_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(key_id): AxumPath<String>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    match state.runtime.delete_key(&key_id) {
        Ok(true) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            let resp = Json(json!({
                "ok": true,
                "message": "Key permanently deleted",
            }))
            .into_response();
            attach_optional_cookie(resp, renew)
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Key not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// MCP Connectors Endpoints (`/api/admin/mcp-connectors*`, `/api/mcp-connectors*`)
// ---------------------------------------------------------------------------

fn mcp_endpoint_url(state: &AppState) -> String {
    let pub_url = state
        .runtime
        .effective_public_url(&state.settings.public_url);
    format!("{}/mcp", pub_url.trim_end_matches('/'))
}

fn env_managed_conflict() -> Response {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "ok": false,
            "error": "env_managed_connector",
            "detail": "Managed by container environment variable — clear CONNECTOR_BEARER to mutate in UI."
        })),
    )
        .into_response()
}

async fn api_list_connectors(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let renew = match require_read_auth_when_active(&state, &headers) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let endpoint = mcp_endpoint_url(&state);
    let tools: Vec<Value> = mcp::mcp_tools_schema()
        .into_iter()
        .map(|t| {
            json!({
                "name": t["name"],
                "description": t["description"],
            })
        })
        .collect();
    let resp = Json(json!({
        "ok": true,
        "endpoint": endpoint,
        "namespace": "styles__*",
        "transport": "Streamable HTTP JSON-RPC 2.0",
        "auth_required": state.settings.mcp_auth_required,
        "audience": state.settings.connector_audience,
        "max_scopes": state.settings.connector_max_scopes,
        "delegation_configured": state.settings.connector_delegation_configured,
        "tools_count": tools.len(),
        "tools": tools,
        "connectors": state.runtime.list_connectors(&endpoint),
    }))
    .into_response();
    attach_optional_cookie(resp, renew)
}

#[derive(Debug, Deserialize)]
struct CreateConnectorPayload {
    name: String,
    #[serde(default)]
    audience: Option<String>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
    #[serde(default)]
    custom_secret: Option<String>,
}

async fn api_create_connector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateConnectorPayload>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let name = payload.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "detail": "Connector name cannot be empty" })),
        )
            .into_response();
    }
    let endpoint = mcp_endpoint_url(&state);
    let audience = payload
        .audience
        .as_deref()
        .unwrap_or(&state.settings.connector_audience);
    let scopes = payload
        .scopes
        .unwrap_or_else(|| state.settings.connector_max_scopes.clone());
    match state.runtime.create_connector(
        name,
        audience,
        scopes,
        payload.custom_secret.as_deref(),
        &endpoint,
    ) {
        Ok((raw_once, item)) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({
                    "ok": true,
                    "secret": raw_once.as_str(),
                    "item": item,
                }))
                .into_response(),
                renew,
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize, Default)]
struct UpdateConnectorPayload {
    name: Option<String>,
    audience: Option<String>,
    scopes: Option<Vec<String>>,
    custom_secret: Option<String>,
    secret: Option<String>,
}

async fn api_update_connector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(connector_id): AxumPath<String>,
    Json(payload): Json<UpdateConnectorPayload>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    if let Some(ref n) = payload.name {
        if n.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "detail": "Connector name cannot be empty" })),
            )
                .into_response();
        }
    }
    let endpoint = mcp_endpoint_url(&state);
    let sec = payload.custom_secret.or(payload.secret);
    match state.runtime.update_connector(
        &connector_id,
        payload.name.as_deref(),
        payload.audience.as_deref(),
        payload.scopes,
        sec.as_deref(),
        &endpoint,
    ) {
        Ok(Ok(item)) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({ "ok": true, "item": item })).into_response(),
                renew,
            )
        }
        Ok(Err(ConnectorMutationError::NotFound)) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Connector not found" })),
        )
            .into_response(),
        Ok(Err(ConnectorMutationError::EnvManaged)) => env_managed_conflict(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_copy_connector(
    State(state): State<Arc<AppState>>,
    extensions: Extensions,
    headers: HeaderMap,
    AxumPath(connector_id): AxumPath<String>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let remote_addr = extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(a)| *a);
    let ip = client_ip(&headers, remote_addr);
    if let Err(retry_secs) = state.runtime.check_copy_rate_limit(&ip) {
        return rate_limited_response(retry_secs, "Too many secret copy requests.");
    }
    let endpoint = mcp_endpoint_url(&state);
    match state.runtime.copy_connector(&connector_id) {
        Ok(CopySecretOutcome::Secret(raw_secret)) => {
            let actor = principal_actor(&principal);
            state.runtime.record_audit_event(
                &actor,
                "secret.copy",
                "mcp_connector",
                &connector_id,
                &ip,
            );
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({
                    "ok": true,
                    "id": connector_id,
                    "secret": raw_secret.as_str(),
                    "copied_field": "token",
                    "endpoint_url": endpoint,
                }))
                .into_response(),
                renew,
            )
        }
        Ok(CopySecretOutcome::LegacyHashOnly) => (
            StatusCode::GONE,
            Json(json!({
                "ok": false,
                "error": "legacy_hash_only",
                "detail": "Connector has no encrypted vault envelope. Rotate the connector to enable Copy."
            })),
        )
            .into_response(),
        Ok(CopySecretOutcome::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Connector not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_rotate_connector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(connector_id): AxumPath<String>,
    payload: Option<Json<RotateKeyPayload>>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let custom = payload.and_then(|Json(p)| p.custom_secret);
    let endpoint = mcp_endpoint_url(&state);
    match state
        .runtime
        .rotate_connector(&connector_id, custom.as_deref(), &endpoint)
    {
        Ok(Ok((raw_once, item))) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({
                    "ok": true,
                    "secret": raw_once.as_str(),
                    "item": item,
                    "message": "Connector token rotated successfully",
                }))
                .into_response(),
                renew,
            )
        }
        Ok(Err(ConnectorMutationError::NotFound)) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Connector not found" })),
        )
            .into_response(),
        Ok(Err(ConnectorMutationError::EnvManaged)) => env_managed_conflict(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_revoke_connector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(connector_id): AxumPath<String>,
    payload: Option<Json<RevokeKeyPayload>>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let revoke_val = payload.and_then(|Json(p)| p.revoke).unwrap_or(true);
    let endpoint = mcp_endpoint_url(&state);
    match state
        .runtime
        .revoke_connector(&connector_id, revoke_val, &endpoint)
    {
        Ok(Ok(item)) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({
                    "ok": true,
                    "message": if revoke_val { "Connector revoked" } else { "Connector reactivated" },
                    "item": item,
                }))
                .into_response(),
                renew,
            )
        }
        Ok(Err(ConnectorMutationError::NotFound)) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Connector not found" })),
        )
            .into_response(),
        Ok(Err(ConnectorMutationError::EnvManaged)) => env_managed_conflict(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_delete_connector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(connector_id): AxumPath<String>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    match state.runtime.delete_connector(&connector_id) {
        Ok(Ok(())) => {
            let renew = match principal {
                AuthPrincipal::Session(_, r) => r,
                _ => None,
            };
            attach_optional_cookie(
                Json(json!({ "ok": true, "message": "Connector removed" })).into_response(),
                renew,
            )
        }
        Ok(Err(ConnectorMutationError::NotFound)) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Connector not found" })),
        )
            .into_response(),
        Ok(Err(ConnectorMutationError::EnvManaged)) => env_managed_conflict(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_test_connector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(connector_id): AxumPath<String>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let t0 = std::time::Instant::now();
    let Some((enabled, scopes, audience)) = state.runtime.connector_status(&connector_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "detail": "Connector not found" })),
        )
            .into_response();
    };
    let endpoint = mcp_endpoint_url(&state);
    let tools_count = mcp::mcp_tools_schema().len();
    let latency_ms = t0.elapsed().as_millis() as u64;
    let renew = match principal {
        AuthPrincipal::Session(_, r) => r,
        _ => None,
    };
    attach_optional_cookie(
        Json(json!({
            "ok": enabled,
            "id": connector_id,
            "status": if enabled { "healthy" } else { "revoked" },
            "endpoint": endpoint,
            "tools_count": tools_count,
            "latency_ms": latency_ms,
            "audience": audience,
            "scopes": scopes,
            "detail": if enabled {
                format!("Streamable HTTP /mcp healthy ({tools_count} tools ready)")
            } else {
                "Connector is revoked; bearer requests are rejected with 401".to_string()
            },
        }))
        .into_response(),
        renew,
    )
}

// ---------------------------------------------------------------------------
// Admin Endpoints (`/api/admin/*`)
// ---------------------------------------------------------------------------

async fn api_admin_reindex(
    State(state): State<Arc<AppState>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, method != Method::GET) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let st = state.corpus.reindex();
    let mut out = st.as_object().cloned().unwrap_or_default();
    out.insert("status".into(), Value::String("reindexed".into()));
    out.insert("summary".into(), st);
    let renew = match principal {
        AuthPrincipal::Session(_, r) => r,
        _ => None,
    };
    attach_optional_cookie(Json(Value::Object(out)).into_response(), renew)
}

async fn api_admin_status(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let renew = match require_read_auth_when_active(&state, &headers) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    attach_optional_cookie(Json(state.corpus.status()).into_response(), renew)
}

async fn api_admin_settings(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let renew = match require_read_auth_when_active(&state, &headers) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let resp = Json(build_admin_settings_payload(&state)).into_response();
    attach_optional_cookie(resp, renew)
}

#[derive(Debug, Deserialize, Default)]
struct UpdateAdminSettingsPayload {
    public_url: Option<String>,
    public_base_url: Option<String>,
}

async fn api_update_admin_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<UpdateAdminSettingsPayload>,
) -> Response {
    let principal = match require_mutation_auth(&state, &headers, true) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    if let Some(new_url) = payload.public_url.or(payload.public_base_url) {
        if let Err(ssrf_err) = state
            .runtime
            .update_public_url(&new_url, state.settings.ssrf_block_private)
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "invalid_url",
                    "detail": ssrf_err.to_string(),
                })),
            )
                .into_response();
        }
    }
    let renew = match principal {
        AuthPrincipal::Session(_, r) => r,
        _ => None,
    };
    attach_optional_cookie(Json(build_admin_settings_payload(&state)).into_response(), renew)
}

fn build_admin_settings_payload(state: &AppState) -> Value {
    let masked = state
        .runtime
        .masked_settings_dict(&state.settings, state.oidc.enabled());
    let st = state.corpus.status();
    let oidc_cfg = state.oidc.settings();
    json!({
        "settings": masked,
        "corpus": st,
        "service_name": masked["service_name"],
        "service_version": masked["version"],
        "host": state.settings.host,
        "port": masked["port"],
        "public_base_url": masked["public_url"],
        "data_root": masked["data_dir"],
        "config_dir": masked["config_dir"],
        "refero_dir": masked["refero_styles_dir"],
        "refero_dir_exists": st["refero_styles_exists"],
        "prompt_ui_dir": masked["prompt_ui_dir"],
        "prompt_ui_dir_exists": st["prompt_ui_exists"],
        "components_dir": masked["components_dir"],
        "components_dir_exists": st["components_dir_exists"],
        "api_key_configured": masked["api_key_configured"],
        "api_key_masked": masked["api_key_masked"],
        "oidc_enabled": state.oidc.enabled(),
        "oidc_issuer": oidc_cfg.issuer,
        "oidc_client_id": oidc_cfg.client_id,
        "oidc_redirect_uri": oidc_cfg.redirect_uri,
        "oidc_scopes": oidc_cfg.scopes,
        "oidc_allowed_users": oidc_cfg.allowed_users,
        "oidc_allowed_groups": oidc_cfg.allowed_groups,
        "oidc_client_secret_configured": !oidc_cfg.client_secret.is_empty(),
        "mcp_auth_required": state.settings.mcp_auth_required,
        "session_epoch": state.runtime.session_epoch(),
        "session_idle_ttl_seconds": state.settings.session_ttl.idle_ttl_secs,
        "session_max_ttl_seconds": state.settings.session_ttl.max_ttl_secs,
        "cookie_secure": state.settings.cookie_secure,
        "master_key_configured": state.settings.master_key_configured,
        "master_key_old_configured": state.settings.master_key_old_configured,
        "rewrapped_from_old_key": state.runtime.rewrapped_from_old_key(),
        "connector_audience": state.settings.connector_audience,
        "connector_max_scopes": state.settings.connector_max_scopes,
        "connector_delegation_configured": state.settings.connector_delegation_configured,
        "recent_audit_events": state.runtime.recent_audit_events(6),
        "cors_origins": state.settings.allowed_hosts,
    })
}

// ---------------------------------------------------------------------------
// Auth & OIDC SSO Endpoints (`/api/auth/*`, `/login.html`, `/login.js`)
// ---------------------------------------------------------------------------

async fn api_auth_session(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let active = auth_is_active(&state);
    let oidc_on = state.oidc.enabled();
    let mgmt_on = state.runtime.has_mgmt_key();
    let epoch = state.runtime.session_epoch();

    if let Some((claims, renew_cookie)) = resolve_session(&state, &headers) {
        let (exp_ms, idle_ms) = oidc::expires_info_with_ttl(&claims, state.settings.session_ttl);
        let csrf_token = oidc::derive_session_csrf_token(state.runtime.master(), &claims);
        let resp = Json(json!({
            "ok": true,
            "authenticated": true,
            "auth_required": active,
            "mode": claims.mode,
            "sub": claims.sub,
            "user": claims.user,
            "oidc_enabled": oidc_on,
            "mgmt_key_configured": mgmt_on,
            "session_epoch": epoch,
            "expires_at": exp_ms,
            "idle_expires_at": idle_ms,
            "csrf_token": csrf_token,
        }))
        .into_response();
        return attach_optional_cookie(resp, renew_cookie);
    }

    if let Some(tok) = extract_header_token(&headers) {
        if state.runtime.verify_mgmt_header(&tok) {
            return Json(json!({
                "ok": true,
                "authenticated": true,
                "auth_required": active,
                "mode": "api_key_header",
                "sub": "mgmt",
                "user": Value::Null,
                "oidc_enabled": oidc_on,
                "mgmt_key_configured": mgmt_on,
                "session_epoch": epoch,
                "expires_at": Value::Null,
                "idle_expires_at": Value::Null,
            }))
            .into_response();
        }
    }

    Json(json!({
        "ok": true,
        "authenticated": false,
        "auth_required": active,
        "mode": Value::Null,
        "sub": Value::Null,
        "user": Value::Null,
        "oidc_enabled": oidc_on,
        "mgmt_key_configured": mgmt_on,
        "session_epoch": epoch,
        "expires_at": Value::Null,
        "idle_expires_at": Value::Null,
    }))
    .into_response()
}

#[derive(Debug, Deserialize, Default)]
struct LoginRequest {
    key: Option<String>,
    password: Option<String>,
    token: Option<String>,
}

async fn api_auth_login(
    State(state): State<Arc<AppState>>,
    extensions: Extensions,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Response {
    if let Err(resp) = check_csrf_origin(&state, &headers, false) {
        return resp;
    }

    let remote_addr = extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(a)| *a);
    let ip = client_ip(&headers, remote_addr);
    if let Err(retry_secs) = state.limiter.check(&ip) {
        let mut resp = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "ok": false,
                "error": "rate_limited",
                "detail": format!("Too many failed login attempts. Retry after {retry_secs}s."),
                "retry_after": retry_secs,
            })),
        )
            .into_response();
        if let Ok(hv) = HeaderValue::from_str(&retry_secs.to_string()) {
            resp.headers_mut().insert(header::RETRY_AFTER, hv);
        }
        return resp;
    }

    let candidate = payload
        .key
        .or(payload.password)
        .or(payload.token)
        .unwrap_or_default();

    let runtime = state.runtime.clone();
    let valid = tokio::task::spawn_blocking(move || runtime.verify_mgmt_login(&candidate))
        .await
        .unwrap_or(false);

    if !valid {
        state.limiter.fail(&ip);
        return unauthorized_json("Invalid management key");
    }

    state.limiter.ok(&ip);
    let now = now_secs();
    let epoch = state.runtime.session_epoch();
    let claims = oidc::new_session(
        "mgmt_key",
        "mgmt",
        Some(json!({ "username": "mgmt", "name": "Management Key" })),
        epoch,
        now,
    );
    let signed = oidc::sign_session(state.runtime.master(), &claims);
    let secure = oidc::request_is_secure(&headers, state.settings.cookie_secure);
    let set_cookie = oidc::session_cookie_with_ttl(
        &signed,
        secure,
        state.settings.session_ttl.max_ttl_secs,
    );
    let csrf_token = oidc::derive_session_csrf_token(state.runtime.master(), &claims);

    attach_optional_cookie(
        Json(json!({
            "ok": true,
            "authenticated": true,
            "mode": "mgmt_key",
            "session_epoch": epoch,
            "csrf_token": csrf_token,
        }))
        .into_response(),
        Some(set_cookie),
    )
}

async fn api_auth_logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    // Strict local-only logout: require same-origin / Origin / Sec-Fetch-Site (ignoring X-Forwarded-Host)
    if let Err(resp) = check_csrf_origin(&state, &headers, true) {
        return resp;
    }
    // Enforce X-CSRF-Token strictly whenever a session cookie is active (or header present)
    if let Some((claims, _)) = resolve_session(&state, &headers) {
        if let Err(resp) = verify_session_csrf_header(&state, &claims, &headers, true) {
            return resp;
        }
    }
    let secure = oidc::request_is_secure(&headers, state.settings.cookie_secure);
    let clear = oidc::clear_cookie(oidc::SESSION_COOKIE, secure);
    attach_optional_cookie(
        Json(json!({ "ok": true, "authenticated": false })).into_response(),
        Some(clear),
    )
}

async fn api_auth_revoke_all(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_mutation_auth(&state, &headers, true) {
        return resp;
    }
    match state.runtime.bump_session_epoch() {
        Ok(new_epoch) => {
            let secure = oidc::request_is_secure(&headers, state.settings.cookie_secure);
            let clear = oidc::clear_cookie(oidc::SESSION_COOKIE, secure);
            attach_optional_cookie(
                Json(json!({
                    "ok": true,
                    "revoked": true,
                    "session_epoch": new_epoch,
                }))
                .into_response(),
                Some(clear),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "detail": e.to_string() })),
        )
            .into_response(),
    }
}

fn redirect_302(uri: &str) -> Response {
    (StatusCode::FOUND, [(header::LOCATION, uri.to_string())]).into_response()
}

#[derive(Debug, Deserialize, Default)]
struct OidcLoginQuery {
    next: Option<String>,
}

async fn api_oidc_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<OidcLoginQuery>,
) -> Response {
    match state
        .oidc
        .begin_login(state.runtime.master(), q.next.as_deref())
        .await
    {
        Ok((auth_url, tx_cookie_val)) => {
            let secure = oidc::request_is_secure(&headers, state.settings.cookie_secure);
            let sc = oidc::oidc_tx_cookie(&tx_cookie_val, secure);
            attach_optional_cookie(redirect_302(&auth_url), Some(sc))
        }
        Err(err) => redirect_302(&format!(
            "/login.html?oidc_error={}&error={}",
            err.code(),
            err.code()
        )),
    }
}

#[derive(Debug, Deserialize, Default)]
struct OidcCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn api_oidc_callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<OidcCallbackQuery>,
) -> Response {
    let secure = oidc::request_is_secure(&headers, state.settings.cookie_secure);
    let clear_tx = oidc::clear_cookie(oidc::OIDC_TX_COOKIE, secure);

    if let Some(idp_err) = q.error.as_deref().filter(|s| !s.is_empty()) {
        let code = if idp_err
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            idp_err
        } else {
            "idp_error"
        };
        return attach_optional_cookie(
            redirect_302(&format!("/login.html?oidc_error={code}&error={code}")),
            Some(clear_tx),
        );
    }

    let tx_cookie = oidc::cookie_value(&headers, oidc::OIDC_TX_COOKIE).unwrap_or_default();
    let code = q.code.as_deref().unwrap_or("");
    let st_param = q.state.as_deref().unwrap_or("");

    match state
        .oidc
        .complete(state.runtime.master(), &tx_cookie, code, st_param)
        .await
    {
        Ok((user, next_path)) => {
            let now = now_secs();
            let epoch = state.runtime.session_epoch();
            let claims = oidc::new_session("oidc", &user.sub, Some(user.to_json()), epoch, now);
            let signed = oidc::sign_session(state.runtime.master(), &claims);
            let sess_sc = oidc::session_cookie_with_ttl(
                &signed,
                secure,
                state.settings.session_ttl.max_ttl_secs,
            );

            let mut resp = redirect_302(&next_path);
            if let Ok(hv1) = HeaderValue::from_str(&clear_tx) {
                resp.headers_mut().append(header::SET_COOKIE, hv1);
            }
            if let Ok(hv2) = HeaderValue::from_str(&sess_sc) {
                resp.headers_mut().append(header::SET_COOKIE, hv2);
            }
            resp
        }
        Err(err) => attach_optional_cookie(
            redirect_302(&format!(
                "/login.html?oidc_error={}&error={}",
                err.code(),
                err.code()
            )),
            Some(clear_tx),
        ),
    }
}

async fn serve_login_html(State(state): State<Arc<AppState>>) -> Response {
    let sso_enabled = state.oidc.enabled();
    let mgmt_enabled = state.runtime.has_mgmt_key();
    let sso_attr = if sso_enabled { "1" } else { "0" };
    let mgmt_attr = if mgmt_enabled { "1" } else { "0" };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Sign In · LNC Style Gallery</title>
  <link rel="icon" type="image/svg+xml" href="/favicon.svg">
  <style>
    :root {{ color-scheme: dark; }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0; min-height: 100vh; display: flex; align-items: center; justify-content: center;
      padding: 24px; background: radial-gradient(circle at 50% 0%, #1e1b4b 0%, #09090b 62%);
      color: #f4f4f5; font-family: Inter, system-ui, -apple-system, sans-serif;
    }}
    .card {{
      width: 100%; max-width: 420px; background: rgba(17, 17, 22, 0.92);
      border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 16px; padding: 28px;
      box-shadow: 0 24px 48px rgba(0, 0, 0, 0.45);
    }}
    .brand {{ display: flex; align-items: center; gap: 12px; margin-bottom: 18px; }}
    .brand img {{ width: 36px; height: 36px; border-radius: 10px; }}
    h1 {{ margin: 0; font-size: 18px; font-weight: 700; letter-spacing: -0.02em; }}
    .sub {{ margin: 2px 0 0; font-size: 12px; color: #a1a1aa; }}
    .desc {{ margin: 0 0 20px; font-size: 13px; line-height: 1.5; color: #a1a1aa; }}
    .alert {{
      display: none; margin-bottom: 16px; padding: 10px 12px; border-radius: 10px;
      background: rgba(239, 68, 68, 0.14); border: 1px solid rgba(239, 68, 68, 0.35);
      color: #fca5a5; font-size: 12px;
    }}
    .btn-sso {{
      display: flex; align-items: center; justify-content: center; gap: 8px; width: 100%;
      height: 42px; border-radius: 10px; background: #6366f1; color: #fff; font-weight: 600;
      font-size: 14px; text-decoration: none; border: none; cursor: pointer; transition: background .15s;
    }}
    .btn-sso:hover {{ background: #4f46e5; }}
    .btn-sso[aria-disabled="true"] {{ opacity: 0.5; pointer-events: none; }}
    details {{
      margin-top: 18px; border-top: 1px solid rgba(255, 255, 255, 0.08); padding-top: 14px;
    }}
    summary {{
      cursor: pointer; font-size: 12px; font-weight: 600; color: #a1a1aa; user-select: none;
    }}
    summary:hover {{ color: #e4e4e7; }}
    form {{ margin-top: 12px; display: flex; flex-direction: column; gap: 10px; }}
    label {{ font-size: 12px; font-weight: 500; color: #d4d4d8; }}
    input {{
      width: 100%; height: 38px; border-radius: 10px; border: 1px solid rgba(255,255,255,0.14);
      background: rgba(9, 9, 11, 0.8); color: #fafafa; padding: 0 12px; font-size: 13px;
    }}
    input:focus {{ outline: 2px solid #6366f1; border-color: transparent; }}
    .btn-key {{
      height: 38px; border-radius: 10px; border: 1px solid rgba(255,255,255,0.16);
      background: rgba(255,255,255,0.06); color: #f4f4f5; font-weight: 600; font-size: 13px; cursor: pointer;
    }}
    .btn-key:hover {{ background: rgba(255,255,255,0.12); }}
    .back {{ display: inline-block; margin-top: 16px; font-size: 12px; color: #818cf8; text-decoration: none; }}
  </style>
</head>
<body data-sso="{sso_attr}" data-mgmt="{mgmt_attr}">
  <main class="card">
    <div class="brand">
      <img src="/favicon.svg" alt="">
      <div>
        <h1>LNC STYLE GALLERY</h1>
        <p class="sub">styles.lengoc.me · Governance &amp; MCP Access</p>
      </div>
    </div>
    <p class="desc">Sign in with your LNC Keycloak SSO account to manage MCP Bearer keys and runtime administration.</p>
    <div id="login-alert" class="alert" role="alert"></div>
    <a id="sso-link" class="btn-sso" href="/api/auth/oidc/login" aria-disabled="{sso_disabled}">Sign in with LNC SSO</a>
    <details id="break-glass">
      <summary>Break-glass Management Key</summary>
      <form id="mgmt-form">
        <label for="mgmt-key">Management key</label>
        <input id="mgmt-key" name="key" type="password" autocomplete="current-password" placeholder="Enter management key..." required>
        <button id="mgmt-submit" class="btn-key" type="submit">Authenticate with Key</button>
      </form>
    </details>
    <a class="back" href="/styles">← Return to public Style Catalog</a>
  </main>
  <script src="/login.js" defer></script>
</body>
</html>"#,
        sso_disabled = if sso_enabled { "false" } else { "true" }
    );

    Html(html).into_response()
}

async fn serve_login_js() -> Response {
    let js = r#"(() => {
  const params = new URLSearchParams(window.location.search);
  const rawNext = params.get("next") || "/styles";
  const next = rawNext.startsWith("/") && !rawNext.startsWith("//") && !rawNext.startsWith("/\\") ? rawNext : "/styles";
  const err = params.get("oidc_error") || params.get("error");
  const alertEl = document.getElementById("login-alert");
  const ssoLink = document.getElementById("sso-link");
  const detailsEl = document.getElementById("break-glass");
  const ssoOn = document.body.dataset.sso === "1";

  if (ssoLink && ssoOn) {
    ssoLink.setAttribute("href", "/api/auth/oidc/login?next=" + encodeURIComponent(next));
  } else if (detailsEl) {
    detailsEl.open = true;
  }

  const showError = (msg) => {
    if (!alertEl) return;
    alertEl.textContent = msg;
    alertEl.style.display = "block";
  };

  if (err) {
    const map = {
      oidc_disabled: "LNC SSO is not configured on this instance. Use Break-glass Management Key below.",
      not_allowed: "Your SSO account is not in OIDC_ALLOWED_USERS / OIDC_ALLOWED_GROUPS.",
      state_mismatch: "SSO state check failed. Please start sign-in again.",
      tx_expired: "SSO login window expired (5m). Please try again.",
      tx_missing: "SSO login session cookie was missing or expired. Please try again.",
      tx_invalid: "SSO transaction token could not be verified. Please try again.",
      missing_code: "SSO provider returned without an authorization code.",
      nonce_mismatch: "SSO ID token nonce verification failed.",
      invalid_token: "SSO ID token validation failed.",
      token_exchange: "SSO token exchange failed with the identity provider.",
      oidc_discovery: "Could not reach OIDC issuer metadata endpoint.",
      idp_error: "Identity provider returned an authentication error.",
    };
    showError(map[err] || ("SSO sign-in failed (" + err + ")."));
    params.delete("oidc_error");
    params.delete("error");
    const qs = params.toString();
    window.history.replaceState({}, "", window.location.pathname + (qs ? "?" + qs : ""));
  }

  const form = document.getElementById("mgmt-form");
  const input = document.getElementById("mgmt-key");
  const btn = document.getElementById("mgmt-submit");
  if (form && input && btn) {
    form.addEventListener("submit", async (e) => {
      e.preventDefault();
      btn.disabled = true;
      try {
        const res = await fetch("/api/auth/login", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          credentials: "same-origin",
          body: JSON.stringify({ key: input.value }),
        });
        const data = await res.json().catch(() => ({}));
        if (res.ok && data.ok) {
          window.location.assign(next);
          return;
        }
        showError(data.detail || ("Authentication failed (HTTP " + res.status + ")"));
      } catch (_) {
        showError("Network error while signing in.");
      } finally {
        btn.disabled = false;
      }
    });
  }
})();"#;
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        js,
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// SPA & Static Asset Serving
// ---------------------------------------------------------------------------

fn resolve_spa_index(settings: &AppSettings) -> Option<PathBuf> {
    let cands = [
        settings.ui_dist_dir.join("index.html"),
        settings.ui_dir.join("index.html"),
        settings.ui_backup_dir.join("index.html"),
    ];
    cands.into_iter().find(|p| p.is_file())
}

async fn serve_spa_index_response(state: &AppState) -> Response {
    if let Some(idx) = resolve_spa_index(&state.settings) {
        if let Some(resp) = serve_disk_file(&idx).await {
            return resp;
        }
    }
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>LNC Style Gallery</title></head><body><div id=\"root\">LNC Style Gallery API Running</div></body></html>",
    )
        .into_response()
}

async fn serve_public_spa(State(state): State<Arc<AppState>>) -> Response {
    serve_spa_index_response(&state).await
}

async fn serve_protected_spa_path(
    state: &AppState,
    headers: &HeaderMap,
    login_redirect: &str,
) -> Response {
    if auth_is_active(state) && extract_principal(state, headers).is_none() {
        return redirect_302(login_redirect);
    }
    serve_spa_index_response(state).await
}

async fn serve_protected_keys_spa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    serve_protected_spa_path(&state, &headers, "/login.html?next=%2Fkeys").await
}

async fn serve_protected_api_keys_spa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    serve_protected_spa_path(&state, &headers, "/login.html?next=%2Fapi-keys").await
}

async fn serve_protected_mcp_connectors_spa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    serve_protected_spa_path(&state, &headers, "/login.html?next=%2Fmcp-connectors").await
}

async fn serve_protected_settings_spa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    serve_protected_spa_path(&state, &headers, "/login.html?next=%2Fsettings").await
}

async fn serve_protected_admin_spa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    serve_protected_spa_path(&state, &headers, "/login.html?next=%2Fadmin").await
}

async fn serve_spa_fallback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(full_path): AxumPath<String>,
) -> Response {
    let clean = full_path.trim_matches('/');
    if clean == "api"
        || clean.starts_with("api/")
        || clean == "mcp"
        || clean.starts_with("mcp/")
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": "Not found" })),
        )
            .into_response();
    }

    if auth_is_active(&state) && extract_principal(&state, &headers).is_none() {
        if clean == "keys" || clean.starts_with("keys/") {
            return redirect_302("/login.html?next=%2Fkeys");
        }
        if clean == "api-keys" || clean.starts_with("api-keys/") {
            return redirect_302("/login.html?next=%2Fapi-keys");
        }
        if clean == "mcp-connectors" || clean.starts_with("mcp-connectors/") {
            return redirect_302("/login.html?next=%2Fmcp-connectors");
        }
        if clean == "settings" || clean.starts_with("settings/") {
            return redirect_302("/login.html?next=%2Fsettings");
        }
        if clean == "admin" || clean.starts_with("admin/") {
            return redirect_302("/login.html?next=%2Fadmin");
        }
    }

    // Serve static files strictly confined within ui/dist, ui/public, or ui/
    let static_roots = [
        state.settings.ui_dist_dir.clone(),
        state.settings.ui_dir.join("public"),
        state.settings.ui_dir.clone(),
    ];
    for root in &static_roots {
        if root.is_dir() {
            if let Some(file_path) = store::resolve_confined_file(root, clean) {
                if let Some(resp) = serve_disk_file(&file_path).await {
                    return resp;
                }
            }
        }
    }

    // Never return SPA HTML for missing hashed JS/CSS/image assets
    if clean.starts_with("static/") || clean.starts_with("assets/") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": "Asset not found" })),
        )
            .into_response();
    }

    serve_spa_index_response(&state).await
}

pub fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "md" | "txt" | "tsx" | "ts" | "jsx" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        _ => "application/octet-stream",
    }
}

async fn serve_disk_file(path: &Path) -> Option<Response> {
    let bytes = tokio::fs::read(path).await.ok()?;
    let ct = content_type_for_path(path);
    Some((StatusCode::OK, [(header::CONTENT_TYPE, ct)], bytes).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnvSnapshot;
    use crate::oidc::OidcSettings;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    use sha2::{Digest, Sha256};
    use std::sync::Mutex;

    const HEX_MASTER: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    const TEST_RSA_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDanCl3dKzLmtqp
vp9kQqbgVkWSdv81I6UxfnNUArMePza6hEiK9gIrqwNZEtC9azOemGvkEIxpMNvD
garbakIv5NWYhP+V/+1QuehNypWjsKC1LZKtSEzizsTyMHyE7Ap2hWvmLXi08kBL
iCl2CsSUeK2abtHyD1BzXsaewykSVU8qXTA5TG18cmp8khFhhlYAnB6djPllhy4W
jMjTKPEXYyxOv2jgubUrTI8bfiKP3L/9DivP/4McdiAb/ZI85c2ZEhiuhL2oY27F
2XK4TQ7n2ZkyFw4IvfNpmwRTqVM3KGXE5yj3CeGpYHZnnT7sgGM2XzeMEW+dpa9t
6D3WuLSLAgMBAAECggEAaRO4xsdZK0CSrBaJLLrw1VICfOUfxF4O0Bx0uMPXbsl5
t+tPpyOaZXz8zfrCRNnbZPbknSnrI8CtUWrpQi/wAm9ez8HVzDbIzg2eAI5IoPaM
OKngH9RpyHcUqUjvAUEnwlJdS7U9mNP8XZHkQ+3nEWp0H4/27/+2bs32cGdkPX/K
B/c6jElxYkVbIfCfPvmwtCoYdtkUCdT9RpcfspGyYYaqk/a2eR6Yz/leKSk7fId2
i0cTbLUX+/wlKAZQGfSl6RxuXrEm9RSLWfDrXsipj5Lu+veWp7CHaxKXeVE1NyLX
RJMirpcjIJbD/K+3uslui1OgJ9d0jEuO5G8iEACaaQKBgQD7WfArn+Q29dI4OQ18
DAngsuX6QmBAHOVAU9PuVOiYXMEs+hcuIEbT5y6YEebnHs9KwdBdt7O9l0IWFwqS
3+u1/j73Xbx8xXQJG+kNpoSNK9xGuQGfGNTA6nR1DZY84qGzuGwcRtEKfpuB78pY
QMXrfxlyj6FpO8hOhvIAvp69TQKBgQDepzR55W7EDAlFibXe08J/OCclrFLjRnZk
m/qORFhgsxUrIwI0ftt82xHMVdlH7RuCKcOQtRa9MqrNTZ1VKblEMrXgzfp0gWsh
kvyp45UgdrBdnZODstyI8aPN6DrObA+JxPIpMObu+wipgvfNF4cxhOr/hvgMAgy/
aG/JOvWtNwKBgQCGaBii61RZA2iPggqw66sYifGqunXHsuvwUyNADg129Bqkl3Nv
yfi19aiXzvLfYj1M54kppEMsPccsEtQnqC/eWP+IvYo+WkPrXw5nq0S0TOD0vJee
YOPb6UOvGD3ffv0jIppFIKVF7F/xwq08Nh7gCE4uUZnpsPsy6rxcI5K0/QKBgAMI
qjli+/Bl7ZZp+vVvUz4iiL4FrBd2tVRHn9FicwXWStqMnpISjHEWFxb+sHg/bdmH
Mif9n7KyRwuHHDa129VnD31hsSD3Rnttok26USThGSQdao4n2MNw73d1y59Wb5vD
+DzBctQczZBpvXCo+mmdUwpqT2TyfjP5e1R1njVfAoGAbnlF04R8W06khOZD9pN6
re/jMjDKZfcE7/th1veJlCqC3urowQz3JLMDhYF8f8civ2pKs55O+w3+ShGZQm7v
kihVJ6UI9STSxTcNWpU1wYdYYBKe8AhhyJ0Jbyg0Ez1CeHibH6Stk1x2P4Ay/Qwq
Vf8UC7ZUga203h6vN6bHdL0=
-----END PRIVATE KEY-----";
    const TEST_RSA_N: &str = "2pwpd3Ssy5raqb6fZEKm4FZFknb_NSOlMX5zVAKzHj82uoRIivYCK6sDWRLQvWsznphr5BCMaTDbw4Gq22pCL-TVmIT_lf_tULnoTcqVo7CgtS2SrUhM4s7E8jB8hOwKdoVr5i14tPJAS4gpdgrElHitmm7R8g9Qc17GnsMpElVPKl0wOUxtfHJqfJIRYYZWAJwenYz5ZYcuFozI0yjxF2MsTr9o4Lm1K0yPG34ij9y__Q4rz_-DHHYgG_2SPOXNmRIYroS9qGNuxdlyuE0O59mZMhcOCL3zaZsEU6lTNyhlxOco9wnhqWB2Z50-7IBjNl83jBFvnaWvbeg91ri0iw";
    const TEST_RSA_E: &str = "AQAB";

    fn populate_fixture_corpus(data: &Path) {
        let sdir = data.join("refero-styles").join("styles").join("s-linear-1");
        std::fs::create_dir_all(sdir.join("shots")).unwrap();
        std::fs::write(
            data.join("refero-styles").join("index.json"),
            r#"[{"id":"s-linear-1","title":"Linear Dark","tagline":"Precision issue tracker","theme":"dark","source_url":"https://linear.app"}]"#,
        )
        .unwrap();
        std::fs::write(sdir.join("DESIGN.md"), "# Linear Dark\nTokens: #090d16").unwrap();
        std::fs::write(sdir.join("shots").join("01.svg"), "<svg/>").unwrap();

        let web_en = data.join("prompt-driven-ui").join("websites-en");
        let prompts_en = data.join("prompt-driven-ui").join("prompts-en");
        std::fs::create_dir_all(&web_en).unwrap();
        std::fs::create_dir_all(&prompts_en).unwrap();
        std::fs::write(
            web_en.join("01-fintech.html"),
            "<html><head><title>Fintech Landing</title></head><body>Demo</body></html>",
        )
        .unwrap();
        std::fs::write(prompts_en.join("01-fintech.md"), "Prompt text").unwrap();

        let cdir = data.join("lnc-components").join("items").join("lnc-btn");
        std::fs::create_dir_all(&cdir).unwrap();
        std::fs::write(
            data.join("lnc-components").join("index.json"),
            r#"[{"id":"lnc-btn","name":"LNC Button","category":"buttons","kind":"html","description":"Glow"}]"#,
        )
        .unwrap();
        std::fs::write(cdir.join("index.html"), "<button>Glow</button>").unwrap();
    }

    #[tokio::test]
    async fn full_router_auth_keys_mcp_ssrf_and_oidc_roundtrip() {
        // 1. Start mock Keycloak OIDC IdP
        #[derive(Default)]
        struct IdpState {
            code_challenge: String,
            nonce: String,
        }
        let idp_state = Arc::new(Mutex::new(IdpState::default()));
        let idp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let idp_base = format!("http://{}/realms/lengoc", idp_listener.local_addr().unwrap());
        let idp_base_disc = idp_base.clone();
        let idp_base_tok = idp_base.clone();
        let idp_st_tok = idp_state.clone();

        let idp_router = Router::new()
            .route(
                "/realms/lengoc/.well-known/openid-configuration",
                get(move || {
                    let b = idp_base_disc.clone();
                    async move {
                        Json(json!({
                            "issuer": b,
                            "authorization_endpoint": format!("{b}/protocol/openid-connect/auth"),
                            "token_endpoint": format!("{b}/protocol/openid-connect/token"),
                            "jwks_uri": format!("{b}/protocol/openid-connect/certs"),
                        }))
                    }
                }),
            )
            .route(
                "/realms/lengoc/protocol/openid-connect/certs",
                get(|| async move {
                    Json(json!({
                        "keys": [{
                            "kty": "RSA",
                            "alg": "RS256",
                            "use": "sig",
                            "kid": "k1",
                            "n": TEST_RSA_N,
                            "e": TEST_RSA_E,
                        }]
                    }))
                }),
            )
            .route(
                "/realms/lengoc/protocol/openid-connect/token",
                post(move |body: String| {
                    let st = idp_st_tok.clone();
                    let iss = idp_base_tok.clone();
                    async move {
                        let params: std::collections::HashMap<String, String> =
                            url::form_urlencoded::parse(body.as_bytes())
                                .into_owned()
                                .collect();
                        let verifier = params.get("code_verifier").cloned().unwrap_or_default();
                        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
                        let guard = st.lock().unwrap();
                        assert_eq!(challenge, guard.code_challenge, "PKCE S256 mismatch");
                        let nonce = guard.nonce.clone();
                        drop(guard);

                        let now = now_secs();
                        let claims = json!({
                            "iss": iss,
                            "sub": "u-louis",
                            "aud": "lnc-styles-gallery",
                            "exp": now + 300,
                            "iat": now,
                            "nonce": nonce,
                            "preferred_username": "louis",
                            "email": "louis@lengoc.me",
                            "groups": ["/admins"]
                        });
                        let mut hdr = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
                        hdr.kid = Some("k1".into());
                        let id_token = jsonwebtoken::encode(
                            &hdr,
                            &claims,
                            &jsonwebtoken::EncodingKey::from_rsa_pem(TEST_RSA_PEM.as_bytes()).unwrap(),
                        )
                        .unwrap();
                        Json(json!({ "id_token": id_token, "token_type": "Bearer" }))
                    }
                }),
            );
        tokio::spawn(async move {
            let _ = axum::serve(idp_listener, idp_router).await;
        });

        // 2. Seed temp corpus + legacy keys.json
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().join("data");
        let cfg_dir = tmp.path().join("cfg");
        populate_fixture_corpus(&data_dir);
        std::fs::create_dir_all(&cfg_dir).unwrap();
        std::fs::write(
            cfg_dir.join("keys.json"),
            r#"{"api_keys":[{"id":"hub_default","label":"Hub","key":"styles_LegacyPlaintextToken99999","enabled":true}]}"#,
        )
        .unwrap();

        let app_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let app_addr = app_listener.local_addr().unwrap();
        let app_base = format!("http://{app_addr}");
        let redirect_uri = format!("{app_base}/api/auth/oidc/callback");

        let env = EnvSnapshot::from_pairs([
            ("STYLES_DATA_DIR", data_dir.to_str().unwrap()),
            ("STYLES_CONFIG_DIR", cfg_dir.to_str().unwrap()),
            ("LNC_MASTER_KEY", HEX_MASTER),
            ("MANAGEMENT_KEY", "mgmt-secret-key-2026"),
            ("STYLES_MCP_AUTH", "1"),
            ("PUBLIC_BASE_URL", app_base.as_str()),
            ("OIDC_ISSUER_URL", idp_base.as_str()),
            ("OIDC_CLIENT_ID", "lnc-styles-gallery"),
            ("OIDC_CLIENT_SECRET", "oidc-client-secret"),
            ("OIDC_REDIRECT_URI", redirect_uri.as_str()),
            ("OIDC_ALLOWED_USERS", "louis"),
        ]);

        let settings = AppSettings::from_map(&env);
        let master = crate::crypto::MasterKeys::from_encoded(HEX_MASTER).unwrap();
        let runtime = Arc::new(
            RuntimeStore::load_or_init(&settings.config_dir, master, &env).unwrap(),
        );
        let corpus = Arc::new(StyleLibraryStore::new(settings.clone()));
        corpus.reindex();
        let oidc = Arc::new(OidcClient::new(
            OidcSettings::from_map(&env, &settings.public_url),
            reqwest::Client::new(),
        ));
        let state = Arc::new(AppState {
            settings,
            runtime,
            corpus,
            oidc,
            limiter: Arc::new(LoginLimiter::default()),
        });
        let router = build_router(state).into_make_service_with_connect_info::<SocketAddr>();
        tokio::spawn(async move {
            let _ = axum::serve(app_listener, router).await;
        });

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        // Health & corpus public endpoints
        let h: Value = client
            .get(format!("{app_base}/health"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(h["status"], "ok");
        assert_eq!(h["styles_count"], 1);
        assert_eq!(h["templates_count"], 1);
        assert_eq!(h["components_count"], 1);

        // Unauthenticated /keys redirects 302 -> /login.html, /api/keys returns 401
        let r_keys_html = client.get(format!("{app_base}/keys")).send().await.unwrap();
        assert_eq!(r_keys_html.status(), StatusCode::FOUND);
        assert_eq!(
            r_keys_html.headers().get("location").unwrap(),
            "/login.html?next=%2Fkeys"
        );
        let r_keys_api = client.get(format!("{app_base}/api/keys")).send().await.unwrap();
        assert_eq!(r_keys_api.status(), StatusCode::UNAUTHORIZED);

        // OIDC PKCE S256 roundtrip
        let r_oidc_start = client
            .get(format!("{app_base}/api/auth/oidc/login?next=/keys"))
            .send()
            .await
            .unwrap();
        assert_eq!(r_oidc_start.status(), StatusCode::FOUND);
        let auth_loc = r_oidc_start
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let tx_set_cookie = r_oidc_start
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let auth_url = Url::parse(&auth_loc).unwrap();
        let qp: std::collections::HashMap<String, String> =
            auth_url.query_pairs().into_owned().collect();
        assert_eq!(qp.get("code_challenge_method").map(String::as_str), Some("S256"));
        {
            let mut g = idp_state.lock().unwrap();
            g.code_challenge = qp["code_challenge"].clone();
            g.nonce = qp["nonce"].clone();
        }
        let r_cb = client
            .get(format!(
                "{app_base}/api/auth/oidc/callback?code=testcode&state={}",
                qp["state"]
            ))
            .header("cookie", &tx_set_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(r_cb.status(), StatusCode::FOUND);
        assert_eq!(r_cb.headers().get("location").unwrap(), "/keys");
        let oidc_sess_cookie = r_cb
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|s| s.starts_with("lnc_styles_session=v2."))
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();

        // Verify GET /api/keys Zero-F12-Leak + POST /copy vault decrypt + audit.jsonl
        let keys_resp = client
            .get(format!("{app_base}/api/keys"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(keys_resp.status(), StatusCode::OK);
        assert!(keys_resp
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .contains("no-store"));
        let keys_body = keys_resp.text().await.unwrap();
        assert!(!keys_body.contains("raw_key"));
        assert!(!keys_body.contains("LegacyPlaintextToken99999"));

        // GET /copy must be 405 Method Not Allowed
        let copy_get_resp = client
            .get(format!("{app_base}/api/keys/hub_default/copy"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(copy_get_resp.status(), StatusCode::METHOD_NOT_ALLOWED);

        // POST /copy returns 200 encrypted secret + no-store + audit.jsonl
        let copy_resp = client
            .post(format!("{app_base}/api/keys/hub_default/copy"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(copy_resp.status(), StatusCode::OK);
        assert!(copy_resp
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .contains("no-store"));
        let copy_json: Value = copy_resp.json().await.unwrap();
        assert_eq!(copy_json["secret"], "styles_LegacyPlaintextToken99999");

        // Canonical /api/admin/api-keys create -> patch rename -> copy -> delete
        let created_key: Value = client
            .post(format!("{app_base}/api/admin/api-keys"))
            .header("cookie", &oidc_sess_cookie)
            .json(&json!({ "label": "Canonical Key", "scopes": ["mcp:read", "styles:read"] }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let created_id = created_key["item"]["id"].as_str().unwrap().to_string();
        let renamed_resp = client
            .patch(format!("{app_base}/api/admin/api-keys/{created_id}"))
            .header("cookie", &oidc_sess_cookie)
            .json(&json!({ "label": "Canonical Key Renamed" }))
            .send()
            .await
            .unwrap();
        assert_eq!(renamed_resp.status(), StatusCode::OK);
        let del_resp = client
            .delete(format!("{app_base}/api/admin/api-keys/{created_id}"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(del_resp.status(), StatusCode::OK);

        // Copy rate limit (15 / 5m / IP -> 16th returns 429 + Retry-After)
        for _ in 0..15 {
            let r = client
                .post(format!("{app_base}/api/admin/api-keys/hub_default/copy"))
                .header("cookie", &oidc_sess_cookie)
                .header("x-forwarded-for", "10.88.1.99")
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), StatusCode::OK);
        }
        let copy_16th = client
            .post(format!("{app_base}/api/admin/api-keys/hub_default/copy"))
            .header("cookie", &oidc_sess_cookie)
            .header("x-forwarded-for", "10.88.1.99")
            .send()
            .await
            .unwrap();
        assert_eq!(copy_16th.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(copy_16th.headers().contains_key("retry-after"));

        // Global ?token=, ?key=, ?api_key= query params blocked with 400
        for qs in ["?token=abc", "?key=abc", "?api_key=abc"] {
            let qr = client
                .get(format!("{app_base}/api/styles{qs}"))
                .send()
                .await
                .unwrap();
            assert_eq!(qr.status(), StatusCode::BAD_REQUEST);
        }

        // Protected SPA routes redirect to /login.html when unauthenticated
        for p in ["/keys", "/api-keys", "/mcp-connectors", "/settings", "/admin"] {
            let sr = client.get(format!("{app_base}{p}")).send().await.unwrap();
            assert_eq!(sr.status(), StatusCode::FOUND);
            assert!(sr
                .headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("/login.html"));
        }

        // MCP Connectors create -> copy -> test -> rotate -> revoke -> delete
        let conn_create: Value = client
            .post(format!("{app_base}/api/admin/mcp-connectors"))
            .header("cookie", &oidc_sess_cookie)
            .json(&json!({ "name": "Playwright Agent" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let conn_id = conn_create["item"]["id"].as_str().unwrap().to_string();
        let conn_sec = conn_create["secret"].as_str().unwrap().to_string();
        let conn_copy: Value = client
            .post(format!("{app_base}/api/admin/mcp-connectors/{conn_id}/copy"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(conn_copy["secret"], conn_sec);
        let conn_test: Value = client
            .post(format!("{app_base}/api/admin/mcp-connectors/{conn_id}/test"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(conn_test["tools_count"], 8);
        assert_eq!(conn_test["status"], "healthy");
        let conn_del = client
            .delete(format!("{app_base}/api/admin/mcp-connectors/{conn_id}"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(conn_del.status(), StatusCode::OK);

        // Fetch session CSRF token + verify GET /logout -> 405, strict POST /logout CSRF & X-Forwarded-Host defense
        let sess_info: Value = client
            .get(format!("{app_base}/api/auth/session"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let csrf_tok = sess_info["csrf_token"].as_str().unwrap().to_string();
        assert!(!csrf_tok.is_empty());

        let logout_get = client
            .get(format!("{app_base}/api/auth/logout"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(logout_get.status(), StatusCode::METHOD_NOT_ALLOWED);

        let logout_no_csrf = client
            .post(format!("{app_base}/api/auth/logout"))
            .header("cookie", &oidc_sess_cookie)
            .header("origin", &app_base)
            .send()
            .await
            .unwrap();
        assert_eq!(logout_no_csrf.status(), StatusCode::FORBIDDEN);

        let logout_xfh_spoof = client
            .post(format!("{app_base}/api/auth/logout"))
            .header("cookie", &oidc_sess_cookie)
            .header("origin", "https://evil.example")
            .header("x-forwarded-host", "evil.example")
            .header("x-csrf-token", &csrf_tok)
            .send()
            .await
            .unwrap();
        assert_eq!(logout_xfh_spoof.status(), StatusCode::FORBIDDEN);

        let logout_ok = client
            .post(format!("{app_base}/api/auth/logout"))
            .header("cookie", &oidc_sess_cookie)
            .header("origin", &app_base)
            .header("x-csrf-token", &csrf_tok)
            .send()
            .await
            .unwrap();
        assert_eq!(logout_ok.status(), StatusCode::OK);

        // CSRF cross-origin POST rejected with 403
        let csrf_resp = client
            .post(format!("{app_base}/api/keys"))
            .header("cookie", &oidc_sess_cookie)
            .header("origin", "https://evil.example")
            .header("x-forwarded-host", "evil.example")
            .json(&json!({ "label": "Attacker" }))
            .send()
            .await
            .unwrap();
        assert_eq!(csrf_resp.status(), StatusCode::FORBIDDEN);

        // SSRF blocked on PUT /api/admin/settings
        let ssrf_resp = client
            .put(format!("{app_base}/api/admin/settings"))
            .header("cookie", &oidc_sess_cookie)
            .json(&json!({ "public_base_url": "http://169.254.169.254/latest/meta-data" }))
            .send()
            .await
            .unwrap();
        assert_eq!(ssrf_resp.status(), StatusCode::BAD_REQUEST);

        // Revoke-all invalidates existing OIDC session cookie
        let rev_resp = client
            .post(format!("{app_base}/api/auth/revoke-all"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(rev_resp.status(), StatusCode::OK);
        let post_rev = client
            .get(format!("{app_base}/api/keys"))
            .header("cookie", &oidc_sess_cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(post_rev.status(), StatusCode::UNAUTHORIZED);

        // MCP notification -> 202 & all 8 tools listed with migrated Bearer token
        let mcp_notif = client
            .post(format!("{app_base}/mcp"))
            .header("authorization", "Bearer styles_LegacyPlaintextToken99999")
            .json(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
            .send()
            .await
            .unwrap();
        assert_eq!(mcp_notif.status(), StatusCode::ACCEPTED);

        let mcp_list: Value = client
            .post(format!("{app_base}/mcp"))
            .header("authorization", "Bearer styles_LegacyPlaintextToken99999")
            .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(mcp_list["result"]["tools"].as_array().unwrap().len(), 8);

        // Rate limit 8/5m -> 9th bad login returns 429 + Retry-After
        for _ in 0..8 {
            let r = client
                .post(format!("{app_base}/api/auth/login"))
                .json(&json!({ "key": "wrong-key" }))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
        }
        let ninth = client
            .post(format!("{app_base}/api/auth/login"))
            .json(&json!({ "key": "wrong-key" }))
            .send()
            .await
            .unwrap();
        assert_eq!(ninth.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(ninth.headers().contains_key("retry-after"));
    }
}
