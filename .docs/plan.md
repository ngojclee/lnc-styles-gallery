# Execution Plan — `lnc-styles-gallery` Rust Migration (`v1.0.0.0`)

## Methodology & Target
- **Workflow**: `/w2-specdrivendev` Track M + `/w5-round-orchestrator` + `/w18-e2e-qa-scorecard` (target **`>= 90/100`**, Hard-Fail `CLEAR`).
- **Workspace**: `D:\Python\projects\lnc-styles-gallery\server-rs\` (Rust `axum` + `tokio` + `rustls` binary serving React SPA `ui/dist`, `/mcp` 8 tools, corpus indexer, and hardened auth/SSO).

## Slices
1. **Wave 0 (`server-rs/src/{lib,crypto,config,store}.rs`)**:
   - `crypto.rs`: `LNC_MASTER_KEY` + `${STYLES_CONFIG_DIR}/.master.key`, HKDF subkeys, AES-256-GCM `$enc:1`, `Argon2id` + `HMAC-SHA256` key hashing.
   - `config.rs`: Env + `${STYLES_CONFIG_DIR}/{config.json, keys.json}` migration (scrubbing plaintext `raw_key`/`key` into `key_hash` + `prefix` + `last4`), read-only `/data` corpus path resolution (`refero-styles`, `prompt-driven-ui`, `lnc-components`).
   - `store.rs`: 1-to-1 fast Rust port of `app/store.py` (in-memory thread-safe index of styles, collections, HTML templates, HTML/TSX components, SVG fallback previews, licenses, full-text search & `reindex()`).
2. **Wave 1A (`server-rs/src/{oidc,mcp,server,main}.rs`, `Dockerfile*`, `infra/docker-compose.yml`)**:
   - `oidc.rs`: Keycloak `lengoc` PKCE `S256`, `lnc_styles_session` `v2` (`12h`/`7d`/`session_epoch`), `8/5m -> 429`, CSRF, `revoke-all`.
   - `mcp.rs`: All 8 `styles__*` tools on `GET|POST /mcp`, optional `STYLES_MCP_AUTH` Bearer check.
   - `server.rs`: All `/api/*` corpus, keys (Zero-F12-Leak), admin settings/reindex, SPA `ui/dist` & `/assets` serving, `/login.html`, security headers, path traversal guard.
   - Multi-stage `Dockerfile` (Node 20 `pnpm build` + Rust 1 builder + minimal Debian runtime, backup `Dockerfile.py`).
3. **Wave 1B (`ui/src/*`, `tests/e2e/e2e_scorecard.py`, `VERSION`, docs)**:
   - Update `ui/src/routes/keys.tsx`, login/header session bar in React SPA (`corepack pnpm build` -> `ui/dist`) for SSO + one-time key reveal + Zero-F12-Leak.
   - Author `tests/e2e/e2e_scorecard.py` with synthetic Refero/Prompt-UI/Components fixture corpus verifying all corpus routes, dynamic SVG previews, all 8 `/mcp` tools, OIDC PKCE flow, Zero-F12-Leak (`GET /api/keys` contains NO plaintext secrets), `429`/CSRF/traversal defenses, and UI screenshots.
