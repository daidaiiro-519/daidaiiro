def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
# Q0：形の外の決まり138件と、だれが確かめているか
o=['<svg viewBox="0 0 680 330" style="min-width:560px" role="img" aria-label="形の外の決まり138件のうち、どこでも検査されていない62件">']
o.append('<text class="title" x="20" y="28">ボードの決まり 397件（出どころごとに数えた）</text>')
o.append('<rect class="box" x="20" y="44" width="380" height="40"/><text class="label" x="34" y="69">形（JSON Schema）259件 ── check が見る</text>')
o.append('<rect class="box kind-2" x="408" y="44" width="252" height="40"/><text class="label" x="422" y="69">形の外 138件</text>')
o.append('<path class="flow" d="M534,86 V112"/>')
o.append('<text class="title" x="20" y="134">形の外の138件を、だれが確かめているか</text>')
x=20; S=640/138
segs=[("box kind-1",76,"確かめている 76件","inspect ・ status ・ lock など"),("box warn",62,"どこでも検査されていない 62件","")]
for cls,n,t,s in segs:
    w=n*S
    o.append(f'<rect class="{cls}" x="{x:.0f}" y="146" width="{w:.0f}" height="44"/>')
    o.append(f'<text class="label" x="{x+12:.0f}" y="173">{e(t)}</text>')
    x+=w
o.append('<text class="note" x="20" y="208">inspect ・ status ・ lock ・ x-ref など、Rust と基盤の検査で見ている</text>')
o.append('<text class="note" x="20" y="236">検査されていない62件の内訳</text>')
for i,t in enumerate(["機械で見られる 37件","意味の判断が要る 12件","外の事実 13件"]):
    x=200+i*156
    o.append(f'<rect class="badge warn" x="{x}" y="220" width="148" height="26"/><text class="note" x="{x+74}" y="238" text-anchor="middle">{e(t)}</text>')
o.append('<rect class="box warn" x="20" y="266" width="640" height="48"/>')
o.append('<text class="label" x="34" y="287">実物の2枚：論点6件（決まりは3〜5件）・ 2文の caption 3件</text>')
o.append('<text class="label warn" x="34" y="305">それでも check と inspect の指摘は0件で、人が承認の前に読むまで見つからない</text>')
o.append('</svg>'); W['q0-unchecked']='\n'.join(o)

# Q1：基盤に足すもの・扱わないもの
o=['<svg viewBox="0 0 700 420" style="min-width:580px" role="img" aria-label="基盤を作り直さずに足すものと、扱わないもの">']
o.append('<rect class="area" x="10" y="10" width="480" height="64"/><text class="title" x="24" y="34">Skill のスキーマ（concrete）</text><text class="label" x="24" y="58">brainstorming ・ ACDR ・ skills-creator</text>')
o.append('<path class="flow" d="M250,76 V100"/>')
o.append('<rect class="area" x="10" y="104" width="480" height="300"/><text class="title" x="24" y="128">基盤 schema-driven</text>')
o.append('<rect class="box" x="26" y="142" width="448" height="50"/><text class="title" x="40" y="164">abstract@v1</text><text class="note" x="40" y="183">変えない。いまの操作と結果もそのまま</text>')
o.append('<rect class="box focus" x="26" y="204" width="448" height="104"/><text class="title focus" x="40" y="226">足す（新しい版 @v2）</text>')
for i,t in enumerate(["1 決まりごとの id と、5つの結果の語","2 決まりごとに、確かめる担い手と要る能力","3 必須の決まりを確かめられなければ成功にしない"]):
    o.append(f'<text class="label" x="40" y="{250+i*22}">{e(t)}</text>')
o.append('<text class="note" x="26" y="330">確かめる担い手</text>')
for i,t in enumerate(["エンジン","Rust の検査","AI","人（承認）"]):
    x=26+i*114
    k="box kind-2" if t=="AI" else "box"
    o.append(f'<rect class="{k}" x="{x}" y="340" width="104" height="40"/><text class="label" x="{x+52}" y="365" text-anchor="middle">{e(t)}</text>')
o.append('<text class="note" x="26" y="396">AI は Agents 契約で呼ぶ</text>')
o.append('<rect class="boundary" x="506" y="104" width="184" height="300"/><text class="title" x="520" y="128">扱わない</text>')
for i,t in enumerate(["UI ・ Workflow の","Runtime","JSON-LD ・ RDF ・","SHACL への写し","自前のモデルを","正本にすること"]):
    o.append(f'<text class="label" x="520" y="{158+i*22 + (i//2)*14}">{e(t)}</text>')
o.append('<text class="note" x="520" y="330">正本は JSON Schema と</text><text class="note" x="520" y="348">独自の語彙のまま。</text><text class="note" x="520" y="366">試作で持てない意味が</text><text class="note" x="520" y="384">出たときだけ見直す</text>')
o.append('</svg>'); W['q1-layers']='\n'.join(o)

# Q2：担い手と5つの結果
rows=[("エンジン（JSON Schema）","合格 ・ 違反","box"),("Rust などの検査","合格 ・ 違反","box"),("AI（Agents 契約で呼ぶ）","違反（機械で裏付けたもの）・ 確かめていない","box kind-2"),("担い手が無い","担い手が無い","box warn"),("どの担い手でも：時間切れ ・ 読めない応答","処理の失敗（記録を作らない）","box warn")]
o=['<svg viewBox="0 0 700 380" style="min-width:580px" role="img" aria-label="決まりを確かめる担い手と、返す結果">']
o.append('<text class="note" x="20" y="24">確かめる担い手</text><text class="note" x="380" y="24">返す結果</text>')
for i,(a,b,k) in enumerate(rows):
    y=36+i*56
    o.append(f'<rect class="{k}" x="20" y="{y}" width="310" height="42"/><text class="label" x="34" y="{y+26}">{e(a)}</text>')
    o.append(f'<path class="flow" d="M332,{y+21} H372"/>')
    o.append(f'<text class="label" x="380" y="{y+26}">{e(b)}</text>')
o.append('<rect class="box focus" x="20" y="320" width="660" height="44"/>')
o.append('<text class="label" x="34" y="340">必須の決まりに合格以外が1件でもあれば、全体を成功にしない</text>')
o.append('<text class="note" x="34" y="357">人の判断（目的に合うか）は5つの外で、いまの approve の記録として持つ</text>')
o.append('</svg>'); W['q2-verifiers']='\n'.join(o)
for k,v in W.items(): open(k+'.src.svg','w').write(v)
