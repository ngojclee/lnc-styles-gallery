"""End-to-end smoke test for lnc-styles-gallery REST API, image previews, UI, and MCP endpoint."""
from __future__ import annotations

import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT_DIR))

from fastapi.testclient import TestClient  # noqa: E402
from app.main import app  # noqa: E402


def run_smoke_tests() -> None:
    with TestClient(app) as client:
        # 1. Health check
        r = client.get("/api/health")
        assert r.status_code == 200, f"Health failed: {r.text}"
        health = r.json()
        print(
            f"[OK] /api/health -> styles={health['styles_count']}, "
            f"designs={health['designs_count']}, "
            f"previews={health.get('previews_count', 0)}, "
            f"collections={health['collections_count']}, "
            f"templates={health['templates_count']}"
        )
        assert health["styles_count"] > 0, "Expected styles_count > 0"
        assert health["templates_count"] > 0, "Expected templates_count > 0"

        # 2. List styles + filters + preview_url field check
        r = client.get("/api/styles?limit=12")
        assert r.status_code == 200
        styles_data = r.json()
        assert styles_data["total"] > 0
        items = styles_data["items"]
        first_style = items[0]
        style_id = first_style["id"]
        assert "has_preview" in first_style, "Expected 'has_preview' field on style item"
        assert "preview_url" in first_style, "Expected 'preview_url' field on style item"
        print(
            f"[OK] /api/styles -> total={styles_data['total']}, "
            f"first={style_id} ({first_style['title']}), "
            f"has_preview={first_style['has_preview']}"
        )

        # 3. Single style detail (DESIGN.md)
        r = client.get(f"/api/styles/{style_id}")
        assert r.status_code == 200
        detail = r.json()
        assert detail["id"] == style_id
        assert "design_md" in detail
        assert "has_preview" in detail
        assert "preview_url" in detail
        print(f"[OK] /api/styles/{style_id} -> design_md len={len(detail['design_md'])} chars")

        # 4. Style preview image & shots endpoints
        # Prefer picking a style that already has a harvested preview on disk
        preview_candidate_id = style_id
        for item in items:
            if item.get("has_preview"):
                preview_candidate_id = item["id"]
                break

        r_prev = client.get(f"/api/styles/{preview_candidate_id}/preview")
        if r_prev.status_code == 200:
            ct = r_prev.headers.get("content-type", "")
            is_fallback = r_prev.headers.get("x-preview-fallback") == "true"
            print(
                f"[OK] /api/styles/{preview_candidate_id}/preview -> status=200, "
                f"content-type={ct}, fallback={is_fallback}, bytes={len(r_prev.content)}"
            )
        elif r_prev.status_code == 404:
            print(
                f"[WARN] /api/styles/{preview_candidate_id}/preview returned 404 "
                "(harvest may not have reached this style yet)"
            )
        else:
            raise AssertionError(
                f"Unexpected status {r_prev.status_code} from /api/styles/{preview_candidate_id}/preview"
            )

        r_shots = client.get(f"/api/styles/{preview_candidate_id}/shots")
        if r_shots.status_code == 200:
            shots = r_shots.json()
            assert isinstance(shots, list), "Expected shots response to be a JSON list"
            print(f"[OK] /api/styles/{preview_candidate_id}/shots -> {len(shots)} shots")
            if shots:
                shot_url = shots[0]["url"]
                r_shot_file = client.get(shot_url)
                if r_shot_file.status_code == 200:
                    print(f"[OK] {shot_url} -> {len(r_shot_file.content)} bytes")
                else:
                    print(f"[WARN] {shot_url} -> status={r_shot_file.status_code}")
        elif r_shots.status_code == 404:
            print(
                f"[WARN] /api/styles/{preview_candidate_id}/shots returned 404 "
                "(harvest may not have reached this style yet)"
            )
        else:
            raise AssertionError(
                f"Unexpected status {r_shots.status_code} from /api/styles/{preview_candidate_id}/shots"
            )

        # 5. Search endpoint (including DESIGN.md body)
        r = client.get("/api/search?q=monospace&include_body=true&limit=5")
        assert r.status_code == 200
        search_data = r.json()
        print(
            f"[OK] /api/search?q=monospace -> styles={search_data['total_styles']}, "
            f"templates={search_data['total_templates']}"
        )

        # 6. Collections endpoint
        r = client.get("/api/collections")
        assert r.status_code == 200
        cols = r.json()
        assert cols["total"] > 0
        col_id = cols["items"][0]["id"]
        r_col = client.get(f"/api/collections/{col_id}?limit=5")
        assert r_col.status_code == 200
        print(f"[OK] /api/collections -> total={cols['total']}, first={col_id}")

        # 7. Templates endpoints
        r = client.get("/api/templates?lang=en&limit=5")
        assert r.status_code == 200
        tpls = r.json()
        assert tpls["total"] == 50
        tpl_id = tpls["items"][0]["id"]
        r_tpl = client.get(f"/api/templates/{tpl_id}")
        assert r_tpl.status_code == 200
        assert len(r_tpl.json()["prompt"]) > 0
        r_html = client.get(f"/api/templates/{tpl_id}/html")
        assert r_html.status_code == 200
        assert "text/html" in r_html.headers["content-type"]
        r_thumb = client.get(f"/api/templates/{tpl_id}/thumb")
        assert r_thumb.status_code == 200
        print(f"[OK] /api/templates/{tpl_id} (+ /html, /thumb) -> verified")

        # 8. Licenses endpoint
        r = client.get("/api/licenses")
        assert r.status_code == 200
        lic = r.json()
        assert len(lic["corpora"]) == 2
        print(
            f"[OK] /api/licenses -> corpora={len(lic['corpora'])}, "
            f"attributed_sources={lic['attributed_sources_total']}"
        )

        # 9. Admin endpoints (status, settings, reindex)
        r = client.get("/api/admin/settings")
        assert r.status_code == 200
        assert r.json()["api_key_masked"] == "***"
        r = client.post("/api/admin/reindex")
        assert r.status_code == 200
        assert r.json()["status"] == "reindexed"
        print("[OK] /api/admin/settings & POST /api/admin/reindex -> verified")

        # 10. UI pages (/ and /admin)
        for path in ("/", "/styles", "/collections", "/templates", "/licenses", "/admin"):
            r = client.get(path)
            assert r.status_code == 200, f"GET {path} failed with {r.status_code}"
            assert "text/html" in r.headers["content-type"]
        print("[OK] SPA routes (/, /styles, /collections, /templates, /licenses, /admin) -> served")

        # 11. MCP Streamable HTTP JSON-RPC (initialize, tools/list, tools/call)
        r = client.post(
            "/mcp",
            json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "smoke-test", "version": "1.0"},
                },
            },
        )
        assert r.status_code == 200
        init_res = r.json()["result"]
        assert init_res["serverInfo"]["name"] == "lnc-styles-gallery"

        r = client.post("/mcp", json={"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        assert r.status_code == 200
        tool_names = [t["name"] for t in r.json()["result"]["tools"]]
        expected_tools = {
            "styles__list_styles",
            "styles__search",
            "styles__get_design",
            "styles__list_collections",
            "styles__get_template",
            "styles__reindex",
        }
        assert expected_tools.issubset(set(tool_names)), f"Missing MCP tools: {tool_names}"
        print(f"[OK] /mcp tools/list -> {tool_names}")

        r = client.post(
            "/mcp",
            json={
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "styles__get_design",
                    "arguments": {"style_id": style_id},
                },
            },
        )
        assert r.status_code == 200
        assert not r.json()["result"]["isError"]
        print("[OK] /mcp tools/call (styles__get_design) -> verified")

    print("\nALL SMOKE TESTS PASSED!")


if __name__ == "__main__":
    run_smoke_tests()
