# 変更の記録の ER 図（案）：セッション ・ 変更 ・ 変更の項目と、対象（Concrete ・ instance ・ 判定の記録）の関係。
import runpy
e = runpy.run_path('make13_table.py')['e']

def entity(o, x, y, w, name, attrs, cls="box focus"):
    h = 40 + 22 * len(attrs)
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/><text class="title" x="{x + 12}" y="{y + 24}">{e(name)}</text>')
    for k, a in enumerate(attrs):
        o.append(f'<text class="note" x="{x + 12}" y="{y + 48 + 22 * k}">{e(a)}</text>')
    return h

o = ['<svg viewBox="0 0 1000 660" style="min-width:900px" role="img" aria-label="変更の記録の ER 図（案）">']
entity(o, 30, 20, 250, "セッション", ["id", "親セッション id（0..1）", "作業した者（人 ・ AI の Agent）", "始めた時刻 ・ 終えた時刻", "目的"])
entity(o, 375, 20, 250, "変更", ["id", "セッション id", "理由（なぜ変えたか）", "比べた案", "出した時刻"])
entity(o, 720, 20, 260, "変更の項目", ["id", "変更 id", "対象（種類と id）", "欄（JSON Pointer）", "変更前の値 ・ 変更後の値", "操作（足す ・ 変える ・ 消す）", "理由（項目ごと）"])
entity(o, 30, 430, 250, "判定の記録", ["instance id ・ rule id", "判定した値 ・ 結果 ・ 根拠", "判定した者", "変更 id"], "box")
entity(o, 375, 430, 250, "instance", ["id", "$schema（Concrete の $id）", "欄の値"], "box")
entity(o, 720, 430, 260, "Semantic Concrete Schema", ["$id", "rule ・ term ・ 文脈の語彙"], "box")
# セッション 1 ─ N 変更、変更 1 ─ N 変更の項目
o.append('<path class="flow" d="M280,90 H375"/><text class="note" x="288" y="82">1</text><text class="note" x="360" y="82">N</text>')
o.append('<path class="flow" d="M625,90 H720"/><text class="note" x="633" y="82">1</text><text class="note" x="705" y="82">N</text>')
# セッションの親子（同じセッションを指す）
o.append('<path class="flow" d="M30,60 H12 V200 H120 V170"/><text class="note" x="130" y="196">親 1 ─ 子 N</text>')
# 変更の項目の対象：instance と Concrete（N ─ 1）
o.append('<path class="flow" d="M850,214 V430"/><path class="flow" d="M760,214 V320 H500 V430"/><text class="note" x="560" y="312">項目の対象（N ─ 1）</text>')
# 判定の記録は変更に入る（変更 1 ─ N 判定の記録）
o.append('<path class="flow" d="M420,170 V260 H155 V430"/><text class="note" x="165" y="252">判定の記録は変更に入る（1 ─ N）</text>')
# instance 1 ─ N 判定の記録、Concrete 1 ─ N instance
o.append('<path class="flow" d="M375,480 H280"/><text class="note" x="360" y="472">1</text><text class="note" x="288" y="472">N</text>')
o.append('<path class="flow" d="M720,480 H625"/><text class="note" x="705" y="472">1</text><text class="note" x="633" y="472">N</text>')
o.append('<text class="note" x="30" y="620">セッションごとに、そのセッションと子のセッションの変更 ・ 項目 ・ 理由を引ける。対象ごとに、その instance ・ Concrete の経緯を引ける。</text>')
o.append('<text class="note" x="30" y="642">ファイルの保存と履歴は Git などのアダプターが持つ。変更の記録は意味の単位（欄）で、理由と一緒に基盤が持つ。</text>')
o.append('</svg>')
open('er-change.src.svg', 'w').write('\n'.join(o))
