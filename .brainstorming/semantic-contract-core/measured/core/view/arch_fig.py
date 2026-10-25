"""アーキテクチャの頁の4節の図（arch.source.svg → arch.svg）。class で書き、design-svg resolve を通す。
層を外から中心へ上から下に並べ、domain/ の crate は依存の深さで段に並べ、矢印は推移で届かない辺だけを真下へ引く（全部の辺は4節の表が持つ）。"""
import os, subprocess
H = os.path.dirname(os.path.abspath(__file__))
DS = os.path.join(H, '..', '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg')
def box(x, y, w, name, sub, kind=''):
    k = f' {kind}' if kind else ''
    return (f'<rect class="box{k}" x="{x}" y="{y}" width="{w}" height="50"/><text class="title{k}" x="{x+10}" y="{y+21}">{name}</text>'
            f'<text class="note small" x="{x+10}" y="{y+40}">{sub}</text>')
down = lambda x, y1, y2: f'<path class="flow" d="M{x},{y1} V{y2 - 2}"/>'
Y = 80  # domain の段を下げる量
band = lambda y, h, label: f'<rect class="area" x="10" y="{y}" width="940" height="{h}"/><text class="note small" x="20" y="{y + 16}">{label}</text>'
P = [
 '<svg viewBox="0 0 1000 720" style="min-width:900px" role="img" aria-label="crate の依存の向き">',
 '<text class="title" x="10" y="24">crate の依存の向き（矢印：依存する。どの矢印も上から下 ＝ 外から中心へ向かう）</text>',
 band(36, 76, '外側：入口（inbound）。outbound を application へ結びつける'), band(120, 76, '外側：外の技術（outbound）。application の ports/ を実装する'), band(204, 76, '使う場面とポート'), band(288, 382, '中心：業務（domain）。値を受け取り値を返す。入出力をしない ・ ポートを持たない'),
 box(250, 58, 400, 'crates/inbound/cli', 'コマンド：セマンティック7つ ・ 機械的 ・ compose', 'kind-3'),
 box(250, 142, 400, 'crates/outbound/*', 'agents-contract ・ ai-cli ・ git ・ fs ・ tools ・ sqlite', 'kind-1'),
 box(300, 226, 300, 'crates/application', '使う場面 ・ ports/（駆動される側13）', 'kind-2'),
 down(450, 108, 142),
 down(450, 192, 226),
 '<path class="flow" d="M650,83 H700 V251 H602"/><text class="label small" x="708" y="166">使う</text>',
 down(450, 276, 308), '<text class="label small" x="460" y="300">domain/ のすべてを使う</text>',
 box(40, 320, 160, 'validation', 'Validator'),
 box(240, 320, 160, 'render', 'Renderer'),
 box(760, 320, 140, 'session', 'どこにも依存しない'),
 box(240, 410, 160, 'search', '探す条件 ・ 出どころ'),
 box(480, 410, 160, 'resolver', 'Resolver'),
 box(140, 500, 160, 'change', '判定 ・ 変更の記録 ・ まとめ'),
 box(420, 500, 180, 'instance', 'インスタンス'),
 box(140, 590, 760, 'declaration', 'Abstract ・ Concrete ・ glossary ・ procedure（Abstract が決める値の型も）'),
 '<path class="link" d="M120,370 V390 M320,370 V390 M120,390 H560"/>',
 '<text class="label small" x="660" y="430">validation と render は、どちらも</text><text class="label small" x="660" y="448">resolver と change を使う</text>',
 down(200, 390, 500), down(560, 390, 410),
 down(280, 460, 500),
 '<path class="flow" d="M360,460 V480 H460 V498"/>',
 down(560, 460, 500),
 down(220, 550, 590), down(510, 550, 590),
 '<path class="flow" d="M975,40 V666"/><text class="label small" x="958" y="34">外</text><text class="label small" x="952" y="690">中心</text>',
 '<text class="note" x="10" y="708">矢印の先の先も直接使ってよい（例：validation → instance ・ declaration）。辺の全部は下の表が持つ</text>',
 '</svg>']
open(os.path.join(H, 'arch.source.svg'), 'w').write('\n'.join(P))
r = subprocess.run([DS, 'resolve', os.path.join(H, 'arch.source.svg'), os.path.join(H, 'arch.svg')], capture_output=True, text=True)
print([l for l in (r.stdout + r.stderr).split('\n') if '×' in l or '検査1' in l])
