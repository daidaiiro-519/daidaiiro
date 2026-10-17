# 第9回の図。標準は JSON Schema だけ、x- の拡張は基盤の契約（Q0 の目的 ・ 原則2 ・ 原則4、Q2）。
# class だけで書き、design-svg resolve で値へ解く。行は手で分ける（単語の途中で折らない）。
import re
def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
def lines(o,x,y,ls,cls="label",step=18):
    for i,l in enumerate(ls): o.append(f'<text class="{cls}" x="{x}" y="{y+i*step}">{e(l)}</text>')

# Q0：先頭に目的を置き、原則2 ・ 原則4 の行を直す。ほかの行は第3回の図のまま。
src=open('q0-principles.src.svg').read()
src=src.replace('>概念を自前で定義して写すこと<','>概念を参考に自前で定義すること<')
src=src.replace('>JSON Schema を<','>標準は JSON Schema だけ。<').replace('>エンジンに組み込んでいる<','>形の確かめを任せている<')
src=src.replace('>足した注釈を確かめ、<','>x- の拡張を基盤のコマンドで<').replace('>確かめなければ返すこと<','>確かめ、できなければ返すこと<')
def shift(m):
    return m.group(1)+str(int(m.group(2))+64)
body=src.split('\n',1)[1]
body=re.sub(r'(\by=")(\d+)',shift,body)
top=['<svg viewBox="0 0 760 514" style="min-width:640px" role="img" aria-label="基盤の目的と、5つの設計原則のいまの基盤で足りているもの ・ 足りないもの">',
     '<rect class="box focus" x="20" y="12" width="722" height="54"/>',
     '<text class="title" x="34" y="34">目的</text>',
     '<text class="label" x="34" y="54">AI が書いたインスタンスを、宣言した決まりで機械が確かめ、確かめられなかった決まりも隠さず返す</text>']
W['q0-principles']='\n'.join(top)+'\n'+body

# Q2：標準は JSON Schema だけ。ほかの3つは概念を参考にし、x- の拡張として基盤のコマンドが確かめる。写さない。
rows=[("JSON Schema","box kind-1",["標準としてそのまま使う","構造と値の制約"],["形の確かめ","JSON Schema の実装が確かめる"]),
      ("JSON-LD","boundary",["参考にする概念：共通の識別子","・ 名前の対応付け ・ 再定義の保護"],["Vocabulary Spec ・ Semantic Model","基盤のコマンドが確かめる"]),
      ("SHACL","boundary",["参考にする概念：関係の制約","・ 経路 ・ 閉じる／開く ・ 結果の報告"],["Constraint Spec ・ 結果の形","基盤のコマンドが確かめる"]),
      ("RDF ・ SPARQL","boundary",["参考にする概念：インスタンスを","またいだ関係のグラフと問い合わせ"],["Relation / Graph","基盤のコマンドが確かめる"])]
o=['<svg viewBox="0 0 780 420" style="min-width:660px" role="img" aria-label="標準は JSON Schema だけにし、ほかの技術の概念は自前の拡張として基盤のコマンドが確かめる">']
o.append('<text class="note" x="20" y="22">技術</text><text class="note" x="196" y="22">使うもの ・ 参考にする概念</text><text class="note" x="486" y="22">置き場所と、確かめる担い手</text>')
for i,(n,k,c,d) in enumerate(rows):
    y=34+i*68
    o.append(f'<rect class="{k}" x="20" y="{y}" width="160" height="56"/><text class="title" x="34" y="{y+34}">{e(n)}</text>')
    o.append(f'<path class="flow" d="M182,{y+28} H192"/>')
    o.append(f'<rect class="box" x="196" y="{y}" width="270" height="56"/>'); lines(o,208,y+24,c)
    o.append(f'<path class="flow" d="M468,{y+28} H482"/>')
    o.append(f'<rect class="box {"kind-1" if i==0 else "focus"}" x="486" y="{y}" width="274" height="56"/>'); lines(o,498,y+24,d)
o.append('<rect class="box warn" x="20" y="312" width="740" height="92"/>')
o.append('<text class="title" x="34" y="336">x- の拡張は、基盤の契約</text>')
o.append('<text class="label" x="34" y="360">JSON Schema の実装は、知らないキーワードを注釈として集めるだけで確かめない</text>')
o.append('<text class="label" x="34" y="384">足した範囲は基盤のコマンドが確かめ、確かめられなかった決まりも結果の5つの語で返す。標準へは写さない</text>')
o.append('</svg>'); W['q2-concepts']='\n'.join(o)
for k,v in W.items(): open(k+'.src.svg','w').write(v)
