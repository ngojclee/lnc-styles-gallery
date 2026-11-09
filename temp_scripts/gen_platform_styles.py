"""Generate Platform & Vendor Design System styles for the corpus.

Each entry produces:
  styles/<id>/DESIGN.md   - Refero-format design-system doc (real official tokens)
  styles/<id>/meta.json   - catalog metadata (platform=ios/android/web, origin=official)
  styles/<id>/preview.svg - themed mockup (phone frame for mobile, window for web)
index.json is appended with matching catalog records.
"""
import json
from pathlib import Path

ROOT = Path(r'//10.21.2.2/data/_agentsync/repos/refero-styles')
STYLES = ROOT / 'styles'
INDEX = ROOT / 'index.json'

# ---------------------------------------------------------------------------
# Design system definitions — canonical token values from official specs.
# colors: (name, value, token, role)
# type_rows: (role, family, weight, size, line_height, tracking, token)
# ---------------------------------------------------------------------------
SYSTEMS = [
    {
        'id': 'apple-hig',
        'title': 'Apple HIG — iOS',
        'platform': 'ios',
        'frame': 'phone',
        'theme': 'light',
        'tagline': 'human interface design system — layered translucency, SF Pro type scale, 8pt rhythm, and semantic color that adapts between light and dark mode.',
        'docs': 'https://developer.apple.com/design/',
        'overview': (
            "Apple's Human Interface Guidelines define the native iOS look: content-first layouts on grouped gray "
            "backgrounds, SF Pro type scale tuned for legibility at arms length, and semantic system colors that "
            "shift between light and dark appearances. Depth is expressed through layering and material, not heavy "
            "shadows — cards sit on systemGroupedBackground tiers, bars use translucent materials, and separators "
            "are hairline-thin. Every interactive element respects the 44x44pt minimum touch target."
        ),
        'colors': [
            ('System Blue', '#007AFF', 'UIColor.systemBlue (light)', 'Primary accent — tint, buttons, links, selected states.'),
            ('System Blue (dark)', '#0A84FF', 'UIColor.systemBlue (dark)', 'Same role in dark appearance — brightened for contrast.'),
            ('System Green', '#34C759', 'UIColor.systemGreen', 'Success, on-states, positive confirmations.'),
            ('System Red', '#FF3B30', 'UIColor.systemRed', 'Destructive actions and errors.'),
            ('System Orange', '#FF9500', 'UIColor.systemOrange', 'Warnings and medium-priority alerts.'),
            ('System Indigo', '#5856D6', 'UIColor.systemIndigo', 'Secondary accent for distinguishing related actions.'),
            ('System Gray', '#8E8E93', 'UIColor.systemGray', 'Inactive/disabled elements and neutral iconography.'),
            ('Label', '#000000', 'UIColor.label', 'Primary text in light mode (inverts to #FFFFFF in dark).'),
            ('Secondary Label', 'rgba(60,60,67,0.60)', 'UIColor.secondaryLabel', 'Subheads, captions, metadata — 60% label ink.'),
            ('System Background', '#FFFFFF', 'UIColor.systemBackground', 'Primary page background (inverts to #000000 dark).'),
            ('Secondary System BG', '#F2F2F7', 'UIColor.secondarySystemBackground', 'Grouped content background — the signature iOS gray.'),
            ('System Fill', 'rgba(120,120,128,0.20)', 'UIColor.systemFill', 'Fills for sliders, segmented controls, progress.'),
            ('Separator', 'rgba(60,60,67,0.12)', 'UIColor.separator', 'Hairline dividers between list cells and sections.'),
        ],
        'fonts': [('SF Pro', 'system font — SF Pro Text <=19pt, SF Pro Display >=20pt, optical sizing automatic', '--font-sf-pro')],
        'type_rows': [
            ('large-title', 'SF Pro Display', '400/700', '34pt', '1.21', '0.37pt', '--text-large-title'),
            ('title-1', 'SF Pro Display', '400/700', '28pt', '1.21', '0.36pt', '--text-title1'),
            ('title-3', 'SF Pro Display', '400/600', '20pt', '1.25', '0.38pt', '--text-title3'),
            ('headline', 'SF Pro Text', '600', '17pt', '1.29', '-0.43pt', '--text-headline'),
            ('body', 'SF Pro Text', '400', '17pt', '1.29', '-0.43pt', '--text-body'),
            ('callout', 'SF Pro Text', '400', '16pt', '1.31', '-0.32pt', '--text-callout'),
            ('subheadline', 'SF Pro Text', '400', '15pt', '1.33', '-0.23pt', '--text-subheadline'),
            ('footnote', 'SF Pro Text', '400', '13pt', '1.38', '-0.08pt', '--text-footnote'),
            ('caption-1', 'SF Pro Text', '400', '12pt', '1.33', '0pt', '--text-caption1'),
        ],
        'base_unit': '8pt',
        'density': 'comfortable',
        'spacing': [('4', '4pt'), ('8', '8pt'), ('16', '16pt'), ('20', '20pt'), ('24', '24pt'), ('32', '32pt')],
        'radius': 'Continuous corners — ~10pt cards, capsule/pill buttons, grouped list insets 16pt',
        'components': [
            'List cells: full-bleed rows on white grouped cells, separator inset 16pt from content edge, chevron accessory right.',
            'Tab bar: 49pt, translucent material blur, 5 items max, selected icon tinted with accent color.',
            'Navigation bar: large title 34pt collapses to inline 17pt on scroll; trailing bar buttons use plain text, not chrome.',
            'Buttons: capsule or 10pt-radius filled shapes, min 44pt height; destructive uses systemRed fill or text.',
            'Grouped forms: inset 16pt margins, rounded-rect section containers on secondarySystemGroupedBackground.',
        ],
        'dos': [
            'Do use semantic colors (systemBlue, label, systemBackground) so dark mode adapts automatically.',
            'Do keep 44x44pt minimum touch targets; iPad pointer hover should still feel roomy.',
            'Do use grouped backgrounds for settings-style content and full-bleed white for media/detail views.',
            'Do let the nav bar go translucent on scroll — bars are material, not opaque slabs.',
            'Do respect Dynamic Type — the scale is relative, never hardcode absolute pixels.',
        ],
        'donts': [
            "Don't hardcode #007AFF — query UIColor.systemBlue so dark mode gets the brightened variant.",
            "Don't stack heavy shadows — iOS depth comes from layering and material blur, not elevation shadows.",
            "Don't use hairline borders under 0.5pt — separators are already at the visibility floor.",
            "Don't place chrome (borders, frames) around content that could be full-bleed.",
            "Don't mix platform conventions — no Material FABs, no hamburger menus, no pull-to-refresh spinners.",
        ],
        'css': ''':root {
  --ios-blue: #007AFF;          /* dark: #0A84FF */
  --ios-green: #34C759;         /* dark: #30D158 */
  --ios-red: #FF3B30;           /* dark: #FF453A */
  --ios-gray: #8E8E93;
  --ios-label: #000000;         /* dark: #FFFFFF */
  --ios-label-2: rgba(60,60,67,.60);
  --ios-bg: #FFFFFF;            /* dark: #000000 */
  --ios-bg-grouped: #F2F2F7;    /* dark: #000000 */
  --ios-cell: #FFFFFF;          /* dark: #1C1C1E */
  --ios-separator: rgba(60,60,67,.12);
  --ios-radius-card: 10pt;
  --ios-font: -apple-system, "SF Pro", system-ui, sans-serif;
}''',
        'mock': dict(page='#f2f2f7', surface='#ffffff', bar='#f9f9f9', card='#ffffff',
                     border='#d1d1d6', text='#000000', muted='#8e8e93', accent='#007aff',
                     accent_text='#ffffff', accent_sub='#e3f0ff', radius=10),
        'mock_tag': 'developer.apple.com · iOS 17+',
    },
    {
        'id': 'material-design-3',
        'title': 'Material Design 3',
        'platform': 'android',
        'frame': 'phone',
        'theme': 'light',
        'tagline': 'material you — dynamic color schemes, tonal surfaces without dividers, expressive shape scale, and state-layer interactions for Android.',
        'docs': 'https://m3.material.io/',
        'overview': (
            "Material Design 3 (Material You) is Google's current system: color schemes generated from a single "
            "source color into tonal palettes (primary/secondary/tertiary + containers), surfaces built from tonal "
            "elevation instead of shadows, and a full shape scale from extra-small 4dp to full 28dp. Components "
            "communicate state through translucent state layers (hover 8%, pressed 12%) rather than color swaps. "
            "On Android 12+ the same tokens can derive from the user's wallpaper — dynamic color is the signature."
        ),
        'colors': [
            ('Primary', '#6750A4', 'md.sys.color.primary', 'Key actions — FABs, filled buttons, active states.'),
            ('On Primary', '#FFFFFF', 'md.sys.color.on-primary', 'Text/icons on Primary fills.'),
            ('Primary Container', '#EADDFF', 'md.sys.color.primary-container', 'Tonal container — chips, selected list items, tonal buttons.'),
            ('On Primary Container', '#21005D', 'md.sys.color.on-primary-container', 'Text on primary-container fills.'),
            ('Secondary', '#625B71', 'md.sys.color.secondary', 'Less prominent actions — filter chips, secondary buttons.'),
            ('Secondary Container', '#E8DEF8', 'md.sys.color.secondary-container', 'Tonal secondary fills.'),
            ('Tertiary', '#7D5260', 'md.sys.color.tertiary', 'Balancing accent for contrasting highlights.'),
            ('Error', '#B3261E', 'md.sys.color.error', 'Errors and destructive emphasis.'),
            ('Error Container', '#F9DEDC', 'md.sys.color.error-container', 'Error surfaces and banners.'),
            ('Surface', '#FEF7FF', 'md.sys.color.surface', 'Default surface — cards, sheets, page background.'),
            ('Surface Variant', '#E7E0EC', 'md.sys.color.surface-variant', 'Variant surface for tonal contrast.'),
            ('On Surface', '#1D1B20', 'md.sys.color.on-surface', 'Primary text on surfaces.'),
            ('On Surface Variant', '#49454F', 'md.sys.color.on-surface-variant', 'Secondary text and icons.'),
            ('Outline', '#79747E', 'md.sys.color.outline', 'Borders and decorative lines where needed.'),
        ],
        'fonts': [('Roboto', 'default typeface — Roboto Flex for variable optical sizing, Google Sans for display', '--font-roboto')],
        'type_rows': [
            ('display-large', 'Roboto', '400', '57sp', '1.12', '-0.25sp', '--text-display-large'),
            ('headline-large', 'Roboto', '400', '32sp', '1.25', '0sp', '--text-headline-large'),
            ('title-large', 'Roboto', '400', '22sp', '1.27', '0sp', '--text-title-large'),
            ('title-medium', 'Roboto', '500', '16sp', '1.5', '0.15sp', '--text-title-medium'),
            ('body-large', 'Roboto', '400', '16sp', '1.5', '0.5sp', '--text-body-large'),
            ('body-medium', 'Roboto', '400', '14sp', '1.43', '0.25sp', '--text-body-medium'),
            ('label-large', 'Roboto', '500', '14sp', '1.43', '0.1sp', '--text-label-large'),
            ('label-small', 'Roboto', '500', '11sp', '1.45', '0.5sp', '--text-label-small'),
        ],
        'base_unit': '4dp',
        'density': 'comfortable',
        'spacing': [('4', '4dp'), ('8', '8dp'), ('12', '12dp'), ('16', '16dp'), ('24', '24dp'), ('32', '32dp')],
        'radius': 'Shape scale — xs 4dp, s 8dp, m 12dp, l 16dp, xl 28dp, full (buttons/FAB fully rounded)',
        'components': [
            'FAB: 56dp circle (or large 96dp), primary-container fill, elevation 3, bottom-right 16dp margin.',
            'Filled button: full-rounded (stadium), primary fill, label-large 14sp, min touch 48dp.',
            'Tonal button: secondary-container fill — the M3-preferred lower-emphasis action over outlined.',
            'Navigation bar: 80dp tall, 3-5 destinations, active indicator = pill-shaped secondary-container.',
            'Cards: m-shape 12dp radius, surface fill — separate by tonal elevation (surface-container-low), not hairline dividers.',
            'State layers: hover 8% / focus+pressed 12% on-color overlays — never bake a hover color.',
        ],
        'dos': [
            'Do map brand to the 13 tonal palette slots — primary/onPrimary/container pairs keep contrast automatic.',
            'Do use container colors for selection and low-emphasis surfaces instead of opacity tricks.',
            'Do prefer tonal elevation (surface-container tiers) over drop shadows for depth.',
            'Do use the full rounded shape language — M3 buttons and FABs are stadium-shaped.',
            'Do apply state layers (8% hover, 12% press) on interactive surfaces.',
        ],
        'donts': [
            "Don't use M2-era 4dp default radius everywhere — M3 cards are 12dp, buttons fully rounded.",
            "Don't put black text on primary-container — use onPrimaryContainer (#21005D baseline).",
            "Don't rely on elevation shadows alone — M3 surfaces are tinted, not just lifted.",
            "Don't hardcode the baseline purple — think in roles (primary, tertiary) that dynamic color can re-tint.",
            "Don't mix in iOS conventions — no 44pt targets (use 48dp), no chevron lists.",
        ],
        'css': ''':root {
  --md-primary: #6750A4;         /* dark: #D0BCFF */
  --md-on-primary: #FFFFFF;
  --md-primary-container: #EADDFF;
  --md-on-primary-container: #21005D;
  --md-secondary: #625B71;
  --md-tertiary: #7D5260;
  --md-error: #B3261E;
  --md-surface: #FEF7FF;         /* dark: #141218 */
  --md-surface-variant: #E7E0EC;
  --md-on-surface: #1D1B20;
  --md-on-surface-variant: #49454F;
  --md-outline: #79747E;
  --md-radius-card: 12dp;
  --md-radius-button: 999px;     /* stadium */
  --md-font: "Roboto", system-ui, sans-serif;
}''',
        'mock': dict(page='#fef7ff', surface='#fef7ff', bar='#f3edff', card='#ffffff',
                     border='#e7e0ec', text='#1d1b20', muted='#79747e', accent='#6750a4',
                     accent_text='#ffffff', accent_sub='#eaddff', radius=12),
        'mock_tag': 'm3.material.io · Material You',
    },
    {
        'id': 'microsoft-fluent-2',
        'title': 'Microsoft Fluent 2',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'fluent design 2 — calm productivity chrome: subtle depth tiers, Segoe type ramp, communication blue accents, and a 4px grid for dense enterprise surfaces.',
        'docs': 'https://fluent2.microsoft.design/',
        'overview': (
            "Fluent 2 is Microsoft's design language for Office, Teams and Windows surfaces: quiet neutral chrome "
            "(#FAFAFA/#FFFFFF layering), a single Communication Blue (#0F6CBD) carrying all interactive meaning, "
            "and depth expressed through a disciplined shadow ramp (shadow2/4/8/16/28/64) paired with 2-8px radii. "
            "Density is high — 4px spacing grid, compact Segoe UI type — built for long-form productivity work "
            "where the interface should disappear behind the document."
        ),
        'colors': [
            ('Compound Brand', '#0F6CBD', 'colorCompoundBrandBackground', 'Primary actions — buttons, links, focus emphasis.'),
            ('Brand Hover', '#115EA3', 'colorCompoundBrandBackgroundHover', 'Hover on brand fills.'),
            ('Brand Pressed', '#0C3B5E', 'colorCompoundBrandBackgroundPressed', 'Pressed state on brand fills.'),
            ('Neutral Foreground', '#242424', 'colorNeutralForeground1', 'Primary text.'),
            ('Neutral Foreground 2', '#424242', 'colorNeutralForeground2', 'Secondary text and labels.'),
            ('Neutral Foreground 3', '#616161', 'colorNeutralForeground3', 'Captions, placeholders, disabled.'),
            ('Neutral Background', '#FFFFFF', 'colorNeutralBackground1', 'Card and page surfaces.'),
            ('Neutral Background 3', '#F5F5F5', 'colorNeutralBackground3', 'Subtle inset areas and wells.'),
            ('Neutral Stroke', '#D1D1D1', 'colorNeutralStroke1', 'Borders on controls and cards.'),
            ('Status Success', '#107C10', 'colorStatusSuccessForeground1', 'Success messages and indicators.'),
            ('Status Danger', '#B10E1C', 'colorStatusDangerForeground1', 'Errors and destructive emphasis.'),
            ('Status Warning', '#CA5010', 'colorStatusWarningForeground1', 'Warnings.'),
        ],
        'fonts': [('Segoe UI', 'system typeface across Windows/Office; web stack falls back to system sans', '--font-segoe')],
        'type_rows': [
            ('caption-1', 'Segoe UI', '400', '12px', '1.33', '0', '--text-caption1'),
            ('body-1', 'Segoe UI', '400', '14px', '1.43', '0', '--text-body1'),
            ('subtitle-1', 'Segoe UI', '600', '16px', '1.38', '0', '--text-subtitle1'),
            ('title-3', 'Segoe UI', '600', '20px', '1.4', '0', '--text-title3'),
            ('title-1', 'Segoe UI', '600', '32px', '1.25', '0', '--text-title1'),
            ('display', 'Segoe UI', '600', '68px', '1.35', '0', '--text-display'),
        ],
        'base_unit': '4px',
        'density': 'compact',
        'spacing': [('4', '4px'), ('8', '8px'), ('12', '12px'), ('16', '16px'), ('20', '20px'), ('24', '24px')],
        'radius': 'none 0, small 2px, medium 4px, large 6px, xlarge 8px, circular (buttons/inputs use medium 4px)',
        'components': [
            'Primary button: 4px radius, brand fill, 32px height, Semibold 14px — single accent per view.',
            'Card: 4-8px radius, shadow4 at rest, shadow8 on hover — depth comes from the shadow ramp.',
            'Inputs: bottom-emphasis underline variant or 4px rounded outline, label-above, 32px tall.',
            'Focus: 2px stroke outline offset 2px (strokeFocus2) — always visible, never suppressed.',
            'Backdrop: flyouts/dialogs use acrylic-like translucency on Windows, shadow64 elevation.',
        ],
        'dos': [
            'Do reserve Communication Blue for interactive elements only — chrome stays neutral gray.',
            'Do pick from the shadow ramp rather than custom box-shadows.',
            'Do keep density compact — 4px grid, 32px controls, 14px body.',
            'Do pair 4px radius on controls with 8px on cards/containers for subtle hierarchy.',
        ],
        'donts': [
            "Don't introduce a second accent hue — Fluent's discipline is one brand color plus neutrals.",
            "Don't use heavy borders — hairline NeutralStroke1 or tonal separation instead.",
            "Don't set body text below 12px — caption floor for legibility in dense grids.",
            "Don't round controls beyond 8px — Fluent is softly squared, not pill-shaped.",
        ],
        'css': ''':root {
  --fluent-brand: #0F6CBD;
  --fluent-brand-hover: #115EA3;
  --fluent-fg: #242424;
  --fluent-fg-2: #424242;
  --fluent-fg-3: #616161;
  --fluent-bg: #FFFFFF;
  --fluent-bg-3: #F5F5F5;
  --fluent-stroke: #D1D1D1;
  --fluent-success: #107C10;
  --fluent-danger: #B10E1C;
  --fluent-radius: 4px;
  --fluent-font: "Segoe UI", system-ui, sans-serif;
}''',
        'mock': dict(page='#f5f5f5', surface='#ffffff', sidebar='#fafafa', card='#ffffff',
                     border='#e1dfdd', text='#242424', muted='#616161', accent='#0f6cbd',
                     accent_text='#ffffff', accent_sub='#deecf9', radius=4),
        'mock_tag': 'fluent2.microsoft.design',
    },
    {
        'id': 'ibm-carbon',
        'title': 'IBM Carbon',
        'platform': 'web',
        'frame': 'window',
        'theme': 'dark',
        'tagline': 'carbon design system — productive density on layered grays, Plex typography, signature zero-radius corners, and the unmistakable IBM blue.',
        'docs': 'https://carbondesignsystem.com/',
        'overview': (
            "IBM Carbon is the enterprise workhorse aesthetic: Plex Sans set tight on an 8px spatial system, "
            "surface hierarchy through flat tonal layers (White theme #F4F4F4, Gray-100 theme #161616), and a "
            "complete absence of border radius — Carbon is deliberately square. Interactive meaning lives in one "
            "blue (#0F62FE light, #4589FF dark). Built for data-dense tooling: tables, structured grids, and "
            "productive reading rather than marketing flourish."
        ),
        'colors': [
            ('Interactive', '#0F62FE', 'interactive (light)', 'Primary action color — buttons, links, selected states.'),
            ('Interactive Dark', '#4589FF', 'interactive (g100)', 'Same role in dark themes — brightened blue.'),
            ('Background Dark', '#161616', 'background (g100)', 'Gray-100 theme page surface.'),
            ('Layer 01 Dark', '#262626', 'layer-01 (g100)', 'First raised layer on dark surfaces.'),
            ('Layer 02 Dark', '#393939', 'layer-02 (g100)', 'Second raised layer / hover-tier.'),
            ('Background Light', '#FFFFFF', 'background (white)', 'White theme page surface.'),
            ('Layer 01 Light', '#F4F4F4', 'layer-01 (white)', 'First raised layer on white surfaces.'),
            ('Text Primary', '#161616', 'text-primary (light)', 'Primary reading text on light themes.'),
            ('Text Primary Dark', '#F4F4F4', 'text-primary (g100)', 'Primary reading text on dark themes.'),
            ('Text Secondary', '#525252', 'text-secondary', 'Secondary text, captions, helper copy.'),
            ('Border Subtle', '#E0E0E0', 'border-subtle-01', 'Hairline borders on light layer-01.'),
            ('Support Error', '#DA1E28', 'support-error', 'Errors and destructive states.'),
            ('Support Success', '#24A148', 'support-success', 'Positive confirmations.'),
            ('Support Warning', '#F1C21B', 'support-warning', 'Warnings — paired with dark text.'),
        ],
        'fonts': [('IBM Plex Sans', 'house typeface — Plex Sans Condensed for display, Plex Mono for code/data', '--font-plex')],
        'type_rows': [
            ('caption', 'IBM Plex Sans', '400', '12px', '1.33', '0.32px', '--text-caption'),
            ('label', 'IBM Plex Sans', '400', '12px', '1.33', '0.32px', '--text-label'),
            ('body-compact', 'IBM Plex Sans', '400', '14px', '1.29', '0.16px', '--text-body-compact'),
            ('body', 'IBM Plex Sans', '400', '14px', '1.43', '0.16px', '--text-body'),
            ('heading-compact', 'IBM Plex Sans', '600', '14px', '1.29', '0.16px', '--text-heading-compact'),
            ('heading-04', 'IBM Plex Sans', '400', '20px', '1.4', '0', '--text-heading-04'),
            ('heading-06', 'IBM Plex Sans', '300', '28px', '1.29', '0', '--text-heading-06'),
            ('display', 'IBM Plex Sans', '300', '54px', '1.19', '0', '--text-display'),
        ],
        'base_unit': '8px (2px mini-unit)',
        'density': 'compact',
        'spacing': [('03', '8px'), ('05', '16px'), ('06', '24px'), ('07', '32px'), ('09', '48px'), ('10', '64px')],
        'radius': 'Zero — Carbon corners are deliberately square; do not round cards, buttons, or inputs',
        'components': [
            'Button: 0 radius, 48px height, blue60 fill, label 14px — expressive buttons can run 64px.',
            'Data table: row separators on border-subtle, zebra optional, 32/40/48px row densities.',
            'Layering: background → layer-01 → layer-02 → layer-03 alternates flat grays; borders only where needed.',
            'Tag: 0 radius chip, tonal fill from the same hue family as status meaning.',
            'Progress/loader: blue60 determinate bars; skeleton shimmer on layer tones.',
        ],
        'dos': [
            'Do keep all corners square — zero radius is the Carbon signature.',
            'Do use layer tiers for elevation instead of shadows.',
            'Do set productive body at 14px — Carbon optimizes for density.',
            'Do use #0F62FE for interactive affordance and nothing decorative.',
            'Do set code and tabular figures in Plex Mono.',
        ],
        'donts': [
            "Don't add border-radius — even 2px breaks the system.",
            "Don't introduce drop shadows — flat tonal layers only.",
            "Don't use the interactive blue for decoration or large fills.",
            "Don't mix themes mid-surface — White, g10, g90, g100 zones stay coherent.",
        ],
        'css': ''':root {
  --carbon-interactive: #0F62FE;   /* dark: #4589FF */
  --carbon-bg: #FFFFFF;            /* g100: #161616 */
  --carbon-layer-01: #F4F4F4;      /* g100: #262626 */
  --carbon-layer-02: #FFFFFF;      /* g100: #393939 */
  --carbon-text: #161616;          /* g100: #F4F4F4 */
  --carbon-text-2: #525252;
  --carbon-border: #E0E0E0;
  --carbon-error: #DA1E28;
  --carbon-success: #24A148;
  --carbon-radius: 0;
  --carbon-font: "IBM Plex Sans", system-ui, sans-serif;
  --carbon-mono: "IBM Plex Mono", monospace;
}''',
        'mock': dict(page='#161616', surface='#262626', sidebar='#161616', card='#262626',
                     border='#393939', text='#f4f4f4', muted='#8d8d8d', accent='#4589ff',
                     accent_text='#ffffff', accent_sub='#262e42', radius=0),
        'mock_tag': 'carbondesignsystem.com · g100',
    },
    {
        'id': 'shopify-polaris',
        'title': 'Shopify Polaris',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'polaris — merchant-first admin chrome: quiet grays, near-black brand buttons, generous card radius, and Inter set for long commerce workflows.',
        'docs': 'https://polaris.shopify.com/',
        'overview': (
            "Polaris styles the Shopify admin: calm warm-gray app frames, white content cards with soft 12-16px "
            "radius, and a deliberately restrained brand expression — primary buttons are near-black, not "
            "Shopify green. Inter handles all UI type. The system optimizes for merchants doing repetitive "
            "operational work: dense index tables, clear status badges, and forms that explain themselves."
        ),
        'colors': [
            ('Brand Fill', '#1A1A1A', 'bg-fill-brand', 'Primary button fill — the signature near-black.'),
            ('Brand Fill Hover', '#303030', 'bg-fill-brand-hover', 'Hover on brand fills.'),
            ('App Background', '#F1F1F1', 'bg', 'Admin app frame background.'),
            ('Surface', '#FFFFFF', 'bg-surface', 'Cards and content surfaces.'),
            ('Surface Secondary', '#F7F7F7', 'bg-surface-secondary', 'Inset areas within cards.'),
            ('Text', '#303030', 'text', 'Primary text.'),
            ('Text Secondary', '#616161', 'text-secondary', 'Subdued labels and metadata.'),
            ('Border', '#C9CCCF', 'border', 'Control and card borders.'),
            ('Border Secondary', '#EBEBEB', 'border-secondary', 'Hairline separators.'),
            ('Focus', '#005BD3', 'focus', 'Focus ring and selected affordance.'),
            ('Critical', '#D72C0D', 'text-critical', 'Destructive actions and errors.'),
            ('Success', '#0C5132', 'text-success', 'Success text and badges.'),
        ],
        'fonts': [('Inter', 'UI typeface for all admin surfaces — tabular figures for money/dates', '--font-inter')],
        'type_rows': [
            ('heading-xs', 'Inter', '500', '12px', '1.33', '0', '--text-heading-xs'),
            ('body-sm', 'Inter', '400', '13px', '1.38', '0', '--text-body-sm'),
            ('body-md', 'Inter', '400', '14px', '1.43', '0', '--text-body-md'),
            ('heading-sm', 'Inter', '600', '13px', '1.38', '0', '--text-heading-sm'),
            ('heading-md', 'Inter', '600', '15px', '1.33', '0', '--text-heading-md'),
            ('heading-lg', 'Inter', '650', '17px', '1.29', '0', '--text-heading-lg'),
            ('heading-xl', 'Inter', '650', '24px', '1.25', '0', '--text-heading-xl'),
        ],
        'base_unit': '4px',
        'density': 'comfortable',
        'spacing': [('100', '4px'), ('200', '8px'), ('300', '12px'), ('400', '16px'), ('500', '20px'), ('600', '24px')],
        'radius': 'Cards 12-16px, buttons/inputs 8px, badges full — Polaris is the roundest admin system here',
        'components': [
            'Primary button: near-black fill, 8px radius, 28-36px height — never green for actions.',
            'Index table: sticky header, row hover, bulk-select checkboxes, subdued column labels 12px.',
            'Badge: tonal pill (success/info/warning/critical), sentence-case, no icons unless needed.',
            'Card: 16px padding minimum, 12-16px radius, optional sectioned dividers.',
            'Text field: 8px radius outline, label-above, helper text under field in secondary tone.',
        ],
        'dos': [
            'Do keep primary actions near-black — brand is expressed through restraint.',
            'Do use tonal badges for status — Polaris never uses solid-color chips.',
            'Do prefer IndexTable patterns for any resource list.',
            'Do use tabular numbers for money, SKUs, and counts.',
            'Do let cards breathe — 12-16px radius with clear section separation.',
        ],
        'donts': [
            "Don't color buttons Shopify green — actions are near-black by design.",
            "Don't put content directly on the app gray — always on white surface cards.",
            "Don't use sentence-case violations — Polaris UI copy is conversational.",
            "Don't stack more than one primary button per surface.",
        ],
        'css': ''':root {
  --polaris-brand: #1A1A1A;
  --polaris-brand-hover: #303030;
  --polaris-bg: #F1F1F1;
  --polaris-surface: #FFFFFF;
  --polaris-surface-2: #F7F7F7;
  --polaris-text: #303030;
  --polaris-text-2: #616161;
  --polaris-border: #C9CCCF;
  --polaris-focus: #005BD3;
  --polaris-critical: #D72C0D;
  --polaris-radius-card: 12px;
  --polaris-radius-control: 8px;
  --polaris-font: "Inter", system-ui, sans-serif;
}''',
        'mock': dict(page='#f1f1f1', surface='#ffffff', sidebar='#ebebeb', card='#ffffff',
                     border='#e3e3e3', text='#303030', muted='#616161', accent='#1a1a1a',
                     accent_text='#ffffff', accent_sub='#ebebeb', radius=12),
        'mock_tag': 'polaris.shopify.com',
    },
    {
        'id': 'atlassian-design',
        'title': 'Atlassian Design System',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'atlassian — teamwork chrome: loft blue accents on structured neutrals, Charlie type, tight 3-4px radii, and navigation-heavy product framing.',
        'docs': 'https://atlassian.design/',
        'overview': (
            "Atlassian's system powers Jira, Confluence and Trello: a refreshed Loft Blue (#0C66E4) over "
            "structured neutral surfaces, Charlie Text for reading comfort at 14px, and tight radii (3-4px "
            "legacy, 8px on newer surfaces). Pages are navigation-dense — global side rails, project sidebars, "
            "sticky headers — so content chrome stays flat and the single blue carries all affordance."
        ),
        'colors': [
            ('Loft Blue', '#0C66E4', 'background-brand-bold / link', 'Primary interactive color — buttons, links, focus.'),
            ('Blue Subtlest', '#E9F2FF', 'background-accent-blue-subtlest', 'Selected/hover tint surfaces.'),
            ('Classic Blue', '#0052CC', 'legacy B400', 'Pre-2023 brand blue — still in older surfaces.'),
            ('Text', '#172B4D', 'text (N800)', 'Primary text — dark desaturated navy.'),
            ('Text Subtle', '#44546F', 'text-subtle (N600)', 'Secondary text and labels.'),
            ('Neutral', '#F1F2F4', 'background-neutral (N20)', 'Tertiary buttons, subtle fills.'),
            ('Neutral Subtle', '#F7F8F9', 'background-neutral-subtle', 'Inset areas.'),
            ('Surface', '#FFFFFF', 'elevation-surface', 'Cards and content surfaces.'),
            ('Border', '#DCDFE4', 'border', 'Hairlines and control outlines.'),
            ('Danger', '#C9372C', 'icon-danger / R400', 'Destructive emphasis.'),
            ('Success', '#22A06B', 'icon-success / G300', 'Positive states.'),
            ('Warning', '#E2B203', 'icon-warning / Y300', 'Caution states.'),
        ],
        'fonts': [('Charlie Text', 'house typeface — Charlie Display for headlines, system stack fallback', '--font-charlie')],
        'type_rows': [
            ('body', 'Charlie Text', '400', '14px', '1.43', '-0.006em', '--text-body'),
            ('small', 'Charlie Text', '400', '11px', '1.45', '0', '--text-small'),
            ('h500', 'Charlie Text', '600', '16px', '1.25', '-0.006em', '--text-h500'),
            ('h600', 'Charlie Text', '600', '20px', '1.2', '-0.008em', '--text-h600'),
            ('h700', 'Charlie Display', '600', '24px', '1.25', '-0.01em', '--text-h700'),
            ('h800', 'Charlie Display', '500', '29px', '1.17', '-0.01em', '--text-h800'),
        ],
        'base_unit': '8px',
        'density': 'compact',
        'spacing': [('050', '4px'), ('100', '8px'), ('200', '16px'), ('300', '24px'), ('400', '32px'), ('500', '40px')],
        'radius': '3px legacy controls → 4px current tokens, 8px on newer card surfaces — tight, never pill',
        'components': [
            'Button: 4px radius, neutral fill default, brand-bold for primary — flat, no shadows at rest.',
            'Lozenge: subtle-fill uppercase-ish status chip for workflow states.',
            'Modal/flag: elevation via shadow overlay tokens — dialogs at elevation.overlay.',
            'Inline edit: content looks like text until hover reveals affordance — editing stays invisible.',
            'Side navigation: narrow global rail + contextual project sidebar, icons-only at collapse.',
        ],
        'dos': [
            'Do keep one blue for affordance — Loft Blue is the only action color.',
            'Do use N-surface hierarchy (neutral fills) for secondary buttons instead of outlines.',
            'Do let content surfaces stay flat — elevation is reserved for overlays.',
            'Do pair Charlie Text body 14px with Display weights at h700+.',
        ],
        'donts': [
            "Don't mix legacy #0052CC with Loft Blue in the same surface — pick one era.",
            "Don't round corners past 8px — the system reads softly squared.",
            "Don't add decorative gradients — surfaces are flat neutrals.",
            "Don't hide destructive actions behind blue — use Danger red sparingly but clearly.",
        ],
        'css': ''':root {
  --atl-brand: #0C66E4;
  --atl-blue-subtle: #E9F2FF;
  --atl-text: #172B4D;
  --atl-text-2: #44546F;
  --atl-neutral: #F1F2F4;
  --atl-surface: #FFFFFF;
  --atl-border: #DCDFE4;
  --atl-danger: #C9372C;
  --atl-success: #22A06B;
  --atl-radius: 4px;
  --atl-font: "Charlie Text", -apple-system, system-ui, sans-serif;
}''',
        'mock': dict(page='#f7f8f9', surface='#ffffff', sidebar='#f1f2f4', card='#ffffff',
                     border='#dcdfe4', text='#172b4d', muted='#44546f', accent='#0c66e4',
                     accent_text='#ffffff', accent_sub='#e9f2ff', radius=4),
        'mock_tag': 'atlassian.design · Loft Blue',
    },
    {
        'id': 'github-primer',
        'title': 'GitHub Primer',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'primer — code-review chrome: system font stack, canvas/subtle layering, accent-fg blue, and 6px radii tuned for diff-dense developer surfaces.',
        'docs': 'https://primer.style/',
        'overview': (
            "GitHub Primer is the quietest system in the set: near-zero decoration, canvas-default/subtle "
            "layering to separate content zones, a single accent blue (#0969DA) for links and focus, and "
            "functional color reserved strictly for state (success green, danger red, attention amber, done "
            "purple). 6px medium radius everywhere, system font stack, 14px body — built so code, diffs and "
            "data carry all the visual weight."
        ),
        'colors': [
            ('Accent', '#0969DA', 'fgColor-accent / accent-fg', 'Links, focus emphasis, primary actions.'),
            ('Accent Emphasis', '#0550AE', 'accent-emphasis', 'Primary button fill.'),
            ('Canvas Default', '#FFFFFF', 'bgColor-default / canvas', 'Primary surface.'),
            ('Canvas Subtle', '#F6F8FA', 'bgColor-muted / canvas-subtle', 'Inset areas, secondary zones.'),
            ('FG Default', '#1F2328', 'fgColor-default', 'Primary text.'),
            ('FG Muted', '#59636E', 'fgColor-muted', 'Secondary text.'),
            ('Border Default', '#D1D9E0', 'borderColor-default', 'Standard borders.'),
            ('Border Muted', '#D1D9E080', 'borderColor-muted', 'Hairline separators.'),
            ('Success', '#1A7F37', 'fgColor-success', 'Passing states, merged, additions.'),
            ('Danger', '#D1242F', 'fgColor-danger', 'Deletions, errors, destructive.'),
            ('Attention', '#9A6700', 'fgColor-attention', 'Warnings and pending states.'),
            ('Done', '#8250DF', 'fgColor-done', 'Merged/closed purple — GitHub signature.'),
        ],
        'fonts': [('System stack', '-apple-system, BlinkMacSystemFont, Segoe UI, Noto Sans, Helvetica, Arial', '--font-system'), ('Mono stack', 'ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, monospace', '--font-mono')],
        'type_rows': [
            ('text-small', 'system', '400', '12px', '1.5', '0', '--text-small'),
            ('body', 'system', '400', '14px', '1.5', '0', '--text-body'),
            ('body-large', 'system', '400', '16px', '1.5', '0', '--text-body-lg'),
            ('title-small', 'system', '600', '14px', '1.5', '0', '--text-title-sm'),
            ('title-medium', 'system', '600', '20px', '1.6', '0', '--text-title-md'),
            ('title-large', 'system', '600', '32px', '1.5', '0', '--text-title-lg'),
        ],
        'base_unit': '8px',
        'density': 'compact',
        'spacing': [('1', '4px'), ('2', '8px'), ('3', '16px'), ('4', '24px'), ('5', '32px'), ('6', '40px')],
        'radius': 'small 4px, medium 6px (default everywhere), large 12px, full — uniform 6px reads as signature',
        'components': [
            'Button: 6px radius, canvas-default with border for default, accent-emphasis for primary.',
            'Box/Panel: 6px radius, canvas-subtle inner headers — the classic repo file-list look.',
            'Label: small pill on border-muted, status colors only for state labels.',
            'Underline nav: text links with 2px accent underline on active — no chrome.',
            'Data zones: canvas-default vs canvas-subtle alternation instead of shadows.',
        ],
        'dos': [
            'Do use canvas-default/subtle alternation for zoning instead of borders everywhere.',
            'Do keep functional color for state only — green=success, red=danger, purple=done.',
            'Do set all interactive text in accent-fg and keep it reserved.',
            'Do use the mono stack for anything code, SHAs, or file paths.',
            'Do keep chrome minimal — Primer disappears behind content by design.',
        ],
        'donts': [
            "Don't introduce non-standard accent colors — the palette is closed.",
            "Don't use purple for anything except merged/done semantics.",
            "Don't add shadows for hierarchy — use subtle-canvas fills.",
            "Don't restyle links with underline always-on — hover reveals it.",
        ],
        'css': ''':root {
  --primer-accent: #0969DA;
  --primer-accent-emphasis: #0550AE;
  --primer-canvas: #FFFFFF;
  --primer-canvas-subtle: #F6F8FA;
  --primer-fg: #1F2328;
  --primer-fg-muted: #59636E;
  --primer-border: #D1D9E0;
  --primer-success: #1A7F37;
  --primer-danger: #D1242F;
  --primer-done: #8250DF;
  --primer-radius: 6px;
  --primer-font: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  --primer-mono: ui-monospace, "SF Mono", Menlo, monospace;
}''',
        'mock': dict(page='#f6f8fa', surface='#ffffff', sidebar='#f6f8fa', card='#ffffff',
                     border='#d1d9e0', text='#1f2328', muted='#59636e', accent='#0969da',
                     accent_text='#ffffff', accent_sub='#ddf4ff', radius=6),
        'mock_tag': 'primer.style',
    },
    {
        'id': 'ant-design',
        'title': 'Ant Design 5',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'ant design — enterprise workbench: primary blue on functional grays, 8px grid, 6px radius, and the fullest component vocabulary in the set.',
        'docs': 'https://ant.design/',
        'overview': (
            "Ant Design is the reference enterprise system for admin tooling: one Geeking Blue (#1677FF) for "
            "all interactive meaning, functional grays from a 10-step scale, 8px spatial rhythm, and consistent "
            "6px radius. Its signature is completeness — the same tokens drive tables, forms, trees, upload "
            "zones and dashboards, so dense back-office UIs stay coherent. Typography is a restrained system "
            "stack that keeps Chinese and Latin text aligned."
        ),
        'colors': [
            ('Primary', '#1677FF', 'colorPrimary', 'All interactive affordance — buttons, links, focus.'),
            ('Success', '#52C41A', 'colorSuccess', 'Positive results and confirmations.'),
            ('Warning', '#FAAD14', 'colorWarning', 'Cautions and pending states.'),
            ('Error', '#FF4D4F', 'colorError', 'Errors, validation failures, destructive.'),
            ('Info', '#1677FF', 'colorInfo', 'Informational — shares the primary blue.'),
            ('Text', 'rgba(0,0,0,0.88)', 'colorText', 'Primary text — near-black at 88%.'),
            ('Text Secondary', 'rgba(0,0,0,0.65)', 'colorTextSecondary', 'Secondary text at 65%.'),
            ('Text Tertiary', 'rgba(0,0,0,0.45)', 'colorTextTertiary', 'Captions, placeholders, disabled.'),
            ('Layout BG', '#F5F5F5', 'colorBgLayout', 'App frame background behind content cards.'),
            ('Container BG', '#FFFFFF', 'colorBgContainer', 'Cards, tables, popover surfaces.'),
            ('Border', '#D9D9D9', 'colorBorder', 'Control and container outlines.'),
            ('Border Secondary', '#F0F0F0', 'colorBorderSecondary', 'Inner hairlines and row separators.'),
        ],
        'fonts': [('System stack', '-apple-system, Segoe UI, Roboto, Helvetica Neue, PingFang SC, Microsoft YaHei', '--font-system')],
        'type_rows': [
            ('text', 'system', '400', '14px', '1.57', '0', '--text-body'),
            ('text-sm', 'system', '400', '12px', '1.67', '0', '--text-sm'),
            ('title-h5', 'system', '600', '16px', '1.5', '0', '--text-h5'),
            ('title-h4', 'system', '600', '20px', '1.4', '0', '--text-h4'),
            ('title-h2', 'system', '600', '30px', '1.27', '0', '--text-h2'),
            ('title-h1', 'system', '600', '38px', '1.21', '0', '--text-h1'),
        ],
        'base_unit': '8px',
        'density': 'compact',
        'spacing': [('xs', '8px'), ('sm', '12px'), ('md', '16px'), ('lg', '24px'), ('xl', '32px'), ('xxl', '48px')],
        'radius': 'borderRadius 6px default, sm 4px, lg 8px — consistent medium rounding across all controls',
        'components': [
            'Button: 6px radius, primary fill for main action, ghost/outline secondary, 32px default height.',
            'Table: bordered optional, row hover tint, sticky header + pagination footer.',
            'Form: vertical labels for admin density, inline for toolbars, error under-field in #FF4D4F.',
            'Menu: dark sidebar variant (#001529) is the classic admin shell.',
            'Alert/Message: tonal fills keyed to the four status colors, 8px radius.',
        ],
        'dos': [
            'Do use colorPrimary for every interactive affordance — never a second hue.',
            'Do keep 6px radius consistent — Ant reads uniform, not nested-round.',
            'Do use the dark #001529 menu for classic admin shells.',
            'Do size controls at 24/32/40 (small/middle/large) — three densities only.',
            'Do express status only via the four functional colors.',
        ],
        'donts': [
            "Don't customize radii per-component — the 4/6/8 scale is a contract.",
            "Don't put text below 45% opacity as primary — hierarchy is 88/65/45.",
            "Don't add gradients or brand colors outside the seed palette.",
            "Don't use #1677FF on dark surfaces without re-tinting — generate a dark algorithm palette.",
        ],
        'css': ''':root {
  --ant-primary: #1677FF;
  --ant-success: #52C41A;
  --ant-warning: #FAAD14;
  --ant-error: #FF4D4F;
  --ant-text: rgba(0,0,0,0.88);
  --ant-text-2: rgba(0,0,0,0.65);
  --ant-text-3: rgba(0,0,0,0.45);
  --ant-bg-layout: #F5F5F5;
  --ant-bg-container: #FFFFFF;
  --ant-border: #D9D9D9;
  --ant-border-2: #F0F0F0;
  --ant-radius: 6px;
  --ant-font: -apple-system, "Segoe UI", Roboto, "PingFang SC", sans-serif;
}''',
        'mock': dict(page='#f5f5f5', surface='#ffffff', sidebar='#001529', card='#ffffff',
                     border='#f0f0f0', text='#1a1a1a', muted='#8c8c8c', accent='#1677ff',
                     accent_text='#ffffff', accent_sub='#e6f4ff', radius=6),
        'mock_tag': 'ant.design · seed #1677FF',
    },
    {
        'id': 'salesforce-lightning',
        'title': 'Salesforce Lightning',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'lightning design — CRM chrome: cloud blue actions, card-on-mist layering, Salesforce Sans, and record-first layouts for sales operations.',
        'docs': 'https://www.lightningdesignsystem.com/',
        'overview': (
            "Salesforce Lightning (SLDS) is record-centric enterprise chrome: cloud-blue #0176D3 actions over "
            "a pale mist frame (#F3F3F3) with white record cards, Salesforce Sans at compact sizes, and "
            "conservative 4px radius. Layouts orbit the record detail — page headers with icon+title+actions, "
            "related-list cards, and activity feeds — so chrome stays predictable and data does the talking."
        ),
        'colors': [
            ('Brand', '#0176D3', 'brand-accessible / brand', 'Primary actions, links, brand marks.'),
            ('Brand Dark', '#014486', 'brand-dark', 'Hover/pressed and header accents.'),
            ('Text Link', '#005FB2', 'text-link', 'Accessible link blue on white.'),
            ('Cloud Background', '#F3F3F3', 'background', 'App frame behind record cards.'),
            ('Card Surface', '#FFFFFF', 'card-background', 'Record and widget cards.'),
            ('Text', '#181818', 'text-default', 'Primary text.'),
            ('Text Weak', '#444444', 'text-weak', 'Secondary text and field labels.'),
            ('Border', '#DDDBDA', 'border', 'Card and control hairlines.'),
            ('Success', '#2E844A', 'text-success', 'Positive states.'),
            ('Error', '#EA001E', 'text-error', 'Errors and destructive emphasis.'),
            ('Warning', '#FE9339', 'warning', 'Warnings — paired with dark text.'),
        ],
        'fonts': [('Salesforce Sans', 'house typeface — wide for headers, regular for UI body', '--font-sf-sans')],
        'type_rows': [
            ('body-small', 'Salesforce Sans', '400', '12px', '1.25', '0', '--text-body-sm'),
            ('body', 'Salesforce Sans', '400', '13px', '1.5', '0', '--text-body'),
            ('body-regular', 'Salesforce Sans', '400', '14px', '1.5', '0', '--text-body-lg'),
            ('heading', 'Salesforce Sans', '700', '14px', '1.25', '0', '--text-heading'),
            ('title', 'Salesforce Sans', '700', '18px', '1.25', '0', '--text-title'),
            ('page-title', 'Salesforce Sans', '300', '24px', '1.25', '0', '--text-page-title'),
        ],
        'base_unit': '4px',
        'density': 'compact',
        'spacing': [('xxx-small', '2px'), ('x-small', '4px'), ('small', '8px'), ('medium', '16px'), ('large', '24px'), ('x-large', '32px')],
        'radius': '4px medium default, 8px on cards — restrained, enterprise-conservative',
        'components': [
            'Button: 4px radius, brand fill, 32px height, 13px medium label.',
            'Page header: entity icon in brand tonal square + record title + action row.',
            'Card: white on mist background, 8px radius, header with title + actions right.',
            'Data table: bordered cells optional, row-level actions in overflow menus.',
            'Toast: bottom-center, brand/success/error tonal with icon square.',
        ],
        'dos': [
            'Do frame records in white cards on the mist background.',
            'Do use icon-in-square glyphs for entity headers — the SLDS signature.',
            'Do keep body at 13px — the system is density-tuned.',
            'Do use text-link #005FB2 for inline links (contrast over #0176D3).',
        ],
        'donts': [
            "Don't use brand blue for decorative fills — it signals action.",
            "Don't round beyond 8px — Lightning is conservative.",
            "Don't place content directly on the mist background.",
            "Don't invent status colors — the token set is closed.",
        ],
        'css': ''':root {
  --slds-brand: #0176D3;
  --slds-brand-dark: #014486;
  --slds-link: #005FB2;
  --slds-bg: #F3F3F3;
  --slds-card: #FFFFFF;
  --slds-text: #181818;
  --slds-text-2: #444444;
  --slds-border: #DDDBDA;
  --slds-success: #2E844A;
  --slds-error: #EA001E;
  --slds-radius: 4px;
  --slds-radius-card: 8px;
  --slds-font: "Salesforce Sans", system-ui, sans-serif;
}''',
        'mock': dict(page='#f3f3f3', surface='#ffffff', sidebar='#eef4ff', card='#ffffff',
                     border='#dddbda', text='#181818', muted='#706e6b', accent='#0176d3',
                     accent_text='#ffffff', accent_sub='#d8edff', radius=8),
        'mock_tag': 'lightningdesignsystem.com',
    },
    {
        'id': 'uswds',
        'title': 'US Web Design System',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'uswds — federal accessibility chrome: vivid primary blue, Public Sans, sharp corners, and 508-compliant contrast on every token pair.',
        'docs': 'https://designsystem.digital.gov/',
        'overview': (
            "USWDS styles US government sites: maximum-legibility Public Sans, a vivid primary blue ramp "
            "(#005EA8 core, #1A4480 darker) against pure white, sharp 0-2px corners, and gold-accent "
            "accessibility cues. Every token pair is graded for Section 508/WCAG contrast — the system "
            "prioritizes readability and keyboard wayfinding over decoration, with generous measure-limited "
            "prose columns for public-information reading."
        ),
        'colors': [
            ('Primary', '#005EA8', 'primary (blue-60v)', 'Primary actions and links — vivid accessible blue.'),
            ('Primary Darker', '#1A4480', 'primary-darker (blue-80v)', 'Hover and emphasis.'),
            ('Primary Darkest', '#162E51', 'primary-darkest (blue-90)', 'Headers and deepest emphasis.'),
            ('Accent Cool', '#00BDE3', 'accent-cool (cyan-30v)', 'Secondary highlights and info accents.'),
            ('Base Ink', '#1B1B1B', 'base-ink', 'Primary text — near-black.'),
            ('Base', '#5B616B', 'base-dark', 'Secondary text.'),
            ('Base Lighter', '#DFE1E2', 'base-lighter', 'Hairlines and disabled fills.'),
            ('Background', '#FFFFFF', 'white', 'Page and card surfaces.'),
            ('Error', '#B50909', 'error-dark', 'Form errors and destructive states.'),
            ('Success', '#00A91C', 'success', 'Positive confirmations.'),
            ('Warning', '#FFBE2E', 'warning', 'Warnings — paired with ink text.'),
            ('Info', '#00BDE3', 'info', 'Informational banners.'),
        ],
        'fonts': [('Public Sans', 'open federal typeface — strong x-height for screen legibility', '--font-public-sans'), ('Merriweather', 'optional serif for long-form reading headers', '--font-merriweather')],
        'type_rows': [
            ('body-xs', 'Public Sans', '400', '13px', '1.62', '0', '--text-body-xs'),
            ('body', 'Public Sans', '400', '17px', '1.65', '0', '--text-body'),
            ('body-lg', 'Public Sans', '400', '22px', '1.64', '0', '--text-body-lg'),
            ('h3', 'Public Sans', '700', '22px', '1.32', '0', '--text-h3'),
            ('h2', 'Merriweather', '700', '32px', '1.25', '0', '--text-h2'),
            ('h1', 'Merriweather', '700', '40px', '1.2', '0', '--text-h1'),
        ],
        'base_unit': '8px',
        'density': 'comfortable',
        'spacing': [('05', '4px'), ('1', '8px'), ('2', '16px'), ('3', '24px'), ('4', '32px'), ('5', '40px')],
        'radius': 'Sharp by default — 0 corners on buttons/inputs, 2px sm only where softened',
        'components': [
            'Button: sharp corners, vivid primary fill, 17px semibold — unmissable affordance.',
            'Banner: thin top strip "An official website of the United States government" — trust signal.',
            'Alert: left border 8px status color + tonal fill, icon + heading + body.',
            'Side navigation: indented tree with active page highlighted, hairline separators.',
            'Prose: max-measure 64ex columns — never full-bleed body text.',
        ],
        'dos': [
            'Do use the vivid primary ramp — every shade pair is contrast-graded.',
            'Do keep corners sharp — squared controls read as official.',
            'Do limit line length to ~64 characters for public-information prose.',
            'Do use Public Sans for UI and let Merriweather carry long-form headers.',
            'Do include skip-links and visible focus — 508 is the point of the system.',
        ],
        'donts': [
            "Don't mute the primary blue — accessibility depends on the vivid ramp.",
            "Don't add rounded or pill shapes — they read as non-governmental.",
            "Don't set body text below 16px equivalent — legibility is non-negotiable.",
            "Don't use color-only status — always pair icon + text.",
        ],
        'css': ''':root {
  --uswds-primary: #005EA8;
  --uswds-primary-darker: #1A4480;
  --uswds-ink: #1B1B1B;
  --uswds-base: #5B616B;
  --uswds-bg: #FFFFFF;
  --uswds-border: #DFE1E2;
  --uswds-error: #B50909;
  --uswds-success: #00A91C;
  --uswds-warning: #FFBE2E;
  --uswds-radius: 0;
  --uswds-font: "Public Sans", system-ui, sans-serif;
  --uswds-serif: "Merriweather", Georgia, serif;
}''',
        'mock': dict(page='#ffffff', surface='#ffffff', sidebar='#f0f0f0', card='#ffffff',
                     border='#dfe1e2', text='#1b1b1b', muted='#5b616b', accent='#005ea8',
                     accent_text='#ffffff', accent_sub='#dbe8f6', radius=0),
        'mock_tag': 'designsystem.digital.gov',
    },
    {
        'id': 'govuk-design',
        'title': 'GOV.UK Design System',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'gov.uk — public-service clarity: GDS blue, Transport type on plain white, square corners, and the famous yellow focus ring.',
        'docs': 'https://design-system.service.gov.uk/',
        'overview': (
            "GOV.UK is the world's most legible government system: no decoration, no hero images, no rounded "
            "corners — GDS Transport type at generous sizes, a single service blue (#1D70B8) for links and "
            "actions, and the unmistakable #FFDD00 yellow focus state. Everything is tuned for stressed, "
            "low-literacy, one-handed mobile users completing tasks quickly. If a component doesn't help "
            "someone finish a form, it doesn't exist."
        ),
        'colors': [
            ('GOV.UK Blue', '#1D70B8', 'govuk-colour("blue")', 'Links, primary buttons, brand bar.'),
            ('Link Hover', '#003078', 'govuk-link-hover-colour', 'Hover state for links.'),
            ('Link Visited', '#4C2C92', 'govuk-link-visited-colour', 'Visited links — purple.'),
            ('Text', '#0B0C0C', 'govuk-text-colour', 'Primary text — near-black.'),
            ('Secondary Text', '#505A5F', 'govuk-secondary-text-colour', 'Captions and metadata.'),
            ('Border', '#B1B4B6', 'govuk-border-colour', 'Input and panel outlines.'),
            ('Light Grey', '#F3F2F1', 'govuk-colour("light-grey")', 'Inset panels and subtle fills.'),
            ('Focus Yellow', '#FFDD00', 'govuk-focus-colour', 'The signature focus indicator.'),
            ('Error Red', '#D4351C', 'govuk-error-colour', 'Form errors — summary + field red border.'),
            ('Success Green', '#00703C', 'govuk-success-colour', 'Confirmation panels.'),
            ('Button Green', '#00703C', 'govuk-button bg', 'Primary submit buttons are green, not blue.'),
        ],
        'fonts': [('GDS Transport', 'proprietary — falls back to Arial/Helvetica for public services', '--font-gds')],
        'type_rows': [
            ('body-s', 'GDS Transport / Arial', '400', '16px', '1.25', '0', '--text-body-s'),
            ('body', 'GDS Transport / Arial', '400', '19px', '1.32', '0', '--text-body'),
            ('body-l', 'GDS Transport / Arial', '400', '24px', '1.29', '0', '--text-body-l'),
            ('h-s', 'GDS Transport / Arial', '700', '19px', '1.32', '0', '--text-h-s'),
            ('h-m', 'GDS Transport / Arial', '700', '24px', '1.29', '0', '--text-h-m'),
            ('h-l', 'GDS Transport / Arial', '700', '36px', '1.22', '0', '--text-h-l'),
            ('h-xl', 'GDS Transport / Arial', '700', '48px', '1.19', '0', '--text-h-xl'),
        ],
        'base_unit': '5px (spacing points)',
        'density': 'comfortable',
        'spacing': [('2', '10px'), ('3', '15px'), ('4', '20px'), ('6', '30px'), ('8', '40px'), ('9', '60px')],
        'radius': 'Zero — every element is square; rounded corners are explicitly against the system',
        'components': [
            'Button: green fill (#00703C), square, bold 19px, 2px bottom shadow — submits are green.',
            'Text input: 2px black border, square — thick enough to read as interactive.',
            'Error summary: red-bordered box at page top listing every error, links to fields.',
            'Focus: yellow background + 3px black underline on links, yellow halo on controls.',
            'Panel: light-grey inset with 30px padding for confirmations and key facts.',
        ],
        'dos': [
            'Do write in plain English at reading age 9 — content design IS the design system.',
            'Do keep one thing per page — split flows into small steps.',
            'Do use the yellow focus state exactly — it is the accessibility signature.',
            'Do keep buttons green and links blue — roles are fixed.',
            'Do use question-page pattern: heading = the question itself.',
        ],
        'donts': [
            "Don't round corners or add shadows — flat and square everywhere.",
            "Don't use icons without text labels — icons are optional, never required.",
            "Don't hide critical content in accordions — conditional reveals only for edge cases.",
            "Don't center or justify body text — left-aligned always.",
            "Don't use placeholder text as labels — real labels above inputs.",
        ],
        'css': ''':root {
  --govuk-blue: #1D70B8;
  --govuk-link-visited: #4C2C92;
  --govuk-text: #0B0C0C;
  --govuk-text-2: #505A5F;
  --govuk-border: #B1B4B6;
  --govuk-light-grey: #F3F2F1;
  --govuk-focus: #FFDD00;
  --govuk-error: #D4351C;
  --govuk-green: #00703C;   /* primary buttons */
  --govuk-radius: 0;
  --govuk-font: "GDS Transport", Arial, Helvetica, sans-serif;
}''',
        'mock': dict(page='#ffffff', surface='#ffffff', sidebar='#f3f2f1', card='#f3f2f1',
                     border='#b1b4b6', text='#0b0c0c', muted='#505a5f', accent='#00703c',
                     accent_text='#ffffff', accent_sub='#e5f2ec', radius=0),
        'mock_tag': 'design-system.service.gov.uk',
    },
    {
        'id': 'shadcn-ui',
        'title': 'shadcn/ui',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'shadcn/ui — neutral zinc minimalism: near-mono surfaces, inverted primary buttons, oklch token pipeline, and copy-paste component primitives.',
        'docs': 'https://ui.shadcn.com/',
        'overview': (
            "shadcn/ui is the React-era default aesthetic: an aggressively neutral zinc palette where surfaces "
            "are near-white and the primary button inverts to near-black — brand comes from your own accent "
            "layer, not the system. Tokens flow CSS-variable-first (now oklch) into Tailwind, radius sits at "
            "8px, and components are unstyled Radix primitives wearing the theme. It reads as modern SaaS "
            "baseline — intentionally blank enough to take any brand."
        ),
        'colors': [
            ('Background', '#FFFFFF', 'background', 'Page surface.'),
            ('Foreground', '#09090B', 'foreground', 'Primary text — zinc-950.'),
            ('Primary', '#18181B', 'primary', 'Primary button fill — inverted near-black.'),
            ('Primary FG', '#FAFAFA', 'primary-foreground', 'Text on primary fills.'),
            ('Secondary', '#F4F4F5', 'secondary', 'Subtle fills — secondary buttons, badges.'),
            ('Muted FG', '#71717A', 'muted-foreground', 'Captions and secondary text — zinc-500.'),
            ('Accent', '#F4F4F5', 'accent', 'Hover tint surfaces.'),
            ('Destructive', '#EF4444', 'destructive', 'Destructive actions — red-500.'),
            ('Border', '#E4E4E7', 'border', 'Hairlines — zinc-200.'),
            ('Ring', '#18181B', 'ring', 'Focus ring.'),
            ('Card', '#FFFFFF', 'card', 'Card surface — same as background in default theme.'),
            ('Dark Background', '#09090B', 'background (dark)', 'Dark mode surface.'),
        ],
        'fonts': [('Inter / Geist', 'theme-dependent — Inter classic, Geist for newer Vercel-flavored themes', '--font-sans')],
        'type_rows': [
            ('text-xs', 'sans', '400', '12px', '1.33', '0', '--text-xs'),
            ('text-sm', 'sans', '400', '14px', '1.43', '0', '--text-sm'),
            ('text-base', 'sans', '400', '16px', '1.5', '0', '--text-base'),
            ('h4', 'sans', '600', '24px', '1.33', '-0.025em', '--text-h4'),
            ('h2', 'sans', '600', '30px', '1.2', '-0.025em', '--text-h2'),
            ('h1', 'sans', '700', '36px', '1.2', '-0.025em', '--text-h1'),
        ],
        'base_unit': '4px',
        'density': 'compact',
        'spacing': [('1', '4px'), ('2', '8px'), ('3', '12px'), ('4', '16px'), ('6', '24px'), ('8', '32px')],
        'radius': '--radius 0.5rem (8px) default — lg 8px, md 6px, sm 4px derived',
        'components': [
            'Button: primary = near-black fill, secondary = zinc-100 fill, ghost = text-only hover.',
            'Card: border + white surface, 8px radius — shadows only on hover variants.',
            'Input: 1px zinc-200 border, 8px radius, ring on focus.',
            'Badge: secondary tonal pill — muted fills, never saturated.',
            'Dark mode: zinc-950 background with inverted primary — same token names flip values.',
        ],
        'dos': [
            'Do treat the theme as a neutral base — inject brand via accent/chart tokens.',
            'Do keep the inverted primary button — near-black on white is the signature.',
            'Do use muted-foreground for all secondary text — hierarchy is zinc-950 vs zinc-500.',
            'Do prefer tonal zinc fills over colored chips.',
            'Do write tokens as CSS vars so Tailwind + dark mode stay in sync.',
        ],
        'donts': [
            "Don't add a second brand color by default — the palette is deliberately neutral.",
            "Don't use colored borders — hairlines stay zinc-200.",
            "Don't rely on shadows for structure — borders and tonal fills do the work.",
            "Don't fork component markup — restyle via tokens so Radix behavior survives.",
        ],
        'css': ''':root {
  --background: #FFFFFF;         /* dark: #09090B */
  --foreground: #09090B;         /* dark: #FAFAFA */
  --primary: #18181B;            /* dark: #FAFAFA */
  --primary-foreground: #FAFAFA; /* dark: #18181B */
  --secondary: #F4F4F5;
  --muted-foreground: #71717A;
  --accent: #F4F4F5;
  --destructive: #EF4444;
  --border: #E4E4E7;
  --ring: #18181B;
  --radius: 8px;
  --font-sans: "Inter", system-ui, sans-serif;
}''',
        'mock': dict(page='#fafafa', surface='#ffffff', sidebar='#fafafa', card='#ffffff',
                     border='#e4e4e7', text='#09090b', muted='#71717a', accent='#18181b',
                     accent_text='#fafafa', accent_sub='#f4f4f5', radius=8),
        'mock_tag': 'ui.shadcn.com · default theme',
    },
    {
        'id': 'radix-themes',
        'title': 'Radix Themes',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'radix themes — 12-step color scales, semantic step-9 accents, Inter with optical sizing, and a scaling factor that sizes the whole UI at once.',
        'docs': 'https://www.radix-ui.com/themes',
        'overview': (
            "Radix Themes wraps the famous Radix color scales into a full design system: every hue ships as a "
            "12-step scale (steps 1-2 backgrounds, 3-5 interactive fills, 6-8 borders, 9-10 solids, 11-12 text), "
            "so a single accentColor prop generates the whole component vocabulary. Surfaces are clean grays, "
            "radius defaults to medium, and the scaling factor uniformly sizes padding, type and gaps — "
            "density becomes one dial."
        ),
        'colors': [
            ('Accent (indigo-9)', '#3E63DD', 'accent-9 / indigo-9', 'Default accent — interactive fills.'),
            ('Accent Track', '#FDDCFF', 'accent-3 (indigo-3)', 'Hover/selected subtle fills — indigo-3 ≈ #EDF2FE used in mock.'),
            ('Gray 1', '#FCFCFD', 'gray-1', 'App background — lightest scale step.'),
            ('Gray 3', '#F0F0F3', 'gray-3', 'Interactive element backgrounds.'),
            ('Gray 6', '#D9D9E0', 'gray-6', 'Interactive borders.'),
            ('Gray 7', '#CDCED6', 'gray-7', 'Stronger borders on hover.'),
            ('Gray 9', '#8B8D98', 'gray-9', 'Solid fills for neutral elements.'),
            ('Gray 11', '#60646C', 'gray-11', 'Secondary text.'),
            ('Gray 12', '#1F2023', 'gray-12', 'Primary text.'),
            ('Danger (red-9)', '#E5484D', 'red-9', 'Destructive fills.'),
            ('Success (green-9)', '#30A46C', 'green-9', 'Positive fills.'),
            ('Amber 9', '#FFC53D', 'amber-9', 'Warnings — dark text required.'),
        ],
        'fonts': [('Inter', 'default UI font — variable weight/optical sizing', '--font-inter')],
        'type_rows': [
            ('size-1', 'Inter', '400', '12px', '1.33', '0.0025em', '--text-1'),
            ('size-2', 'Inter', '400', '14px', '1.43', '0', '--text-2'),
            ('size-3', 'Inter', '400', '16px', '1.5', '0', '--text-3'),
            ('size-4', 'Inter', '500', '18px', '1.44', '-0.0025em', '--text-4'),
            ('size-6', 'Inter', '600', '24px', '1.33', '-0.006em', '--text-6'),
            ('size-8', 'Inter', '700', '35px', '1.26', '-0.012em', '--text-8'),
        ],
        'base_unit': '4px (scaling factor multiplies all)',
        'density': 'compact',
        'spacing': [('1', '4px'), ('2', '8px'), ('3', '12px'), ('4', '16px'), ('5', '24px'), ('6', '32px')],
        'radius': 'none / small ~3px / medium ~6px (default) / large ~12px / full — pick at Theme level',
        'components': [
            'Button: solid = accent-9 fill, soft = accent-3 + accent-11 text, ghost = transparent.',
            'Card: panel surface + subtle shadow or border variant.',
            'Text field: surface variant = soft gray-3 fill, outline = gray-7 border.',
            'Callout: tonal accent-2 fill + accent-11 text + icon slot.',
            'Theme props: accentColor, grayColor, radius, scaling — one prop retunes the whole tree.',
        ],
        'dos': [
            'Do think in the 12-step scale — pick the step for the role, not a hex.',
            'Do use accent-9 for solid fills and accent-11 for accent text — pairing is pre-graded.',
            'Do retune density via scaling factor instead of hand-tweaking paddings.',
            'Do pair soft/ghost button variants for secondary actions.',
        ],
        'donts': [
            "Don't mix arbitrary hexes with scale steps — contrast math assumes the scale.",
            "Don't put white text on accent-9 without checking the scale's intended pairing.",
            "Don't override component radius per-instance — set it once at Theme.",
            "Don't use gray-9 for text — gray-11/12 are the text steps.",
        ],
        'css': ''':root {
  --radix-accent-3: #EDF2FE;
  --radix-accent-9: #3E63DD;
  --radix-accent-11: #3A5CCC;
  --radix-gray-1: #FCFCFD;
  --radix-gray-3: #F0F0F3;
  --radix-gray-6: #D9D9E0;
  --radix-gray-7: #CDCED6;
  --radix-gray-9: #8B8D98;
  --radix-gray-11: #60646C;
  --radix-gray-12: #1F2023;
  --radix-red-9: #E5484D;
  --radix-green-9: #30A46C;
  --radix-radius: 6px;
  --radix-font: "Inter", system-ui, sans-serif;
}''',
        'mock': dict(page='#fcfcfd', surface='#ffffff', sidebar='#f7f7f9', card='#ffffff',
                     border='#d9d9e0', text='#1f2023', muted='#8b8d98', accent='#3e63dd',
                     accent_text='#ffffff', accent_sub='#edf2fe', radius=6),
        'mock_tag': 'radix-ui.com/themes · indigo-9',
    },
    {
        'id': 'tailwind-default',
        'title': 'Tailwind CSS Default',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'tailwind default theme — utility-first palette scales (50-950), slate neutrals, 4px spacing scale, and utility-composed surfaces.',
        'docs': 'https://tailwindcss.com/docs/theme',
        'overview': (
            "Tailwind's default theme isn't a design system so much as a design algebra: 11-step color scales "
            "(50→950) per hue, slate as the de-facto neutral, a 4px-based spacing scale, and composable radius/"
            "shadow/typography utilities. The 'look' it produces is predictable — blue-500/600 accents, slate "
            "text hierarchy, white cards on gray-50 pages, rounded-lg defaults — which is why so much of the "
            "modern web reads as 'Tailwind' at a glance."
        ),
        'colors': [
            ('Blue 500', '#3B82F6', 'blue-500', 'Default interactive accent.'),
            ('Blue 600', '#2563EB', 'blue-600', 'Hover/emphasis on blue-500.'),
            ('Indigo 500', '#6366F1', 'indigo-500', 'Common alt accent.'),
            ('Slate 50', '#F8FAFC', 'slate-50', 'Page background tint.'),
            ('Slate 100', '#F1F5F9', 'slate-100', 'Subtle fills and wells.'),
            ('Slate 200', '#E2E8F0', 'slate-200', 'Hairline borders.'),
            ('Slate 500', '#64748B', 'slate-500', 'Muted/secondary text.'),
            ('Slate 700', '#334155', 'slate-700', 'Strong body text.'),
            ('Slate 900', '#0F172A', 'slate-900', 'Headings and near-black text.'),
            ('Red 500', '#EF4444', 'red-500', 'Destructive and errors.'),
            ('Green 500', '#22C55E', 'green-500', 'Success.'),
            ('Amber 500', '#F59E0B', 'amber-500', 'Warnings.'),
        ],
        'fonts': [('ui-sans-serif stack', 'system-ui, sans-serif — teams typically substitute Inter', '--font-sans'), ('ui-monospace stack', 'SFMono, Menlo, monospace for code', '--font-mono')],
        'type_rows': [
            ('xs', 'sans', '400', '12px', '1.33', '0', '--text-xs'),
            ('sm', 'sans', '400', '14px', '1.43', '0', '--text-sm'),
            ('base', 'sans', '400', '16px', '1.5', '0', '--text-base'),
            ('lg', 'sans', '500', '18px', '1.56', '0', '--text-lg'),
            ('xl', 'sans', '600', '20px', '1.4', '-0.025em?', '--text-xl'),
            ('3xl', 'sans', '700', '30px', '1.2', '-0.025em', '--text-3xl'),
        ],
        'base_unit': '4px (spacing 0.25rem base)',
        'density': 'comfortable',
        'spacing': [('1', '4px'), ('2', '8px'), ('4', '16px'), ('6', '24px'), ('8', '32px'), ('12', '48px')],
        'radius': 'rounded .25rem, rounded-md .375rem, rounded-lg .5rem, rounded-xl .75rem — lg is the web default',
        'components': [
            'Button: bg-blue-600 + white text + rounded-md + px-4 py-2 — the canonical utility combo.',
            'Card: bg-white + rounded-lg + shadow-sm or ring-1 ring-slate-200 + p-6.',
            'Input: border-slate-300 + rounded-md + focus:ring-2 focus:ring-blue-500.',
            'Badge: bg-{hue}-100 + text-{hue}-800 + rounded-full + text-xs.',
            'Page: bg-slate-50 body + white surface cards — the default admin recipe.',
        ],
        'dos': [
            'Do stay on the scale — slate for neutrals, one accent hue, status hues for state.',
            'Do use space scale consistently — gap/padding from the same 4px grid.',
            'Do extract repeated combos into components — utilities are for composition, not duplication.',
            'Do pair text-slate-900 headings with slate-500/600 body for instant hierarchy.',
        ],
        'donts': [
            "Don't use arbitrary values when a scale step exists — consistency is the point.",
            "Don't mix slate/gray/zinc neutrals in one surface — pick one neutral family.",
            "Don't nest shadow on shadow — one elevation cue per surface.",
            "Don't reach for custom CSS before exhausting utilities.",
        ],
        'css': ''':root {
  --tw-blue-500: #3B82F6;
  --tw-blue-600: #2563EB;
  --tw-slate-50: #F8FAFC;
  --tw-slate-100: #F1F5F9;
  --tw-slate-200: #E2E8F0;
  --tw-slate-500: #64748B;
  --tw-slate-900: #0F172A;
  --tw-red-500: #EF4444;
  --tw-green-500: #22C55E;
  --tw-radius: 8px;           /* rounded-lg */
  --tw-font: ui-sans-serif, system-ui, sans-serif;
}''',
        'mock': dict(page='#f8fafc', surface='#ffffff', sidebar='#f1f5f9', card='#ffffff',
                     border='#e2e8f0', text='#0f172a', muted='#64748b', accent='#3b82f6',
                     accent_text='#ffffff', accent_sub='#dbeafe', radius=8),
        'mock_tag': 'tailwindcss.com · default theme',
    },
    {
        'id': 'bootstrap-5',
        'title': 'Bootstrap 5',
        'platform': 'web',
        'frame': 'window',
        'theme': 'light',
        'tagline': 'bootstrap 5 — the web\'s utility baseline: electric primary blue, system font stack, .375rem radius, and the component grammar a decade of sites still speaks.',
        'docs': 'https://getbootstrap.com/docs/',
        'overview': (
            "Bootstrap 5 remains the most-deployed UI vocabulary on the web: electric #0D6EFD primary, "
            "eight semantic theme colors, a system font stack, 6px (.375rem) default radius, and "
            "utility + component classes that standardize cards, navs, modals and forms. Its look is the "
            "'default web' — instantly familiar, deliberately unbranded, and easy to re-skin through "
            "Sass variables or CSS custom properties."
        ),
        'colors': [
            ('Primary', '#0D6EFD', 'bs-primary', 'Primary actions and links.'),
            ('Secondary', '#6C757D', 'bs-secondary', 'Secondary buttons and muted accents.'),
            ('Success', '#198754', 'bs-success', 'Positive states.'),
            ('Danger', '#DC3545', 'bs-danger', 'Errors and destructive actions.'),
            ('Warning', '#FFC107', 'bs-warning', 'Warnings — requires dark text.'),
            ('Info', '#0DCAF0', 'bs-info', 'Informational accents.'),
            ('Light', '#F8F9FA', 'bs-light', 'Subtle fills and light surfaces.'),
            ('Dark', '#212529', 'bs-dark', 'Dark surfaces and inverse text.'),
            ('Body Text', '#212529', 'bs-body-color', 'Primary reading text.'),
            ('Body BG', '#FFFFFF', 'bs-body-bg', 'Page background.'),
            ('Border', '#DEE2E6', 'bs-border-color', 'Default borders.'),
            ('Link', '#0D6EFD', 'bs-link-color', 'Links share primary.'),
        ],
        'fonts': [('System stack', 'system-ui, -apple-system, Segoe UI, Roboto, Helvetica Neue, Arial', '--bs-font-sans-serif')],
        'type_rows': [
            ('small', 'system', '400', '12.8px', '1.5', '0', '--text-small'),
            ('body', 'system', '400', '16px', '1.5', '0', '--text-body'),
            ('lead', 'system', '300', '20px', '1.5', '0', '--text-lead'),
            ('h6', 'system', '500', '16px', '1.2', '0', '--text-h6'),
            ('h3', 'system', '500', '28px', '1.2', '0', '--text-h3'),
            ('h1', 'system', '500', '40px', '1.2', '0', '--text-h1'),
        ],
        'base_unit': '4px (spacer 1rem = 16px at base)',
        'density': 'comfortable',
        'spacing': [('1', '4px'), ('2', '8px'), ('3', '16px'), ('4', '24px'), ('5', '48px')],
        'radius': '.375rem (6px) default, sm .25rem, lg .5rem, pill 50rem — conservative web default',
        'components': [
            'Button: btn-primary = #0D6EFD fill, 6px radius, 400 16px label.',
            'Card: border + white surface + 6px radius, header/body/footer sections.',
            'Navbar: light navbar-expand with container, brand left, collapse right.',
            'Form control: border #DEE2E6, 6px radius, focus = blue ring shadow.',
            'Alert: tonal background-10% fills + matching darker text, dismissible.',
        ],
        'dos': [
            'Do use the eight semantic color roles as the whole palette.',
            'Do compose with utilities (d-flex, gap-3, mb-4) inside components.',
            'Do re-skin via Sass $primary/CSS vars — never fork markup.',
            'Do keep 6px radius for the classic look; pill only for deliberate accents.',
        ],
        'donts': [
            "Don't leave default blue on a 'custom' brand — re-tokenize first.",
            "Don't override component CSS inline — change the variables.",
            "Don't nest cards in cards — spacing utilities do layout instead.",
            "Don't ignore color-mode() — dark mode is first-class since 5.3.",
        ],
        'css': ''':root {
  --bs-primary: #0D6EFD;
  --bs-secondary: #6C757D;
  --bs-success: #198754;
  --bs-danger: #DC3545;
  --bs-warning: #FFC107;
  --bs-info: #0DCAF0;
  --bs-light: #F8F9FA;
  --bs-dark: #212529;
  --bs-body-color: #212529;
  --bs-body-bg: #FFFFFF;
  --bs-border-color: #DEE2E6;
  --bs-border-radius: 6px;
  --bs-font-sans-serif: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
}''',
        'mock': dict(page='#ffffff', surface='#ffffff', sidebar='#f8f9fa', card='#ffffff',
                     border='#dee2e6', text='#212529', muted='#6c757d', accent='#0d6efd',
                     accent_text='#ffffff', accent_sub='#cfe2ff', radius=6),
        'mock_tag': 'getbootstrap.com · v5.3',
    },
]


# ---------------------------------------------------------------------------
# DESIGN.md writer
# ---------------------------------------------------------------------------

def design_md(s: dict) -> str:
    lines = [
        f"# {s['title']} — Style Reference",
        f"> {s['tagline']}",
        "",
        f"**Theme:** {s['theme']}",
        f"**Platform:** {s['platform']}",
        "",
        f"Official vendor design system — tokens reproduced from the published specification at [{s['docs']}]({s['docs']}).",
        "",
        s['overview'],
        "",
        "## Tokens — Colors",
        "",
        "| Name | Value | Token | Role |",
        "|------|-------|-------|------|",
    ]
    for name, val, tok, role in s['colors']:
        lines.append(f"| {name} | `{val}` | `{tok}` | {role} |")
    lines += ["", "## Tokens — Typography", ""]
    for fam, desc, tok in s['fonts']:
        lines.append(f"### {fam} — {desc} · `{tok}`")
        lines.append("")
    lines += [
        "### Type Scale",
        "",
        "| Role | Family | Weight | Size | Line Height | Letter Spacing | Token |",
        "|------|--------|--------|------|-------------|----------------|-------|",
    ]
    for role, fam, wt, size, lh, tr, tok in s['type_rows']:
        lines.append(f"| {role} | {fam} | {wt} | {size} | {lh} | {tr} | `{tok}` |")
    lines += [
        "",
        "## Tokens — Spacing & Shapes",
        "",
        f"**Base unit:** {s['base_unit']}",
        f"**Density:** {s['density']}",
        "",
        "### Spacing Scale",
        "",
        "| Name | Value | Token |",
        "|------|-------|-------|",
    ]
    for name, val in s['spacing']:
        lines.append(f"| {name} | {val} | `--spacing-{name}` |")
    lines += [
        "",
        f"**Radius:** {s['radius']}",
        "",
        "## Component Contracts",
        "",
    ]
    lines += [f"- {c}" for c in s['components']]
    lines += ["", "## Do", ""]
    lines += [f"- {d}" for d in s['dos']]
    lines += ["", "## Don't", ""]
    lines += [f"- {d}" for d in s['donts']]
    lines += [
        "",
        "## Apply — CSS Variables",
        "",
        "```css",
        s['css'],
        "```",
        "",
    ]
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Preview SVG mockups — phone frame for mobile, app window for web
# ---------------------------------------------------------------------------

def phone_mockup(t: dict, title: str, tag: str, platform: str) -> str:
    # status bar, nav bar, grouped list cells, tab bar — inside a phone frame
    cells = ''
    for i in range(5):
        y = 118 + i * 44
        cells += f'''<rect x="48" y="{y}" width="304" height="40" rx="10" fill="{t["card"]}"/>
<circle cx="66" cy="{y+20}" r="10" fill="{t["accent_sub"]}"/>
<rect x="84" y="{y+12}" width="{120 - i*10}" height="7" rx="3.5" fill="{t["text"]}" opacity="0.8"/>
<rect x="84" y="{y+24}" width="{80 - i*8}" height="5" rx="2.5" fill="{t["muted"]}" opacity="0.6"/>
<rect x="328" y="{y+15}" width="12" height="10" rx="2" fill="{t["muted"]}" opacity="0.5"/>
'''
    tabs = ''
    for i in range(4):
        x = 84 + i * 62
        fill = t['accent'] if i == 0 else t['muted']
        op = '1' if i == 0 else '0.45'
        tabs += f'<circle cx="{x}" cy="334" r="9" fill="{fill}" opacity="{op}"/>\n<rect x="{x-14}" y="348" width="28" height="4" rx="2" fill="{fill}" opacity="{op}"/>\n'
    fab = ''
    if platform == 'android':
        fab = f'''<rect x="296" y="262" width="52" height="52" rx="16" fill="{t["accent_sub"]}"/>
<rect x="320" y="278" width="4" height="20" rx="2" fill="{t["accent"]}"/>
<rect x="312" y="286" width="20" height="4" rx="2" fill="{t["accent"]}"/>
'''
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 400" width="640" height="400">
<rect width="640" height="400" fill="{t["page"]}"/>
<rect x="140" y="16" width="360" height="368" rx="28" fill="{t["border"]}" opacity="0.55"/>
<rect x="146" y="22" width="348" height="356" rx="24" fill="{t["page"]}"/>
<rect x="250" y="30" width="140" height="14" rx="7" fill="{t["border"]}" opacity="0.7"/>
<rect x="170" y="56" width="60" height="8" rx="4" fill="{t["text"]}" opacity="0.8"/>
<rect x="410" y="56" width="60" height="8" rx="4" fill="{t["muted"]}" opacity="0.6"/>
<rect x="146" y="72" width="348" height="34" fill="{t["bar"]}"/>
<rect x="170" y="84" width="110" height="11" rx="5.5" fill="{t["text"]}" opacity="0.85"/>
<rect x="452" y="82" width="18" height="14" rx="3" fill="{t["accent"]}"/>
{cells}
{fab}
<rect x="146" y="318" width="348" height="60" rx="0" fill="{t["bar"]}"/>
{tabs}
<rect x="260" y="370" width="120" height="4" rx="2" fill="{t["muted"]}" opacity="0.6"/>
<text x="48" y="390" fill="{t["muted"]}" font-family="system-ui, sans-serif" font-size="11">{tag}</text>
<text x="592" y="390" fill="{t["text"]}" font-family="system-ui, sans-serif" font-size="12" font-weight="700" text-anchor="end">{title}</text>
</svg>'''


def window_mockup(t: dict, title: str, tag: str) -> str:
    nav = ''
    for i in range(4):
        y = 78 + i * 30
        if i == 0:
            nav += f'<rect x="36" y="{y}" width="128" height="22" rx="6" fill="{t["accent_sub"]}"/>\n'
            nav += f'<rect x="42" y="{y+7}" width="60" height="8" rx="4" fill="{t["accent"]}"/>\n'
        else:
            nav += f'<rect x="42" y="{y+7}" width="{88 - i*12}" height="8" rx="4" fill="{t["muted"]}" opacity="0.55"/>\n'
    cards = ''
    for i in range(3):
        x = 196 + i * 140
        cards += f'''<rect x="{x}" y="84" width="126" height="62" rx="{t["radius"]}" fill="{t["card"]}" stroke="{t["border"]}" stroke-width="1"/>
<rect x="{x+12}" y="97" width="52" height="7" rx="3.5" fill="{t["muted"]}" opacity="0.7"/>
<rect x="{x+12}" y="112" width="{64 - i*8}" height="14" rx="4" fill="{t["text"]}" opacity="0.9"/>
'''
    rows = ''
    for i in range(4):
        y = 208 + i * 30
        rows += f'''<circle cx="212" cy="{y+8}" r="9" fill="{t["accent_sub"]}"/>
<rect x="230" y="{y+3}" width="{200 - i*30}" height="8" rx="4" fill="{t["text"]}" opacity="0.75"/>
<rect x="230" y="{y+15}" width="{130 - i*15}" height="6" rx="3" fill="{t["muted"]}" opacity="0.5"/>
<rect x="540" y="{y+3}" width="44" height="14" rx="7" fill="{t["accent_sub"]}"/>
'''
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 400" width="640" height="400">
<defs><clipPath id="win"><rect x="24" y="24" width="592" height="352" rx="14"/></clipPath></defs>
<rect width="640" height="400" fill="{t["page"]}"/>
<g clip-path="url(#win)">
<rect x="24" y="24" width="592" height="352" fill="{t["surface"]}"/>
<rect x="24" y="24" width="152" height="352" fill="{t["sidebar"]}"/>
<rect x="175.5" y="24" width="1" height="352" fill="{t["border"]}" opacity="0.6"/>
<circle cx="46" cy="48" r="7" fill="{t["accent"]}"/>
<rect x="62" y="42" width="78" height="11" rx="5.5" fill="{t["text"]}" opacity="0.85"/>
{nav}
<rect x="196" y="40" width="150" height="13" rx="6" fill="{t["text"]}" opacity="0.9"/>
<rect x="506" y="36" width="90" height="20" rx="10" fill="{t["card"]}" stroke="{t["border"]}"/>
<rect x="24" y="63.5" width="592" height="1" fill="{t["border"]}" opacity="0.6"/>
{cards}
<rect x="196" y="162" width="400" height="196" rx="{t["radius"]}" fill="{t["card"]}" stroke="{t["border"]}"/>
<rect x="196" y="162" width="400" height="34" fill="{t["accent_sub"]}" opacity="0.45"/>
<rect x="212" y="174" width="120" height="10" rx="5" fill="{t["text"]}" opacity="0.8"/>
<rect x="496" y="168" width="88" height="22" rx="{t["radius"]}" fill="{t["accent"]}"/>
<rect x="516" y="175" width="48" height="8" rx="4" fill="{t["accent_text"]}"/>
{rows}
</g>
<rect x="24" y="24" width="592" height="352" rx="14" fill="none" stroke="{t["border"]}" stroke-width="1.5"/>
<rect x="24" y="332" width="592" height="44" fill="{t["surface"]}" opacity="0.92" clip-path="url(#win)"/>
<text x="48" y="360" fill="{t["text"]}" font-family="system-ui, sans-serif" font-size="15" font-weight="700">{title}</text>
<text x="596" y="360" fill="{t["muted"]}" font-family="system-ui, sans-serif" font-size="12" text-anchor="end">{tag}</text>
</svg>'''


# ---------------------------------------------------------------------------

def main() -> None:
    index = json.loads(INDEX.read_text('utf-8')) if INDEX.exists() else []
    have = {e['id'] for e in index}
    created = 0
    for s in SYSTEMS:
        d = STYLES / s['id']
        d.mkdir(parents=True, exist_ok=True)
        (d / 'DESIGN.md').write_text(design_md(s), 'utf-8', newline='\n')
        svg = (phone_mockup(s['mock'], s['title'], s['mock_tag'], s['platform'])
               if s['frame'] == 'phone'
               else window_mockup(s['mock'], s['title'], s['mock_tag']))
        (d / 'preview.svg').write_text(svg, 'utf-8')
        meta = {
            'id': s['id'],
            'url': s['docs'],
            'title': s['title'],
            'tagline': s['tagline'],
            'theme': s['theme'],
            'source_url': s['docs'],
            'platform': s['platform'],
            'origin': 'official-design-system',
            'preview': 'preview.svg',
            'shots': ['preview.svg'],
        }
        (d / 'meta.json').write_text(json.dumps(meta, ensure_ascii=False, indent=2), 'utf-8')
        if s['id'] not in have:
            index.append({k: meta[k] for k in ('id', 'url', 'title', 'tagline', 'theme', 'source_url')})
        else:
            for e in index:
                if e['id'] == s['id']:
                    e.update({k: meta[k] for k in ('url', 'title', 'tagline', 'theme', 'source_url')})
        created += 1
        print('ok', s['id'])
    INDEX.write_text(json.dumps(index, ensure_ascii=False, indent=2), 'utf-8', newline='\n')
    print(f"done: {created} platform styles, index total {len(index)}")


if __name__ == '__main__':
    main()
