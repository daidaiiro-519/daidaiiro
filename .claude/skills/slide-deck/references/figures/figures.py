"""スライドに置く線画を組むための語彙。Python 3 の標準ライブラリだけで動く。

色は持たない。テーマの CSS から読む ── 色の正本は `../themes/*.css` の1か所だけである。

    from figures import Palette, Figure
    f = Figure(Palette.from_theme('../themes/warm-paper.css'))
    html = f.svg('打ち合わせの流れ', f.rect(0, 0, 200, 90) + f.text(24, 40, '画面を見せる'))
"""
import re
from html import escape

# 図が使う役割と、テーマのどのトークンから取るか。
ROLES = {
    'ink': '--ink',                 # 図の中の見出し・強い文字
    'dim': '--dim',                 # 図の中の補足
    'line': '--diagram-line',       # 図の線。本文の文字色とは分ける
    'panel': '--surface',           # 面
    'paper': '--paper',             # 明るい面
    'on_paper': '--on-paper',       # 明るい面の上に置く文字
    'on_paper_dim': '--on-paper-dim',  # 明るい面の上に置く補足
    'accent': '--accent',           # 強調。1つの図に1か所
    'on_accent': '--on-accent',     # 強調面の上に置く文字
}


class Palette:
    """テーマ CSS の :root から、図が使う色だけを取り出したもの。"""

    def __init__(self, **colors):
        missing = [k for k in ROLES if k not in colors]
        if missing:
            raise ValueError(f'色が足りない: {", ".join(missing)}')
        self.__dict__.update(colors)

    @classmethod
    def from_theme(cls, path):
        """`../themes/<name>.css` を読んで組み立てる。"""
        text = open(path, encoding='utf-8').read()
        found = dict(re.findall(r'(--[a-z0-9-]+)\s*:\s*([^;]+);', text))
        colors = {}
        for role, token in ROLES.items():
            if token not in found:
                raise ValueError(f'{path} に {token} が無い')
            colors[role] = found[token].strip()
        return cls(**colors)


class Figure:
    """1つのテーマに固定された、図の部品一式。

    既定値は線幅 2、角丸 12、線端は丸め。枚をまたいで同じ形になるように、
    呼ぶ側では指定しない。
    """

    WIDTH = 1112   # 本文領域の幅。図はこれに合わせる

    def __init__(self, palette, width=WIDTH):
        self.p = palette
        self.width = width

    # ── 文字 ──────────────────────────────────────────────
    def text(self, x, y, lines, size=22, color=None, weight=400,
             anchor='start', gap=None):
        """1行でも、行の並びでも受ける。行間は文字の大きさから決まる。"""
        if isinstance(lines, str):
            lines = [lines]
        color = color or self.p.ink
        gap = gap or size * 1.55
        return ''.join(
            f'<text x="{x}" y="{y + i * gap}" font-size="{size}" fill="{color}"'
            f' font-weight="{weight}" text-anchor="{anchor}">{escape(s)}</text>'
            for i, s in enumerate(lines))

    # ── 面と線 ────────────────────────────────────────────
    def rect(self, x, y, w, h, fill=None, r=12, stroke='none'):
        fill = self.p.panel if fill is None else fill
        return (f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}"'
                f' fill="{fill}" stroke="{stroke}" stroke-width="2"/>')

    def path(self, d, color=None, width=2, fill='none'):
        color = color or self.p.line
        return (f'<path d="{d}" stroke="{color}" stroke-width="{width}"'
                f' fill="{fill}" stroke-linecap="round" stroke-linejoin="round"/>')

    def circle(self, x, y, r, fill=None, stroke=None):
        return (f'<circle cx="{x}" cy="{y}" r="{r}"'
                f' fill="{fill or self.p.paper}" stroke="{stroke or self.p.line}"'
                f' stroke-width="2"/>')

    def arrow(self, x1, y1, x2, y2, color=None):
        """横向きの矢印。矢じりは向きに合わせて付け替わる。"""
        tip = (f'M{x2 - 8} {y2 - 7} l8 7 -8 7' if x2 > x1
               else f'M{x2 + 8} {y2 - 7} l-8 7 8 7')
        return self.path(f'M{x1} {y1} H{x2} ' + tip, color)

    # ── 絵 ────────────────────────────────────────────────
    def person(self, x, y, scale=1, color=None):
        color = color or self.p.ink
        return (f'<g transform="translate({x} {y}) scale({scale})">'
                + self.circle(0, -14, 12, self.p.paper, color)
                + self.path('M-24 28 v-6 c0 -27 48 -27 48 0 v6', color, 2.5)
                + '</g>')

    ICONS = {
        'screen':   'M0 0 H56 V36 H0 Z M28 36 V47 M13 48 H43',
        'doc':      'M3 0 H33 L46 13 V55 H3 Z M33 0 V13 H46 M12 25 H35 M12 35 H35 M12 45 H28',
        'chat':     'M0 0 H54 V33 H23 L10 44 V33 H0 Z M12 11 H42 M12 22 H31',
        'check':    'M0 20 L13 33 L40 0',
        'book':     'M0 3 Q14 -3 27 3 Q40 -3 54 3 V44 Q40 38 27 44 Q14 38 0 44 Z M27 3 V44',
        'search':   'M35 17 a17 17 0 1 1 -34 0 a17 17 0 1 1 34 0 M30 30 L49 49',
        'calendar': 'M0 7 H50 V48 H0 Z M0 19 H50 M13 0 V12 M37 0 V12 M10 29 H19 M28 29 H39 M10 39 H19',
        'branch':   'M0 8 H20 V39 H49 M20 8 H49 M20 24 H49',
    }

    def icon(self, x, y, kind, color=None, scale=1):
        if kind not in self.ICONS:
            raise ValueError(
                f'{kind} という絵は無い。あるのは: {", ".join(sorted(self.ICONS))}')
        return (f'<g transform="translate({x} {y}) scale({scale})">'
                + self.path(self.ICONS[kind], color or self.p.ink, 2.3) + '</g>')

    # ── 器 ────────────────────────────────────────────────
    def on(self, fill):
        """その面の上に置く文字の色を返す。

        `paper` を敷いて `ink` を載せると、地が暗いテーマで文字が消える
        （`--ink` は地に対する本文色であって、明るい面に対する色ではない）。
        面を選んだら、文字色はこれで参照する。
        """
        return {self.p.paper: self.p.on_paper,
                self.p.accent: self.p.on_accent}.get(fill, self.p.ink)

    def svg(self, label, body, height=330):
        """図1つを包む。幅は本文領域に合わせて固定する。

        label は読み上げ用。何を示す図かを1文で書く。
        """
        if not label:
            raise ValueError('label が要る。何を示す図かを1文で書く')
        return (f'<svg class="figure" viewBox="0 0 {self.width} {height}"'
                f' role="img" aria-label="{escape(label)}">{body}</svg>')
