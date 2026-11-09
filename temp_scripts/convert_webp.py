"""Batch-convert all gallery images to WebP (q82, alpha preserved) on the canonical NAS corpus.

- refero-styles/styles/<id>/preview.* and shots/* -> .webp
- prompt-driven-ui/thumbs/**/*.png -> .webp
- Rewrites meta.json preview/shots fields; deletes originals after successful write.
"""
import json
import sys
import time
from pathlib import Path
from PIL import Image

ROOT = Path(r'//10.21.2.2/data/_agentsync/repos')
REFERO = ROOT / 'refero-styles' / 'styles'
THUMBS = ROOT / 'prompt-driven-ui' / 'thumbs'
QUALITY = 82
stats = {'done': 0, 'skipped': 0, 'failed': 0, 'bytes_before': 0, 'bytes_after': 0}


def convert_one(src: Path, dst: Path) -> bool:
    if dst.exists() and dst.stat().st_size > 0:
        stats['skipped'] += 1
        return True
    try:
        with Image.open(src) as im:
            im.load()
            if im.mode not in ('RGB', 'RGBA'):
                im = im.convert('RGBA' if 'A' in im.getbands() or 'transparency' in im.info else 'RGB')
            dst.parent.mkdir(parents=True, exist_ok=True)
            im.save(dst, 'WEBP', quality=QUALITY, method=4)
        stats['bytes_before'] += src.stat().st_size
        stats['bytes_after'] += dst.stat().st_size
        stats['done'] += 1
        return True
    except Exception as e:
        stats['failed'] += 1
        print('FAIL', src, e, flush=True)
        return False


t0 = time.time()

# 1. Refero styles: preview + shots
for i, sdir in enumerate(sorted(REFERO.iterdir())):
    if not sdir.is_dir():
        continue
    meta_file = sdir / 'meta.json'
    meta = {}
    if meta_file.exists():
        try:
            meta = json.loads(meta_file.read_text(encoding='utf-8'))
        except Exception:
            meta = {}
    changed = False
    # preview
    prev_field = meta.get('preview')
    for cand in list(sdir.glob('preview.*')):
        if cand.suffix.lower() in ('.jpg', '.jpeg', '.png'):
            dst = cand.with_suffix('.webp')
            if convert_one(cand, dst):
                cand.unlink()
                changed = True
                if not prev_field or prev_field == cand.name:
                    meta['preview'] = dst.name
    # shots
    shots_field = meta.get('shots') or []
    new_shots = []
    for cand in sorted((sdir / 'shots').glob('*.*')) if (sdir / 'shots').exists() else []:
        if cand.suffix.lower() in ('.jpg', '.jpeg', '.png'):
            dst = cand.with_suffix('.webp')
            if convert_one(cand, dst):
                cand.unlink()
                new_shots.append(f'shots/{dst.name}')
        elif cand.suffix.lower() in ('.webp', '.svg'):
            new_shots.append(f'shots/{cand.name}')
    if new_shots:
        meta['shots'] = new_shots
        changed = True
    if changed and meta_file.exists():
        meta_file.write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding='utf-8')
    if i % 200 == 0:
        print(f'[{i}] {stats["done"]} converted, {stats["failed"]} failed, {time.time()-t0:.0f}s', flush=True)

print(f'refero done: {stats}', flush=True)

# 2. prompt-driven-ui thumbs (recursive, keep alpha)
for cand in sorted(THUMBS.rglob('*.*')):
    if cand.suffix.lower() in ('.png', '.jpg', '.jpeg'):
        dst = cand.with_suffix('.webp')
        if convert_one(cand, dst):
            cand.unlink()

print(f'ALL DONE {stats} elapsed={time.time()-t0:.0f}s saved={stats["bytes_before"]-stats["bytes_after"]}', flush=True)
