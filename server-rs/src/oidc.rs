//! Management sessions (`lnc_styles_session` v2) + Keycloak OIDC SSO (`lengoc` realm).
//!
//! - `lnc_styles_session` v2 signed session cookie:
//!   `v2.<b64url(json claims)>.<b64url(HMAC-SHA256)>` keyed by HKDF `session-hmac`;
//!   `12h` sliding idle TTL, `7d` hard max TTL, and `session_epoch` instant revocation.
//! - Per-IP login brute-force limiter (`8` failures / `5` minutes -> `429` + `Retry-After`).
//! - OIDC Authorization Code + PKCE `S256` + `state` + `nonce`, sealed with AES-256-GCM
//!   (`oidc-tx` subkey) in a 5-minute `lnc_styles_oidc_tx` cookie; ID token verified
//!   against cached JWKS (`RS256`/`ES256`, `iss`, `aud`, `exp`, `nonce`) and
//!   `OIDC_ALLOWED_USERS` / `OIDC_ALLOWED_GROUPS`.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::config::{now_secs, EnvSnapshot};
use crate::crypto::{self, MasterKeys};

pub const SESSION_COOKIE: &str = "lnc_styles_session";
pub const OIDC_TX_COOKIE: &str = "lnc_styles_oidc_tx";
/// Sliding idle lifetime (12h).
pub const IDLE_TTL_SECS: i64 = 43_200;
/// Hard maximum lifetime (7d).
pub const MAX_TTL_SECS: i64 = 604_800;
/// Re-issue sliding cookie at most every 15 minutes.
pub const RENEW_AFTER_SECS: i64 = 900;
/// OIDC transaction cookie lifetime (5 minutes).
pub const OIDC_TX_TTL_SECS: i64 = 300;
pub const SESSION_VERSION: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionTtlConfig {
    pub idle_ttl_secs: i64,
    pub max_ttl_secs: i64,
}

impl Default for SessionTtlConfig {
    fn default() -> Self {
        Self {
            idle_ttl_secs: IDLE_TTL_SECS,
            max_ttl_secs: MAX_TTL_SECS,
        }
    }
}

impl SessionTtlConfig {
    pub fn from_env() -> Self {
        Self::from_map(&EnvSnapshot::from_process())
    }

    pub fn from_map(env: &EnvSnapshot) -> Self {
        let idle = env
            .first(&["SESSION_IDLE_TTL_SECONDS", "STYLES_SESSION_IDLE_TTL_SECONDS"])
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|&v| v >= 60)
            .unwrap_or(IDLE_TTL_SECS);
        let max = env
            .first(&["SESSION_MAX_TTL_SECONDS", "STYLES_SESSION_MAX_TTL_SECONDS"])
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|&v| v >= idle)
            .unwrap_or(MAX_TTL_SECS)
            .max(idle);
        Self {
            idle_ttl_secs: idle,
            max_ttl_secs: max,
        }
    }
}

const TX_FIELD_PATH: &str = "oidc-tx";
const DISCOVERY_TTL: Duration = Duration::from_secs(3600);
const JWKS_TTL: Duration = Duration::from_secs(600);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const JWT_LEEWAY_SECS: u64 = 60;

fn b64u(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

fn random_b64u(n: usize) -> String {
    let mut b = vec![0u8; n];
    rand::rngs::OsRng.fill_bytes(&mut b);
    b64u(&b)
}

/// Sanitize post-login relative redirect target to prevent open redirects.
pub fn sanitize_next_path(raw: Option<&str>) -> String {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return "/styles".to_string();
    };
    if !s.starts_with('/') || s.starts_with("//") || s.starts_with("/\\") || s.contains('\r') || s.contains('\n') {
        return "/styles".to_string();
    }
    if s.starts_with("/login.html") || s.starts_with("/api/auth/") {
        return "/styles".to_string();
    }
    s.chars().take(256).collect()
}

// ---------------------------------------------------------------------------
// Session cookie (v2)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionClaims {
    pub v: u8,
    /// `"oidc"` or `"mgmt_key"`.
    pub mode: String,
    pub sub: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<Value>,
    pub iat: i64,
    pub last: i64,
    pub epoch: i64,
}

pub fn new_session(
    mode: &str,
    sub: &str,
    user: Option<Value>,
    epoch: i64,
    now: i64,
) -> SessionClaims {
    SessionClaims {
        v: SESSION_VERSION,
        mode: mode.to_string(),
        sub: sub.to_string(),
        user,
        iat: now,
        last: now,
        epoch,
    }
}

fn session_mac(keys: &MasterKeys, signed_part: &str) -> [u8; 32] {
    crypto::hmac_sha256(keys.session_hmac(), signed_part.as_bytes())
}

pub fn sign_session(keys: &MasterKeys, claims: &SessionClaims) -> String {
    let payload = b64u(&serde_json::to_vec(claims).unwrap_or_default());
    let signed = format!("v{SESSION_VERSION}.{payload}");
    let mac = session_mac(keys, &signed);
    format!("{signed}.{}", b64u(&mac))
}

pub fn verify_session(
    keys: &MasterKeys,
    value: &str,
    current_epoch: i64,
    now: i64,
) -> Option<SessionClaims> {
    verify_session_with_ttl(keys, value, current_epoch, now, SessionTtlConfig::default())
}

pub fn verify_session_with_ttl(
    keys: &MasterKeys,
    value: &str,
    current_epoch: i64,
    now: i64,
    ttl: SessionTtlConfig,
) -> Option<SessionClaims> {
    let value = value.trim();
    if value.len() > 8192 {
        return None;
    }
    let mut parts = value.split('.');
    let (ver, payload, sig) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || ver != "v2" || payload.is_empty() {
        return None;
    }
    let sig = URL_SAFE_NO_PAD.decode(sig).ok()?;
    let expected = session_mac(keys, &format!("{ver}.{payload}"));
    if !crypto::ct_eq(&expected, &sig) {
        return None;
    }
    let claims: SessionClaims =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()?;
    if claims.v != SESSION_VERSION || claims.epoch != current_epoch {
        return None;
    }
    if claims.mode.is_empty()
        || claims.iat > now + 60
        || claims.last > now + 60
        || claims.last < claims.iat
    {
        return None;
    }
    if now - claims.last > ttl.idle_ttl_secs || now - claims.iat > ttl.max_ttl_secs {
        return None;
    }
    Some(claims)
}

pub fn needs_renewal(claims: &SessionClaims, now: i64) -> bool {
    now - claims.last >= RENEW_AFTER_SECS
}

pub fn renew(mut claims: SessionClaims, now: i64) -> SessionClaims {
    claims.last = now.max(claims.last);
    claims
}

pub fn expires_info(claims: &SessionClaims) -> (i64, i64) {
    expires_info_with_ttl(claims, SessionTtlConfig::default())
}

pub fn expires_info_with_ttl(claims: &SessionClaims, ttl: SessionTtlConfig) -> (i64, i64) {
    let hard = claims.iat + ttl.max_ttl_secs;
    let idle = (claims.last + ttl.idle_ttl_secs).min(hard);
    (hard * 1000, idle * 1000)
}

pub fn session_cookie(value: &str, secure: bool) -> String {
    session_cookie_with_ttl(value, secure, MAX_TTL_SECS)
}

pub fn session_cookie_with_ttl(value: &str, secure: bool, max_ttl_secs: i64) -> String {
    let ttl = max_ttl_secs.max(60);
    format!(
        "{SESSION_COOKIE}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={ttl}{}",
        if secure { "; Secure" } else { "" }
    )
}

pub fn derive_session_csrf_token(keys: &MasterKeys, claims: &SessionClaims) -> String {
    let msg = format!(
        "csrf:{}:{}:{}:{}",
        claims.mode, claims.sub, claims.iat, claims.epoch
    );
    b64u(&crypto::hmac_sha256(keys.session_hmac(), msg.as_bytes()))
}

pub fn verify_session_csrf_token(
    keys: &MasterKeys,
    claims: &SessionClaims,
    candidate: &str,
) -> bool {
    let cand = candidate.trim();
    if cand.is_empty() {
        return false;
    }
    let expected = derive_session_csrf_token(keys, claims);
    crypto::ct_eq_str(&expected, cand)
}

pub fn oidc_tx_cookie(value: &str, secure: bool) -> String {
    format!(
        "{OIDC_TX_COOKIE}={value}; Path=/api/auth/oidc; HttpOnly; SameSite=Lax; Max-Age={OIDC_TX_TTL_SECS}{}",
        if secure { "; Secure" } else { "" }
    )
}

pub fn clear_cookie(name: &str, secure: bool) -> String {
    let path = if name == OIDC_TX_COOKIE {
        "/api/auth/oidc"
    } else {
        "/"
    };
    format!(
        "{name}=; Path={path}; HttpOnly; SameSite=Lax; Max-Age=0{}",
        if secure { "; Secure" } else { "" }
    )
}

pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    for hv in headers.get_all(axum::http::header::COOKIE) {
        let Ok(s) = hv.to_str() else { continue };
        for part in s.split(';') {
            if let Some((k, v)) = part.split_once('=') {
                if k.trim() == name {
                    let v = v.trim().trim_matches('"');
                    if !v.is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn request_is_secure(headers: &HeaderMap, cookie_secure_env: bool) -> bool {
    if cookie_secure_env {
        return true;
    }
    let xfp = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().eq_ignore_ascii_case("https"))
        .unwrap_or(false);
    if xfp {
        return true;
    }
    headers
        .get("forwarded")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().contains("proto=https"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Login rate limiter (8 failures / 5 minutes -> 429 + Retry-After)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct LoginLimiter {
    max: u32,
    window: Duration,
    map: Mutex<HashMap<String, (u32, Instant)>>,
}

impl LoginLimiter {
    pub fn new(max: u32, window: Duration) -> Self {
        Self {
            max,
            window,
            map: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, (u32, Instant)>> {
        self.map.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn check(&self, ip: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut m = self.lock();
        match m.get(ip) {
            Some((_, until)) if now >= *until => {
                m.remove(ip);
                Ok(())
            }
            Some((n, until)) if *n >= self.max => {
                let secs = until.saturating_duration_since(now).as_secs_f64().ceil() as u64;
                Err(secs.max(1))
            }
            _ => Ok(()),
        }
    }

    pub fn fail(&self, ip: &str) {
        let now = Instant::now();
        let mut m = self.lock();
        if m.len() > 10_000 {
            m.retain(|_, (_, until)| *until > now);
        }
        let e = m.entry(ip.to_string()).or_insert((0, now + self.window));
        if now >= e.1 {
            *e = (0, now + self.window);
        }
        e.0 += 1;
    }

    pub fn ok(&self, ip: &str) {
        self.lock().remove(ip);
    }

    /// Increment fixed-window hit counter (used for `/copy` `15/5m/IP` throttling).
    pub fn check_and_hit(&self, ip: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut m = self.lock();
        if m.len() > 10_000 {
            m.retain(|_, (_, until)| *until > now);
        }
        let e = m.entry(ip.to_string()).or_insert((0, now + self.window));
        if now >= e.1 {
            *e = (0, now + self.window);
        }
        if e.0 >= self.max {
            let secs = e.1.saturating_duration_since(now).as_secs_f64().ceil() as u64;
            return Err(secs.max(1));
        }
        e.0 += 1;
        Ok(())
    }
}

impl Default for LoginLimiter {
    fn default() -> Self {
        Self::new(8, Duration::from_secs(300))
    }
}

// ---------------------------------------------------------------------------
// OIDC Settings & Client
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OidcSettings {
    pub enabled: bool,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub allowed_users: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub scopes: String,
}

fn split_list(v: Option<&str>) -> Vec<String> {
    v.unwrap_or("")
        .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

impl OidcSettings {
    pub fn from_env(default_public_url: &str) -> Self {
        Self::from_map(&EnvSnapshot::from_process(), default_public_url)
    }

    pub fn from_map(env: &EnvSnapshot, default_public_url: &str) -> Self {
        let issuer = env
            .first(&["OIDC_ISSUER", "OIDC_ISSUER_URL"])
            .unwrap_or("")
            .trim()
            .trim_end_matches('/')
            .to_string();
        let client_id = env.first(&["OIDC_CLIENT_ID"]).unwrap_or("").to_string();
        let client_secret = env.first(&["OIDC_CLIENT_SECRET"]).unwrap_or("").to_string();
        let redirect_uri = env
            .first(&["OIDC_REDIRECT_URI"])
            .map(str::to_string)
            .unwrap_or_else(|| {
                if !issuer.is_empty() && !client_id.is_empty() {
                    format!(
                        "{}/api/auth/oidc/callback",
                        default_public_url.trim_end_matches('/')
                    )
                } else {
                    String::new()
                }
            });

        let explicit = env
            .get("OIDC_ENABLED")
            .map(|v| v.trim().to_ascii_lowercase());
        let enabled = match explicit.as_deref() {
            Some("true") | Some("1") | Some("yes") | Some("on") => true,
            Some("false") | Some("0") | Some("no") | Some("off") => false,
            _ => !issuer.is_empty() && !client_id.is_empty(),
        };

        OidcSettings {
            enabled,
            issuer,
            client_id,
            client_secret,
            redirect_uri,
            allowed_users: split_list(env.get("OIDC_ALLOWED_USERS")),
            allowed_groups: split_list(env.get("OIDC_ALLOWED_GROUPS")),
            scopes: env
                .first(&["OIDC_SCOPES"])
                .unwrap_or("openid profile email")
                .to_string(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled
            && !self.issuer.is_empty()
            && !self.client_id.is_empty()
            && !self.redirect_uri.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OidcError {
    Disabled,
    Discovery(String),
    TxMissing,
    TxInvalid,
    TxExpired,
    StateMismatch,
    MissingCode,
    IdpError(String),
    TokenExchange(String),
    InvalidToken(String),
    NonceMismatch,
    NotAllowed,
}

impl OidcError {
    pub fn code(&self) -> &'static str {
        match self {
            OidcError::Disabled => "oidc_disabled",
            OidcError::Discovery(_) => "oidc_discovery",
            OidcError::TxMissing => "tx_missing",
            OidcError::TxInvalid => "tx_invalid",
            OidcError::TxExpired => "tx_expired",
            OidcError::StateMismatch => "state_mismatch",
            OidcError::MissingCode => "missing_code",
            OidcError::IdpError(_) => "idp_error",
            OidcError::TokenExchange(_) => "token_exchange",
            OidcError::InvalidToken(_) => "invalid_token",
            OidcError::NonceMismatch => "nonce_mismatch",
            OidcError::NotAllowed => "not_allowed",
        }
    }
}

impl fmt::Display for OidcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OidcError::Disabled => write!(f, "OIDC SSO is not enabled"),
            OidcError::Discovery(d) => write!(f, "OIDC discovery failed: {d}"),
            OidcError::TxMissing => write!(f, "OIDC transaction cookie missing"),
            OidcError::TxInvalid => write!(f, "OIDC transaction cookie invalid"),
            OidcError::TxExpired => write!(f, "OIDC transaction expired"),
            OidcError::StateMismatch => write!(f, "OIDC state mismatch"),
            OidcError::MissingCode => write!(f, "OIDC callback without authorization code"),
            OidcError::IdpError(e) => write!(f, "identity provider returned error: {e}"),
            OidcError::TokenExchange(d) => write!(f, "OIDC token exchange failed: {d}"),
            OidcError::InvalidToken(d) => write!(f, "OIDC ID token rejected: {d}"),
            OidcError::NonceMismatch => write!(f, "OIDC nonce mismatch"),
            OidcError::NotAllowed => write!(f, "user is not in the OIDC allowlist"),
        }
    }
}

impl std::error::Error for OidcError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OidcUser {
    pub sub: String,
    pub username: String,
    pub email: String,
    pub name: String,
    pub groups: Vec<String>,
}

impl OidcUser {
    pub fn to_json(&self) -> Value {
        json!({
            "username": self.username,
            "email": self.email,
            "name": self.name,
            "groups": self.groups,
        })
    }
}

/// Allowlist: email | preferred_username | sub in `OIDC_ALLOWED_USERS` (case-insensitive),
/// or group (with/without leading `/`) in `OIDC_ALLOWED_GROUPS`.
/// If `"*"` is explicitly passed in `OIDC_ALLOWED_USERS`, any valid realm user is allowed;
/// if both lists are empty, fail closed (deny).
pub fn is_allowed(settings: &OidcSettings, user: &OidcUser) -> bool {
    if settings.allowed_users.iter().any(|u| u == "*") {
        return true;
    }
    let users: Vec<String> = settings
        .allowed_users
        .iter()
        .map(|u| u.to_lowercase())
        .collect();
    let ids = [&user.email, &user.username, &user.sub];
    if ids
        .iter()
        .any(|id| !id.is_empty() && users.contains(&id.to_lowercase()))
    {
        return true;
    }
    let norm = |g: &str| g.trim().trim_start_matches('/').to_lowercase();
    let allowed: Vec<String> = settings
        .allowed_groups
        .iter()
        .map(|g| norm(g))
        .filter(|g| !g.is_empty())
        .collect();
    user.groups.iter().any(|g| allowed.contains(&norm(g)))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TxData {
    state: String,
    nonce: String,
    verifier: String,
    #[serde(default)]
    next: String,
    ts: i64,
}

fn seal_tx(keys: &MasterKeys, tx: &TxData) -> Result<String, OidcError> {
    let pt = serde_json::to_string(tx).map_err(|_| OidcError::TxInvalid)?;
    let env =
        crypto::seal_field(keys.oidc_tx(), TX_FIELD_PATH, &pt).map_err(|_| OidcError::TxInvalid)?;
    Ok(b64u(env.to_string().as_bytes()))
}

fn open_tx(keys: &MasterKeys, cookie: &str, now: i64) -> Result<TxData, OidcError> {
    let cookie = cookie.trim();
    if cookie.is_empty() {
        return Err(OidcError::TxMissing);
    }
    if cookie.len() > 4096 {
        return Err(OidcError::TxInvalid);
    }
    let raw = URL_SAFE_NO_PAD
        .decode(cookie)
        .map_err(|_| OidcError::TxInvalid)?;
    let env: Value = serde_json::from_slice(&raw).map_err(|_| OidcError::TxInvalid)?;
    let pt = crypto::open_field(keys.oidc_tx(), TX_FIELD_PATH, &env)
        .map_err(|_| OidcError::TxInvalid)?;
    let tx: TxData = serde_json::from_str(&pt).map_err(|_| OidcError::TxInvalid)?;
    if now - tx.ts > OIDC_TX_TTL_SECS || tx.ts > now + 60 {
        return Err(OidcError::TxExpired);
    }
    Ok(tx)
}

pub fn pkce_challenge(verifier: &str) -> String {
    b64u(&Sha256::digest(verifier.as_bytes()))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Discovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    #[serde(default)]
    pub end_session_endpoint: Option<String>,
}

fn de_groups<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Value::String(s) => vec![s],
        _ => Vec::new(),
    })
}

#[derive(Debug, Deserialize)]
struct IdClaims {
    sub: String,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    azp: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default, deserialize_with = "de_groups")]
    groups: Vec<String>,
}

fn allowed_alg(alg: Algorithm) -> bool {
    matches!(
        alg,
        Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::EdDSA
    )
}

pub struct OidcClient {
    settings: OidcSettings,
    http: reqwest::Client,
    discovery: Mutex<Option<(Discovery, Instant)>>,
    jwks: Mutex<Option<(Vec<Value>, Instant)>>,
}

impl fmt::Debug for OidcClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OidcClient")
            .field("enabled", &self.enabled())
            .field("issuer", &self.settings.issuer)
            .field("client_id", &self.settings.client_id)
            .finish()
    }
}

impl OidcClient {
    pub fn new(settings: OidcSettings, http: reqwest::Client) -> Self {
        Self {
            settings,
            http,
            discovery: Mutex::new(None),
            jwks: Mutex::new(None),
        }
    }

    pub fn settings(&self) -> &OidcSettings {
        &self.settings
    }

    pub fn enabled(&self) -> bool {
        self.settings.is_active()
    }

    async fn get_json(&self, url: &str) -> Result<Value, String> {
        let resp = self
            .http
            .get(url)
            .timeout(HTTP_TIMEOUT)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    "timeout".to_string()
                } else {
                    "unreachable".to_string()
                }
            })?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("http {}", status.as_u16()));
        }
        resp.json::<Value>()
            .await
            .map_err(|_| "bad json".to_string())
    }

    pub async fn discovery(&self) -> Result<Discovery, OidcError> {
        if let Some((d, at)) = self
            .discovery
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
        {
            if at.elapsed() < DISCOVERY_TTL {
                return Ok(d);
            }
        }
        let url = format!("{}/.well-known/openid-configuration", self.settings.issuer);
        let v = self.get_json(&url).await.map_err(OidcError::Discovery)?;
        let d: Discovery = serde_json::from_value(v)
            .map_err(|_| OidcError::Discovery("invalid metadata".into()))?;
        if d.issuer.trim_end_matches('/') != self.settings.issuer {
            return Err(OidcError::Discovery("issuer mismatch".into()));
        }
        *self.discovery.lock().unwrap_or_else(|p| p.into_inner()) =
            Some((d.clone(), Instant::now()));
        Ok(d)
    }

    async fn jwks_keys(&self, force: bool) -> Result<Vec<Value>, OidcError> {
        if !force {
            if let Some((k, at)) = self.jwks.lock().unwrap_or_else(|p| p.into_inner()).clone() {
                if at.elapsed() < JWKS_TTL {
                    return Ok(k);
                }
            }
        }
        let d = self.discovery().await?;
        let v = self
            .get_json(&d.jwks_uri)
            .await
            .map_err(|e| OidcError::InvalidToken(format!("jwks fetch: {e}")))?;
        let keys = v
            .get("keys")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        *self.jwks.lock().unwrap_or_else(|p| p.into_inner()) = Some((keys.clone(), Instant::now()));
        Ok(keys)
    }

    fn find_jwk(keys: &[Value], kid: Option<&str>) -> Option<Jwk> {
        let usable = keys
            .iter()
            .filter(|k| k.get("use").and_then(Value::as_str) != Some("enc"));
        let candidates: Vec<&Value> = match kid {
            Some(kid) => usable
                .filter(|k| k.get("kid").and_then(Value::as_str) == Some(kid))
                .collect(),
            None => usable.collect(),
        };
        if kid.is_none() && candidates.len() != 1 {
            return None;
        }
        candidates
            .into_iter()
            .find_map(|k| serde_json::from_value::<Jwk>(k.clone()).ok())
    }

    pub async fn begin_login(
        &self,
        keys: &MasterKeys,
        next_path: Option<&str>,
    ) -> Result<(String, String), OidcError> {
        if !self.enabled() {
            return Err(OidcError::Disabled);
        }
        let d = self.discovery().await?;
        let tx = TxData {
            state: random_b64u(16),
            nonce: random_b64u(16),
            verifier: random_b64u(32),
            next: sanitize_next_path(next_path),
            ts: now_secs(),
        };
        let mut url = url::Url::parse(&d.authorization_endpoint)
            .map_err(|_| OidcError::Discovery("bad authorization_endpoint".into()))?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.settings.client_id)
            .append_pair("redirect_uri", &self.settings.redirect_uri)
            .append_pair("scope", &self.settings.scopes)
            .append_pair("state", &tx.state)
            .append_pair("nonce", &tx.nonce)
            .append_pair("code_challenge", &pkce_challenge(&tx.verifier))
            .append_pair("code_challenge_method", "S256");
        let cookie = seal_tx(keys, &tx)?;
        Ok((url.to_string(), cookie))
    }

    /// Complete OIDC Authorization Code + PKCE callback; returns `(OidcUser, next_path)`.
    pub async fn complete(
        &self,
        keys: &MasterKeys,
        tx_cookie: &str,
        code: &str,
        state: &str,
    ) -> Result<(OidcUser, String), OidcError> {
        if !self.enabled() {
            return Err(OidcError::Disabled);
        }
        let tx = open_tx(keys, tx_cookie, now_secs())?;
        if !crypto::ct_eq_str(&tx.state, state.trim()) {
            return Err(OidcError::StateMismatch);
        }
        if code.trim().is_empty() {
            return Err(OidcError::MissingCode);
        }
        let d = self.discovery().await?;
        let id_token = self.exchange_code(&d, code.trim(), &tx.verifier).await?;
        let user = self.verify_id_token(&id_token, &tx.nonce).await?;
        if !is_allowed(&self.settings, &user) {
            return Err(OidcError::NotAllowed);
        }
        Ok((user, sanitize_next_path(Some(&tx.next))))
    }

    async fn exchange_code(
        &self,
        d: &Discovery,
        code: &str,
        verifier: &str,
    ) -> Result<String, OidcError> {
        let form_body = {
            let mut form = url::form_urlencoded::Serializer::new(String::new());
            form.append_pair("grant_type", "authorization_code")
                .append_pair("code", code)
                .append_pair("redirect_uri", &self.settings.redirect_uri)
                .append_pair("client_id", &self.settings.client_id)
                .append_pair("code_verifier", verifier);
            if !self.settings.client_secret.is_empty() {
                form.append_pair("client_secret", &self.settings.client_secret);
            }
            form.finish()
        };
        let resp = self
            .http
            .post(&d.token_endpoint)
            .timeout(HTTP_TIMEOUT)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .header(reqwest::header::ACCEPT, "application/json")
            .body(form_body)
            .send()
            .await
            .map_err(|e| {
                OidcError::TokenExchange(if e.is_timeout() {
                    "timeout".into()
                } else {
                    "unreachable".into()
                })
            })?;
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            let err = body
                .get("error")
                .and_then(Value::as_str)
                .filter(|e| {
                    e.len() <= 64 && e.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                })
                .unwrap_or("error");
            return Err(OidcError::TokenExchange(format!(
                "http {} {err}",
                status.as_u16()
            )));
        }
        body.get("id_token")
            .and_then(Value::as_str)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .ok_or_else(|| OidcError::TokenExchange("no id_token in response".into()))
    }

    pub async fn verify_id_token(
        &self,
        id_token: &str,
        expected_nonce: &str,
    ) -> Result<OidcUser, OidcError> {
        let header = jsonwebtoken::decode_header(id_token)
            .map_err(|_| OidcError::InvalidToken("malformed".into()))?;
        if !allowed_alg(header.alg) {
            return Err(OidcError::InvalidToken(format!(
                "algorithm {:?} not allowed",
                header.alg
            )));
        }
        let kid = header.kid.as_deref();
        let mut jwk = Self::find_jwk(&self.jwks_keys(false).await?, kid);
        if jwk.is_none() {
            jwk = Self::find_jwk(&self.jwks_keys(true).await?, kid);
        }
        let jwk = jwk.ok_or_else(|| OidcError::InvalidToken("signing key not found".into()))?;
        let key = DecodingKey::from_jwk(&jwk)
            .map_err(|_| OidcError::InvalidToken("unusable signing key".into()))?;
        let mut v = Validation::new(header.alg);
        v.set_issuer(&[self.settings.issuer.as_str()]);
        v.set_audience(&[self.settings.client_id.as_str()]);
        v.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        v.leeway = JWT_LEEWAY_SECS;
        v.validate_exp = true;
        v.validate_nbf = true;
        let data = jsonwebtoken::decode::<IdClaims>(id_token, &key, &v)
            .map_err(|_| OidcError::InvalidToken("verification_failed".into()))?;
        let c = data.claims;
        match c.nonce.as_deref() {
            Some(n) if crypto::ct_eq_str(n, expected_nonce) => {}
            _ => return Err(OidcError::NonceMismatch),
        }
        if let Some(azp) = c.azp.as_deref() {
            if azp != self.settings.client_id {
                return Err(OidcError::InvalidToken("azp".into()));
            }
        }
        Ok(OidcUser {
            sub: c.sub,
            username: c.preferred_username.unwrap_or_default(),
            email: c.email.unwrap_or_default(),
            name: c.name.unwrap_or_default(),
            groups: c.groups,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX_KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    #[test]
    fn session_v2_ttl_and_epoch() {
        let k = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let t0 = 1_800_000_000;
        let c = new_session("oidc", "sub-1", Some(json!({"username": "louis"})), 1, t0);
        let tok = sign_session(&k, &c);
        assert!(verify_session(&k, &tok, 1, t0 + 100).is_some());
        assert!(verify_session(&k, &tok, 2, t0 + 100).is_none(), "epoch revokes");
        assert!(verify_session(&k, &tok, 1, t0 + IDLE_TTL_SECS + 1).is_none());
        let renewed = renew(c, t0 + MAX_TTL_SECS - 10);
        let tok2 = sign_session(&k, &renewed);
        assert!(verify_session(&k, &tok2, 1, t0 + MAX_TTL_SECS).is_some());
        assert!(verify_session(&k, &tok2, 1, t0 + MAX_TTL_SECS + 1).is_none());
    }

    #[test]
    fn login_limiter_enforces_8_per_5m() {
        let l = LoginLimiter::default();
        for _ in 0..8 {
            assert!(l.check("10.0.0.1").is_ok());
            l.fail("10.0.0.1");
        }
        assert!(l.check("10.0.0.1").is_err());
        assert!(l.check("10.0.0.2").is_ok());
        l.ok("10.0.0.1");
        assert!(l.check("10.0.0.1").is_ok());
    }

    #[test]
    fn sanitize_next_blocks_open_redirects() {
        assert_eq!(sanitize_next_path(Some("/keys")), "/keys");
        assert_eq!(sanitize_next_path(Some("//evil.com")), "/styles");
        assert_eq!(sanitize_next_path(Some("https://evil.com")), "/styles");
        assert_eq!(sanitize_next_path(Some("/login.html")), "/styles");
        assert_eq!(sanitize_next_path(None), "/styles");
    }
}
