# LNC Style Library (`lnc-styles-gallery`)

Browsable, searchable library of product website design styles (`DESIGN.md` token & guideline specifications from Refero Styles) and full HTML template samples (`prompt-driven-ui`), built for both humans (Web Gallery UI) and AI agents (REST API + `/mcp` Streamable HTTP).

## Quick Start

### 1. Sync Corpus (Optional if NAS share is mounted)
```bat
scripts\sync_data_from_nas.bat
```

### 2. Run Locally
```powershell
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8460
```

Open `http://localhost:8460` for the Web UI:
- `#styles` — Browse & search Refero styles and inspect/copy `DESIGN.md`
- `#collections` — Curated mood, category, and theme groupings
- `#templates` — 100 full HTML templates (EN + VI) with thumbnails, prompts, and live HTML preview
- `#licenses` — Source & license attributions

### 3. Docker Compose
```bash
docker compose -f infra/docker-compose.yml up --build -d
```
Container listens on internal port `8000`, mapped to host port `8460` (target domain: `styles.lengoc.me`).
