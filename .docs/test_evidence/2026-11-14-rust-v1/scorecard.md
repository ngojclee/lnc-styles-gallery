# E2E & Migration Parity Scorecard — `lnc-styles-gallery` (`v1.0.0.1`)

- **Track**: Track M (Python/FastAPI -> Rust/Axum Migration + Security Hardening)
- **Final Weighted Score**: **100.0 / 100**
- **Hard-Fail Gate**: **CLEAR** (`Critical=0`, `High=0`, `Medium=0`, `Low=0`)
- **Release Decision**: **RELEASE_READY (SHIP / CUTOVER)**
- **Runtime Metrics**: Binary `7,356,928` bytes | Boot `521.21 ms` | Idle RSS `8.49 MB` | Post-Load RSS `8.9 MB` | Latency `p50=1.4 ms`, `p95=22.36 ms`

## Weighted Category Breakdown

| Category | Weight | Checks Passed | Score |
| :--- | :---: | :---: | :---: |
| Core User Journeys & Corpus REST Parity | 25% | `5/5` | **25.0 / 25** |
| Security, Crypto, OIDC & Zero-F12-Leak Parity | 30% | `9/9` | **30.0 / 30** |
| MCP Streamable HTTP JSON-RPC 2.0 Parity | 20% | `3/3` | **20.0 / 20** |
| Failure Recovery & Read-Only Corpus Resilience | 15% | `2/2` | **15.0 / 15** |
| Build, Startup, Latency & Footprint | 10% | `4/4` | **10.0 / 10** |

## Detailed Check Results

| ID | Category | Severity | Status | Check | Evidence |
| :--- | :--- | :---: | :---: | :--- | :--- |
| `H01` | `health` | Critical | **PASS** | Compiled release binary & built React SPA bundle present | binary=7,356,928 bytes, ui/dist/index.html=True |
| `S01` | `security` | High | **PASS** | Compiled SPA bundle contains zero `raw_key` references | raw_key_in_dist=False, sample_bundle=/static/admin-BXVMU7f3.js |
| `H02` | `health` | Critical | **PASS** | Server boots cleanly on ephemeral PORT and serves /health | boot_ms=521.21ms, idle_rss=8.49MB |
| `H03` | `health` | Medium | **PASS** | Built-in `--healthcheck` subcommand exits 0 | exit_code=0 |
| `H04` | `health` | High | **PASS** | Version 1.0.0.1, corpus counts, p95 latency (<50ms) & RSS (<80MB) | v=1.0.0.1, styles=3, p50=1.4ms, p95=22.36ms, rss=8.9MB |
| `J01` | `journeys` | High | **PASS** | Styles list, filter, detail, preview SVG, fallback SVG & shots | dark_total=2, fallback_svg_bytes=2307 |
| `J02` | `journeys` | High | **PASS** | Full-text search (styles + templates + components) & collections (incl. sitemap) | collections=15, lnc_style_count=1 |
| `J03` | `journeys` | High | **PASS** | Templates EN/VI list, detail, iframe HTML, fallback thumb, prompt & sibling CSS | vi_total=1, html_bytes=209, css_bytes=45 |
| `J04` | `journeys` | High | **PASS** | Components catalog (HTML + TSX), inline sources & file serving | components_total=2 |
| `J05` | `journeys` | Medium | **PASS** | Licenses (4 corpora), /api/mcp/info, SPA HTML shell, hashed JS & missing-asset 404 | corpora=4, mcp_tools=8 |
| `S02` | `security` | Critical | **PASS** | Legacy `keys.json` & `MANAGEMENT_KEY` scrubbed on disk (HMAC + Argon2id only) | keys.json holds salted HMAC digest; config.json holds Argon2id PHC hash |
| `S03` | `security` | Critical | **PASS** | Unauthenticated requests blocked (302 on /keys & /admin; 401 on /api/keys, /api/admin/*, /mcp) | /keys=302, /admin=302, /api/keys=401, /mcp=401 |
| `S04` | `security` | High | **PASS** | /login.html OIDC SSO primary + collapsed `<details>` break-glass + strict headers/CSP | xfo=DENY, nosniff=nosniff |
| `S05` | `security` | Critical | **PASS** | OIDC PKCE S256 + RS256 JWKS + nonce + allowlist + state tamper rejection | oidc_user=kc-louis, deny_loc=/login.html?oidc_error=not_allowed&error=not_allowed |
| `S06` | `security` | Critical | **PASS** | Zero-F12-Leak vault key & MCP connector lifecycle (masked GET, HKDF POST /copy + 15/5m 429, 405 GET /copy, legacy 410, rotate/revoke/delete) | copy=(200,200,429=True), legacy410=True, conn=(200,200,200,200) |
| `S07` | `security` | High | **PASS** | CSRF Origin+Token guard (403/405), ?token= 400 block, SSRF metadata block (400) & revoke-all session_epoch invalidation | csrf=403, logout=(405,403,403,200), q=(400,400,400), ssrf=(400,400), post_revoke_all=401 |
| `S08` | `security` | High | **PASS** | Break-glass Argon2id login succeeds & 9th failed attempt returns 429 + Retry-After | mgmt_login=200, 9th_status=429, retry_after=299s |
| `S09` | `security` | High | **PASS** | Path traversal probes rejected across styles, templates, components & static routes | probe_statuses=[400, 400, 400, 400, 400] |
| `M01` | `mcp` | Critical | **PASS** | MCP `initialize`, `notifications/initialized` (HTTP 202) & `tools/list` (all 8 tools) | notif=202, tools=8 (list_styles,search,get_design,list_collections,get_template,list_components,get_component,reindex) |
| `M02` | `mcp` | Critical | **PASS** | All 8 MCP tools execute cleanly (both unprefixed and `styles__*` prefixed) | tools_passed=8/8 |
| `M03` | `mcp` | Medium | **PASS** | JSON-RPC `-32601` on unknown method & `isError: true` on unknown tool | method_err=-32601, tool_isError=True |
| `R01` | `resilience` | Medium | **PASS** | Graceful error handling (malformed JSON, missing entities) & hot `/api/admin/reindex` | bad_json=400, 404s=(404,404,404), reindex=200 |
| `R02` | `resilience` | High | **PASS** | Read-only `${STYLES_DATA_DIR}` corpus completely unmutated across boot, queries & reindex | files_tracked=34, mutated=False |
