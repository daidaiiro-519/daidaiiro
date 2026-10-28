"""作業コピーの生成元から、正本と同じ名前・同じ形（書籍の表紙は data URI）で out/ を作る。正本の proposal/out/ へは書かない。"""
from pathlib import Path
import re, sys, types
HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'CLAUDE.md').exists())
SRC = HERE / 'after/proposal/src'
OUT = HERE / 'after/proposal/out'
OUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(SRC))
theme = types.ModuleType('theme')
theme.T = dict(re.findall(r'--([a-z-]+)\s*:\s*([^;]+);', (ROOT / '.claude/skills/slide-deck/references/themes/warm-paper.css').read_text()))
sys.modules['theme'] = theme
import visuals
_cover = visuals.cover
visuals.cover = lambda x, y, w, h, isbn: _cover.__wrapped__(x, y, w, h, isbn) if hasattr(_cover, '__wrapped__') else (
    __import__('base64') and (lambda b: f'<image x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="xMidYMid meet" href="data:image/jpeg;base64,{b}"/>'
    + f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke="{visuals.LINE}" stroke-width="1"/>')(__import__('base64').b64encode((ROOT / 'docs/workshops/ai-workshop/books' / f'{isbn}.jpg').read_bytes()).decode()))
import build_deck
build_deck.OUT = OUT
build_deck.build(build_deck.ORDER, apx_from=build_deck.ORDER.index(build_deck.APPENDIX_INDEX) + 1)
build_deck.build(build_deck.ORDER_BRIEF, stem='director-deck-brief', notes_name='speaker-notes-brief.md',
                 title='下期AI活用ワークショップ（要点版）', apx_from=build_deck.ORDER_BRIEF.index(build_deck.APPENDIX_BRIEF) + 1)
