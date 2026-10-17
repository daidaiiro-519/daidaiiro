# 第5回の図：標準の概念と写し（Q2）、protocol と4つの口（Q4）
def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
def lines(o,x,y,ls,cls="label",step=18):
    for i,l in enumerate(ls): o.append(f'<text class="{cls}" x="{x}" y="{y+i*step}">{e(l)}</text>')
rows=[("JSON Schema",["構造と値の制約"],["抽象の仕様の土台（いまもある）"]),
      ("JSON-LD",["共通の識別子 ・ 名前の対応付け","・ 再定義の保護"],["Vocabulary Spec","Semantic Model"]),
      ("SHACL",["ノードと関係の制約 ・ 経路","・ 閉じる／開く ・ 結果の報告"],["Constraint Spec","結果の形"]),
      ("RDF ・ SPARQL",["インスタンスをまたいだ関係の","グラフと問い合わせ"],["Relation / Graph"])]
o=['<svg viewBox="0 0 780 420" style="min-width:660px" role="img" aria-label="標準の概念を自前の概念として定義し、標準へは写して相互運用する">']
o.append('<text class="note" x="20" y="22">標準</text><text class="note" x="196" y="22">取り込む概念</text><text class="note" x="486" y="22">自前の抽象の仕様のどこに定義するか</text>')
for i,(n,c,d) in enumerate(rows):
    y=34+i*68
    o.append(f'<rect class="boundary" x="20" y="{y}" width="160" height="56"/><text class="title" x="34" y="{y+34}">{e(n)}</text>')
    o.append(f'<path class="flow" d="M182,{y+28} H192"/>')
    o.append(f'<rect class="box" x="196" y="{y}" width="270" height="56"/>'); lines(o,208,y+24 if len(c)>1 else y+34,c)
    o.append(f'<path class="flow" d="M468,{y+28} H482"/>')
    o.append(f'<rect class="box focus" x="486" y="{y}" width="274" height="56"/>'); lines(o,498,y+24 if len(d)>1 else y+34,d)
o.append('<rect class="box warn" x="20" y="312" width="740" height="92"/>')
o.append('<text class="title" x="34" y="336">写し（Projection）も基盤の概念</text>')
o.append('<text class="label" x="34" y="360">自前の概念から標準へ写して、ほかのシステムとやりとりする</text>')
o.append('<text class="label" x="34" y="384">写せた ・ 写せなかった ・ 扱えないを返し、必須の意味が写せなければ合格にしない</text>')
o.append('</svg>'); W['q2-concepts']='\n'.join(o)

o=['<svg viewBox="0 0 760 330" style="min-width:640px" role="img" aria-label="1つの protocol を、4つの口が同じ形で運ぶ">']
ports=[("CLI（標準入出力）","いまある"),("MCP","いまある"),("HTTP","足す"),("WASM Component","足す")]
for i,(n,st) in enumerate(ports):
    x=20+i*186
    k="box" if st=="いまある" else "box warn"
    o.append(f'<rect class="{k}" x="{x}" y="20" width="172" height="62"/><text class="title" x="{x+12}" y="{46}">{e(n)}</text><text class="note" x="{x+12}" y="68">{st}</text>')
    o.append(f'<path class="flow" d="M{x+86},84 V122"/>')
o.append('<rect class="box focus" x="20" y="126" width="730" height="74"/>')
o.append('<text class="title" x="34" y="152">protocol（JSON Schema で公開）</text>')
o.append('<text class="label" x="34" y="178">要求と応答の形 ・ 版 ・ 能力の表明 ・ 結果の5つの語。どの口からでも同じ形で返す</text>')
o.append('<path class="flow" d="M385,202 V236"/>')
o.append('<rect class="box" x="20" y="240" width="730" height="74"/>')
o.append('<text class="title" x="34" y="266">エンジン（Rust）</text>')
o.append('<text class="label" x="34" y="292">適合の試験は入力と期待結果の JSON の組。どの口とどの言語の実装も同じ試験を通す</text>')
o.append('</svg>'); W['q4-protocol']='\n'.join(o)
for k,v in W.items(): open(k+'.src.svg','w').write(v)
