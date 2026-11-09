"""In-process and live HTTP smoke test for lnc-styles-gallery."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from fastapi.testclient import TestClient
from app.main import app


def run_smoke() -> None:
    with TestClient(app) as client:
        r_health = client.get("/health")
        assert r_health.status_code == 200, r_health.text
        health = r_health.json()
        print("HEALTH:", health)

        r_styles = client.get("/api/styles?limit=5")
        assert r_styles.status_code == 200, r_styles.text
        styles_payload = r_styles.json()
        print(f"/api/styles -> total={styles_payload['total']}, returned={len(styles_payload['items'])}")

        if styles_payload["items"]:
            first_id = styles_payload["items"][0]["id"]
            r_detail = client.get(f"/api/styles/{first_id}")
            assert r_detail.status_code == 200, r_detail.text
            detail = r_detail.json()
            print(f"/api/styles/{first_id} -> title={detail['title']!r}, design_md_len={len(detail.get('design_md', ''))}")

        r_cols = client.get("/api/collections")
        assert r_cols.status_code == 200, r_cols.text
        cols_payload = r_cols.json()
        print(f"/api/collections -> total={cols_payload['total']}")

        r_tpls = client.get("/api/templates")
        assert r_tpls.status_code == 200, r_tpls.text
        tpls_payload = r_tpls.json()
        print(f"/api/templates -> total={tpls_payload['total']}, languages={tpls_payload['languages']}")

        if tpls_payload["items"]:
            first_tpl = tpls_payload["items"][0]["id"]
            r_tpl_html = client.get(f"/api/templates/{first_tpl}/html")
            assert r_tpl_html.status_code == 200, r_tpl_html.text
            print(f"/api/templates/{first_tpl}/html -> html_len={len(r_tpl_html.text)}")

        r_search = client.get("/api/search?q=dark&limit=5")
        assert r_search.status_code == 200, r_search.text
        print(f"/api/search?q=dark -> total_styles={r_search.json()['total_styles']}")

        r_lic = client.get("/api/licenses")
        assert r_lic.status_code == 200, r_lic.text
        print(f"/api/licenses -> attributed_sources_count={r_lic.json()['attributed_sources_count']}")

        r_mcp_init = client.post(
            "/mcp",
            json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "smoke", "version": "1.0"}},
            },
        )
        assert r_mcp_init.status_code == 200, r_mcp_init.text
        print("/mcp initialize ->", r_mcp_init.json()["result"]["serverInfo"])

        r_mcp_tools = client.post(
            "/mcp",
            json={"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        )
        assert r_mcp_tools.status_code == 200, r_mcp_tools.text
        tool_names = [t["name"] for t in r_mcp_tools.json()["result"]["tools"]]
        print("/mcp tools/list ->", tool_names)

        r_reidx = client.post("/api/admin/reindex")
        assert r_reidx.status_code == 200, r_reidx.text
        print("/api/admin/reindex ->", r_reidx.json()["last_index_duration_ms"], "ms")


if __name__ == "__main__":
    run_smoke()
