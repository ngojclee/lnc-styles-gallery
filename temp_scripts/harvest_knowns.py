#!/usr/bin/env python3
"""Harvest Knowns UI (MIT) components into the lnc-components corpus.

Fetches https://ui.knowns.dev/r/registry.json then each /r/<name>.json
(shadcn registry format) and saves source files under items/knowns-<name>/.
Third-party logo icons under lib/knowns/icons are trademark-restricted — skipped.
"""
import json
import time
import urllib.request
from pathlib import Path

OUT = Path(r"//10.21.2.2/data/_agentsync/repos/lnc-components")
ITEMS = OUT / "items"
BASE = "https://ui.knowns.dev"
UA = {"User-Agent": "lnc-styles-gallery/0.3 (+https://styles.lengoc.me)"}
SKIP = "lib/knowns/icons"  # trademark-restricted third-party logos


def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.read()


def main() -> None:
    ITEMS.mkdir(parents=True, exist_ok=True)
    registry = json.loads(fetch(f"{BASE}/r/registry.json"))
    items = registry.get("items") or []
    print(f"registry items: {len(items)}")

    idx_path = OUT / "index.json"
    existing = []
    if idx_path.exists():
        try:
            existing = [r for r in json.loads(idx_path.read_text(encoding="utf-8")) if isinstance(r, dict)]
        except Exception:
            existing = []
    keep = [r for r in existing if not str(r.get("id", "")).startswith("knowns-")]

    new_records = []
    failed = []
    for i, it in enumerate(items, 1):
        name = str(it.get("name") or "").strip()
        if not name:
            continue
        try:
            item = json.loads(fetch(f"{BASE}/r/{name}.json"))
        except Exception as exc:
            failed.append((name, str(exc)))
            continue

        d = ITEMS / f"knowns-{name}"
        d.mkdir(parents=True, exist_ok=True)
        saved_files = []
        skipped = 0
        for f in item.get("files", []):
            rel = str(f.get("path") or f.get("target") or "").replace("\\", "/").lstrip("/")
            content = f.get("content")
            if not rel or content is None:
                continue
            if SKIP in rel:
                skipped += 1
                continue
            dest = d / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(content, encoding="utf-8")
            saved_files.append(rel)

        meta = {
            "id": f"knowns-{name}",
            "name": item.get("title") or it.get("title") or name,
            "registry_name": name,
            "category": "knowns-ui",
            "kind": "tsx",
            "description": item.get("description") or it.get("description") or "",
            "type": item.get("type") or it.get("type") or "registry:ui",
            "tags": [t for t in (item.get("categories") or it.get("categories") or [])],
            "license": "MIT (Knowns Lab)",
            "source_url": f"https://ui.knowns.dev/docs/{name}",
            "registry_url": f"{BASE}/r/{name}.json",
            "author": "Knowns Lab",
            "dependencies": item.get("dependencies") or [],
            "registryDependencies": item.get("registryDependencies") or [],
            "files": saved_files,
            "files_skipped_trademark": skipped,
        }
        (d / "meta.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1), encoding="utf-8")
        new_records.append({
            "id": meta["id"], "name": meta["name"], "category": meta["category"],
            "kind": meta["kind"], "description": meta["description"], "tags": meta["tags"],
            "license": meta["license"], "source_url": meta["source_url"], "author": meta["author"],
        })
        if i % 25 == 0:
            print(f"  {i}/{len(items)}...")
        time.sleep(0.05)

    idx_path.write_text(json.dumps(keep + new_records, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"harvested: {len(new_records)} | failed: {len(failed)} {failed[:5]}")
    print(f"index.json total: {len(keep) + len(new_records)}")


if __name__ == "__main__":
    main()
