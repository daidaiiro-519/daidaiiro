def compare(t):
    # t: 題（前）,題（後）, 前の3つ, 後の4つ, 差分の注記, 凡例
    a,b,before,after,mark,legend=t
    s=['<svg viewBox="0 0 720 330" style="min-width:600px" role="img" aria-label="'+a+'と'+b+'の比較">']
    s.append(f'<rect class="area" x="10" y="10" width="340" height="280"/><text class="title" x="26" y="36">{a}</text>')
    s.append(f'<rect class="area" x="370" y="10" width="340" height="280"/><text class="title focus" x="386" y="36">{b}</text>')
    ys=[56,126,196]
    for i,(txt,cls) in enumerate(before):
        y=ys[i] if len(before)==3 else 56+i*56
        s.append(f'<rect class="box{cls}" x="30" y="{y}" width="230" height="44"/><text class="label{cls}" x="145" y="{y+27}" text-anchor="middle">{txt}</text>')
        if i: s.append(f'<path class="flow" d="M145,{ys[i-1]+44} V{y-2}"/>')
    ys2=[56,112,168,224]
    for i,(txt,cls) in enumerate(after):
        y=ys2[i]
        s.append(f'<rect class="box{cls}" x="390" y="{y}" width="230" height="44"/><text class="label{cls}" x="505" y="{y+27}" text-anchor="middle">{txt}</text>')
        if i: s.append(f'<path class="flow" d="M505,{ys2[i-1]+44} V{y-2}"/>')
    s.append(f'<rect class="badge focus" x="630" y="124" width="62" height="20"/><text class="note small focus" x="661" y="138" text-anchor="middle">{mark[1]}</text>')
    s.append(f'<rect class="badge warn" x="270" y="138" width="62" height="20"/><text class="note small warn" x="301" y="152" text-anchor="middle">{mark[0]}</text>')
    s.append(f'<text class="note" x="10" y="318">{legend}</text></svg>')
    return '\n'.join(s)
def gantt(t):
    title,cols,rows,today,legend=t
    W0,CW,RH,Y0=150,110,36,64
    n=len(cols)
    s=[f'<svg viewBox="0 0 {W0+CW*n+70} {Y0+RH*len(rows)+60}" style="min-width:560px" role="img" aria-label="{title}">',f'<text class="title" x="10" y="24">{title}</text>']
    for i,c in enumerate(cols):
        x=W0+CW*i
        s.append(f'<text class="note small" x="{x+CW//2}" y="50" text-anchor="middle">{c}</text><line class="grid" x1="{x}" y1="56" x2="{x}" y2="{Y0+RH*len(rows)}"/>')
    s.append(f'<line class="grid" x1="{W0+CW*n}" y1="56" x2="{W0+CW*n}" y2="{Y0+RH*len(rows)}"/>')
    for j,(name,a,b,k,lab) in enumerate(rows):
        y=Y0+RH*j
        s.append(f'<text class="label" x="{W0-10}" y="{y+22}" text-anchor="end">{name}</text>')
        x0=W0+a*CW/4; x1=W0+b*CW/4
        s.append(f'<rect class="series-{k}" x="{x0:.0f}" y="{y+8}" width="{x1-x0:.0f}" height="20"/>')
        if lab: s.append(f'<text class="note small" x="{x1+6:.0f}" y="{y+22}">{lab}</text>')
    xt=W0+today*CW/4; yb=Y0+RH*len(rows)
    s.append(f'<line class="link warn" x1="{xt:.0f}" y1="56" x2="{xt:.0f}" y2="{yb+4}"/><text class="note small warn" x="{xt:.0f}" y="{yb+18}" text-anchor="middle">今日</text>')
    s.append(f'<text class="note" x="10" y="{yb+44}">{legend}</text></svg>')
    return '\n'.join(s)
def er(t):
    title,ents,rels,legend=t
    H=max(y+36+18*len(at) for (x,y,n,at,k) in ents)+ (70 if len(ents)>3 else 50)
    s=[f'<svg viewBox="0 0 720 {H}" style="min-width:600px" role="img" aria-label="{title}">',f'<text class="title" x="10" y="24">{title}</text>']
    for (x,y,name,attrs,k) in ents:
        h=36+18*len(attrs)
        kc=f' {k}' if k else ''
        s.append(f'<rect class="box{kc}" x="{x}" y="{y}" width="180" height="{h}"/><text class="title{kc}" x="{x+12}" y="{y+22}">{name}</text><line class="grid" x1="{x}" y1="{y+32}" x2="{x+180}" y2="{y+32}"/>')
        for i,a in enumerate(attrs):
            cls='note small' if not a.startswith('PK') else 'label small'
            s.append(f'<text class="{cls}" x="{x+12}" y="{y+48+18*i}">{a}</text>')
    for (d,c1,p1,c2,p2) in rels:
        s.append(f'<path class="link" d="{d}"/><text class="label small" x="{p1[0]}" y="{p1[1]}">{c1}</text><text class="label small" x="{p2[0]}" y="{p2[1]}">{c2}</text>')
    s.append(f'<text class="note" x="10" y="{H-10}">{legend}</text></svg>')
    return '\n'.join(s)
def venn(t):
    title,sets,regions,legend=t
    s=[f'<svg viewBox="0 0 520 396" style="min-width:440px;max-width:560px" role="img" aria-label="{title}">',f'<text class="title" x="10" y="24">{title}</text>']
    C=[(200,150),(320,150),(260,250)]
    for i,(cx,cy) in enumerate(C):
        s.append(f'<path class="link kind-{i+1}" d="M{cx-100},{cy} A100,100 0 1,0 {cx+100},{cy} A100,100 0 1,0 {cx-100},{cy} Z"/>')
    pos=[(140,70,'end'),(380,70,'start'),(260,370,'middle')]
    for i,(name) in enumerate(sets):
        x,y,an=pos[i]
        s.append(f'<text class="title kind-{i+1}" x="{x}" y="{y}" text-anchor="{an}">{name}</text>')
    for (x,y,txt,cls) in regions:
        s.append(f'<text class="{cls}" x="{x}" y="{y}" text-anchor="middle">{txt}</text>')
    s.append(f'<text class="note small" x="10" y="388">{legend}</text></svg>')
    return '\n'.join(s)
NEW={
'compare-before-after':(compare,
 ('変更前','変更後',[('手順1',''),('手順2（なくす）',' warn'),('手順3','')],[('手順1',''),('手順4（足す）',' focus'),('手順2の代わり',''),('手順3','')],('削除','追加'),'塗りの色：なくすもの（warn）と、足すもの（focus）'),
 ('今の手順','新しい手順',[('変更を書く',''),('担当者が手で確かめる',' warn'),('本番へ反映する','')],[('変更を書く',''),('自動テストを実行する',' focus'),('担当者が差分を読む',''),('本番へ反映する','')],('削除','追加'),'レビューの手順の変更：手で確かめる作業をなくし、自動テストを足す')),
'gantt':(gantt,
 ('計画の題',['期間1','期間2','期間3','期間4'],[('作業1',0,3,1,''),('作業2',2,6,1,''),('作業3',5,11,2,''),('作業4',10,14,3,''),('作業5',13,16,3,'節目')],7,'帯の色：作業の種類（series-1〜3）'),
 ('チームの第2四半期の計画',['4月','5月','6月','7月'],[('利用者への聞き取り',0,3,1,''),('画面の設計',2,6,1,''),('実装',5,11,2,''),('結合テスト',10,14,3,''),('リリース準備',13,15,3,'7月中旬')],6,'帯の色：調べる（series-1）・ 作る（series-2）・ 確かめる（series-3）。1列は1か月')),
'er-diagram':(er,
 ('エンティティの関係',[(20,60,'エンティティA',['PK ID','属性1','属性2'],None),(270,60,'エンティティB',['PK ID','FK A の ID','属性1'],'focus'),(520,60,'エンティティC',['PK ID','FK B の ID','属性1'],None)],
  [('M200,96 H270','1',(206,90),'多',(252,90)),('M450,96 H520','1',(456,90),'多',(502,90))],'1 と 多：関連の数（カーディナリティ）。PK は主キー、FK は外部キー'),
 ('ブログのデータモデル',[(20,60,'ユーザー',['PK ユーザー ID','名前','メールアドレス'],None),(270,60,'投稿',['PK 投稿 ID','FK ユーザー ID','題名','公開日'],'focus'),(520,60,'コメント',['PK コメント ID','FK 投稿 ID','FK ユーザー ID','本文'],None),(270,230,'タグ',['PK タグ ID','名前'],None)],
  [('M200,96 H270','1',(206,90),'多',(252,90)),('M450,96 H520','1',(456,90),'多',(502,90)),('M360,168 V230','多',(366,184),'多',(366,224)),('M110,150 V318 H610 V186','1',(116,166),'多',(616,202))],'1 と 多：関連の数（カーディナリティ）。投稿とタグは多対多で、中間の表が持つ')),
'venn':(venn,
 ('集合の重なり',['集合A','集合B','集合C'],[(170,150,'A だけ','label'),(350,150,'B だけ','label'),(260,300,'C だけ','label'),(260,128,'A と B','note'),(215,215,'A と C','note'),(305,215,'B と C','note'),(260,190,'3つとも','label focus')],'円の重なりが、両方に属する要素を示す'),
 ('チームのスキルの重なり',['設計','開発','運用'],[(165,140,'佐藤','label'),(165,158,'田中','label'),(355,150,'鈴木','label'),(260,305,'高橋','label'),(260,128,'伊藤','note'),(215,215,'渡辺','note'),(305,215,'山本','note'),(260,190,'中村','label focus')],'中村さんは3つとも担当できる。運用だけの人は高橋さんひとりである')),
}
