# Live Cutover & `/w18` Verification — `lnc-styles-gallery` (`v1.0.0.0`)

- **Cutover Timestamp**: `2026-11-14T06:07:25Z`
- **Production URL**: `https://styles.lengoc.me` (`10.21.1.101:8460`)
- **Portainer Stack**: `portainer-fr` Stack `53` (upgraded from legacy standalone Stack `50` to GitOps repo-stack tracking `https://gitea.lengoc.me/apps/lnc-styles-gallery`)
- **Container**: `4ebde1ee7e83` (`running (healthy)`, `uid:gid 10001:10001`, compose labels intact)
- **Rollback**: Tags `lnc-styles-gallery:rollback-20261114` & `lnc-styles-gallery-rollback:20261114`; state backup `/home/Docker/lnc-styles-gallery-config.bak-20261114`

## Before -> After Production Metrics

| Metric | Before (Python) | After (Rust `v1.0.0.0`) | Delta |
| :--- | :---: | :---: | :---: |
| Live Container `VmRSS` | `20.1 MiB` | **`15.62 MiB`** | **-22.3%** |
| Docker Image Size | `238.4 MB` | **`91.9 MB`** | **-61.5%** |

*(Live Rust process holds 1,363 styles, 100 templates, and 199 components indexed in memory in `514 ms`.)*

## Live Gate Verification Results

| Check | Outcome | Evidence |
| :--- | :---: | :--- |
| Container health & non-root `10001:10001` | **PASS** | `running (healthy)`, `user=10001:10001`, `compose_project=lnc-styles-gallery` |
| `GET /health` & security headers | **PASS** | `HTTP 200`, `version=1.0.0.0`, `styles_count=1363`, `warnings=[]`, CSP/XFO/nosniff/Referrer-Policy set |
| `/login.html` & `/api/auth/status` | **PASS** | `HTTP 200`, `oidc_enabled=true`, SSO primary button active |
| Keycloak OIDC PKCE `S256` handshake | **PASS** | `GET /api/auth/oidc/login -> 302` (`code_challenge_method=S256`, `HttpOnly; SameSite=Lax; Secure` tx cookie) -> Keycloak `200` |
| Auth isolation (`/api/keys`, `/api/admin/reindex`, `/mcp`) | **PASS** | Unauthenticated `401` on all protected & MCP endpoints |
| Business MCP Hub live E2E | **PASS** | `hub_cache_refresh` (`styles: ok, 8 tools`) & `styles__list_styles` (`total: 1363`) |
| Legacy `/config/keys.json` Vault migration | **PASS** | `625 B` scrubbed to salted HMAC (`prefix`/`last4` only, zero plaintext keys) |
