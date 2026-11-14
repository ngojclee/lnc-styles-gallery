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
    "1.0.0.0".to_string()
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeysFile {
    #[serde(default = "default_cfg_version")]
    pub version: u32,
    #[serde(default)]
    pub api_keys: Vec<StoredApiKey>,
}

/// Redacted public view returned by `GET /api/keys` and key mutation `item`.
/// Never contains `raw_key`, `key`, or `key_hash`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKeyPublicView {
    pub id: String,
    pub label: String,
    pub prefix: String,
    pub last4: String,
    pub masked: String,
    pub enabled: bool,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

impl StoredApiKey {
    pub fn to_public_view(&self) -> ApiKeyPublicView {
        ApiKeyPublicView {
            id: self.id.clone(),
            label: self.label.clone(),
            prefix: self.prefix.clone(),
            last4: self.last4.clone(),
            masked: crypto::format_masked_key(&self.prefix, &self.last4),
            enabled: self.enabled,
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
}

/// Thread-safe manager for `config.json` and `keys.json` in `${STYLES_CONFIG_DIR}`.
pub struct RuntimeStore {
    config_dir: PathBuf,
    master: MasterKeys,
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
        let config_path = config_dir.join("config.json");
        let keys_path = config_dir.join("keys.json");

        let (mut cfg, mut cfg_dirty) = load_and_migrate_config(&config_path, &master)?;

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

        let (mut keys, mut keys_dirty) = load_and_migrate_keys(&keys_path, &master)?;

        // Optional seed connector token from env (`MCP_CONNECTOR_TOKEN` / `STYLES_HUB_API_KEY`).
        if let Some(seed_tok) = env.first(&["STYLES_HUB_API_KEY", "MCP_CONNECTOR_TOKEN"]) {
            let h = master.api_key_hash(seed_tok);
            let exists = keys.iter().any(|k| crypto::ct_eq_str(&k.key_hash, &h));
            if !exists {
                let (prefix, last4) = crypto::extract_prefix_and_last4(seed_tok);
                keys.push(StoredApiKey {
                    id: "hub_default".to_string(),
                    label: "Business MCP Hub".to_string(),
                    key_hash: h,
                    prefix,
                    last4,
                    enabled: true,
                    created: now_iso8601(),
                    rotated_at: None,
                    revoked_at: None,
                });
                keys_dirty = true;
            }
        }

        if keys_dirty {
            save_keys_file(&keys_path, &keys)?;
        }

        Ok(Self {
            config_dir: config_dir.to_path_buf(),
            master,
            inner: RwLock::new(StoreInner { cfg, keys }),
        })
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

    /// Constant-time check of an MCP Bearer token against enabled `keys.json` entries OR admin key.
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
        matched
    }

    pub fn keys_count(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .keys
            .len()
    }

    pub fn list_keys(&self) -> Vec<ApiKeyPublicView> {
        let g = self.inner.read().unwrap_or_else(|p| p.into_inner());
        g.keys.iter().map(StoredApiKey::to_public_view).collect()
    }

    pub fn create_key(
        &self,
        label: &str,
        custom_secret: Option<&str>,
    ) -> Result<(Zeroizing<String>, ApiKeyPublicView), CryptoError> {
        let clean_label = {
            let t = label.trim();
            if t.is_empty() {
                "Unnamed API Key".to_string()
            } else {
                t.chars().take(120).collect()
            }
        };
        let raw = Zeroizing::new(match custom_secret.map(str::trim).filter(|s| !s.is_empty()) {
            Some(c) => c.to_string(),
            None => crypto::generate_api_token("styles"),
        });
        let key_hash = self.master.api_key_hash(&raw);
        let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
        let stored = StoredApiKey {
            id: crypto::random_hex(6),
            label: clean_label,
            key_hash,
            prefix,
            last4,
            enabled: true,
            created: now_iso8601(),
            rotated_at: None,
            revoked_at: None,
        };
        let view = stored.to_public_view();
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        g.keys.push(stored);
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        Ok((raw, view))
    }

    pub fn rotate_key(
        &self,
        key_id: &str,
        custom_secret: Option<&str>,
    ) -> Result<Option<(Zeroizing<String>, ApiKeyPublicView)>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(entry) = g.keys.iter_mut().find(|k| k.id == key_id) else {
            return Ok(None);
        };
        let raw = Zeroizing::new(match custom_secret.map(str::trim).filter(|s| !s.is_empty()) {
            Some(c) => c.to_string(),
            None => crypto::generate_api_token("styles"),
        });
        let (prefix, last4) = crypto::extract_prefix_and_last4(&raw);
        entry.key_hash = self.master.api_key_hash(&raw);
        entry.prefix = prefix;
        entry.last4 = last4;
        entry.enabled = true;
        entry.rotated_at = Some(now_iso8601());
        entry.revoked_at = None;
        let view = entry.to_public_view();
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        Ok(Some((raw, view)))
    }

    pub fn revoke_key(
        &self,
        key_id: &str,
        revoke: bool,
    ) -> Result<Option<ApiKeyPublicView>, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let Some(entry) = g.keys.iter_mut().find(|k| k.id == key_id) else {
            return Ok(None);
        };
        entry.enabled = !revoke;
        entry.revoked_at = if revoke { Some(now_iso8601()) } else { None };
        let view = entry.to_public_view();
        save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
        Ok(Some(view))
    }

    pub fn delete_key(&self, key_id: &str) -> Result<bool, CryptoError> {
        let mut g = self.inner.write().unwrap_or_else(|p| p.into_inner());
        let before = g.keys.len();
        g.keys.retain(|k| k.id != key_id);
        if g.keys.len() < before {
            save_keys_file(&self.config_dir.join("keys.json"), &g.keys)?;
            return Ok(true);
        }
        Ok(false)
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
            "auto_warm_cache": settings.auto_warm_cache,
        })
    }
}

fn load_and_migrate_config(
    path: &Path,
    master: &MasterKeys,
) -> Result<(PersistedConfig, bool), CryptoError> {
    if !path.exists() {
        return Ok((PersistedConfig::default(), false));
    }
    let raw_text = std::fs::read_to_string(path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let val: Value = match serde_json::from_str(&raw_text) {
        Ok(v) => v,
        Err(_) => return Ok((PersistedConfig::default(), true)),
    };
    let mut cfg: PersistedConfig = serde_json::from_value(val.clone()).unwrap_or_default();
    let mut dirty = false;

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
    if cfg.version < 2 {
        cfg.version = 2;
        dirty = true;
    }
    Ok((cfg, dirty))
}

fn load_and_migrate_keys(
    path: &Path,
    master: &MasterKeys,
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

        // Detect legacy Python plaintext `"key"` or `"raw_key"` and hash + scrub!
        let legacy_plain = obj
            .get("key")
            .and_then(Value::as_str)
            .or_else(|| obj.get("raw_key").and_then(Value::as_str))
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if let Some(plain) = legacy_plain {
            let key_hash = master.api_key_hash(plain);
            let (prefix, last4) = crypto::extract_prefix_and_last4(plain);
            out.push(StoredApiKey {
                id,
                label,
                key_hash,
                prefix,
                last4,
                enabled,
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
}
