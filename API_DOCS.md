# API & MCP Documentation — `lnc-styles-gallery`

Base URL (local): `http://localhost:8460`
Base URL (production): `https://styles.lengoc.me`

---

## REST API Endpoints

### `GET /health` | `GET /api/health`
Returns service health and current indexed item counts.

### `GET /api/styles`
Query parameters:
- `q` (string, optional): Filter by title, tagline, ID, or source URL.
- `theme` (`light` | `dark`, optional): Filter by color theme.
- `category` / `collection` (string, optional): Filter by collection slug or category name.
- `has_design` (bool, optional): Only return styles that have a `DESIGN.md` file on disk.
- `limit` (int, default `48`, max `500`): Page size.
- `offset` (int, default `0`): Pagination offset.

### `GET /api/styles/{style_id}`
Returns full metadata for a single style, including:
- `id`, `title`, `tagline`, `theme`, `url`, `source_url`, `categories`
- `design_md`: Full Markdown contents of `styles/<id>/DESIGN.md`
- `meta`: Parsed JSON contents of `styles/<id>/meta.json`

### `GET /api/search?q=<query>`
Full-text search across styles (`title`, `tagline`, `source_url`, and `DESIGN.md` content) and HTML templates.
Query parameters:
- `q` (string, required): Search query.
- `theme` (`light` | `dark`, optional)
- `category` (string, optional)
- `include_content` (bool, default `true`): Search inside `DESIGN.md` body text and return matching snippets.
- `limit` (int, default `30`, max `200`)

### `GET /api/collections`
Returns grouped browsable style collections derived from theme, mood/category keywords (`Editorial`, `Terminal / Dev-Tool`, `Luxury`, `Dark Hardware`, `Warm Minimal`, `Playful`, `AI & SaaS`, `Fintech & Crypto`, `Portfolio & Creative`, `E-Commerce & Consumer`), and optional `sitemaps/collections.xml`.

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
Returns full metadata and generation prompt for a template (`include_html=true` optionally embeds raw HTML).
- `GET /api/templates/{template_id}/html`: Serves the raw HTML template (`text/html; charset=utf-8`) read-only.
- `GET /api/templates/{template_id}/thumb`: Serves the thumbnail image (or SVG preview card fallback).

### `GET /api/licenses`
Returns attribution metadata for Refero Styles (`styles.refero.design`), per-style `source_url` product websites, and `vuhung16au/prompt-driven-ui` (MIT).

### `POST /api/admin/reindex`
Re-scans the Refero Styles and Prompt-Driven UI corpus directories in memory without restarting the server.

---

## MCP Endpoint (`/mcp`)

Protocol: **MCP Streamable HTTP** (`POST /mcp` JSON-RPC 2.0).

Tools (`styles__*` namespace):
1. `styles__list_styles(theme?, category?, limit?, offset?)`
2. `styles__search(query, theme?, category?, include_content?, limit?)`
3. `styles__get_design(style_id)`
4. `styles__list_collections()`
5. `styles__get_template(template_id, include_html?)`
6. `styles__reindex()`
