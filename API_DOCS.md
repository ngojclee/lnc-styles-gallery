# API & MCP Documentation — `lnc-styles-gallery` (v1.0.0.1)

Base URL (local): `http://localhost:8460`
Base URL (production): `https://styles.lengoc.me`

---

## REST API Endpoints

### `GET /health` | `GET /api/health`
Returns service health and current indexed item counts (`styles_count`, `designs_count`, `previews_count`, `collections_count`, `templates_count`).

### `GET /api/styles`
Query parameters:
- `q` (string, optional): Filter by title, tagline, ID, or source URL.
- `theme` (`light` | `dark`, optional): Filter by color theme.
- `category` / `collection` (string, optional): Filter by collection slug or category name.
- `has_design` (bool, optional): Only return styles that have a `DESIGN.md` file on disk.
- `limit` (int, default `48`, max `500`): Page size.
- `offset` (int, default `0`): Pagination offset.

Each item in `items` includes:
- `id`, `title`, `tagline`, `theme`, `url`, `source_url`, `category`, `category_label`
- `has_design` (bool), `has_meta` (bool)
- `has_preview` (bool): Whether `styles/<id>/preview.<ext>` exists on disk
- `preview_url` (string | null): `"/api/styles/<id>/preview"` when `has_preview` is `true`

### `GET /api/styles/{style_id}`
Returns full metadata for a single style, including:
- `id`, `title`, `tagline`, `theme`, `url`, `source_url`, `category`, `category_label`
- `has_preview`, `preview_url`
- `shots`: Array of `{n, url, filename}` for harvested gallery screenshots
- `design_md`: Full Markdown contents of `styles/<id>/DESIGN.md`
- `meta`: Parsed JSON contents of `styles/<id>/meta.json`

### `GET /api/styles/{style_id}/preview`
- Serves `styles/<style_id>/preview.<ext>` via `FileResponse` when present on disk.
- Falls back to a dynamic theme-aware SVG preview card (`image/svg+xml`, header `X-Preview-Fallback: true`) if `harvest_images.py` has not reached the style yet.
- Returns `404` if `style_id` does not exist.

### `GET /api/styles/{style_id}/shots`
- Returns a JSON list `[{"n": 1, "url": "/api/styles/<id>/shots/1", "filename": "01.jpg"}, ...]` of gallery screenshots under `styles/<style_id>/shots/`.

### `GET /api/styles/{style_id}/shots/{n}`
- Serves shot `#n` (`styles/<style_id>/shots/NN.<ext>`) via `FileResponse`, or `404` if not found.

### `GET /api/search?q=<query>`
Full-text search across styles (`title`, `tagline`, `source_url`, and `DESIGN.md` content) and HTML templates.
Query parameters:
- `q` (string, required): Search query.
- `theme` (`light` | `dark`, optional)
- `category` (string, optional)
- `include_body` / `include_content` (bool, default `true`): Search inside `DESIGN.md` body text and return matching snippets.
- `limit` (int, default `40`, max `200`)

### `GET /api/collections`
Returns grouped browsable style collections derived from theme, mood/category keywords, and optional `sitemaps/collections.xml`.

### `GET /api/collections/{collection_id}`
Returns collection details and paginated styles belonging to that collection.

### `GET /api/templates`
Lists the 100 `prompt-driven-ui` HTML sample templates (`websites-en/` and `websites/`).
Query parameters:
- `lang` (`en` | `vi` | `all`, default `all`)
- `q` (string, optional): Search by title, slug, or prompt text.
- `limit` (int, default `100`)
- `offset` (int, default `0`)

### `GET /api/templates/{template_id}`
Returns full metadata and generation prompt for a template.
- `GET /api/templates/{template_id}/html`: Serves the raw HTML template (`text/html; charset=utf-8`) read-only.
- `GET /api/templates/{template_id}/thumb`: Serves the thumbnail image (or SVG preview card fallback).

### `GET /api/licenses`
Returns attribution metadata for Refero Styles (`styles.refero.design`), per-style `source_url` product websites, and `vuhung16au/prompt-driven-ui` (MIT).

### `POST /api/admin/reindex`
Re-scans the Refero Styles and Prompt-Driven UI corpus directories (including newly harvested preview/shot images) in memory without restarting the server.

### Auth, Sessions & Security Controls (`/api/auth/*`)
- `GET /api/auth/session` — Returns active session metadata (`authenticated`, `mode`, `user`, `csrf_token`, `session_epoch`, `expires_at`, `idle_expires_at`).
- `GET /api/auth/oidc/login` & `GET /api/auth/oidc/callback` — Keycloak OIDC Authorization Code flow with PKCE `S256`, `state`, `nonce`, and allowlist checks.
- `POST /api/auth/login` — Break-glass management key login (Argon2id verified, IP rate-limited).
- `POST /api/auth/logout` — Local-only session logout requiring valid `X-CSRF-Token` + strict `Origin`/`Sec-Fetch-Site` host match (`X-Forwarded-Host` ignored). `GET /api/auth/logout` returns `405 Method Not Allowed`.
- `POST /api/auth/sessions/revoke-all` — Increments `session_epoch` to invalidate all active browser cookies (`X-CSRF-Token` required).
- Query-string secret tokens (`?token=`, `?key=`, `?api_key=`) are rejected globally with `400 Bad Request`.
- Sensitive endpoints (`/login.html`, `/login.js`, `/api/auth/*`, `/api/keys*`, `/api/admin/*`, `/api/mcp-connectors*`) set `Cache-Control: no-store, private, max-age=0`.

### Admin API Keys & Vault Copy (`/api/admin/api-keys*` & `/api/keys*`)
- `GET /api/admin/api-keys` (alias `GET /api/keys`) — Masked list (`prefix...last4`, `scopes`, `copy_available`, never returns plaintext secret).
- `POST /api/admin/api-keys` (alias `POST /api/keys`) — Issue SHA-256 hashed key + HKDF AES-256-GCM vault envelope.
- `PATCH /api/admin/api-keys/{id}` (alias `PATCH /api/keys/{id}`) — Update label/scopes without rotating secret.
- `POST /api/admin/api-keys/{id}/copy` (aliases `POST /api/admin/keys/{id}/copy`, `POST /api/keys/{id}/copy`) — Rate-limited (`15/5m/IP`), CSRF-protected decrypt & copy with `secret.copy` `audit.jsonl` record (`410 Gone` for legacy hash-only keys lacking a vault envelope).
- `POST /api/admin/api-keys/{id}/rotate` (alias `POST /api/keys/{id}/rotate`) — Generate new key, re-seal vault envelope, invalidate previous secret.
- `POST /api/admin/api-keys/{id}/revoke` (alias `POST /api/keys/{id}/revoke`) — Enable/revoke key.
- `DELETE /api/admin/api-keys/{id}` (alias `DELETE /api/keys/{id}`) — Permanently remove active or revoked key and purge vault envelope.

### Inbound MCP Connectors (`/api/admin/mcp-connectors*` & `/api/mcp-connectors*`)
- `GET /api/admin/mcp-connectors` — Inbound `/mcp` posture, audience, max scopes, exposed tools (`8`), and connector list (`prefix...last4` masked).
- `POST /api/admin/mcp-connectors` — Provision audience-bound inbound connector credential + HKDF vault envelope.
- `PATCH /api/admin/mcp-connectors/{id}` — Update connector metadata or optional replacement secret (blank leaves existing secret unchanged).
- `POST /api/admin/mcp-connectors/{id}/copy` — Audited (`secret.copy`), rate-limited (`15/5m/IP`), CSRF-protected decrypt & copy.
- `POST /api/admin/mcp-connectors/{id}/rotate`, `/revoke`, `/test`, `DELETE` — Rotate, revoke/enable, live self-probe against `POST /mcp`, or delete (Env-managed connectors lock mutations while permitting `/test`).

---

## MCP Endpoint (`/mcp`)

Protocol: **MCP Streamable HTTP** (`POST /mcp` JSON-RPC 2.0, `Authorization: Bearer <token>` or `X-API-Key: <token>`).

Tools (`styles__*` namespace, 8 tools):
1. `styles__list_styles(theme?, category?, limit?, offset?)`
2. `styles__search(query, theme?, category?, include_content?, limit?)`
3. `styles__get_design(style_id)`
4. `styles__list_collections()`
5. `styles__get_template(template_id, include_html?)`
6. `styles__list_components(category?, kind?, limit?, offset?)`
7. `styles__get_component(component_id, include_source?)`
8. `styles__reindex()`
