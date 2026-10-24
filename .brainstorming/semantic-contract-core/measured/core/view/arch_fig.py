"""アーキテクチャの頁の4節の図（arch.source.svg → arch.svg）。class で書き、design-svg resolve を通す。
domain/ の crate は依存の深さで段に並べ、矢印は推移で届かない辺だけを真下へ引く（全部の辺は4節の表が持つ）。"""
import os, subprocess
H = os.path.dirname(os.path.abspath(__file__))
DS = os.path.join(H, '..', '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg')
def box(x, y, w, name, sub, kind=''):
    k = f' {kind}' if kind else ''
    return (f'<rect class="box{k}" x="{x}" y="{y}" width="{w}" height="50"/><text class="title{k}" x="{x+10}" y="{y+21}">{name}</text>'
            f'<text class="note small" x="{x+10}" y="{y+40}">{sub}</text>')
down = lambda x, y1, y2: f'<path class="flow" d="M{x},{y1} V{y2 - 2}"/>'
P = [
 '<svg viewBox="0 0 1000 640" style="min-width:900px" role="img" aria-label="crate の依存の向き">',
 '<text class="title" x="10" y="24">crate の依存の向き（矢印：依存する。上の段が下の段を使う）</text>',
 box(300, 40, 300, 'crates/inbound/cli', '起動と結びつけ（compose）', 'kind-3'),
 box(300, 120, 300, 'crates/application', '使う場面10 ・ ports/（駆動される側12）', 'kind-2'),
 box(700, 120, 260, 'crates/outbound/*', 'ports/ を実装する', 'kind-1'),
 down(450, 90, 120),
 '<path class="flow" d="M698,145 H602"/><text class="label small" x="635" y="138">実装</text>',
 '<path class="flow" d="M600,65 H830 V118"/><text class="label small" x="690" y="58">結びつける</text>',
 down(450, 170, 200),
 '<rect class="area" x="10" y="200" width="980" height="390"/><text class="note small" x="20" y="218">crates/domain/（値を受け取り値を返す。入出力をしない ・ ポートを持たない）</text>',
 box(40, 240, 160, 'render', '描画'),
 box(760, 240, 140, 'session', 'どこにも依存しない'),
 box(40, 330, 160, 'validation', '意味の検証'),
 box(240, 330, 160, 'search', '意味で探す'),
 box(140, 420, 160, 'change', '判定の記録 ・ 変更の記録'),
 box(420, 420, 140, 'instance', '作成と変更'),
 box(590, 420, 140, 'migration', '移す'),
 box(760, 420, 140, 'publish', '公開'),
 box(140, 510, 760, 'declaration', 'Abstract ・ Concrete ・ glossary（Abstract が決める値の型も）'),
 down(120, 290, 330), down(170, 380, 420), down(270, 380, 420),
 down(220, 470, 510), down(490, 470, 510), down(660, 470, 510), down(830, 470, 510),
 '<text class="note" x="10" y="612">矢印の先の先も直接使ってよい（render → change ・ declaration、validation ・ search → declaration）。辺の全部は下の表が持つ</text>',
 '<text class="note" x="10" y="630">application は domain/ のすべてに依存する</text>',
 '</svg>']
open(os.path.join(H, 'arch.source.svg'), 'w').write('\n'.join(P))
r = subprocess.run([DS, 'resolve', os.path.join(H, 'arch.source.svg'), os.path.join(H, 'arch.svg')], capture_output=True, text=True)
print([l for l in (r.stdout + r.stderr).split('\n') if '×' in l or '検査1' in l])
