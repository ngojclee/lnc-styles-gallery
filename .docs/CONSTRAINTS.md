# Architectural & Security Constraints — lnc-styles-gallery (Rust v1.0.0.0)

> Mechanical invariants enforced across `server-rs/`, `ui/`, `Dockerfile`, and deployment manifests. Any `FAIL` on a MUST invariant blocks cutover.

## 1. Drop-In Runtime, Read-Only Corpus & API Parity (MUST)
- **Port & Health**: HTTP server listens on `0.0.0.0:${PORT:-8000}` (also honor `STYLES_PORT`; host-mapped to `8460`). `GET /health` & `GET /api/health` return `200` unauthenticated corpus status without secrets.
- **Data & Read-Only Corpus Paths**:
  - Read read-only corpus from `${STYLES_DATA_DIR:-/data}` (`refero-styles/`, `prompt-driven-ui/`, `lnc-components/`, or env overrides `REFERO_STYLES_DIR`, `PROMPT_UI_DIR`, `COMPONENTS_DIR`) safely without mutating `/data`.
  - Write mutable state ONLY into writable `${STYLES_CONFIG_DIR:-/config}` (`keys.json`, `config.json`, `.master.key` with `0600` atomic write). Auto-migrate legacy plaintext `keys.json` (`styles_*` keys) on first Rust boot into hashed/encrypted storage.
- **API & MCP Contract Parity**:
  - Serve React + TanStack Router SPA (`ui/dist`, `/static`, `/assets`, fallback `/styles`, `/collections`, `/templates`, `/components`, `/platforms`, `/keys`, `/licenses`, `/admin`) + `/login.html`.
  - Preserve all corpus browse/search endpoints (`/api/styles*`, `/api/search`, `/api/collections*`, `/api/templates*`, `/api/components*`, `/api/licenses`).
  - Preserve Streamable HTTP JSON-RPC 2.0 `POST|GET /mcp` with all **8 MCP tools**: `list_styles`, `search`, `get_design`, `list_collections`, `get_template`, `list_components`, `get_component`, `reindex`. Enforce Bearer auth on `/mcp` when `STYLES_MCP_AUTH=1|true` (also support key validation via `key_hash` in constant time).
- **Side-by-Side Preservation**: Keep `app/*.py`, `requirements.txt`, and backup `Dockerfile` -> `Dockerfile.py` intact until E2E parity sign-off.

## 2. Cryptographic & Zero-F12-Leak Hardening (MUST)
- **Fix Legacy Plaintext `raw_key` Leak**:
  - Legacy `app/key_service.py` leaked `"raw_key"` in `GET /api/keys` and seeded keys on disk in plaintext. **MUST** store `key_hash = HMAC-SHA256(api-key-pepper, raw)` + `prefix` + `last4` only, and NEVER return `raw_key` / full tokens in `GET /api/keys`, `/api/admin/settings`, or `/api/mcp/info`.
  - Raw `styles_...` key is returned **once** on `POST /api/keys` and `POST /api/keys/{id}/rotate` (and initial dev seed log/bootstrap if needed, never over HTTP GET).
- **`LNC_MASTER_KEY` + HKDF + `Argon2id`**:
  - Derive `vault-kek`, `session-hmac`, `oidc-tx`, `api-key-pepper` via `HKDF-SHA256(salt = b"lnc-v1", ikm = master_key)` (auto-create `${STYLES_CONFIG_DIR}/.master.key` `0600` in dev only).
  - Hash `STYLES_API_KEY` / `ADMIN_API_KEY` / management credential via `Argon2id` (`m=65536,t=3,p=4`) + `HMAC-SHA256`; constant-time comparison (`subtle::ConstantTimeEq`).

## 3. Auth, SSO (Keycloak OIDC), Session & HTTP Hardening (MUST)
- **Authentication Gate on Admin/Key Routes**:
  - Unlike legacy `app/main.py` where `GET /api/keys` and `POST /api/keys` had weak/missing session gates when `STYLES_API_KEY` was unset, require authenticated session (`lnc_styles_session` cookie) OR valid admin `X-API-Key`/`Bearer` for key mutations (`POST|DELETE /api/keys*`, `POST|GET /api/admin/reindex`) and optionally protect `/api/keys` + `/keys` UI when auth is configured (allow graceful internal setup banner when unconfigured, or require OIDC/break-glass login once `MANAGEMENT_KEY` / `STYLES_API_KEY` / `OIDC_ENABLED` is active!).
- **Keycloak OIDC SSO (`lengoc` realm)**:
  - `GET /api/auth/oidc/{login,callback}`, `GET /api/auth/session`, `GET /api/auth/status`, `POST /api/auth/login`, `POST /api/auth/logout`, `POST /api/auth/revoke-all` (`session_epoch` bump).
  - `PKCE S256` + `state` + `nonce` (`5m` encrypted cookie), JWKS RS256/ES256 validation, `OIDC_ALLOWED_USERS`/`OIDC_ALLOWED_GROUPS`.
  - **`12h` idle TTL + `7d` max TTL + `session_epoch`**, `8/5m -> 429` brute-force guard, CSRF origin guard on cookie mutations, strict CSP & security headers, SSRF metadata block on any URL settings.
- **UI & Select Polish**:
  - Preserve Shadcn/Radix portaled rounded dropdowns (`border-radius: 12px` / `6px`), upgrade `ui/src/routes/keys.tsx` & header/login for OIDC + Zero-F12-Leak one-time key modal + `corepack pnpm build` (`ui/dist`).

## 4. Toolchain & Verification Gates (MUST)
- `cargo clippy --all-targets -- -D warnings` exit `0`, `cargo test` 100%, `cargo build --release` single static binary (`rustls`), `corepack pnpm build` succeeds in `ui/`.
- `py -3 tests/e2e/e2e_scorecard.py` against temp Refero/Template/Component fixture corpus + live Rust binary scores **`>= 90/100`** (`0` Critical/High, Hard-Fail `CLEAR`).
