import { chromium } from 'playwright-core';
const b = await chromium.launch();
const pg = await b.newPage({ viewport: { width: 1200, height: 800 } });
await pg.goto('https://ui.knowns.dev/docs/button', { waitUntil: 'networkidle', timeout: 45000 });
const info = await pg.evaluate(() => {
  const tabs = [...document.querySelectorAll('[role="tab"], button')].filter(e => /^(Preview|Code)$/.test((e.textContent || '').trim()));
  const t = tabs.find(e => (e.textContent || '').trim() === 'Preview');
  if (!t) return { err: 'no Preview tab' };
  let card = t; for (let i = 0; i < 6 && card.parentElement; i++) card = card.parentElement;
  const r = card.getBoundingClientRect();
  const panels = document.querySelectorAll('[role="tabpanel"]').length;
  return { cardTag: card.tagName, cardClass: (card.className || '').toString().slice(0, 160), rect: r.toJSON(), panels };
});
console.log(JSON.stringify(info, null, 1));
await b.close();
