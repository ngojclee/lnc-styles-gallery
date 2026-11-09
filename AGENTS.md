# AGENTS.md — `lnc-styles-gallery` (LNC Style Library)

## 1. Architecture Overview (Shape A Blueprint)

`lnc-styles-gallery` is an internal LNC application modeled on `lnc-pose-gallery` (Shape A: single FastAPI container serving REST API, Streamable HTTP MCP server, and React + Vite + TanStack Router + shadcn/ui SPA frontend ported from `pose.lengoc.me`).

- **Service Name**: `lnc-styles-gallery`
- **Target Domain**: `styles.lengoc.me`
- **Ports**: Container internal `8000` → Host port `8460`
- **Stack**: Python 3.11 + FastAPI + Uvicorn + FastMCP / JSON-RPC 2.0 Streamable HTTP + React 19 + Vite 8 + TanStack Router + TanStack Query + shadcn/ui + Tailwind v4 (`ui/` → `ui/dist/`)

```
lnc-styles-gallery/
├── app/
│   ├── __init__.py
│   ├── config.py          # Env settings, NAS/local data path resolution, secret masking
│   ├── store.py           # Thread-safe in-memory index, preview/shots lookup, full-text search, re-indexer
│   ├── mcp_server.py      # /mcp Streamable HTTP endpoint + FastMCP tool registration (styles__*)
│   └── main.py            # FastAPI routes (/api/*, /mcp, /admin, SPA static mount for ui/dist)
├── ui/                    # React + Vite + TanStack Router + shadcn/ui SPA (ported from pose.lengoc.me)
│   ├── src/
│   │   ├── api/           # Typed REST API client & TanStack Query options
│   │   ├── components/
│   │   │   ├── common/    # PageHeader, StatCard, EmptyState, ImageViewer, ZoomViewport, StyleDetailDialog
│   │   │   ├── layout/    # AppSidebar, AppHeader, nav
│   │   │   ├── theme/     # ThemeProvider, ThemeToggle
│   │   │   └── ui/        # shadcn/ui primitives
│   │   ├── routes/        # __root, index (/), styles, collections, templates, platforms, components, licenses, admin
│   │   ├── types/         # TypeScript interfaces for styles, shots, collections, templates, licenses
│   │   ├── index.css      # Exact pose.lengoc.me dark/light theme tokens
│   │   ├── main.tsx       # React + Router + QueryClient entry
│   │   └── routeTree.gen.ts
│   ├── index.html
│   ├── package.json
│   ├── pnpm-lock.yaml
│   └── vite.config.ts
├── ui-vanilla-backup/     # Preserved v0.1.0 vanilla JS/CSS UI backup
├── infra/
│   └── docker-compose.yml # Portainer / Docker Compose stack (8460:8000, read-only data mount)
├── scripts/
│   ├── smoke_test.py      # End-to-end smoke test for /api/*, previews/shots, SPA routes, and /mcp
│   └── sync_data_from_nas.bat # Robocopy sync from NAS _agentsync/repos into ./data
├── Dockerfile             # Multi-stage: node:20-alpine (pnpm build) -> python:3.11-slim runtime
├── requirements.txt
├── VERSION                # 0.3.0
└── README.md
```

---

## 2. Data Layout & Hot Re-Indexing

The backend reads three read-only corpora. By default it checks `./data/` first and falls back to the NAS UNC paths if `./data/` has not been populated yet:

1. **Refero Styles Corpus** (`data/refero-styles` or `\\10.21.2.2\data\_agentsync\repos\refero-styles`)
   - `index.json` — Array of catalog records: `{id, url, title, tagline, theme ("light"|"dark"), source_url}`
   - `styles/<uuid>/DESIGN.md` — ~20KB Markdown design system guide (color tokens, typography, spacing, border radius, layout, do/don't rules)
   - `styles/<uuid>/meta.json` — Additional per-style metadata (including optional `"preview"` and `"shots"` arrays)
   - `styles/<uuid>/preview.<ext>` — Hero screenshot (`og:image` of Refero page)
   - `styles/<uuid>/shots/NN.<ext>` — Up to 8 gallery screenshots per style (**images only — video shots deliberately skipped**; `harvest_images.py` drops `mp4/webm/mov` via `VIDEO_EXTS` to keep the corpus lightweight)
   - `styles/<special-id>/` — Non-UUID entries: 6 `lnc-*`/`ln-*` LNC-owned themes + 15 official **Platform Design Systems** (`apple-hig`, `material-design-3`, `microsoft-fluent-2`, `ibm-carbon`, `shopify-polaris`, `atlassian-design`, `github-primer`, `ant-design`, `salesforce-lightning`, `uswds`, `govuk-design`, `shadcn-ui`, `radix-themes`, `tailwind-default`, `bootstrap-5`) whose `meta.json` declares `"categories": ["platform-system"]` (takes precedence over keyword classification)
   - `sitemaps/collections.xml` — Optional sitemap collections metadata (when present)

2. **Prompt-Driven UI Corpus** (`data/prompt-driven-ui` or `\\10.21.2.2\data\_agentsync\repos\prompt-driven-ui`)
   - `websites-en/` — 50 English single-file HTML sample designs
   - `websites/` — 50 Vietnamese single-file HTML sample designs
   - `thumbs/` — Preview thumbnails for each template
   - `prompts-en/` & `prompts/` — Source generation prompts for each template

3. **LNC Components Corpus** (`data/lnc-components` or `\\10.21.2.2\data\_agentsync\repos\lnc-components`) — 199 items
   - `index.json` — Array of component records: `{id, name, category, kind ("html"|"tsx"), description, tags, license, source_url, files[]}`
   - `items/<id>/` — One directory per component
     - LNC-authored effects (49): `index.html` self-contained demo (live iframe preview) + `meta.json` — license "LNC Internal"
     - Knowns UI kit (150): `meta.json` + `registry/.../*.tsx` upstream source files — MIT (Knowns Lab), `registry_url` preserved for re-harvest
   - Categories: `knowns-ui`, `backgrounds`, `buttons`, `ai-agent`, `cards`, `inputs-toggles`, `loaders`, `text-effects`, `marketing`
   - Generated/harvested by `temp_scripts/gen_components.py` and `temp_scripts/harvest_knowns.py`

### Harvest Resilience & Re-Index
Because the Refero harvest script (`harvest_images.py`) continues adding styles and images progressively:
- Missing `DESIGN.md`, `meta.json`, `preview.*`, or `shots/*` files are handled gracefully without failing index load.
- At index time, `store.py` checks for `styles/<uuid>/preview.*` and caches `"has_preview"` + `"preview_url": "/api/styles/<uuid>/preview"` (when present on disk).
- `GET /api/styles/{id}/preview` serves the real preview image via `FileResponse`, or dynamically renders a theme-aware SVG preview card fallback if the image has not been harvested yet.
- `POST /api/admin/reindex` (and MCP tool `styles__reindex`) re-scans the corpus directories in memory with zero downtime.

---

## 3. REST API Contract (`/api/*`)

- `GET /health` & `GET /api/health` — Service health & index counts (`styles_count`, `designs_count`, `previews_count`, `collections_count`, `templates_count`)
- `GET /api/styles` — Filterable catalog (`q`, `theme`, `category`, `collection`, `has_design`, `limit`, `offset`); each item includes `has_preview` and `preview_url`
- `GET /api/styles/{style_id}` — Full style record including `design_md` (`DESIGN.md` content), `meta`, `has_preview`, `preview_url`, and `shots`
- `GET /api/styles/{style_id}/preview` — `FileResponse` of `styles/<id>/preview.*` (or SVG fallback card if not yet harvested)
- `GET /api/styles/{style_id}/shots` — JSON list `[{n, url, filename}]` of existing gallery screenshots
- `GET /api/styles/{style_id}/shots/{n}` — `FileResponse` of shot `#n` under `styles/<id>/shots/`
- `GET /api/search?q=` — Full-text search across style titles, taglines, `DESIGN.md` body snippets, and templates
- `GET /api/collections` — Grouped style sets by category/mood, theme, and optional sitemap groups
- `GET /api/collections/{collection_id}` — Detailed collection with matching styles
- `GET /api/templates` — List of 100 prompt-driven-ui templates (`lang`, `q`, `limit`, `offset`)
- `GET /api/templates/{template_id}` — Template metadata, generation prompt, and preview URLs
- `GET /api/templates/{template_id}/html` — Serves the raw template HTML (`text/html`) read-only
- `GET /api/templates/{template_id}/thumb` — Serves the template thumbnail image (or SVG fallback)
- `GET /api/components` — Component catalog (`category`, `kind`, `q`, `limit`, `offset`)
- `GET /api/components/{component_id}` — Component metadata; `?include_source=true` adds `sources` dict (file contents)
- `GET /api/components/{component_id}/html` — Self-contained HTML demo (`text/html`) for iframe preview
- `GET /api/components/{component_id}/files/{path}` — Raw source file download (e.g. TSX)
- `GET /api/licenses` — Structured license and source attribution manifest (4 corpora incl. update/re-harvest URLs)
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
- `styles__list_components` — List/filter components by `category`, `kind`, `limit`, `offset`
- `styles__get_component` — Fetch component metadata + source file contents
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
3. **LNC UI Effects & Components (`lnc-components`)**
   - 49 self-contained HTML/CSS effect demos authored in-house — **LNC Internal**, no third-party code.
4. **Knowns UI (`knowns-dev/knowns-ui`, ui.knowns.dev)**
   - 150 AI-product component TSX sources harvested from the public shadcn registry (`/r/registry.json`), licensed under **MIT** (third-party logos/trademarks excluded from the MIT grant). Each item preserves `source_url` (docs page) and `registry_url` for re-harvest via `temp_scripts/harvest_knowns.py`.
