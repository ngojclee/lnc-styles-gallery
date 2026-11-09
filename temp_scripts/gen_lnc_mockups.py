"""Regenerate LNC style preview.svg files as mini app-window mockups drawn from real theme tokens."""
import json
from pathlib import Path

STYLES = Path(r'//10.21.2.2/data/_agentsync/repos/refero-styles/styles')

T = {
    'lnc-app-shell': dict(page='#fafafa', surface='#ffffff', sidebar='#f9fafb', card='#ffffff',
                          border='#e5e7eb', text='#18181b', muted='#a1a1aa', accent='#18181b',
                          accent_text='#ffffff', accent_sub='#f4f4f5', radius=10),
    'lnc-blueprint-lite': dict(page='#080a10', surface='#12141d', sidebar='#0b0d14', card='#171a26',
                               border='#252c40', text='#e9edf5', muted='#8b93a7', accent='#c99bd8',
                               accent_text='#241026', accent_sub='#241b2b', radius=12),
    'lnc-blueprint-full': dict(page='#fbfbfa', surface='#ffffff', sidebar='#f7f7f5', card='#ffffff',
                               border='#e3e3e0', text='#37352f', muted='#9b9a97', accent='#2383e2',
                               accent_text='#ffffff', accent_sub='#e8f0fe', radius=6),
    'ln-ui-engine': dict(page='#111111', surface='#252525', sidebar='#1e1e1e', card='#252525',
                         border='#333333', text='#e3e3e3', muted='#6b6b6b', accent='#529cca',
                         accent_text='#10202e', accent_sub='#1a3a52', radius=6),
}

TAGS = {
    'lnc-app-shell': 'pose.lengoc.me shell',
    'lnc-blueprint-lite': 'ops console · dark-first',
    'lnc-blueprint-full': 'Engine theme.css · light',
    'ln-ui-engine': '@luxeclaw/ui-engine · dark',
}

TITLES = {
    'lnc-app-shell': 'LNC App Shell',
    'lnc-blueprint-lite': 'LNC Blueprint Lite',
    'lnc-blueprint-full': 'LNC Blueprint Full',
    'ln-ui-engine': 'LN UI Engine',
}


def mockup(sid: str) -> str:
    t = T[sid]
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
<text x="48" y="360" fill="{t["text"]}" font-family="system-ui, sans-serif" font-size="15" font-weight="700">{TITLES[sid]}</text>
<text x="596" y="360" fill="{t["muted"]}" font-family="system-ui, sans-serif" font-size="12" text-anchor="end">{TAGS[sid]}</text>
</svg>'''


for sid in T:
    p = STYLES / sid / 'preview.svg'
    p.write_text(mockup(sid), encoding='utf-8')
    meta_file = STYLES / sid / 'meta.json'
    meta = json.loads(meta_file.read_text(encoding='utf-8'))
    meta['preview'] = 'preview.svg'
    meta['shots'] = ['preview.svg']
    meta_file.write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding='utf-8')
    print('mockup ->', sid, p.stat().st_size, 'B')
