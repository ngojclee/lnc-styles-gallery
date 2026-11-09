import { chromium } from 'playwright-core';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

const ROOT = '//10.21.2.2/data/_agentsync/repos/lnc-components';
const items = JSON.parse(readFileSync(join(ROOT, 'index.json'), 'utf-8'));
const targets = ['knowns-accordion', 'knowns-dialog', 'knowns-popover'];
const m = Object.fromEntries(items.map(i => [i.id, i]));

const b = await chromium.launch();
const pg = await (await b.newContext({ viewport: { width: 1200, height: 675 } })).newPage();

for (const id of targets) {
  const dir = join(ROOT, 'items', id);
  if (existsSync(join(dir, 'preview.png'))) continue;
  try {
    await pg.goto(m[id].source_url, { waitUntil: 'domcontentloaded', timeout: 40000 });
    await pg.waitForTimeout(3500);
    // find first visible tabpanel (demo area), relax to >60px height
    const el = await pg.evaluateHandle(() => {
      const panels = [...document.querySelectorAll('[role="tabpanel"]')].filter(e => {
        const r = e.getBoundingClientRect();
        const txt = e.textContent || '';
        return !e.hidden && r.height > 60 && r.width > 300 && !/shadcn@latest|coding agent|npx /.test(txt);
      });
      return panels[0] || null;
    }).then(h => h.asElement());
    if (!el) { console.log(id, 'STILL no-panel'); continue; }
    // click a trigger button inside to open overlay components (dialog/popover)
    const btn = await el.$('button');
    if (btn) { await btn.click().catch(() => {}); await pg.waitForTimeout(900); }
    // if an overlay opened, shoot it; otherwise shoot the panel
    const overlay = await pg.evaluateHandle(() => {
      for (const sel of ['[role="dialog"]', '[data-radix-popper-content-wrapper]', '[role="menu"]', '[role="alertdialog"]']) {
        for (const e of document.querySelectorAll(sel)) {
          const r = e.getBoundingClientRect();
          if (r.width > 100 && r.height > 50) return e;
        }
      }
      return null;
    }).then(h => h.asElement());
    const target = overlay || el;
    await target.screenshot({ path: join(dir, 'preview.png') });
    console.log(id, 'OK', overlay ? '(overlay)' : '(panel)');
  } catch (e) { console.log(id, 'FAIL', String(e).slice(0, 80)); }
}
await b.close();
