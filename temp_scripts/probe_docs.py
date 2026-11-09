import asyncio, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
async def main():
    from playwright.async_api import async_playwright
    async with async_playwright() as pw:
        b = await pw.chromium.launch()
        pg = await b.new_page(viewport={"width":1200,"height":800})
        await pg.goto("https://ui.knowns.dev/docs/button", wait_until="networkidle", timeout=45000)
        info = await pg.evaluate("""() => {
          const out = [];
          // candidates: elements with data-docs-place, or first large section with buttons rendered
          document.querySelectorAll('[data-docs-place]').forEach(e=>out.push(['docs-place:'+e.dataset.docsPlace, e.getBoundingClientRect().toJSON()]));
          // find elements containing a real <button>
          document.querySelectorAll('button').forEach((e,i)=>{ if(i<12){const r=e.getBoundingClientRect(); out.push(['button:'+(e.textContent||'').trim().slice(0,30), {x:r.x,y:r.y,w:r.width,h:r.height}]);}});
          return out;
        }""")
        for t in info: print(t)
        await pg.screenshot(path="docs-button-full.png", full_page=False)
        await b.close()
asyncio.run(main())
