"""作業コピーの入力から、企画書（詳細版・要点版）の比較用データを生成する。

変更前は before/proposal/src（2026-10-26 の正本の複製）、変更後は after/proposal/src から組む。
正本の proposal/out/ と、生成後の共通HTMLは変更しない。
"""
from pathlib import Path
import hashlib
import html
import json
import re
import shutil
import subprocess
import sys
import markdown

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'CLAUDE.md').exists())
PUBLIC = ROOT / 'docs/workshops/ai-workshop/review/common-intro-20261019/viewer/proposal-review-20261026'
CACHE = ROOT / '.artifact-backup/ai-workshop-proposal-review-20261026/generated'
PUBLIC.mkdir(parents=True, exist_ok=True)
NAMES = {'detail': '企画書 · 詳細版', 'brief': '企画書 · 要点版'}

slides = {}
for side in ['before', 'after']:
    out = CACHE / side
    subprocess.run([sys.executable, str(HERE / 'render_deck.py'), str(HERE / side / 'proposal/src'), str(out)], check=True, capture_output=True)
    slides[side] = json.loads((out / 'slides.json').read_text())
    for key in NAMES:
        shutil.copy2(out / f'{key}.html', PUBLIC / f'{side}-{key}.html')
        shutil.copy2(out / f'{key}-notes.md', HERE / side / f'{key}-notes.md')

# 語の検査は教材の words.json を使う。現行の企画書にも該当があるため、変更後に新しく増えた該当だけを止める
sys.path.insert(0, str(ROOT / 'docs/workshops/ai-workshop/material-1/src'))
import check_words
def hits(side):
    # 枚番号が行に入るので、番号を除いて比べる
    return {(re.sub(r'[0-9０-９]+', '', h[1]),) + tuple(h[2:]) for key in NAMES for h in check_words.check([CACHE / side / f'{key}.html', CACHE / side / f'{key}-notes.md'])}
new_hits = hits('after') - hits('before')
assert not new_hits, '\n'.join(map(str, sorted(new_hits)))


def note(s):
    return dict(number=s['number'], title=s['heading'],
                body='<h4>見出し</h4><p>' + html.escape(s['heading']) + '</p><h4>導入文</h4><p>' + html.escape(s['lede'])
                + '</p><h4>原稿</h4>' + markdown.markdown(s['narration']))


# 理由と反映状況は枚のラベルで持ち、詳細版と要点版のそれぞれの枚番号へ割り当てる
revision = json.loads((HERE / 'revisions.json').read_text())
payload = {}
for key, name in NAMES.items():
    def version(side):
        return hashlib.sha256((PUBLIC / f'{side}-{key}.html').read_bytes()).hexdigest()[:12]
    before, after = slides['before'][key], slides['after'][key]
    # 枚を外したり足したりすると番号がずれるので、変更前はラベルで対応させる
    by_label = {s['label']: s for s in before}
    payload[key] = dict(id=key, name=name,
                        before=f'proposal-review-20261026/before-{key}.html?v={version("before")}',
                        after=f'proposal-review-20261026/after-{key}.html?v={version("after")}',
                        notes=[note(s) for s in after], beforeNotes=[note(by_label.get(revision.get('renamed', {}).get(s['label'], s['label']), s)) for s in after], afterNotes=[note(s) for s in after],
                        reasons={str(s['number']): revision['reasons'].get(s['label'], '未修正です。') for s in after},
                        structureAppliedSlides=[s['number'] for s in after if s['label'] in revision['applied']],
                        structureApprovedSlides=[s['number'] for s in after if s['label'] in revision['approved']], addedSlides=[])
(HERE / 'comparison.json').write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n')
if (HERE / 'story-plan.md').exists():
    (PUBLIC / 'story-plan.html').write_text('<!doctype html><html lang="ja"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>企画書の構成見直し案</title><link rel="stylesheet" href="../review-navigation.css"><style>td:first-child{white-space:nowrap}</style><main>'
                                            + markdown.markdown((HERE / 'story-plan.md').read_text(), extensions=['tables']) + '</main></html>')
print('比較入力を生成:', {k: len(v) for k, v in slides['after'].items()}, '変更反映済み:', revision['applied'])
