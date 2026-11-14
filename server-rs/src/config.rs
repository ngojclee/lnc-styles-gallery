//! Runtime configuration, `config.json` v2 persistence, and `keys.json`
//! Zero-F12-Leak auto-migration & CRUD.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zeroize::Zeroizing;

use crate::crypto::{self, CryptoError, MasterKeys, SsrfError};

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn now_secs() -> i64 {
    now_ms() / 1000
}

/// Format current UTC timestamp as ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ`) using
/// Howard Hinnant's civil-from-days algorithm (zero external date crates).
pub fn now_iso8601() -> String {
    unix_secs_to_iso8601(now_secs())
}

pub fn unix_secs_to_iso8601(secs: i64) -> String {
    let s = secs.max(0);
    let days = s.div_euclid(86_400);
    let rem = s.rem_euclid(86_400);
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    format!("{year:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

// ---------------------------------------------------------------------------
// Environment Snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct EnvSnapshot {
    map: HashMap<String, String>,
}

impl EnvSnapshot {
    pub fn from_process() -> Self {
        let mut map = HashMap::new();
        for (k, v) in std::env::vars() {
            map.insert(k, v);
        }
        Self { map }
    }

    pub fn from_pairs<I, K, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut map = HashMap::new();
        for (k, v) in iter {
            map.insert(k.into(), v.into());
        }
        Self { map }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }

    pub fn first(&self, keys: &[&str]) -> Option<&str> {
        for k in keys {
            if let Some(v) = self.get(k) {
                let t = v.trim();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        None
    }

    pub fn bool_or(&self, key: &str, default: bool) -> bool {
        match self.get(key).map(|s| s.trim().to_ascii_lowercase()) {
            Some(v) if matches!(v.as_str(), "1" | "true" | "yes" | "on") => true,
            Some(v) if matches!(v.as_str(), "0" | "false" | "no" | "off") => false,
            _ => default,
        }
    }

    pub fn u16_first(&self, keys: &[&str], default: u16) -> u16 {
        for k in keys {
            if let Some(v) = self.get(k) {
                if let Ok(n) = v.trim().parse::<u16>() {
                    if n > 0 {
                        return n;
                    }
                }
            }
        }
        default
    }
}

// ---------------------------------------------------------------------------
// AppSettings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub service_name: String,
    pub app_title: String,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub public_url: String,
    pub root_dir: PathBuf,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub ui_dist_dir: PathBuf,
    pub ui_dir: PathBuf,
    pub ui_backup_dir: PathBuf,
    pub refero_override: Option<PathBuf>,
    pub prompt_ui_override: Option<PathBuf>,
    pub components_override: Option<PathBuf>,
    pub mcp_auth_required: bool,
    pub auto_warm_cache: bool,
    pub cookie_secure: bool,
    pub ssrf_block_private: bool,
    pub dev_open_admin: bool,
    pub allowed_hosts: String,
    pub session_ttl: crate::oidc::SessionTtlConfig,
    pub master_key_configured: bool,
    pub master_key_old_configured: bool,
    pub connector_audience: String,
    pub connector_max_scopes: Vec<String>,
    pub connector_delegation_configured: bool,
}

fn detect_root_dir(env: &EnvSnapshot) -> PathBuf {
    if let Some(r) = env.first(&["STYLES_ROOT_DIR"]) {
        return PathBuf::from(r);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("VERSION").exists() || cwd.join("ui").exists() || cwd.join("app").exists() {
        return cwd;
    }
    if let Some(parent) = cwd.parent() {
        if parent.join("VERSION").exists() || parent.join("ui").exists() {
            return parent.to_path_buf();
        }
    }
    let app_dir = PathBuf::from("/app");
    if app_dir.exists() {
        return app_dir;
    }
    cwd
}

fn read_version(root_dir: &Path, env: &EnvSnapshot) -> String {
    let candidates = [
        env.get("STYLES_VERSION_FILE").map(PathBuf::from),
        Some(root_dir.join("VERSION")),
        Some(PathBuf::from("/app/VERSION")),
    ];
    for cand in candidates.into_iter().flatten() {
        if let Ok(s) = std::fs::read_to_string(&cand) {
            let v = s.trim();
            if !v.is_empty() {
                return v.to_string();
            }
        }
    }
    "1.0.0.1".to_string()
}

impl AppSettings {
    pub fn from_env() -> Self {
        Self::from_map(&EnvSnapshot::from_process())
    }

    pub fn from_map(env: &EnvSnapshot) -> Self {
        let root_dir = detect_root_dir(env);
        let version = read_version(&root_dir, env);
        let host = env
            .first(&["STYLES_HOST", "HOST"])
            .unwrap_or("0.0.0.0")
            .to_string();
        let port = env.u16_first(&["PORT", "STYLES_PORT"], 8000);
        let ssrf_block_private = env.bool_or("SSRF_BLOCK_PRIVATE", false);

        let raw_pub = env
            .first(&["STYLES_PUBLIC_URL", "PUBLIC_URL", "PUBLIC_BASE_URL"])
            .unwrap_or("https://styles.lengoc.me");
        let public_url = match crypto::validate_outbound_url(raw_pub, ssrf_block_private) {
            Ok(u) => u.as_str().trim_end_matches('/').to_string(),
            Err(_) => "https://styles.lengoc.me".to_string(),
        };

        let data_dir = if let Some(d) = env.first(&["STYLES_DATA_DIR", "STYLES_DATA_ROOT"]) {
            PathBuf::from(d)
        } else if Path::new("/data").exists() {
            PathBuf::from("/data")
        } else if root_dir.join("data").exists() {
            root_dir.join("data")
        } else {
            PathBuf::from("/data")
        };

        let config_dir = if let Some(c) = env.first(&["STYLES_CONFIG_DIR"]) {
            PathBuf::from(c)
        } else if cfg!(unix) {
            PathBuf::from("/config")
        } else {
            root_dir.join("data-runtime")
        };

        let ui_dir = if let Some(u) = env.first(&["STYLES_UI_DIR"]) {
            PathBuf::from(u)
        } else if root_dir.join("ui").exists() {
            root_dir.join("ui")
        } else {
            PathBuf::from("/app/ui")
        };

        let ui_dist_dir = if let Some(d) = env.first(&["STYLES_UI_DIST_DIR"]) {
            PathBuf::from(d)
        } else {
            ui_dir.join("dist")
        };

        let ui_backup_dir = root_dir.join("ui-vanilla-backup");

        Self {
            service_name: "lnc-styles-gallery".to_string(),
            app_title: "LNC Style Library".to_string(),
            version,
            host,
            port,
            public_url,
            root_dir,
            data_dir,
            config_dir,
            ui_dist_dir,
            ui_dir,
            ui_backup_dir,
            refero_override: env.first(&["REFERO_STYLES_DIR"]).map(PathBuf::from),
            prompt_ui_override: env.first(&["PROMPT_UI_DIR"]).map(PathBuf::from),
            components_override: env.first(&["COMPONENTS_DIR"]).map(PathBuf::from),
            mcp_auth_required: env.bool_or("STYLES_MCP_AUTH", false),
            auto_warm_cache: env.bool_or("STYLES_AUTO_WARM_CACHE", false),
            cookie_secure: env.bool_or("COOKIE_SECURE", false),
            ssrf_block_private,
            dev_open_admin: env.bool_or("STYLES_DEV_OPEN_ADMIN", false),
            allowed_hosts: env.first(&["ALLOWED_HOSTS"]).unwrap_or("*").to_string(),
            session_ttl: crate::oidc::SessionTtlConfig::from_map(env),
            master_key_configured: env.first(&["LNC_MASTER_KEY"]).is_some(),
            master_key_old_configured: env.first(&["LNC_MASTER_KEY_OLD"]).is_some(),
            connector_audience: env
                .first(&["CONNECTOR_AUDIENCE"])
                .unwrap_or("styles-mcp")
                .to_string(),
            connector_max_scopes: env
                .first(&["CONNECTOR_MAX_SCOPES"])
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .filter(|v: &Vec<String>| !v.is_empty())
                .unwrap_or_else(|| {
                    vec![
                        "mcp:read".to_string(),
                        "styles:read".to_string(),
                        "templates:read".to_string(),
                        "components:read".to_string(),
                        "corpus:reindex".to_string(),
                    ]
                }),
            connector_delegation_configured: env
                .first(&["DELEGATION_SECRET", "CONNECTOR_DELEGATION_SECRET"])
                .is_some(),
        }
    }

    pub fn resolve_refero_dir(&self) -> PathBuf {
        if let Some(ref p) = self.refero_override {
            return p.clone();
        }
        self.data_dir.join("refero-styles")
    }

    pub fn resolve_prompt_ui_dir(&self) -> PathBuf {
        if let Some(ref p) = self.prompt_ui_override {
            return p.clone();
        }
        self.data_dir.join("prompt-driven-ui")
    }

    pub fn resolve_components_dir(&self) -> PathBuf {
        if let Some(ref p) = self.components_override {
            return p.clone();
        }
        self.data_dir.join("lnc-components")
    }
}

// ---------------------------------------------------------------------------
// Persisted Config (`config.json` v2) & Hardened Keys (`keys.json`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedConfig {
    #[serde(default = "default_cfg_version")]
    pub version: u32,
    #[serde(default = "default_epoch")]
    pub session_epoch: i64,
    #[serde(default)]
    pub mgmt_key_hash: String,
    #[serde(default)]
    pub mgmt_key_hmac: String,
    #[serde(default)]
    pub mgmt_key_masked: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oidc_client_secret_enc: Option<Value>,
}

fn default_cfg_version() -> u32 {
    2
}
fn default_epoch() -> i64 {
    1
}

impl Default for PersistedConfig {
    fn default() -> Self {
        Self {
            version: 2,
            session_epoch: 1,
            mgmt_key_hash: String::new(),
            mgmt_key_hmac: String::new(),
            mgmt_key_masked: String::new(),
            public_url: None,
            oidc_client_secret_enc: None,
        }
    }
}

/// On-disk record inside `keys.json` — ZERO plaintext `key` or `raw_key`.
/// Encrypted clipboard envelopes live in sibling `vault.json` (`$enc:1` AES-256-GCM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredApiKey {
    pub id: String,
    pub label: String,
    pub key_hash: String,
    pub prefix: String,
    #[serde(default)]
    pub last4: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_key_scopes")]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub created: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

fn default_enabled() -> bool {
    true
}

fn default_source() -> String {
    "runtime".to_string()
}

fn default_key_scopes() -> Vec<String> {
    vec![
        "mcp:read".to_string(),
        "styles:read".to_string(),
        "templates:read".to_string(),
        "components:read".to_string(),
    ]
}

fn default_vault_version() -> u32 {
    1
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeysFile {
    #[serde(default = "default_cfg_version")]
    pub version: u32,
    #[serde(default)]
    pub api_keys: Vec<StoredApiKey>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VaultFile {
    #[serde(default = "default_vault_version")]
    pub version: u32,
    #[serde(default)]
    pub api_keys: std::collections::BTreeMap<String, Value>,
    #[serde(default)]
    pub mcp_connectors: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMcpConnector {
    pub id: String,
    pub name: String,
    #[serde(default = "default_connector_direction")]
    pub direction: String,
    #[serde(default = "default_connector_transport")]
    pub transport: String,
    #[serde(default)]
    pub audience: String,
    #[serde(default = "default_key_scopes")]
    pub scopes: Vec<String>,
    pub token_hash: String,
    pub prefix: String,
    #[serde(default)]
    pub last4: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub created: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

fn default_connector_direction() -> String {
    "inbound".to_string()
}

fn default_connector_transport() -> String {
    "streamable_http".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectorsFile {
    #[serde(default = "default_vault_version")]
    pub version: u32,
    #[serde(default)]
    pub connectors: Vec<StoredMcpConnector>,
}

/// Redacted public view returned by `GET /api/keys` / `GET /api/admin/api-keys`.
/// Never contains `raw_key`, `key`, `key_hash`, or ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKeyPublicView {
    pub id: String,
    pub label: String,
    pub prefix: String,
    pub last4: String,
    pub masked: String,
    pub enabled: bool,
    pub status: String,
    pub source: String,
    pub scopes: Vec<String>,
    pub copy_available: bool,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpConnectorPublicView {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub transport: String,
    pub endpoint_url: String,
    pub audience: String,
    pub scopes: Vec<String>,
    pub prefix: String,
    pub last4: String,
    pub masked: String,
    pub enabled: bool,
    pub status: String,
    pub source: String,
    pub copy_available: bool,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopySecretOutcome {
    Secret(Zeroizing<String>),
    LegacyHashOnly,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectorMutationError {
    NotFound,
    EnvManaged,
}

impl StoredApiKey {
    pub fn to_public_view(&self, copy_available: bool) -> ApiKeyPublicView {
        ApiKeyPublicView {
            id: self.id.clone(),
            label: self.label.clone(),
            prefix: self.prefix.clone(),
            last4: self.last4.clone(),
            masked: crypto::format_masked_key(&self.prefix, &self.last4),
            enabled: self.enabled,
            status: if self.enabled {
                "active".to_string()
            } else {
                "revoked".to_string()
            },
            source: if self.source.is_empty() {
                "runtime".to_string()
            } else {
                self.source.clone()
            },
            scopes: if self.scopes.is_empty() {
                default_key_scopes()
            } else {
                self.scopes.clone()
            },
            copy_available,
            created: self.created.clone(),
            rotated_at: self.rotated_at.clone(),
            revoked_at: self.revoked_at.clone(),
        }
    }
}

impl StoredMcpConnector {
    pub fn to_public_view(&self, endpoint_url: &str, copy_available: bool) -> McpConnectorPublicView {
        McpConnectorPublicView {
            id: self.id.clone(),
            name: self.name.clone(),
            direction: if self.direction.is_empty() {
                "inbound".to_string()
            } else {
                self.direction.clone()
            },
            transport: if self.transport.is_empty() {
                "streamable_http".to_string()
            } else {
                self.transport.clone()
            },
            endpoint_url: endpoint_url.to_string(),
            audience: if self.audience.is_empty() {
                "styles-mcp".to_string()
            } else {
                self.audience.clone()
            },
            scopes: if self.scopes.is_empty() {
                default_key_scopes()
            } else {
                self.scopes.clone()
            },
            prefix: self.prefix.clone(),
            last4: self.last4.clone(),
            masked: crypto::format_masked_key(&self.prefix, &self.last4),
            enabled: self.enabled,
            status: if self.enabled {
                "active".to_string()
            } else {
                "revoked".to_string()
            },
            source: if self.source.is_empty() {
                "runtime".to_string()
            } else {
                self.source.clone()
            },
            copy_available,
            created: self.created.clone(),
            rotated_at: self.rotated_at.clone(),
            revoked_at: self.revoked_at.clone(),
        }
    }
}

#[derive(Debug)]
struct StoreInner {
    cfg: PersistedConfig,
    keys: Vec<StoredApiKey>,
    connectors: Vec<StoredMcpConnector>,
    vault: VaultFile,
}

/// Thread-safe manager for `config.json`, `keys.json`, `connectors.json`, `vault.json`, and `audit.jsonl`.
pub struct RuntimeStore {
    config_dir: PathBuf,
    master: MasterKeys,
    rewrapped_from_old_key: bool,
    copy_limiter: crate::oidc::LoginLimiter,
    inner: RwLock<StoreInner>,
}

impl std::fmt::Debug for RuntimeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeStore")
            .field("config_dir", &self.config_dir)
            .finish()
    }
}

impl RuntimeStore {
    pub fn load_or_init(
        config_dir: &Path,
        master: MasterKeys,
        env: &EnvSnapshot,
    ) -> Result<Self, CryptoError> {
        std::fs::create_dir_all(config_dir).map_err(|e| CryptoError::Io(e.to_string()))?;
        let old_master = MasterKeys::load_old_from_str(env.first(&["LNC_MASTER_KEY_OLD"]))?;
        let config_path = config_dir.join("config.json");
        let keys_path = config_dir.join("keys.json");
        let connectors_path = config_dir.join("connectors.json");
        let vault_path = config_dir.join("vault.json");

        let (mut cfg, mut cfg_dirty, mut rewrapped) =
            load_and_migrate_config(&config_path, &master, old_master.as_ref())?;

        // Sync management credential from env if configured.
        if let Some(raw_mgmt) = env.first(&[
            "MANAGEMENT_KEY",
            "MANAGEMENT_TOKEN",
            "STYLES_API_KEY",
            "ADMIN_API_KEY",
        ]) {
            let raw_z = Zeroizing::new(raw_mgmt.to_string());
            let env_hmac = master.api_key_hash(&raw_z);
            if cfg.mgmt_key_hmac != env_hmac || cfg.mgmt_key_hash.is_empty() {
                cfg.mgmt_key_hash = crypto::hash_password(&raw_z)?;
                cfg.mgmt_key_hmac = env_hmac;
                cfg.mgmt_key_masked = crypto::mask_admin_key(&raw_z);
                cfg_dirty = true;
            }
        }

        // Seal OIDC client secret in config.json if present so `$enc:1` vault KEK is active.
        if let Some(oidc_sec) = env.first(&["OIDC_CLIENT_SECRET"]) {
            let needs_seal = match &cfg.oidc_client_secret_enc {
                Some(v) => master
                    .open("config.oidc_client_secret", v)
                    .map(|pt| pt != oidc_sec)
                    .unwrap_or(true),
                None => true,
            };
            if needs_seal {
                cfg.oidc_client_secret_enc =
                    Some(master.seal("config.oidc_client_secret", oidc_sec)?);
                cfg_dirty = true;
            }
        }

        if cfg.session_epoch < 1 {
            cfg.session_epoch = 1;
            cfg_dirty = true;
        }

        if cfg_dirty || !config_path.exists() {
            save_config_file(&config_path, &cfg)?;
        }

        let (mut vault, mut vault_dirty, vault_rewrapped) =
            load_vault_file(&vault_path, &master, old_master.as_ref())?;
        if vault_rewrapped {
            rewrapped = true;
        }

        let (mut keys, mut keys_dirty) =
            load_and_migrate_keys(&keys_path, &master, &mut vault, &mut vault_dirty)?;

        // Optional seed connector token from env (`STYLES_HUB_API_KEY` / `MCP_CONNECTOR_TOKEN`).
        if let Some(seed_tok) = env.first(&["STYLES_HUB_API_KEY", "MCP_CONNECTOR_TOKEN"]) {
            let h = master.api_key_hash(seed_tok);
            let exists = keys.iter().any(|k| crypto::ct_eq_str(&k.key_hash, &h));
            if !exists {
                let (prefix, last4) = crypto::extract_prefix_and_last4(seed_tok);
                let id = "hub_default".to_string();
                let env_box = master.seal(&format!("api_keys.{id}.secret"), seed_tok)?;
                vault.api_keys.insert(id.clone(), env_box);
                vault_dirty = true;
                keys.push(StoredApiKey {
                    id,
                    label: "Business MCP Hub".to_string(),
                    key_hash: h,
                    prefix,
                    last4,
                    enabled: true,
                    source: "env".to_string(),
                    scopes: default_key_scopes(),
                    created: now_iso8601(),
                    rotated_at: None,
                    revoked_at: None,
                });
                keys_dirty = true;
            }
        }

        // Load or bootstrap inbound MCP connectors (`connectors.json`).
        let (mut connectors, mut connectors_dirty) =
            load_and_migrate_connectors(&connectors_path, &master, &mut vault, &mut vault_dirty)?;

        let default_aud = env
            .first(&["CONNECTOR_AUDIENCE"])
            .unwrap_or("styles-mcp")
            .to_string();
        let default_scopes: Vec<String> = env
            .first(&["CONNECTOR_MAX_SCOPES"])
            .map(|s| {
                s.split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(default_key_scopes);

        if let Some(env_bearer) = env.first(&["CONNECTOR_BEARER"]) {
            let h = master.api_key_hash(env_bearer);
            let exists = connectors
                .iter()
                .any(|c| c.id == "mcp_env_bearer" && crypto::ct_eq_str(&c.token_hash, &h));
            if !exists {
                let (prefix, last4) = crypto::extract_prefix_and_last4(env_bearer);
                let id = "mcp_env_bearer".to_string();
                let env_box = master.seal(&format!("mcp_connectors.{id}.secret"), env_bearer)?;
                vault.mcp_connectors.insert(id.clone(), env_box);
                vault_dirty = true;
                connectors.retain(|c| c.id != "mcp_env_bearer");
                connectors.insert(
                    0,
                    StoredMcpConnector {
                        id,
                        name: "Inbound MCP Connector (Env)".to_string(),
                        direction: "inbound".to_string(),
                        transport: "streamable_http".to_string(),
                        audience: default_aud.clone(),
                        scopes: default_scopes.clone(),
                        token_hash: h,
                        prefix,
                        last4,
                        enabled: true,
                        source: "env".to_string(),
                        created: now_iso8601(),
                        rotated_at: None,
                        revoked_at: None,
                    },
                );
                connectors_dirty = true;
            }
        } else if !connectors_path.exists() {
            let raw = Zeroizing::new(crypto::generate_api_token("styles_mcp"));
            let h = master.api_key_hash(&raw);
            let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
            let id = "mcp_default".to_string();
            let env_box = master.seal(&format!("mcp_connectors.{id}.secret"), &raw)?;
            vault.mcp_connectors.insert(id.clone(), env_box);
            vault_dirty = true;
            connectors.push(StoredMcpConnector {
                id,
                name: "LNC Inbound MCP Connector".to_string(),
                direction: "inbound".to_string(),
                transport: "streamable_http".to_string(),
                audience: default_aud,
                scopes: default_scopes,
                token_hash: h,
                prefix,
                last4,
                enabled: true,
                source: "runtime".to_string(),
                created: now_iso8601(),
                rotated_at: None,
                revoked_at: None,
            });
            connectors_dirty = true;
        }

        if keys_dirty {
            save_keys_file(&keys_path, &keys)?;
        }
        if connectors_dirty || !connectors_path.exists() {
            save_connectors_file(&connectors_path, &connectors)?;
        }
        if vault_dirty {
            save_vault_file(&vault_path, &vault)?;
        }

        Ok(Self {
            config_dir: config_dir.to_path_buf(),
            master,
            rewrapped_from_old_key: rewrapped,
            copy_limiter: crate::oidc::LoginLimiter::new(15, std::time::Duration::from_secs(300)),
            inner: RwLock::new(StoreInner {
                cfg,
                keys,
                connectors,
                vault,
            }),
        })
    }

    pub fn rewrapped_from_old_key(&self) -> bool {
        self.rewrapped_from_old_key
    }

    pub fn check_copy_rate_limit(&self, ip: &str) -> Result<(), u64> {
        self.copy_limiter.check_and_hit(ip)
    }

    pub fn record_audit_event(
        &self,
        actor: &str,
        action: &str,
        kind: &str,
        record_id: &str,
        ip: &str,
    ) {
        let entry = json!({
            "ts": now_iso8601(),
            "actor": actor,
            "action": action,
            "kind": kind,
            "record_id": record_id,
            "ip": ip,
        });
        let _ = crypto::append_private_jsonl_line(
            &self.config_dir.join("audit.jsonl"),
            &entry.to_string(),
        );
    }

    pub fn recent_audit_events(&self, max_items: usize) -> Vec<Value> {
        let path = self.config_dir.join("audit.jsonl");
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        let mut items: Vec<Value> = raw
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok())
            .collect();
        if items.len() > max_items {
            items.drain(..items.len() - max_items);
        }
        items.reverse();
        items
    }

    pub fn master(&self) -> &MasterKeys {
        &self.master
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn session_epoch(&self) -> i64 {
        self.inner
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .cfg
            .session_epoch
    }

    pub fn has_mgmt_key(&self) -> bool {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        !g.cfg.mgmt_key_hmac.is_empty() || !g.cfg.mgmt_key_hash.is_empty()
    }

    pub fn mgmt_key_masked(&self) -> String {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        if g.cfg.mgmt_key_masked.is_empty() {
            "***".to_string()
        } else {
            g.cfg.mgmt_key_masked.clone()
        }
    }

    pub fn effective_public_url(&self, fallback: &str) -> String {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.cfg
            .public_url
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| fallback.to_string())
    }

    /// Update runtime settings (e.g. `public_url`), enforcing SSRF protection on URL fields.
    pub fn update_public_url(
        &self,
        new_url: &str,
        block_private: bool,
    ) -> Result<String, SsrfError> {
        let parsed = crypto::validate_outbound_url(new_url, block_private)?;
        let clean = parsed.as_str().trim_end_matches('/').to_string();
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.cfg.public_url = Some(clean.clone());
        let _ = save_config_file(&self.config_dir.join("config.json"), &g.cfg);
        Ok(clean)
    }

    /// Increment `session_epoch` and persist atomically, revoking all issued session cookies.
    pub fn bump_session_epoch(&self) -> Result<i64, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.cfg.session_epoch = g.cfg.session_epoch.saturating_add(1).max(2);
        save_config_file(&self.config_dir.join("config.json"), &g.cfg)?;
        Ok(g.cfg.session_epoch)
    }

    /// Verify management key for break-glass interactive login (`Argon2id` + constant-time `HMAC`).
    pub fn verify_mgmt_login(&self, candidate: &str) -> bool {
        let cand = candidate.trim();
        if cand.is_empty() {
            return false;
        }
        let (phc, hmac_hex) = {
            let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
            (g.cfg.mgmt_key_hash.clone(), g.cfg.mgmt_key_hmac.clone())
        };
        if phc.is_empty() && hmac_hex.is_empty() {
            return false;
        }
        let hmac_ok = !hmac_hex.is_empty() && self.master.api_key_matches(cand, &hmac_hex);
        let argon_ok = !phc.is_empty() && crypto::verify_password(cand, &phc);
        if !phc.is_empty() && !hmac_hex.is_empty() {
            hmac_ok && argon_ok
        } else {
            hmac_ok || argon_ok
        }
    }

    /// Fast constant-time verification for `X-API-Key` / `Bearer` admin header.
    pub fn verify_mgmt_header(&self, candidate: &str) -> bool {
        let cand = candidate.trim();
        if cand.is_empty() {
            return false;
        }
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        if !g.cfg.mgmt_key_hmac.is_empty() {
            return self.master.api_key_matches(cand, &g.cfg.mgmt_key_hmac);
        }
        if !g.cfg.mgmt_key_hash.is_empty() {
            return crypto::verify_password(cand, &g.cfg.mgmt_key_hash);
        }
        false
    }

    /// Constant-time check of an MCP Bearer token against enabled `keys.json`, active inbound
    /// `connectors.json` entries, OR admin key.
    pub fn validate_api_key(&self, token: &str) -> bool {
        let tok = token.trim();
        if tok.is_empty() {
            return false;
        }
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        let mut matched = false;
        if !g.cfg.mgmt_key_hmac.is_empty() && self.master.api_key_matches(tok, &g.cfg.mgmt_key_hmac)
        {
            matched = true;
        }
        for entry in &g.keys {
            if entry.enabled && self.master.api_key_matches(tok, &entry.key_hash) {
                matched = true;
            }
        }
        for conn in &g.connectors {
            if conn.enabled && self.master.api_key_matches(tok, &conn.token_hash) {
                matched = true;
            }
        }
        matched
    }

    pub fn keys_count(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .keys
            .len()
    }

    pub fn connectors_count(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .connectors
            .len()
    }

    pub fn list_keys(&self) -> Vec<ApiKeyPublicView> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.keys
            .iter()
            .map(|k| k.to_public_view(g.vault.api_keys.contains_key(&k.id)))
            .collect()
    }

    pub fn create_key(
        &self,
        label: &str,
        custom_secret: Option<&str>,
    ) -> Result<(Zeroizing<String>, ApiKeyPublicView), CryptoError> {
        self.create_key_with_scopes(label, custom_secret, None)
    }

    pub fn create_key_with_scopes(
        &self,
        label: &str,
        custom_secret: Option<&str>,
        scopes: Option<Vec<String>>,
    ) -> Result<(Zeroizing<String>, ApiKeyPublicView), CryptoError> {
        let clean_label = {
            let t = label.trim();
            if t.is_empty() {
                "Unnamed API Key".to_string()
            } else {
                t.chars().take(120).collect()
            }
        };
        let clean_scopes = scopes
            .map(|s| {
                s.into_iter()
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(default_key_scopes);
        let raw = Zeroizing::new(match custom_secret.map(str::trim).filter(|s| !s.is_empty()) {
            Some(c) => c.to_string(),
            None => crypto::generate_api_token("styles"),
        });
        let id = crypto::random_hex(6);
        let key_hash = self.master.api_key_hash(&raw);
        let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
        let enc_box = self.master.seal(&format!("api_keys.{id}.secret"), &raw)?;
        let stored = StoredApiKey {
            id: id.clone(),
            label: clean_label,
            key_hash,
            prefix,
            last4,
            enabled: true,
            source: "runtime".to_string(),
            scopes: clean_scopes,
            created: now_iso8601(),
            rotated_at: None,
            revoked_at: None,
        };
        let view = stored.to_public_view(true);
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.keys.push(stored);
        g.vault.api_keys.insert(id, enc_box);
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
        Ok((raw, view))
    }

    pub fn update_key(
        &self,
        key_id: &str,
        label: Option<&str>,
        scopes: Option<Vec<String>>,
    ) -> Result<Option<ApiKeyPublicView>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.keys.iter().position(|k| k.id == key_id) else {
            return Ok(None);
        };
        if let Some(new_label) = label.map(str::trim).filter(|s| !s.is_empty()) {
            g.keys[idx].label = new_label.chars().take(120).collect();
        }
        if let Some(new_scopes) = scopes {
            let clean: Vec<String> = new_scopes
                .into_iter()
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect();
            if !clean.is_empty() {
                g.keys[idx].scopes = clean;
            }
        }
        let copy_ok = g.vault.api_keys.contains_key(key_id);
        let view = g.keys[idx].to_public_view(copy_ok);
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        Ok(Some(view))
    }

    pub fn copy_key(&self, key_id: &str) -> Result<CopySecretOutcome, CryptoError> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        let Some(_entry) = g.keys.iter().find(|k| k.id == key_id) else {
            return Ok(CopySecretOutcome::NotFound);
        };
        let Some(env_box) = g.vault.api_keys.get(key_id) else {
            return Ok(CopySecretOutcome::LegacyHashOnly);
        };
        let secret = self
            .master
            .open(&format!("api_keys.{key_id}.secret"), env_box)?;
        Ok(CopySecretOutcome::Secret(Zeroizing::new(secret)))
    }

    pub fn rotate_key(
        &self,
        key_id: &str,
        custom_secret: Option<&str>,
    ) -> Result<Option<(Zeroizing<String>, ApiKeyPublicView)>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.keys.iter().position(|k| k.id == key_id) else {
            return Ok(None);
        };
        let raw = Zeroizing::new(match custom_secret.map(str::trim).filter(|s| !s.is_empty()) {
            Some(c) => c.to_string(),
            None => crypto::generate_api_token("styles"),
        });
        let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
        let key_hash = self.master.api_key_hash(&raw);
        let enc_box = self
            .master
            .seal(&format!("api_keys.{key_id}.secret"), &raw)?;
        g.keys[idx].key_hash = key_hash;
        g.keys[idx].prefix = prefix;
        g.keys[idx].last4 = last4;
        g.keys[idx].enabled = true;
        g.keys[idx].source = "runtime".to_string();
        g.keys[idx].rotated_at = Some(now_iso8601());
        g.keys[idx].revoked_at = None;
        let view = g.keys[idx].to_public_view(true);
        g.vault.api_keys.insert(key_id.to_string(), enc_box);
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
        Ok(Some((raw, view)))
    }

    pub fn revoke_key(
        &self,
        key_id: &str,
        revoke: bool,
    ) -> Result<Option<ApiKeyPublicView>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.keys.iter().position(|k| k.id == key_id) else {
            return Ok(None);
        };
        g.keys[idx].enabled = !revoke;
        g.keys[idx].revoked_at = if revoke { Some(now_iso8601()) } else { None };
        let copy_ok = g.vault.api_keys.contains_key(key_id);
        let view = g.keys[idx].to_public_view(copy_ok);
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        Ok(Some(view))
    }

    pub fn delete_key(&self, key_id: &str) -> Result<bool, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let before = g.keys.len();
        g.keys.retain(|k| k.id != key_id);
        let removed_vault = g.vault.api_keys.remove(key_id).is_some();
        if g.keys.len() < before || removed_vault {
            save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
            if removed_vault {
                save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
            }
            return Ok(true);
        }
        Ok(false)
    }

    // --- MCP Connectors CRUD + Copy · Rotate · Revoke · Remove ---

    pub fn list_connectors(&self, endpoint_url: &str) -> Vec<McpConnectorPublicView> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.connectors
            .iter()
            .map(|c| {
                c.to_public_view(
                    endpoint_url,
                    g.vault.mcp_connectors.contains_key(&c.id),
                )
            })
            .collect()
    }

    pub fn create_connector(
        &self,
        name: &str,
        audience: &str,
        scopes: Vec<String>,
        custom_secret: Option<&str>,
        endpoint_url: &str,
    ) -> Result<(Zeroizing<String>, McpConnectorPublicView), CryptoError> {
        let clean_name = {
            let t = name.trim();
            if t.is_empty() {
                "Inbound MCP Connector".to_string()
            } else {
                t.chars().take(120).collect()
            }
        };
        let clean_aud = {
            let a = audience.trim();
            if a.is_empty() {
                "styles-mcp".to_string()
            } else {
                a.chars().take(120).collect()
            }
        };
        let clean_scopes: Vec<String> = scopes
            .into_iter()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect();
        let clean_scopes = if clean_scopes.is_empty() {
            default_key_scopes()
        } else {
            clean_scopes
        };
        let raw = Zeroizing::new(
            match custom_secret
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != crypto::MASK)
            {
                Some(c) => c.to_string(),
                None => crypto::generate_api_token("styles_mcp"),
            },
        );
        let id = format!("mcp_{}", crypto::random_hex(5));
        let token_hash = self.master.api_key_hash(&raw);
        let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
        let enc_box = self
            .master
            .seal(&format!("mcp_connectors.{id}.secret"), &raw)?;
        let stored = StoredMcpConnector {
            id: id.clone(),
            name: clean_name,
            direction: "inbound".to_string(),
            transport: "streamable_http".to_string(),
            audience: clean_aud,
            scopes: clean_scopes,
            token_hash,
            prefix,
            last4,
            enabled: true,
            source: "runtime".to_string(),
            created: now_iso8601(),
            rotated_at: None,
            revoked_at: None,
        };
        let view = stored.to_public_view(endpoint_url, true);
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.connectors.push(stored);
        g.vault.mcp_connectors.insert(id, enc_box);
        save_connectors_file(&self.config_dir.join("connectors.json"), &g.connectors)?;
        save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
        Ok((raw, view))
    }

    pub fn update_connector(
        &self,
        id: &str,
        name: Option<&str>,
        audience: Option<&str>,
        scopes: Option<Vec<String>>,
        custom_secret: Option<&str>,
        endpoint_url: &str,
    ) -> Result<Result<McpConnectorPublicView, ConnectorMutationError>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.connectors.iter().position(|c| c.id == id) else {
            return Ok(Err(ConnectorMutationError::NotFound));
        };
        if g.connectors[idx].source == "env" {
            return Ok(Err(ConnectorMutationError::EnvManaged));
        }
        if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
            g.connectors[idx].name = n.chars().take(120).collect();
        }
        if let Some(a) = audience.map(str::trim).filter(|s| !s.is_empty()) {
            g.connectors[idx].audience = a.chars().take(120).collect();
        }
        if let Some(sc) = scopes {
            let clean: Vec<String> = sc
                .into_iter()
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect();
            if !clean.is_empty() {
                g.connectors[idx].scopes = clean;
            }
        }
        // `omitBlankSecrets`: ignore blank or `"********"`
        let mut vault_dirty = false;
        if let Some(sec) = custom_secret
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != crypto::MASK)
        {
            let (prefix, last4) = crypto::extract_prefix_and_last4(sec);
            let token_hash = self.master.api_key_hash(sec);
            let enc_box = self
                .master
                .seal(&format!("mcp_connectors.{id}.secret"), sec)?;
            g.connectors[idx].token_hash = token_hash;
            g.connectors[idx].prefix = prefix;
            g.connectors[idx].last4 = last4;
            g.connectors[idx].rotated_at = Some(now_iso8601());
            g.vault.mcp_connectors.insert(id.to_string(), enc_box);
            vault_dirty = true;
        }
        let copy_ok = g.vault.mcp_connectors.contains_key(id);
        let view = g.connectors[idx].to_public_view(endpoint_url, copy_ok);
        save_connectors_file(&self.config_dir.join("connectors.json"), &g.connectors)?;
        if vault_dirty {
            save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
        }
        Ok(Ok(view))
    }

    pub fn copy_connector(&self, id: &str) -> Result<CopySecretOutcome, CryptoError> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        let Some(_conn) = g.connectors.iter().find(|c| c.id == id) else {
            return Ok(CopySecretOutcome::NotFound);
        };
        let Some(env_box) = g.vault.mcp_connectors.get(id) else {
            return Ok(CopySecretOutcome::LegacyHashOnly);
        };
        let secret = self
            .master
            .open(&format!("mcp_connectors.{id}.secret"), env_box)?;
        Ok(CopySecretOutcome::Secret(Zeroizing::new(secret)))
    }

    pub fn rotate_connector(
        &self,
        id: &str,
        custom_secret: Option<&str>,
        endpoint_url: &str,
    ) -> Result<Result<(Zeroizing<String>, McpConnectorPublicView), ConnectorMutationError>, CryptoError>
    {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.connectors.iter().position(|c| c.id == id) else {
            return Ok(Err(ConnectorMutationError::NotFound));
        };
        if g.connectors[idx].source == "env" {
            return Ok(Err(ConnectorMutationError::EnvManaged));
        }
        let raw = Zeroizing::new(
            match custom_secret
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != crypto::MASK)
            {
                Some(c) => c.to_string(),
                None => crypto::generate_api_token("styles_mcp"),
            },
        );
        let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
        let token_hash = self.master.api_key_hash(&raw);
        let enc_box = self
            .master
            .seal(&format!("mcp_connectors.{id}.secret"), &raw)?;
        g.connectors[idx].token_hash = token_hash;
        g.connectors[idx].prefix = prefix;
        g.connectors[idx].last4 = last4;
        g.connectors[idx].enabled = true;
        g.connectors[idx].rotated_at = Some(now_iso8601());
        g.connectors[idx].revoked_at = None;
        let view = g.connectors[idx].to_public_view(endpoint_url, true);
        g.vault.mcp_connectors.insert(id.to_string(), enc_box);
        save_connectors_file(&self.config_dir.join("connectors.json"), &g.connectors)?;
        save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
        Ok(Ok((raw, view)))
    }

    pub fn revoke_connector(
        &self,
        id: &str,
        revoke: bool,
        endpoint_url: &str,
    ) -> Result<Result<McpConnectorPublicView, ConnectorMutationError>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.connectors.iter().position(|c| c.id == id) else {
            return Ok(Err(ConnectorMutationError::NotFound));
        };
        if g.connectors[idx].source == "env" {
            return Ok(Err(ConnectorMutationError::EnvManaged));
        }
        g.connectors[idx].enabled = !revoke;
        g.connectors[idx].revoked_at = if revoke { Some(now_iso8601()) } else { None };
        let copy_ok = g.vault.mcp_connectors.contains_key(id);
        let view = g.connectors[idx].to_public_view(endpoint_url, copy_ok);
        save_connectors_file(&self.config_dir.join("connectors.json"), &g.connectors)?;
        Ok(Ok(view))
    }

    pub fn delete_connector(&self, id: &str) -> Result<Result<(), ConnectorMutationError>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(idx) = g.connectors.iter().position(|c| c.id == id) else {
            return Ok(Err(ConnectorMutationError::NotFound));
        };
        if g.connectors[idx].source == "env" {
            return Ok(Err(ConnectorMutationError::EnvManaged));
        }
        g.connectors.remove(idx);
        let removed_vault = g.vault.mcp_connectors.remove(id).is_some();
        save_connectors_file(&self.config_dir.join("connectors.json"), &g.connectors)?;
        if removed_vault {
            save_vault_file(&self.config_dir.join("vault.json"), &g.vault)?;
        }
        Ok(Ok(()))
    }

    pub fn connector_status(&self, id: &str) -> Option<(bool, Vec<String>, String)> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.connectors
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.enabled, c.scopes.clone(), c.audience.clone()))
    }

    pub fn masked_settings_dict(&self, settings: &AppSettings, oidc_enabled: bool) -> Value {
        let refero_dir = settings.resolve_refero_dir();
        let prompt_ui_dir = settings.resolve_prompt_ui_dir();
        let components_dir = settings.resolve_components_dir();
        let api_key_configured = self.has_mgmt_key();
        let auth_mode = if oidc_enabled && api_key_configured {
            "oidc+api_key"
        } else if oidc_enabled {
            "oidc"
        } else if api_key_configured {
            "api_key"
        } else {
            "internal_open"
        };
        json!({
            "service_name": settings.service_name,
            "app_title": settings.app_title,
            "version": settings.version,
            "port": settings.port,
            "public_url": self.effective_public_url(&settings.public_url),
            "data_dir": settings.data_dir.to_string_lossy(),
            "refero_styles_dir": refero_dir.to_string_lossy(),
            "prompt_ui_dir": prompt_ui_dir.to_string_lossy(),
            "components_dir": components_dir.to_string_lossy(),
            "config_dir": settings.config_dir.to_string_lossy(),
            "mcp_auth_required": settings.mcp_auth_required,
            "auth_mode": auth_mode,
            "oidc_enabled": oidc_enabled,
            "api_key_configured": api_key_configured,
            "api_key_masked": self.mgmt_key_masked(),
            "session_epoch": self.session_epoch(),
            "session_idle_ttl_seconds": settings.session_ttl.idle_ttl_secs,
            "session_max_ttl_seconds": settings.session_ttl.max_ttl_secs,
            "cookie_secure": settings.cookie_secure,
            "master_key_configured": settings.master_key_configured,
            "master_key_old_configured": settings.master_key_old_configured,
            "rewrapped_from_old_key": self.rewrapped_from_old_key,
            "connector_audience": settings.connector_audience,
            "connector_max_scopes": settings.connector_max_scopes,
            "connector_delegation_configured": settings.connector_delegation_configured,
            "auto_warm_cache": settings.auto_warm_cache,
        })
    }
}

fn load_and_migrate_config(
    path: &Path,
    master: &MasterKeys,
    old_master: Option<&MasterKeys>,
) -> Result<(PersistedConfig, bool, bool), CryptoError> {
    if !path.exists() {
        return Ok((PersistedConfig::default(), false, false));
    }
    let raw_text = std::fs::read_to_string(path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let val: Value = match serde_json::from_str(&raw_text) {
        Ok(v) => v,
        Err(_) => return Ok((PersistedConfig::default(), true, false)),
    };
    let mut cfg: PersistedConfig = serde_json::from_value(val.clone()).unwrap_or_default();
    let mut dirty = false;
    let mut rewrapped = false;

    if let Some(obj) = val.as_object() {
        // Scrub any legacy plaintext mgmt_key / api_key stored on disk.
        for legacy_field in ["mgmt_key", "api_key", "management_key"] {
            if let Some(plain) = obj
                .get(legacy_field)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                cfg.mgmt_key_hash = crypto::hash_password(plain)?;
                cfg.mgmt_key_hmac = master.api_key_hash(plain);
                cfg.mgmt_key_masked = crypto::mask_admin_key(plain);
                dirty = true;
            }
        }
        if let Some(plain_oidc) = obj
            .get("oidc_client_secret")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            cfg.oidc_client_secret_enc =
                Some(master.seal("config.oidc_client_secret", plain_oidc)?);
            dirty = true;
        }
    }
    if let Some(ref enc_val) = cfg.oidc_client_secret_enc {
        let (pt, used_old) =
            master.open_with_old(old_master, "config.oidc_client_secret", enc_val)?;
        if used_old {
            cfg.oidc_client_secret_enc = Some(master.seal("config.oidc_client_secret", &pt)?);
            dirty = true;
            rewrapped = true;
        }
    }
    if cfg.version < 2 {
        cfg.version = 2;
        dirty = true;
    }
    Ok((cfg, dirty, rewrapped))
}

fn load_vault_file(
    path: &Path,
    master: &MasterKeys,
    old_master: Option<&MasterKeys>,
) -> Result<(VaultFile, bool, bool), CryptoError> {
    if !path.exists() {
        return Ok((VaultFile::default(), false, false));
    }
    let raw_text = Zeroizing::new(
        std::fs::read_to_string(path).map_err(|e| CryptoError::Io(e.to_string()))?,
    );
    let mut vault: VaultFile = serde_json::from_str(&raw_text)
        .map_err(|e| CryptoError::InvalidEnvelope(format!("vault.json: {e}")))?;
    let mut dirty = false;
    let mut rewrapped = false;

    for (id, env_val) in vault.api_keys.iter_mut() {
        let aad = format!("api_keys.{id}.secret");
        let (pt, used_old) = master.open_with_old(old_master, &aad, env_val)?;
        if used_old {
            *env_val = master.seal(&aad, &pt)?;
            dirty = true;
            rewrapped = true;
        }
    }
    for (id, env_val) in vault.mcp_connectors.iter_mut() {
        let aad = format!("mcp_connectors.{id}.secret");
        let (pt, used_old) = master.open_with_old(old_master, &aad, env_val)?;
        if used_old {
            *env_val = master.seal(&aad, &pt)?;
            dirty = true;
            rewrapped = true;
        }
    }
    Ok((vault, dirty, rewrapped))
}

fn load_and_migrate_keys(
    path: &Path,
    master: &MasterKeys,
    vault: &mut VaultFile,
    vault_dirty: &mut bool,
) -> Result<(Vec<StoredApiKey>, bool), CryptoError> {
    if !path.exists() {
        return Ok((Vec::new(), false));
    }
    let raw_text = Zeroizing::new(
        std::fs::read_to_string(path).map_err(|e| CryptoError::Io(e.to_string()))?,
    );
    let val: Value = match serde_json::from_str(&raw_text) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse keys.json; starting with empty list");
            return Ok((Vec::new(), false));
        }
    };
    let Some(arr) = val.get("api_keys").and_then(Value::as_array) else {
        return Ok((Vec::new(), false));
    };

    let mut migrated = false;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| crypto::random_hex(6));
        let label = obj
            .get("label")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Migrated API Key")
            .to_string();
        let enabled = obj.get("enabled").and_then(Value::as_bool).unwrap_or(true);
        let source = obj
            .get("source")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("runtime")
            .to_string();
        let scopes = obj
            .get("scopes")
            .and_then(Value::as_array)
            .map(|xs| {
                xs.iter()
                    .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|xs| !xs.is_empty())
            .unwrap_or_else(default_key_scopes);
        let created = obj
            .get("created")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let rotated_at = obj
            .get("rotated_at")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let revoked_at = obj
            .get("revoked_at")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        // Detect legacy Python plaintext `"key"` or `"raw_key"` and hash + encrypt into vault.json!
        let legacy_plain = obj
            .get("key")
            .and_then(Value::as_str)
            .or_else(|| obj.get("raw_key").and_then(Value::as_str))
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if let Some(plain) = legacy_plain {
            let key_hash = master.api_key_hash(plain);
            let (prefix, last4) = crypto::extract_prefix_and_last4(plain);
            let enc_box = master.seal(&format!("api_keys.{id}.secret"), plain)?;
            vault.api_keys.insert(id.clone(), enc_box);
            *vault_dirty = true;
            out.push(StoredApiKey {
                id,
                label,
                key_hash,
                prefix,
                last4,
                enabled,
                source,
                scopes,
                created,
                rotated_at,
                revoked_at,
            });
            migrated = true;
        } else if let Some(hash) = obj
            .get("key_hash")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let prefix = obj
                .get("prefix")
                .and_then(Value::as_str)
                .unwrap_or("styles_")
                .to_string();
            let last4 = obj
                .get("last4")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            out.push(StoredApiKey {
                id,
                label,
                key_hash: hash.to_string(),
                prefix,
                last4,
                enabled,
                source,
                scopes,
                created,
                rotated_at,
                revoked_at,
            });
        }
    }

    if val.get("version").and_then(Value::as_u64).unwrap_or(0) < 2 {
        migrated = true;
    }

    Ok((out, migrated))
}

fn load_and_migrate_connectors(
    path: &Path,
    master: &MasterKeys,
    vault: &mut VaultFile,
    vault_dirty: &mut bool,
) -> Result<(Vec<StoredMcpConnector>, bool), CryptoError> {
    if !path.exists() {
        return Ok((Vec::new(), false));
    }
    let raw_text = Zeroizing::new(
        std::fs::read_to_string(path).map_err(|e| CryptoError::Io(e.to_string()))?,
    );
    let mut parsed: ConnectorsFile = match serde_json::from_str(&raw_text) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse connectors.json; starting empty");
            return Ok((Vec::new(), false));
        }
    };
    let mut dirty = false;
    for conn in &mut parsed.connectors {
        if conn.id.is_empty() {
            conn.id = format!("mcp_{}", crypto::random_hex(5));
            dirty = true;
        }
        if !vault.mcp_connectors.contains_key(&conn.id) && conn.token_hash.is_empty() {
            let raw = Zeroizing::new(crypto::generate_api_token("styles_mcp"));
            let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
            conn.token_hash = master.api_key_hash(&raw);
            conn.prefix = prefix;
            conn.last4 = last4;
            vault.mcp_connectors.insert(
                conn.id.clone(),
                master.seal(&format!("mcp_connectors.{}.secret", conn.id), &raw)?,
            );
            *vault_dirty = true;
            dirty = true;
        }
    }
    Ok((parsed.connectors, dirty))
}

fn save_config_file(path: &Path, cfg: &PersistedConfig) -> Result<(), CryptoError> {
    let bytes = serde_json::to_vec_pretty(cfg).map_err(|e| CryptoError::Io(e.to_string()))?;
    crypto::write_private_file_atomic(path, &bytes)
}

fn save_keys_file(path: &Path, keys: &[StoredApiKey]) -> Result<(), CryptoError> {
    let payload = KeysFile {
        version: 2,
        api_keys: keys.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| CryptoError::Io(e.to_string()))?;
    crypto::write_private_file_atomic(path, &bytes)
}

fn save_connectors_file(
    path: &Path,
    connectors: &[StoredMcpConnector],
) -> Result<(), CryptoError> {
    let payload = ConnectorsFile {
        version: 1,
        connectors: connectors.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| CryptoError::Io(e.to_string()))?;
    crypto::write_private_file_atomic(path, &bytes)
}

fn save_vault_file(path: &Path, vault: &VaultFile) -> Result<(), CryptoError> {
    let bytes = serde_json::to_vec_pretty(vault).map_err(|e| CryptoError::Io(e.to_string()))?;
    crypto::write_private_file_atomic(path, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX_KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    #[test]
    fn legacy_keys_json_auto_migrates_and_scrubs_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let legacy_secret = "styles_LegacyPlaintextToken9876543210AbCdEf";
        let legacy_json = json!({
            "api_keys": [
                {
                    "id": "hub_default",
                    "label": "Business MCP Hub",
                    "key": legacy_secret,
                    "enabled": true,
                    "created": "2026-01-01T00:00:00Z"
                }
            ]
        });
        std::fs::write(
            dir.path().join("keys.json"),
            serde_json::to_string_pretty(&legacy_json).unwrap(),
        )
        .unwrap();

        let master = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let env = EnvSnapshot::from_pairs([("STYLES_API_KEY", "admin-super-secret-key")]);
        let store = RuntimeStore::load_or_init(dir.path(), master, &env).unwrap();

        // Disk file no longer contains plaintext secret or `"key"` / `"raw_key"`.
        let on_disk = std::fs::read_to_string(dir.path().join("keys.json")).unwrap();
        assert!(!on_disk.contains(legacy_secret), "plaintext leaked on disk");
        assert!(!on_disk.contains("\"raw_key\""));
        assert!(!on_disk.contains("\"key\":"));
        assert!(on_disk.contains("\"key_hash\""));

        // Migrated key still authenticates in constant time!
        assert!(store.validate_api_key(legacy_secret));
        assert!(!store.validate_api_key("styles_wrong"));

        // Public list view never leaks raw_key or key_hash.
        let list = store.list_keys();
        assert_eq!(list.len(), 1);
        let serialized = serde_json::to_string(&list).unwrap();
        assert!(!serialized.contains(legacy_secret));
        assert!(!serialized.contains("raw_key"));
        assert!(!serialized.contains("key_hash"));

        // Admin login works via Argon2id + HMAC.
        assert!(store.verify_mgmt_login("admin-super-secret-key"));
        assert!(!store.verify_mgmt_login("wrong-admin-key"));
        assert!(store.verify_mgmt_header("admin-super-secret-key"));
    }

    #[test]
    fn key_lifecycle_create_rotate_revoke_delete() {
        let dir = tempfile::tempdir().unwrap();
        let master = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let store = RuntimeStore::load_or_init(dir.path(), master, &EnvSnapshot::default()).unwrap();

        let (raw1, v1) = store.create_key("Agent A", None).unwrap();
        assert!(raw1.starts_with("styles_"));
        assert!(store.validate_api_key(&raw1));

        let (raw2, v2) = store.rotate_key(&v1.id, None).unwrap().unwrap();
        assert_ne!(*raw1, *raw2);
        assert!(!store.validate_api_key(&raw1), "old key invalidated");
        assert!(store.validate_api_key(&raw2), "rotated key valid");
        assert_eq!(v1.id, v2.id);

        let rev = store.revoke_key(&v1.id, true).unwrap().unwrap();
        assert!(!rev.enabled);
        assert!(!store.validate_api_key(&raw2), "revoked key rejected");

        let re_enabled = store.revoke_key(&v1.id, false).unwrap().unwrap();
        assert!(re_enabled.enabled);
        assert!(store.validate_api_key(&raw2));

        assert!(store.delete_key(&v1.id).unwrap());
        assert!(!store.validate_api_key(&raw2));
        assert_eq!(store.keys_count(), 0);
    }

    #[test]
    fn vault_copy_key_and_master_key_rotation_rewrap() {
        let dir = tempfile::tempdir().unwrap();
        let old_hex = "1111111111111111111111111111111111111111111111111111111111111111";
        let old_master = MasterKeys::from_encoded(old_hex).unwrap();
        let store1 =
            RuntimeStore::load_or_init(dir.path(), old_master, &EnvSnapshot::default()).unwrap();
        assert!(!store1.rewrapped_from_old_key());

        let (raw1, k1) = store1.create_key("Vault Key", None).unwrap();
        assert!(k1.copy_available);
        match store1.copy_key(&k1.id).unwrap() {
            CopySecretOutcome::Secret(s) => assert_eq!(*s, *raw1),
            other => panic!("unexpected copy outcome: {other:?}"),
        }

        // Seed a legacy hash-only key without vault.json entry -> returns LegacyHashOnly
        let legacy_hash_only = StoredApiKey {
            id: "legacy_hash_1".to_string(),
            label: "Legacy Hash Only".to_string(),
            key_hash: "deadbeef".to_string(),
            prefix: "styles_old".to_string(),
            last4: "9999".to_string(),
            enabled: true,
            source: "migrated".to_string(),
            scopes: default_key_scopes(),
            created: now_iso8601(),
            rotated_at: None,
            revoked_at: None,
        };
        let mut keys_disk: KeysFile =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join("keys.json")).unwrap())
                .unwrap();
        keys_disk.api_keys.push(legacy_hash_only);
        save_keys_file(&dir.path().join("keys.json"), &keys_disk.api_keys).unwrap();

        // Booting with wrong key WITHOUT LNC_MASTER_KEY_OLD must fail closed.
        let new_master = MasterKeys::from_encoded(HEX_KEY).unwrap();
        assert!(RuntimeStore::load_or_init(
            dir.path(),
            new_master.clone(),
            &EnvSnapshot::default()
        )
        .is_err());

        // Booting with LNC_MASTER_KEY_OLD re-wraps vault.json cleanly.
        let env_rot = EnvSnapshot::from_pairs([("LNC_MASTER_KEY_OLD", old_hex)]);
        let store2 = RuntimeStore::load_or_init(dir.path(), new_master.clone(), &env_rot).unwrap();
        assert!(store2.rewrapped_from_old_key());

        // Subsequent boot WITHOUT LNC_MASTER_KEY_OLD succeeds cleanly!
        let store3 =
            RuntimeStore::load_or_init(dir.path(), new_master, &EnvSnapshot::default()).unwrap();
        assert!(!store3.rewrapped_from_old_key());
        match store3.copy_key(&k1.id).unwrap() {
            CopySecretOutcome::Secret(s) => assert_eq!(*s, *raw1),
            other => panic!("unexpected copy outcome after rotation: {other:?}"),
        }
        assert_eq!(
            store3.copy_key("legacy_hash_1").unwrap(),
            CopySecretOutcome::LegacyHashOnly
        );
    }

    #[test]
    fn connector_lifecycle_and_env_lock() {
        let dir = tempfile::tempdir().unwrap();
        let master = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let env = EnvSnapshot::from_pairs([("CONNECTOR_BEARER", "styles_env_connector_bearer_001")]);
        let store = RuntimeStore::load_or_init(dir.path(), master, &env).unwrap();

        assert!(store.validate_api_key("styles_env_connector_bearer_001"));
        // Env-managed connector supports Copy, rejects Rotate/Revoke/Delete
        match store.copy_connector("mcp_env_bearer").unwrap() {
            CopySecretOutcome::Secret(s) => assert_eq!(&*s, "styles_env_connector_bearer_001"),
            other => panic!("unexpected copy: {other:?}"),
        }
        assert_eq!(
            store
                .rotate_connector("mcp_env_bearer", None, "https://styles.lengoc.me/mcp")
                .unwrap()
                .unwrap_err(),
            ConnectorMutationError::EnvManaged
        );

        let (rt_sec, rt_conn) = store
            .create_connector(
                "Claude Desktop",
                "styles-mcp",
                vec!["mcp:read".into()],
                None,
                "https://styles.lengoc.me/mcp",
            )
            .unwrap();
        assert!(store.validate_api_key(&rt_sec));

        // omitBlankSecrets: passing "********" preserves current secret
        let upd = store
            .update_connector(
                &rt_conn.id,
                Some("Claude Desktop Updated"),
                None,
                None,
                Some(crypto::MASK),
                "https://styles.lengoc.me/mcp",
            )
            .unwrap()
            .unwrap();
        assert_eq!(upd.name, "Claude Desktop Updated");
        assert!(store.validate_api_key(&rt_sec));

        let (rot_sec, _) = store
            .rotate_connector(&rt_conn.id, None, "https://styles.lengoc.me/mcp")
            .unwrap()
            .unwrap();
        assert!(!store.validate_api_key(&rt_sec));
        assert!(store.validate_api_key(&rot_sec));

        let rev = store
            .revoke_connector(&rt_conn.id, true, "https://styles.lengoc.me/mcp")
            .unwrap()
            .unwrap();
        assert!(!rev.enabled);
        assert!(!store.validate_api_key(&rot_sec));

        store.delete_connector(&rt_conn.id).unwrap().unwrap();
    }
}
