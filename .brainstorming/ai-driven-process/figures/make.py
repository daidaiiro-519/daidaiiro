def e(s): return s.replace('&','&amp;').replace('<','&lt;')
W={}
# Q0：3つの指標と見る順
o=['<svg viewBox="0 0 640 400" style="min-width:540px" role="img" aria-label="プロセスを測る3つの指標と見る順">']
o.append('<text class="label" x="18" y="22" font-weight="700" text-anchor="middle">先</text>')
o.append('<line class="flow" x1="18" y1="32" x2="18" y2="282"/>')
o.append('<text class="label" x="18" y="300" font-weight="700" text-anchor="middle">後</text>')
rows=[("focus","1 目的","利用者の目的に届いたか","作ったものの妥当性確認で、目的を果たせた割合"),
      ("kind-2","2 制約","revert と障害を増やしていないか","上がったら、速くなったのではなく問題を後ろへ送っただけ"),
      ("kind-3","3 目的","人の時間を最小にしたか","人の承認数 ・ 人を待つ時間 ・ 目的と基準に使った時間")]
for i,(k,b,t,n) in enumerate(rows):
    y=12+i*96
    o.append(f'<rect class="box {k}" x="40" y="{y}" width="590" height="80"/>')
    o.append(f'<rect class="badge {k}" x="54" y="{y+14}" width="72" height="22"/><text class="note {k}" x="90" y="{y+29}" font-weight="700" text-anchor="middle">{b}</text>')
    o.append(f'<text class="title {k}" x="140" y="{y+31}">{e(t)}</text><text class="label" x="54" y="{y+62}">{e(n)}</text>')
o.append('<rect class="box warn" x="40" y="318" width="250" height="62"/><text class="label warn" x="54" y="342" font-weight="700">言わないこと</text><text class="label" x="54" y="366">品質が担保された</text>')
o.append('<rect class="box" x="306" y="318" width="324" height="62"/><text class="label" x="320" y="342" font-weight="700">言えること</text><text class="label" x="320" y="366">終了基準を満たし、残った危険を受け入れた</text>')
o.append('</svg>'); W['q0-measures']='\n'.join(o)

# Q2：業務領域のカテゴリーごとに残すもの
cols=["同じ言葉","受け入れテスト","集約の公開面と不変条件","外の製品とつなぐ面"]
rows=[("中核","kind-1",[2,2,2,0]),("補完","kind-2",[2,2,0,0]),("一般","kind-3",[2,0,0,2])]
o=['<svg viewBox="0 0 660 260" style="min-width:560px" role="img" aria-label="業務領域のカテゴリーごとに残して育てるもの">']
for j,c in enumerate(cols):
    x=210+j*120; o.append(f'<text class="note" x="{x}" y="22" font-weight="700" text-anchor="middle">{e(c)}</text>')
for i,(n,k,v) in enumerate(rows):
    y=38+i*60
    o.append(f'<rect class="box {k}" x="10" y="{y}" width="130" height="48"/><text class="title {k}" x="75" y="{y+30}" text-anchor="middle">{n}</text>')
    for j,s in enumerate(v):
        x=210+j*120
        if s==2: o.append(f'<rect class="badge focus" x="{x-34}" y="{y+13}" width="68" height="22"/><text class="note focus" x="{x}" y="{y+28}" font-weight="700" text-anchor="middle">残す</text>')
        else: o.append(f'<text class="note" x="{x}" y="{y+29}" text-anchor="middle">―</text>')
o.append('<text class="note" x="10" y="238">同じ言葉はどのカテゴリーでも残す。集約の層まで持つのは中核だけ</text>')
o.append('</svg>'); W['q2-categories']='\n'.join(o)

# Q3：受け入れ基準とテストの対応表
rows=[("UC-3 主成功","確定すると在庫が減る","test UC-3/main","あり","済","focus"),
      ("UC-3 最低保証","失敗してもログが残る","test UC-3/min","あり","済","focus"),
      ("UC-3 4a","在庫切れなら確定しない","test UC-3/4a","あり","済","focus"),
      ("UC-3 6a","決済が落ちたら取り消す","（無い）","―","欠け","warn"),
      ("（無い）","","test 3回やり直す","あり","範囲の越え","warn")]
o=['<svg viewBox="0 0 700 330" style="min-width:600px" role="img" aria-label="受け入れ基準とテストの対応表">']
for x,t in [(10,"受け入れ基準"),(330,"テスト"),(500,"実装前に失敗"),(620,"人の確認")]:
    o.append(f'<text class="note" x="{x}" y="22" font-weight="700">{t}</text>')
for i,(a,b,t,f,s,k) in enumerate(rows):
    y=36+i*54
    o.append(f'<rect class="box" x="10" y="{y}" width="300" height="44"/><text class="label" x="22" y="{y+19}" font-weight="700">{e(a)}</text>')
    if b: o.append(f'<text class="label small" x="22" y="{y+36}">{e(b)}</text>')
    o.append(f'<rect class="box" x="330" y="{y}" width="150" height="44"/><text class="label" x="342" y="{y+27}">{e(t)}</text>')
    o.append(f'<text class="label" x="540" y="{y+27}" text-anchor="middle">{f}</text>')
    o.append(f'<rect class="badge {k}" x="606" y="{y+11}" width="88" height="22"/><text class="note {k}" x="650" y="{y+26}" font-weight="700" text-anchor="middle">{s}</text>')
o.append('<text class="note" x="10" y="314">人が見るのは右の列だけ：欠け ・ 範囲の越え ・ 重複</text>')
o.append('</svg>'); W['q3-trace']='\n'.join(o)

# Q4：変更の種類ごとの設定
cols=["まとめて決める量","出す量","出したあとの守り"]
rows=[("コード","kind-1",["ユースケース1つずつ","小さく","フラグ ・ 段階リリース ・ すぐ戻す"]),
      ("設定","kind-1",["ユースケース1つずつ","小さく","段階リリース ・ すぐ戻す"]),
      ("データ","kind-2",["リリースの分を先に","小さく","直前に退避 ・ 手順を試す"]),
      ("外への作用","kind-2",["リリースの分を先に","小さく","フィックスフォワードを試す"])]
o=['<svg viewBox="0 0 700 330" style="min-width:600px" role="img" aria-label="変更の種類ごとの3つの設定">']
xs=[(140,170),(326,90),(432,262)]
for (x,w),c in zip(xs,cols): o.append(f'<text class="note" x="{x+w//2}" y="22" font-weight="700" text-anchor="middle">{c}</text>')
for i,(n,k,v) in enumerate(rows):
    y=36+i*62
    o.append(f'<rect class="box {k}" x="10" y="{y}" width="116" height="50"/><text class="title {k}" x="68" y="{y+31}" text-anchor="middle">{e(n)}</text>')
    for (x,w),t in zip(xs,v):
        o.append(f'<rect class="box" x="{x}" y="{y}" width="{w}" height="50"/><text class="label" x="{x+w//2}" y="{y+31}" text-anchor="middle">{e(t)}</text>')
o.append('<text class="note" x="10" y="294">色：すぐ戻せる変更（kind-1）・ 戻せない変更（kind-2）</text>')
o.append('<text class="note" x="10" y="316">戻しやすさは、戻すまでの時間 ・ 手順を試したか ・ 失うデータで測る</text>')
o.append('</svg>'); W['q4-settings']='\n'.join(o)

# Q5：ユースケースを変更に切る
o=['<svg viewBox="0 0 700 360" style="min-width:600px" role="img" aria-label="1つのユースケースを拡張ごとの変更に切る">']
o.append('<rect class="area" x="10" y="10" width="290" height="306"/><text class="title" x="26" y="36">UC-3 注文を確定する</text>')
for t_,y in [("主成功シナリオ（1〜7）",62),("最低保証：ログが残る",112),("拡張 4a：在庫切れ",184),("拡張 6a：決済が落ちた",250)]:
    o.append(f'<rect class="box" x="26" y="{y}" width="258" height="36"/><text class="label" x="40" y="{y+23}">{e(t_)}</text>')
for t_,s,y,k in [("変更 UC-3/main","受け入れテスト UC-3/main",80,"focus"),("変更 UC-3/4a","受け入れテスト UC-3/4a",177,""),("変更 UC-3/6a","受け入れテスト UC-3/6a",243,"")]:
    cls=f"box {k}".strip(); o.append(f'<rect class="{cls}" x="420" y="{y}" width="270" height="50"/><text class="label{(" "+k) if k else ""}" x="434" y="{y+20}" font-weight="700">{e(t_)}</text><text class="label small" x="434" y="{y+40}">{e(s)}</text>')
o.append('<path class="flow" d="M284,80 H350 V100 H418"/><path class="flow" d="M284,130 H350 V110 H418"/>')
o.append('<path class="flow" d="M284,202 H418"/><path class="flow" d="M284,268 H418"/>')
o.append('<text class="note" x="10" y="344">どの変更でも、UC-3 の受け入れテスト全部を回帰テストとして回す</text>')
o.append('</svg>'); W['q5-split']='\n'.join(o)
for k,v in W.items(): open(f'{k}.src.svg','w').write(v)
