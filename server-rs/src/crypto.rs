//! Cryptographic primitives for `lnc-styles-gallery`.
//!
//! - [`MasterKeys`]: unified `LNC_MASTER_KEY` (64 hex chars or base64 of 32
//!   bytes) expanded via `HKDF-SHA256(salt = b"lnc-v1")` into four
//!   domain-separated subkeys, zeroized on drop.
//! - `$enc: 1` AES-256-GCM envelopes bound to a field path through the AAD
//!   `styles:<field_path>`; any tampering / wrong path / wrong key fails closed.
//! - Argon2id (`m=65536, t=3, p=4`) password hashing, HMAC-SHA256 hex digests
//!   with constant-time verification (`subtle::ConstantTimeEq`).
//! - Secret masking, random token generation, outbound URL SSRF validation.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::password_hash::rand_core::OsRng as PhOsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine as _;
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde_json::{json, Value};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use url::{Host, Url};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

type HmacSha256 = Hmac<Sha256>;

/// HKDF salt shared across LNC services deriving from `LNC_MASTER_KEY`.
pub const HKDF_SALT: &[u8] = b"lnc-v1";
pub const INFO_VAULT_KEK: &[u8] = b"vault-kek";
pub const INFO_SESSION_HMAC: &[u8] = b"session-hmac";
pub const INFO_OIDC_TX: &[u8] = b"oidc-tx";
pub const INFO_API_KEY_PEPPER: &[u8] = b"api-key-pepper";

/// Envelope format version (`"$enc": 1`).
pub const ENC_VERSION: u64 = 1;
/// Key name recorded in envelopes (`"kn"`).
pub const VAULT_KEY_NAME: &str = "vault-kek";
/// AAD prefix; full AAD is `styles:<field_path>`.
pub const AAD_PREFIX: &str = "styles:";
/// Placeholder shown instead of secrets.
pub const MASK: &str = "********";
/// Dev-fallback master key file name inside `STYLES_CONFIG_DIR`.
pub const MASTER_KEY_FILE: &str = ".master.key";

// Argon2id parameters mandated by CONSTRAINTS.md §2.
pub const ARGON2_M_COST_KIB: u32 = 65_536;
pub const ARGON2_T_COST: u32 = 3;
pub const ARGON2_P_COST: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    InvalidMasterKey(String),
    MasterKeyRequired,
    Io(String),
    InvalidEnvelope(String),
    DecryptFailed(String),
    Encrypt(String),
    Hash(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidMasterKey(m) => write!(f, "invalid LNC_MASTER_KEY: {m}"),
            CryptoError::MasterKeyRequired => {
                write!(
                    f,
                    "LNC_MASTER_KEY is required (LNC_REQUIRE_MASTER_KEY=true) but unset"
                )
            }
            CryptoError::Io(m) => write!(f, "master key io error: {m}"),
            CryptoError::InvalidEnvelope(m) => write!(f, "invalid $enc envelope: {m}"),
            CryptoError::DecryptFailed(p) => {
                write!(
                    f,
                    "failed to decrypt $enc field '{p}' (wrong key, tampered data or path mismatch)"
                )
            }
            CryptoError::Encrypt(m) => write!(f, "encryption failed: {m}"),
            CryptoError::Hash(m) => write!(f, "hashing failed: {m}"),
        }
    }
}

impl std::error::Error for CryptoError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    Env,
    File(PathBuf),
    Generated(PathBuf),
    Explicit,
}

/// Four HKDF-derived subkeys. Raw master key bytes are never retained.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKeys {
    vault_kek: [u8; 32],
    session_hmac: [u8; 32],
    oidc_tx: [u8; 32],
    api_key_pepper: [u8; 32],
}

impl fmt::Debug for MasterKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MasterKeys { <redacted> }")
    }
}

fn derive(hk: &Hkdf<Sha256>, info: &[u8]) -> [u8; 32] {
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    okm
}

impl MasterKeys {
    pub fn from_master(ikm: &[u8]) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), ikm);
        MasterKeys {
            vault_kek: derive(&hk, INFO_VAULT_KEK),
            session_hmac: derive(&hk, INFO_SESSION_HMAC),
            oidc_tx: derive(&hk, INFO_OIDC_TX),
            api_key_pepper: derive(&hk, INFO_API_KEY_PEPPER),
        }
    }

    pub fn from_encoded(encoded: &str) -> Result<Self, CryptoError> {
        let raw = parse_master_key(encoded)?;
        Ok(Self::from_master(raw.as_slice()))
    }

    pub fn load_or_init(config_dir: &Path) -> Result<(Self, KeySource), CryptoError> {
        let env_val = std::env::var("LNC_MASTER_KEY").ok();
        let require = matches!(
            std::env::var("LNC_REQUIRE_MASTER_KEY")
                .ok()
                .as_deref()
                .map(str::trim),
            Some("true") | Some("1") | Some("yes")
        );
        Self::load_or_init_with(env_val.as_deref(), require, config_dir)
    }

    pub fn load_or_init_with(
        env_value: Option<&str>,
        require_env: bool,
        config_dir: &Path,
    ) -> Result<(Self, KeySource), CryptoError> {
        if let Some(v) = env_value.map(str::trim).filter(|v| !v.is_empty()) {
            return Ok((Self::from_encoded(v)?, KeySource::Env));
        }
        if require_env {
            return Err(CryptoError::MasterKeyRequired);
        }
        let path = config_dir.join(MASTER_KEY_FILE);
        if path.exists() {
            let content = Zeroizing::new(
                std::fs::read_to_string(&path).map_err(|e| CryptoError::Io(e.to_string()))?,
            );
            let keys = Self::from_encoded(content.trim())?;
            tracing::warn!(
                path = %path.display(),
                "LNC_MASTER_KEY unset: using dev fallback master key file; set LNC_MASTER_KEY in production"
            );
            return Ok((keys, KeySource::File(path)));
        }
        std::fs::create_dir_all(config_dir).map_err(|e| CryptoError::Io(e.to_string()))?;
        let mut raw = Zeroizing::new([0u8; 32]);
        rand::rngs::OsRng.fill_bytes(raw.as_mut());
        let encoded = Zeroizing::new(hex::encode(raw.as_ref()));
        write_private_file_atomic(&path, encoded.as_bytes())?;
        tracing::warn!(
            path = %path.display(),
            "LNC_MASTER_KEY unset: GENERATED dev master key file; set LNC_MASTER_KEY explicitly in production"
        );
        Ok((Self::from_master(raw.as_ref()), KeySource::Generated(path)))
    }

    pub fn vault_kek(&self) -> &[u8; 32] {
        &self.vault_kek
    }
    pub fn session_hmac(&self) -> &[u8; 32] {
        &self.session_hmac
    }
    pub fn oidc_tx(&self) -> &[u8; 32] {
        &self.oidc_tx
    }
    pub fn api_key_pepper(&self) -> &[u8; 32] {
        &self.api_key_pepper
    }

    pub fn seal(&self, field_path: &str, plaintext: &str) -> Result<Value, CryptoError> {
        seal_field(&self.vault_kek, field_path, plaintext)
    }

    pub fn open(&self, field_path: &str, envelope: &Value) -> Result<String, CryptoError> {
        open_field(&self.vault_kek, field_path, envelope)
    }

    /// Peppered lookup hash for API keys / management key (`HMAC-SHA256` hex).
    pub fn api_key_hash(&self, secret: &str) -> String {
        hmac_sha256_hex(&self.api_key_pepper, secret.as_bytes())
    }

    /// Constant-time check of `secret` against a stored peppered hash.
    pub fn api_key_matches(&self, secret: &str, stored_hex: &str) -> bool {
        !secret.is_empty()
            && !stored_hex.is_empty()
            && hmac_verify_hex(&self.api_key_pepper, secret.as_bytes(), stored_hex)
    }
}

pub fn parse_master_key(encoded: &str) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let s = encoded.trim();
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        let v = hex::decode(s).map_err(|e| CryptoError::InvalidMasterKey(e.to_string()))?;
        return Ok(Zeroizing::new(v));
    }
    for engine in [&STANDARD, &STANDARD_NO_PAD, &URL_SAFE, &URL_SAFE_NO_PAD] {
        if let Ok(v) = engine.decode(s) {
            let v = Zeroizing::new(v);
            if v.len() == 32 {
                return Ok(v);
            }
        }
    }
    Err(CryptoError::InvalidMasterKey(
        "expected 64 hex chars or base64-encoded 32 bytes".into(),
    ))
}

/// Write a file atomically via sibling temp file + rename, with `0600` permissions on Unix.
pub fn write_private_file_atomic(path: &Path, data: &[u8]) -> Result<(), CryptoError> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CryptoError::Io(e.to_string()))?;
    }
    let tmp_name = format!(
        ".{}.tmp.{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("state"),
        random_hex(4)
    );
    let tmp_path = path.with_file_name(tmp_name);
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts
        .open(&tmp_path)
        .map_err(|e| CryptoError::Io(e.to_string()))?;
    f.write_all(data)
        .map_err(|e| CryptoError::Io(e.to_string()))?;
    f.sync_all().map_err(|e| CryptoError::Io(e.to_string()))?;
    drop(f);
    std::fs::rename(&tmp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_path);
        CryptoError::Io(e.to_string())
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// `$enc: 1` AES-256-GCM envelopes
// ---------------------------------------------------------------------------

pub fn is_envelope(v: &Value) -> bool {
    v.as_object().is_some_and(|o| o.contains_key("$enc"))
}

fn aad_for(field_path: &str) -> Vec<u8> {
    format!("{AAD_PREFIX}{field_path}").into_bytes()
}

pub fn seal_field(kek: &[u8; 32], field_path: &str, plaintext: &str) -> Result<Value, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(kek).map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    let mut nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let aad = aad_for(field_path);
    let ct = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext.as_bytes(),
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Encrypt("aes-gcm seal".into()))?;
    Ok(json!({
        "$enc": ENC_VERSION,
        "kn": VAULT_KEY_NAME,
        "dn": field_path,
        "key": STANDARD.encode(nonce),
        "data": STANDARD.encode(ct),
    }))
}

pub fn open_field(
    kek: &[u8; 32],
    field_path: &str,
    envelope: &Value,
) -> Result<String, CryptoError> {
    let obj = envelope
        .as_object()
        .ok_or_else(|| CryptoError::InvalidEnvelope(format!("{field_path}: not an object")))?;
    match obj.get("$enc").and_then(Value::as_u64) {
        Some(ENC_VERSION) => {}
        _ => {
            return Err(CryptoError::InvalidEnvelope(format!(
                "{field_path}: unsupported $enc version"
            )))
        }
    }
    if let Some(kn) = obj.get("kn") {
        if kn.as_str() != Some(VAULT_KEY_NAME) {
            return Err(CryptoError::InvalidEnvelope(format!(
                "{field_path}: unknown key name"
            )));
        }
    }
    let nonce_b64 = obj
        .get("key")
        .and_then(Value::as_str)
        .ok_or_else(|| CryptoError::InvalidEnvelope(format!("{field_path}: missing nonce")))?;
    let data_b64 = obj
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| CryptoError::InvalidEnvelope(format!("{field_path}: missing data")))?;
    let nonce = STANDARD
        .decode(nonce_b64)
        .map_err(|_| CryptoError::InvalidEnvelope(format!("{field_path}: bad nonce encoding")))?;
    if nonce.len() != 12 {
        return Err(CryptoError::InvalidEnvelope(format!(
            "{field_path}: nonce must be 12 bytes"
        )));
    }
    let ct = STANDARD
        .decode(data_b64)
        .map_err(|_| CryptoError::InvalidEnvelope(format!("{field_path}: bad data encoding")))?;
    if ct.len() < 16 {
        return Err(CryptoError::InvalidEnvelope(format!(
            "{field_path}: ciphertext too short"
        )));
    }
    let cipher = Aes256Gcm::new_from_slice(kek)
        .map_err(|_| CryptoError::DecryptFailed(field_path.into()))?;
    let aad = aad_for(field_path);
    let pt = Zeroizing::new(
        cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ct,
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::DecryptFailed(field_path.into()))?,
    );
    String::from_utf8(pt.to_vec()).map_err(|_| CryptoError::DecryptFailed(field_path.into()))
}

// ---------------------------------------------------------------------------
// Argon2id + HMAC-SHA256 + ConstantTimeEq
// ---------------------------------------------------------------------------

fn argon2() -> Argon2<'static> {
    let params = Params::new(ARGON2_M_COST_KIB, ARGON2_T_COST, ARGON2_P_COST, None)
        .expect("static argon2 params are valid");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Argon2id PHC hash (`$argon2id$v=19$m=65536,t=3,p=4$...`).
pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut PhOsRng);
    argon2()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| CryptoError::Hash(e.to_string()))
}

/// Verify a password against an Argon2 PHC string.
pub fn verify_password(password: &str, phc: &str) -> bool {
    if password.is_empty() || phc.is_empty() {
        return false;
    }
    match PasswordHash::new(phc) {
        Ok(parsed) => argon2()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn hmac_sha256_hex(key: &[u8], msg: &[u8]) -> String {
    hex::encode(hmac_sha256(key, msg))
}

pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().into()
}

pub fn hmac_verify_hex(key: &[u8], msg: &[u8], expected_hex: &str) -> bool {
    let Ok(expected) = hex::decode(expected_hex.trim()) else {
        return false;
    };
    let actual = hmac_sha256(key, msg);
    ct_eq(&actual, &expected)
}

pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    !a.is_empty() && a.len() == b.len() && bool::from(a.ct_eq(b))
}

pub fn ct_eq_str(a: &str, b: &str) -> bool {
    ct_eq(a.as_bytes(), b.as_bytes())
}

// ---------------------------------------------------------------------------
// Token & redaction helpers
// ---------------------------------------------------------------------------

/// Extract non-reversible `(prefix, last4)` display metadata from a raw token.
/// Ensures short secrets never expose enough characters to reconstruct the secret.
pub fn extract_prefix_and_last4(secret: &str) -> (String, String) {
    let chars: Vec<char> = secret.chars().collect();
    let n = chars.len();
    if n >= 18 {
        let prefix: String = chars[..10].iter().collect();
        let last4: String = chars[n - 4..].iter().collect();
        (prefix, last4)
    } else if n >= 10 {
        let prefix: String = chars[..4].iter().collect();
        let last4: String = chars[n - 2..].iter().collect();
        (prefix, last4)
    } else {
        ("***".to_string(), String::new())
    }
}

/// Format masked key representation (`styles_abc••••••••wxyz`) without raw key material.
pub fn format_masked_key(prefix: &str, last4: &str) -> String {
    if last4.is_empty() {
        format!("{prefix}••••••••")
    } else {
        format!("{prefix}••••••••{last4}")
    }
}

pub fn mask_admin_key(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    match chars.len() {
        0 => "***".to_string(),
        n if n <= 6 => "***".to_string(),
        n => {
            let last4: String = chars[n - 4..].iter().collect();
            format!("***{last4}")
        }
    }
}

/// Generate `"<prefix>_<base64url(24 bytes)>"` token (e.g. `styles_<32 chars>`).
pub fn generate_api_token(prefix: &str) -> String {
    let mut b = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut b);
    let body = URL_SAFE_NO_PAD.encode(b);
    b.zeroize();
    if prefix.is_empty() {
        body
    } else {
        format!("{prefix}_{body}")
    }
}

pub fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    rand::rngs::OsRng.fill_bytes(&mut b);
    hex::encode(b)
}

// ---------------------------------------------------------------------------
// SSRF validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsrfError {
    Invalid(String),
    Scheme(String),
    MissingHost,
    Blocked(String),
}

impl fmt::Display for SsrfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsrfError::Invalid(m) => write!(f, "invalid url: {m}"),
            SsrfError::Scheme(s) => write!(f, "url scheme must be http or https (got '{s}')"),
            SsrfError::MissingHost => write!(f, "url has no host"),
            SsrfError::Blocked(r) => write!(f, "url target blocked: {r}"),
        }
    }
}

impl std::error::Error for SsrfError {}

pub fn ssrf_block_private_from_env() -> bool {
    matches!(
        std::env::var("SSRF_BLOCK_PRIVATE")
            .ok()
            .map(|v| v.trim().to_ascii_lowercase())
            .as_deref(),
        Some("true") | Some("1") | Some("yes")
    )
}

const METADATA_HOSTS: &[&str] = &[
    "metadata",
    "metadata.google.internal",
    "instance-data",
    "instance-data.ec2.internal",
];

fn v4_block_reason(ip: Ipv4Addr, block_private: bool) -> Option<&'static str> {
    let o = ip.octets();
    if o[0] == 169 && o[1] == 254 {
        return Some("cloud metadata / link-local address (169.254.0.0/16)");
    }
    if o == [100, 100, 100, 200] {
        return Some("cloud metadata address");
    }
    if block_private {
        if ip.is_loopback() {
            return Some("loopback address");
        }
        if ip.is_private() {
            return Some("private network address");
        }
        if ip.is_unspecified() || ip.is_broadcast() {
            return Some("unspecified/broadcast address");
        }
        if o[0] == 100 && (o[1] & 0xc0) == 64 {
            return Some("shared address space (100.64.0.0/10)");
        }
    }
    None
}

pub fn ip_block_reason(ip: IpAddr, block_private: bool) -> Option<&'static str> {
    match ip {
        IpAddr::V4(v4) => v4_block_reason(v4, block_private),
        IpAddr::V6(v6) => {
            if v6 == Ipv6Addr::new(0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254) {
                return Some("cloud metadata address (fd00:ec2::254)");
            }
            let seg = v6.segments();
            let embedded = if let Some(v4) = v6.to_ipv4_mapped() {
                Some(v4)
            } else if seg[..6] == [0x64, 0xff9b, 0, 0, 0, 0] || (seg[..6] == [0; 6] && seg[6] != 0)
            {
                Some(Ipv4Addr::new(
                    (seg[6] >> 8) as u8,
                    seg[6] as u8,
                    (seg[7] >> 8) as u8,
                    seg[7] as u8,
                ))
            } else {
                None
            };
            if let Some(v4) = embedded {
                if let Some(r) = v4_block_reason(v4, block_private) {
                    return Some(r);
                }
            }
            if (seg[0] & 0xffc0) == 0xfe80 {
                return Some("IPv6 link-local address (fe80::/10)");
            }
            if block_private {
                if v6.is_loopback() || v6.is_unspecified() {
                    return Some("loopback/unspecified address");
                }
                if (seg[0] & 0xfe00) == 0xfc00 {
                    return Some("unique local address (fc00::/7)");
                }
            }
            None
        }
    }
}

pub fn validate_outbound_url(raw: &str, block_private: bool) -> Result<Url, SsrfError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SsrfError::Invalid("empty".into()));
    }
    let url = Url::parse(trimmed).map_err(|e| SsrfError::Invalid(e.to_string()))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(SsrfError::Scheme(other.to_string())),
    }
    let host = url.host().ok_or(SsrfError::MissingHost)?;
    match host {
        Host::Domain(d) => {
            let d = d.trim_end_matches('.').to_ascii_lowercase();
            if d.is_empty() {
                return Err(SsrfError::MissingHost);
            }
            if METADATA_HOSTS.contains(&d.as_str()) || d.ends_with(".metadata.google.internal") {
                return Err(SsrfError::Blocked(format!("cloud metadata host '{d}'")));
            }
            if block_private && (d == "localhost" || d.ends_with(".localhost")) {
                return Err(SsrfError::Blocked("loopback host".into()));
            }
        }
        Host::Ipv4(v4) => {
            if let Some(r) = ip_block_reason(IpAddr::V4(v4), block_private) {
                return Err(SsrfError::Blocked(r.into()));
            }
        }
        Host::Ipv6(v6) => {
            if let Some(r) = ip_block_reason(IpAddr::V6(v6), block_private) {
                return Err(SsrfError::Blocked(r.into()));
            }
        }
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX_KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    #[test]
    fn master_keys_hkdf_and_fallback() {
        let k = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let raw: Vec<u8> = (0u8..32).collect();
        let k_b64 = MasterKeys::from_encoded(&STANDARD.encode(&raw)).unwrap();
        assert_eq!(k.vault_kek(), k_b64.vault_kek());
        assert_ne!(k.vault_kek(), k.session_hmac());
        assert_ne!(k.oidc_tx(), k.api_key_pepper());

        let dir = tempfile::tempdir().unwrap();
        let (g1, src1) = MasterKeys::load_or_init_with(None, false, dir.path()).unwrap();
        assert!(matches!(src1, KeySource::Generated(_)));
        let (g2, src2) = MasterKeys::load_or_init_with(None, false, dir.path()).unwrap();
        assert!(matches!(src2, KeySource::File(_)));
        assert_eq!(g1.api_key_pepper(), g2.api_key_pepper());
    }

    #[test]
    fn envelope_seal_open_and_tamper_rejection() {
        let k = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let env = k.seal("config.oidc_secret", "super-secret-value").unwrap();
        assert!(!env.to_string().contains("super-secret-value"));
        assert_eq!(
            k.open("config.oidc_secret", &env).unwrap(),
            "super-secret-value"
        );
        assert!(k.open("config.wrong_aad", &env).is_err());
    }

    #[test]
    fn argon2id_and_hmac_constant_time() {
        let phc = hash_password("styles-mgmt-secret").unwrap();
        assert!(phc.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"));
        assert!(verify_password("styles-mgmt-secret", &phc));
        assert!(!verify_password("wrong", &phc));

        let k = MasterKeys::from_encoded(HEX_KEY).unwrap();
        let tok = generate_api_token("styles");
        assert!(tok.starts_with("styles_"));
        let h = k.api_key_hash(&tok);
        assert!(k.api_key_matches(&tok, &h));
        assert!(!k.api_key_matches("styles_wrong", &h));
    }

    #[test]
    fn ssrf_blocks_cloud_metadata() {
        assert!(validate_outbound_url("http://169.254.169.254/latest/meta-data/", false).is_err());
        assert!(validate_outbound_url("http://[fd00:ec2::254]/latest/", false).is_err());
        assert!(validate_outbound_url("http://metadata.google.internal/", false).is_err());
        assert!(validate_outbound_url("ftp://styles.lengoc.me", false).is_err());
        assert!(validate_outbound_url("https://styles.lengoc.me", false).is_ok());
    }
}
