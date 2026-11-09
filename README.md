# LNC Style Library (`lnc-styles-gallery` v0.2.0)

Browsable, searchable library of product website design styles (`DESIGN.md` token & guideline specifications + visual hero previews and gallery screenshots from Refero Styles) and full HTML template samples (`prompt-driven-ui`), built for both humans (React + Vite + TanStack Router + shadcn/ui SPA ported from `pose.lengoc.me`) and AI agents (REST API + `/mcp` Streamable HTTP).

## Quick Start

### 1. Sync Corpus (Optional if NAS share is mounted)
```bat
scripts\sync_data_from_nas.bat
```

### 2. Build Frontend & Run Locally
```powershell
# Build React SPA into ui/dist
cd ui
corepack enable
pnpm install
pnpm build
cd ..

# Start FastAPI server
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8460
```

Open `http://localhost:8460` for the Web UI (supports both path routing and hash links):
- `/styles` (or `#/styles`) — Browse & search Refero styles with visual preview cards, hero + shots gallery strip, and `DESIGN.md` inspector (`Rendered | Raw | Meta | Copy`)
- `/collections` (or `#/collections`) — Curated mood, industry, and theme groupings with sample preview strips
- `/templates` (or `#/templates`) — 100 full HTML templates (50 EN + 50 VI) with thumbnails, generation prompts, and live iframe HTML preview dialog
- `/licenses` (or `#/licenses`) — Source & license attributions
- `/admin` — Corpus diagnostics, preview/shot index counts, hot re-index trigger, and masked runtime settings

### 3. Docker Compose (Multi-Stage Build)
```bash
docker compose -f infra/docker-compose.yml up --build -d
```
The multi-stage `Dockerfile` builds `ui/dist` in `node:20-alpine` (`corepack enable && pnpm install && pnpm build`) and copies the compiled SPA into the `python:3.11-slim` runtime container. Container listens on internal port `8000`, mapped to host port `8460` (target domain: `styles.lengoc.me`).

### 4. Smoke Test
```bash
python scripts/smoke_test.py
```
Verifies `/api/health`, `/api/styles`, `/api/styles/{id}/preview`, `/api/styles/{id}/shots`, `/api/search`, `/api/collections`, `/api/templates`, `/api/licenses`, `/api/admin/*`, SPA routes, and `/mcp` JSON-RPC tools.
