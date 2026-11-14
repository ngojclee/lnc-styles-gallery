# Project Status — `lnc-styles-gallery` (`v1.0.0.0`)

- **Last Updated**: `2026-11-14`
- **Runtime**: Rust (`axum 0.8` + `tokio` + `reqwest` + `rustls`) + React 19 / Vite 8 / TanStack Router SPA
- **Version**: `1.0.0.0` (legacy Python preserved in `app/*.py` and `Dockerfile.py`)
- **Container / Host Port**: `8000` (`PORT` / `STYLES_PORT`) -> host `8460` (`https://styles.lengoc.me`)
- **Release Readiness (`/w18` Track M Scorecard)**: **`100.0 / 100`** (`23/23` E2E checks passed, `0` Critical/High/Medium/Low, Hard-Fail `CLEAR`, `RELEASE_READY`)

---

## Architecture & Security Summary

1. **Single-Binary Rust Server (`server-rs/`)**:
   - `src/crypto.rs`: `LNC_MASTER_KEY` (64 hex or 32-byte base64; dev fallback `${STYLES_CONFIG_DIR}/.master.key` `0600`) -> `HKDF-SHA256(salt=b"lnc-v1")` (`vault-kek`, `session-hmac`, `oidc-tx`, `api-key-pepper`), AES-256-GCM `$enc:1` field envelopes with path AAD, `Argon2id` (`m=65536, t=3, p=4`), salted `HMAC-SHA256` (`subtle::ConstantTimeEq`), `zeroize`, and outbound SSRF metadata guard (`169.254.169.254`, `169.254.0.0/16`, `fd00:ec2::254`, `metadata.google.internal`).
   - `src/config.rs`: Mutable state isolated to `${STYLES_CONFIG_DIR:-/config}`. Auto-migrates legacy plaintext `keys.json` on startup—computing salted `HMAC-SHA256` hashes + `prefix` + `last4` and atomically scrubbing plaintext `key`/`raw_key` from disk while preserving seamless MCP Hub authentication.
   - `src/oidc.rs`: Keycloak OIDC (`lengoc` realm, `PKCE S256` + `state` + `nonce` in 5m AES-256-GCM `lnc_styles_oidc_tx` cookie, asymmetric JWKS verification, `OIDC_ALLOWED_USERS` / `OIDC_ALLOWED_GROUPS`), `lnc_styles_session` v2 (`12h` idle / `7d` max / `session_epoch` / `POST /api/auth/revoke-all`), and `8/5m -> 429` + `Retry-After` login rate limiter.
   - `src/store.rs`: Strictly read-only in-memory corpus indexer over `${STYLES_DATA_DIR:-/data}` (`refero-styles/`, `prompt-driven-ui/`, `lnc-components/`), dynamic SVG fallbacks, full-text search, and canonicalized path confinement.
   - `src/mcp.rs`: Streamable HTTP JSON-RPC 2.0 (`POST|GET /mcp`) exposing all 8 tools (`list_styles`, `search`, `get_design`, `list_collections`, `get_template`, `list_components`, `get_component`, `reindex`) with `styles__` Hub prefix stripping and HTTP `202 Accepted` on notifications.
   - `src/server.rs` & `src/main.rs`: Full REST `/api/*` parity, Zero-F12-Leak `/api/keys*` (`410 Gone` on `/copy`), CSRF Origin check, external-JS `/login.html` + `/login.js`, SPA fallback, `--healthcheck` CLI probe, and strict CSP/security headers.

2. **Hardened React 19 SPA (`ui/src/*`)**:
   - `AuthGateCard.tsx` + `AppHeader.tsx`: Primary **"Sign in with LNC SSO"** button + collapsed `<details>` break-glass management key form, active session badge, sign-out, and emergency **Revoke All Sessions**.
   - `routes/keys.tsx` & `routes/admin.tsx`: Zero `raw_key` in table/DOM/`localStorage`, one-time creation/rotation modal secret display, rounded Radix/Shadcn `<Select>` (`rounded-[12px]` / `rounded-[6px]`), and auth-gated admin diagnostics.

3. **Packaging & Evidence**:
   - `Dockerfile`: Multi-stage `node:20-alpine` + `rust:1-bookworm` + `debian:bookworm-slim` running as `uid:gid 10001:10001` with native `HEALTHCHECK CMD ["/app/lnc-styles-gallery", "--healthcheck"]`.
   - `Dockerfile.py` & `app/*.py`: Preserved untouched as Python rollback reference.
   - Evidence: [scorecard.md](file:///D:/Python/projects/lnc-styles-gallery/.docs/test_evidence/2026-11-14-rust-v1/scorecard.md) & [baseline.json](file:///D:/Python/projects/lnc-styles-gallery/.docs/test_evidence/2026-11-14-rust-v1/baseline.json).

---

## Verification Metrics (`2026-11-14`)

| Gate | Result |
| :--- | :--- |
| `cargo clippy --all-targets -- -D warnings` | **PASS** (`0` warnings) |
| `cargo test` | **PASS** (`11/11` unit + router/OIDC/MCP tests) |
| `cargo build --release` | **PASS** (`6,929,920` bytes / `6.61 MiB`) |
| `corepack pnpm build` | **PASS** (`tsc -b && vite build`, `2,353` modules, zero `raw_key` in `ui/dist`) |
| `py -3 tests/e2e/e2e_scorecard.py` | **`100.0 / 100`** (`23/23` checks, `0` Crit/High, Hard-Fail `CLEAR`, idle RSS `8.32 MB`, post-load RSS `8.65 MB`, `p50=0.63 ms`, `p95=16.18 ms`) |
