import json
from typing import Any, Dict, List, Optional
from fastapi import APIRouter, Request, Response
from fastapi.responses import JSONResponse

from app.config import settings
from app.store import store

mcp_router = APIRouter()

# Optional FastMCP instance for callers that introspect `mcp` directly
try:
    from fastmcp import FastMCP  # type: ignore

    mcp = FastMCP("lnc-styles-gallery")

    @mcp.tool(name="list_styles", description="List Refero design styles with optional theme and category filters.")
    def list_styles(
        theme: Optional[str] = None,
        category: Optional[str] = None,
        limit: int = 48,
        offset: int = 0,
    ) -> Dict[str, Any]:
        return store.list_styles(theme=theme, category=category, limit=limit, offset=offset)

    @mcp.tool(name="search", description="Search design styles by title, tagline, source URL, and DESIGN.md content.")
    def search(
        query: str,
        theme: Optional[str] = None,
        category: Optional[str] = None,
        include_content: bool = True,
        limit: int = 25,
    ) -> Dict[str, Any]:
        return store.search(query=query, theme=theme, category=category, include_content=include_content, limit=limit)

    @mcp.tool(name="get_design", description="Get full DESIGN.md specification and metadata for a specific Refero style ID.")
    def get_design(style_id: str) -> Dict[str, Any]:
        item = store.get_style(style_id, include_design=True)
        if not item:
            return {"error": f"Style not found: {style_id}"}
        return item

    @mcp.tool(name="list_collections", description="List grouped style collections by category, mood, and theme.")
    def list_collections() -> Dict[str, Any]:
        cols = store.list_collections()
        return {"total": len(cols), "collections": cols}

    @mcp.tool(name="get_template", description="Get prompt-driven-ui HTML template metadata, generation prompt, and optional HTML source.")
    def get_template(template_id: str, include_html: bool = False) -> Dict[str, Any]:
        tpl = store.get_template(template_id, include_html=include_html)
        if not tpl:
            return {"error": f"Template not found: {template_id}"}
        return tpl

    @mcp.tool(name="list_components", description="List UI effect/component recipes (LNC HTML demos + Knowns UI TSX sources) by category.")
    def list_components(
        category: Optional[str] = None,
        kind: Optional[str] = None,
        q: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> Dict[str, Any]:
        return store.list_components(category=category, kind=kind, q=q, limit=limit, offset=offset)

    @mcp.tool(name="get_component", description="Get a UI component's metadata and optionally its full source (HTML demo or TSX files).")
    def get_component(component_id: str, include_source: bool = False) -> Dict[str, Any]:
        comp = store.get_component(component_id, include_source=include_source)
        if not comp:
            return {"error": f"Component not found: {component_id}"}
        return comp

    @mcp.tool(name="reindex", description="Re-scan the Refero Styles and Prompt-Driven UI corpus directories without restarting.")
    def reindex() -> Dict[str, Any]:
        return store.reindex()

except Exception:
    mcp = None


MCP_TOOLS_SCHEMA: List[Dict[str, Any]] = [
    {
        "name": "list_styles",
        "description": "List Refero design styles with optional theme ('light'|'dark') and category/collection filters.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "theme": {"type": "string", "description": "Filter by color theme: 'light' or 'dark'."},
                "category": {"type": "string", "description": "Filter by collection/category slug (e.g. 'editorial', 'terminal-dev', 'luxury')."},
                "limit": {"type": "integer", "default": 48, "description": "Max items to return (1-500)."},
                "offset": {"type": "integer", "default": 0, "description": "Pagination offset."},
            },
        },
    },
    {
        "name": "search",
        "description": "Search Refero design styles (title, tagline, source_url, DESIGN.md tokens/rules) and HTML templates.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search keywords (e.g. 'monospace terminal', 'serif editorial', 'fintech dark')."},
                "theme": {"type": "string", "description": "Optional theme filter ('light' or 'dark')."},
                "category": {"type": "string", "description": "Optional category/collection slug."},
                "include_content": {"type": "boolean", "default": True, "description": "Whether to search inside DESIGN.md content."},
                "limit": {"type": "integer", "default": 25, "description": "Maximum results to return."},
            },
            "required": ["query"],
        },
    },
    {
        "name": "get_design",
        "description": "Retrieve full DESIGN.md markdown style guide and metadata for a style ID.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "style_id": {"type": "string", "description": "UUID or ID of the Refero style."},
            },
            "required": ["style_id"],
        },
    },
    {
        "name": "list_collections",
        "description": "List all browsable style collections (by mood/category, theme, and sitemap) with counts and sample styles.",
        "inputSchema": {
            "type": "object",
            "properties": {},
        },
    },
    {
        "name": "get_template",
        "description": "Retrieve a prompt-driven-ui HTML template's metadata, generation prompt, and optionally its full HTML code.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "template_id": {"type": "string", "description": "Template ID (e.g. 'en-01-saas-landing' or slug/number)."},
                "include_html": {"type": "boolean", "default": False, "description": "Include full raw HTML string in response."},
            },
            "required": ["template_id"],
        },
    },
    {
        "name": "list_components",
        "description": "List UI effect/component recipes: LNC self-contained HTML demos (buttons, cards, loaders, backgrounds, AI-agent components) and Knowns UI TSX sources.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "category": {"type": "string", "description": "Filter by category (buttons, cards, loaders, inputs-toggles, backgrounds, text-effects, ai-agent, marketing, knowns-ui)."},
                "kind": {"type": "string", "description": "Filter by kind: 'html' (live demo) or 'tsx' (React source)."},
                "q": {"type": "string", "description": "Search name, description, or tags."},
                "limit": {"type": "integer", "default": 100},
                "offset": {"type": "integer", "default": 0},
            },
        },
    },
    {
        "name": "get_component",
        "description": "Get a UI component's metadata; pass include_source=true to receive the full HTML demo source or all TSX file contents.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "component_id": {"type": "string", "description": "Component ID (e.g. 'lnc-aurora-bg'... or 'knowns-button')."},
                "include_source": {"type": "boolean", "default": False},
            },
            "required": ["component_id"],
        },
    },
    {
        "name": "reindex",
        "description": "Re-scan the Refero Styles and Prompt-Driven UI corpus directories on disk without restarting the server.",
        "inputSchema": {
            "type": "object",
            "properties": {},
        },
    },
]


def _dispatch_tool_call(name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
    while name.startswith("styles__"):
        name = name[len("styles__"):]
    if name == "list_styles":
        return store.list_styles(
            q=arguments.get("q"),
            theme=arguments.get("theme"),
            category=arguments.get("category"),
            limit=int(arguments.get("limit", 48)),
            offset=int(arguments.get("offset", 0)),
        )
    if name == "search":
        return store.search(
            query=str(arguments.get("query") or arguments.get("q") or ""),
            theme=arguments.get("theme"),
            category=arguments.get("category"),
            include_content=bool(arguments.get("include_content", True)),
            limit=int(arguments.get("limit", 25)),
        )
    if name == "get_design":
        style_id = str(arguments.get("style_id") or arguments.get("id") or "").strip()
        item = store.get_style(style_id, include_design=True)
        if not item:
            raise ValueError(f"Style not found: {style_id}")
        return item
    if name == "list_collections":
        cols = store.list_collections()
        return {"total": len(cols), "collections": cols}
    if name == "get_template":
        template_id = str(arguments.get("template_id") or arguments.get("id") or "").strip()
        include_html = bool(arguments.get("include_html", False))
        tpl = store.get_template(template_id, include_html=include_html)
        if not tpl:
            raise ValueError(f"Template not found: {template_id}")
        return tpl
    if name == "list_components":
        return store.list_components(
            category=arguments.get("category"),
            kind=arguments.get("kind"),
            q=arguments.get("q"),
            limit=int(arguments.get("limit", 100)),
            offset=int(arguments.get("offset", 0)),
        )
    if name == "get_component":
        component_id = str(arguments.get("component_id") or arguments.get("id") or "").strip()
        comp = store.get_component(component_id, include_source=bool(arguments.get("include_source", False)))
        if not comp:
            raise ValueError(f"Component not found: {component_id}")
        return comp
    if name == "reindex":
        return store.reindex()
    raise KeyError(f"Unknown MCP tool: {name}")


def _handle_jsonrpc_message(msg: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    method = msg.get("method", "")
    req_id = msg.get("id")
    params = msg.get("params") or {}

    # Notifications (no id)
    if req_id is None and method.startswith("notifications/"):
        return None

    if method == "initialize":
        protocol_version = params.get("protocolVersion") or "2024-11-05"
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": protocol_version,
                "capabilities": {
                    "tools": {"listChanged": False},
                },
                "serverInfo": {
                    "name": settings.service_name,
                    "version": settings.version,
                },
            },
        }

    if method == "ping":
        return {"jsonrpc": "2.0", "id": req_id, "result": {}}

    if method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": MCP_TOOLS_SCHEMA,
            },
        }

    if method == "tools/call":
        tool_name = str(params.get("name") or "")
        tool_args = params.get("arguments") or {}
        try:
            payload = _dispatch_tool_call(tool_name, tool_args)
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": json.dumps(payload, ensure_ascii=False),
                        }
                    ],
                    "structuredContent": payload,
                    "isError": False,
                },
            }
        except Exception as exc:
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": json.dumps({"error": str(exc)}, ensure_ascii=False),
                        }
                    ],
                    "isError": True,
                },
            }

    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {
            "code": -32601,
            "message": f"Method not found: {method}",
        },
    }


@mcp_router.get("/mcp")
async def mcp_info() -> Dict[str, Any]:
    return {
        "protocol": "mcp-streamable-http",
        "service": settings.service_name,
        "version": settings.version,
        "tools": [t["name"] for t in MCP_TOOLS_SCHEMA],
    }


@mcp_router.post("/mcp")
async def mcp_streamable_http(request: Request) -> Response:
    try:
        body = await request.json()
    except Exception:
        return JSONResponse(
            status_code=400,
            content={
                "jsonrpc": "2.0",
                "id": None,
                "error": {"code": -32700, "message": "Parse error"},
            },
        )

    headers = {"Mcp-Session-Id": request.headers.get("Mcp-Session-Id", "lnc-styles-session")}

    if isinstance(body, list):
        responses = []
        for item in body:
            if isinstance(item, dict):
                resp = _handle_jsonrpc_message(item)
                if resp is not None:
                    responses.append(resp)
        if not responses:
            return Response(status_code=202, headers=headers)
        return JSONResponse(content=responses, headers=headers)

    if isinstance(body, dict):
        resp = _handle_jsonrpc_message(body)
        if resp is None:
            return Response(status_code=202, headers=headers)
        return JSONResponse(content=resp, headers=headers)

    return JSONResponse(
        status_code=400,
        content={
            "jsonrpc": "2.0",
            "id": None,
            "error": {"code": -32600, "message": "Invalid Request"},
        },
    )
