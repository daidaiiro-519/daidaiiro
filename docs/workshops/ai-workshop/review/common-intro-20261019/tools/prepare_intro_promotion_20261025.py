"""確認済み導入の入力から、正式反映用の作業コピーと差分一覧を作る。"""
from pathlib import Path
import hashlib
import json
import shutil

ROOT = next(p for p in Path(__file__).resolve().parents if (p / 'CLAUDE.md').exists())
WORKSHOP = ROOT / 'docs/workshops/ai-workshop'
WORK = Path(__file__).resolve().parents[1] / 'workspace'
STAGE = WORK / 'promotion-20261025'
SRC = STAGE / 'material-1/src'
ORIGINAL = WORKSHOP / 'material-1/src'
SRC.mkdir(parents=True, exist_ok=True)
for pattern in ['*.py', '*.json']:
    for path in ORIGINAL.glob(pattern):
        shutil.copy2(path, SRC / path.name)
shutil.copytree(ORIGINAL / 'figures', SRC / 'figures', dirs_exist_ok=True)
(STAGE / 'proposal').mkdir(exist_ok=True)
link = STAGE / 'proposal/src'
if not link.exists():
    link.symlink_to(WORKSHOP / 'proposal/src', target_is_directory=True)
for folder in ['slides', 'scripts']:
    (STAGE / 'material-1/out' / folder).mkdir(parents=True, exist_ok=True)
layout = json.loads((WORK / 'review-layout.json').read_text())
cover = json.loads((WORK / 'cover.json').read_text())
slides = [json.loads((WORK / path).read_text()) for path in layout['sources']]
lesson = dict(no=1, key='lead', title='はじめに ── 共通の題材と、教材1〜3の学習の流れ',
              scope='3教材共通のはじめに', include_cover=True,
              cover={key: cover[key] for key in ['heading', 'lede', 'narration']},
              slides=[dict(id=f'COMMON-INTRO-{n:02d}', heading=s['heading'], narration=s['narration'])
                      for n, s in enumerate(slides, 1)])
(SRC / 'lesson-01-lead.json').write_text(json.dumps(lesson, ensure_ascii=False, indent=2) + '\n')
ledes = json.loads((SRC / 'ledes.json').read_text())
for key in list(ledes):
    if key.startswith('COMMON-INTRO-'):
        del ledes[key]
ledes.update({f'COMMON-INTRO-{n:02d}': s['lede'] for n, s in enumerate(slides, 1)})
(SRC / 'ledes.json').write_text(json.dumps(ledes, ensure_ascii=False, indent=2) + '\n')
# 担当する変更の説明を指す表現は、既存のタスク説明の例外と同じ用途である。
words = json.loads((SRC / 'words.json').read_text())
for group in words['同じものを指す語']:
    if 'タスク' in group.get('使わない', []):
        group['例外'] += ['タスクには「承認後は変更不可」', 'タスクの「承認後」']
(SRC / 'words.json').write_text(json.dumps(words, ensure_ascii=False, indent=2) + '\n')
figures = SRC / 'figures/common-intro'
for n, source in enumerate(layout['sources'], 1):
    for ext in ['source.svg', 'svg']:
        shutil.copy2(WORK / source.replace('.json', '.' + ext), figures / f'slide-{n:02d}.{ext}')
for ext in ['source.svg', 'svg']:
    shutil.copy2(WORK / f'cover.{ext}', figures / f'cover.{ext}')
    (figures / f'slide-10.{ext}').unlink()
shutil.copy2(WORK / 'theme.json', figures / 'theme.json')
p = SRC / 'common_intro_visuals.py'
p.write_text(p.read_text().replace(".read_text()", ".read_text().replace('<svg ', '<svg class=\"ws-visual\" ', 1)") + '\n\ndef cover():\n    return (HERE / "cover.svg").read_text().replace(\'<svg \', \'<svg class="ws-visual" \', 1)\n')
p = SRC / 'build_lessons.py'
t = p.read_text().replace(" 'COMMON-INTRO-10': lambda: CI.figure(10),\n", '')
old = "    slides = [dict(label='表紙', title=f'{no}本目　{name}', cls='ws-titlepage',\n                   intro=sub, body=V.cover(no), notes=d['slides'][0]['narration'])] if d.get('include_cover', True) else []"
new = "    cover = d.get('cover', {})\n    slides = [dict(label='表紙', title=cover.get('heading', f'{no}本目　{name}'), cls='ws-titlepage',\n                   intro=cover.get('lede', sub), body=CI.cover() if no == 1 and cover else V.cover(no),\n                   notes=cover.get('narration', d['slides'][0]['narration']))] if d.get('include_cover', True) else []"
assert old in t
t = t.replace(old, new)
old = "    for i, s in enumerate(content, 1):\n        notes.append"
new = "    if cover and d.get('include_cover', True):\n        notes.append(f'## 表紙　{cover[\"heading\"]}\\n\\n{cover[\"narration\"]}\\n')\n    for i, s in enumerate(content, 1):\n        notes.append"
assert old in t
t = t.replace(old, new)
t = t.replace('sum(len(x["notes"]) for x in content)', 'sum(len(x["notes"]) for x in (slides if cover else content))')
p.write_text(t)
p = SRC / 'build_narration.py'
t = p.read_text().replace("'text': title}", "'text': d.get('cover', {}).get('narration', title)}")
p.write_text(t)
files = []
for path in sorted(SRC.rglob('*')):
    if not path.is_file() or '__pycache__' in path.parts or path.suffix == '.pyc':
        continue
    rel = path.relative_to(SRC)
    orig = ORIGINAL / rel
    if orig.exists() and orig.read_bytes() == path.read_bytes():
        continue
    files.append(dict(target=str(orig.relative_to(ROOT)), staged=str(path.relative_to(ROOT)),
                      before_sha256=hashlib.sha256(orig.read_bytes()).hexdigest() if orig.exists() else None,
                      after_sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
for name in ['slide-10.source.svg', 'slide-10.svg']:
    orig = ORIGINAL / 'figures/common-intro' / name
    files.append(dict(target=str(orig.relative_to(ROOT)), staged=None,
                      before_sha256=hashlib.sha256(orig.read_bytes()).hexdigest(), after_sha256=None))
(STAGE / 'manifest.json').write_text(json.dumps(dict(scope='三教材共通のはじめに', files=files), ensure_ascii=False, indent=2) + '\n')
print(f'正式反映用の差分: {len(files)}ファイル')
