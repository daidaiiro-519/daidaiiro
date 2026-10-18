# 変更の記録の ER 図（案）：セッション ・ 変更 ・ 変更の項目と、対象（Concrete ・ instance ・ 判定の記録）の関係。
import runpy
e = runpy.run_path('make13_table.py')['e']

def entity(o, x, y, w, name, attrs, cls="box focus"):
    h = 40 + 22 * len(attrs)
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/><text class="title" x="{x + 12}" y="{y + 24}">{e(name)}</text>')
    for k, a in enumerate(attrs):
        o.append(f'<text class="note" x="{x + 12}" y="{y + 48 + 22 * k}">{e(a)}</text>')
    return h

o = ['<svg viewBox="0 0 1000 760" style="min-width:900px" role="img" aria-label="変更の記録の ER 図（案）">']
entity(o, 30, 20, 250, "変更のまとまり", ["id", "目的", "まとめ方（セッション ・ 目的 ・ 対象）", "作った者（harness など）"])
entity(o, 375, 20, 250, "セッション", ["id", "親セッション id（0..1）", "作業した者（人 ・ AI の Agent）", "始めた時刻 ・ 終えた時刻", "目的"])
entity(o, 30, 230, 250, "変更", ["id", "セッション id", "理由（なぜ変えたか）", "比べた案", "出した時刻"])
entity(o, 375, 230, 250, "変更の項目", ["id", "変更 id", "対象（種類と id）", "欄（JSON Pointer）", "変更前の値 ・ 変更後の値", "操作（足す ・ 変える ・ 消す）", "理由（項目ごと）"])
entity(o, 30, 540, 250, "判定の記録", ["instance id ・ rule id", "判定した値 ・ 結果 ・ 根拠", "判定した者", "変更 id"], "box")
entity(o, 375, 540, 250, "instance（仕様の document）", ["id", "$schema（Concrete の $id）", "欄の値（JSON）"], "box")
entity(o, 720, 540, 260, "Semantic Concrete Schema", ["$id", "rule ・ term ・ 文脈の語彙"], "box")
# まとまり N ─ N 変更
o.append('<path class="flow" d="M155,148 V230"/><text class="note" x="165" y="192">まとまり N ─ N 変更</text>')
# セッション 1 ─ N 変更
o.append('<path class="flow" d="M500,170 V200 H230 V230"/><text class="note" x="300" y="214">セッション 1 ─ N 変更</text>')
# セッションの親子
o.append('<path class="flow" d="M625,60 H660 V200 H560 V170"/><text class="note" x="668" y="130">親 1 ─ 子 N</text>')
# 変更 1 ─ N 変更の項目
o.append('<path class="flow" d="M280,300 H375"/><text class="note" x="288" y="292">1</text><text class="note" x="360" y="292">N</text>')
# 変更の項目の対象：instance と Concrete
o.append('<path class="flow" d="M500,424 V540"/><path class="flow" d="M560,424 V480 H850 V540"/><text class="note" x="620" y="472">項目の対象（N ─ 1）</text>')
# 判定の記録は変更に入る
o.append('<path class="flow" d="M155,380 V540"/><text class="note" x="165" y="470">判定の記録は変更に入る（1 ─ N）</text>')
# instance 1 ─ N 判定の記録、Concrete 1 ─ N instance
o.append('<path class="flow" d="M375,590 H280"/><text class="note" x="360" y="582">1</text><text class="note" x="288" y="582">N</text>')
o.append('<path class="flow" d="M720,590 H625"/><text class="note" x="705" y="582">1</text><text class="note" x="633" y="582">N</text>')
o.append('<text class="note" x="30" y="706">まとまりごとに、対象ごとの意味の差分 ・ 理由 ・ 変更後の検証の結果 ・ 人が見るものを出せる。PR は、まとまりの使い方の1つ（基盤は PR を知らない）。</text>')
o.append('<text class="note" x="30" y="728">対象は仕様の document（Concrete の instance）まで。ソースコードは追わない（開発のハーネス ・ UATDD の側が担保する）。</text>')
o.append('<text class="note" x="30" y="750">ファイルの保存と履歴は Git などのアダプターが持つ。変更の記録は意味の単位（欄）で、理由と一緒に基盤が持つ。</text>')
o.append('</svg>')
open('er-change.src.svg', 'w').write('\n'.join(o))
