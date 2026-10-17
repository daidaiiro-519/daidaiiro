# 第2回の図。class だけで書き、design-svg resolve で値へ解く。
def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
# Q0：基盤が持つもの ・ 持たないもの（いま足りている／足りない）
o=['<svg viewBox="0 0 720 440" style="min-width:600px" role="img" aria-label="基盤が持つものと持たないもの">']
o.append('<rect class="area" x="10" y="10" width="470" height="420"/><text class="title" x="24" y="36">基盤 schema-driven が持つもの</text>')
rows=[("宣言の契約","abstract スキーマ ・ 注釈（x-ref ・ x-derive ・ x-prompt）","足りている","box"),
      ("確かめる仕組み","書いたその場の検証 ・ check ・ approve ・ 公開した版の指紋","足りている","box"),
      ("決まりごとの結果","確かめた担い手と結果を、決まり1件ずつ返す","足りない","box warn"),
      ("確かめなかったことの報告","検査しなかった決まりを、合格にせず返す","足りない","box warn"),
      ("Agents 契約","AI を確かめる担い手 ・ 会話の相手として呼ぶ約束","足りない","box warn"),
      ("描画","インスタンスを承認の画面へ写す（render）","足りている","box")]
for i,(t,n,s,k) in enumerate(rows):
    y=50+i*62
    o.append(f'<rect class="{k}" x="24" y="{y}" width="442" height="52"/><text class="title" x="38" y="{y+22}">{e(t)}</text><text class="note" x="38" y="{y+42}">{e(n)}</text>')
    bk="badge warn" if s=="足りない" else "badge"
    o.append(f'<rect class="{bk}" x="380" y="{y+10}" width="76" height="22"/><text class="note" x="418" y="{y+26}" text-anchor="middle">{s}</text>')
o.append('<rect class="boundary" x="496" y="10" width="214" height="420"/><text class="title" x="510" y="36">基盤が持たないもの</text>')
for i,(t,n) in enumerate([("使う側のデータ","何を持つかは Skill"),("UI ・ Workflow の","Runtime（動かす側）"),("コードの生成と","技術の選び方"),("JSON-LD ・ RDF ・","SHACL への写し")]):
    y=50+i*90
    o.append(f'<rect class="box" x="510" y="{y}" width="186" height="74"/><text class="label" x="522" y="{y+30}">{e(t)}</text><text class="label" x="522" y="{y+54}">{e(n)}</text>')
o.append('</svg>'); W['q0-boundary']='\n'.join(o)

# Q2：抽象と具体の関係と、版の足し方
o=['<svg viewBox="0 0 680 330" style="min-width:560px" role="img" aria-label="抽象と具体の関係と、版の足し方">']
o.append('<rect class="box" x="20" y="20" width="300" height="70"/><text class="title" x="34" y="46">abstract@v1</text><text class="note" x="34" y="72">変えない。いまのインスタンスも同じ結果</text>')
o.append('<rect class="box focus" x="360" y="20" width="300" height="70"/><text class="title focus" x="374" y="46">abstract@v2（足す）</text><text class="note" x="374" y="72">独自の語彙を $vocabulary で必須にする</text>')
o.append('<path class="flow" d="M322,55 H358"/>')
o.append('<path class="flow" d="M170,92 V138"/><text class="note" x="182" y="120">concrete は abstract のインスタンス</text>')
o.append('<rect class="box" x="20" y="142" width="640" height="70"/><text class="title" x="34" y="168">concrete スキーマ（Skill ごと）</text><text class="note" x="34" y="194">決まり1件ごとの id ・ 確かめる担い手 ・ 要る能力を宣言する</text>')
o.append('<path class="flow" d="M340,214 V250"/>')
o.append('<rect class="box" x="20" y="254" width="640" height="56"/><text class="title" x="34" y="278">インスタンス（JSON のまま。正本）</text><text class="note" x="34" y="298">グラフや外の標準への写しは、必要が出たときの派生にとどめる</text>')
o.append('</svg>'); W['q2-versions']='\n'.join(o)
for k,v in W.items(): open(k+'.src.svg','w').write(v)
