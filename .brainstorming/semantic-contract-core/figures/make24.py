# 変更の記録の ER 図（第70回）：集約の境界（変更の記録と内に持つ値）、記録が指す対象、外が持つもの（Git などの変更の前後 ・ セッションの中身）。
import runpy
e = runpy.run_path('make13_table.py')['e']

def entity(o, x, y, w, name, attrs, cls="box focus"):
    h = 40 + 22 * len(attrs)
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/><text class="title" x="{x + 12}" y="{y + 24}">{e(name)}</text>')
    for k, a in enumerate(attrs):
        o.append(f'<text class="note" x="{x + 12}" y="{y + 48 + 22 * k}">{e(a)}</text>')
    return h

o = ['<svg viewBox="0 0 1000 570" style="min-width:900px" role="img" aria-label="変更の記録の ER 図">']
o.append('<rect class="boundary" x="16" y="16" width="664" height="276"/>')
o.append('<text class="note" x="30" y="276">集約の境界：変更の記録（1回の保存で変えるのはこの中だけ）</text>')
entity(o, 30, 40, 290, "変更の記録（集約）", ["id（打ち合わせずに一意な値）", "理由（なぜ変えたか）", "目的の参照（id）", "セッションの参照（値）", "変更の項目の並び（値）"])
entity(o, 380, 30, 280, "セッションの参照（値）", ["id", "根から親までの id の並び", "作業した者（呼ぶ側の申告）"], "box")
entity(o, 380, 160, 280, "変更の項目（値）", ["対象（種類と id）", "欄（JSON Pointer）", "項目ごとの理由（違うときだけ）"], "box")
entity(o, 730, 30, 250, "セッションの中身（外）", ["会話の記録。手元の AI のツールが持つ", "リポジトリに入れない"], "boundary")
entity(o, 730, 160, 250, "Git など（外）", ["ファイルの前後の値 ・ 履歴", "出した者 ・ 時刻 ・ 取り込み ・ 衝突"], "boundary")
entity(o, 30, 340, 250, "判定の記録", ["instance id ・ rule id", "判定した値 ・ 結果 ・ 根拠", "判定した者"], "box")
entity(o, 380, 340, 250, "instance（仕様の document）", ["id", "$schema（Concrete の $id）", "欄の値（JSON）"], "box")
entity(o, 720, 340, 260, "Semantic Concrete Schema", ["$id", "rule ・ term ・ 文脈の語彙"], "box")
o.append('<path class="flow" d="M320,80 H380"/><text class="note" x="326" y="72">1 ─ 1</text>')
o.append('<path class="flow" d="M320,175 H380"/><text class="note" x="326" y="167">1 ─ N</text>')
o.append('<path class="link" d="M730,75 H660"/><text class="note" x="692" y="67">id</text>')
o.append('<path class="flow" d="M730,210 H660"/><text class="note" x="690" y="202">前後</text>')
o.append('<path class="flow" d="M500,266 V340"/>')
o.append('<path class="flow" d="M420,266 V310 H155 V340"/>')
o.append('<path class="flow" d="M580,266 V310 H850 V340"/><text class="note" x="700" y="302">項目の対象（N ─ 1）</text>')
o.append('<path class="flow" d="M380,390 H280"/><text class="note" x="364" y="382">1</text><text class="note" x="288" y="382">N</text>')
o.append('<path class="flow" d="M720,390 H630"/><text class="note" x="705" y="382">1</text><text class="note" x="638" y="382">N</text>')
o.append('<text class="note" x="30" y="478">保存：正本は文脈のリポジトリのファイル（最初のアダプターは1件1ファイル）で、変更（PR）と一緒に共有する。DB は使わない。</text>')
o.append('<text class="note" x="30" y="500">まとまりは保存しない。セッション ・ 目的 ・ 対象を記録が持つ id で引き、出すときに組み立てる。</text>')
o.append('<text class="note" x="30" y="522">理由と変更の項目が合っているかは Core の rule で、変更を書いた AI と別のセッションの AI が判定する。</text>')
o.append('<text class="note" x="30" y="544">対象は仕様の document まで。ソースコードは追わない（開発のハーネス ・ UATDD の側が担保する）。</text>')
o.append('</svg>')
open('er-change.src.svg', 'w').write('\n'.join(o))
