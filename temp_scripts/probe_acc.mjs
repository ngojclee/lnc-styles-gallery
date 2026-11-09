import { chromium } from 'playwright-core';
const b = await chromium.launch();
const pg = await (await b.newContext({ viewport: { width: 1200, height: 800 } })).newPage();
await pg.goto('https://ui.knowns.dev/docs/accordion', { waitUntil: 'domcontentloaded', timeout: 40000 });
await pg.waitForTimeout(3500);
const info = await pg.evaluate(() => {
  const ps = [...document.querySelectorAll('[role="tabpanel"]')].map(e => { const r = e.getBoundingClientRect(); return { h: Math.round(r.height), w: Math.round(r.width), hidden: e.hidden, txt: (e.textContent||'').slice(0,50) }; });
  const details = document.querySelectorAll('details, [data-state]').length;
  return { panels: ps, details };
});
console.log(JSON.stringify(info, null, 1));
await b.close();
