import html
import mimetypes
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any, Dict, List, Optional
from urllib.parse import urlparse

from fastapi import Depends, FastAPI, Header, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, HTMLResponse, Response
from fastapi.staticfiles import StaticFiles

from app.config import ROOT_DIR, settings
from app.mcp_server import mcp_router
from app.store import store

# FileResponse guesses content-type via mimetypes — Alpine's table lacks these
mimetypes.add_type("image/webp", ".webp")
mimetypes.add_type("image/avif", ".avif")
mimetypes.add_type("video/mp4", ".mp4")
mimetypes.add_type("video/webm", ".webm")

UI_DIR = ROOT_DIR / "ui"
UI_DIST_DIR = UI_DIR / "dist"
UI_BACKUP_DIR = ROOT_DIR / "ui-vanilla-backup"


def _resolve_spa_index() -> Path:
    if (UI_DIST_DIR / "index.html").exists():
        return UI_DIST_DIR / "index.html"
    if (UI_DIR / "index.html").exists():
        return UI_DIR / "index.html"
    return UI_BACKUP_DIR / "index.html"


@asynccontextmanager
async def lifespan(_app: FastAPI):
    store.reindex()
    yield


app = FastAPI(
    title="LNC Style Library (lnc-styles-gallery)",
    description="Browsable, searchable library of Refero design styles (DESIGN.md + visual previews) and Prompt-Driven UI HTML templates.",
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


@app.get("/api/styles/{style_id}/preview")
async def api_get_style_preview(style_id: str) -> Response:
    preview_path = store.get_style_preview_path(style_id)
    if preview_path and preview_path.exists() and preview_path.is_file():
        return FileResponse(preview_path)

    style = store.get_style(style_id, include_design=False)
    if not style:
        raise HTTPException(status_code=404, detail=f"Style not found: {style_id}")

    # Dynamic SVG card fallback when preview.<ext> has not been harvested yet
    is_dark = style.get("theme") == "dark"
    bg_start = "#090d16" if is_dark else "#f8fafc"
    bg_end = "#1e293b" if is_dark else "#e2e8f0"
    card_bg = "#0f172a" if is_dark else "#ffffff"
    card_border = "#334155" if is_dark else "#cbd5e1"
    fg_main = "#f8fafc" if is_dark else "#0f172a"
    fg_sub = "#94a3b8" if is_dark else "#475569"
    accent = "#6366f1" if is_dark else "#0284c7"
    theme_badge = "DARK" if is_dark else "LIGHT"

    title_raw = str(style.get("title") or style_id)
    title = html.escape(title_raw[:38] + ("…" if len(title_raw) > 38 else ""))
    tagline_raw = str(style.get("tagline") or "Design System & UI Tokens")
    tagline = html.escape(tagline_raw[:64] + ("…" if len(tagline_raw) > 64 else ""))

    source_url = str(style.get("source_url") or "")
    try:
        domain = urlparse(source_url).netloc.replace("www.", "") if source_url else ""
    except Exception:
        domain = ""
    domain_esc = html.escape(domain or "styles.refero.design")

    cats = style.get("categories") or []
    cat_label = html.escape(str(cats[0] if cats else "design-system").upper())

    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 400" width="640" height="400">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="{bg_start}"/>
      <stop offset="100%" stop-color="{bg_end}"/>
    </linearGradient>
  </defs>
  <rect width="640" height="400" fill="url(#bg)"/>
  <rect x="24" y="24" width="592" height="352" rx="16" fill="{card_bg}" stroke="{card_border}" stroke-width="1.5"/>
  <circle cx="52" cy="54" r="6" fill="#ef4444"/>
  <circle cx="72" cy="54" r="6" fill="#f59e0b"/>
  <circle cx="92" cy="54" r="6" fill="#10b981"/>
  <text x="116" y="59" fill="{fg_sub}" font-family="ui-monospace, monospace" font-size="12">{domain_esc}</text>
  <rect x="516" y="38" width="76" height="28" rx="6" fill="{accent}" opacity="0.18"/>
  <text x="554" y="57" fill="{accent}" font-family="system-ui, sans-serif" font-size="12" font-weight="700" text-anchor="middle">{theme_badge}</text>
  <line x1="24" y1="82" x2="616" y2="82" stroke="{card_border}" stroke-width="1"/>
  <rect x="52" y="112" width="130" height="24" rx="12" fill="{accent}" opacity="0.15"/>
  <text x="117" y="128" fill="{accent}" font-family="system-ui, sans-serif" font-size="11" font-weight="700" text-anchor="middle">{cat_label}</text>
  <text x="52" y="182" fill="{fg_main}" font-family="system-ui, sans-serif" font-size="26" font-weight="700">{title}</text>
  <text x="52" y="218" fill="{fg_sub}" font-family="system-ui, sans-serif" font-size="14">{tagline}</text>
  <rect x="52" y="264" width="160" height="68" rx="10" fill="{bg_start}" stroke="{card_border}" stroke-width="1"/>
  <rect x="68" y="280" width="60" height="10" rx="5" fill="{accent}"/>
  <rect x="68" y="300" width="110" height="8" rx="4" fill="{fg_sub}" opacity="0.45"/>
  <rect x="228" y="264" width="160" height="68" rx="10" fill="{bg_start}" stroke="{card_border}" stroke-width="1"/>
  <rect x="244" y="280" width="76" height="10" rx="5" fill="#10b981"/>
  <rect x="244" y="300" width="104" height="8" rx="4" fill="{fg_sub}" opacity="0.45"/>
  <rect x="404" y="264" width="160" height="68" rx="10" fill="{bg_start}" stroke="{card_border}" stroke-width="1"/>
  <rect x="420" y="280" width="52" height="10" rx="5" fill="#f59e0b"/>
  <rect x="420" y="300" width="116" height="8" rx="4" fill="{fg_sub}" opacity="0.45"/>
</svg>"""
    return Response(
        content=svg,
        media_type="image/svg+xml",
        headers={"X-Preview-Fallback": "true"},
    )


@app.get("/api/styles/{style_id}/shots")
async def api_get_style_shots(style_id: str) -> List[Dict[str, Any]]:
    style = store.get_style(style_id, include_design=False)
    if not style and not (store.refero_dir / "styles" / style_id).exists():
        raise HTTPException(status_code=404, detail=f"Style not found: {style_id}")
    return store.list_style_shots(style_id)


@app.get("/api/styles/{style_id}/shots/{n}")
async def api_get_style_shot(style_id: str, n: str) -> Response:
    shot_path = store.get_style_shot_path(style_id, n)
    if not shot_path or not shot_path.exists() or not shot_path.is_file():
        raise HTTPException(status_code=404, detail=f"Shot {n} not found for style: {style_id}")
    return FileResponse(shot_path)


@app.get("/api/search")
async def api_search(
    q: str = Query(default="", description="Search query"),
    theme: Optional[str] = Query(default=None),
    category: Optional[str] = Query(default=None),
    include_content: bool = Query(default=True),
    include_body: Optional[bool] = Query(default=None),
    limit: int = Query(default=30, ge=1, le=200),
) -> Dict[str, Any]:
    effective_include = include_body if include_body is not None else include_content
    return store.search(
        query=q,
        theme=theme,
        category=category,
        include_content=effective_include,
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


@app.get("/api/templates/{template_id}/{filename}")
async def api_get_template_asset(template_id: str, filename: str) -> Response:
    """Serve sibling assets (style.css, script.js, images) referenced relatively by the template HTML."""
    raw = store.get_template_raw_paths(template_id)
    if not raw:
        raise HTTPException(status_code=404, detail=f"Template not found: {template_id}")
    html_path_str = raw.get("_html_path", "")
    if not html_path_str:
        raise HTTPException(status_code=404, detail=f"Template has no HTML file: {template_id}")
    base_dir = Path(html_path_str).parent.resolve()
    target = (base_dir / filename).resolve()
    if target != base_dir and base_dir in target.parents and target.is_file():
        return FileResponse(target)
    raise HTTPException(status_code=404, detail=f"Asset not found: {filename}")


@app.get("/api/components")
async def api_list_components(
    category: Optional[str] = Query(default=None, description="Filter by component category"),
    kind: Optional[str] = Query(default=None, description="Filter by kind: html (live demo) or tsx (source)"),
    q: Optional[str] = Query(default=None, description="Filter by name, description, or tags"),
    limit: int = Query(default=200, ge=1, le=500),
    offset: int = Query(default=0, ge=0),
) -> Dict[str, Any]:
    return store.list_components(category=category, kind=kind, q=q, limit=limit, offset=offset)


@app.get("/api/components/{component_id}/html")
async def api_get_component_html(component_id: str) -> Response:
    file_path = store.get_component_file_path(component_id, "html")
    if not file_path:
        raise HTTPException(status_code=404, detail=f"Component has no HTML demo: {component_id}")
    return HTMLResponse(content=file_path.read_text(encoding="utf-8", errors="replace"))


@app.get("/api/components/{component_id}/files/{filename:path}")
async def api_get_component_file(component_id: str, filename: str) -> Response:
    file_path = store.get_component_file_path(component_id, filename)
    if not file_path:
        raise HTTPException(status_code=404, detail=f"File not found: {filename}")
    return FileResponse(file_path)


@app.get("/api/components/{component_id}")
async def api_get_component(
    component_id: str,
    include_source: bool = Query(default=False, description="Include source file contents in payload"),
) -> Dict[str, Any]:
    comp = store.get_component(component_id, include_source=include_source)
    if not comp:
        raise HTTPException(status_code=404, detail=f"Component not found: {component_id}")
    return comp


@app.get("/api/licenses")
async def api_licenses() -> Dict[str, Any]:
    return store.get_licenses()


@app.post("/api/admin/reindex", dependencies=[Depends(verify_admin_auth)])
@app.get("/api/admin/reindex", dependencies=[Depends(verify_admin_auth)])
async def api_admin_reindex() -> Dict[str, Any]:
    st = store.reindex()
    return {
        **st,
        "status": "reindexed",
        "summary": st,
    }


@app.get("/api/admin/status")
async def api_admin_status() -> Dict[str, Any]:
    return store.status()


@app.get("/api/admin/settings")
async def api_admin_settings() -> Dict[str, Any]:
    masked = settings.masked_dict()
    st = store.status()
    return {
        "settings": masked,
        "corpus": st,
        "service_name": masked["service_name"],
        "service_version": masked["version"],
        "host": "0.0.0.0",
        "port": masked["port"],
        "public_base_url": masked["public_url"],
        "data_root": masked["data_dir"],
        "refero_dir": masked["refero_styles_dir"],
        "refero_dir_exists": st["refero_styles_exists"],
        "prompt_ui_dir": masked["prompt_ui_dir"],
        "prompt_ui_dir_exists": st["prompt_ui_exists"],
        "api_key_configured": masked["api_key_configured"],
        "api_key_masked": masked["api_key_masked"] or "***",
        "cors_origins": "*",
    }


if (UI_DIST_DIR / "static").exists():
    app.mount("/static", StaticFiles(directory=str(UI_DIST_DIR / "static")), name="static")
if (UI_DIST_DIR / "assets").exists():
    app.mount("/assets", StaticFiles(directory=str(UI_DIST_DIR / "assets")), name="assets")
elif UI_DIR.exists():
    app.mount("/assets", StaticFiles(directory=str(UI_DIR)), name="assets")

if UI_BACKUP_DIR.exists():
    app.mount("/ui-vanilla", StaticFiles(directory=str(UI_BACKUP_DIR)), name="ui-vanilla")
if UI_DIR.exists():
    app.mount("/ui", StaticFiles(directory=str(UI_DIR)), name="ui")


@app.get("/admin", response_class=HTMLResponse)
@app.get("/styles", response_class=HTMLResponse)
@app.get("/collections", response_class=HTMLResponse)
@app.get("/templates", response_class=HTMLResponse)
@app.get("/components", response_class=HTMLResponse)
@app.get("/platforms", response_class=HTMLResponse)
@app.get("/licenses", response_class=HTMLResponse)
@app.get("/", response_class=HTMLResponse)
async def serve_spa_index() -> Response:
    return FileResponse(_resolve_spa_index())


@app.get("/{full_path:path}", response_class=HTMLResponse)
async def serve_spa_fallback(full_path: str) -> Response:
    clean_path = full_path.strip("/")
    if clean_path.startswith("api/") or clean_path == "api" or clean_path.startswith("mcp"):
        raise HTTPException(status_code=404, detail="Not found")

    # Serve static files from ui/dist (e.g. icon.svg, favicon.ico, static/*) if present
    for base_dir in (UI_DIST_DIR, UI_DIR):
        if base_dir.exists():
            candidate = (base_dir / clean_path).resolve()
            if str(candidate).startswith(str(base_dir.resolve())) and candidate.is_file():
                return FileResponse(candidate)

    return FileResponse(_resolve_spa_index())
