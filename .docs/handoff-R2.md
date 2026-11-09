# R2 — Port the exact pose.lengoc.me UI onto lnc-styles-gallery + image previews

## Mission

The owner wants styles.lengoc.me to look and feel EXACTLY like pose.lengoc.me
(same shell, same components, same dark theme) and for each style card to show
a real visual preview image (like the original styles.refero.design site).

Two workstreams in this repo `D:\Python\projects\lnc-styles-gallery`:

## 1. UI port — copy pose's React app, swap pages only

Source of truth: `./_pose-ui-ref/` (already vendored into this workdir —
React + Vite + TanStack Router + shadcn/ui + Tailwind, ~70 files; this is
pose.lengoc.me's UI source). You CANNOT read paths outside this workdir —
use `./_pose-ui-ref/` only. Copy its structure to build the new `ui/`
(replacing the current vanilla `ui/` — move vanilla files to
`ui-vanilla-backup/` first, do not delete).

KEEP (these carry the look): `components/layout/*` (AppSidebar, AppHeader,
nav), `components/theme/*`, `components/common/*` (PageHeader, ImageViewer,
EmptyState, StatCard, dialogs), `components/ui/*` (shadcn), `index.css`,
`main.tsx`, `routeTree.gen.ts` (regenerate), notifications.css.

REMOVE pose-only domains: `components/hand-editor/*`, `components/studio/*`,
`components/layout/RecipeDock.tsx`, pose pages/routes, pose API client code
(replace with our `/api/*` calls), anything importing ComfyUI/pose metadata.

WRITE new pages (same layout primitives, same card/dialog patterns pose uses):

- `#/` or `#/styles` (default) — style grid: card = preview image
  (`/api/styles/{id}/preview`) + title + tagline + theme badge (light/dark);
  search input + theme/category filters; click → detail modal/dialog showing
  preview hero + image strip (shots) + DESIGN.md rendered (Rendered | Raw |
  Meta | Copy buttons like before) + source_url link.
- `#/collections` — collection cards (name, count, mood) → filtered styles.
- `#/templates` — prompt-driven-ui template grid with `/api/templates/{id}/thumb`
  thumbnails + iframe HTML preview dialog (`/api/templates/{id}/html`).
- `#/licenses` — attribution list from `/api/licenses`.
- `/admin` — corpus status + POST `/api/admin/reindex` button + settings.

Match pose's page anatomy: left sidebar nav with section icons, AppHeader,
theme toggle, PageHeader per page, stat cards, empty states, loading
skeletons — i.e. reuse the SAME components, don't restyle.

## 2. API additions (FastAPI)

Corpus now contains images (`harvest_images.py` is running in background —
files appear progressively under `styles/<uuid>/`):
  - `preview.<ext>` — hero screenshot (og:image of Refero page)
  - `shots/NN.<ext>` — up to 8 gallery screenshots
  - `meta.json` gets `"preview": "preview.jpg"` + `"shots": [...]`

Add to `app/store.py` + `app/main.py`:

- `GET /api/styles/{id}/preview` → FileResponse of `styles/<id>/preview.*`
  (glob any ext; 404/fallback SVG if missing — mirror the template thumb
  SVG fallback pattern already in main.py).
- `GET /api/styles/{id}/shots` → JSON list `{n, url}` of existing shots.
- `GET /api/styles/{id}/shots/{n}` → FileResponse of the shot file.
- Include `"preview_url": "/api/styles/<id>/preview"` (only when the file
  exists — or let client 404-handle; prefer checking at index time and
  caching) in `list_styles` items and style detail.
- Keep `/api/templates/{id}/thumb` as-is.
- Files are READ-ONLY under `/data`; never write.

## 3. Build & serve

- Multi-stage Dockerfile: `node:20-alpine` stage → `corepack enable && pnpm
  install && pnpm build` in `ui/` → copy `ui/dist` into runtime image;
  FastAPI serves `ui/dist` as the SPA (keep `/api/*`, `/mcp`, `/admin`
  routes working; SPA fallback route for TanStack Router).
- pnpm-lock: copy `pnpm-lock.yaml`/`package-lock.json` from pose ui so deps
  pin; prune pose-only deps if obvious (three.js, comfy) but keep any shared
  by shell components.
- VERSION bump to `0.2.0`. Update AGENTS.md + API_DOCS.md + README
  (build steps, new endpoints, preview fields).

## 4. Verify

- `python scripts/smoke_test.py` — extend it to also hit
  `/api/styles/{id}/preview` and `/api/styles/{id}/shots` for a style that
  HAS preview (pick one present in corpus; tolerate 404 if harvest hasn't
  reached it — warn don't fail).
- `pnpm build` (or `npm run build`) MUST pass in ui/ before REPORT_BACK —
  that's the acceptance gate. If deps/toolchain unavailable in your env,
  say so explicitly in the report (orchestrator will build in Docker).

## Constraints

- Do NOT `git init`/push — orchestrator owns git.
- Do NOT modify files outside this workdir (corpus on NAS is read-only for you).
- Do NOT remove `/api/*` endpoints or rename the `/mcp` tools — Hub connector
  `styles__*` is live.
- REPORT_BACK at the end: files changed, pages ported, build status, gaps.
