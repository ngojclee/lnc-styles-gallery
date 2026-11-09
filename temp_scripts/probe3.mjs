import { chromium } from 'playwright-core';
const b = await chromium.launch();
const pg = await b.newPage({ viewport: { width: 1200, height: 800 } });
await pg.goto('https://ui.knowns.dev/docs/button', { waitUntil: 'networkidle', timeout: 45000 });
const info = await pg.evaluate(() => {
  return [...document.querySelectorAll('[role="tabpanel"]')].map((e, i) => {
    const r = e.getBoundingClientRect();
    const style = getComputedStyle(e);
    return { i, rect: { x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height) },
             hidden: e.hidden || style.display === 'none' || r.height === 0,
             btns: e.querySelectorAll('button').length, text: (e.textContent || '').trim().slice(0, 60) };
  });
});
console.log(JSON.stringify(info, null, 1));
await b.close();
