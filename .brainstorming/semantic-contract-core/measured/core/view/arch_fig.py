"""アーキテクチャの頁の4節の図（arch.source.svg → arch.svg）。class で書き、design-svg resolve を通す。
domain/ の crate の配置は arch/ports/crates.source.svg と同じで、上に inbound ・ application、右に outbound を置く。"""
import os, re, subprocess
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
DS = os.path.join(C, '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg')
src = open(os.path.join(C, 'arch', 'ports', 'crates.source.svg')).read()
body = src[src.index('>', src.index('<svg')) + 1: src.rindex('</svg>')]
lines = [l for l in body.strip().split('\n') if 'class="title" x="10" y="24"' not in l and 'y="455"' not in l]
DY = 190
def shift(l):
    l = re.sub(r'\by="(\d+)"', lambda m: f'y="{int(m.group(1)) + DY}"', l)
    def d(m):
        out, toks = [], re.findall(r'[MLHV]|-?\d+', m.group(1)); cmd = None; xy = 0
        for t in toks:
            if t in 'MLHV': cmd = t; out.append(t); xy = 0; continue
            v = int(t)
            if cmd == 'V' or (cmd in 'ML' and xy % 2 == 1): v += DY
            if cmd in 'ML': xy += 1
            out.append(str(v))
        s = ''.join(o if o in 'MLHV' else (o if prev in 'MLHV' else ',' + o) for prev, o in zip(['M'] + out, out))
        return f'd="{s}"'
    return re.sub(r'd="([^"]+)"', d, l)
inner = '\n'.join(shift(l) for l in lines)
svg = f'''<svg viewBox="0 0 1000 670" style="min-width:900px" role="img" aria-label="crate の依存の向き">
<text class="title" x="10" y="24">crate の依存の向き（矢印：依存する ・ 外から中へ）</text>
<rect class="box kind-3" x="300" y="40" width="300" height="50"/><text class="title kind-3" x="310" y="61">crates/inbound/cli</text><text class="note small" x="310" y="80">起動と結びつけ（compose）</text>
<rect class="box kind-2" x="300" y="120" width="300" height="50"/><text class="title kind-2" x="310" y="141">crates/application</text><text class="note small" x="310" y="160">使う場面10 ・ ports/（駆動される側12）</text>
<rect class="box kind-1" x="720" y="120" width="270" height="50"/><text class="title kind-1" x="730" y="141">crates/outbound/*</text><text class="note small" x="730" y="160">ports/ を実装する</text>
<path class="flow" d="M450,90 V118"/>
<path class="flow" d="M718,145 H602"/><text class="label small" x="640" y="138">実装</text>
<path class="flow" d="M600,65 H855 V118"/><text class="label small" x="700" y="58">結びつける</text>
<path class="flow" d="M450,170 V198"/>
<rect class="area" x="10" y="200" width="985" height="420"/><text class="note small" x="20" y="216">crates/domain/（値を受け取り値を返す。入出力をしない ・ ポートを持たない）</text>
{inner}
<text class="note" x="10" y="650">application は domain/ のすべてに依存する。矢印の先の先を直接使う辺は表に書く。session はどこにも依存しない</text>
</svg>'''
open(os.path.join(H, 'arch.source.svg'), 'w').write(svg)
r = subprocess.run([DS, 'resolve', os.path.join(H, 'arch.source.svg'), os.path.join(H, 'arch.svg')], capture_output=True, text=True)
print([l for l in (r.stdout + r.stderr).split('\n') if '×' in l or '検査1' in l])
