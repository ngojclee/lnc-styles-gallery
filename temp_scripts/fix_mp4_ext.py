"""Rename shot/preview files that are actually MP4 videos saved with .jpg/.png ext.
Detects ftyp magic; renames to .mp4; rewrites meta.json shots lists.
"""
import json
from pathlib import Path

ROOT = Path(r'//10.21.2.2/data/_agentsync/repos/refero-styles/styles')
renamed = []
checked = 0


def is_mp4(p: Path) -> bool:
    try:
        with p.open('rb') as fh:
            head = fh.read(16)
        return len(head) >= 8 and head[4:8] == b'ftyp'
    except Exception:
        return False


for sdir in sorted(ROOT.iterdir()):
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
    for cand in list(sdir.rglob('*.*')):
        if cand.suffix.lower() not in ('.jpg', '.jpeg', '.png'):
            continue
        checked += 1
        if not is_mp4(cand):
            continue
        dst = cand.with_suffix('.mp4')
        if dst.exists():
            cand.unlink()
        else:
            cand.rename(dst)
        renamed.append(str(dst.relative_to(ROOT)))
        if meta.get('preview') == cand.name:
            meta['preview'] = dst.name
            changed = True
        if isinstance(meta.get('shots'), list):
            new_shots = []
            for item in meta['shots']:
                s = str(item)
                if s.endswith(cand.name):
                    s = s[: -len(cand.name)] + dst.name
                new_shots.append(s)
            meta['shots'] = new_shots
            changed = True
    if changed and meta_file.exists():
        meta_file.write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding='utf-8')

print(f'checked={checked} renamed={len(renamed)}')
for r in renamed[:15]:
    print(' ', r)
if len(renamed) > 15:
    print(f'  ... +{len(renamed)-15} more')
