import { chromium } from 'playwright-core';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

const ROOT = '//10.21.2.2/data/_agentsync/repos/lnc-components';
const idx = JSON.parse(readFileSync(join(ROOT, 'index.json'), 'utf-8'));
const items = idx.items || idx;
console.log('items:', items.length);

const b = await chromium.launch();
const ctx = await b.newContext({ viewport: { width: 1200, height: 675 }, deviceScaleFactor: 1 });
const pg = await ctx.newPage();

let ok = 0, skip = 0, fail = [];
for (const it of items) {
  const dir = join(ROOT, 'items', it.id);
  const out = join(dir, 'preview.png');
  if (existsSync(out) || existsSync(join(dir, 'preview.webp'))) { skip++; continue; }
  try {
    if (it.id.startsWith('knowns-')) {
      const url = it.source_url || it.meta?.source_url;
      if (!url) { fail.push(it.id + ':no-url'); continue; }
      await pg.goto(url, { waitUntil: 'networkidle', timeout: 40000 });
      const handle = await pg.evaluateHandle(() => {
        for (const e of document.querySelectorAll('[role="tabpanel"]')) {
          const r = e.getBoundingClientRect();
          const cs = getComputedStyle(e);
          const txt = (e.textContent || '');
          if (!e.hidden && cs.display !== 'none' && r.height > 120 && r.width > 300
              && !/shadcn@latest|coding agent|npx /.test(txt)) return e;
        }
        return null;
      });
      const el = handle.asElement();
      if (!el) { fail.push(it.id + ':no-panel'); continue; }
      await el.scrollIntoViewIfNeeded();
      await pg.waitForTimeout(1200);
      await el.screenshot({ path: out });
    } else {
      const file = join(dir, 'index.html').replace(/\\/g, '/');
      await pg.goto('file:///' + file, { waitUntil: 'networkidle', timeout: 20000 });
      await pg.waitForTimeout(1600);
      await pg.screenshot({ path: out });
    }
    ok++; if (ok % 20 === 0) console.log('...', ok);
  } catch (e) { fail.push(it.id + ':' + String(e).slice(0, 60)); }
}
console.log('done ok:', ok, 'skip:', skip, 'fail:', fail.length);
if (fail.length) console.log(fail.join('\n'));
await b.close();
