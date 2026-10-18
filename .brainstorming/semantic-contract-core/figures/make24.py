# 変更の記録の ER 図（第75回。関係の索引と instance の kind ・ x-ref を足した）：セッション（手元の索引と、外の管理する役）、集約の境界（変更の記録と変更の項目）、記録が指す対象、外が持つ Git などの変更の前後。
import runpy
e = runpy.run_path('make13_table.py')['e']

def entity(o, x, y, w, name, attrs, cls="box focus"):
    h = 40 + 22 * len(attrs)
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/><text class="title" x="{x + 12}" y="{y + 24}">{e(name)}</text>')
    for k, a in enumerate(attrs):
        o.append(f'<text class="note" x="{x + 12}" y="{y + 48 + 22 * k}">{e(a)}</text>')
    return h

o = ['<svg viewBox="0 0 1000 860" style="min-width:900px" role="img" aria-label="変更の記録とセッションの ER 図">']
entity(o, 30, 20, 290, "手元のセッションの索引", ["セッション ID ・ 根から親までの id の並び", "変更の記録の id の並び", "SQLite 1ファイル（最初のアダプター）", "コミットしない"], "box")
entity(o, 380, 20, 280, "セッションを管理する役（外）", ["harness ・ AI のツール", "ID の発行 ・ 始まりと終わり", "子のセッションへの受け渡し"], "boundary")
entity(o, 730, 20, 250, "セッションの中身（外）", ["会話の記録。手元の AI のツール", "リポジトリに入れない"], "boundary")
o.append('<path class="flow" d="M380,70 H320"/><text class="note" x="334" y="62">渡す</text>')
o.append('<path class="link" d="M730,60 H660"/><text class="note" x="692" y="52">id</text>')
o.append('<rect class="boundary" x="16" y="186" width="664" height="196"/>')
o.append('<text class="note" x="30" y="372">集約の境界：変更の記録（1回の保存で変えるのはこの中だけ）</text>')
entity(o, 30, 206, 290, "変更の記録（集約）", ["id（書いた時に決める一意な値）", "理由（なぜ変えたか）", "目的の参照（id）", "書いた者の種類（人 ・ AI）", "変更の項目の並び（値）"])
entity(o, 380, 226, 280, "変更の項目（値）", ["対象（種類と id）", "欄（JSON Pointer）", "項目ごとの理由（違うときだけ）"], "box")
entity(o, 730, 226, 250, "Git など（外）", ["ファイルの前後の値 ・ 履歴", "出した者 ・ 時刻 ・ 取り込み ・ 衝突"], "boundary")
o.append('<path class="flow" d="M175,148 V206"/><text class="note" x="186" y="172">記録の id で指す</text>')
o.append('<path class="flow" d="M320,276 H380"/><text class="note" x="326" y="268">1 ─ N</text>')
o.append('<path class="flow" d="M730,276 H660"/><text class="note" x="690" y="268">前後</text>')
o.append('<path class="flow" d="M500,332 V420"/>')
o.append('<path class="flow" d="M420,332 V400 H155 V420"/>')
o.append('<path class="flow" d="M580,332 V400 H850 V420"/><text class="note" x="700" y="392">項目の対象（N ─ 1）</text>')
entity(o, 30, 420, 250, "判定の記録", ["instance id ・ rule id", "判定した値 ・ 結果 ・ 根拠", "判定した者"], "box")
entity(o, 380, 420, 250, "instance（仕様の document）", ["id ・ kind（範囲は Concrete の $id）", "$schema（Concrete の $id）", "欄の値（JSON）", "x-ref（指す先の instance ・ 版）"], "box")
entity(o, 380, 620, 280, "関係の索引（手元の DB）", ["instance id ・ kind", "x-ref：指す側 → 指される側（版）", "正本のファイルから作り直せる", "コミットしない"], "box")
o.append('<path class="flow" d="M500,548 V620"/><text class="note" x="512" y="590">正本から作る</text>')
entity(o, 720, 420, 260, "Semantic Concrete Schema", ["$id", "rule ・ term ・ 文脈の語彙"], "box")
o.append('<path class="flow" d="M380,470 H280"/><text class="note" x="364" y="462">1</text><text class="note" x="288" y="462">N</text>')
o.append('<path class="flow" d="M720,470 H630"/><text class="note" x="705" y="462">1</text><text class="note" x="638" y="462">N</text>')
o.append('<text class="note" x="30" y="772">コミットするのは、集約の境界の中（JSON のファイル。1件1ファイル）と対象だけ。セッション ID は手元の索引にだけ置く。</text>')
o.append('<text class="note" x="30" y="794">手元の DB（SQLite）：手元のセッションの索引と関係の索引。DB のスキーマは Core が持ち、Concrete は宣言しない。コミットしない。</text>')
o.append('<text class="note" x="30" y="816">変更の記録は、書いた時にセッション ID の無い形で作業ツリーに置く。まとまりは保存せず、id で引いて組み立てる。</text>')
o.append('<text class="note" x="30" y="838">対象は仕様の document まで。ソースコードは追わない（開発のハーネス ・ UATDD の側が担保する）。</text>')
o.append('</svg>')
open('er-change.src.svg', 'w').write('\n'.join(o))
