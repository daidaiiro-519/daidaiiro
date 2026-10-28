"""参加への導入。架空の仕事場面と、教材で扱う方法を描く。"""
import json
from pathlib import Path
from lesson_visuals import text, rect, svg, INK, DIM, LINE, PAPER, ACCENT
SCENES = json.loads(Path(__file__).with_name('orientation-scenes.json').read_text())

def scene(sid):
    d = SCENES[sid]
    cards = d['cards']
    gap = 24
    width = (1112 - gap * (len(cards) - 1)) / len(cards)
    a = ''
    for i, (heading, first, second) in enumerate(cards):
        x = i * (width + gap)
        a += rect(x, 24, width, 218, PAPER, 12, LINE)
        a += text(x + 24, 66, heading, 23, INK, 700)
        a += text(x + 24, 129, first, 21, INK)
        a += text(x + 24, 173, second, 21, DIM)
    a += text(0, 305, d['closing'], 23, ACCENT, 700)
    return svg('。'.join('、'.join(c) for c in cards) + '。' + d['closing'], a, 334)
