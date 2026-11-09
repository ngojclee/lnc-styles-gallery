import json
from pathlib import Path

ROOT = Path(r'//10.21.2.2/data/_agentsync/repos/refero-styles')
STYLES = ROOT / 'styles'
IDX = ROOT / 'index.json'


def preview_svg(title, tagline, colors, accent):
    sw = 640 // len(colors)
    blocks = ''.join(f'<rect x="{i*sw}" y="300" width="{sw}" height="100" fill="{c}"/>' for i, c in enumerate(colors))
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 400" width="640" height="400">
  <rect width="640" height="400" fill="#111318"/>
  <rect x="24" y="24" width="592" height="352" rx="18" fill="#181b23" stroke="#2a2f3d" stroke-width="1.5"/>
  <circle cx="52" cy="52" r="6" fill="{accent}"/>
  <text x="72" y="57" fill="{accent}" font-family="system-ui, sans-serif" font-size="13" font-weight="700">LNC INTERNAL</text>
  <text x="48" y="150" fill="#f2f4fa" font-family="system-ui, sans-serif" font-size="30" font-weight="800">{title}</text>
  <text x="48" y="185" fill="#9aa3b8" font-family="system-ui, sans-serif" font-size="14">{tagline}</text>
  <clipPath id="sw"><rect x="48" y="230" width="544" height="90" rx="10"/></clipPath>
  <g clip-path="url(#sw)">{blocks}</g>
</svg>'''


NEW = {
    'lnc-app-shell': dict(
        title='LNC App Shell',
        tagline='Shared sidebar console look used across lengoc.me apps (pose.lengoc.me reference). Neutral oklch surfaces, Inter Variable, 10px base radius, borderless gray sidebar, active row bold.',
        theme='light',
        url='https://pose.lengoc.me',
        source_url='https://pose.lengoc.me',
        colors=['#ffffff', '#f9fafb', '#e5e7eb', '#18181b', '#52525b', '#a1a1aa'],
        accent='#8b5cf6',
        design='''# LNC App Shell — Design System

The shared shell theme for internal lengoc.me applications (reference: pose.lengoc.me).
Neutral oklch grayscale surfaces, Inter Variable type, borderless sidebar, card grids.

## Color Tokens (light)

- `--background`: oklch(1 0 0) — pure white page
- `--foreground`: oklch(0.145 0 0)
- `--card`: oklch(1 0 0), `--border`: oklch(0.922 0 0), `--input`: oklch(0.922 0 0)
- `--primary`: oklch(0.205 0 0) (near-black), `--primary-foreground`: oklch(0.985 0 0)
- `--secondary` / `--muted` / `--accent`: oklch(0.97 0 0); `--muted-foreground`: oklch(0.556 0 0)
- `--ring`: oklch(0.708 0 0); `--destructive`: oklch(0.577 0.245 27.325)
- Sidebar: `--sidebar` rgb(249 250 251) (gray-50), `--sidebar-accent` rgb(243 244 246), `--sidebar-border` transparent — borderless, selected item = gray-100 fill + font-weight 600
- Charts: 1 red (oklch .58/.19/25), 2 green, 3 gold, 4 blue, 5 purple

## Color Tokens (dark)

- `--background`: oklch(0.145 0 0), `--card`/`--popover`: oklch(0.205 0 0)
- `--primary`: oklch(0.922 0 0) inverted (light button), `--border`: oklch(1 0 0 / 10%)
- `--muted`/`--secondary`/`--accent`: oklch(0.269 0 0); `--muted-foreground`: oklch(0.708 0 0)
- Sidebar: rgb(31 41 55) surface, rgb(55 65 81) accent, primary oklch(0.488 0.243 264.376)
- `.dark` class on `<html>` via `@custom-variant dark (&:is(.dark *))`

## Typography

- Sans: `'Inter Variable', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif`
- Headings: font-bold tracking-tight; PageHeader h1 = text-lg→2xl
- Metadata/muted: text-xs→sm `text-muted-foreground`

## Radius & Spacing

- `--radius`: 0.625rem (10px). sm=0.6x, md=0.8x, lg=1x, xl=1.4x, 2xl=1.8x, 3xl=2.2x, 4xl=2.6x
- Content container: `max-w-7xl mx-auto`, page padding px-4 pt-4 pb-10 sm:px-6 md:px-8
- Cards: rounded-[20px] bg-muted (stat tiles) / rounded-xl bg-card border

## Components

- shadcn/ui (Tailwind v4): Button, Card, Dialog, Sidebar (borderless), Breadcrumb, Tooltip, Table, Badge, Skeleton, Sonner toasts
- AppShell: AppSidebar (brand mark + nav groups + external links) + AppHeader (sidebar trigger, breadcrumb, theme toggle) + max-w-7xl Outlet
- ZoomViewport/ImageViewer for galleries; PageHeader + StatCard + EmptyState + FormSection primitives

## Do / Don't

- DO keep sidebar borderless (transparent `--sidebar-border`); DON'T add a vertical hairline
- DO bold the active sidebar item; DON'T use accent color fills for selection
- DO dual-theme via `.dark` class only; DON'T hardcode oklch values in components
''',
    ),
    'lnc-blueprint-lite': dict(
        title='LNC Blueprint Lite',
        tagline='LN UI Engine "lite" ops-console theme (canonical: lnc-notebooklm :8472). Dark-first surfaces, per-app plum accent, 12px radius, translucent hairlines, CSS-vars dual theme.',
        theme='dark',
        url='https://lnc-notebooklm.lengoc.me',
        source_url='https://lnc-notebooklm.lengoc.me',
        colors=['#0b0d14', '#12141d', '#171a26', '#c99bd8', '#e9edf5', '#8b93a7'],
        accent='#c99bd8',
        design='''# LNC Blueprint Lite — Design System

The "lite" ops-console theme from the Internal App Blueprint (Shape 0/B, vanilla, no build step).
Canonical production: lnc-notebooklm (port 8472). Full token set as CSS vars, dual light/dark.

## Color Tokens (dark, default)

- `--background`: #0b0d14; `--surface`: #12141d; `--surface-2`: #171a26; `--surface-hover`: #1b1f2e
- `--foreground`: #e9edf5; `--muted`: #8b93a7; `--faint`: #5d6478
- `--border`: rgba(160,180,220,.10); `--border-strong`: rgba(160,180,220,.17) — translucent hairlines, never solid gray
- `--primary`: #c99bd8 (plum, from app icon); `--primary-hover`: #dcbce8; `--primary-foreground`: #241026; `--primary-light`: rgba(201,155,216,.13)
- Semantic: success #00d492, warning #ff8904, danger #ff637e, error #ff6467, neutral #9f9fa9 (+ -bg variants)
- `--shadow-popover`: 0 10px 15px -3px rgb(0 0 0/.5); `--scroll`: rgba(160,180,220,.22)

## Color Tokens (light, `[data-theme=light]`)

- `--background`: #f5f3f7; `--surface`: #fdfcfe; `--surface-2`: #f6f3f9; `--surface-hover`: #ede8f2
- `--foreground`: #1f1624; `--muted`: #685d70; `--faint`: #9b8fa3
- `--border`: rgba(60,35,75,.12); `--border-focus`: rgba(107,56,128,.6)
- `--primary`: #6b3880; `--primary-hover`: #582c6b; `--primary-light`: rgba(107,56,128,.12)

## Typography

- `--font`: "Inter", ui-sans-serif, system-ui; `--mono`: ui-monospace, "SF Mono", "Cascadia Mono", Consolas

## Radius & Spacing

- `--radius`: 12px; `--radius-sm`: 8px. Cards 12px, inputs/buttons 12px, option rows 8px
- Sidebar 232px fixed; `.app-shell` grid `232px minmax(0,1fr)`; brand border spans full width

## Component Contract (class names)

- `.card` (+`.card-header`/`.card-body`/`.flush`), `.btn(-primary/-ghost/-danger/-sm/-icon)`
- `.badge` b-ok|b-warn|b-err|b-dim|b-plum + `.pill` health indicator
- `.stat-grid`/`.stat` metric tiles, `.kv` definition rows, `.data-table` (row hover, first/last cell keep card padding)
- `.field`/`.field-label`/`.field-help`, `.input-wrap` (eye toggle), `.copy-chip`+`.copy-btn`
- `.empty`, `.skel` shimmer, `.theme-toggle` sidebar-footer ghost row
- Dropdowns: custom `.select` trigger + `.select-menu` — popup radius MUST equal trigger radius (never native square list)

## Do / Don't

- DO swap only `--primary` per app (accent = app identity); DON'T rename tokens
- DO use `data-theme` attr + inline head script (no dark flash); DON'T add a system/auto third state
- DON'T use paper-sketch wobble/feTurbulence on layout lines — artwork only
''',
    ),
    'lnc-blueprint-full': dict(
        title='LNC Blueprint Full',
        tagline='Full LN UI Engine theme (@luxeclaw/ui-engine theme.css). Notion-like ops surfaces, blue accent, 6px radius, Engine* primitives, workbench layout tokens.',
        theme='light',
        url='https://ui-engine.lengoc.me',
        source_url='https://gitea.lengoc.me',
        colors=['#ffffff', '#fbfbfa', '#e3e3e0', '#37352f', '#2383e2', '#e8f0fe'],
        accent='#2383e2',
        design='''# LNC Blueprint Full — Design System

The full `@luxeclaw/ui-engine` theme used by Shape-A internal apps — the complete
token set + Engine* primitives. Notion-inspired ops surfaces, subtle shadows, blue accent.

## Color Tokens (light, default)

- `--bg-primary`: #ffffff; `--bg-secondary`: #fbfbfa; `--bg-card`: #ffffff; `--bg-hover`: #f1f1ef; `--bg-sidebar`: #f7f7f5
- `--text-primary`: #37352f; `--text-secondary`: #6b6b6b; `--text-dim`: #9b9a97
- `--border`: #e3e3e0; `--border-hover`: #c7c7c4
- `--accent`: #2383e2; `--accent-hover`: #1b6ec2; `--accent-subtle`: #e8f0fe
- Semantic: success #0f7b0f/#e6f4e6, warning #c77c14/#fef3e0, danger #e03e3e/#fce8e8
- Shadows: sm 0 1px 2px /4%, md 0 2px 8px /6%, lg 0 8px 24px /10%

## Color Tokens (dark, `[data-theme=dark]`)

- `--bg-primary`: #191919; `--bg-secondary`: #202020; `--bg-card`: #252525; `--bg-elevated`: #2e2e2e; `--bg-hover`: #2f2f2f; `--bg-sidebar`: #1e1e1e
- `--text-primary`: #e3e3e3; `--text-secondary`: #9b9b9b; `--text-dim`: #6b6b6b
- `--border`: #333; `--border-hover`: #444
- `--accent`: #529cca; `--accent-hover`: #6bb3de; `--accent-subtle`: #1a3a52
- Semantic: success #4caf50/#1a2e1a, warning #f59e0b/#2d2206, danger #ef5350/#3b1219

## Typography

- `--font`: 'Inter', -apple-system, 'Segoe UI', sans-serif
- `--header-band-height`: 58px

## Radius & Layout

- `--radius`: 6px; `--radius-lg`: 10px — crisper than lite theme
- Workbench tokens: `--engine-workbench-gap` 16px, sidebar width clamp(240px,16vw,300px), height max(620px, 100dvh-118px)
- Card contract: header padding 16px 20px, body 16px 18px 18px

## Components

- Engine* primitives (React: primitives.tsx) — shells, not brains; EngineSelectMenu for rounded lists
- theme.css = index.css + theme-light + theme-dark + semantic.css + components.css
- `engineTokens` export for adapters (never fork theme CSS)

## Do / Don't

- DO import `@luxeclaw/ui-engine/theme.css` once + set `data-theme` on html
- DON'T copy LN-OMS index.css into another app — use the package
- DON'T add a second theme to "match" a sample — map engineTokens instead
''',
    ),
    'ln-ui-engine': dict(
        title='LN UI Engine',
        tagline='Portable shared design engine for LN-OMS and internal apps — semantic tokens, Engine* primitives, light/dark theme files, class contract. Single source for the LN operations look.',
        theme='light',
        url='https://ui-engine.lengoc.me',
        source_url='https://gitea.lengoc.me',
        colors=['#ffffff', '#f7f7f5', '#252525', '#191919', '#2383e2', '#529cca'],
        accent='#0ea5e9',
        design='''# LN UI Engine — Shared Design System

The portable engine (repo `D:\\Python\\projects\\LN-UI-Engine`, published as
`@luxeclaw/ui-engine`). One source of truth for the LN operations look across apps.

## What it ships

- `src/theme/`: `index.css` + `theme-light.css` + `theme-dark.css` + `semantic.css` + `components.css`
- `src/tokens.ts`: `engineTokens` for adapters needing values not CSS
- `src/primitives.tsx`: Engine* React primitives (EngineCard, EngineSelectMenu, workbench shells)

## Token Surface (semantic layer)

- Surfaces: bg-primary / bg-secondary / bg-card / bg-elevated / bg-hover / bg-input / bg-sidebar
- Text: text-primary / text-secondary / text-dim
- Borders: border / border-hover
- Brand: accent / accent-hover / accent-subtle (light #2383e2, dark #529cca)
- Status: success / warning / danger (+ -bg each)
- Motion/depth: shadow-sm/md/lg; radius 6px / radius-lg 10px
- Layout: header-band-height 58px; engine-workbench-* sizing tokens

## Light / Dark

- Light is the default read (`:root, [data-theme="light"]`), dark overrides via `[data-theme="dark"]`
- Dark is NOT an inversion: surfaces step #191919 -> #252525 -> #2e2e2e, accent shifts to sky #529cca

## Rules

- Primitives are shells, not brains — app supplies data + callbacks
- Dropdown lists always match their control's radius (lite AND full)
- One theme import per app; `data-theme` attr controls mode
- Change theme = edit src/theme/*.css + bump version, consumers re-import

## Do / Don't

- DO map `engineTokens` when a consumer uses a different var naming
- DON'T fork theme CSS per app or paste LN-OMS index.css elsewhere
''',
    ),
}

idx = json.loads(IDX.read_text(encoding='utf-8'))
existing = {e['id'] for e in idx}
for sid, cfg in NEW.items():
    d = STYLES / sid
    d.mkdir(parents=True, exist_ok=True)
    (d / 'DESIGN.md').write_text(cfg['design'].strip() + '\n', encoding='utf-8')
    svg = preview_svg(cfg['title'], cfg['tagline'][:96], cfg['colors'], cfg['accent'])
    (d / 'preview.svg').write_text(svg, encoding='utf-8')
    meta = {
        'id': sid, 'url': cfg['url'], 'title': cfg['title'], 'tagline': cfg['tagline'],
        'theme': cfg['theme'], 'source_url': cfg['source_url'],
        'preview': 'preview.svg', 'shots': ['preview.svg'],
    }
    (d / 'meta.json').write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding='utf-8')
    if sid not in existing:
        idx.append({
            'id': sid, 'url': cfg['url'], 'title': cfg['title'],
            'tagline': cfg['tagline'], 'theme': cfg['theme'], 'source_url': cfg['source_url'],
        })
    print('created', sid)

IDX.write_text(json.dumps(idx, ensure_ascii=False, indent=2), encoding='utf-8')
print('index.json now', len(idx), 'entries')
