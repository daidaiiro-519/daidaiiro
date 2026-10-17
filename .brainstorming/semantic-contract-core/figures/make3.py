# 第4回の図。class だけで書き、design-svg resolve で値へ解く。行は手で分ける（単語の途中で折らない）。
def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
def lines(o,x,y,ls,cls="label",step=18):
    for i,l in enumerate(ls): o.append(f'<text class="{cls}" x="{x}" y="{y+i*step}">{e(l)}</text>')
# Q0：5つの設計原則と、いまの基盤
rows=[(["1 宣言と解釈の分離"],["宣言（スキーマ ・ 注釈）と","確かめ（エンジン）は分かれている"],["確かめる担い手を","外から足す仕組み"]),
      (["2 既存技術は部品"],["JSON Schema を","エンジンに組み込んでいる"],["JSON-LD ・ SHACL ・ RDF の","概念を自前で定義して写すこと"]),
      (["3 言語非依存の","意味モデル"],["Rust で作り、","CLI ・ MCP ・ JSON で返す"],["出力の形を","JSON Schema で公開すること"]),
      (["4 拡張可能な","表現モデル"],["x-annotations で","注釈を足せる"],["足した注釈を確かめ、","確かめなければ返すこと"]),
      (["5 検証可能で","実用に耐える"],["同じ入力から","同じ結果を返す"],["別の言語の実装が","使える適合の試験"])]
o=['<svg viewBox="0 0 760 450" style="min-width:640px" role="img" aria-label="5つの設計原則と、いまの基盤で足りているもの ・ 足りないもの">']
o.append('<text class="note" x="20" y="22">設計原則（構想 v0.3）</text><text class="note" x="236" y="22">いまの基盤で足りている</text><text class="note" x="506" y="22">足りない</text>')
for i,(p,h,n) in enumerate(rows):
    y=34+i*70
    o.append(f'<rect class="box" x="20" y="{y}" width="204" height="58"/>'); lines(o,32,y+24 if len(p)>1 else y+34,p,"title")
    o.append(f'<rect class="box kind-1" x="234" y="{y}" width="262" height="58"/>'); lines(o,246,y+24,h)
    o.append(f'<rect class="box warn" x="506" y="{y}" width="236" height="58"/>'); lines(o,518,y+24,n)
o.append('<rect class="box focus" x="20" y="390" width="722" height="48"/>')
o.append('<text class="label" x="34" y="410">足す：基盤から AI を呼ぶ契約（Agents 契約）</text>')
o.append('<text class="note" x="34" y="428">全体構成図の外部インターフェースは、4つとも外から基盤を呼ぶ向きである</text>')
o.append('</svg>'); W['q0-principles']='\n'.join(o)

# Q2：標準技術アダプターの選定
cols=[("いま採る",234),("後で決める",424),("いまは採らない",614)]
rows=[("JSON Schema","box",[["組み込んだまま使う"],None,["外のプロセスで呼ぶ"]]),
      ("JSON-LD","box",[["@id ・ @context ・","@protected の考えを写す"],["組み込み","（手元の Context だけ）"],["リモートの Context を","取りに行く"]]),
      ("SHACL","box",[["検証結果の形","（必ず持つ3項目）を写す"],["組み込むか、","外のプロセスで呼ぶか"],["Python ・ Java の","実行環境が要る形"]]),
      ("RDF ・ SPARQL","box",[None,["x-ref で足りない実例が","出たら測って決める"],["いまストアを入れる"]])]
o=['<svg viewBox="0 0 800 340" style="min-width:680px" role="img" aria-label="標準技術アダプターの選定">']
for t,x in cols: o.append(f'<text class="note" x="{x}" y="22">{t}</text>')
for i,(n,k,v) in enumerate(rows):
    y=34+i*68
    o.append(f'<rect class="{k}" x="20" y="{y}" width="200" height="56"/><text class="title" x="34" y="{y+34}">{e(n)}</text>')
    for j,t in enumerate(v):
        x=cols[j][1]
        if t is None: o.append(f'<text class="note" x="{x+88}" y="{y+34}" text-anchor="middle">―</text>'); continue
        cls="box focus" if j==0 else "box"
        o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="178" height="56"/>')
        lines(o,x+10,y+24 if len(t)>1 else y+34,t,"label")
o.append('<text class="note" x="20" y="322">写せなかった必須の意味があれば合格にしない。写せた ・ 写せなかった ・ 扱えないを返す</text>')
o.append('</svg>'); W['q2-adapters']='\n'.join(o)
for k,v in W.items(): open(k+'.src.svg','w').write(v)
