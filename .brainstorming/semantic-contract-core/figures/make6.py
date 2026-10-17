# 第8回の図：Q0 の基盤の境界と向き、Q1 の5つの仕様の対応
def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
def lines(o,x,y,ls,cls="label",step=18):
    for i,l in enumerate(ls): o.append(f'<text class="{cls}" x="{x}" y="{y+i*step}">{e(l)}</text>')
# Q0：基盤の境界と向き
o=['<svg viewBox="0 0 780 400" style="min-width:660px" role="img" aria-label="基盤の境界と、呼ぶ向き">']
o.append('<rect class="boundary" x="20" y="20" width="170" height="150"/><text class="title" x="34" y="44">使う側</text>')
lines(o,34,70,["Skill ・ ほかの言語","の担い手 ・ 人"])
o.append('<path class="flow" d="M192,95 H246"/><text class="note" x="200" y="80">呼ぶ</text>')
o.append('<rect class="area" x="250" y="20" width="300" height="320"/><text class="title" x="264" y="44">基盤 schema-driven</text>')
o.append('<rect class="box" x="264" y="56" width="272" height="56"/><text class="label" x="276" y="80">インターフェース</text><text class="note" x="276" y="100">CLI ・ MCP ・ HTTP ・ WASM Component</text>')
o.append('<rect class="box" x="264" y="122" width="272" height="88"/><text class="label" x="276" y="146">持つもの</text>')
lines(o,276,168,["宣言の契約 ・ 確かめる仕組み ・ 結果の語","作成 ・ 更新 ・ 承認 ・ 描画"],"note")
o.append('<rect class="box focus" x="264" y="220" width="272" height="56"/><text class="label" x="276" y="244">出ていく側のアダプター（足す）</text><text class="note" x="276" y="264">Agents 契約で AI の CLI を起動する</text>')
o.append('<path class="flow" d="M538,248 H596"/><text class="note" x="552" y="232">呼ぶ</text>')
o.append('<rect class="boundary" x="600" y="210" width="160" height="76"/><text class="title" x="614" y="236">AI の CLI</text><text class="note" x="614" y="258">Claude Code ・ Codex</text>')
o.append('<rect class="boundary" x="600" y="20" width="160" height="150"/><text class="title" x="614" y="44">持たないもの</text>')
lines(o,614,70,["使う側のデータ","UI ・ Workflow を","動かす側","コードの生成"],"note")
o.append('<text class="note" x="20" y="366">全体構成図の外部インターフェースは「使う側 → 基盤」の向きだけ。</text>')
o.append('<text class="note" x="20" y="386">「基盤 → AI」の向きは、出ていく側のアダプターとして足す</text>')
o.append('</svg>'); W['q0-directions']='\n'.join(o)
# Q1：5つの仕様の対応
rows=[("Semantic Model","kind と id ・ プロパティと型","instance.rs ・ schema.rs","一部ある"),
      ("Vocabulary Spec","$ref で形だけ共有","meta-schema.json","無い"),
      ("JSON Representation","JSON Schema 2020-12 と注釈","meta-schema.json","同じ"),
      ("Constraint Spec","形 ・ x-ref ・ x-derive。意味の決まりは文だけ","annotations.schema.json ・ check.rs","一部ある"),
      ("Interpreter Protocol","結果の形だけ。能力の表明が無い","values.rs","一部ある")]
o=['<svg viewBox="0 0 800 380" style="min-width:680px" role="img" aria-label="抽象の5つの仕様と、いまの基盤の対応">']
o.append('<text class="note" x="20" y="22">仕様（構想 v0.3）</text><text class="note" x="210" y="22">いまの基盤にあるもの</text><text class="note" x="520" y="22">ファイル</text><text class="note" x="710" y="22">判定</text>')
for i,(a,b,c,dv) in enumerate(rows):
    y=34+i*66
    o.append(f'<rect class="box" x="16" y="{y}" width="188" height="54"/><text class="title" x="26" y="{y+33}">{e(a)}</text>')
    o.append(f'<rect class="box" x="210" y="{y}" width="300" height="54"/>')
    bl=[b] if len(b)<=22 else [b[:b.index('。')+1], b[b.index('。')+1:]] if '。' in b else [b]
    lines(o,222,y+24 if len(bl)>1 else y+33,bl)
    cl=c.split(' ・ ') if len(c)>24 else [c]
    o.append(f'<rect class="box" x="520" y="{y}" width="180" height="54"/>'); lines(o,530,y+24 if len(cl)>1 else y+33,cl,"note")
    k={"同じ":"badge","一部ある":"badge","無い":"badge warn"}[dv]
    o.append(f'<rect class="{k}" x="710" y="{y+15}" width="72" height="24"/><text class="note" x="746" y="{y+32}" text-anchor="middle">{dv}</text>')
o.append('<text class="note" x="20" y="366">足すものは abstract の新しい版（@v2）に置き、abstract@v1 は変えない</text>')
o.append('</svg>'); W['q1-specs']='\n'.join(o)
for k,v in W.items(): open(k+'.src.svg','w').write(v)
