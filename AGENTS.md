# AGENTS.md — `lnc-styles-gallery` (LNC Style Library)

## 1. Architecture Overview (Shape A Blueprint)

`lnc-styles-gallery` is an internal LNC application modeled on `lnc-pose-gallery` (Shape A: single FastAPI container serving REST API, Streamable HTTP MCP server, and static SPA frontend).

- **Service Name**: `lnc-styles-gallery`
- **Target Domain**: `styles.lengoc.me`
- **Ports**: Container internal `8000` → Host port `8460`
- **Stack**: Python 3.11 + FastAPI + Uvicorn + FastMCP / JSON-RPC 2.0 Streamable HTTP + Vanilla JS/CSS SPA (`ui/`)

```
lnc-styles-gallery/
├── app/
│   ├── __init__.py
│   ├── config.py          # Env settings, NAS/local data path resolution, secret masking
│   ├── store.py           # Thread-safe in-memory index, category taxonomy, full-text search, re-indexer
│   ├── mcp_server.py      # /mcp Streamable HTTP endpoint + FastMCP tool registration (styles__*)
│   └── main.py            # FastAPI routes (/api/*, /mcp, /admin, static UI mount)
├── ui/
│   ├── index.html         # Gallery UI (#styles, #collections, #templates, #licenses)
│   ├── admin.html         # Internal admin dashboard (/admin) for corpus status, reindex, settings
│   ├── styles.css         # Dark gallery theme styled after pose.lengoc.me
│   └── app.js             # Client logic, Markdown renderer, modal viewers, search/filter state
├── infra/
│   └── docker-compose.yml # Portainer / Docker Compose stack (8460:8000, read-only data mount)
├── scripts/
│   └── sync_data_from_nas.bat # Robocopy sync from NAS _agentsync/repos into ./data
├── Dockerfile
├── requirements.txt
├── VERSION                # 0.1.0
└── README.md
```

---

## 2. Data Layout & Hot Re-Indexing

The backend reads two read-only corpora. By default it checks `./data/` first and falls back to the NAS UNC paths if `./data/` has not been populated yet:

1. **Refero Styles Corpus** (`data/refero-styles` or `\\10.21.2.2\data\_agentsync\repos\refero-styles`)
   - `index.json` — Array of catalog records: `{id, url, title, tagline, theme ("light"|"dark"), source_url}`
   - `styles/<uuid>/DESIGN.md` — ~20KB Markdown design system guide (color tokens, typography, spacing, border radius, layout, do/don't rules)
   - `styles/<uuid>/meta.json` — Additional per-style metadata
   - `sitemaps/collections.xml` — Optional sitemap collections metadata (when present)

2. **Prompt-Driven UI Corpus** (`data/prompt-driven-ui` or `\\10.21.2.2\data\_agentsync\repos\prompt-driven-ui`)
   - `websites-en/` — 50 English single-file HTML sample designs
   - `websites/` — 50 Vietnamese single-file HTML sample designs
   - `thumbs/` — Preview thumbnails for each template
   - `prompts-en/` & `prompts/` — Source generation prompts for each template

### Harvest Resilience & Re-Index
Because the Refero harvest script may continue adding styles over time:
- Missing `DESIGN.md` or `meta.json` files are handled gracefully without failing index load.
- Extra `styles/<uuid>` folders not yet listed in `index.json` are discovered and merged automatically.
- `POST /api/admin/reindex` (and MCP tool `styles__reindex`) re-scans the corpus directories in memory with zero downtime.

---

## 3. REST API Contract (`/api/*`)

- `GET /health` & `GET /api/health` — Service health & index counts
- `GET /api/styles` — Filterable catalog (`q`, `theme`, `category`, `collection`, `has_design`, `limit`, `offset`)
- `GET /api/styles/{style_id}` — Full style record including `design_md` (`DESIGN.md` content) and `meta`
- `GET /api/search?q=` — Full-text search across style titles, taglines, `DESIGN.md` body snippets, and templates
- `GET /api/collections` — Grouped style sets by category/mood, theme, and optional sitemap groups
- `GET /api/collections/{collection_id}` — Detailed collection with matching styles
- `GET /api/templates` — List of 100 prompt-driven-ui templates (`lang`, `q`, `limit`, `offset`)
- `GET /api/templates/{template_id}` — Template metadata, generation prompt, and preview URLs
- `GET /api/templates/{template_id}/html` — Serves the raw template HTML (`text/html`) read-only
- `GET /api/templates/{template_id}/thumb` — Serves the template thumbnail image (or SVG fallback)
- `GET /api/licenses` — Structured license and source attribution manifest
- `POST /api/admin/reindex` — Re-scan corpus directories on demand
- `GET /api/admin/status` & `GET /api/admin/settings` — Corpus diagnostics and masked configuration

---

## 4. MCP Tools (`/mcp` — Streamable HTTP)

Namespace: `styles__*`
- `styles__list_styles` — List/filter styles by `theme`, `category`, `limit`, `offset`
- `styles__search` — Search styles (and optionally `DESIGN.md` content) by keyword query
- `styles__get_design` — Fetch full `DESIGN.md` and metadata for a style ID
- `styles__list_collections` — List all derived and sitemap style collections with counts
- `styles__get_template` — Fetch prompt-driven-ui template metadata, prompt, and optional HTML source
- `styles__reindex` — Trigger a live corpus re-scan

---

## 5. Auth & House Conventions

- Internal LNC network service (`styles.lengoc.me`).
- Read endpoints (`/api/styles`, `/api/collections`, `/api/templates`, `/api/search`, `/api/licenses`, `/mcp`) are accessible without auth on the internal network; if `STYLES_API_KEY` is configured, `/api/admin/*` mutations verify `X-API-Key` or `Authorization: Bearer`.
- All secret/key values in `/api/admin/settings` are masked (`***`).

---

## 6. License & Source Attributions

1. **Refero Styles (`styles.refero.design`)**
   - Distilled design system guides (`DESIGN.md`) curated from real-world product websites.
   - Each style record preserves its original `url` (Refero page) and `source_url` (attributed product website).
2. **Prompt-Driven UI (`vuhung16au/prompt-driven-ui`)**
   - 100 HTML/CSS/JS sample templates and prompts (50 EN + 50 VI), licensed under the **MIT License**.
