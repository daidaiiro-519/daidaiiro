# 第46回の図：Q1 を、概念モデル ・ 不変条件 ・ エンジンの能力の3つで組み直す。
# 材料は advisors/ddd-domain-model.json（ddd-advisor）と measured/invariants-inventory.json（実装の洗い出し）。
import runpy
T = runpy.run_path('make13_table.py')
table, e = T['table'], T['e']

# ── 概念モデルの図 ──
def chip(o, x, y, w, t, cls="box"):
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="30"/><text class="label" x="{x+10}" y="{y+20}">{e(t)}</text>')

o = ['<svg viewBox="0 0 960 500" style="min-width:860px" role="img" aria-label="Semantic Schema Core の9つの概念と3つの集約">']
o.append('<rect class="box" x="120" y="16" width="260" height="48"/><text class="label" x="132" y="37">Abstract スキーマ（版）</text><text class="note" x="132" y="55">値オブジェクト。名前@vN で決まる</text>')
o.append('<rect class="area" x="16" y="110" width="424" height="208"/><text class="title" x="30" y="134">Concrete スキーマの集約</text>')
chip(o, 30, 150, 396, "Concrete スキーマ（集約のルート）", "box focus")
chip(o, 30, 190, 396, "rule（id ・ rule ・ why ・ test）")
chip(o, 30, 230, 396, "公開した版（値オブジェクト）")
chip(o, 30, 270, 396, "term（terms@v1 の $defs。欄は $ref で指す）")
o.append('<path class="flow" d="M250,110 V64"/><text class="note" x="262" y="92">x-abstract で申告する</text>')
o.append('<rect class="area" x="16" y="370" width="424" height="118"/><text class="title" x="30" y="394">instance の集約</text>')
chip(o, 30, 406, 396, "instance（集約のルート）", "box focus")
chip(o, 30, 446, 396, "承認の記録（承認した時点のハッシュ）")
o.append('<path class="flow" d="M250,370 V318"/><text class="note" x="262" y="350">$schema で指す</text>')
o.append('<rect class="area" x="540" y="110" width="404" height="118"/><text class="title" x="554" y="134">view の集約</text>')
chip(o, 554, 150, 376, "view（集約のルート）", "box focus")
o.append('<text class="note" x="554" y="206">ページテンプレート ・ カタログ ・ デザインシステム</text>')
o.append('<path class="flow" d="M540,165 H440"/><text class="note" x="490" y="157" text-anchor="middle">x-view を読む</text>')
o.append('<rect class="box" x="526" y="406" width="418" height="70"/><text class="label" x="540" y="428">検証結果（値オブジェクト。保存しない）</text><text class="note" x="540" y="448">rule 1件ごとに、5つの値のどれか1つ</text><text class="note" x="540" y="466">verified ・ violated ・ unverified ・ unsupported ・ error</text>')
o.append('<path class="flow" d="M440,421 H526"/><text class="note" x="483" y="413" text-anchor="middle">検証で返る</text>')
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
 ("7", "Concrete スキーマ", ["申告していない x- を使わず、", "基盤の名前を申告しない"], ["S3 ・ F9", "（F9 は未検査）"], "check-schemas"),
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
