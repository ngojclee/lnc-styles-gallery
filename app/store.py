import html
import json
import os
import re
import threading
import time
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from app.config import settings

IMAGE_SUFFIXES: Tuple[str, ...] = (".jpg", ".jpeg", ".png", ".webp", ".avif", ".gif", ".svg")


COLLECTION_DEFINITIONS: List[Dict[str, Any]] = [
    {
        "id": "lnc-style",
        "name": "LNC Style",
        "description": "In-house lengoc.me design systems — internal app shells, blueprint lite/full themes, LN UI Engine, and LNC-branded palettes.",
        "kind": "category",
        "accent": "#5B2E63",
        "keywords": [
            "lnc", "lengoc", "ln ui engine", "paper-sketch",
        ],
    },
    {
        "id": "editorial",
        "name": "Editorial & Publishing",
        "description": "High-contrast serif typography, magazine layouts, literary journals, and long-form reading experiences.",
        "kind": "category",
        "accent": "#f59e0b",
        "keywords": [
            "editorial", "magazine", "serif", "publication", "journal", "typography",
            "blog", "news", "story", "literary", "book", "reading", "press", "essay", "column",
        ],
    },
    {
        "id": "terminal-dev",
        "name": "Terminal & Dev-Tool",
        "description": "Monospace precision, CLI aesthetics, developer documentation, infrastructure dashboards, and code-first interfaces.",
        "kind": "category",
        "accent": "#10b981",
        "keywords": [
            "terminal", "developer", "cli", "code", "monospace", "devops", "api", "sdk",
            "git", "cloud", "infra", "engineering", "debug", "database", "technical",
            "compiler", "command", "console", "deploy", "server",
        ],
    },
    {
        "id": "luxury",
        "name": "Luxury & High-Craft",
        "description": "Refined spacing, bespoke fashion, architecture studios, gallery presentation, and understated restraint.",
        "kind": "category",
        "accent": "#d4af37",
        "keywords": [
            "luxury", "fashion", "atelier", "bespoke", "architect", "jewelry", "premium",
            "watch", "refined", "elegance", "couture", "gallery", "museum", "craft",
            "heritage", "monolith", "scented", "fragrance", "curated",
        ],
    },
    {
        "id": "dark-hardware",
        "name": "Dark Hardware & Industrial",
        "description": "Tactile dark surfaces, industrial design, audio/synth gear, aerospace, robotics, and precision instruments.",
        "kind": "category",
        "accent": "#ef4444",
        "keywords": [
            "hardware", "industrial", "audio", "synth", "robotics", "device", "tactile",
            "precision", "aerospace", "automotive", "camera", "instrument", "cyber",
            "mechanical", "sound", "speaker", "drone", "electric", "metal",
        ],
    },
    {
        "id": "warm-minimal",
        "name": "Warm Minimal & Organic",
        "description": "Soft neutral palettes, calm breathing room, wellness, interior studios, and human-centric minimalism.",
        "kind": "category",
        "accent": "#d97706",
        "keywords": [
            "warm", "minimal", "calm", "organic", "wellness", "natural", "soft", "earthy",
            "mindful", "interior", "ceramic", "botanical", "clean", "gentle", "quiet",
            "scandinavian", "linen", "stone", "zen", "health",
        ],
    },
    {
        "id": "playful",
        "name": "Playful & Expressive",
        "description": "Vibrant color blocking, neo-brutalist accents, illustration-heavy layouts, retro pop, and energetic motion.",
        "kind": "category",
        "accent": "#ec4899",
        "keywords": [
            "playful", "vibrant", "bold", "colorful", "creative", "game", "fun",
            "illustration", "retro", "pop", "quirky", "animated", "social", "sticker",
            "brutalist", "bright", "joyful", "whimsical", "candy", "arcade",
        ],
    },
    {
        "id": "ai-saas",
        "name": "AI & Modern SaaS",
        "description": "Intelligent workflows, copilot interfaces, productivity platforms, analytics dashboards, and B2B software.",
        "kind": "category",
        "accent": "#6366f1",
        "keywords": [
            "ai", "agent", "llm", "neural", "copilot", "saas", "workflow", "productivity",
            "automation", "analytics", "workspace", "platform", "dashboard", "assistant",
            "intelligence", "model", "chat", "search", "collaboration", "team",
        ],
    },
    {
        "id": "fintech-crypto",
        "name": "Fintech & Web3",
        "description": "High-trust financial interfaces, modern banking, payment rails, trading terminals, and crypto/Web3 protocols.",
        "kind": "category",
        "accent": "#06b6d4",
        "keywords": [
            "fintech", "bank", "payment", "crypto", "web3", "trading", "finance", "wallet",
            "wealth", "invest", "card", "ledger", "defi", "money", "capital", "currency",
            "blockchain", "exchange", "treasury", "tax",
        ],
    },
    {
        "id": "portfolio-agency",
        "name": "Portfolio & Design Studio",
        "description": "Showcase grids, design agencies, creative directors, typography-led case studies, and interactive portfolios.",
        "kind": "category",
        "accent": "#8b5cf6",
        "keywords": [
            "portfolio", "agency", "designer", "studio", "showcase", "case study",
            "branding", "digital", "freelance", "director", "photography", "film",
            "motion", "visual", "collective",
        ],
    },
    {
        "id": "ecommerce-retail",
        "name": "E-Commerce & Consumer",
        "description": "Direct-to-consumer storefronts, product storytelling, food & beverage, apparel, and tactile merchandising.",
        "kind": "category",
        "accent": "#14b8a6",
        "keywords": [
            "shop", "store", "commerce", "product", "retail", "food", "beverage", "coffee",
            "apparel", "skincare", "home", "lifestyle", "brand", "kitchen", "furniture",
            "drink", "beauty", "wear", "market",
        ],
    },
]


def _classify_style(title: str, tagline: str, theme: str, source_url: str) -> List[str]:
    haystack = f"{title} {tagline} {source_url}".lower()
    matched: List[str] = []
    for col in COLLECTION_DEFINITIONS:
        for kw in col["keywords"]:
            if kw in haystack:
                matched.append(col["id"])
                break
    if not matched:
        if theme == "dark":
            matched.append("ai-saas")
        else:
            matched.append("warm-minimal")
    return matched


def _extract_html_title_and_desc(html_path: Path) -> Tuple[str, str]:
    try:
        with html_path.open("r", encoding="utf-8", errors="ignore") as f:
            head_chunk = f.read(12288)
        title_match = re.search(r"<title[^>]*>(.*?)</title>", head_chunk, re.IGNORECASE | re.DOTALL)
        title = ""
        if title_match:
            title = html.unescape(re.sub(r"\s+", " ", title_match.group(1)).strip())
        if not title:
            h1_match = re.search(r"<h1[^>]*>(.*?)</h1>", head_chunk, re.IGNORECASE | re.DOTALL)
            if h1_match:
                raw_h1 = re.sub(r"<[^>]+>", "", h1_match.group(1))
                title = html.unescape(re.sub(r"\s+", " ", raw_h1).strip())
        desc_match = re.search(
            r'<meta\s+[^>]*name=["\']description["\'][^>]*content=["\'](.*?)["\']',
            head_chunk,
            re.IGNORECASE | re.DOTALL,
        )
        desc = html.unescape(re.sub(r"\s+", " ", desc_match.group(1)).strip()) if desc_match else ""
        return title, desc
    except Exception:
        return "", ""


def _humanize_slug(slug: str) -> str:
    cleaned = re.sub(r"^\d+[-_.]*", "", slug)
    cleaned = cleaned.replace("-", " ").replace("_", " ").strip()
    return cleaned.title() if cleaned else slug


class StyleLibraryStore:
    def __init__(self) -> None:
        self._lock = threading.RLock()
        self.styles: List[Dict[str, Any]] = []
        self.styles_by_id: Dict[str, Dict[str, Any]] = {}
        self.collections: List[Dict[str, Any]] = []
        self.collections_by_id: Dict[str, Dict[str, Any]] = {}
        self.templates: List[Dict[str, Any]] = []
        self.templates_by_id: Dict[str, Dict[str, Any]] = {}
        self._design_cache: Dict[str, str] = {}
        self._meta_cache: Dict[str, Dict[str, Any]] = {}
        self._preview_cache: Dict[str, str] = {}
        self.last_indexed_at: Optional[float] = None
        self.last_index_duration_ms: int = 0
        self.refero_dir: Path = settings.resolve_refero_dir()
        self.prompt_ui_dir: Path = settings.resolve_prompt_ui_dir()
        self.index_warnings: List[str] = []

    def _ensure_indexed(self) -> None:
        if self.last_indexed_at is None:
            self.reindex()

    def reindex(self) -> Dict[str, Any]:
        t0 = time.perf_counter()
        refero_dir = settings.resolve_refero_dir()
        prompt_ui_dir = settings.resolve_prompt_ui_dir()
        warnings: List[str] = []

        styles_list, styles_map, preview_map = self._scan_refero_styles(refero_dir, warnings)
        collections_list, collections_map = self._build_collections(refero_dir, styles_list, warnings)
        templates_list, templates_map = self._scan_prompt_ui_templates(prompt_ui_dir, warnings)

        duration_ms = int((time.perf_counter() - t0) * 1000)

        with self._lock:
            self.refero_dir = refero_dir
            self.prompt_ui_dir = prompt_ui_dir
            self.styles = styles_list
            self.styles_by_id = styles_map
            self.collections = collections_list
            self.collections_by_id = collections_map
            self.templates = templates_list
            self.templates_by_id = templates_map
            self._design_cache.clear()
            self._meta_cache.clear()
            self._preview_cache = preview_map
            self.last_indexed_at = time.time()
            self.last_index_duration_ms = duration_ms
            self.index_warnings = warnings

        return self.status()

    def _inspect_style_dir(
        self, style_dir: Path, raw_preview: Optional[str] = None
    ) -> Tuple[bool, Optional[Path]]:
        has_design_file = False
        preview_path: Optional[Path] = None
        raw_prev_name = os.path.basename(str(raw_preview or "").strip()).lower()
        try:
            if not style_dir.exists() or not style_dir.is_dir():
                return False, None
            with os.scandir(style_dir) as it:
                for entry in it:
                    if not entry.is_file():
                        continue
                    name_l = entry.name.lower()
                    if name_l == "design.md":
                        has_design_file = True
                    else:
                        _, ext = os.path.splitext(name_l)
                        if ext in IMAGE_SUFFIXES:
                            if raw_prev_name and name_l == raw_prev_name:
                                preview_path = Path(entry.path)
                            elif name_l.startswith("preview.") and preview_path is None:
                                preview_path = Path(entry.path)
        except Exception:
            pass
        return has_design_file, preview_path

    def _scan_refero_styles(
        self, refero_dir: Path, warnings: List[str]
    ) -> Tuple[List[Dict[str, Any]], Dict[str, Dict[str, Any]], Dict[str, str]]:
        styles_list: List[Dict[str, Any]] = []
        styles_map: Dict[str, Dict[str, Any]] = {}
        preview_map: Dict[str, str] = {}

        index_file = refero_dir / "index.json"
        styles_subdir = refero_dir / "styles"

        existing_style_dirs: set[str] = set()
        try:
            if styles_subdir.exists() and styles_subdir.is_dir():
                with os.scandir(styles_subdir) as it:
                    for entry in it:
                        if entry.is_dir():
                            existing_style_dirs.add(entry.name)
        except Exception as exc:
            warnings.append(f"Could not scan styles directory {styles_subdir}: {exc}")

        raw_records: List[Dict[str, Any]] = []
        if index_file.exists():
            try:
                raw_data = json.loads(index_file.read_text(encoding="utf-8", errors="replace"))
                if isinstance(raw_data, list):
                    raw_records = [r for r in raw_data if isinstance(r, dict)]
                elif isinstance(raw_data, dict):
                    for key in ("styles", "items", "data", "results"):
                        if isinstance(raw_data.get(key), list):
                            raw_records = [r for r in raw_data[key] if isinstance(r, dict)]
                            break
            except Exception as exc:
                warnings.append(f"Failed to parse {index_file}: {exc}")
        else:
            warnings.append(f"Refero index.json not found at {index_file}")

        seen_ids: set[str] = set()
        for item in raw_records:
            style_id = str(item.get("id") or item.get("uuid") or item.get("slug") or "").strip()
            if not style_id or style_id in seen_ids:
                continue
            seen_ids.add(style_id)
            record, prev_path = self._normalize_style_record(
                style_id, item, existing_style_dirs, styles_subdir
            )
            if prev_path is not None:
                preview_map[style_id] = str(prev_path)
            styles_list.append(record)
            styles_map[style_id] = record

        # Discover any newly harvested style folders on disk that are not yet in index.json
        extra_dirs = sorted(existing_style_dirs - seen_ids)
        for style_id in extra_dirs:
            meta_path = styles_subdir / style_id / "meta.json"
            meta_obj: Dict[str, Any] = {}
            if meta_path.exists():
                try:
                    meta_obj = json.loads(meta_path.read_text(encoding="utf-8", errors="replace"))
                except Exception:
                    meta_obj = {}
            record, prev_path = self._normalize_style_record(
                style_id, meta_obj, existing_style_dirs, styles_subdir
            )
            if prev_path is not None:
                preview_map[style_id] = str(prev_path)
            styles_list.append(record)
            styles_map[style_id] = record

        return styles_list, styles_map, preview_map

    def _normalize_style_record(
        self,
        style_id: str,
        raw: Dict[str, Any],
        existing_style_dirs: set[str],
        styles_subdir: Optional[Path] = None,
    ) -> Tuple[Dict[str, Any], Optional[Path]]:
        title = str(raw.get("title") or raw.get("name") or style_id).strip()
        tagline = str(raw.get("tagline") or raw.get("description") or raw.get("summary") or "").strip()
        raw_theme = str(raw.get("theme") or raw.get("color_scheme") or "").strip().lower()
        if raw_theme not in ("light", "dark"):
            lower_blob = f"{title} {tagline}".lower()
            raw_theme = "dark" if any(w in lower_blob for w in ("dark", "night", "obsidian", "noir", "black", "midnight")) else "light"

        url = str(raw.get("url") or f"https://styles.refero.design/styles/{style_id}").strip()
        source_url = str(
            raw.get("source_url")
            or raw.get("sourceUrl")
            or raw.get("website")
            or raw.get("site_url")
            or ""
        ).strip()

        categories = _classify_style(title, tagline, raw_theme, source_url)
        has_folder = (style_id in existing_style_dirs) if existing_style_dirs else True

        preview_path: Optional[Path] = None
        if has_folder and styles_subdir is not None:
            _, preview_path = self._inspect_style_dir(
                styles_subdir / style_id,
                raw_preview=raw.get("preview"),
            )

        has_preview = preview_path is not None
        preview_url = f"/api/styles/{style_id}/preview" if has_preview else None
        primary_cat = categories[0] if categories else "warm-minimal"

        return (
            {
                "id": style_id,
                "title": title,
                "tagline": tagline,
                "theme": raw_theme,
                "url": url,
                "source_url": source_url,
                "categories": categories,
                "category": primary_cat,
                "category_label": _humanize_slug(primary_cat),
                "has_design": has_folder,
                "has_meta": has_folder,
                "has_preview": has_preview,
                "preview_url": preview_url,
            },
            preview_path,
        )

    def _build_collections(
        self,
        refero_dir: Path,
        styles_list: List[Dict[str, Any]],
        warnings: List[str],
    ) -> Tuple[List[Dict[str, Any]], Dict[str, Dict[str, Any]]]:
        collections_list: List[Dict[str, Any]] = []
        collections_map: Dict[str, Dict[str, Any]] = {}

        by_cat: Dict[str, List[Dict[str, Any]]] = {c["id"]: [] for c in COLLECTION_DEFINITIONS}
        dark_styles: List[Dict[str, Any]] = []
        light_styles: List[Dict[str, Any]] = []

        for s in styles_list:
            if s["theme"] == "dark":
                dark_styles.append(s)
            else:
                light_styles.append(s)
            for cat_id in s.get("categories", []):
                if cat_id in by_cat:
                    by_cat[cat_id].append(s)

        def _pick_samples(members: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
            prioritized = sorted(members, key=lambda m: not bool(m.get("has_preview")))
            return [
                {
                    "id": m["id"],
                    "title": m["title"],
                    "tagline": m["tagline"],
                    "theme": m["theme"],
                    "source_url": m["source_url"],
                    "has_preview": bool(m.get("has_preview")),
                    "preview_url": m.get("preview_url"),
                }
                for m in prioritized[:6]
            ]

        for col_def in COLLECTION_DEFINITIONS:
            cid = col_def["id"]
            members = by_cat.get(cid, [])
            col_obj = {
                "id": cid,
                "name": col_def["name"],
                "description": col_def["description"],
                "kind": col_def["kind"],
                "type": col_def["kind"],
                "accent": col_def["accent"],
                "count": len(members),
                "sample_styles": _pick_samples(members),
                "style_ids": [m["id"] for m in members],
            }
            collections_list.append(col_obj)
            collections_map[cid] = col_obj

        theme_collections = [
            {
                "id": "theme-dark",
                "name": "Dark Mode Systems",
                "description": "All dark-themed design systems featuring deep surfaces, luminous accents, and low-glare contrast tokens.",
                "kind": "theme",
                "accent": "#38bdf8",
                "members": dark_styles,
            },
            {
                "id": "theme-light",
                "name": "Light Mode Systems",
                "description": "All light-themed design systems featuring daylight surfaces, crisp ink hierarchy, and subtle shadows.",
                "kind": "theme",
                "accent": "#fbbf24",
                "members": light_styles,
            },
        ]
        for tc in theme_collections:
            members = tc["members"]
            col_obj = {
                "id": tc["id"],
                "name": tc["name"],
                "description": tc["description"],
                "kind": tc["kind"],
                "type": tc["kind"],
                "accent": tc["accent"],
                "count": len(members),
                "sample_styles": _pick_samples(members),
                "style_ids": [m["id"] for m in members],
            }
            collections_list.append(col_obj)
            collections_map[tc["id"]] = col_obj

        # Optionally inspect sitemaps/collections.xml if present
        sitemap_file = refero_dir / "sitemaps" / "collections.xml"
        if sitemap_file.exists():
            try:
                tree = ET.parse(sitemap_file)
                root = tree.getroot()
                for elem in root.iter():
                    if elem.tag.endswith("loc") and elem.text:
                        loc = elem.text.strip()
                        slug = loc.rstrip("/").split("/")[-1]
                        if slug and slug not in collections_map and slug != "collections":
                            col_id = f"sitemap-{slug}"
                            matched_members = [
                                s for s in styles_list
                                if slug.replace("-", " ").lower() in f"{s['title']} {s['tagline']}".lower()
                            ]
                            col_obj = {
                                "id": col_id,
                                "name": _humanize_slug(slug),
                                "description": f"Mirrored Refero sitemap collection ({loc}).",
                                "kind": "sitemap",
                                "type": "sitemap",
                                "accent": "#a855f7",
                                "count": len(matched_members),
                                "sample_styles": _pick_samples(matched_members),
                                "style_ids": [m["id"] for m in matched_members],
                            }
                            collections_list.append(col_obj)
                            collections_map[col_id] = col_obj
            except Exception as exc:
                warnings.append(f"Optional sitemaps/collections.xml parse skipped: {exc}")

        return collections_list, collections_map

    def _scan_prompt_ui_templates(
        self, prompt_ui_dir: Path, warnings: List[str]
    ) -> Tuple[List[Dict[str, Any]], Dict[str, Dict[str, Any]]]:
        templates_list: List[Dict[str, Any]] = []
        templates_map: Dict[str, Dict[str, Any]] = {}

        if not prompt_ui_dir.exists():
            warnings.append(f"Prompt-driven-ui directory not found at {prompt_ui_dir}")
            return templates_list, templates_map

        thumbs_index = self._index_auxiliary_files(prompt_ui_dir / "thumbs", (".png", ".jpg", ".jpeg", ".webp", ".svg"))
        prompts_vi_index = self._index_auxiliary_files(prompt_ui_dir / "prompts", (".md", ".txt"))
        prompts_en_index = self._index_auxiliary_files(prompt_ui_dir / "prompts-en", (".md", ".txt"))

        sources = [
            ("en", prompt_ui_dir / "websites-en", prompts_en_index, prompts_vi_index),
            ("vi", prompt_ui_dir / "websites", prompts_vi_index, prompts_en_index),
        ]

        for lang, web_dir, primary_prompts, fallback_prompts in sources:
            if not web_dir.exists():
                continue
            html_files: List[Path] = []
            try:
                for entry in sorted(web_dir.iterdir(), key=lambda p: p.name.lower()):
                    if entry.is_file() and entry.suffix.lower() in (".html", ".htm"):
                        html_files.append(entry)
                    elif entry.is_dir():
                        idx = entry / "index.html"
                        if idx.exists():
                            html_files.append(idx)
                        else:
                            sub_htmls = sorted(entry.glob("*.html"))
                            if sub_htmls:
                                html_files.append(sub_htmls[0])
            except Exception as exc:
                warnings.append(f"Error scanning {web_dir}: {exc}")
                continue

            for idx_num, html_path in enumerate(html_files, start=1):
                stem = html_path.parent.name if html_path.name.lower() == "index.html" else html_path.stem
                num_match = re.match(r"^(\d+)", stem)
                number = int(num_match.group(1)) if num_match else idx_num
                template_id = f"{lang}-{stem}"

                extracted_title, extracted_desc = _extract_html_title_and_desc(html_path)
                title = extracted_title or _humanize_slug(stem)

                thumb_path = self._match_aux_file(stem, number, lang, thumbs_index)
                prompt_path = (
                    self._match_aux_file(stem, number, lang, primary_prompts)
                    or self._match_aux_file(stem, number, lang, fallback_prompts)
                )

                prompt_preview = extracted_desc
                if prompt_path and prompt_path.exists():
                    try:
                        raw_p = prompt_path.read_text(encoding="utf-8", errors="ignore").strip()
                        first_line = re.sub(r"\s+", " ", raw_p)[:240]
                        if first_line:
                            prompt_preview = first_line
                    except Exception:
                        pass

                record = {
                    "id": template_id,
                    "slug": stem,
                    "number": number,
                    "num": number,
                    "lang": lang,
                    "title": title,
                    "filename": html_path.name,
                    "description": extracted_desc,
                    "prompt_preview": prompt_preview,
                    "prompt_excerpt": prompt_preview,
                    "has_prompt": bool(prompt_path and prompt_path.exists()),
                    "has_thumb": bool(thumb_path and thumb_path.exists()),
                    "has_thumb_file": bool(thumb_path and thumb_path.exists()),
                    "html_url": f"/api/templates/{template_id}/html",
                    "thumb_url": f"/api/templates/{template_id}/thumb",
                    "_html_path": str(html_path),
                    "_thumb_path": str(thumb_path) if thumb_path else "",
                    "_prompt_path": str(prompt_path) if prompt_path else "",
                }
                templates_list.append(record)
                templates_map[template_id] = record

        templates_list.sort(key=lambda r: (0 if r["lang"] == "en" else 1, r["number"], r["slug"]))
        return templates_list, templates_map

    @staticmethod
    def _index_auxiliary_files(directory: Path, suffixes: Tuple[str, ...]) -> Dict[str, Path]:
        result: Dict[str, Path] = {}
        if not directory.exists() or not directory.is_dir():
            return result
        try:
            for entry in sorted(directory.rglob("*"), key=lambda p: ("landscape" in p.parts, p.name)):
                if entry.is_file() and entry.suffix.lower() in suffixes:
                    stem_lower = entry.stem.lower()
                    result[stem_lower] = entry
                    num_m = re.match(r"^(\d+)", stem_lower)
                    if num_m:
                        num_int = int(num_m.group(1))
                        result.setdefault(f"__num_{num_int}", entry)
        except Exception:
            pass
        return result

    @staticmethod
    def _match_aux_file(stem: str, number: int, lang: str, index: Dict[str, Path]) -> Optional[Path]:
        if not index:
            return None
        s_low = stem.lower()
        for candidate in (
            s_low,
            f"{lang}-{s_low}",
            f"{s_low}-{lang}",
            f"{number:02d}",
            f"{number:03d}",
            f"{number}",
            f"__num_{number}",
        ):
            if candidate in index:
                return index[candidate]
        return None

    def public_template_dict(self, tpl: Dict[str, Any]) -> Dict[str, Any]:
        return {k: v for k, v in tpl.items() if not k.startswith("_")}

    def get_style(self, style_id: str, include_design: bool = True) -> Optional[Dict[str, Any]]:
        self._ensure_indexed()
        with self._lock:
            base = self.styles_by_id.get(style_id)
            refero_dir = self.refero_dir
        if not base:
            return None

        result = dict(base)
        # Re-check preview if not cached yet in case harvest_images.py wrote it recently
        if not result.get("has_preview"):
            prev_path = self.get_style_preview_path(style_id, refero_dir)
            if prev_path is not None:
                result["has_preview"] = True
                result["preview_url"] = f"/api/styles/{style_id}/preview"

        if include_design:
            result["design_md"] = self.get_design_markdown(style_id, refero_dir)
            result["meta"] = self.get_style_meta(style_id, refero_dir)
            result["shots"] = self.list_style_shots(style_id, refero_dir)
        return result

    def get_style_preview_path(self, style_id: str, refero_dir: Optional[Path] = None) -> Optional[Path]:
        self._ensure_indexed()
        with self._lock:
            cached = self._preview_cache.get(style_id)
            r_dir = refero_dir or self.refero_dir

        if cached:
            p_cached = Path(cached)
            if p_cached.exists() and p_cached.is_file():
                return p_cached

        style_dir = r_dir / "styles" / style_id
        meta_obj = self.get_style_meta(style_id, r_dir)
        _, found_path = self._inspect_style_dir(style_dir, raw_preview=meta_obj.get("preview"))
        if found_path is not None and found_path.exists() and found_path.is_file():
            with self._lock:
                self._preview_cache[style_id] = str(found_path)
                if style_id in self.styles_by_id:
                    self.styles_by_id[style_id]["has_preview"] = True
                    self.styles_by_id[style_id]["preview_url"] = f"/api/styles/{style_id}/preview"
            return found_path
        return None

    def _resolve_style_shots(self, style_id: str, refero_dir: Optional[Path] = None) -> List[Tuple[int, Path]]:
        self._ensure_indexed()
        with self._lock:
            r_dir = refero_dir or self.refero_dir

        style_dir = r_dir / "styles" / style_id
        shots_dir = style_dir / "shots"
        raw_files: List[Path] = []
        try:
            if shots_dir.exists() and shots_dir.is_dir():
                with os.scandir(shots_dir) as it:
                    for entry in it:
                        if entry.is_file():
                            _, ext = os.path.splitext(entry.name.lower())
                            if ext in IMAGE_SUFFIXES:
                                raw_files.append(Path(entry.path))
        except Exception:
            pass

        if not raw_files:
            meta_obj = self.get_style_meta(style_id, r_dir)
            meta_shots = meta_obj.get("shots")
            if isinstance(meta_shots, list):
                for item in meta_shots:
                    rel = str(item.get("file") or item.get("path") or item if isinstance(item, dict) else item).strip()
                    if rel:
                        cand = style_dir / rel
                        if not cand.exists():
                            cand = shots_dir / os.path.basename(rel)
                        if cand.exists() and cand.is_file():
                            raw_files.append(cand)

        raw_files.sort(key=lambda p: p.name.lower())
        resolved: List[Tuple[int, Path]] = []
        used_nums: set[int] = set()
        for idx, fpath in enumerate(raw_files, start=1):
            m = re.search(r"(\d+)", fpath.stem)
            n_val = int(m.group(1)) if m else idx
            if n_val in used_nums or n_val <= 0:
                n_val = idx
            while n_val in used_nums:
                n_val += 1
            used_nums.add(n_val)
            resolved.append((n_val, fpath))

        resolved.sort(key=lambda pair: pair[0])
        return resolved

    def list_style_shots(self, style_id: str, refero_dir: Optional[Path] = None) -> List[Dict[str, Any]]:
        resolved = self._resolve_style_shots(style_id, refero_dir)
        return [
            {
                "n": n_val,
                "url": f"/api/styles/{style_id}/shots/{n_val}",
                "filename": fpath.name,
            }
            for n_val, fpath in resolved
        ]

    def get_style_shot_path(self, style_id: str, n: str, refero_dir: Optional[Path] = None) -> Optional[Path]:
        n_clean = str(n or "").strip()
        if not n_clean:
            return None
        resolved = self._resolve_style_shots(style_id, refero_dir)
        n_lower = n_clean.lower()
        n_stem = os.path.splitext(n_lower)[0]
        n_int: Optional[int] = int(n_stem) if n_stem.isdigit() else None

        for shot_num, fpath in resolved:
            if n_int is not None and shot_num == n_int:
                return fpath
            if fpath.name.lower() == n_lower or fpath.stem.lower() == n_stem:
                return fpath
        return None

    def get_design_markdown(self, style_id: str, refero_dir: Optional[Path] = None) -> str:
        with self._lock:
            if style_id in self._design_cache:
                return self._design_cache[style_id]
            r_dir = refero_dir or self.refero_dir

        design_path = r_dir / "styles" / style_id / "DESIGN.md"
        content = ""
        try:
            if design_path.exists():
                content = design_path.read_text(encoding="utf-8", errors="replace")
        except Exception:
            content = ""

        if content:
            with self._lock:
                if len(self._design_cache) < 2000:
                    self._design_cache[style_id] = content
        return content

    def get_style_meta(self, style_id: str, refero_dir: Optional[Path] = None) -> Dict[str, Any]:
        with self._lock:
            if style_id in self._meta_cache:
                return self._meta_cache[style_id]
            r_dir = refero_dir or self.refero_dir

        meta_path = r_dir / "styles" / style_id / "meta.json"
        meta_obj: Dict[str, Any] = {}
        try:
            if meta_path.exists():
                meta_obj = json.loads(meta_path.read_text(encoding="utf-8", errors="replace"))
        except Exception:
            meta_obj = {}

        if meta_obj:
            with self._lock:
                if len(self._meta_cache) < 2000:
                    self._meta_cache[style_id] = meta_obj
        return meta_obj

    def list_styles(
        self,
        q: Optional[str] = None,
        theme: Optional[str] = None,
        category: Optional[str] = None,
        has_design: Optional[bool] = None,
        limit: int = 48,
        offset: int = 0,
    ) -> Dict[str, Any]:
        self._ensure_indexed()
        with self._lock:
            all_styles = list(self.styles)
            collections_map = dict(self.collections_by_id)

        filtered = all_styles
        if theme and theme.lower() in ("light", "dark"):
            t_norm = theme.lower()
            filtered = [s for s in filtered if s.get("theme") == t_norm]

        if category and category.lower() != "all":
            cat_norm = category.lower().strip()
            if cat_norm in collections_map:
                allowed_ids = set(collections_map[cat_norm].get("style_ids", []))
                filtered = [s for s in filtered if s["id"] in allowed_ids]
            else:
                filtered = [s for s in filtered if cat_norm in s.get("categories", [])]

        if has_design is not None:
            filtered = [s for s in filtered if bool(s.get("has_design")) == has_design]

        if q and q.strip():
            terms = [t for t in q.lower().strip().split() if t]
            filtered = [
                s for s in filtered
                if all(
                    term in f"{s['title']} {s['tagline']} {s['source_url']} {s['id']} {' '.join(s.get('categories', []))}".lower()
                    for term in terms
                )
            ]

        total = len(filtered)
        limit = max(1, min(limit, 500))
        offset = max(0, offset)
        page_items = filtered[offset : offset + limit]

        theme_counts = {"light": 0, "dark": 0}
        for s in all_styles:
            t_val = s.get("theme", "light")
            theme_counts[t_val] = theme_counts.get(t_val, 0) + 1

        category_facets = [
            {
                "id": c["id"],
                "label": c["name"],
                "count": collections_map.get(c["id"], {}).get("count", 0),
            }
            for c in COLLECTION_DEFINITIONS
        ]

        return {
            "total": total,
            "limit": limit,
            "offset": offset,
            "themes": theme_counts,
            "facets": {
                "themes": theme_counts,
                "categories": category_facets,
            },
            "items": page_items,
        }

    def search(
        self,
        query: str,
        theme: Optional[str] = None,
        category: Optional[str] = None,
        include_content: bool = True,
        limit: int = 30,
    ) -> Dict[str, Any]:
        self._ensure_indexed()
        q_clean = (query or "").strip()
        limit = max(1, min(limit, 200))
        if not q_clean:
            base = self.list_styles(theme=theme, category=category, limit=limit, offset=0)
            return {
                "query": "",
                "total_styles": base["total"],
                "styles": base["items"],
                "templates": [],
            }

        terms = [t for t in q_clean.lower().split() if t]
        with self._lock:
            candidates = list(self.styles)
            collections_map = dict(self.collections_by_id)
            templates = list(self.templates)
            r_dir = self.refero_dir

        if theme and theme.lower() in ("light", "dark"):
            t_norm = theme.lower()
            candidates = [s for s in candidates if s.get("theme") == t_norm]

        if category and category.lower() != "all":
            cat_norm = category.lower().strip()
            if cat_norm in collections_map:
                allowed = set(collections_map[cat_norm].get("style_ids", []))
                candidates = [s for s in candidates if s["id"] in allowed]
            else:
                candidates = [s for s in candidates if cat_norm in s.get("categories", [])]

        scored_styles: List[Tuple[int, Dict[str, Any]]] = []
        content_reads = 0
        max_uncached_reads = 120  # Bound per-query disk reads on cold cache

        for s in candidates:
            title_l = s["title"].lower()
            tagline_l = s["tagline"].lower()
            source_l = s["source_url"].lower()
            cats_l = " ".join(s.get("categories", [])).lower()
            meta_blob = f"{title_l} {tagline_l} {source_l} {s['id'].lower()} {cats_l}"

            score = 0
            matched_all_in_meta = all(term in meta_blob for term in terms)
            if matched_all_in_meta:
                score += 50
                for term in terms:
                    if term in title_l:
                        score += 40
                    if term in tagline_l:
                        score += 20
                    if term in source_l:
                        score += 10

            snippet = ""
            if include_content and (score == 0 or len(scored_styles) < limit):
                cached_md = self._design_cache.get(s["id"])
                if cached_md is None and content_reads < max_uncached_reads and s.get("has_design"):
                    cached_md = self.get_design_markdown(s["id"], r_dir)
                    content_reads += 1
                if cached_md:
                    md_lower = cached_md.lower()
                    if all(term in md_lower for term in terms):
                        score += 15
                        pos = md_lower.find(terms[0])
                        if pos >= 0:
                            start = max(0, pos - 60)
                            end = min(len(cached_md), pos + 160)
                            snippet = re.sub(r"\s+", " ", cached_md[start:end]).strip()

            if score > 0:
                entry = dict(s)
                if snippet:
                    entry["match_snippet"] = snippet
                    entry["snippet"] = snippet
                entry["score"] = score
                scored_styles.append((score, entry))

        scored_styles.sort(key=lambda pair: (-pair[0], pair[1]["title"].lower()))
        matched_styles = [item for _, item in scored_styles[:limit]]

        matched_templates: List[Dict[str, Any]] = []
        for tpl in templates:
            tpl_blob = f"{tpl['title']} {tpl['slug']} {tpl['prompt_preview']} {tpl['lang']}".lower()
            if all(term in tpl_blob for term in terms):
                matched_templates.append(self.public_template_dict(tpl))
                if len(matched_templates) >= 20:
                    break

        return {
            "query": q_clean,
            "total_styles": len(scored_styles),
            "styles": matched_styles,
            "templates": matched_templates,
        }

    def list_collections(self) -> List[Dict[str, Any]]:
        self._ensure_indexed()
        with self._lock:
            return [
                {k: v for k, v in col.items() if k != "style_ids"}
                for col in self.collections
            ]

    def get_collection(self, collection_id: str, limit: int = 100, offset: int = 0) -> Optional[Dict[str, Any]]:
        self._ensure_indexed()
        with self._lock:
            col = self.collections_by_id.get(collection_id)
            if not col:
                return None
            style_ids = col.get("style_ids", [])
            items = [self.styles_by_id[sid] for sid in style_ids if sid in self.styles_by_id]

        total = len(items)
        page_items = items[max(0, offset) : max(0, offset) + max(1, min(limit, 500))]
        result = {k: v for k, v in col.items() if k != "style_ids"}
        result["total"] = total
        result["limit"] = limit
        result["offset"] = offset
        result["items"] = page_items
        result["styles"] = page_items
        return result

    def list_templates(
        self,
        lang: Optional[str] = None,
        q: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> Dict[str, Any]:
        self._ensure_indexed()
        with self._lock:
            items = list(self.templates)

        lang_counts = {"en": 0, "vi": 0}
        for t in items:
            l_code = t.get("lang", "en")
            lang_counts[l_code] = lang_counts.get(l_code, 0) + 1

        if lang and lang.lower() in ("en", "vi"):
            l_norm = lang.lower()
            items = [t for t in items if t.get("lang") == l_norm]

        if q and q.strip():
            terms = [term for term in q.lower().strip().split() if term]
            items = [
                t for t in items
                if all(
                    term in f"{t['title']} {t['slug']} {t['prompt_preview']} {t['id']}".lower()
                    for term in terms
                )
            ]

        total = len(items)
        limit = max(1, min(limit, 200))
        offset = max(0, offset)
        sliced = [self.public_template_dict(t) for t in items[offset : offset + limit]]

        return {
            "total": total,
            "limit": limit,
            "offset": offset,
            "languages": lang_counts,
            "facets": {
                "languages": lang_counts,
            },
            "items": sliced,
        }

    def get_template(self, template_id: str, include_html: bool = False) -> Optional[Dict[str, Any]]:
        self._ensure_indexed()
        with self._lock:
            tpl = self.templates_by_id.get(template_id)
            if not tpl:
                # Allow lookup by slug or number fallback
                for candidate in self.templates:
                    if candidate["slug"] == template_id or str(candidate["number"]) == template_id:
                        tpl = candidate
                        break
        if not tpl:
            return None

        result = self.public_template_dict(tpl)
        prompt_text = ""
        prompt_path_str = tpl.get("_prompt_path", "")
        if prompt_path_str:
            p_path = Path(prompt_path_str)
            if p_path.exists():
                try:
                    prompt_text = p_path.read_text(encoding="utf-8", errors="replace")
                except Exception:
                    prompt_text = ""
        result["prompt"] = prompt_text

        if include_html:
            html_path_str = tpl.get("_html_path", "")
            html_content = ""
            if html_path_str:
                h_path = Path(html_path_str)
                if h_path.exists():
                    try:
                        html_content = h_path.read_text(encoding="utf-8", errors="replace")
                    except Exception:
                        html_content = ""
            result["html"] = html_content

        return result

    def get_template_raw_paths(self, template_id: str) -> Optional[Dict[str, str]]:
        self._ensure_indexed()
        with self._lock:
            tpl = self.templates_by_id.get(template_id)
            if not tpl:
                for candidate in self.templates:
                    if candidate["slug"] == template_id:
                        tpl = candidate
                        break
            return dict(tpl) if tpl else None

    def get_licenses(self) -> Dict[str, Any]:
        self._ensure_indexed()
        with self._lock:
            styles = list(self.styles)
            templates = list(self.templates)

        attributed_sites = [
            {
                "id": s["id"],
                "title": s["title"],
                "theme": s["theme"],
                "category": s.get("category_label") or "Design System",
                "refero_url": s["url"],
                "source_url": s["source_url"],
            }
            for s in styles
        ]

        return {
            "service": "lnc-styles-gallery",
            "version": settings.version,
            "corpora": [
                {
                    "id": "refero-styles",
                    "name": "Refero Design Styles",
                    "upstream_url": "https://styles.refero.design",
                    "source": "https://styles.refero.design",
                    "license": "Curated Reference / Fair Use",
                    "total_styles": len(styles),
                    "count": len(styles),
                    "with_design_md": sum(1 for s in styles if s.get("has_design")),
                    "license_note": (
                        "Distilled design system token & layout specifications (DESIGN.md) curated from "
                        "real-world product websites via styles.refero.design. All trademarks, brand identities, "
                        "and original website designs belong to their respective product owners listed in source_url."
                    ),
                    "attribution_note": (
                        "Distilled design system token & layout specifications (DESIGN.md) curated from "
                        "real-world product websites via styles.refero.design."
                    ),
                    "terms_summary": [
                        "Each style preserves its original Refero URL and attributed product website (source_url).",
                        "DESIGN.md guides provide color tokens, typography, layout structure, and component guidelines.",
                        "All trademarks and brand assets remain the property of their respective product owners.",
                    ],
                },
                {
                    "id": "prompt-driven-ui",
                    "name": "Prompt-Driven UI HTML Templates",
                    "upstream_url": "https://github.com/vuhung16au/prompt-driven-ui",
                    "source": "https://github.com/vuhung16au/prompt-driven-ui",
                    "author": "vuhung16au",
                    "license": "MIT",
                    "total_templates": len(templates),
                    "count": len(templates),
                    "license_note": (
                        "100 single-file HTML/CSS/JS sample designs (50 English, 50 Vietnamese) with generation "
                        "prompts, distributed under the MIT License."
                    ),
                    "attribution_note": (
                        "100 single-file HTML/CSS/JS sample designs (50 English + 50 Vietnamese) by vuhung16au."
                    ),
                    "terms_summary": [
                        "Licensed under the open-source MIT License (vuhung16au/prompt-driven-ui).",
                        "Includes 50 English (websites-en/) and 50 Vietnamese (websites/) self-contained HTML templates.",
                        "Includes corresponding generation prompts and preview thumbnails.",
                    ],
                },
            ],
            "attributed_sources_count": len(attributed_sites),
            "attributed_sources_total": len(attributed_sites),
            "attributed_sources": attributed_sites,
            "attributed_sources_sample": attributed_sites,
        }

    def status(self) -> Dict[str, Any]:
        with self._lock:
            theme_counts = {"light": 0, "dark": 0}
            designs_count = 0
            for s in self.styles:
                t_val = s.get("theme", "light")
                theme_counts[t_val] = theme_counts.get(t_val, 0) + 1
                if s.get("has_design"):
                    designs_count += 1
            category_facets = [
                {
                    "id": c["id"],
                    "label": c["name"],
                    "count": self.collections_by_id.get(c["id"], {}).get("count", 0),
                }
                for c in COLLECTION_DEFINITIONS
            ]
            return {
                "status": "ok",
                "service": settings.service_name,
                "version": settings.version,
                "refero_styles_dir": str(self.refero_dir),
                "refero_styles_exists": self.refero_dir.exists(),
                "refero_dir": str(self.refero_dir),
                "refero_dir_exists": self.refero_dir.exists(),
                "prompt_ui_dir": str(self.prompt_ui_dir),
                "prompt_ui_exists": self.prompt_ui_dir.exists(),
                "prompt_ui_dir_exists": self.prompt_ui_dir.exists(),
                "styles_count": len(self.styles),
                "designs_count": designs_count,
                "previews_count": len(self._preview_cache),
                "collections_count": len(self.collections),
                "templates_count": len(self.templates),
                "cached_designs_count": len(self._design_cache),
                "last_indexed_at": self.last_indexed_at,
                "indexed_at": self.last_indexed_at,
                "last_index_duration_ms": self.last_index_duration_ms,
                "themes": theme_counts,
                "categories": category_facets,
                "warnings": list(self.index_warnings),
            }


store = StyleLibraryStore()
