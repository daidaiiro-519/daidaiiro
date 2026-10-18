# 変更の記録の ER 図（第69回）：集約「変更の記録」と、内に持つ値オブジェクト、対象（Concrete ・ instance ・ 判定の記録）、外が持つ変更の前後。
import runpy
e = runpy.run_path('make13_table.py')['e']

def entity(o, x, y, w, name, attrs, cls="box focus"):
    h = 40 + 22 * len(attrs)
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/><text class="title" x="{x + 12}" y="{y + 24}">{e(name)}</text>')
    for k, a in enumerate(attrs):
        o.append(f'<text class="note" x="{x + 12}" y="{y + 48 + 22 * k}">{e(a)}</text>')
    return h

o = ['<svg viewBox="0 0 1000 590" style="min-width:900px" role="img" aria-label="変更の記録の ER 図">']
entity(o, 30, 20, 290, "変更の記録（集約）", ["id", "理由（なぜ変えたか）", "比べた案", "出した時刻", "セッションの参照（値）", "変更の項目の並び（値）"])
entity(o, 380, 20, 280, "セッションの参照（値）", ["id ・ 親の id（0..1）", "作業した者（呼ぶ側の申告）", "セッションの開始 ・ 終了は持たない"], "box")
entity(o, 380, 150, 280, "変更の項目（値）", ["対象（種類と id）", "欄（JSON Pointer）", "項目ごとの理由（違うときだけ）"], "box")
entity(o, 760, 150, 220, "変更の前後（Git など）", ["ファイルの前後の値 ・ 履歴", "出した者 ・ 取り込み ・ 衝突"], "box")
entity(o, 30, 360, 250, "判定の記録", ["instance id ・ rule id", "判定した値 ・ 結果 ・ 根拠", "判定した者"], "box")
entity(o, 380, 360, 250, "instance（仕様の document）", ["id", "$schema（Concrete の $id）", "欄の値（JSON）"], "box")
entity(o, 720, 360, 260, "Semantic Concrete Schema", ["$id", "rule ・ term ・ 文脈の語彙"], "box")
o.append('<path class="flow" d="M320,60 H380"/><text class="note" x="326" y="52">1 ─ 1</text>')
o.append('<path class="flow" d="M320,175 H380"/><text class="note" x="326" y="167">1 ─ N</text>')
o.append('<path class="flow" d="M760,200 H660"/><text class="note" x="668" y="192">前後の値を導く</text>')
o.append('<path class="flow" d="M500,256 V360"/>')
o.append('<path class="flow" d="M420,256 V310 H155 V360"/>')
o.append('<path class="flow" d="M580,256 V310 H850 V360"/><text class="note" x="600" y="302">項目の対象（N ─ 1）</text>')
o.append('<path class="flow" d="M380,410 H280"/><text class="note" x="364" y="402">1</text><text class="note" x="288" y="402">N</text>')
o.append('<path class="flow" d="M720,410 H630"/><text class="note" x="705" y="402">1</text><text class="note" x="638" y="402">N</text>')
o.append('<text class="note" x="30" y="510">まとまりは保存しない。出すときに、まとめ方（セッション ・ 目的 ・ 対象）に紐づく変更の記録を取り出して組み立てる。</text>')
o.append('<text class="note" x="30" y="532">理由と変更の項目が合っているかは Core の rule で、変更を書いた AI と別のセッションの AI が判定する。</text>')
o.append('<text class="note" x="30" y="554">対象は仕様の document まで。ソースコードは追わない（開発のハーネス ・ UATDD の側が担保する）。</text>')
o.append('</svg>')
open('er-change.src.svg', 'w').write('\n'.join(o))
