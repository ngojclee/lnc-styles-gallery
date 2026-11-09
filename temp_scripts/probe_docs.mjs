import { chromium } from 'playwright-core';
const b = await chromium.launch();
const pg = await b.newPage({ viewport: { width: 1200, height: 800 } });
await pg.goto('https://ui.knowns.dev/docs/button', { waitUntil: 'networkidle', timeout: 45000 });
const info = await pg.evaluate(() => {
  const out = [];
  document.querySelectorAll('[data-docs-place]').forEach(e => out.push(['docs-place:' + e.dataset.docsPlace, e.getBoundingClientRect().toJSON()]));
  document.querySelectorAll('button').forEach((e, i) => { if (i < 12) { const r = e.getBoundingClientRect(); out.push(['btn:' + (e.textContent || '').trim().slice(0, 28), { x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height) }]); } });
  return out;
});
info.forEach(t => console.log(JSON.stringify(t)));
await pg.screenshot({ path: 'docs-button-full.png' });
await b.close();
