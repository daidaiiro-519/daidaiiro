"""指定した生成元から企画書の詳細版と要点版を組み、HTMLと各枚の文面を出力先へ書く。

使い方: python3 render_deck.py <生成元のsrc> <出力先>
正本の out/ へは書かない。書籍の表紙は data URI にせず、閲覧ページの /workshop/books/ を参照する。
"""
from pathlib import Path
import json
import re
import sys

src, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(src))
# theme.py は正本の位置からリポジトリの根を数えるので、作業コピーでは同じ値を根から読んで差し込む
import types
ROOT = next(p for p in Path(__file__).resolve().parents if (p / 'CLAUDE.md').exists())
theme = types.ModuleType('theme')
theme.T = dict(re.findall(r'--([a-z-]+)\s*:\s*([^;]+);', (ROOT / '.claude/skills/slide-deck/references/themes/warm-paper.css').read_text()))
sys.modules['theme'] = theme
import visuals
LINE = visuals.LINE
visuals.cover = lambda x, y, w, h, isbn: (
    f'<image x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="xMidYMid meet" href="/workshop/books/{isbn}.jpg"/>'
    + f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke="{LINE}" stroke-width="1"/>')
import build_deck
build_deck.OUT = out


def plain(s):
    return re.sub(r'<[^>]*>', '', s)


result = {}
for key, order, stem, notes_name, title, index in [
        ('detail', build_deck.ORDER, 'detail', 'detail-notes.md', '下期AI活用ワークショップ', build_deck.APPENDIX_INDEX),
        ('brief', build_deck.ORDER_BRIEF, 'brief', 'brief-notes.md', '下期AI活用ワークショップ（要点版）', build_deck.APPENDIX_BRIEF)]:
    build_deck.build(order, stem=stem, notes_name=notes_name, title=title, apx_from=order.index(index) + 1)
    result[key] = [dict(number=n, label=s['label'], heading=plain(s['title']), lede=s['intro'], narration=s['notes'])
                   for n, s in enumerate(build_deck._deck(order), 1)]
(out / 'slides.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
