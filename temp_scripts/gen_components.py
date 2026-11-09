#!/usr/bin/env python3
"""Generate the lnc-components corpus: self-contained HTML effect/component demos.

Each component = items/<id>/index.html (single-file demo, no external deps)
plus a record in index.json. Written by LNC — license-clean.
"""
import json
import re
from pathlib import Path

OUT = Path(r"//10.21.2.2/data/_agentsync/repos/lnc-components")
ITEMS = OUT / "items"

WRAPPER = """<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — LNC Component</title>
<style>
*{{margin:0;box-sizing:border-box}}
body{{min-height:100vh;display:grid;place-items:center;background:{bg};color:{fg};
font-family:ui-sans-serif,system-ui,-apple-system,sans-serif;padding:28px;overflow-x:hidden}}
.stage{{width:min(760px,100%);display:grid;place-items:center;gap:18px;position:relative;z-index:1}}
.tag{{position:fixed;top:14px;left:16px;font:600 10px/1 ui-monospace,monospace;letter-spacing:.16em;
text-transform:uppercase;opacity:.45;z-index:5}}
{css}
</style></head><body>
<div class="tag">LNC · {cat} · {cid}</div>
{bgmarkup}
<div class="stage">
{body}
</div>
{js}
</body></html>
"""

DARK = ("#0a0c12", "#e8eaf0")
LIGHT = ("#f5f6f8", "#181b22")

def c(cid, cat, name, desc, tags, css, body, js="", dark=True, bgmarkup=""):
    bg, fg = DARK if dark else LIGHT
    return {
        "id": cid, "cat": cat, "name": name, "desc": desc, "tags": tags,
        "html": WRAPPER.format(title=name, bg=bg, fg=fg, cat=cat, cid=cid,
                               css=css, body=body, js=js, bgmarkup=bgmarkup),
    }

COMPS = []

# ---------------------------------------------------------------- backgrounds
COMPS += [
c("aurora-bg","backgrounds","Aurora Background","Slow-drifting aurora gradient blobs for hero sections.",
 "aurora,gradient,animated,hero,background","""
.aurora{position:absolute;inset:-40%;filter:blur(70px);opacity:.55}
.aurora i{position:absolute;border-radius:50%}
.aurora i:nth-child(1){width:44vw;height:44vw;background:#7c3aed;top:6%;left:8%;animation:d1 14s ease-in-out infinite alternate}
.aurora i:nth-child(2){width:36vw;height:36vw;background:#0ea5e9;top:34%;left:44%;animation:d2 18s ease-in-out infinite alternate}
.aurora i:nth-child(3){width:30vw;height:30vw;background:#ec4899;top:52%;left:20%;animation:d3 16s ease-in-out infinite alternate}
@keyframes d1{to{transform:translate(16vw,10vh) scale(1.15)}}
@keyframes d2{to{transform:translate(-14vw,-8vh) scale(.9)}}
@keyframes d3{to{transform:translate(10vw,-14vh) scale(1.2)}}
h1{font-size:clamp(28px,5vw,52px);font-weight:800;text-align:center;letter-spacing:-.02em}
p{opacity:.6;text-align:center}
""","""<div class="aurora"><i></i><i></i><i></i></div>
<h1>Aurora gradient drift</h1>
<p>Three blurred color fields on slow 14–18s alternate loops.</p>"""),
c("grid-pattern","backgrounds","Grid Pattern","Fine blueprint grid with radial fade mask — classic dev-tool backdrop.",
 "grid,pattern,dev,background","""
.grid{position:absolute;inset:0;
background-image:linear-gradient(#ffffff14 1px,transparent 1px),linear-gradient(90deg,#ffffff14 1px,transparent 1px);
background-size:44px 44px;
-webkit-mask-image:radial-gradient(ellipse 80% 70% at 50% 45%,#000 30%,transparent 75%);
mask-image:radial-gradient(ellipse 80% 70% at 50% 45%,#000 30%,transparent 75%)}
h1{font-size:clamp(28px,5vw,48px);font-weight:800;text-align:center}
p{opacity:.6;text-align:center}
""","""<div class="grid"></div><h1>Blueprint grid + radial mask</h1><p>44px cells, faded edges via mask-image.</p>"""),
c("dot-grid","backgrounds","Dot Matrix Spotlight","Dot grid that reveals a spotlight ring following the cursor.",
 "dots,grid,spotlight,interactive,background","""
.dots{position:absolute;inset:0;background-image:radial-gradient(#ffffff2e 1.4px,transparent 1.4px);background-size:26px 26px;
-webkit-mask-image:radial-gradient(220px circle at var(--mx,50%) var(--my,50%),#000 0%,transparent 100%);
mask-image:radial-gradient(220px circle at var(--mx,50%) var(--my,50%),#000 0%,transparent 100%)}
.base{position:absolute;inset:0;background-image:radial-gradient(#ffffff10 1.4px,transparent 1.4px);background-size:26px 26px}
h1{font-size:clamp(26px,4.5vw,44px);font-weight:800;text-align:center}
p{opacity:.6;text-align:center}
""","""<div class="base"></div><div class="dots"></div><h1>Move your cursor</h1><p>Bright dot layer revealed through a 220px mask.</p>""",
"""<script>
document.addEventListener('pointermove',e=>{
 document.querySelector('.dots').style.setProperty('--mx',e.clientX+'px');
 document.querySelector('.dots').style.setProperty('--my',e.clientY+'px');});
</script>"""),
c("noise-texture","backgrounds","Film Grain Noise","SVG fractal-noise grain overlay — instantly adds print/film texture.",
 "noise,grain,texture,svg,background","""
.grain{position:fixed;inset:0;pointer-events:none;opacity:.5;z-index:0}
.panel{background:#141722cc;backdrop-filter:blur(4px);border:1px solid #ffffff1a;border-radius:16px;
padding:34px 44px;text-align:center;box-shadow:0 18px 50px -18px #000}
h1{font-size:26px;font-weight:800;margin-bottom:8px}
p{opacity:.6;font-size:14px}
""","""<svg class="grain"><filter id="n"><feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="2"/><feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope="0.05"/></feComponentTransfer></filter><rect width="100%" height="100%" filter="url(#n)"/></svg>
<div class="panel"><h1>Grain on demand</h1><p>feTurbulence at 5% alpha — cheap cinematic texture.</p></div>"""),
c("gradient-drift","backgrounds","Mesh Gradient Drift","Conic mesh gradient slowly rotating behind content.",
 "gradient,mesh,conic,animated,background","""
.mesh{position:absolute;inset:-50%;background:conic-gradient(from 0deg,#312e81,#0e7490,#7c3aed,#be185d,#312e81);
filter:blur(90px);opacity:.5;animation:spin 24s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
h1{font-size:clamp(28px,5vw,48px);font-weight:800;text-align:center}
p{opacity:.65;text-align:center}
""","""<div class="mesh"></div><h1>Conic mesh drift</h1><p>One blurred conic-gradient rotating at 24s.</p>"""),
c("flickering-grid","backgrounds","Flickering Grid","Grid cells randomly pulse on/off — synthwave ambience.",
 "grid,flicker,animated,canvas,background","""
canvas{position:absolute;inset:0;width:100%;height:100%}
h1{font-size:clamp(28px,5vw,48px);font-weight:800;text-align:center}
p{opacity:.6;text-align:center}
""","""<canvas id="cv"></canvas><h1>Flickering grid</h1><p>Canvas cells fade in/out at random.</p>""",
"""<script>
const cv=document.getElementById('cv'),x=cv.getContext('2d');let W,H,cells=[];
const S=34;
function rs(){W=cv.width=innerWidth;H=cv.height=innerHeight;cells=[];
for(let i=0;i<W;i+=S)for(let j=0;j<H;j+=S)cells.push({x:i,y:j,a:Math.random()*.12,v:(Math.random()*.008+.002)*(Math.random()<.5?1:-1)});}
rs();addEventListener('resize',rs);
(function tick(){x.clearRect(0,0,W,H);
for(const c of cells){c.a+=c.v;if(c.a<=0||c.a>.14)c.v*=-1;
x.fillStyle='rgba(140,160,255,'+Math.max(0,c.a)+')';x.fillRect(c.x,c.y,S-1,S-1);}
requestAnimationFrame(tick);})();
</script>"""),
c("perspective-grid","backgrounds","Perspective Grid Floor","3D floor grid scrolling toward the viewer — retro-future hero base.",
 "grid,perspective,3d,retro,background","""
.floor{position:absolute;left:-20%;right:-20%;bottom:-8%;height:60%;
background-image:linear-gradient(#7c3aed55 1.5px,transparent 1.5px),linear-gradient(90deg,#7c3aed55 1.5px,transparent 1.5px);
background-size:54px 54px;transform:perspective(480px) rotateX(62deg);
animation:roll 2.6s linear infinite;
-webkit-mask-image:linear-gradient(to top,#000 40%,transparent);mask-image:linear-gradient(to top,#000 40%,transparent)}
@keyframes roll{to{background-position:0 54px,0 0}}
h1{font-size:clamp(30px,5vw,52px);font-weight:800;text-align:center;
background:linear-gradient(180deg,#fff,#a78bfa);-webkit-background-clip:text;background-clip:text;color:transparent}
p{opacity:.6;text-align:center}
""","""<div class="floor"></div><h1>OUTRUN FLOOR</h1><p>rotateX(62deg) + background-position scroll.</p>"""),
c("signal-lines","backgrounds","Signal Lines","Thin light pulses traveling along horizontal traces.",
 "lines,signal,pulse,circuit,background","""
.trace{position:absolute;left:0;right:0;height:1px;background:#ffffff10}
.trace i{position:absolute;top:-1px;width:120px;height:3px;border-radius:3px;
background:linear-gradient(90deg,transparent,#38bdf8,transparent);animation:fly linear infinite}
.trace:nth-child(1){top:22%}.trace:nth-child(1) i{animation-duration:5s}
.trace:nth-child(2){top:52%}.trace:nth-child(2) i{animation-duration:7.5s;animation-delay:-3s;background:linear-gradient(90deg,transparent,#a78bfa,transparent)}
.trace:nth-child(3){top:78%}.trace:nth-child(3) i{animation-duration:6.2s;animation-delay:-1.4s;background:linear-gradient(90deg,transparent,#34d399,transparent)}
@keyframes fly{from{left:-140px}to{left:105%}}
h1{font-size:clamp(28px,5vw,48px);font-weight:800;text-align:center}
p{opacity:.6;text-align:center}
""","""<div class="trace"><i></i></div><div class="trace"><i></i></div><div class="trace"><i></i></div>
<h1>Signal traces</h1><p>Gradient pulses on thin tracks, staggered delays.</p>"""),
]

# ---------------------------------------------------------------- buttons
COMPS += [
c("shimmer-button","buttons","Shimmer Button","Primary CTA with a light beam sweeping across on loop.",
 "button,shimmer,cta,animated","""
.btn{position:relative;overflow:hidden;border:0;border-radius:10px;padding:14px 34px;cursor:pointer;
background:linear-gradient(135deg,#6366f1,#8b5cf6);color:#fff;font:600 15px/1 inherit;letter-spacing:.02em;
box-shadow:0 8px 30px -8px #6366f1aa}
.btn::after{content:"";position:absolute;top:0;left:-80%;width:55%;height:100%;
background:linear-gradient(105deg,transparent,#ffffff66,transparent);transform:skewX(-20deg);
animation:sweep 2.4s ease-in-out infinite}
@keyframes sweep{55%,100%{left:130%}}
""","""<button class="btn">Deploy Preview</button>"""),
c("gradient-border-button","buttons","Gradient Border Button","Rotating conic-gradient border around a dark inner button.",
 "button,gradient,border,animated","""
.wrap{position:relative;padding:2px;border-radius:12px;background:conic-gradient(from var(--a,0deg),#38bdf8,#a78bfa,#f472b6,#38bdf8);
animation:rot 3s linear infinite}
@property --a{syntax:"<angle>";initial-value:0deg;inherits:false}
@keyframes rot{to{--a:360deg}}
.wrap button{border:0;border-radius:10px;padding:14px 34px;cursor:pointer;background:#0d0f16;color:#fff;font:600 15px/1 inherit}
""","""<div class="wrap"><button>Rotating border</button></div>
<p style="opacity:.5;font-size:12px;max-width:420px;text-align:center">@property --a drives the conic angle; falls back to a static gradient in older engines.</p>"""),
c("magnetic-button","buttons","Magnetic Button","Button is gently attracted toward the cursor, springs back on leave.",
 "button,magnetic,interactive,js","""
.btn{border:0;border-radius:12px;padding:16px 38px;cursor:pointer;
background:#f5f5f4;color:#111;font:600 15px/1 inherit;transition:transform .18s cubic-bezier(.2,.8,.3,1.4);
box-shadow:0 10px 40px -12px #ffffff44}
""","""<button class="btn" id="b">Hover near me</button>""",
"""<script>
const b=document.getElementById('b');
document.addEventListener('pointermove',e=>{
const r=b.getBoundingClientRect(),cx=r.left+r.width/2,cy=r.top+r.height/2;
const dx=e.clientX-cx,dy=e.clientY-cy,d=Math.hypot(dx,dy);
if(d<160){b.style.transform='translate('+dx*.28+'px,'+dy*.28+'px)'}else{b.style.transform=''}});
b.addEventListener('pointerleave',()=>b.style.transform='');
</script>"""),
c("glass-button","buttons","Glass Button","Frosted-glass pill over a colorful backdrop — blur + inner highlight.",
 "button,glass,glassmorphism,blur","""
.orb{position:absolute;width:260px;height:260px;border-radius:50%;filter:blur(50px);opacity:.8;z-index:0}
.o1{background:#f472b6;top:-60px;left:-40px}.o2{background:#38bdf8;bottom:-70px;right:-50px}
.stage{border-radius:20px;overflow:hidden;position:relative;padding:80px 110px;background:#1e293b}
.btn{position:relative;z-index:1;border:1px solid #ffffff3a;border-radius:14px;padding:15px 38px;cursor:pointer;
background:#ffffff14;backdrop-filter:blur(14px);color:#fff;font:600 15px/1 inherit;
box-shadow:inset 0 1px 0 #ffffff45,0 12px 34px -12px #0009}
.btn:hover{background:#ffffff22}
""","""<div class="stage"><i class="orb o1"></i><i class="orb o2"></i><button class="btn">Glass action</button></div>"""),
c("neon-glow-button","buttons","Neon Glow Button","Cyan neon ring button with breathing glow.",
 "button,neon,glow,dark","""
.btn{border:2px solid #22d3ee;border-radius:12px;padding:14px 36px;cursor:pointer;background:#06232b;
color:#a5f3fc;font:700 15px/1 inherit;letter-spacing:.08em;text-transform:uppercase;
box-shadow:0 0 14px #22d3ee66,inset 0 0 12px #22d3ee33;animation:br 2.2s ease-in-out infinite}
.btn:hover{background:#0a3540}
@keyframes br{50%{box-shadow:0 0 30px #22d3eeaa,inset 0 0 20px #22d3ee55}}
""","""<button class="btn">Initialize</button>"""),
c("ripple-button","buttons","Ripple Button","Material-style ink ripple expanding from the click point.",
 "button,ripple,material,interactive,js","""
.btn{position:relative;overflow:hidden;border:0;border-radius:10px;padding:14px 36px;cursor:pointer;
background:#0284c7;color:#fff;font:600 15px/1 inherit}
.rp{position:absolute;border-radius:50%;background:#ffffff55;transform:scale(0);animation:rip .55s ease-out forwards;pointer-events:none}
@keyframes rip{to{transform:scale(3);opacity:0}}
""","""<button class="btn" id="b">Click for ripple</button>""",
"""<script>
document.getElementById('b').addEventListener('click',function(e){
const r=this.getBoundingClientRect(),s=Math.max(r.width,r.height);
const i=document.createElement('i');i.className='rp';
i.style.cssText='width:'+s+'px;height:'+s+'px;left:'+(e.clientX-r.left-s/2)+'px;top:'+(e.clientY-r.top-s/2)+'px';
this.appendChild(i);setTimeout(()=>i.remove(),600);});
</script>"""),
c("push-3d-button","buttons","3D Push Button","Physical key-cap feel — hard shadow collapses on press.",
 "button,3d,push,tactile","""
.btn{border:0;border-radius:12px;padding:14px 34px;cursor:pointer;background:#fbbf24;color:#422006;
font:800 15px/1 inherit;box-shadow:0 6px 0 #b45309,0 10px 20px -6px #0008;transition:.08s}
.btn:active{transform:translateY(6px);box-shadow:0 0 0 #b45309,0 3px 8px -4px #0008}
""","""<button class="btn">Press me</button>"""),
c("loading-button","buttons","Loading Button","Click morphs label into a spinner then resolves to success.",
 "button,loading,state,js","""
.btn{display:inline-flex;align-items:center;gap:9px;border:0;border-radius:10px;padding:14px 32px;cursor:pointer;
background:#16a34a;color:#fff;font:600 15px/1 inherit;min-width:170px;justify-content:center;transition:background .3s}
.btn:disabled{opacity:.85;cursor:default}
.sp{width:16px;height:16px;border:2.5px solid #ffffff55;border-top-color:#fff;border-radius:50%;animation:sp .7s linear infinite}
@keyframes sp{to{transform:rotate(360deg)}}
""","""<button class="btn" id="b"><span id="t">Save changes</span></button>""",
"""<script>
const b=document.getElementById('b'),t=document.getElementById('t');
b.onclick=()=>{b.disabled=true;t.innerHTML='<span class="sp"></span>';
setTimeout(()=>{b.style.background='#15803d';t.textContent='✓ Saved';
setTimeout(()=>{b.disabled=false;b.style.background='';t.textContent='Save changes'},1600)},1400)};
</script>"""),
]

# ---------------------------------------------------------------- cards
COMPS += [
c("spotlight-card","cards","Spotlight Card","Radial highlight tracks the cursor across card surfaces.",
 "card,spotlight,interactive,js","""
.row{display:flex;gap:16px;flex-wrap:wrap;justify-content:center}
.cd{position:relative;width:220px;padding:22px;border-radius:14px;border:1px solid #ffffff14;background:#12141c;overflow:hidden}
.cd::before{content:"";position:absolute;inset:0;opacity:0;transition:opacity .3s;
background:radial-gradient(240px circle at var(--mx,50%) var(--my,50%),#6366f12b,transparent 70%)}
.cd:hover::before{opacity:1}
.cd h3{font-size:15px;margin-bottom:6px}.cd p{font-size:12.5px;opacity:.55;line-height:1.5}
""","""<div class="row">
<div class="cd"><h3>Realtime Sync</h3><p>Changes propagate to every client within 80ms via websockets.</p></div>
<div class="cd"><h3>Edge Cache</h3><p>Content served from 14 PoPs; stale-while-revalidate keeps it hot.</p></div>
<div class="cd"><h3>Zero Config</h3><p>Sensible defaults ship enabled — tune only what you need.</p></div>
</div>""",
"""<script>
document.querySelectorAll('.cd').forEach(c=>c.addEventListener('pointermove',e=>{
const r=c.getBoundingClientRect();
c.style.setProperty('--mx',(e.clientX-r.left)+'px');c.style.setProperty('--my',(e.clientY-r.top)+'px');}));
</script>"""),
c("tilt-card","cards","3D Tilt Card","Card tilts in 3D toward the pointer with a moving sheen.",
 "card,tilt,3d,interactive,js","""
.cd{width:300px;border-radius:16px;padding:26px;background:linear-gradient(160deg,#1a1d29,#12141c);
border:1px solid #ffffff14;transform-style:preserve-3d;transition:transform .12s;position:relative;overflow:hidden}
.cd::after{content:"";position:absolute;inset:0;background:radial-gradient(420px circle at var(--mx,50%) var(--my,50%),#ffffff12,transparent 60%)}
.cd h3{margin-bottom:8px;font-size:17px}
.cd p{font-size:13px;opacity:.55;line-height:1.55}
.k{display:inline-block;margin-top:14px;font:600 11px ui-monospace,monospace;color:#a78bfa;letter-spacing:.1em}
""","""<div class="cd" id="t"><h3>Perspective tilt</h3>
<p>RotateX/RotateY bounded to ±10° with perspective(800px). Sheen follows the pointer via CSS vars.</p>
<span class="k">HOVER TO TILT</span></div>""",
"""<script>
const t=document.getElementById('t');
t.addEventListener('pointermove',e=>{const r=t.getBoundingClientRect();
const x=(e.clientX-r.left)/r.width-.5,y=(e.clientY-r.top)/r.height-.5;
t.style.transform='perspective(800px) rotateY('+(x*10)+'deg) rotateX('+(-y*10)+'deg)';
t.style.setProperty('--mx',(e.clientX-r.left)+'px');t.style.setProperty('--my',(e.clientY-r.top)+'px');});
t.addEventListener('pointerleave',()=>t.style.transform='');
</script>"""),
c("glass-card","cards","Glass Card","Frosted panel floating over vivid gradient orbs.",
 "card,glass,blur,glassmorphism","""
.bg{position:absolute;inset:0;overflow:hidden;border-radius:20px}
.bg i{position:absolute;border-radius:50%;filter:blur(46px);opacity:.85}
.bg i:nth-child(1){width:220px;height:220px;background:#f472b6;top:-50px;left:-30px}
.bg i:nth-child(2){width:200px;height:200px;background:#38bdf8;bottom:-60px;right:-20px}
.bg i:nth-child(3){width:160px;height:160px;background:#a78bfa;top:40%;left:44%}
.stage{position:relative;border-radius:20px;overflow:hidden;padding:0}
.cd{position:relative;z-index:1;margin:64px;padding:26px 30px;border-radius:16px;
background:#ffffff14;backdrop-filter:blur(16px);border:1px solid #ffffff30;
box-shadow:inset 0 1px 0 #ffffff40,0 20px 50px -20px #000}
.cd h3{font-size:16px;margin-bottom:6px}.cd p{font-size:12.5px;opacity:.7;line-height:1.55}
""","""<div class="stage"><div class="bg"><i></i><i></i><i></i></div>
<div class="cd"><h3>Frosted panel</h3><p>backdrop-filter: blur(16px) over colored orbs, 18% white fill, hairline top highlight.</p></div></div>"""),
c("border-beam-card","cards","Border Beam Card","A light beam travels the card's border loop continuously.",
 "card,border,beam,animated","""
@property --a{syntax:"<angle>";initial-value:0deg;inherits:false}
.cd{position:relative;width:300px;border-radius:14px;padding:26px;background:#101218;border:1px solid transparent}
.cd::before{content:"";position:absolute;inset:-1px;border-radius:15px;padding:1.5px;
background:conic-gradient(from var(--a),transparent 0 78%,#38bdf8 88%,#a78bfa 96%,transparent 100%);
-webkit-mask:linear-gradient(#000 0 0) content-box,linear-gradient(#000 0 0);
-webkit-mask-composite:xor;mask-composite:exclude;animation:beam 3.6s linear infinite;pointer-events:none}
@keyframes beam{to{--a:360deg}}
.cd h3{font-size:16px;margin-bottom:6px}.cd p{font-size:12.5px;opacity:.55;line-height:1.55}
""","""<div class="cd"><h3>Beam border</h3><p>Conic gradient masked to the border ring, rotating via @property --a.</p></div>"""),
c("hover-lift-card","cards","Hover Lift Card","Elevation cue — card rises and deepens shadow on hover.",
 "card,hover,lift,shadow","""
.cd{width:280px;padding:26px;border-radius:16px;background:#16181f;border:1px solid #ffffff10;
transition:transform .25s cubic-bezier(.2,.7,.3,1.3),box-shadow .25s}
.cd:hover{transform:translateY(-8px);box-shadow:0 24px 50px -18px #000c,0 0 0 1px #ffffff14}
.cd h3{font-size:16px;margin-bottom:6px}.cd p{font-size:12.5px;opacity:.55;line-height:1.55}
.badge{display:inline-block;margin-bottom:12px;padding:4px 10px;border-radius:99px;background:#38bdf81c;color:#38bdf8;font:600 10px ui-monospace,monospace;letter-spacing:.1em}
""","""<div class="cd"><span class="badge">PRO TIP</span><h3>Lift on hover</h3>
<p>translateY(-8px) + two-layer shadow. Springy cubic-bezier overshoot sells the motion.</p></div>"""),
c("gradient-glow-card","cards","Gradient Glow Card","Colored glow bleeds behind the card on hover.",
 "card,glow,gradient,hover","""
.cd{position:relative;width:280px;padding:26px;border-radius:16px;background:#13151d;border:1px solid #ffffff10}
.cd::before{content:"";position:absolute;inset:-2px;border-radius:18px;z-index:-1;
background:linear-gradient(135deg,#38bdf8,#a78bfa,#f472b6);opacity:0;filter:blur(18px);transition:opacity .35s}
.cd:hover::before{opacity:.65}
.cd h3{font-size:16px;margin-bottom:6px}.cd p{font-size:12.5px;opacity:.55;line-height:1.55}
""","""<div class="cd"><h3>Back-glow</h3><p>Blurred gradient pseudo-element fades in behind the panel.</p></div>"""),
]

# ---------------------------------------------------------------- loaders
COMPS += [
c("skeleton-loader","loaders","Skeleton Loader","Shimmer sweep over content placeholders.",
 "loader,skeleton,placeholder,shimmer","""
.sk{width:min(420px,90vw);padding:20px;border-radius:14px;background:#12141c;border:1px solid #ffffff10}
.row{display:flex;gap:12px;align-items:center;margin-bottom:16px}
.av{width:44px;height:44px;border-radius:50%}
.l{height:12px;border-radius:6px;margin:8px 0}
.av,.l{position:relative;overflow:hidden;background:#ffffff12}
.av::after,.l::after{content:"";position:absolute;inset:0;transform:translateX(-100%);
background:linear-gradient(90deg,transparent,#ffffff14,transparent);animation:sh 1.6s infinite}
@keyframes sh{to{transform:translateX(100%)}}
""","""<div class="sk">
<div class="row"><div class="av"></div><div style="flex:1"><div class="l" style="width:55%"></div><div class="l" style="width:35%"></div></div></div>
<div class="l" style="width:100%"></div><div class="l" style="width:92%"></div><div class="l" style="width:70%"></div>
</div>"""),
c("orbit-spinner","loaders","Orbit Spinner","Dot orbiting a static core — compact indeterminate indicator.",
 "loader,spinner,orbit","""
.orb{position:relative;width:64px;height:64px}
.orb .core{position:absolute;inset:22px;border-radius:50%;background:#6366f1;box-shadow:0 0 18px #6366f1aa}
.orb .sat{position:absolute;inset:0;animation:sp 1.1s linear infinite}
.orb .sat i{position:absolute;top:0;left:50%;width:10px;height:10px;margin-left:-5px;border-radius:50%;background:#a78bfa}
@keyframes sp{to{transform:rotate(360deg)}}
""","""<div class="orb"><div class="sat"><i></i></div><div class="core"></div></div>"""),
c("dots-bounce","loaders","Bouncing Dots","Three dots stagger-bouncing — the universal typing indicator.",
 "loader,dots,typing,bounce","""
.dots{display:flex;gap:8px}
.dots i{width:11px;height:11px;border-radius:50%;background:#38bdf8;animation:b .9s ease-in-out infinite}
.dots i:nth-child(2){animation-delay:.15s}.dots i:nth-child(3){animation-delay:.3s}
@keyframes b{40%{transform:translateY(-14px)}}
""","""<div class="dots"><i></i><i></i><i></i></div>"""),
c("progress-glow","loaders","Glowing Progress","Progress bar with a luminous leading edge and stripes.",
 "loader,progress,bar,glow","""
.bar{width:min(420px,90vw);height:12px;border-radius:99px;background:#ffffff10;overflow:hidden}
.fill{height:100%;width:68%;border-radius:99px;position:relative;
background:linear-gradient(90deg,#38bdf8,#a78bfa);
box-shadow:0 0 14px #a78bfaaa;animation:p 2.4s ease-in-out infinite alternate}
.fill::after{content:"";position:absolute;inset:0;
background:repeating-linear-gradient(115deg,transparent 0 10px,#ffffff22 10px 20px);
animation:mv 1s linear infinite}
@keyframes p{from{width:18%}to{width:86%}}
@keyframes mv{to{background-position:28px 0}}
""","""<div class="bar"><div class="fill"></div></div>"""),
c("pulse-ring","loaders","Sonar Pulse","Expanding rings radiate from a point — beacon/live indicator.",
 "loader,pulse,sonar,beacon","""
.pt{position:relative;width:18px;height:18px}
.pt i{position:absolute;inset:0;border-radius:50%;background:#34d399}
.pt::before,.pt::after{content:"";position:absolute;inset:0;border-radius:50%;
border:2px solid #34d399;animation:sn 1.8s ease-out infinite}
.pt::after{animation-delay:.9s}
@keyframes sn{from{transform:scale(1);opacity:.9}to{transform:scale(3.4);opacity:0}}
""","""<div style="display:flex;align-items:center;gap:14px"><div class="pt"><i></i></div>
<span style="font:600 13px/1 inherit;letter-spacing:.08em;color:#34d399">LIVE</span></div>"""),
]

# ---------------------------------------------------------------- inputs & toggles
COMPS += [
c("ios-switch","inputs-toggles","iOS Switch","Springy iOS-style toggle — knob stretches while moving.",
 "toggle,switch,ios,input","""
input{display:none}
.sw{display:inline-block;width:58px;height:34px;border-radius:99px;background:#3a3f4d;position:relative;cursor:pointer;transition:background .25s}
.sw i{position:absolute;top:3px;left:3px;width:28px;height:28px;border-radius:99px;background:#fff;
transition:transform .3s cubic-bezier(.3,1.4,.4,1),width .2s;box-shadow:0 2px 6px #0006}
.sw:active i{width:34px}
input:checked+.sw{background:#34d399}
input:checked+.sw i{transform:translateX(24px)}
input:checked+.sw:active i{transform:translateX(18px)}
""","""<label><input type="checkbox" checked><span class="sw"><i></i></span></label>
<p style="opacity:.5;font-size:12px">Hold to see the knob stretch.</p>"""),
c("animated-checkbox","inputs-toggles","Animated Checkbox","Check draws itself with an SVG stroke-dash animation.",
 "checkbox,animated,svg,input","""
input{display:none}
.cb{display:inline-flex;align-items:center;gap:10px;cursor:pointer;font:500 15px/1 inherit}
.box{width:24px;height:24px;border-radius:7px;border:2px solid #4b5265;display:grid;place-items:center;transition:.2s}
.box svg{width:14px;height:14px}
.box path{stroke:#fff;stroke-width:3.4;fill:none;stroke-linecap:round;stroke-linejoin:round;
stroke-dasharray:22;stroke-dashoffset:22;transition:stroke-dashoffset .25s ease}
input:checked+.cb .box{background:#6366f1;border-color:#6366f1}
input:checked+.cb .box path{stroke-dashoffset:0}
""","""<label><input type="checkbox" checked><span class="cb"><span class="box">
<svg viewBox="0 0 16 16"><path d="M3 8.5 6.5 12 13 4"/></svg></span>Enable autosave</span></label>"""),
c("floating-label-input","inputs-toggles","Floating Label Input","Label lifts into a caption when the field gains focus or content.",
 "input,label,form,floating","""
.f{position:relative;width:min(340px,90vw)}
.f input{width:100%;padding:16px 14px 8px;border-radius:10px;border:1.5px solid #ffffff1f;background:#12141c;color:#e8eaf0;
font:500 15px/1.2 inherit;outline:none;transition:border .2s}
.f input:focus{border-color:#6366f1}
.f label{position:absolute;left:14px;top:13px;color:#8b91a3;pointer-events:none;transition:.18s;font-size:15px}
.f input:focus~label,.f input:not(:placeholder-shown)~label{top:4px;font-size:10.5px;color:#818cf8;letter-spacing:.04em}
""","""<div class="f"><input id="i" placeholder=" " autocomplete="off"><label for="i">Workspace name</label></div>"""),
c("expanding-search","inputs-toggles","Expanding Search","Icon pill that stretches into a full search field on focus.",
 "input,search,expand,animated","""
.s{display:flex;align-items:center;gap:8px;width:44px;padding:11px;border-radius:99px;background:#16181f;
border:1px solid #ffffff14;transition:width .35s cubic-bezier(.2,.7,.3,1);overflow:hidden;cursor:text}
.s:focus-within{width:min(340px,80vw);border-color:#6366f1}
.s svg{flex:none;width:18px;height:18px;stroke:#8b91a3}
.s input{flex:1;min-width:0;border:0;background:none;color:#e8eaf0;font:500 14px/1 inherit;outline:none}
""","""<div class="s"><svg viewBox="0 0 24 24" fill="none" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>
<input placeholder="Search components..."></div>
<p style="opacity:.5;font-size:12px">Click the pill to expand.</p>"""),
c("otp-input","inputs-toggles","OTP Input","Six auto-advancing verification digit boxes.",
 "input,otp,code,js","""
.otp{display:flex;gap:9px}
.otp input{width:46px;height:54px;text-align:center;border-radius:11px;border:1.5px solid #ffffff1f;
background:#12141c;color:#e8eaf0;font:700 20px/1 ui-monospace,monospace;outline:none;transition:.15s}
.otp input:focus{border-color:#38bdf8;box-shadow:0 0 0 3px #38bdf826}
.otp input.filled{border-color:#34d39977}
""","""<div class="otp" id="o">
<input maxlength="1" inputmode="numeric"><input maxlength="1" inputmode="numeric"><input maxlength="1" inputmode="numeric">
<input maxlength="1" inputmode="numeric"><input maxlength="1" inputmode="numeric"><input maxlength="1" inputmode="numeric">
</div>""",
"""<script>
const box=[...document.querySelectorAll('#o input')];
box.forEach((b,i)=>{b.addEventListener('input',()=>{b.value=b.value.replace(/\\D/g,'');
b.classList.toggle('filled',!!b.value);if(b.value&&i<5)box[i+1].focus();});
b.addEventListener('keydown',e=>{if(e.key==='Backspace'&&!b.value&&i>0)box[i-1].focus();});});
box[0].focus();
</script>"""),
c("rating-stars","inputs-toggles","Rating Stars","Hover preview + click to lock rating, subtle pop on select.",
 "input,rating,stars,js","""
.st{display:flex;gap:6px}
.st svg{width:34px;height:34px;fill:none;stroke:#525a70;stroke-width:1.6;cursor:pointer;transition:.15s}
.st svg.on{fill:#fbbf24;stroke:#fbbf24;filter:drop-shadow(0 0 8px #fbbf2466)}
.st svg.pop{animation:pp .3s cubic-bezier(.3,1.6,.4,1)}
@keyframes pp{50%{transform:scale(1.25)}}
.lbl{font:600 13px/1 inherit;color:#8b91a3}
""","""<div style="display:grid;place-items:center;gap:10px">
<div class="st" id="s"></div><span class="lbl" id="l">Rate this component</span></div>""",
"""<script>
const S=document.getElementById('s'),L=document.getElementById('l');let val=0;
for(let i=1;i<=5;i++){const s=document.createElementNS('http://www.w3.org/2000/svg','svg');
s.setAttribute('viewBox','0 0 24 24');s.dataset.v=i;
s.innerHTML='<path d="M12 2.6l2.9 6 6.6.9-4.8 4.6 1.2 6.5L12 17.5l-5.9 3.1 1.2-6.5L2.5 9.5l6.6-.9z"/>';
s.onmouseenter=()=>paint(i);s.onmouseleave=()=>paint(val);
s.onclick=()=>{val=i;paint(i);s.classList.remove('pop');void s.offsetWidth;s.classList.add('pop');
L.textContent=i+'/5 — nice';};
S.appendChild(s);}
function paint(n){[...S.children].forEach((c,j)=>c.classList.toggle('on',j<n));}
</script>"""),
]

# ---------------------------------------------------------------- text effects
COMPS += [
c("gradient-text","text-effects","Animated Gradient Text","Headline gradient slowly pans across the glyphs.",
 "text,gradient,animated,headline","""
h1{font-size:clamp(34px,7vw,68px);font-weight:800;letter-spacing:-.02em;text-align:center;
background:linear-gradient(100deg,#38bdf8,#a78bfa,#f472b6,#fbbf24,#38bdf8);background-size:300% 100%;
-webkit-background-clip:text;background-clip:text;color:transparent;animation:pan 6s linear infinite}
@keyframes pan{to{background-position:300% 0}}
""","""<h1>Ship beautiful UI<br>without the fight</h1>"""),
c("typewriter","text-effects","Typewriter","Terminal-style typing with blinking block cursor, loops phrases.",
 "text,typewriter,typing,js","""
.t{font:600 clamp(20px,3.4vw,34px)/1.4 ui-monospace,monospace;color:#34d399}
.t::after{content:"▌";margin-left:2px;color:#34d399;animation:bl 1s steps(1) infinite}
@keyframes bl{50%{opacity:0}}
""","""<div class="t"><span id="t"></span></div>""",
"""<script>
const P=['design systems on demand.','copy-paste motion recipes.','effects without deps.'];
const el=document.getElementById('t');let pi=0,ci=0,del=false;
(function tick(){const p=P[pi];
el.textContent=p.slice(0,ci);
if(!del){ci++;if(ci>p.length){del=true;return setTimeout(tick,1600)}}
else{ci--;if(ci<0){del=false;pi=(pi+1)%P.length}}
setTimeout(tick,del?26:58);})();
</script>"""),
c("decode-scramble","text-effects","Decode Scramble","Text scrambles through glyphs then resolves — hover to replay.",
 "text,scramble,decode,js","""
.d{font:700 clamp(24px,4vw,42px)/1.2 ui-monospace,monospace;letter-spacing:.04em;cursor:default}
""","""<div class="d" id="d">DECODE_ME</div>
<p style="opacity:.5;font-size:12px">Hover to re-scramble.</p>""",
"""<script>
const CH='!<>-_\\\\/[]{}—=+*^?#';
const el=document.getElementById('d');const orig=el.textContent;let raf;
function run(){let f=0;cancelAnimationFrame(raf);
(function step(){f++;
el.textContent=orig.split('').map((c,i)=>i<f/2.4?c:CH[Math.random()*CH.length|0]).join('');
if(f/2.4<orig.length)raf=requestAnimationFrame(step);else el.textContent=orig;})();}
run();el.addEventListener('mouseenter',run);
</script>"""),
c("shimmer-text","text-effects","Shimmer Text","A thin shine sweeps periodically across solid text.",
 "text,shimmer,shine","""
h1{font-size:clamp(34px,6vw,60px);font-weight:800;letter-spacing:-.01em;text-align:center;
background:linear-gradient(110deg,#e8eaf0 40%,#ffffff 48%,#7dd3fc 50%,#ffffff 52%,#e8eaf0 60%);
background-size:250% 100%;-webkit-background-clip:text;background-clip:text;color:transparent;
animation:sh 3.2s ease-in-out infinite}
@keyframes sh{60%,100%{background-position:-160% 0}}
""","""<h1>Press to continue</h1>"""),
c("number-ticker","text-effects","Number Ticker","Stat counts up with eased animation when visible.",
 "text,number,counter,stat,js","""
.n{font:800 clamp(40px,7vw,72px)/1 ui-monospace,monospace;color:#fbbf24}
.s{opacity:.5;font-size:13px;letter-spacing:.1em;text-transform:uppercase}
""","""<div style="display:grid;place-items:center;gap:8px"><div class="n" id="n">0</div>
<span class="s">components shipped</span></div>""",
"""<script>
const T=1284,el=document.getElementById('n');let t0=null;
function f(ts){if(!t0)t0=ts;const p=Math.min(1,(ts-t0)/1800),e=1-Math.pow(1-p,3);
el.textContent=Math.round(T*e).toLocaleString();
if(p<1)requestAnimationFrame(f);}
requestAnimationFrame(f);
</script>"""),
]

# ---------------------------------------------------------------- ai-agent
COMPS += [
c("tool-call-card","ai-agent","Tool Call Card","Agent tool invocation card — name, args, running/done status.",
 "ai,tool,call,card,agent","""
.tc{width:min(460px,92vw);border-radius:13px;border:1px solid #ffffff16;background:#12141c;overflow:hidden}
.hd{display:flex;align-items:center;gap:9px;padding:11px 15px;border-bottom:1px solid #ffffff0e;font:600 12.5px ui-monospace,monospace}
.hd .ic{width:22px;height:22px;border-radius:6px;background:#6366f126;display:grid;place-items:center;font-size:11px}
.hd .st{margin-left:auto;font-size:10.5px;color:#34d399;display:flex;align-items:center;gap:5px}
.hd .st::before{content:"";width:6px;height:6px;border-radius:50%;background:#34d399;box-shadow:0 0 8px #34d399}
pre{padding:13px 15px;font:12px/1.6 ui-monospace,monospace;color:#9aa3b8;overflow-x:auto}
.k{color:#7dd3fc}.s{color:#fbbf24}
""","""<div class="tc"><div class="hd"><span class="ic">⌁</span>styles__search<span class="st">done · 84ms</span></div>
<pre>{ <span class="k">"query"</span>: <span class="s">"minimal dashboard"</span>, <span class="k">"limit"</span>: 12 }</pre></div>"""),
c("approval-prompt","ai-agent","Approval Prompt","Inline approve/deny gate for agent actions — keyboard-friendly.",
 "ai,approval,prompt,agent,js","""
.ap{width:min(440px,92vw);border-radius:14px;border:1px solid #fbbf2440;background:#17140e;padding:16px 18px}
.ap h4{font-size:13.5px;margin-bottom:4px;display:flex;gap:8px;align-items:center}
.ap p{font-size:12px;opacity:.6;margin-bottom:14px;line-height:1.5}
.ap .row{display:flex;gap:9px}
.ap button{flex:1;border-radius:9px;padding:9px;border:1px solid #ffffff1c;background:transparent;color:#e8eaf0;
font:600 12.5px/1 inherit;cursor:pointer;transition:.15s}
.ap button:first-child{background:#34d399;border-color:#34d399;color:#06281c}
.ap button:hover{filter:brightness(1.1)}
.res{font:600 12.5px/1 inherit;text-align:center;padding:4px}
""","""<div class="ap" id="a"><h4>⚠ Agent requests write access</h4>
<p>Post <b>invoice-0421.pdf</b> to <code>POST /api/files</code>. Review before allowing.</p>
<div class="row"><button id="y">✓ Approve</button><button id="n">✕ Deny</button></div></div>""",
"""<script>
const a=document.getElementById('a');
y.onclick=()=>a.innerHTML='<div class="res" style="color:#34d399">✓ Approved — request sent</div>';
n.onclick=()=>a.innerHTML='<div class="res" style="color:#f87171">✕ Denied — action cancelled</div>';
</script>"""),
c("streaming-text","ai-agent","Streaming Text","Token-by-token response rendering with a live caret.",
 "ai,streaming,text,llm,js","""
.st{width:min(460px,92vw);font:14px/1.7 ui-sans-serif,system-ui;color:#d7dbe6}
.st .cur{display:inline-block;width:8px;height:15px;background:#a78bfa;margin-left:1px;vertical-align:-2px;
animation:bl .9s steps(1) infinite}
@keyframes bl{50%{opacity:0}}
""","""<div class="st"><span id="s"></span><span class="cur" id="c"></span></div>""",
"""<script>
const T='Searching 1,363 design systems… found 22 matches. The closest is "Heptabase" — a warm-minimal AI workspace style with generous radius and muted indigo accents.';
const s=document.getElementById('s');let i=0;
(function t(){if(i<=T.length){s.textContent=T.slice(0,i);i+=1+(Math.random()*2|0);setTimeout(t,34)}
else document.getElementById('c').style.display='none';})();
</script>"""),
c("voice-orb","ai-agent","Voice Orb","Breathing orb for voice-agent idle/listening states.",
 "ai,voice,orb,animated","""
.orb{position:relative;width:120px;height:120px;border-radius:50%;
background:radial-gradient(circle at 34% 30%,#a78bfa,#6366f1 55%,#312e81);
box-shadow:0 0 60px #6366f188,inset -12px -16px 34px #0007,inset 8px 10px 22px #ffffff30;
animation:br 3.2s ease-in-out infinite}
.orb::after{content:"";position:absolute;inset:-14px;border-radius:50%;
border:1.5px solid #a78bfa33;animation:wv 3.2s ease-in-out infinite}
@keyframes br{50%{transform:scale(1.08);box-shadow:0 0 90px #818cf8aa,inset -12px -16px 34px #0007,inset 8px 10px 22px #ffffff30}}
@keyframes wv{50%{transform:scale(1.12);opacity:.4}}
""","""<div class="orb"></div>"""),
c("agent-plan","ai-agent","Agent Plan Steps","Step checklist with done/active/pending states and connector line.",
 "ai,plan,steps,agent,checklist","""
.pl{width:min(400px,92vw);display:grid;gap:0}
.st{display:flex;gap:12px;position:relative;padding:10px 0}
.st:not(:last-child)::before{content:"";position:absolute;left:10.5px;top:34px;bottom:-4px;width:1.5px;background:#ffffff14}
.d{width:22px;height:22px;flex:none;border-radius:50%;display:grid;place-items:center;font-size:10px;z-index:1;
border:1.5px solid #ffffff22;background:#12141c;color:#8b91a3}
.done .d{background:#34d399;border-color:#34d399;color:#06281c}
.act .d{border-color:#a78bfa;color:#a78bfa;box-shadow:0 0 0 4px #a78bfa22;animation:pl 1.6s infinite}
@keyframes pl{50%{box-shadow:0 0 0 7px #a78bfa11}}
.st b{font-size:13px;display:block}
.st span{font-size:11.5px;opacity:.5}
.act b{color:#c4b5fd}.done{opacity:.65}
""","""<div class="pl">
<div class="st done"><div class="d">✓</div><div><b>Parse brief</b><span>Extracted 4 UI requirements</span></div></div>
<div class="st done"><div class="d">✓</div><div><b>Pick base style</b><span>Matched "Heptabase" (warm-minimal)</span></div></div>
<div class="st act"><div class="d">●</div><div><b>Generate layout</b><span>Composing hero + feature grid…</span></div></div>
<div class="st"><div class="d">4</div><div><b>Apply tokens</b><span>Pending</span></div></div>
</div>"""),
c("context-gauge","ai-agent","Context Gauge","Circular context-window usage ring with token count.",
 "ai,context,gauge,token,ring","""
.g{position:relative;width:120px;height:120px}
.g svg{transform:rotate(-90deg)}
.g circle{fill:none;stroke-width:9;stroke-linecap:round}
.g .tr{stroke:#ffffff12}
.g .us{stroke:url(#gg);stroke-dasharray:339;stroke-dashoffset:calc(339*(1 - var(--p,.62)));transition:stroke-dashoffset .8s}
.g .lbl{position:absolute;inset:0;display:grid;place-items:center;text-align:center}
.g .lbl b{font:800 20px/1 ui-monospace,monospace}
.g .lbl span{font-size:9.5px;opacity:.5;letter-spacing:.1em}
""","""<div class="g" style="--p:.62"><svg width="120" height="120">
<defs><linearGradient id="gg" x1="0" x2="1"><stop offset="0" stop-color="#38bdf8"/><stop offset="1" stop-color="#a78bfa"/></linearGradient></defs>
<circle class="tr" cx="60" cy="60" r="54"/><circle class="us" cx="60" cy="60" r="54"/></svg>
<div class="lbl"><div><b>62%</b><br><span>126K TOKENS</span></div></div></div>"""),
c("thinking-dots","ai-agent","Thinking Indicator","Waveform 'thinking' bars for agent processing state.",
 "ai,thinking,loading,dots,agent","""
.tk{display:flex;align-items:center;gap:4px;height:26px}
.tk i{width:4px;border-radius:3px;background:#a78bfa;animation:w 1.1s ease-in-out infinite}
.tk i:nth-child(1){height:10px;animation-delay:0s}
.tk i:nth-child(2){height:18px;animation-delay:.12s}
.tk i:nth-child(3){height:24px;animation-delay:.24s}
.tk i:nth-child(4){height:18px;animation-delay:.36s}
.tk i:nth-child(5){height:10px;animation-delay:.48s}
@keyframes w{50%{transform:scaleY(.35);opacity:.5}}
""","""<div style="display:flex;align-items:center;gap:12px"><div class="tk"><i></i><i></i><i></i><i></i><i></i></div>
<span style="font:500 13px/1 inherit;color:#8b91a3">Agent is thinking…</span></div>"""),
]

# ---------------------------------------------------------------- marketing
COMPS += [
c("badge-pill","marketing","Announcement Pill","Gradient announcement chip with shimmer + chevron.",
 "badge,pill,announcement,marketing","""
.p{display:inline-flex;align-items:center;gap:9px;padding:7px 8px 7px 15px;border-radius:99px;
border:1px solid #a78bfa44;background:linear-gradient(90deg,#6366f114,#a78bfa14);font:500 13px/1 inherit}
.p b{color:#c4b5fd;font-weight:600}
.p .go{display:inline-flex;align-items:center;gap:3px;padding:4px 10px;border-radius:99px;background:#6366f1;
color:#fff;font-size:11px;font-weight:600}
.p:hover{background:linear-gradient(90deg,#6366f124,#a78bfa24);cursor:pointer}
""","""<div class="p"><b>New</b> Components corpus is live
<span class="go">Explore →</span></div>"""),
c("logo-marquee","marketing","Logo Marquee","Infinite logo strip with edge fade masks.",
 "marquee,logos,scroll,infinite,marketing","""
.mq{width:min(600px,92vw);overflow:hidden;position:relative;
-webkit-mask-image:linear-gradient(90deg,transparent,#000 12%,#000 88%,transparent);
mask-image:linear-gradient(90deg,transparent,#000 12%,#000 88%,transparent)}
.tr{display:flex;gap:44px;width:max-content;animation:mq 18s linear infinite}
.mq:hover .tr{animation-play-state:paused}
.tr span{font:700 15px/1 inherit;color:#8b91a3;white-space:nowrap;display:flex;align-items:center;gap:8px}
.tr span i{width:9px;height:9px;border-radius:3px;background:#525a70;display:inline-block}
@keyframes mq{to{transform:translateX(-50%)}}
""","""<div class="mq"><div class="tr">
<span><i></i>ACME</span><span><i></i>NORTHWIND</span><span><i></i>INITECH</span><span><i></i>GLOBEX</span>
<span><i></i>HOOLI</span><span><i></i>PIED PIPER</span><span><i></i>STARK</span><span><i></i>WAYNE</span>
<span><i></i>ACME</span><span><i></i>NORTHWIND</span><span><i></i>INITECH</span><span><i></i>GLOBEX</span>
<span><i></i>HOOLI</span><span><i></i>PIED PIPER</span><span><i></i>STARK</span><span><i></i>WAYNE</span>
</div></div>"""),
c("notification-pop","marketing","Toast Notification","Slide-in toast with progress bar and auto-dismiss.",
 "toast,notification,pop,js","""
.t{position:fixed;right:24px;bottom:24px;width:320px;padding:15px 17px;border-radius:13px;
background:#14161f;border:1px solid #34d39944;box-shadow:0 18px 50px -14px #000;overflow:hidden;
transform:translateX(120%);animation:in .45s cubic-bezier(.2,1,.3,1) forwards}
.t h4{font-size:13px;display:flex;gap:8px;align-items:center;margin-bottom:3px}
.t p{font-size:12px;opacity:.6}
.t::after{content:"";position:absolute;left:0;bottom:0;height:2.5px;width:100%;background:#34d399;
transform-origin:left;animation:bar 4s linear forwards}
@keyframes in{to{transform:none}}
@keyframes bar{from{transform:scaleX(1)}to{transform:scaleX(0)}}
button{border:1px solid #ffffff22;background:#16181f;color:#e8eaf0;border-radius:9px;padding:10px 22px;
font:600 13px/1 inherit;cursor:pointer}
""","""<button id="b">Show toast</button>""",
"""<script>
b.onclick=()=>{const o=document.querySelector('.t');if(o)o.remove();
const t=document.createElement('div');t.className='t';
t.innerHTML='<h4>✓ Deploy complete</h4><p>styles.lengoc.me updated in 42s.</p>';
document.body.appendChild(t);setTimeout(()=>t.remove(),4300);};
b.click();
</script>"""),
c("code-window","marketing","Code Window Card","Mac-style code window with traffic lights + syntax tint.",
 "code,window,terminal,marketing","""
.w{width:min(460px,94vw);border-radius:13px;background:#0d0f16;border:1px solid #ffffff14;overflow:hidden;
box-shadow:0 24px 60px -20px #000}
.h{display:flex;gap:7px;padding:11px 14px;border-bottom:1px solid #ffffff0e}
.h i{width:11px;height:11px;border-radius:50%}
.h i:nth-child(1){background:#f87171}.h i:nth-child(2){background:#fbbf24}.h i:nth-child(3){background:#34d399}
pre{padding:16px 18px;font:12.5px/1.75 ui-monospace,monospace;color:#9aa3b8;overflow-x:auto}
.c{color:#5b6274}.k{color:#7dd3fc}.s{color:#fbbf24}.f{color:#a78bfa}
""","""<div class="w"><div class="h"><i></i><i></i><i></i></div>
<pre><span class="c"># fetch a design system</span>
<span class="k">await</span> mcp.<span class="f">call</span>(<span class="s">"styles__get_design"</span>, {
  style_id: <span class="s">"apple-hig"</span>
})</pre></div>"""),
]

# ---------------------------------------------------------------- write files
ITEMS.mkdir(parents=True, exist_ok=True)
index = []
for comp in COMPS:
    d = ITEMS / comp["id"]
    d.mkdir(parents=True, exist_ok=True)
    (d / "index.html").write_text(comp["html"], encoding="utf-8")
    index.append({
        "id": comp["id"],
        "name": comp["name"],
        "category": comp["cat"],
        "kind": "html",
        "description": comp["desc"],
        "tags": [t.strip() for t in comp["tags"].split(",")],
        "license": "LNC Internal (original work)",
        "source_url": "",
        "author": "LNC",
        "entry": "index.html",
    })

idx_path = OUT / "index.json"
existing = []
if idx_path.exists():
    try:
        existing = [r for r in json.loads(idx_path.read_text(encoding="utf-8")) if isinstance(r, dict)]
    except Exception:
        existing = []
keep = [r for r in existing if not str(r.get("id","")).startswith(tuple(i["id"] for i in index))]
idx_path.write_text(json.dumps(keep + index, ensure_ascii=False, indent=1), encoding="utf-8")
print(f"Wrote {len(index)} LNC components -> {ITEMS}")
print(f"index.json total: {len(keep) + len(index)}")
