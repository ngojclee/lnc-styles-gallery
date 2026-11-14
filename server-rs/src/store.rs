//! Read-only in-memory corpus indexer (`StyleLibraryStore`) — 1-to-1 Rust port
//! of `app/store.py` covering Refero Design Styles, Collections, Prompt-Driven UI
//! HTML Templates, LNC / Knowns UI Components, Full-Text Search, and Licenses.
//!
//! Strictly read-only against `${STYLES_DATA_DIR}`; never writes to corpus paths.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::RwLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};
use url::Url;

use crate::config::AppSettings;

const IMAGE_SUFFIXES: &[&str] = &[".jpg", ".jpeg", ".png", ".webp", ".avif", ".gif", ".svg"];
const VIDEO_SUFFIXES: &[&str] = &[".mp4", ".webm", ".mov", ".m4v"];
const PREVIEW_COMP_SUFFIXES: &[&str] = &[".webp", ".png", ".jpg", ".jpeg", ".gif", ".svg"];
const COMP_SOURCE_SUFFIXES: &[&str] = &[
    ".tsx", ".ts", ".jsx", ".js", ".css", ".html", ".json", ".md",
];

#[derive(Debug, Clone)]
pub struct CollectionDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub kind: &'static str,
    pub accent: &'static str,
    pub keywords: &'static [&'static str],
}

pub const COLLECTION_DEFINITIONS: &[CollectionDef] = &[
    CollectionDef {
        id: "lnc-style",
        name: "LNC Style",
        description: "In-house lengoc.me design systems — internal app shells, blueprint lite/full themes, LN UI Engine, and LNC-branded palettes.",
        kind: "category",
        accent: "#5B2E63",
        keywords: &["lnc", "lengoc", "ln ui engine", "paper-sketch"],
    },
    CollectionDef {
        id: "editorial",
        name: "Editorial & Publishing",
        description: "High-contrast serif typography, magazine layouts, literary journals, and long-form reading experiences.",
        kind: "category",
        accent: "#f59e0b",
        keywords: &[
            "editorial", "magazine", "serif", "publication", "journal", "typography", "blog",
            "news", "story", "literary", "book", "reading", "press", "essay", "column",
        ],
    },
    CollectionDef {
        id: "terminal-dev",
        name: "Terminal & Dev-Tool",
        description: "Monospace precision, CLI aesthetics, developer documentation, infrastructure dashboards, and code-first interfaces.",
        kind: "category",
        accent: "#10b981",
        keywords: &[
            "terminal", "developer", "cli", "code", "monospace", "devops", "api", "sdk", "git",
            "cloud", "infra", "engineering", "debug", "database", "technical", "compiler",
            "command", "console", "deploy", "server",
        ],
    },
    CollectionDef {
        id: "luxury",
        name: "Luxury & High-Craft",
        description: "Refined spacing, bespoke fashion, architecture studios, gallery presentation, and understated restraint.",
        kind: "category",
        accent: "#d4af37",
        keywords: &[
            "luxury", "fashion", "atelier", "bespoke", "architect", "jewelry", "premium", "watch",
            "refined", "elegance", "couture", "gallery", "museum", "craft", "heritage", "monolith",
            "scented", "fragrance", "curated",
        ],
    },
    CollectionDef {
        id: "dark-hardware",
        name: "Dark Hardware & Industrial",
        description: "Tactile dark surfaces, industrial design, audio/synth gear, aerospace, robotics, and precision instruments.",
        kind: "category",
        accent: "#ef4444",
        keywords: &[
            "hardware", "industrial", "audio", "synth", "robotics", "device", "tactile",
            "precision", "aerospace", "automotive", "camera", "instrument", "cyber", "mechanical",
            "sound", "speaker", "drone", "electric", "metal",
        ],
    },
    CollectionDef {
        id: "warm-minimal",
        name: "Warm Minimal & Organic",
        description: "Soft neutral palettes, calm breathing room, wellness, interior studios, and human-centric minimalism.",
        kind: "category",
        accent: "#d97706",
        keywords: &[
            "warm", "minimal", "calm", "organic", "wellness", "natural", "soft", "earthy",
            "mindful", "interior", "ceramic", "botanical", "clean", "gentle", "quiet",
            "scandinavian", "linen", "stone", "zen", "health",
        ],
    },
    CollectionDef {
        id: "playful",
        name: "Playful & Expressive",
        description: "Vibrant color blocking, neo-brutalist accents, illustration-heavy layouts, retro pop, and energetic motion.",
        kind: "category",
        accent: "#ec4899",
        keywords: &[
            "playful", "vibrant", "bold", "colorful", "creative", "game", "fun", "illustration",
            "retro", "pop", "quirky", "animated", "social", "sticker", "brutalist", "bright",
            "joyful", "whimsical", "candy", "arcade",
        ],
    },
    CollectionDef {
        id: "ai-saas",
        name: "AI & Modern SaaS",
        description: "Intelligent workflows, copilot interfaces, productivity platforms, analytics dashboards, and B2B software.",
        kind: "category",
        accent: "#6366f1",
        keywords: &[
            "ai", "agent", "llm", "neural", "copilot", "saas", "workflow", "productivity",
            "automation", "analytics", "workspace", "platform", "dashboard", "assistant",
            "intelligence", "model", "chat", "search", "collaboration", "team",
        ],
    },
    CollectionDef {
        id: "fintech-crypto",
        name: "Fintech & Web3",
        description: "High-trust financial interfaces, modern banking, payment rails, trading terminals, and crypto/Web3 protocols.",
        kind: "category",
        accent: "#06b6d4",
        keywords: &[
            "fintech", "bank", "payment", "crypto", "web3", "trading", "finance", "wallet",
            "wealth", "invest", "card", "ledger", "defi", "money", "capital", "currency",
            "blockchain", "exchange", "treasury", "tax",
        ],
    },
    CollectionDef {
        id: "portfolio-agency",
        name: "Portfolio & Design Studio",
        description: "Showcase grids, design agencies, creative directors, typography-led case studies, and interactive portfolios.",
        kind: "category",
        accent: "#8b5cf6",
        keywords: &[
            "portfolio", "agency", "designer", "studio", "showcase", "case study", "branding",
            "digital", "freelance", "director", "photography", "film", "motion", "visual",
            "collective",
        ],
    },
    CollectionDef {
        id: "ecommerce-retail",
        name: "E-Commerce & Consumer",
        description: "Direct-to-consumer storefronts, product storytelling, food & beverage, apparel, and tactile merchandising.",
        kind: "category",
        accent: "#14b8a6",
        keywords: &[
            "shop", "store", "commerce", "product", "retail", "food", "beverage", "coffee",
            "apparel", "skincare", "home", "lifestyle", "brand", "kitchen", "furniture", "drink",
            "beauty", "wear", "market",
        ],
    },
    CollectionDef {
        id: "platform-system",
        name: "Platform Design Systems",
        description: "Official vendor design systems — Apple HIG, Material 3, Fluent 2, Carbon, Polaris, Primer, Ant Design, and other published token specs.",
        kind: "category",
        accent: "#0ea5e9",
        keywords: &[
            "human interface",
            "material you",
            "material design",
            "fluent design",
            "fluent2",
            "carbon design",
            "carbondesignsystem",
            "polaris",
            "primer.style",
            "ant design",
            "ant.design",
            "lightning design",
            "lightningdesignsystem",
            "uswds",
            "digital.gov",
            "gov.uk",
            "govuk",
            "shadcn",
            "radix",
            "bootstrap",
        ],
    },
];

// ---------------------------------------------------------------------------
// Safe path & HTML/string helpers
// ---------------------------------------------------------------------------

/// Reject empty IDs or any segment containing traversal / separator chars.
pub fn is_safe_segment(seg: &str) -> bool {
    !seg.is_empty()
        && seg != "."
        && seg != ".."
        && !seg.contains('/')
        && !seg.contains('\\')
        && !seg.contains('\0')
        && !seg.contains("..")
}

/// Ensure a relative sub-path contains only normal non-dotfile path components and resolves inside `base_dir`.
pub fn resolve_confined_file(base_dir: &Path, rel: &str) -> Option<PathBuf> {
    if rel.contains('\\') || rel.contains(':') || rel.contains('\0') || rel.contains("..") {
        return None;
    }
    let trimmed = rel.trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let rel_path = Path::new(trimmed);
    for comp in rel_path.components() {
        match comp {
            Component::Normal(seg) => {
                if seg.to_str().map(|s| s.starts_with('.')).unwrap_or(true) {
                    return None;
                }
            }
            _ => return None,
        }
    }
    let base_canon = std::fs::canonicalize(base_dir).ok()?;
    let target_canon = std::fs::canonicalize(base_dir.join(rel_path)).ok()?;
    if target_canon != base_canon && target_canon.starts_with(&base_canon) && target_canon.is_file()
    {
        Some(target_canon)
    } else {
        None
    }
}

pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&middot;", "·")
}

pub fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn humanize_slug(slug: &str) -> String {
    let stripped = slug.trim_start_matches(|c: char| c.is_ascii_digit() || c == '-' || c == '_' || c == '.');
    let replaced = stripped.replace(['-', '_'], " ");
    let trimmed = replaced.trim();
    if trimmed.is_empty() {
        return slug.to_string();
    }
    trimmed
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_uppercase().to_string();
                    out.push_str(&chars.as_str().to_lowercase());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn leading_digits(s: &str) -> Option<usize> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<usize>().ok()
    }
}

fn first_number_in(s: &str) -> Option<usize> {
    let mut digits = String::new();
    let mut started = false;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    if digits.is_empty() {
        None
    } else {
        digits.parse::<usize>().ok()
    }
}

fn classify_style(title: &str, tagline: &str, theme: &str, source_url: &str) -> Vec<String> {
    let haystack = format!("{title} {tagline} {source_url}").to_lowercase();
    let mut matched = Vec::new();
    for col in COLLECTION_DEFINITIONS {
        if col.keywords.iter().any(|kw| haystack.contains(kw)) {
            matched.push(col.id.to_string());
        }
    }
    if matched.is_empty() {
        if theme == "dark" {
            matched.push("ai-saas".to_string());
        } else {
            matched.push("warm-minimal".to_string());
        }
    }
    matched
}

fn extract_tag_inner(chunk: &str, tag: &str) -> Option<String> {
    let lower = chunk.to_ascii_lowercase();
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let start_tag = lower.find(&open_pat)?;
    let after_open = chunk.get(start_tag + open_pat.len()..)?;
    // Ensure tag boundary (space or '>')
    let first_ch = after_open.chars().next()?;
    if first_ch != '>' && !first_ch.is_ascii_whitespace() {
        return None;
    }
    let gt = after_open.find('>')?;
    let body_and_rest = after_open.get(gt + 1..)?;
    let end_rel = body_and_rest.to_ascii_lowercase().find(&close_pat)?;
    Some(body_and_rest[..end_rel].to_string())
}

fn strip_html_tags(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn extract_meta_description(chunk: &str) -> String {
    let lower = chunk.to_ascii_lowercase();
    let mut pos = 0;
    while let Some(rel) = lower[pos..].find("<meta") {
        let start = pos + rel;
        let Some(end_rel) = lower[start..].find('>') else {
            break;
        };
        let tag_slice = &chunk[start..start + end_rel + 1];
        let tag_lower = &lower[start..start + end_rel + 1];
        if tag_lower.contains("name=\"description\"") || tag_lower.contains("name='description'") {
            if let Some(c_idx) = tag_lower.find("content=") {
                let rest = tag_slice[c_idx + 8..].trim_start();
                if let Some(quote) = rest.chars().next() {
                    if quote == '"' || quote == '\'' {
                        if let Some(end_q) = rest[1..].find(quote) {
                            return html_unescape(&collapse_ws(&rest[1..1 + end_q]));
                        }
                    }
                }
            }
        }
        pos = start + end_rel + 1;
    }
    String::new()
}

fn extract_html_title_and_desc(html_path: &Path) -> (String, String) {
    let Ok(bytes) = std::fs::read(html_path) else {
        return (String::new(), String::new());
    };
    let head_len = bytes.len().min(12_288);
    let chunk = String::from_utf8_lossy(&bytes[..head_len]);
    let mut title = extract_tag_inner(&chunk, "title")
        .map(|t| html_unescape(&collapse_ws(&t)))
        .unwrap_or_default();
    if title.is_empty() {
        if let Some(h1) = extract_tag_inner(&chunk, "h1") {
            title = html_unescape(&collapse_ws(&strip_html_tags(&h1)));
        }
    }
    let desc = extract_meta_description(&chunk);
    (title, desc)
}

fn extract_xml_locs(xml_text: &str) -> Vec<String> {
    let mut locs = Vec::new();
    let mut rest = xml_text;
    while let Some(open) = rest.find("<loc>") {
        let after = &rest[open + 5..];
        let Some(close) = after.find("</loc>") else {
            break;
        };
        let val = after[..close].trim();
        if !val.is_empty() {
            locs.push(val.to_string());
        }
        rest = &after[close + 6..];
    }
    locs
}

fn ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()))
        .unwrap_or_default()
}

fn val_str<'a>(obj: &'a Map<String, Value>, keys: &[&str]) -> &'a str {
    for k in keys {
        if let Some(s) = obj.get(*k).and_then(Value::as_str) {
            let t = s.trim();
            if !t.is_empty() {
                return t;
            }
        }
    }
    ""
}

// ---------------------------------------------------------------------------
// Store State
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct CorpusSnapshot {
    refero_dir: PathBuf,
    prompt_ui_dir: PathBuf,
    components_dir: PathBuf,
    styles: Vec<Map<String, Value>>,
    styles_by_id: HashMap<String, Map<String, Value>>,
    collections: Vec<Map<String, Value>>,
    collections_by_id: HashMap<String, Map<String, Value>>,
    templates: Vec<Map<String, Value>>,
    templates_by_id: HashMap<String, Map<String, Value>>,
    components: Vec<Map<String, Value>>,
    components_by_id: HashMap<String, Map<String, Value>>,
    preview_cache: HashMap<String, PathBuf>,
    design_cache: HashMap<String, String>,
    meta_cache: HashMap<String, Value>,
    last_indexed_at: Option<f64>,
    last_index_duration_ms: u64,
    warnings: Vec<String>,
}

pub struct StyleLibraryStore {
    settings: AppSettings,
    state: RwLock<CorpusSnapshot>,
}

impl std::fmt::Debug for StyleLibraryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StyleLibraryStore")
            .field("service", &self.settings.service_name)
            .finish()
    }
}

impl StyleLibraryStore {
    pub fn new(settings: AppSettings) -> Self {
        let snap = CorpusSnapshot {
            refero_dir: settings.resolve_refero_dir(),
            prompt_ui_dir: settings.resolve_prompt_ui_dir(),
            components_dir: settings.resolve_components_dir(),
            ..Default::default()
        };
        Self {
            settings,
            state: RwLock::new(snap),
        }
    }

    fn ensure_indexed(&self) {
        let indexed = self
            .state
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .last_indexed_at
            .is_some();
        if !indexed {
            let _ = self.reindex();
        }
    }

    pub fn reindex(&self) -> Value {
        let t0 = Instant::now();
        let refero_dir = self.settings.resolve_refero_dir();
        let prompt_ui_dir = self.settings.resolve_prompt_ui_dir();
        let components_dir = self.settings.resolve_components_dir();
        let mut warnings = Vec::new();

        let (styles, styles_by_id, preview_cache) =
            scan_refero_styles(&refero_dir, &mut warnings);
        let (collections, collections_by_id) =
            build_collections(&refero_dir, &styles, &mut warnings);
        let (templates, templates_by_id) =
            scan_prompt_ui_templates(&prompt_ui_dir, &mut warnings);
        let (components, components_by_id) =
            scan_components(&components_dir, &mut warnings);

        let duration_ms = t0.elapsed().as_millis() as u64;
        let indexed_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        {
            let mut g = self.state.write().unwrap_or_else(|p| p.into_inner());
            g.refero_dir = refero_dir;
            g.prompt_ui_dir = prompt_ui_dir;
            g.components_dir = components_dir;
            g.styles = styles;
            g.styles_by_id = styles_by_id;
            g.collections = collections;
            g.collections_by_id = collections_by_id;
            g.templates = templates;
            g.templates_by_id = templates_by_id;
            g.components = components;
            g.components_by_id = components_by_id;
            g.preview_cache = preview_cache;
            g.design_cache.clear();
            g.meta_cache.clear();
            g.last_indexed_at = Some(indexed_ts);
            g.last_index_duration_ms = duration_ms;
            g.warnings = warnings;
        }

        self.status()
    }

    pub fn status(&self) -> Value {
        let g = self.state.read().unwrap_or_else(|p| p.into_inner());
        let mut light_cnt = 0usize;
        let mut dark_cnt = 0usize;
        let mut designs_count = 0usize;
        for s in &g.styles {
            match s.get("theme").and_then(Value::as_str).unwrap_or("light") {
                "dark" => dark_cnt += 1,
                _ => light_cnt += 1,
            }
            if s.get("has_design").and_then(Value::as_bool).unwrap_or(false) {
                designs_count += 1;
            }
        }
        let categories: Vec<Value> = COLLECTION_DEFINITIONS
            .iter()
            .map(|c| {
                let cnt = g
                    .collections_by_id
                    .get(c.id)
                    .and_then(|m| m.get("count"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                json!({
                    "id": c.id,
                    "label": c.name,
                    "count": cnt,
                })
            })
            .collect();

        let ref_str = g.refero_dir.to_string_lossy().to_string();
        let ref_exists = g.refero_dir.exists();
        let pui_str = g.prompt_ui_dir.to_string_lossy().to_string();
        let pui_exists = g.prompt_ui_dir.exists();
        let comp_str = g.components_dir.to_string_lossy().to_string();
        let comp_exists = g.components_dir.exists();

        json!({
            "status": "ok",
            "service": self.settings.service_name,
            "version": self.settings.version,
            "refero_styles_dir": ref_str,
            "refero_styles_exists": ref_exists,
            "refero_dir": g.refero_dir.to_string_lossy(),
            "refero_dir_exists": ref_exists,
            "prompt_ui_dir": pui_str,
            "prompt_ui_exists": pui_exists,
            "prompt_ui_dir_exists": pui_exists,
            "components_dir": comp_str,
            "components_dir_exists": comp_exists,
            "components_count": g.components.len(),
            "styles_count": g.styles.len(),
            "designs_count": designs_count,
            "previews_count": g.preview_cache.len(),
            "collections_count": g.collections.len(),
            "templates_count": g.templates.len(),
            "cached_designs_count": g.design_cache.len(),
            "last_indexed_at": g.last_indexed_at,
            "indexed_at": g.last_indexed_at,
            "last_index_duration_ms": g.last_index_duration_ms,
            "themes": { "light": light_cnt, "dark": dark_cnt },
            "categories": categories,
            "warnings": g.warnings,
        })
    }

    pub fn refero_style_dir_exists(&self, style_id: &str) -> bool {
        if !is_safe_segment(style_id) {
            return false;
        }
        let g = self.state.read().unwrap_or_else(|p| p.into_inner());
        g.refero_dir.join("styles").join(style_id).is_dir()
    }

    pub fn list_styles(
        &self,
        q: Option<&str>,
        theme: Option<&str>,
        category: Option<&str>,
        has_design: Option<bool>,
        limit: usize,
        offset: usize,
    ) -> Value {
        self.ensure_indexed();
        let (all_styles, collections_by_id) = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            (g.styles.clone(), g.collections_by_id.clone())
        };

        let mut filtered: Vec<&Map<String, Value>> = all_styles.iter().collect();

        if let Some(t) = theme.map(|s| s.trim().to_ascii_lowercase()) {
            if t == "light" || t == "dark" {
                filtered.retain(|s| s.get("theme").and_then(Value::as_str) == Some(t.as_str()));
            }
        }

        if let Some(cat) = category
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty() && s != "all")
        {
            if let Some(col) = collections_by_id.get(&cat) {
                let allowed: HashSet<&str> = col
                    .get("style_ids")
                    .and_then(Value::as_array)
                    .map(|arr| arr.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default();
                filtered.retain(|s| {
                    s.get("id")
                        .and_then(Value::as_str)
                        .is_some_and(|id| allowed.contains(id))
                });
            } else {
                filtered.retain(|s| {
                    s.get("categories")
                        .and_then(Value::as_array)
                        .is_some_and(|arr| arr.iter().any(|c| c.as_str() == Some(cat.as_str())))
                });
            }
        }

        if let Some(hd) = has_design {
            filtered.retain(|s| s.get("has_design").and_then(Value::as_bool).unwrap_or(false) == hd);
        }

        if let Some(q_raw) = q.map(str::trim).filter(|s| !s.is_empty()) {
            let terms: Vec<String> = q_raw
                .to_lowercase()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            filtered.retain(|s| {
                let cats = s
                    .get("categories")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                let blob = format!(
                    "{} {} {} {} {}",
                    val_str(s, &["title"]),
                    val_str(s, &["tagline"]),
                    val_str(s, &["source_url"]),
                    val_str(s, &["id"]),
                    cats
                )
                .to_lowercase();
                terms.iter().all(|term| blob.contains(term))
            });
        }

        let total = filtered.len();
        let eff_limit = limit.clamp(1, 500);
        let page_items: Vec<Value> = filtered
            .into_iter()
            .skip(offset)
            .take(eff_limit)
            .cloned()
            .map(Value::Object)
            .collect();

        let mut light_cnt = 0usize;
        let mut dark_cnt = 0usize;
        for s in &all_styles {
            match s.get("theme").and_then(Value::as_str).unwrap_or("light") {
                "dark" => dark_cnt += 1,
                _ => light_cnt += 1,
            }
        }
        let category_facets: Vec<Value> = COLLECTION_DEFINITIONS
            .iter()
            .map(|c| {
                let cnt = collections_by_id
                    .get(c.id)
                    .and_then(|m| m.get("count"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                json!({
                    "id": c.id,
                    "label": c.name,
                    "count": cnt,
                })
            })
            .collect();

        json!({
            "total": total,
            "limit": eff_limit,
            "offset": offset,
            "themes": { "light": light_cnt, "dark": dark_cnt },
            "facets": {
                "themes": { "light": light_cnt, "dark": dark_cnt },
                "categories": category_facets,
            },
            "items": page_items,
        })
    }

    pub fn get_style(&self, style_id: &str, include_design: bool) -> Option<Value> {
        if !is_safe_segment(style_id) {
            return None;
        }
        self.ensure_indexed();
        let mut base = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            g.styles_by_id.get(style_id).cloned()?
        };

        if !base
            .get("has_preview")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && self.get_style_preview_path(style_id).is_some()
        {
            base.insert("has_preview".into(), Value::Bool(true));
            base.insert(
                "preview_url".into(),
                Value::String(format!("/api/styles/{style_id}/preview")),
            );
        }

        if include_design {
            base.insert(
                "design_md".into(),
                Value::String(self.get_design_markdown(style_id)),
            );
            base.insert("meta".into(), self.get_style_meta(style_id));
            base.insert("shots".into(), Value::Array(self.list_style_shots(style_id)));
        }
        Some(Value::Object(base))
    }

    pub fn get_style_preview_path(&self, style_id: &str) -> Option<PathBuf> {
        if !is_safe_segment(style_id) {
            return None;
        }
        self.ensure_indexed();
        let (cached, refero_dir) = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            (g.preview_cache.get(style_id).cloned(), g.refero_dir.clone())
        };
        if let Some(p) = cached {
            if p.is_file() {
                return Some(p);
            }
        }
        let style_dir = refero_dir.join("styles").join(style_id);
        let meta = self.get_style_meta(style_id);
        let raw_prev = meta.get("preview").and_then(Value::as_str);
        let (_, found) = inspect_style_dir(&style_dir, raw_prev);
        if let Some(p) = found {
            if p.is_file() {
                let mut g = self.state.write().unwrap_or_else(|p| p.into_inner());
                g.preview_cache.insert(style_id.to_string(), p.clone());
                if let Some(rec) = g.styles_by_id.get_mut(style_id) {
                    rec.insert("has_preview".into(), Value::Bool(true));
                    rec.insert(
                        "preview_url".into(),
                        Value::String(format!("/api/styles/{style_id}/preview")),
                    );
                }
                return Some(p);
            }
        }
        None
    }

    pub fn get_design_markdown(&self, style_id: &str) -> String {
        if !is_safe_segment(style_id) {
            return String::new();
        }
        let refero_dir = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            if let Some(cached) = g.design_cache.get(style_id) {
                return cached.clone();
            }
            g.refero_dir.clone()
        };
        let path = refero_dir.join("styles").join(style_id).join("DESIGN.md");
        let content = std::fs::read(&path)
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .unwrap_or_default();
        if !content.is_empty() {
            let mut g = self.state.write().unwrap_or_else(|p| p.into_inner());
            if g.design_cache.len() < 2000 {
                g.design_cache.insert(style_id.to_string(), content.clone());
            }
        }
        content
    }

    pub fn get_style_meta(&self, style_id: &str) -> Value {
        if !is_safe_segment(style_id) {
            return json!({});
        }
        let refero_dir = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            if let Some(cached) = g.meta_cache.get(style_id) {
                return cached.clone();
            }
            g.refero_dir.clone()
        };
        let path = refero_dir.join("styles").join(style_id).join("meta.json");
        let meta = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));
        if meta.as_object().is_some_and(|o| !o.is_empty()) {
            let mut g = self.state.write().unwrap_or_else(|p| p.into_inner());
            if g.meta_cache.len() < 2000 {
                g.meta_cache.insert(style_id.to_string(), meta.clone());
            }
        }
        meta
    }

    fn resolve_style_shots(&self, style_id: &str) -> Vec<(usize, PathBuf)> {
        if !is_safe_segment(style_id) {
            return Vec::new();
        }
        self.ensure_indexed();
        let refero_dir = self
            .state
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .refero_dir
            .clone();
        let style_dir = refero_dir.join("styles").join(style_id);
        let shots_dir = style_dir.join("shots");

        let mut raw_files: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&shots_dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let ext = ext_lower(&p);
                    if IMAGE_SUFFIXES.contains(&ext.as_str())
                        || VIDEO_SUFFIXES.contains(&ext.as_str())
                    {
                        raw_files.push(p);
                    }
                }
            }
        }

        if raw_files.is_empty() {
            let meta = self.get_style_meta(style_id);
            if let Some(arr) = meta.get("shots").and_then(Value::as_array) {
                for item in arr {
                    let rel = if let Some(s) = item.as_str() {
                        s.trim()
                    } else if let Some(obj) = item.as_object() {
                        val_str(obj, &["file", "path"])
                    } else {
                        ""
                    };
                    if rel.is_empty() {
                        continue;
                    }
                    if let Some(p) = resolve_confined_file(&style_dir, rel).or_else(|| {
                        Path::new(rel)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .and_then(|n| resolve_confined_file(&shots_dir, n))
                    }) {
                        raw_files.push(p);
                    }
                }
            }
        }

        raw_files.sort_by_key(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
        });

        let mut resolved = Vec::with_capacity(raw_files.len());
        let mut used_nums = HashSet::new();
        for (idx0, fpath) in raw_files.into_iter().enumerate() {
            let idx = idx0 + 1;
            let stem = fpath.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let mut n_val = first_number_in(stem).unwrap_or(idx);
            if n_val == 0 || used_nums.contains(&n_val) {
                n_val = idx;
            }
            while used_nums.contains(&n_val) {
                n_val += 1;
            }
            used_nums.insert(n_val);
            resolved.push((n_val, fpath));
        }
        resolved.sort_by_key(|(n, _)| *n);
        resolved
    }

    pub fn list_style_shots(&self, style_id: &str) -> Vec<Value> {
        self.resolve_style_shots(style_id)
            .into_iter()
            .map(|(n_val, fpath)| {
                let ext = ext_lower(&fpath);
                let filename = fpath
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let kind = if VIDEO_SUFFIXES.contains(&ext.as_str()) {
                    "video"
                } else {
                    "image"
                };
                json!({
                    "n": n_val,
                    "url": format!("/api/styles/{style_id}/shots/{n_val}"),
                    "filename": filename,
                    "ext": ext,
                    "kind": kind,
                })
            })
            .collect()
    }

    pub fn get_style_shot_path(&self, style_id: &str, n: &str) -> Option<PathBuf> {
        let n_clean = n.trim();
        if !is_safe_segment(style_id) || !is_safe_segment(n_clean) {
            return None;
        }
        let resolved = self.resolve_style_shots(style_id);
        let n_lower = n_clean.to_ascii_lowercase();
        let n_stem = Path::new(&n_lower)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&n_lower);
        let n_int = n_stem.parse::<usize>().ok();
        for (shot_num, fpath) in resolved {
            if n_int == Some(shot_num) {
                return Some(fpath);
            }
            let fname = fpath
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let fstem = fpath
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if fname == n_lower || fstem == n_stem {
                return Some(fpath);
            }
        }
        None
    }

    pub fn search(
        &self,
        query: &str,
        theme: Option<&str>,
        category: Option<&str>,
        include_content: bool,
        limit: usize,
    ) -> Value {
        self.ensure_indexed();
        let q_clean = query.trim();
        let eff_limit = limit.clamp(1, 200);
        if q_clean.is_empty() {
            let base = self.list_styles(None, theme, category, None, eff_limit, 0);
            return json!({
                "query": "",
                "total_styles": base.get("total").and_then(Value::as_u64).unwrap_or(0),
                "styles": base.get("items").cloned().unwrap_or_else(|| json!([])),
                "total_templates": 0,
                "templates": [],
                "total_components": 0,
                "components": [],
            });
        }

        let terms: Vec<String> = q_clean
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();

        let (mut candidates, collections_by_id, templates, components) = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            (
                g.styles.clone(),
                g.collections_by_id.clone(),
                g.templates.clone(),
                g.components.clone(),
            )
        };

        if let Some(t) = theme.map(|s| s.trim().to_ascii_lowercase()) {
            if t == "light" || t == "dark" {
                candidates.retain(|s| s.get("theme").and_then(Value::as_str) == Some(t.as_str()));
            }
        }

        if let Some(cat) = category
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty() && s != "all")
        {
            if let Some(col) = collections_by_id.get(&cat) {
                let allowed: HashSet<&str> = col
                    .get("style_ids")
                    .and_then(Value::as_array)
                    .map(|arr| arr.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default();
                candidates.retain(|s| {
                    s.get("id")
                        .and_then(Value::as_str)
                        .is_some_and(|id| allowed.contains(id))
                });
            } else {
                candidates.retain(|s| {
                    s.get("categories")
                        .and_then(Value::as_array)
                        .is_some_and(|arr| arr.iter().any(|c| c.as_str() == Some(cat.as_str())))
                });
            }
        }

        let mut scored_styles: Vec<(i64, String, Value)> = Vec::new();
        let mut content_reads = 0usize;
        let max_uncached_reads = 120usize;

        for s in candidates {
            let id_s = val_str(&s, &["id"]).to_string();
            let title_l = val_str(&s, &["title"]).to_lowercase();
            let tagline_l = val_str(&s, &["tagline"]).to_lowercase();
            let source_l = val_str(&s, &["source_url"]).to_lowercase();
            let cats_l = s
                .get("categories")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_lowercase()
                })
                .unwrap_or_default();
            let meta_blob = format!("{title_l} {tagline_l} {source_l} {} {cats_l}", id_s.to_lowercase());

            let mut score = 0i64;
            if terms.iter().all(|term| meta_blob.contains(term)) {
                score += 50;
                for term in &terms {
                    if title_l.contains(term) {
                        score += 40;
                    }
                    if tagline_l.contains(term) {
                        score += 20;
                    }
                    if source_l.contains(term) {
                        score += 10;
                    }
                }
            }

            let mut snippet = String::new();
            if include_content && (score == 0 || scored_styles.len() < eff_limit) {
                let cached_md = {
                    let g = self.state.read().unwrap_or_else(|p| p.into_inner());
                    g.design_cache.get(&id_s).cloned()
                };
                let md = match cached_md {
                    Some(m) => m,
                    None if content_reads < max_uncached_reads
                        && s.get("has_design").and_then(Value::as_bool).unwrap_or(false) =>
                    {
                        content_reads += 1;
                        self.get_design_markdown(&id_s)
                    }
                    None => String::new(),
                };
                if !md.is_empty() {
                    let md_lower = md.to_lowercase();
                    if terms.iter().all(|term| md_lower.contains(term)) {
                        score += 15;
                        if let Some(first_term) = terms.first() {
                            if let Some(pos) = md_lower.find(first_term.as_str()) {
                                let chars: Vec<char> = md.chars().collect();
                                // Approximate char index safely via char boundaries
                                let char_pos = md[..pos.min(md.len())].chars().count();
                                let start = char_pos.saturating_sub(60);
                                let end = (char_pos + 160).min(chars.len());
                                let raw_snip: String = chars[start..end].iter().collect();
                                snippet = collapse_ws(&raw_snip);
                            }
                        }
                    }
                }
            }

            if score > 0 {
                let mut entry = s;
                if !snippet.is_empty() {
                    entry.insert("match_snippet".into(), Value::String(snippet.clone()));
                    entry.insert("snippet".into(), Value::String(snippet));
                }
                entry.insert("score".into(), json!(score));
                scored_styles.push((score, title_l, Value::Object(entry)));
            }
        }

        scored_styles.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let total_styles = scored_styles.len();
        let matched_styles: Vec<Value> = scored_styles
            .into_iter()
            .take(eff_limit)
            .map(|(_, _, v)| v)
            .collect();

        let mut matched_templates = Vec::new();
        for tpl in &templates {
            let blob = format!(
                "{} {} {} {}",
                val_str(tpl, &["title"]),
                val_str(tpl, &["slug"]),
                val_str(tpl, &["prompt_preview"]),
                val_str(tpl, &["lang"])
            )
            .to_lowercase();
            if terms.iter().all(|t| blob.contains(t)) {
                matched_templates.push(Value::Object(public_dict(tpl)));
                if matched_templates.len() >= 20 {
                    break;
                }
            }
        }

        let mut matched_components = Vec::new();
        for comp in &components {
            let tags_s = comp
                .get("tags")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            let blob = format!(
                "{} {} {} {} {}",
                val_str(comp, &["name"]),
                val_str(comp, &["category"]),
                val_str(comp, &["description"]),
                tags_s,
                val_str(comp, &["id"])
            )
            .to_lowercase();
            if terms.iter().all(|t| blob.contains(t)) {
                matched_components.push(Value::Object(public_dict(comp)));
                if matched_components.len() >= 20 {
                    break;
                }
            }
        }

        json!({
            "query": q_clean,
            "total_styles": total_styles,
            "styles": matched_styles,
            "total_templates": matched_templates.len(),
            "templates": matched_templates,
            "total_components": matched_components.len(),
            "components": matched_components,
        })
    }

    pub fn list_collections(&self) -> Vec<Value> {
        self.ensure_indexed();
        let g = self.state.read().unwrap_or_else(|p| p.into_inner());
        g.collections
            .iter()
            .map(|col| {
                let mut m = col.clone();
                m.remove("style_ids");
                Value::Object(m)
            })
            .collect()
    }

    pub fn get_collection(
        &self,
        collection_id: &str,
        limit: usize,
        offset: usize,
    ) -> Option<Value> {
        if !is_safe_segment(collection_id) {
            return None;
        }
        self.ensure_indexed();
        let (mut col, items) = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            let col = g.collections_by_id.get(collection_id)?.clone();
            let items: Vec<Value> = col
                .get("style_ids")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .filter_map(|sid| g.styles_by_id.get(sid).cloned().map(Value::Object))
                        .collect()
                })
                .unwrap_or_default();
            (col, items)
        };
        let total = items.len();
        let eff_limit = limit.clamp(1, 500);
        let page_items: Vec<Value> = items.into_iter().skip(offset).take(eff_limit).collect();
        col.remove("style_ids");
        col.insert("total".into(), json!(total));
        col.insert("limit".into(), json!(eff_limit));
        col.insert("offset".into(), json!(offset));
        col.insert("items".into(), Value::Array(page_items.clone()));
        col.insert("styles".into(), Value::Array(page_items));
        Some(Value::Object(col))
    }

    pub fn list_templates(
        &self,
        lang: Option<&str>,
        q: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Value {
        self.ensure_indexed();
        let all_templates = self
            .state
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .templates
            .clone();

        let mut en_cnt = 0usize;
        let mut vi_cnt = 0usize;
        for t in &all_templates {
            match val_str(t, &["lang"]) {
                "vi" => vi_cnt += 1,
                _ => en_cnt += 1,
            }
        }

        let mut items: Vec<&Map<String, Value>> = all_templates.iter().collect();
        if let Some(l) = lang.map(|s| s.trim().to_ascii_lowercase()) {
            if l == "en" || l == "vi" {
                items.retain(|t| val_str(t, &["lang"]) == l);
            }
        }
        if let Some(q_raw) = q.map(str::trim).filter(|s| !s.is_empty()) {
            let terms: Vec<String> = q_raw
                .to_lowercase()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            items.retain(|t| {
                let blob = format!(
                    "{} {} {} {}",
                    val_str(t, &["title"]),
                    val_str(t, &["slug"]),
                    val_str(t, &["prompt_preview"]),
                    val_str(t, &["id"])
                )
                .to_lowercase();
                terms.iter().all(|term| blob.contains(term))
            });
        }

        let total = items.len();
        let eff_limit = limit.clamp(1, 200);
        let sliced: Vec<Value> = items
            .into_iter()
            .skip(offset)
            .take(eff_limit)
            .map(|t| Value::Object(public_dict(t)))
            .collect();

        json!({
            "total": total,
            "limit": eff_limit,
            "offset": offset,
            "languages": { "en": en_cnt, "vi": vi_cnt },
            "facets": {
                "languages": { "en": en_cnt, "vi": vi_cnt },
            },
            "items": sliced,
        })
    }

    fn find_raw_template(&self, template_id: &str) -> Option<Map<String, Value>> {
        if !is_safe_segment(template_id) {
            return None;
        }
        self.ensure_indexed();
        let g = self.state.read().unwrap_or_else(|p| p.into_inner());
        if let Some(t) = g.templates_by_id.get(template_id) {
            return Some(t.clone());
        }
        for cand in &g.templates {
            if val_str(cand, &["slug"]) == template_id
                || cand
                    .get("number")
                    .and_then(Value::as_u64)
                    .is_some_and(|n| n.to_string() == template_id)
            {
                return Some(cand.clone());
            }
        }
        None
    }

    pub fn get_template(&self, template_id: &str, include_html: bool) -> Option<Value> {
        let tpl = self.find_raw_template(template_id)?;
        let mut res = public_dict(&tpl);
        let prompt_path = val_str(&tpl, &["_prompt_path"]);
        let prompt_text = if !prompt_path.is_empty() {
            std::fs::read(prompt_path)
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        res.insert("prompt".into(), Value::String(prompt_text));

        if include_html {
            let html_path = val_str(&tpl, &["_html_path"]);
            let html_text = if !html_path.is_empty() {
                std::fs::read(html_path)
                    .map(|b| String::from_utf8_lossy(&b).to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            res.insert("html".into(), Value::String(html_text));
        }
        Some(Value::Object(res))
    }

    pub fn get_template_html_content(&self, template_id: &str) -> Option<String> {
        let tpl = self.find_raw_template(template_id)?;
        let html_path = val_str(&tpl, &["_html_path"]);
        if html_path.is_empty() {
            return None;
        }
        std::fs::read(html_path)
            .ok()
            .map(|b| String::from_utf8_lossy(&b).to_string())
    }

    pub fn get_template_thumb_or_meta(&self, template_id: &str) -> Option<(Option<PathBuf>, Map<String, Value>)> {
        let tpl = self.find_raw_template(template_id)?;
        let thumb_s = val_str(&tpl, &["_thumb_path"]);
        let thumb_path = if !thumb_s.is_empty() {
            let p = PathBuf::from(thumb_s);
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        } else {
            None
        };
        Some((thumb_path, tpl))
    }

    pub fn get_template_asset_path(&self, template_id: &str, filename: &str) -> Option<PathBuf> {
        let tpl = self.find_raw_template(template_id)?;
        let html_s = val_str(&tpl, &["_html_path"]);
        if html_s.is_empty() {
            return None;
        }
        let base_dir = Path::new(html_s).parent()?;
        resolve_confined_file(base_dir, filename)
    }

    pub fn list_components(
        &self,
        category: Option<&str>,
        kind: Option<&str>,
        q: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Value {
        self.ensure_indexed();
        let all_comps = self
            .state
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .components
            .clone();

        let mut cat_counts: HashMap<String, usize> = HashMap::new();
        for c in &all_comps {
            let cat = val_str(c, &["category"]);
            let key = if cat.is_empty() { "misc" } else { cat };
            *cat_counts.entry(key.to_string()).or_insert(0) += 1;
        }

        let mut items: Vec<&Map<String, Value>> = all_comps.iter().collect();
        if let Some(cat) = category
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty() && s != "all")
        {
            items.retain(|c| val_str(c, &["category"]) == cat);
        }
        if let Some(k) = kind.map(|s| s.trim().to_ascii_lowercase()) {
            if k == "html" || k == "tsx" {
                items.retain(|c| val_str(c, &["kind"]) == k);
            }
        }
        if let Some(q_raw) = q.map(str::trim).filter(|s| !s.is_empty()) {
            let terms: Vec<String> = q_raw
                .to_lowercase()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            items.retain(|c| {
                let tags_s = c
                    .get("tags")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                let blob = format!(
                    "{} {} {} {} {}",
                    val_str(c, &["name"]),
                    val_str(c, &["category"]),
                    val_str(c, &["description"]),
                    tags_s,
                    val_str(c, &["id"])
                )
                .to_lowercase();
                terms.iter().all(|term| blob.contains(term))
            });
        }

        let total = items.len();
        let eff_limit = limit.clamp(1, 500);
        let sliced: Vec<Value> = items
            .into_iter()
            .skip(offset)
            .take(eff_limit)
            .map(|c| Value::Object(public_dict(c)))
            .collect();

        let mut cat_facets: Vec<(String, usize)> = cat_counts.into_iter().collect();
        cat_facets.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let categories_json: Vec<Value> = cat_facets
            .into_iter()
            .map(|(id, count)| json!({ "id": id, "count": count }))
            .collect();

        json!({
            "total": total,
            "limit": eff_limit,
            "offset": offset,
            "facets": {
                "categories": categories_json,
            },
            "items": sliced,
        })
    }

    pub fn get_component(&self, component_id: &str, include_source: bool) -> Option<Value> {
        if !is_safe_segment(component_id) {
            return None;
        }
        self.ensure_indexed();
        let comp = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            g.components_by_id.get(component_id)?.clone()
        };
        let mut result = public_dict(&comp);
        let item_dir = PathBuf::from(val_str(&comp, &["_dir"]));

        let meta_path = item_dir.join("meta.json");
        if meta_path.is_file() {
            let meta_val = std::fs::read_to_string(&meta_path)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}));
            result.insert("meta".into(), meta_val);
        }

        if include_source {
            let mut sources = Map::new();
            if let Some(files) = comp.get("files").and_then(Value::as_array) {
                for rel_v in files {
                    let Some(rel) = rel_v.as_str() else { continue };
                    if let Some(f_path) = resolve_confined_file(&item_dir, rel) {
                        if let Ok(md) = std::fs::metadata(&f_path) {
                            if md.len() <= 512_000 {
                                if let Ok(bytes) = std::fs::read(&f_path) {
                                    sources.insert(
                                        rel.to_string(),
                                        Value::String(String::from_utf8_lossy(&bytes).to_string()),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            result.insert("sources".into(), Value::Object(sources));
        }

        Some(Value::Object(result))
    }

    pub fn get_component_preview_path(&self, component_id: &str) -> Option<PathBuf> {
        if !is_safe_segment(component_id) {
            return None;
        }
        self.ensure_indexed();
        let item_dir = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            let comp = g.components_by_id.get(component_id)?;
            PathBuf::from(val_str(comp, &["_dir"]))
        };
        let rd = std::fs::read_dir(&item_dir).ok()?;
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file()
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case("preview"))
            {
                let ext = ext_lower(&p);
                if PREVIEW_COMP_SUFFIXES.contains(&ext.as_str()) {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn get_component_file_path(&self, component_id: &str, filename: &str) -> Option<PathBuf> {
        if !is_safe_segment(component_id) {
            return None;
        }
        self.ensure_indexed();
        let (item_dir, entry_file) = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            let comp = g.components_by_id.get(component_id)?;
            let dir = PathBuf::from(val_str(comp, &["_dir"]));
            let entry = {
                let e = val_str(comp, &["_entry"]);
                if e.is_empty() {
                    "index.html".to_string()
                } else {
                    e.to_string()
                }
            };
            (dir, entry)
        };
        let target_rel = if filename.is_empty() || filename == "html" {
            entry_file.as_str()
        } else {
            filename
        };
        resolve_confined_file(&item_dir, target_rel)
    }

    pub fn get_licenses(&self) -> Value {
        self.ensure_indexed();
        let (styles, templates, components) = {
            let g = self.state.read().unwrap_or_else(|p| p.into_inner());
            (g.styles.clone(), g.templates.clone(), g.components.clone())
        };

        let attributed_sites: Vec<Value> = styles
            .iter()
            .map(|s| {
                let cat = val_str(s, &["category_label"]);
                json!({
                    "id": val_str(s, &["id"]),
                    "title": val_str(s, &["title"]),
                    "theme": val_str(s, &["theme"]),
                    "category": if cat.is_empty() { "Design System" } else { cat },
                    "refero_url": val_str(s, &["url"]),
                    "source_url": val_str(s, &["source_url"]),
                })
            })
            .collect();

        let with_design_md = styles
            .iter()
            .filter(|s| s.get("has_design").and_then(Value::as_bool).unwrap_or(false))
            .count();
        let html_comps = components
            .iter()
            .filter(|c| val_str(c, &["kind"]) == "html")
            .count();
        let tsx_comps = components
            .iter()
            .filter(|c| val_str(c, &["kind"]) == "tsx")
            .count();

        json!({
            "service": "lnc-styles-gallery",
            "version": self.settings.version,
            "corpora": [
                {
                    "id": "refero-styles",
                    "name": "Refero Design Styles",
                    "upstream_url": "https://styles.refero.design",
                    "source": "https://styles.refero.design",
                    "license": "Curated Reference / Fair Use",
                    "total_styles": styles.len(),
                    "count": styles.len(),
                    "with_design_md": with_design_md,
                    "license_note": "Distilled design system token & layout specifications (DESIGN.md) curated from real-world product websites via styles.refero.design. All trademarks, brand identities, and original website designs belong to their respective product owners listed in source_url.",
                    "attribution_note": "Distilled design system token & layout specifications (DESIGN.md) curated from real-world product websites via styles.refero.design.",
                    "terms_summary": [
                        "Each style preserves its original Refero URL and attributed product website (source_url).",
                        "DESIGN.md guides provide color tokens, typography, layout structure, and component guidelines.",
                        "All trademarks and brand assets remain the property of their respective product owners."
                    ]
                },
                {
                    "id": "prompt-driven-ui",
                    "name": "Prompt-Driven UI HTML Templates",
                    "upstream_url": "https://github.com/vuhung16au/prompt-driven-ui",
                    "source": "https://github.com/vuhung16au/prompt-driven-ui",
                    "author": "vuhung16au",
                    "license": "MIT",
                    "total_templates": templates.len(),
                    "count": templates.len(),
                    "license_note": "100 single-file HTML/CSS/JS sample designs (50 English, 50 Vietnamese) with generation prompts, distributed under the MIT License.",
                    "attribution_note": "100 single-file HTML/CSS/JS sample designs (50 English + 50 Vietnamese) by vuhung16au.",
                    "terms_summary": [
                        "Licensed under the open-source MIT License (vuhung16au/prompt-driven-ui).",
                        "Includes 50 English (websites-en/) and 50 Vietnamese (websites/) self-contained HTML templates.",
                        "Includes corresponding generation prompts and preview thumbnails."
                    ]
                },
                {
                    "id": "lnc-components",
                    "name": "LNC UI Effects & Components",
                    "upstream_url": "https://styles.lengoc.me",
                    "source": "lnc-components/items/",
                    "author": "LNC",
                    "license": "LNC Internal (original work)",
                    "total_components": html_comps,
                    "count": html_comps,
                    "license_note": "Self-contained HTML/CSS/JS effect and component demos authored in-house: buttons, cards, loaders, toggles, animated backgrounds, text effects, and AI-agent components. Original work — free to reuse inside any LNC project.",
                    "attribution_note": "Original LNC-authored UI effect recipes.",
                    "terms_summary": [
                        "Single-file HTML demos — open in an iframe or copy the inline CSS/JS.",
                        "Standard CSS techniques (no third-party code vendored)."
                    ]
                },
                {
                    "id": "knowns-ui",
                    "name": "Knowns UI — AI Product Component Kit",
                    "upstream_url": "https://ui.knowns.dev",
                    "source": "https://github.com/knowns-dev/knowns-ui",
                    "author": "Knowns Lab",
                    "license": "MIT",
                    "total_components": tsx_comps,
                    "count": tsx_comps,
                    "license_note": "React/TSX components and blocks for AI products (tool-call cards, approval prompts, agent plans, voice orb, animated backgrounds), harvested from the public shadcn registry at ui.knowns.dev under the MIT License (copyright 2026 Knowns Lab). Third-party logo icons in lib/knowns/icons are trademark-restricted and were skipped.",
                    "attribution_note": "Knowns UI component sources by Knowns Lab — ui.knowns.dev — installed via `npx shadcn@latest add https://ui.knowns.dev/r/<name>.json`.",
                    "terms_summary": [
                        "MIT License — free to use, copy, modify, and distribute with attribution.",
                        "Update path: re-run temp_scripts/harvest_knowns.py to pull registry updates.",
                        "Some entries marked Pro upstream are not fetched; only the free registry set is mirrored."
                    ]
                }
            ],
            "attributed_sources_count": attributed_sites.len(),
            "attributed_sources_total": attributed_sites.len(),
            "attributed_sources": attributed_sites,
            "attributed_sources_sample": attributed_sites,
        })
    }
}

// ---------------------------------------------------------------------------
// Dynamic SVG Preview Fallbacks (parity with `app/main.py`)
// ---------------------------------------------------------------------------

pub fn render_style_fallback_svg(style: &Value, style_id: &str) -> String {
    let is_dark = style.get("theme").and_then(Value::as_str) == Some("dark");
    let (bg_start, bg_end, card_bg, card_border, fg_main, fg_sub, accent, theme_badge) = if is_dark
    {
        (
            "#090d16", "#1e293b", "#0f172a", "#334155", "#f8fafc", "#94a3b8", "#6366f1", "DARK",
        )
    } else {
        (
            "#f8fafc", "#e2e8f0", "#ffffff", "#cbd5e1", "#0f172a", "#475569", "#0284c7", "LIGHT",
        )
    };

    let title_raw = style
        .get("title")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(style_id);
    let title_chars: Vec<char> = title_raw.chars().collect();
    let title_trunc = if title_chars.len() > 38 {
        format!("{}…", title_chars[..38].iter().collect::<String>())
    } else {
        title_raw.to_string()
    };
    let title = xml_escape(&title_trunc);

    let tagline_raw = style
        .get("tagline")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("Design System & UI Tokens");
    let tag_chars: Vec<char> = tagline_raw.chars().collect();
    let tag_trunc = if tag_chars.len() > 64 {
        format!("{}…", tag_chars[..64].iter().collect::<String>())
    } else {
        tagline_raw.to_string()
    };
    let tagline = xml_escape(&tag_trunc);

    let source_url = style
        .get("source_url")
        .and_then(Value::as_str)
        .unwrap_or("");
    let domain = Url::parse(source_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "styles.refero.design".to_string());
    let domain_esc = xml_escape(&domain);

    let cat_label = xml_escape(
        &style
            .get("categories")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(Value::as_str)
            .unwrap_or("design-system")
            .to_uppercase(),
    );

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 400" width="640" height="400">
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
</svg>"##
    )
}

pub fn render_template_fallback_svg(tpl: &Map<String, Value>, template_id: &str) -> String {
    let title_raw = val_str(tpl, &["title"]);
    let t_src = if title_raw.is_empty() {
        template_id
    } else {
        title_raw
    };
    let title_trunc: String = t_src.chars().take(42).collect();
    let title = xml_escape(&title_trunc);
    let lang = val_str(tpl, &["lang"]);
    let lang_eff = if lang.is_empty() { "en" } else { lang };
    let lang_badge = xml_escape(&lang_eff.to_uppercase());
    let num = tpl.get("number").and_then(Value::as_u64).unwrap_or(1);
    let num_str = format!("#{num:02}");
    let accent = if lang_eff == "en" {
        "#6366f1"
    } else {
        "#10b981"
    };
    let slug_esc = xml_escape(val_str(tpl, &["slug"]));

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 360" width="640" height="360">
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
  <text x="52" y="205" fill="#94a3b8" font-family="monospace" font-size="14">{slug_esc}</text>
  <rect x="52" y="250" width="140" height="34" rx="8" fill="{accent}"/>
  <text x="122" y="272" fill="#ffffff" font-family="system-ui, sans-serif" font-size="13" font-weight="600" text-anchor="middle">HTML Template</text>
</svg>"##
    )
}

// ---------------------------------------------------------------------------
// Corpus Scanner Functions
// ---------------------------------------------------------------------------

type RecordMap = Map<String, Value>;
type IndexedRecords = (Vec<RecordMap>, HashMap<String, RecordMap>);
type IndexedStyles = (
    Vec<RecordMap>,
    HashMap<String, RecordMap>,
    HashMap<String, PathBuf>,
);

fn public_dict(m: &Map<String, Value>) -> Map<String, Value> {
    m.iter()
        .filter(|(k, _)| !k.starts_with('_'))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn inspect_style_dir(style_dir: &Path, raw_preview: Option<&str>) -> (bool, Option<PathBuf>) {
    if !style_dir.is_dir() {
        return (false, None);
    }
    let raw_prev_name = raw_preview
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| Path::new(s).file_name())
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    let mut has_design = false;
    let mut preview_path: Option<PathBuf> = None;

    if let Ok(rd) = std::fs::read_dir(style_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let Some(fname) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            let name_l = fname.to_ascii_lowercase();
            if name_l == "design.md" {
                has_design = true;
            } else {
                let ext = ext_lower(&p);
                if IMAGE_SUFFIXES.contains(&ext.as_str())
                    && ((!raw_prev_name.is_empty() && name_l == raw_prev_name)
                        || (name_l.starts_with("preview.") && preview_path.is_none()))
                {
                    preview_path = Some(p);
                }
            }
        }
    }
    (has_design, preview_path)
}

fn normalize_style_record(
    style_id: &str,
    raw: &Map<String, Value>,
    existing_style_dirs: &HashSet<String>,
    styles_subdir: &Path,
) -> (Map<String, Value>, Option<PathBuf>) {
    let title = {
        let t = val_str(raw, &["title", "name"]);
        if t.is_empty() {
            style_id.to_string()
        } else {
            t.to_string()
        }
    };
    let tagline = val_str(raw, &["tagline", "description", "summary"]).to_string();
    let mut raw_theme = val_str(raw, &["theme", "color_scheme"]).to_ascii_lowercase();
    if raw_theme != "light" && raw_theme != "dark" {
        let lower_blob = format!("{title} {tagline}").to_lowercase();
        if ["dark", "night", "obsidian", "noir", "black", "midnight"]
            .iter()
            .any(|w| lower_blob.contains(w))
        {
            raw_theme = "dark".to_string();
        } else {
            raw_theme = "light".to_string();
        }
    }

    let url = {
        let u = val_str(raw, &["url"]);
        if u.is_empty() {
            format!("https://styles.refero.design/styles/{style_id}")
        } else {
            u.to_string()
        }
    };
    let source_url = val_str(raw, &["source_url", "sourceUrl", "website", "site_url"]).to_string();

    let categories: Vec<String> = raw
        .get("categories")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| classify_style(&title, &tagline, &raw_theme, &source_url));

    let has_folder = if existing_style_dirs.is_empty() {
        true
    } else {
        existing_style_dirs.contains(style_id)
    };

    let preview_path = if has_folder {
        inspect_style_dir(&styles_subdir.join(style_id), raw.get("preview").and_then(Value::as_str)).1
    } else {
        None
    };

    let has_preview = preview_path.is_some();
    let preview_url = if has_preview {
        Value::String(format!("/api/styles/{style_id}/preview"))
    } else {
        Value::Null
    };
    let primary_cat = categories
        .first()
        .cloned()
        .unwrap_or_else(|| "warm-minimal".to_string());
    let category_label = humanize_slug(&primary_cat);

    let mut m = Map::new();
    m.insert("id".into(), Value::String(style_id.to_string()));
    m.insert("title".into(), Value::String(title));
    m.insert("tagline".into(), Value::String(tagline));
    m.insert("theme".into(), Value::String(raw_theme));
    m.insert("url".into(), Value::String(url));
    m.insert("source_url".into(), Value::String(source_url));
    m.insert(
        "categories".into(),
        Value::Array(categories.into_iter().map(Value::String).collect()),
    );
    m.insert("category".into(), Value::String(primary_cat));
    m.insert("category_label".into(), Value::String(category_label));
    m.insert("has_design".into(), Value::Bool(has_folder));
    m.insert("has_meta".into(), Value::Bool(has_folder));
    m.insert("has_preview".into(), Value::Bool(has_preview));
    m.insert("preview_url".into(), preview_url);

    (m, preview_path)
}

fn scan_refero_styles(refero_dir: &Path, warnings: &mut Vec<String>) -> IndexedStyles {
    let mut styles_list = Vec::new();
    let mut styles_map = HashMap::new();
    let mut preview_map = HashMap::new();

    let index_file = refero_dir.join("index.json");
    let styles_subdir = refero_dir.join("styles");

    let mut existing_style_dirs = HashSet::new();
    if styles_subdir.is_dir() {
        match std::fs::read_dir(&styles_subdir) {
            Ok(rd) => {
                for entry in rd.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            if is_safe_segment(name) {
                                existing_style_dirs.insert(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(e) => warnings.push(format!(
                "Could not scan styles directory {}: {e}",
                styles_subdir.display()
            )),
        }
    }

    let mut raw_records: Vec<Map<String, Value>> = Vec::new();
    if index_file.is_file() {
        match std::fs::read(&index_file)
            .ok()
            .and_then(|b| serde_json::from_str::<Value>(&String::from_utf8_lossy(&b)).ok())
        {
            Some(Value::Array(arr)) => {
                for item in arr {
                    if let Value::Object(o) = item {
                        raw_records.push(o);
                    }
                }
            }
            Some(Value::Object(obj)) => {
                for key in ["styles", "items", "data", "results"] {
                    if let Some(arr) = obj.get(key).and_then(Value::as_array) {
                        for item in arr {
                            if let Value::Object(o) = item.clone() {
                                raw_records.push(o);
                            }
                        }
                        break;
                    }
                }
            }
            _ => warnings.push(format!("Failed to parse {}", index_file.display())),
        }
    } else {
        warnings.push(format!("Refero index.json not found at {}", index_file.display()));
    }

    let mut seen_ids = HashSet::new();
    for item in &raw_records {
        let sid = val_str(item, &["id", "uuid", "slug"]);
        if !is_safe_segment(sid) || seen_ids.contains(sid) {
            continue;
        }
        seen_ids.insert(sid.to_string());
        let (rec, prev_path) =
            normalize_style_record(sid, item, &existing_style_dirs, &styles_subdir);
        if let Some(p) = prev_path {
            preview_map.insert(sid.to_string(), p);
        }
        styles_map.insert(sid.to_string(), rec.clone());
        styles_list.push(rec);
    }

    let mut extra_dirs: Vec<String> = existing_style_dirs
        .difference(&seen_ids)
        .cloned()
        .collect();
    extra_dirs.sort();

    for sid in extra_dirs {
        let meta_path = styles_subdir.join(&sid).join("meta.json");
        let meta_obj = std::fs::read_to_string(&meta_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        let (rec, prev_path) =
            normalize_style_record(&sid, &meta_obj, &existing_style_dirs, &styles_subdir);
        if let Some(p) = prev_path {
            preview_map.insert(sid.clone(), p);
        }
        styles_map.insert(sid, rec.clone());
        styles_list.push(rec);
    }

    (styles_list, styles_map, preview_map)
}

fn pick_samples(members: &[&Map<String, Value>]) -> Vec<Value> {
    let mut prioritized: Vec<&Map<String, Value>> = members.to_vec();
    prioritized.sort_by_key(|m| !m.get("has_preview").and_then(Value::as_bool).unwrap_or(false));
    prioritized
        .into_iter()
        .take(6)
        .map(|m| {
            json!({
                "id": val_str(m, &["id"]),
                "title": val_str(m, &["title"]),
                "tagline": val_str(m, &["tagline"]),
                "theme": val_str(m, &["theme"]),
                "source_url": val_str(m, &["source_url"]),
                "has_preview": m.get("has_preview").and_then(Value::as_bool).unwrap_or(false),
                "preview_url": m.get("preview_url").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}

fn make_collection_obj(
    id: &str,
    name: &str,
    description: &str,
    kind: &str,
    accent: &str,
    members: &[&Map<String, Value>],
) -> Map<String, Value> {
    let style_ids: Vec<Value> = members
        .iter()
        .map(|m| Value::String(val_str(m, &["id"]).to_string()))
        .collect();
    let mut col = Map::new();
    col.insert("id".into(), Value::String(id.to_string()));
    col.insert("name".into(), Value::String(name.to_string()));
    col.insert("description".into(), Value::String(description.to_string()));
    col.insert("kind".into(), Value::String(kind.to_string()));
    col.insert("type".into(), Value::String(kind.to_string()));
    col.insert("accent".into(), Value::String(accent.to_string()));
    col.insert("count".into(), json!(members.len()));
    col.insert("sample_styles".into(), Value::Array(pick_samples(members)));
    col.insert("style_ids".into(), Value::Array(style_ids));
    col
}

fn build_collections(
    refero_dir: &Path,
    styles_list: &[Map<String, Value>],
    warnings: &mut Vec<String>,
) -> IndexedRecords {
    let mut collections_list = Vec::new();
    let mut collections_map = HashMap::new();

    let mut by_cat: HashMap<&str, Vec<&Map<String, Value>>> = COLLECTION_DEFINITIONS
        .iter()
        .map(|c| (c.id, Vec::new()))
        .collect();
    let mut dark_styles = Vec::new();
    let mut light_styles = Vec::new();

    for s in styles_list {
        if val_str(s, &["theme"]) == "dark" {
            dark_styles.push(s);
        } else {
            light_styles.push(s);
        }
        if let Some(cats) = s.get("categories").and_then(Value::as_array) {
            for c in cats.iter().filter_map(Value::as_str) {
                if let Some(vec) = by_cat.get_mut(c) {
                    vec.push(s);
                }
            }
        }
    }

    for def in COLLECTION_DEFINITIONS {
        let members = by_cat.get(def.id).map(Vec::as_slice).unwrap_or(&[]);
        let col = make_collection_obj(def.id, def.name, def.description, def.kind, def.accent, members);
        collections_map.insert(def.id.to_string(), col.clone());
        collections_list.push(col);
    }

    let dark_col = make_collection_obj(
        "theme-dark",
        "Dark Mode Systems",
        "All dark-themed design systems featuring deep surfaces, luminous accents, and low-glare contrast tokens.",
        "theme",
        "#38bdf8",
        &dark_styles,
    );
    collections_map.insert("theme-dark".into(), dark_col.clone());
    collections_list.push(dark_col);

    let light_col = make_collection_obj(
        "theme-light",
        "Light Mode Systems",
        "All light-themed design systems featuring daylight surfaces, crisp ink hierarchy, and subtle shadows.",
        "theme",
        "#fbbf24",
        &light_styles,
    );
    collections_map.insert("theme-light".into(), light_col.clone());
    collections_list.push(light_col);

    let sitemap_file = refero_dir.join("sitemaps").join("collections.xml");
    if sitemap_file.is_file() {
        match std::fs::read_to_string(&sitemap_file) {
            Ok(xml_text) => {
                for loc in extract_xml_locs(&xml_text) {
                    let slug = loc.trim_end_matches('/').rsplit('/').next().unwrap_or("");
                    if is_safe_segment(slug)
                        && !collections_map.contains_key(slug)
                        && slug != "collections"
                    {
                        let col_id = format!("sitemap-{slug}");
                        if collections_map.contains_key(&col_id) {
                            continue;
                        }
                        let needle = slug.replace('-', " ").to_lowercase();
                        let matched: Vec<&Map<String, Value>> = styles_list
                            .iter()
                            .filter(|s| {
                                format!(
                                    "{} {}",
                                    val_str(s, &["title"]),
                                    val_str(s, &["tagline"])
                                )
                                .to_lowercase()
                                .contains(&needle)
                            })
                            .collect();
                        let col = make_collection_obj(
                            &col_id,
                            &humanize_slug(slug),
                            &format!("Mirrored Refero sitemap collection ({loc})."),
                            "sitemap",
                            "#a855f7",
                            &matched,
                        );
                        collections_map.insert(col_id, col.clone());
                        collections_list.push(col);
                    }
                }
            }
            Err(e) => warnings.push(format!("Optional sitemaps/collections.xml parse skipped: {e}")),
        }
    }

    (collections_list, collections_map)
}

fn collect_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_files_recursive(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

fn index_auxiliary_files(directory: &Path, suffixes: &[&str]) -> HashMap<String, PathBuf> {
    let mut result = HashMap::new();
    if !directory.is_dir() {
        return result;
    }
    let mut files = Vec::new();
    collect_files_recursive(directory, &mut files);
    files.sort_by_key(|p| {
        let in_landscape = p
            .components()
            .any(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case("landscape"));
        let fname = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        (in_landscape, fname)
    });

    for entry in files {
        let ext = ext_lower(&entry);
        if suffixes.contains(&ext.as_str()) {
            let stem_lower = entry
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if stem_lower.is_empty() {
                continue;
            }
            if let Some(num_int) = leading_digits(&stem_lower) {
                result
                    .entry(format!("__num_{num_int}"))
                    .or_insert_with(|| entry.clone());
            }
            result.insert(stem_lower, entry);
        }
    }
    result
}

fn match_aux_file(
    stem: &str,
    number: usize,
    lang: &str,
    index: &HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    if index.is_empty() {
        return None;
    }
    let s_low = stem.to_ascii_lowercase();
    let candidates = [
        s_low.clone(),
        format!("{lang}-{s_low}"),
        format!("{s_low}-{lang}"),
        format!("{number:02}"),
        format!("{number:03}"),
        format!("{number}"),
        format!("__num_{number}"),
    ];
    for c in candidates {
        if let Some(p) = index.get(&c) {
            return Some(p.clone());
        }
    }
    None
}

fn scan_prompt_ui_templates(prompt_ui_dir: &Path, warnings: &mut Vec<String>) -> IndexedRecords {
    let mut templates_list = Vec::new();
    let mut templates_map = HashMap::new();

    if !prompt_ui_dir.exists() {
        warnings.push(format!(
            "Prompt-driven-ui directory not found at {}",
            prompt_ui_dir.display()
        ));
        return (templates_list, templates_map);
    }

    let thumbs_index = index_auxiliary_files(
        &prompt_ui_dir.join("thumbs"),
        &[".png", ".jpg", ".jpeg", ".webp", ".svg"],
    );
    let prompts_vi_index =
        index_auxiliary_files(&prompt_ui_dir.join("prompts"), &[".md", ".txt"]);
    let prompts_en_index =
        index_auxiliary_files(&prompt_ui_dir.join("prompts-en"), &[".md", ".txt"]);

    let sources = [
        (
            "en",
            prompt_ui_dir.join("websites-en"),
            &prompts_en_index,
            &prompts_vi_index,
        ),
        (
            "vi",
            prompt_ui_dir.join("websites"),
            &prompts_vi_index,
            &prompts_en_index,
        ),
    ];

    for (lang, web_dir, primary_prompts, fallback_prompts) in sources {
        if !web_dir.is_dir() {
            continue;
        }
        let mut entries: Vec<PathBuf> = match std::fs::read_dir(&web_dir) {
            Ok(rd) => rd.flatten().map(|e| e.path()).collect(),
            Err(e) => {
                warnings.push(format!("Error scanning {}: {e}", web_dir.display()));
                continue;
            }
        };
        entries.sort_by_key(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
        });

        let mut html_files = Vec::new();
        for entry in entries {
            if entry.is_file() {
                let ext = ext_lower(&entry);
                if ext == ".html" || ext == ".htm" {
                    html_files.push(entry);
                }
            } else if entry.is_dir() {
                let idx = entry.join("index.html");
                if idx.is_file() {
                    html_files.push(idx);
                } else if let Ok(sub_rd) = std::fs::read_dir(&entry) {
                    let mut sub_htmls: Vec<PathBuf> = sub_rd
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && ext_lower(p) == ".html")
                        .collect();
                    sub_htmls.sort();
                    if let Some(first) = sub_htmls.into_iter().next() {
                        html_files.push(first);
                    }
                }
            }
        }

        for (idx0, html_path) in html_files.into_iter().enumerate() {
            let idx_num = idx0 + 1;
            let fname = html_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("index.html")
                .to_string();
            let stem = if fname.eq_ignore_ascii_case("index.html") {
                html_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("template")
                    .to_string()
            } else {
                html_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("template")
                    .to_string()
            };
            if !is_safe_segment(&stem) {
                continue;
            }
            let number = leading_digits(&stem).unwrap_or(idx_num);
            let template_id = format!("{lang}-{stem}");

            let (extracted_title, extracted_desc) = extract_html_title_and_desc(&html_path);
            let title = if extracted_title.is_empty() {
                humanize_slug(&stem)
            } else {
                extracted_title
            };

            let thumb_path = match_aux_file(&stem, number, lang, &thumbs_index);
            let prompt_path = match_aux_file(&stem, number, lang, primary_prompts)
                .or_else(|| match_aux_file(&stem, number, lang, fallback_prompts));

            let mut prompt_preview = extracted_desc.clone();
            if let Some(ref pp) = prompt_path {
                if let Ok(bytes) = std::fs::read(pp) {
                    let collapsed = collapse_ws(&String::from_utf8_lossy(&bytes));
                    if !collapsed.is_empty() {
                        prompt_preview = collapsed.chars().take(240).collect();
                    }
                }
            }

            let has_prompt = prompt_path.as_ref().is_some_and(|p| p.is_file());
            let has_thumb = thumb_path.as_ref().is_some_and(|p| p.is_file());

            let mut m = Map::new();
            m.insert("id".into(), Value::String(template_id.clone()));
            m.insert("slug".into(), Value::String(stem));
            m.insert("number".into(), json!(number));
            m.insert("num".into(), json!(number));
            m.insert("lang".into(), Value::String(lang.to_string()));
            m.insert("title".into(), Value::String(title));
            m.insert("filename".into(), Value::String(fname));
            m.insert("description".into(), Value::String(extracted_desc));
            m.insert(
                "prompt_preview".into(),
                Value::String(prompt_preview.clone()),
            );
            m.insert("prompt_excerpt".into(), Value::String(prompt_preview));
            m.insert("has_prompt".into(), Value::Bool(has_prompt));
            m.insert("has_thumb".into(), Value::Bool(has_thumb));
            m.insert("has_thumb_file".into(), Value::Bool(has_thumb));
            m.insert(
                "html_url".into(),
                Value::String(format!("/api/templates/{template_id}/html")),
            );
            m.insert(
                "thumb_url".into(),
                Value::String(format!("/api/templates/{template_id}/thumb")),
            );
            m.insert(
                "_html_path".into(),
                Value::String(html_path.to_string_lossy().to_string()),
            );
            m.insert(
                "_thumb_path".into(),
                Value::String(
                    thumb_path
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                ),
            );
            m.insert(
                "_prompt_path".into(),
                Value::String(
                    prompt_path
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                ),
            );

            templates_map.insert(template_id, m.clone());
            templates_list.push(m);
        }
    }

    templates_list.sort_by(|a, b| {
        let la = if val_str(a, &["lang"]) == "en" { 0 } else { 1 };
        let lb = if val_str(b, &["lang"]) == "en" { 0 } else { 1 };
        let na = a.get("number").and_then(Value::as_u64).unwrap_or(0);
        let nb = b.get("number").and_then(Value::as_u64).unwrap_or(0);
        la.cmp(&lb)
            .then_with(|| na.cmp(&nb))
            .then_with(|| val_str(a, &["slug"]).cmp(val_str(b, &["slug"])))
    });

    (templates_list, templates_map)
}

fn scan_components(components_dir: &Path, warnings: &mut Vec<String>) -> IndexedRecords {
    let mut components_list = Vec::new();
    let mut components_map = HashMap::new();

    if !components_dir.exists() {
        warnings.push(format!(
            "Components directory not found at {}",
            components_dir.display()
        ));
        return (components_list, components_map);
    }

    let items_dir = components_dir.join("items");
    let index_file = components_dir.join("index.json");
    let mut raw_records: Vec<Map<String, Value>> = Vec::new();

    if index_file.is_file() {
        match std::fs::read(&index_file)
            .ok()
            .and_then(|b| serde_json::from_str::<Value>(&String::from_utf8_lossy(&b)).ok())
        {
            Some(Value::Array(arr)) => {
                for item in arr {
                    if let Value::Object(o) = item {
                        raw_records.push(o);
                    }
                }
            }
            _ => warnings.push(format!("Failed to parse {}", index_file.display())),
        }
    } else {
        warnings.push(format!(
            "Components index.json not found at {}",
            index_file.display()
        ));
    }

    let mut seen_ids = HashSet::new();
    for item in raw_records {
        let comp_id = val_str(&item, &["id"]);
        if !is_safe_segment(comp_id) || seen_ids.contains(comp_id) {
            continue;
        }
        seen_ids.insert(comp_id.to_string());
        let item_dir = items_dir.join(comp_id);
        if !item_dir.is_dir() {
            continue;
        }

        let kind = {
            let k = val_str(&item, &["kind"]).to_ascii_lowercase();
            if k.is_empty() {
                "html".to_string()
            } else {
                k
            }
        };
        let entry = {
            let e = val_str(&item, &["entry"]);
            if e.is_empty() {
                "index.html".to_string()
            } else {
                e.to_string()
            }
        };
        let has_html = kind == "html" && resolve_confined_file(&item_dir, &entry).is_some();

        let mut has_preview = false;
        if let Ok(rd) = std::fs::read_dir(&item_dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                if p.is_file()
                    && p.file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.eq_ignore_ascii_case("preview"))
                    && PREVIEW_COMP_SUFFIXES.contains(&ext_lower(&p).as_str())
                {
                    has_preview = true;
                    break;
                }
            }
        }

        let mut files: Vec<String> = Vec::new();
        if let Some(declared) = item.get("files").and_then(Value::as_array) {
            for rel_v in declared {
                if let Some(rel_s) = rel_v.as_str() {
                    let norm = rel_s.replace('\\', "/");
                    let trimmed = norm.trim_start_matches('/');
                    if !trimmed.is_empty() && resolve_confined_file(&item_dir, trimmed).is_some() {
                        files.push(trimmed.to_string());
                    }
                }
            }
        }
        if files.is_empty() {
            let mut all_p = Vec::new();
            collect_files_recursive(&item_dir, &mut all_p);
            all_p.sort();
            for p in all_p {
                if p.file_name().and_then(|s| s.to_str()) == Some("meta.json") {
                    continue;
                }
                if COMP_SOURCE_SUFFIXES.contains(&ext_lower(&p).as_str()) {
                    if let Ok(rel) = p.strip_prefix(&item_dir) {
                        files.push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
        }

        let name = {
            let n = val_str(&item, &["name"]);
            if n.is_empty() {
                comp_id.to_string()
            } else {
                n.to_string()
            }
        };
        let category = {
            let c = val_str(&item, &["category"]);
            if c.is_empty() {
                "misc".to_string()
            } else {
                c.to_string()
            }
        };
        let tags: Vec<Value> = item
            .get("tags")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| Value::String(s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut m = Map::new();
        m.insert("id".into(), Value::String(comp_id.to_string()));
        m.insert("name".into(), Value::String(name));
        m.insert("category".into(), Value::String(category));
        m.insert("kind".into(), Value::String(kind));
        m.insert(
            "description".into(),
            Value::String(val_str(&item, &["description"]).to_string()),
        );
        m.insert("tags".into(), Value::Array(tags));
        m.insert(
            "license".into(),
            Value::String(val_str(&item, &["license"]).to_string()),
        );
        m.insert(
            "source_url".into(),
            Value::String(val_str(&item, &["source_url"]).to_string()),
        );
        m.insert(
            "author".into(),
            Value::String(val_str(&item, &["author"]).to_string()),
        );
        m.insert(
            "files".into(),
            Value::Array(files.into_iter().map(Value::String).collect()),
        );
        m.insert("has_html".into(), Value::Bool(has_html));
        m.insert(
            "html_url".into(),
            if has_html {
                Value::String(format!("/api/components/{comp_id}/html"))
            } else {
                Value::Null
            },
        );
        m.insert("has_preview".into(), Value::Bool(has_preview));
        m.insert(
            "preview_url".into(),
            if has_preview {
                Value::String(format!("/api/components/{comp_id}/preview"))
            } else {
                Value::Null
            },
        );
        m.insert(
            "_dir".into(),
            Value::String(item_dir.to_string_lossy().to_string()),
        );
        m.insert("_entry".into(), Value::String(entry));

        components_map.insert(comp_id.to_string(), m.clone());
        components_list.push(m);
    }

    components_list.sort_by(|a, b| {
        val_str(a, &["category"])
            .cmp(val_str(b, &["category"]))
            .then_with(|| {
                val_str(a, &["name"])
                    .to_lowercase()
                    .cmp(&val_str(b, &["name"]).to_lowercase())
            })
    });

    (components_list, components_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnvSnapshot;

    #[test]
    fn indexes_styles_templates_components_and_blocks_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        let cfg = tmp.path().join("cfg");

        // Refero style fixture
        let sdir = data.join("refero-styles").join("styles").join("s-terminal-1");
        std::fs::create_dir_all(sdir.join("shots")).unwrap();
        std::fs::write(
            data.join("refero-styles").join("index.json"),
            r#"[{"id":"s-terminal-1","title":"Obsidian CLI","tagline":"Monospace terminal dashboard","theme":"dark","source_url":"https://linear.app"}]"#,
        )
        .unwrap();
        std::fs::write(
            sdir.join("DESIGN.md"),
            "# Obsidian CLI\nUse JetBrains Mono and #090d16 background.",
        )
        .unwrap();
        std::fs::write(sdir.join("shots").join("01.svg"), "<svg/>").unwrap();

        // Template fixture
        let web_en = data.join("prompt-driven-ui").join("websites-en");
        let prompts_en = data.join("prompt-driven-ui").join("prompts-en");
        std::fs::create_dir_all(&web_en).unwrap();
        std::fs::create_dir_all(&prompts_en).unwrap();
        std::fs::write(
            web_en.join("01-saas-landing.html"),
            r#"<html><head><title>SaaS Landing</title><meta name="description" content="Modern SaaS hero"></head><body><h1>Hello</h1></body></html>"#,
        )
        .unwrap();
        std::fs::write(prompts_en.join("01-saas-landing.md"), "Build a SaaS landing page").unwrap();

        // Component fixture
        let cdir = data.join("lnc-components").join("items").join("lnc-glow-btn");
        std::fs::create_dir_all(&cdir).unwrap();
        std::fs::write(
            data.join("lnc-components").join("index.json"),
            r#"[{"id":"lnc-glow-btn","name":"Glow Button","category":"buttons","kind":"html","description":"Neon button","tags":["glow"]}]"#,
        )
        .unwrap();
        std::fs::write(cdir.join("index.html"), "<button>Glow</button>").unwrap();

        let env = EnvSnapshot::from_pairs([
            ("STYLES_DATA_DIR", data.to_str().unwrap()),
            ("STYLES_CONFIG_DIR", cfg.to_str().unwrap()),
        ]);
        let settings = AppSettings::from_map(&env);
        let store = StyleLibraryStore::new(settings);
        let st = store.reindex();

        assert_eq!(st["styles_count"], 1);
        assert_eq!(st["templates_count"], 1);
        assert_eq!(st["components_count"], 1);

        let search = store.search("jetbrains", None, None, true, 10);
        assert_eq!(search["total_styles"], 1);

        let detail = store.get_style("s-terminal-1", true).unwrap();
        assert!(detail["design_md"].as_str().unwrap().contains("JetBrains Mono"));
        assert_eq!(detail["shots"].as_array().unwrap().len(), 1);

        // Traversal attempts return None
        assert!(store.get_style("../etc/passwd", true).is_none());
        assert!(store.get_component_file_path("lnc-glow-btn", "../../index.json").is_none());
        assert!(store.get_template_asset_path("en-01-saas-landing", "../prompts-en/01-saas-landing.md").is_none());
    }
}
