# 第35回の図：抽象の層を Q1（概念モデル）・ Q6（スキーマの構成）・ Q7（スキーマの中身）に分けた図。
# 材料は advisors/ddd-abstract-model.data.json（ddd-advisor）。class だけで書き、design-svg resolve で値へ解く。
def e(s): return s.replace('&','&amp;').replace('<','&lt;')

def table(name, label, heads, widths, rows, emph=(1,), row_h=40, note=None):
    X=[16]
    for w in widths[:-1]: X.append(X[-1]+w+8)
    W=X[-1]+widths[-1]+16
    H=32+len(rows)*(row_h+6)+(28 if note else 8)
    o=[f'<svg viewBox="0 0 {W} {H}" style="min-width:{min(W,900)}px" role="img" aria-label="{e(label)}">']
    for x,h in zip(X,heads): o.append(f'<text class="note" x="{x+4}" y="22">{e(h)}</text>')
    for i,r in enumerate(rows):
        y=32+i*(row_h+6)
        for j,(x,w) in enumerate(zip(X,widths)):
            cell=r[j]; lines=cell if isinstance(cell,list) else [cell]
            cls="box focus" if j in emph else ("boundary" if j==len(widths)-1 and heads[-1].startswith("構想") else "box")
            tc="title" if j in emph else ("note" if cls=="boundary" else "label")
            o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{row_h}"/>')
            if len(lines)==1: o.append(f'<text class="{tc}" x="{x+10}" y="{y+row_h//2+5}">{e(lines[0])}</text>')
            else:
                for k,l in enumerate(lines): o.append(f'<text class="{tc}" x="{x+10}" y="{y+row_h//2-4+16*k}">{e(l)}</text>')
    if note: o.append(f'<text class="note" x="20" y="{H-10}">{e(note)}</text>')
    o.append('</svg>'); open(name+'.src.svg','w').write('\n'.join(o))

# ── Q1 概念図 ──
def chip(o,x,y,w,t,cls="box"): o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="30"/><text class="label" x="{x+8}" y="{y+20}">{e(t)}</text>')
o=['<svg viewBox="0 0 920 620" style="min-width:820px" role="img" aria-label="抽象の層の概念と、群どうしの関係">']
groups=[("宣言（Abstract スキーマ）",20,["Abstract schema","domain schema","x- keyword","x- キーワードの定義","検証の種類","base contract schema"]),
        ("使う側（Concrete スキーマ）",170,["Concrete schema","instance","prompt","rule","ubiquitous language ・ term","relation","derived value"]),
        ("検証",320,["Semantic Validator","Schema Validator","validator","Resolver","capability","validation result"])]
for title,y,items in groups:
    o.append(f'<rect class="area" x="16" y="{y}" width="888" height="130"/><text class="title" x="30" y="{y+24}">{e(title)}</text>')
    for i,t in enumerate(items):
        col=i%4; row=i//4
        chip(o,30+col*216,y+40+row*42,206,t,"box focus" if i==0 else "box")
o.append('<rect class="area" x="16" y="470" width="436" height="130"/><text class="title" x="30" y="494">描画の契約</text>')
for i,t in enumerate(["view","page template","catalog","design system"]): chip(o,30+(i%2)*206,510+(i//2)*42,196,t)
o.append('<rect class="area" x="468" y="470" width="436" height="130"/><text class="title" x="482" y="494">版と記録</text>')
for i,t in enumerate(["schema name and version","published record"]): chip(o,482,510+i*42,280,t)
o.append('<path class="flow" d="M120,152 V168"/><text class="note" x="130" y="164">定義した x- の値を書くだけ</text>')
o.append('<path class="flow" d="M120,302 V318"/><text class="note" x="130" y="314">rule ごとに validator を選んで検証する</text>')
o.append('<path class="flow" d="M120,452 V468"/><text class="note" x="130" y="464">Resolver の結果を使う</text>')
o.append('</svg>'); open('q1-model.src.svg','w').write('\n'.join(o))

# ── Q1 概念の表 ──
C=[("Abstract schema","抽象スキーマ","基盤の契約の1つの版","Abstract Schema"),
("domain schema","ドメインスキーマ","JSON Schema の構造と値の条件","Schema"),
("x- keyword","x- キーワード","抽象スキーマが定義する拡張のキー","無い"),
("keyword definition","x- キーワードの定義","形 ・ 置く場所 ・ 検証の仕方の1行","無い"),
("validation kind","検証の種類","機械的かセマンティックか","02:25 の2区分"),
("base contract schema","基盤の契約のスキーマ","抽象スキーマを組み立てるスキーマ","無い"),
("Concrete schema","具体スキーマ","Skill が書き、x- の値を使う","Concrete Schema"),
("instance","インスタンス","Concrete スキーマに従う JSON の1件","Instance"),
("prompt","書き方の案内","読み方と書くときの rule","無い"),
("rule","ルール","スキーマに書く条件の1件","Constraint"),
("ubiquitous language","ユビキタス言語","Skill ごとの terms@v1 に置く語","Vocabulary"),
("term","語","$defs の1件。欄が $ref で指す","Term（未決）"),
("relation","関係","値がほかのインスタンスを指す","Relation"),
("derived value","導出値","答えから1つに決まる値","無い"),
("validator","検証器","rule 1件の合否を出す側","Interpreter"),
("Schema Validator","スキーマ検証器","構造と値の検証","Schema Validator"),
("Semantic Validator","意味検証器","検証の全体。validator を選ぶ","Semantic Interpreter"),
("capability","能力","validator が検証できることの宣言","Capability"),
("validation result","検証結果","rule ごとの結果（5つの値）","outcome"),
("Resolver","解決器","参照 ・ 導出値 ・ 語を解決する","Vocabulary Resolver"),
("view","表示の文の型","値を人が読む文にする型","無い"),
("page template","ページテンプレート","kind ごとの節の並びと部品","無い"),
("catalog","部品のカタログ","部品と状態ごとの振る舞い","無い"),
("design system","デザインシステム","トークンの値とコントラスト","無い"),
("schema name and version","スキーマの名前と版","所有者 ・ 種類 ・ 版の名前","protocol version など"),
("published record","公開の記録","公開した版の名前と指紋","Contract Registry"),
("approval","承認","人が確かめた記録（層の外）","承認（04:42）")]
OWN=['基盤', '基盤', '基盤', '基盤', '基盤', '基盤', 'Skill', 'Skill', 'Skill', 'Skill', 'Skill', 'Skill', 'Skill（値）', 'Skill（値）', '基盤 ・ 外（AI ・ 人）', '基盤', '基盤', '基盤（形）', '基盤', '基盤', '基盤（形）・ Skill', '基盤（形）・ Skill', '基盤（形）・ Skill', '基盤（形）・ Skill', '基盤', '基盤', '基盤の外（人）']
table('q1-concepts','抽象の層の概念27件の名前と持ち主と、構想 v0.3 との対応',["英語","日本語","持ち主","何を表すか","構想 v0.3（参考）"],[230,170,170,280,180],[[c[0],c[1],o,c[2],c[3]] for c,o in zip(C,OWN)],emph=(0,),row_h=34)

# ── Q1 採らない構想 v0.3 の概念 ──
N=[(["Semantic Model","（Entity ・ Property ・ Type）"],["docs 自身が Entity を必須にしない。抽象スキーマと","別にモデルを持つと、同じものに2つのモデルができる"]),
("Interpreter ・ Interpretation",["検証 ・ 調停 ・ 実行の3つの意味を1語に載せている","（Q5 で validator ・ Semantic Validator ・ Resolver に分けた）"]),
("Contract（examples の kind）",["Concrete スキーマと同じものを指す。名前を1つにする"]),
(["Relation / Graph Engine と","Constraint Engine を分けること"],["Q3 で1つにまとめた。正本は JSON で、Graph は","Resolver の中の作業の形"]),
(["Projection","（JSON-LD ・ SHACL への投影）"],["Q2 で投影しないと決めた。概念だけを使う"]),
(["検証の4つの側面と","capability の4区別"],["docs 自身が議論の視点としている。","機械的 ／ セマンティックの2つで足りる"]),
("Vocabulary の階層 ・ 登録 ・ 衝突",["docs に階層の記述が無く、登録と衝突は未決。","いま要るのは id ・ 意味 ・ 値の形 ・ 単位だけ"])]
table('q1-notadopted','構想 v0.3 から採らない概念と理由',["構想 v0.3 の概念","採らない理由"],[320,500],[[a,b] for a,b in N],emph=(),row_h=48)

# ── Q6 スキーマの一覧 ──
S=[("meta-schema.json","abstract@v1","Concrete スキーマの形","Skill"),
("annotations.schema.json","annotations@v1","x- キーワードの形","Skill（値）"),
("view.schema.json","view@v1","x- キーワードの形","Skill（値）"),
("（足す）x-term の形","（Q7 で決める）","x- キーワードの形","Skill（値）"),
("page.schema.json","page@v1","Skill が書くファイルの形","Skill"),
("catalog.schema.json","catalog@v1","Skill が書くファイルの形","Skill"),
("design-system.schema.json","design-system@v1","Skill が書くファイルの形","Skill"),
("（足す）vocabulary.schema.json","vocabulary@v1","Skill が書くファイルの形","Skill"),
("published.schema.json","published@v1","基盤の記録の形","基盤のコマンド")]
table('q6-schemas','基盤の契約のスキーマの一覧と、4つの群',["ファイル","$id（schema-driven/…）","群","インスタンスを書く人"],[250,190,220,160],[list(s) for s in S],emph=(0,),row_h=34,note="validation result と capability の形は抽象スキーマに置かず、protocol（Q4）に置く")

# ── Q6 境界の直し ──
B=[(["6つが、自分は Concrete スキーマ","として書かれている"],["準拠と所有を分ける。7つとも meta-schema に準拠し、","所有は基盤（名前の頭 schema-driven/）"]),
("自分の検査で13件の指摘",["除外の道を作らず、description と x-prompt を","足して0件にする"]),
(["注釈の一覧がメタスキーマ（5）と","コード（7）で違う"],["x- キーワードの表の1か所にまとめる。","コードはこの表を読む"]),
(["annotations ・ view と meta-schema を","つなぐのがコードだけ"],["x- キーワードの表が、定義のスキーマを","$ref で指す"]),
("x-ref の label を誰も読まない",["消す。名前の出どころは x-view の label の1か所"]),
("7つとも公開の記録が無い",["版を上げずに足したあと、公開の前に1回記録する"])]
table('q6-fixes','基盤の契約のスキーマの境界の崩れと直し方',["いまの崩れ","直し方"],[380,440],[[a,b] for a,b in B],emph=(),row_h=48)

# ── Q7 x- キーワードの表 ──
K=[("x-abstract","根","機械的","Schema Validator","description の文を直す"),
("x-generates","根","機械的","Schema Validator","置き場所を create と照らす"),
("x-prompt の read","書く単位","形だけ","Schema Validator","中身は検証しない（案内）"),
("x-prompt の write","書く単位","両方","by が名指す validator","機械で決まる rule は形へ"),
("x-ref","プロパティ","機械的","Resolver ・ Checker","label を消し、矛盾を形で止める"),
("x-derive","オブジェクト","機械的","Resolver ・ Checker","3つを schema_rules に足す"),
("x-view","プロパティ ・ $defs","両方","描画 ・ AI","hidden を単独にする"),
("x-states","カタログの部品","両方","catalog.rs ・ AI","表に載せる。照合は AI"),
("x-vocabularies","根","機械的","Resolver","足す"),
("x-term","プロパティ","両方","Resolver ・ AI","足す"),
("rule の by","$defs/rule","機械的","schema_rules","足す（validator の名前）"),
("rule の requires","$defs/rule","機械的","Semantic Validator","足す（必要な capability）"),
("rule の required","$defs/rule","機械的","Semantic Validator","足す（必須か）"),
("x-annotations","根","—","—","消す（使う所が0件）")]
table('q7-keywords','x- キーワードの表：置く場所 ・ 検証の種類 ・ validator ・ 扱い',["x- キーワード","置く場所","検証の種類","validator","扱い"],[170,150,100,190,230],[list(k) for k in K],emph=(0,),row_h=34)
# はみ出した項目を2行に分け、列を広げた。
# 第38回：利用者の指摘で、q1-model を持ち主（基盤 ・ Skill ・ 基盤の外）の枠で描き直した（このファイルの外で直接書いた）。
