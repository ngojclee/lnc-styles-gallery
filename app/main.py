import html
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any, Dict, Optional

from fastapi import Depends, FastAPI, Header, HTTPException, Query, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, HTMLResponse, Response
from fastapi.staticfiles import StaticFiles

from app.config import ROOT_DIR, settings
from app.mcp_server import mcp_router
from app.store import store

UI_DIR = ROOT_DIR / "ui"


@asynccontextmanager
async def lifespan(_app: FastAPI):
    store.reindex()
    yield


app = FastAPI(
    title="LNC Style Library (lnc-styles-gallery)",
    description="Browsable, searchable library of Refero design styles (DESIGN.md) and Prompt-Driven UI HTML templates.",
    version=settings.version,
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(mcp_router)


def verify_admin_auth(
    x_api_key: Optional[str] = Header(default=None),
    authorization: Optional[str] = Header(default=None),
) -> None:
    if not settings.api_key:
        return
    token = (x_api_key or "").strip()
    if not token and authorization and authorization.lower().startswith("bearer "):
        token = authorization[7:].strip()
    if token != settings.api_key:
        raise HTTPException(status_code=401, detail="Invalid or missing API key")


@app.get("/health")
@app.get("/api/health")
async def health() -> Dict[str, Any]:
    return store.status()


@app.get("/api/styles")
async def api_list_styles(
    q: Optional[str] = Query(default=None, description="Search query across title, tagline, ID, source URL"),
    theme: Optional[str] = Query(default=None, description="Theme filter: light or dark"),
    category: Optional[str] = Query(default=None, description="Category or collection slug"),
    collection: Optional[str] = Query(default=None, description="Alias for category"),
    has_design: Optional[bool] = Query(default=None, description="Filter by DESIGN.md presence"),
    limit: int = Query(default=48, ge=1, le=500),
    offset: int = Query(default=0, ge=0),
) -> Dict[str, Any]:
    effective_cat = category or collection
    return store.list_styles(
        q=q,
        theme=theme,
        category=effective_cat,
        has_design=has_design,
        limit=limit,
        offset=offset,
    )


@app.get("/api/styles/{style_id}")
async def api_get_style(style_id: str) -> Dict[str, Any]:
    item = store.get_style(style_id, include_design=True)
    if not item:
        raise HTTPException(status_code=404, detail=f"Style not found: {style_id}")
    return item


@app.get("/api/search")
async def api_search(
    q: str = Query(default="", description="Search query"),
    theme: Optional[str] = Query(default=None),
    category: Optional[str] = Query(default=None),
    include_content: bool = Query(default=True),
    limit: int = Query(default=30, ge=1, le=200),
) -> Dict[str, Any]:
    return store.search(
        query=q,
        theme=theme,
        category=category,
        include_content=include_content,
        limit=limit,
    )


@app.get("/api/collections")
async def api_list_collections() -> Dict[str, Any]:
    cols = store.list_collections()
    return {
        "total": len(cols),
        "items": cols,
    }


@app.get("/api/collections/{collection_id}")
async def api_get_collection(
    collection_id: str,
    limit: int = Query(default=100, ge=1, le=500),
    offset: int = Query(default=0, ge=0),
) -> Dict[str, Any]:
    col = store.get_collection(collection_id, limit=limit, offset=offset)
    if not col:
        raise HTTPException(status_code=404, detail=f"Collection not found: {collection_id}")
    return col


@app.get("/api/templates")
async def api_list_templates(
    lang: Optional[str] = Query(default=None, description="Filter by language: en or vi"),
    q: Optional[str] = Query(default=None, description="Filter by title, slug, or prompt"),
    limit: int = Query(default=100, ge=1, le=200),
    offset: int = Query(default=0, ge=0),
) -> Dict[str, Any]:
    return store.list_templates(lang=lang, q=q, limit=limit, offset=offset)


@app.get("/api/templates/{template_id}")
async def api_get_template(
    template_id: str,
    include_html: bool = Query(default=False, description="Include raw HTML string in payload"),
) -> Dict[str, Any]:
    tpl = store.get_template(template_id, include_html=include_html)
    if not tpl:
        raise HTTPException(status_code=404, detail=f"Template not found: {template_id}")
    return tpl


@app.get("/api/templates/{template_id}/html")
async def api_get_template_html(template_id: str) -> Response:
    raw = store.get_template_raw_paths(template_id)
    if not raw:
        raise HTTPException(status_code=404, detail=f"Template not found: {template_id}")
    html_path_str = raw.get("_html_path", "")
    if not html_path_str or not Path(html_path_str).exists():
        raise HTTPException(status_code=404, detail=f"HTML file missing for template: {template_id}")
    content = Path(html_path_str).read_text(encoding="utf-8", errors="replace")
    return HTMLResponse(content=content)


@app.get("/api/templates/{template_id}/thumb")
async def api_get_template_thumb(template_id: str) -> Response:
    raw = store.get_template_raw_paths(template_id)
    if not raw:
        raise HTTPException(status_code=404, detail=f"Template not found: {template_id}")
    thumb_path_str = raw.get("_thumb_path", "")
    if thumb_path_str:
        t_path = Path(thumb_path_str)
        if t_path.exists() and t_path.is_file():
            return FileResponse(t_path)

    # Dynamic SVG card fallback when a thumbnail image file is not on disk
    title = html.escape(str(raw.get("title") or template_id)[:42])
    lang_badge = html.escape(str(raw.get("lang") or "en").upper())
    num_str = f"#{int(raw.get('number', 1)):02d}"
    accent = "#6366f1" if raw.get("lang") == "en" else "#10b981"
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 360" width="640" height="360">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#0f172a"/>
      <stop offset="100%" stop-color="#1e293b"/>
    </linearGradient>
  </defs>
  <rect width="640" height="360" fill="url(#bg)"/>
  <rect x="24" y="24" width="592" height="312" rx="14" fill="#0b1120" stroke="#334155" stroke-width="1.5"/>
  <circle cx="52" cy="52" r="6" fill="#ef4444"/>
  <circle cx="72" cy="52" r="6" fill="#f59e0b"/>
  <circle cx="92" cy="52" r="6" fill="#10b981"/>
  <rect x="520" y="38" width="72" height="28" rx="6" fill="{accent}" opacity="0.2"/>
  <text x="556" y="57" fill="{accent}" font-family="system-ui, sans-serif" font-size="13" font-weight="700" text-anchor="middle">{lang_badge} {num_str}</text>
  <text x="52" y="165" fill="#f8fafc" font-family="system-ui, sans-serif" font-size="24" font-weight="700">{title}</text>
  <text x="52" y="205" fill="#94a3b8" font-family="monospace" font-size="14">{html.escape(str(raw.get("slug", "")))}</text>
  <rect x="52" y="250" width="140" height="34" rx="8" fill="{accent}"/>
  <text x="122" y="272" fill="#ffffff" font-family="system-ui, sans-serif" font-size="13" font-weight="600" text-anchor="middle">HTML Template</text>
</svg>"""
    return Response(content=svg, media_type="image/svg+xml")


@app.get("/api/licenses")
async def api_licenses() -> Dict[str, Any]:
    return store.get_licenses()


@app.post("/api/admin/reindex", dependencies=[Depends(verify_admin_auth)])
@app.get("/api/admin/reindex", dependencies=[Depends(verify_admin_auth)])
async def api_admin_reindex() -> Dict[str, Any]:
    return store.reindex()


@app.get("/api/admin/status")
async def api_admin_status() -> Dict[str, Any]:
    return store.status()


@app.get("/api/admin/settings")
async def api_admin_settings() -> Dict[str, Any]:
    return {
        "settings": settings.masked_dict(),
        "corpus": store.status(),
    }


@app.get("/admin", response_class=HTMLResponse)
async def serve_admin_page() -> Response:
    admin_html = UI_DIR / "admin.html"
    if admin_html.exists():
        return FileResponse(admin_html)
    return FileResponse(UI_DIR / "index.html")


@app.get("/", response_class=HTMLResponse)
async def serve_index() -> Response:
    return FileResponse(UI_DIR / "index.html")


if UI_DIR.exists():
    app.mount("/ui", StaticFiles(directory=str(UI_DIR)), name="ui")
    app.mount("/assets", StaticFiles(directory=str(UI_DIR)), name="assets")
