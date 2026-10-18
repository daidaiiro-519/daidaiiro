# 第46回の図：Q1 を、概念モデル ・ 不変条件 ・ エンジンの能力の3つで組み直す。
# 材料は advisors/ddd-domain-model.json（ddd-advisor）と measured/invariants-inventory.json（実装の洗い出し）。
import runpy
T = runpy.run_path('make13_table.py')
table, e = T['table'], T['e']

# ── 概念モデルの図 ──
def chip(o, x, y, w, t, cls="box"):
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="30"/><text class="label" x="{x+10}" y="{y+20}">{e(t)}</text>')

o = ['<svg viewBox="0 0 960 660" style="min-width:860px" role="img" aria-label="Semantic Schema Core の概念と4つの集約">']
o.append('<rect class="box" x="90" y="16" width="340" height="48"/><text class="label" x="102" y="37">Semantic Abstract Schema（一員7つ）</text><text class="note" x="102" y="55">値オブジェクト。名前@vN で決まる</text>')
o.append('<rect class="area" x="16" y="110" width="424" height="248"/><text class="title" x="30" y="134">Concrete スキーマの集約（人が承認する）</text>')
chip(o, 30, 150, 396, "Concrete の版（下書き → 承認待ち → 公開 → 引退）", "box focus")
chip(o, 30, 190, 396, "rule（判定する者 ・ 読む欄 ・ why ・ test）")
chip(o, 30, 230, 396, "validator の割り当て（Concrete に1つ）", "box focus")
chip(o, 30, 270, 396, "term（terms@v1 の $defs。欄は $ref で指す）")
o.append('<text class="note" x="30" y="342">承認は依頼の時点の中身のハッシュに結び付く</text>')
o.append('<path class="flow" d="M250,110 V64"/><text class="note" x="262" y="92">$schema で申告する</text>')
o.append('<rect class="area" x="16" y="410" width="424" height="120"/><text class="title" x="30" y="434">instance の集約</text>')
chip(o, 30, 446, 396, "instance（従う版1つ ・ 変えてよい範囲）", "box focus")
o.append('<text class="note" x="30" y="512">状態は持たず、読むたびに決定表で導く</text>')
o.append('<path class="flow" d="M250,410 V358"/><text class="note" x="262" y="390">$schema で指す</text>')
o.append('<rect class="box" x="526" y="110" width="418" height="70"/><text class="label" x="540" y="132">view（値オブジェクト）</text><text class="note" x="540" y="152">ページテンプレート ・ カタログ ・ デザインシステム</text><text class="note" x="540" y="170">読むときに Semantic Abstract Schema で検証する</text>')
o.append('<path class="flow" d="M526,165 H440"/><text class="note" x="483" y="157" text-anchor="middle">x-view を読む</text>')
for (y, t, sub) in [(220, "検証の記録の集約", "検証の記録（rule ごとの入力と結果）"), (318, "人の判断の集約（人だけが足す）", "人の判断（合っている ・ 合っていない）")]:
    o.append(f'<rect class="area" x="526" y="{y}" width="418" height="80"/><text class="title" x="540" y="{y + 24}">{t}</text>')
    chip(o, 540, y + 36, 390, sub, "box focus")
o.append('<path class="flow" d="M526,260 H496 V470 H440"/>')
o.append('<path class="flow" d="M526,372 H470 V490 H440"/>')
o.append('<rect class="box" x="526" y="430" width="418" height="100"/><text class="label" x="540" y="452">状態を導く（ドメインサービス）</text><text class="note" x="540" y="472">rule の効いている結果：未記入 → 人の判断 → いまの結果</text><text class="note" x="540" y="492">いまの結果：入力がいまと一致する最後の記録</text><text class="note" x="540" y="512">instance の状態：決定表で導く（下書き ・ 判断待ち など）</text>')
o.append('<rect class="box" x="526" y="560" width="418" height="56"/><text class="label" x="540" y="582">検証結果（値オブジェクト）</text><text class="note" x="540" y="602">verified ・ violated ・ unverified ・ unsupported ・ error</text>')
o.append('<text class="note" x="16" y="648">集約は4つ。人が承認するのは Concrete の版と割り当てだけで、instance には承認を付けない</text>')
o.append('</svg>')
open('q1-domain.src.svg', 'w').write('\n'.join(o))

# ── 不変条件の表 ──
INV = [
 ("1", "Abstract スキーマ", ["Abstract の版は名前で1つに決まり、", "公開した版は中身を変えない"], "S1 ・ S2", "check-schemas"),
 ("2", "Concrete スキーマ", ["1つの Abstract の版を申告し、", "その版の契約を満たす"], "M（版の行）", "check-schemas"),
 ("3", "Concrete スキーマ", ["$id は <名>/<種類>@v<n> で、", "文脈の中で一意で、変えない"], "M（$id）・ S11", "check-schemas"),
 ("4", "Concrete スキーマ", "title と description を持つ", "M", "check-schemas"),
 ("5", "Concrete スキーマ", ["instance を作る Concrete は、", "kind と $schema の const を持つ"], ["M ・ F7", "（F7 は未検査）"], "check-schemas"),
 ("6", "Concrete スキーマ", ["書く単位ごとに description と", "x-prompt を持ち、葉には書かない"], ["M ・ S4 ・ W7", "（W7 は未検査）"], "check-schemas"),
 ("7", "Concrete スキーマ", ["x- を定義できるのは Abstract だけで、", "文脈は x- を定義しない"], ["S3 ・ F9", "（F9 は未検査）"], "check-schemas"),
 ("8", "Concrete スキーマ", ["参照 ・ 導出 ・ 描画の宣言は、", "読める形である"], "S6〜S10", "check-schemas"),
 ("9", "Concrete スキーマ", "参照される種類は id を持つ", "S12", "check-schemas"),
 ("10", "rule", ["id ・ rule ・ why ・ test を持ち、", "id はスキーマの中で一意である"], "M22〜M26 ・ S5", "check-schemas"),
 ("11", "term", ["欄の意味は term への $ref で決まり、", "1つの term を2つの意味で使わない"], "いま無い", "（いま無い）"),
 ("12", "公開した版", ["書き換えない ・ 消さない ・", "末尾に足すだけ"], ["P1〜P5", "（P3〜P5 は未検査）"], "check-schemas"),
 ("13", "instance", ["$schema の名前で、", "Concrete 1つに解決する"], "I1〜I4", ["create ・ update ・", "check"]),
 ("14", "instance", "同じ場所に2つ作らない", "I6", "create"),
 ("15", "instance", ["更新は、読んだ時点のハッシュと", "一致するときだけ成り立つ"], "I8", "update"),
 ("16", "instance", ["Concrete を満たし、", "未記入は違反と分けて持つ"], "I10", "update ・ check"),
 ("17", "instance", ["x-ref の指す先があり、1つに決まり、", "受け付ける値である"], "R1〜R5", "check"),
 ("18", "instance", ["関係の数の制約（covered_by ・", "unique ・ inverse）を満たす"], "R6〜R10", "check"),
 ("19", "instance", ["x-derive の宣言値が導出値と", "一致し、expect を満たす"], "D1 ・ D2", "check"),
 ("20", "承認の記録", ["検証エラー ・ ずれ ・ 未記入が無いときだけ", "承認し、承認した時点のハッシュを持つ"], "A1〜A6 ・ R11", "approve ・ check"),
 ("21", "検証結果", ["rule 1件ごとに5つの値のどれかを返し、", "検証できなかった rule も返す"], "いま無い", "（いま無い）"),
 ("22", "view", ["形とコントラストを満たし、", "失敗したら1ページも書かない"], "V1〜V21", "render"),
]
# 概念ごとに固めて並べる。概念の欄は、その概念の行をまたぐ1つの枠にする
def grouped(name, label, heads, widths, rows, row_h=46):
    X=[16]
    for w in widths[:-1]: X.append(X[-1]+w+8)
    W=X[-1]+widths[-1]+16; H=32+len(rows)*(row_h+6)+8
    o=[f'<svg viewBox="0 0 {W} {H}" style="min-width:{min(W,900)}px" role="img" aria-label="{e(label)}">']
    for x,h in zip(X,heads): o.append(f'<text class="note" x="{x+4}" y="22">{e(h)}</text>')
    i=0
    while i<len(rows):
        j=i
        while j+1<len(rows) and rows[j+1][1]==rows[i][1]: j+=1
        y=32+i*(row_h+6); h=(j-i+1)*(row_h+6)-6
        o.append(f'<rect class="box focus" x="{X[1]}" y="{y}" width="{widths[1]}" height="{h}"/><text class="title" x="{X[1]+10}" y="{y+h//2+5}">{e(rows[i][1])}</text>')
        i=j+1
    for i,r in enumerate(rows):
        y=32+i*(row_h+6)
        for c in [0]+list(range(2,len(widths))):
            cell=r[c]; lines=cell if isinstance(cell,list) else [cell]
            o.append(f'<rect class="box" x="{X[c]}" y="{y}" width="{widths[c]}" height="{row_h}"/>')
            if len(lines)==1: o.append(f'<text class="label" x="{X[c]+10}" y="{y+row_h//2+5}">{e(lines[0])}</text>')
            else:
                for k,l in enumerate(lines): o.append(f'<text class="label" x="{X[c]+10}" y="{y+row_h//2-4+16*k}">{e(l)}</text>')
    o.append('</svg>'); open(name+'.src.svg','w').write('\n'.join(o))
grouped('q1-invariants', 'Semantic Schema Core の不変条件22行を概念ごとに固めた表と、守る command',
      ["#", "概念", "不変条件", "command"], [40, 140, 420, 190],
      [[r[0], r[1], r[2], r[4]] for r in INV])

# ── エンジンの能力の表 ──
CAP = [
 ("作成", "create", ["Abstract から Concrete の雛形、", "Concrete から instance"], "コマンド", "2 ・ 13 ・ 14"),
 ("取得（Query）", "get（JMESPath）", "instance ・ Concrete", "クエリ", "──"),
 ("更新", ["update（JSON Patch。", "項目の削除は remove）"], "instance", "コマンド", "15 ・ 16"),
 ("削除", "delete", "instance", "コマンド", ["15。指されている", "instance の削除は未検査"]),
 ("案内", "prompt", "Concrete の x-prompt", "クエリ", "6"),
 ("検証", ["check（instance）", "check-schemas（スキーマ）"], "すべての集約", "クエリ", "全行"),
 ("解決", "単独のコマンドは無い", ["$schema → Concrete、$ref → term、", "x-ref → 指す先"], "中で使う", "11 ・ 13 ・ 17"),
 ("承認", "approve", "承認の記録", "コマンド", "20"),
 ("描画", "render", "instance と view", "クエリ", "22"),
 ("公開", "無い", "公開した版を末尾に足す", "コマンド", "12"),
 ("走査 ・ 索引", "無い（Scan ・ Index）", "instance の集まり", "クエリ", "──"),
]
table('q1-capabilities', 'エンジンの能力11と、いまのコマンドと、対象と、守る不変条件',
      ["能力", "いまのコマンド", "対象", "種別", "守る不変条件"], [110, 200, 270, 90, 170],
      [list(r) for r in CAP], emph=(0,), row_h=46)

# ── 第52回：Semantic Abstract Schema の一員（第67回に tool permissions の形を足して7つ）──
MEM = [
 ("メタスキーマ", ["x- キーワードの定義と、", "スキーマの書き方"], "定義する", "する", "一員ごとに名前@vN", "──"),
 ("ページテンプレートの形", ["kind ごとの節 ・ 部品 ・", "値を取る式"], "使う", "する", "一員ごとに名前@vN", "メタスキーマの版"),
 ("部品のカタログの形", "部品の一覧の枠", "使う", "する", "一員ごとに名前@vN", "メタスキーマの版"),
 ("デザインシステムの形", "トークンの形", "使う", "する", "一員ごとに名前@vN", "メタスキーマの版"),
 ("公開の記録の形", ["公開した版の名前と指紋"], "使う", ["しない", "（文脈が読むなら、する）"], "エンジンの版", "メタスキーマの版"),
 ("判定の記録の形", ["入力のハッシュ ・ 結果 ・", "根拠 ・ 判定した者"], "使う", ["しない", "（文脈が読むなら、する）"], "エンジンの版", "メタスキーマの版"),
 ("tool permissions の形", ["呼んでよい外のツールの名前 ・", "引数 ・ 権限"], "使う", "する", "一員ごとに名前@vN", "メタスキーマの版"),
]
table('q6-abstract', 'Semantic Abstract Schema の一員7つ：基盤が持ち、基盤だけが変えるスキーマ',
      ["一員", "何の形か", "x- を", "公開するか", "版", "従う x- 定義の版"], [200, 220, 90, 190, 160, 160],
      [list(r) for r in MEM], emph=(0,), row_h=46)
