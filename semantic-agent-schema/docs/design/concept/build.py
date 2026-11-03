"""Semantic Agent Schema の概念のページを作る。土台・設計のページと同じ作り（タブ・結論の1文・図・カード・details の表）。
図は design-svg の figure と chart exchange で作り、ER 図と3段の型は src/ の検査済みの図を使う。"""
import os, re, json, subprocess, html
H = os.path.dirname(os.path.abspath(__file__))
R4 = os.path.join(H, 'src')  # 検査済みの図と頁（3段の型・ユースケースの表）
DS = os.path.normpath(os.path.join(H, '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg'))
os.makedirs(os.path.join(H, 'fig'), exist_ok=True)
REPORT = []
import figman
WORDFIX = [('検査器', 'バリデーター'), ('差し込む物の契約', '受け取りの型'), ('アダプターで差し込む', 'アダプターとして渡す'), ('差し込んだ', '渡した'), ('差し込む', '渡す'), ('組み立てる', 'コンポジションルートを置く'), ('組み立て', 'コンポジションルート'), ('見分けた', '判別した'), ('見分ける', '判別する'), ('見分け', '判別し'), ('頼み、', '依頼し、'), ('頼む', '依頼する'), ('組む', '作る'), ('道具', 'ツール'), ('依存が要る', '依存が必要'), ('要ると確かめたら', '必要と確かめたら'), ('要らない', '必要ない'), ('札の付いた', 'ラベルの付いた'), ('右の札の', '右のラベルの')]
def wfix(t):
    for a, b in WORDFIX: t = t.replace(a, b)
    return t

def run(kind, name, data):
    src = os.path.join(H, 'fig', name + '.json'); out = os.path.join(H, 'fig', name + '.svg')
    json.dump(data, open(src, 'w'), ensure_ascii=False, indent=1)
    cmd = [DS, 'figure', src, '--out', out] if kind == 'figure' else [DS, 'chart', 'exchange', src, '--out', out]
    p = subprocess.run(cmd, capture_output=True, text=True)
    REPORT.append((name, (p.stdout + p.stderr).strip().splitlines()[0] if (p.stdout + p.stderr).strip() else ''))
    return open(out).read()

def svg_fit(s, label, k=1.1):
    # 本文の列に収める。広い図は列の幅に縮め、狭い画面では図の面の中で横にスクロールする
    m = re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', s); w = float(m.group(1))
    s = re.sub(r'<svg[^>]*?(?=\s(?:viewBox|class|role))', '<svg', s, count=1)
    s = re.sub(r'\s(width|height|style)="[^"]*"', '', s[:s.find('>') + 1], count=0) + s[s.find('>') + 1:]
    minw = min(int(w), 760)
    s = s.replace('<svg', f'<svg width="100%" style="min-width:{minw}px;max-width:{int(w * k)}px;display:block;margin:0 auto" aria-label="{html.escape(label)}"', 1)
    return s

def fig(s, label, hint='', k=1.1):
    return f'<div class="fig">{svg_fit(s, label, k)}</div>' + (f'<p class="hint">{hint}</p>' if hint else '')

# ---------- 図 ----------
F = {}
F['types'] = open(os.path.join(R4, 'f1.svg')).read()
F['flow'] = run('exchange', 'flow', {"label": "意味の部分を含む操作の流れ", "participants": ["main", "別のフォルダ", "SAS"], "steps": [
    {"from": "main", "to": "別のフォルダ", "label": "立てて目的を渡す（使う側）"},
    {"from": "別のフォルダ", "to": "SAS", "label": "create --semantic"},
    {"from": "SAS", "to": "別のフォルダ", "label": "手順（x-prompt の rules）"},
    {"from": "別のフォルダ", "to": "SAS", "label": "create --value"},
    {"from": "SAS", "to": "別のフォルダ", "label": "断った理由か created"},
    {"from": "別のフォルダ", "to": "main", "label": "最後の答え"},
    {"from": "main", "to": "別のフォルダ", "label": "続き（再開する）"}]})
SC = {
 'a': ("エージェント", [("エージェント", "SAS", "U10 を作る（--semantic）"), ("SAS", "エージェント", "書く手順（記録しない）"), ("エージェント", "SAS", "create --value で欄の値"),
       ("SAS", "SAS", "検査して書く・DB へ o-0031"), ("SAS", "エージェント", "created とセッションの ID"), ("エージェント", "SAS", "U10 を検証（--semantic）"),
       ("SAS", "エージェント", "判定の手順（記録しない）"), ("エージェント", "エージェント", "別の文脈で判定"), ("エージェント", "SAS", "--answer で判定"), ("SAS", "SAS", "突き合わせて DB へ o-0032"), ("SAS", "エージェント", "結果")]),
 'b': ("エージェント", [("エージェント", "SAS", "範囲と問い（--semantic）"), ("SAS", "エージェント", "探す手順（記録しない）"),
       ("エージェント", "エージェント", "範囲の文書を読んで答える"), ("エージェント", "SAS", "--answer で答えと出どころ"),
       ("SAS", "SAS", "出どころを確かめる"), ("SAS", "エージェント", "答えと出どころの値")]),
 'c': ("人", [("人", "人", "直接書き換え"), ("人", "SAS", "U10 を検証"), ("SAS", "SAS", "ハッシュが違う"),
       ("SAS", "人", "通らない変更と案内"), ("人", "SAS", "--adopt --reason"), ("SAS", "SAS", "DB へ o-0043")]),
 'd': ("人", [("人", "SAS", "U11 を削除（--reason）"), ("SAS", "SAS", "従うインスタンスが無いかを見る"), ("SAS", "SAS", "DB へ o-0040"),
       ("SAS", "人", "o-0040")]),
 'e': ("エージェント", [("エージェント", "SAS", "U10 を更新（--semantic）"), ("SAS", "エージェント", "書く手順（記録しない）"),
       ("エージェント", "SAS", "update --patch"), ("SAS", "エージェント", "断った理由（形が誤り）"), ("エージェント", "SAS", "直して打ち直す"),
       ("SAS", "SAS", "書く・o-0050"), ("SAS", "エージェント", "updated"),
       ("エージェント", "SAS", "U10 を検証（--semantic）"), ("SAS", "エージェント", "変わった所の判定の手順")]),
}
for k, (who, steps) in SC.items():
    parts = [who, "SAS"]
    F['sc' + k] = run('exchange', 'sc' + k, {"label": "場面 " + k, "participants": parts, "steps": [{"from": a, "to": b, "label": l} for a, b, l in steps]})
F['link'] = run('figure', 'link', {"direction": "LR", "nodes": [
    {"id": "u", "label": "使う側の Concrete（例：目的のセッション）：目的・親子・状態・コミット", "role": "kind-2"},
    {"id": "cli", "label": "SAS の CLI"},
    {"id": "s", "label": "SAS の記録（手元の DB・作り直せる）", "role": "focus"}],
    "edges": [{"from": "u", "to": "cli", "label": "セッションの ID（--session ・ 環境変数 ・ semantic-agent-schema init）"}, {"from": "cli", "to": "s", "label": "記録する"},
              {"from": "s", "to": "u", "label": "セッションと操作の ID を返し、使う側が自分のインスタンスに書く", "dashed": True}]})
F['uc'] = run('figure', 'uc', {"direction": "TB", "nodes": [
    {"id": "u0", "label": "UC0 その型で成果物を作り、検証された状態で使い続ける（要約・使う側）", "role": "kind-2"},
    {"id": "u1", "label": "UC1 作成する"}, {"id": "u2", "label": "UC2 更新する"}, {"id": "u3", "label": "UC3 削除する"},
    {"id": "u4", "label": "UC4 検証する（変形：意味の判定）"}, {"id": "u5", "label": "UC5 描画する"}, {"id": "u6", "label": "UC6 検索する"},
    {"id": "u7", "label": "UC7 操作の記録を読む"}],
    "edges": [{"from": "u0", "to": x} for x in ["u1", "u2", "u3", "u4", "u5", "u6", "u7"]]})


F['er'] = run('figure', 'er16', {"direction": "TB", "nodes": [
    {"id": "s", "label": "sessions：id・tool・folder・via・first_seen", "role": "focus"},
    {"id": "ch", "label": "changes：id・at・kind・target_path・前後のハッシュ・reason・request_id・source・session_id・schema・worktree"},
    {"id": "fc", "label": "field_changes：change_id・pointer・change・before・after"},
    {"id": "v", "label": "validations：id・at・target_path・hash・session_id・schema"},
    {"id": "rr", "label": "rule_results：validation_id・rule・pointer・by・result・reason・reads_hash"}],
    "edges": [{"from": "s", "to": "ch", "label": "session_id"}, {"from": "s", "to": "v", "label": "session_id"},
              {"from": "ch", "to": "fc", "label": "1 対 多"}, {"from": "v", "to": "rr", "label": "1 対 多"}]})

# ---------- src/ の頁から details に入れる表 ----------
v5 = open(os.path.join(R4, 'index.html')).read()
def v5sec(i):
    a = v5.find(f'id="{i}"'); b = v5.find('<section', a + 5); return v5[a:b if b > 0 else len(v5)]
def tables(frag):
    return re.findall(r'<div class="tw">.*?</table></div>', frag, re.S)

def tbl(head, rows):
    return '<div class="tw"><table><thead><tr>' + ''.join(f'<th>{h}</th>' for h in head) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</tbody></table></div>'
def cards(items):
    return '<div class="cards">' + ''.join(f'<div class="card"><b>{t}</b><span>{d}</span></div>' for t, d in items) + '</div>'
def det(summary, body):
    return f'<details><summary>{summary}</summary>{body}</details>'


# ---------- 見つける・設計する（事業領域・文脈・ドメインモデル・アーキテクチャ・スキーマ） ----------
def rd(n): return open(os.path.join(H, 'fig', n + '.svg')).read()
for n in ['ctxmap']: F[n] = rd(n)
def jd(n): return json.load(open(os.path.join(H, 'fig', 'base', n + '.json')))
def nl(d, i, label=None, role=None):
    for x in d['nodes']:
        if x['id'] == i:
            if label: x['label'] = label
            if role is not None: x['role'] = role
SUB = [("書いて検証する（モジュール writing・validation）", "中核", "ドメインモデル（集約 Concrete・インスタンスは writing、集約を読んで rule に照らす業務サービスと判定の区分は validation）", "UC1 作成・UC2 更新・UC3 削除・取り込み・UC4 検証・入力が変わった rule だけ。意味の部分は x-prompt の欄を書くことと x-prompt の rule を判定すること", "人・AI のエージェント（打ったセッション）／Git（読むだけ）・JSON Schema のバリデーター／Abstract・Concrete・インスタンスのファイルと判定の結果"),
       ("記録（モジュール recording）", "補完", "トランザクションスクリプト", "UC7 操作の記録を読む（semantic-agent-schema log。読むだけ）と、入口が使う「誰が打ったかを決めて sessions に登録する」手続き。変更と検証の追記は、書く ・ 検証 ・ 描画がそれぞれのポートで行う", "人・AI のエージェント・レビューする人／ツールのセッション ID／sessions・changes・field_changes・validations・rule_results"),
       ("検索（モジュール search）", "補完", "トランザクションスクリプト", "UC6（範囲を宣言して、構造で絞る ・ 意味で探す。--answer で出どころを確かめる。記録しない）", "人・AI のエージェント・オーケストレーター／Abstract・Concrete・インスタンスのファイル／書かない"),
       ("描画（モジュール rendering）", "補完", "トランザクションスクリプト", "UC5（デザインシステムとインスタンスの値を受け取り、テンプレートが値を置き、トークンが見た目を決める。同じ入力から同じページ）", "読み手／デザインシステム／書かない"),
       ("版の管理・JSON Schema の検証・描画の部品", "一般", "製品は作らずに使い、アダプターとして渡す。ポートと受け取りの型は、使うモジュールが SAS の中に持つ", "―", "Git・バリデーター・テンプレートの処理系")]
WORDS = [("同じ言葉（文脈「スキーマとインスタンス」）", "セッション（semantic-agent-schema を打ったツールのセッション ID。sessions の1行で指す。状態は持たない）・ハッシュ（中身の sha256。記録の列 before_hash・after_hash の値）・書いた変更・検証・変わった欄・rule の結果・ハッシュ・理由（reason）・依頼（task）・手順のプロンプト・エージェントの答え"),
         ("意味が違う（境界の外）", "ツールの「セッション」の寿命と再開（ツールと使う側が持つ。SAS は ID を記録するだけ）・Git の「コミット」・使う側の「目的のセッション」（目的・親子・承認・コミットを持つ）")]
INV = [("集約 Concrete（書くのモジュール）", "① 申告する Abstract の版は閉じた一覧にある（Abstract の enum）② x-published の欄を同じ版のまま変えない"),
       ("集約 Instance（書くのモジュール）", "① id を変えない ② x-decided-by が human の欄は人の入口からだけ変える（導く値は、Concrete の JSON Schema の if ・ then ・ else が形の確認で見る）"),
       ("組み込みの検査（検証のモジュールの業務サービスが判定）", "JSON Schema で書けない Concrete の決まり3つ：$schema の const と $id が同じ（instance-schema-name）・rule の id が重ならない（unique-id）・例の related のキーは rule の related_fields にある所だけ（example-related）。x-prompt の rule ではなく、Concrete の検証のたびに SAS が確かめる。x-prompt の rule はどれも打ったセッションが判定して --answer で渡す（判定できなければ unverified）。人だけが変える欄は書く前に断る。x- の閉じた一覧は語彙の propertyNames（JSON Schema）が見る"),
       ("書く手順の条件（書くのモジュールの use_cases）", "読んだ時のハッシュといまのハッシュが一致したときだけ書く・形の確認・機械の検証・ハッシュの一致がそろったときだけ書く・最後の操作の事後のハッシュ（値で受け取る）と違えば SAS を通らない変更として知らせる"),
       ("sessions の行（集約ではない）", "追記だけ・1つのセッションに1行・via は env ・ flag ・ init のどれか（DDL・trigger）"),
       ("記録の行（集約ではない）", "追記だけ・request_id は1回（DDL・trigger・一意の索引）")]
CMD = [("Concrete", "作る／版を変える（公開した版は変えない）"),
       ("インスタンス", "作る／直す（ハッシュを確かめる）／消す（ほかの文書は変えない。Concrete は従うインスタンスがあれば止める）／取り込む（--adopt）"),
       ("意味の手順（集約ではない。use_cases の手順）", "作って返す（記録しない）／渡された値と判定を、形 ・ ハッシュ ・ step ・ rule ・ pointer で確かめる")]
VAL = "書くのモジュール：ハッシュ・差分（欄ごとの前後）・書く条件の外れ・文書が Concrete かインスタンスか（TargetKind）・リポジトリの文書の一覧（Corpus）・誰が打ったか（Caller。セッションのツール ・ フォルダ ・ ID の出どころ via を持つ）・記録にある変更の行（ChangeRow ・ WriteRecord）。検証のモジュール：判定の結果（判定の5つの区分と、何を verified に数えるか）・誰が判定したか（形の検査と組み込みの検査は machine、rule は ai）・記録にある検証と判定の行（ValidateRow ・ RuleRow ・ Judgment）。複数のモジュールが使う値は、ここ（domain）に置く。業務イベントは作らない（購読する手続きが無い）"
SVC = [("当たる rule・Abstract の rule（検証のモジュール）", "集約を読んで rule に照らす"), ("判定が読んだ所のハッシュ reads_hash（検証のモジュール）", "書く ・ 検証が、入力が変わった前の判定を同じ基準で決める"), ("欄の定義を引く field_schema（書くのモジュール）", "スキーマの所に当たる欄の title ・ description ・ x-prompt を引く。書く ・ 検証の手順が使う")]
PORTS = [["document_store", "書く ・ 検証 ・ 描画 ・ 検索", "outbound/filesystem（WorkingTree）", "書くは読む ・ 書く ・ 消す、ほかは読むだけ。書くのは書くのモジュールの use_cases だけ"], ["schema_validator", "書く ・ 検証 ・ 描画 ・ 検索", "outbound/jsonschema（Validator）", "2020-12 の形の検査。入れて出すスキーマは書く ・ 検証、注釈は書くだけが使う。x- の判定は validation-domain"], ["commit_reader", "書く", "outbound/git", "コミットにある中身を読むだけ"], ["change_log", "書く", "outbound/sqlite（SqliteLog）", "変更を足す ・ request_id を引く ・ 経路を書いた操作 ・ rule ごとの最後の判定"], ["validation_log", "検証", "outbound/sqlite（SqliteLog）", "検証を足す ・ 経路を書いた操作 ・ rule ごとの最後の判定"], ["validation_log", "描画", "outbound/sqlite（SqliteLog）", "デザインシステムの検証を足すだけ"], ["template_renderer", "描画", "outbound/templates", "Mustache（mustache/spec の必須の部分）の処理系。値を置く・繰り返す・有無で出し分けるだけで、ロジックを持たない。描かずに木を読み、検証に使う"], ["script_reader", "描画", "outbound/javascript", "JS のパーサー boa。テンプレートの JS が値の入口 sas-data から読む欄を取り出す"], ["page_writer", "描画", "outbound/filesystem（OutDir）", "描いたページを書き出す。入力と実体の経路で重なれば断り、1つでも置けなければどれも置かない"], ["identify_caller", "記録（入口だけが呼ぶ）", "inbound/cli", "誰が打ったか（Caller）を --session → 環境変数（callers.json）→ このフォルダで semantic-agent-schema init が登録したセッションの順で決める"], ["session_log", "記録（入口だけが呼ぶ）", "outbound/sqlite（SqliteLog）", "sessions に1行足す ・ semantic-agent-schema init の登録を引く ・ 新しいセッション ID を作る"], ["operation_reader", "記録", "outbound/sqlite（SqliteLog）", "記録にある操作を新しい順に絞って読む・1件の中身を読む（UC7。読むだけ）"]]
LIBS = [["rusqlite", "outbound/sqlite の crate だけが依存", "行の型で変換する（orphan rule）"], ["2020-12 のバリデーターのライブラリ", "outbound/jsonschema の crate だけが依存", "schema_validator を実装。$ref はネットワークから取らない。x- の判定は domain"], ["テンプレートの処理系のライブラリ", "outbound/templates の crate だけが依存", "template_renderer を実装。種類がそろうかは application が確かめる"]]
RULES_ANY = ["リポジトリの根は製品 Semantic Agent Schema の置き場で、文脈の根にしない。文脈の根は、文脈の名前のフォルダ（リポジトリの根の直下。ここでは schemas-and-instances/）。その中は業務領域を作るモジュール（writing・validation・recording・rendering・search）と adapters で、文脈の根に domain を置かない。名前はその文脈の言葉（スキーマ・インスタンス・書く・検証・記録）から付け、製品の名前を使わない。文脈の中はポートとアダプターで作り、外とのやりとりはすべて application の ports を adapters が実装する（呼び出し元を決めるポートは inbound/cli、ほかは outbound）。本ではポートは業務ロジック層が定義するが、記録と描画はドメインモデルを持たないので application に置く", "業務ロジックの置き方は、上から決めて下から確かめる。守る条件がすでに決定・Abstract の rule・試験の場面に書けている所は初めから domain に置き、それ以外は use_cases にトランザクションスクリプトで書く。引き上げの表で判別し、突き合わせで確かめたものだけを domain に置く", "domain の中は部品の種類でフォルダを分ける：aggregates/（集約ごと）・values/・services/。購読する手続きが無いうちは events/ を作らない", "application の中は use_cases/ と ports/。use_cases/ のファイルは集約か手順のまとまりごとに分け、ユースケースごとのファイルやフォルダは作らない（土台の境界6）", "domain を持つのは、突き合わせで集約・値・サービスが確かめられたモジュールだけ（いまは書く・検証）。集約は書くのモジュール、検証のモジュールは集約を読んで rule に照らすサービスと判定の値を持つ。記録・描画・検索は application だけ", "ポートは、それを使うユースケースを持つモジュールが、自分の application/ports に、使うメソッドだけで宣言する。共有の置き場を作らない。同じ形のポート（document_store ・ schema_validator）もモジュールごとに宣言し、1つのアダプターがそれらを実装する。記録のポートも追記するモジュールが宣言する（書く change_log、検証と描画 validation_log。検索は持たない）", "モジュールの application は、ほかのモジュールの application（ポートもユースケースも）に依存しない。依存してよいのは domain の crate だけ（validation-domain → writing-domain の一方向）。複数のモジュールが使う判断と値は domain に置く", "誰が打ったか（Caller）は入口が決めて sessions に登録し、値でユースケースへ渡す。決める手続きは記録のモジュールに置き、入口だけが呼ぶ", "断る理由の型（Refused）は各モジュールが持つ。出力の形（kind = error）は inbound/cli が持ち、各モジュールの Refused を移す", "外のライブラリ（rusqlite・2020-12 のバリデーター・テンプレートの処理系）は、それを使う outbound の crate だけが依存する", "コンポジションルート（アダプターをポートへ渡す）は入口の1か所だけ", "公開された言葉（Abstract の本体・語彙・方言と CLI の入出力・手順と答えの形）は文脈の根の published-language/&lt;版&gt;/ に置き、出した版は変えない"]
RULES_RS = ["依存の向きを強制したい境界と、外のライブラリを閉じ込めたい所だけを crate にし、それ以外はモジュールで分ける。crate の数の基準は原典に無い<span class=\"src\">（Book ch07-01:186・ch14-03:187（基準の記述が無い）・ch07-03:296・Reference visibility:476）</span>", "domain はモジュールごとに crate を分ける（いまは writing-domain と validation-domain。検証が集約を読む向きを crate で強制する）。1つの crate の中では子モジュールが祖先の項目を使えるので、集約の間の向きはモジュールでは止められない<span class=\"src\">（Book ch07-03:296・ch14-03:299）</span>", "根の Cargo.toml は virtual manifest。[workspace] に members と resolver = \"3\" を書く<span class=\"src\">（Cargo workspaces:241・258、Book ch14-03:207〜211）</span>", "外のライブラリの版は [workspace.dependencies] に置き、各 crate は workspace = true で継承する（optional にはできない）。SAS の中の crate どうしは path dependencies<span class=\"src\">（Cargo workspaces:359・363、specifying-dependencies:425）</span>", "crate が使えるのは自分の Cargo.toml に書いた crate だけ。これで domain の無依存と、ライブラリの閉じ込めを強制する<span class=\"src\">（Book ch14-03:299・390）</span>", "domain の lib.rs は pub mod aggregates; pub mod values; pub mod services;（集約の無い validation-domain は values と services だけ。events は作らない）。crate の中だけで使う部品は pub(crate)<span class=\"src\">（Book ch07-02:273、Reference visibility:390・476）</span>", "mod.rs を使わない。src/aggregates.rs と src/aggregates/concrete.rs・instance.rs の形にする<span class=\"src\">（Book ch07-05:251・265・268）</span>", "adapters は、実装するポートの application と、ポートで受け渡す型を持つ domain の crate に依存する（outbound/sqlite は記録の行の型のため writing-domain ・ validation-domain にも）。application は domain の型を再公開しない", "コンポジションルートは cli の main.rs。outbound の crate に依存してよいのは cli だけ<span class=\"src\">（Book ch14-03:299（推論：実装の型の名前を書くので依存が必要））</span>", "ポートは trait で、どれも dyn で使う。dyn compatible に保つ：型引数のあるメソッド・async fn・関連定数を持たず、Self を receiver 以外で使わない。sealed にしない<span class=\"src\">（Reference traits:412・429・445・459、API Guidelines C-OBJECT:322・324・321、C-SEALED:169）</span>", "集約の欄は private、メソッドだけ pub。ID・ハッシュは newtype で区別する<span class=\"src\">（Book ch07-03:477、API Guidelines C-STRUCT-PRIVATE:218・C-NEWTYPE:169）</span>", "adapters で外の trait（rusqlite・serde など）を domain の型に実装しない（orphan rule）。sqlite の crate は自分の行の型を持って変換する<span class=\"src\">（Book ch10-02:302・309・313、Reference implementations:658・659・669）</span>", "依存のグラフは循環させない<span class=\"src\">（Cargo resolver:568・585）</span>"]
TREE = [["semantic-agent-schema/", "リポジトリ ＝ 製品 Semantic Agent Schema（文脈ではない）。直下に文脈の名前のフォルダと、製品全体のものだけを置く", ""], ["├── Cargo.toml", "［製品全体］virtual manifest：[workspace] members に下の crate のパス（ラベルの付いたフォルダ。いま15）・resolver = \"3\"・[workspace.dependencies]", ""], ["├── architecture.json", "［製品全体］crate の間で許す依存の向きと crate の名前の決め方（検査に使う）", ""], ["├── docs/", "［製品全体］設計の文書（土台・設計のページ）", ""], ["└── schemas-and-instances/", "文脈「スキーマとインスタンス」（区切られた文脈）。中の作り方はポートとアダプター。中は業務領域を作るモジュールと adapters", ""], ["    ├── published-language/v1/", "公開された言葉：Concrete を書く型（abstract ・ vocabulary ・ dialect）・ CLI の出力（command-result）・ 打ったセッションへの手順（procedure）・ 打ったセッションが --answer で渡す答えの形（judgment-answer ・ search-answer）・ 受け取りの型（design-system ・ template-language）の9つ。操作の依頼の型は JSON で依頼するインターフェースを作るときに足す", ""], ["    ├── writing/", "業務領域「書いて検証する」の書くモジュール", ""], ["    │   ├── domain/", "依存：無し。aggregates/（Concrete・インスタンスと書く条件）・values/（ハッシュ・差分・書く条件の外れ・TargetKind・Corpus・Caller・変更の行）・services/（field_schema）", "writing-domain"], ["    │   └── application/", "use_cases/write.rs（作る・直す・取り込む）・write/delete.rs・write/patch.rs・write/semantic.rs・ports.rs（document_store・schema_validator・commit_reader・change_log）", "writing-application"], ["    ├── validation/", "業務領域「書いて検証する」の検証のモジュール", ""], ["    │   ├── domain/", "依存：writing-domain。values/（判定の5つの区分・rule の結果・by・検証と判定の行）・services/（当たる rule・Abstract の rule・reads_hash）", "validation-domain"], ["    │   └── application/", "use_cases/validate.rs（機械で検証する・入力が変わった rule だけ）・meaning.rs（意味の判定の手順を作る・判定を突き合わせる）・ports.rs（document_store・schema_validator・validation_log）", "validation-application"], ["    ├── recording/", "業務領域「記録」のモジュール（トランザクションスクリプト。domain は無い）", ""], ["    │   └── application/", "use_cases/history.rs（誰が打ったかを決めて登録する。入口だけが呼ぶ・UC7 を読む）・ports.rs（identify_caller・session_log・operation_reader）", "recording-application"], ["    ├── rendering/", "業務領域「描画」のモジュール（domain は無い）", ""], ["    │   └── application/", "use_cases/render.rs・design.rs・tokens.rs・ports.rs（document_store・schema_validator・template_renderer・script_reader・page_writer・validation_log）", "rendering-application"], ["    ├── search/", "業務領域「検索」（補完）のモジュール（domain は無い。記録しない）", ""], ["    │   └── application/", "use_cases/search.rs（範囲を読む・構造で絞る・意味で探す手順を作る・答えの出どころを確かめる）・ports.rs（document_store・schema_validator）", "search-application"], ["    ├── adapters/", "アダプターの置き場", ""], ["    │   ├── inbound/cli/", "[[bin]] の名前は semantic-agent-schema（製品の名前はここにだけ残す）。main.rs がコンポジションルート。caller.rs（identify_caller）・output.rs（出力の形と Refused）・callers.json（セッションの ID の環境変数）。outbound に依存してよいのはここだけ", "inbound-cli"], ["    │   └── outbound/", "filesystem（document_store・page_writer）・git（commit_reader。読むだけ）・jsonschema（schema_validator）・sqlite（change_log・validation_log・session_log・operation_reader。migrations/）・templates（template_renderer）・javascript（script_reader）", "outbound-&lt;技術&gt;"], ["    ├── system-tests/", "システムテストの場面（scenarios/）・材料（fixtures/）・trace.json・untested.json", ""], ["    └── test-support/", "試験の部品。dev-dependencies からだけ", "test-support"]]
CRATES = [["writing-domain", "domain", "無し"], ["validation-domain", "domain", "writing-domain（集約を読む）"], ["recording-application", "application", "writing-domain ・ validation-domain（Caller と記録の行の型）"], ["validation-application", "application", "writing-domain ・ validation-domain"], ["writing-application", "application", "writing-domain ・ validation-domain（前の判定を検証と同じ基準で決める）"], ["rendering-application", "application", "writing-domain ・ validation-domain"], ["search-application", "application", "writing-domain"], ["outbound-filesystem", "adapters", "書く ・ 検証 ・ 描画 ・ 検索の application（document_store）・ rendering-application（page_writer）"], ["outbound-git", "adapters", "writing-application（commit_reader。読むだけ）"], ["outbound-jsonschema", "adapters", "書く ・ 検証 ・ 描画 ・ 検索の application（schema_validator。2020-12 のバリデーターのライブラリはここだけ）"], ["outbound-sqlite", "adapters", "書く ・ 検証 ・ 描画 ・ 記録の application（記録のポート）・ writing-domain ・ validation-domain（記録の行の型）。rusqlite はここだけ"], ["outbound-templates", "adapters", "rendering-application（template_renderer。Mustache の処理系）"], ["outbound-javascript", "adapters", "rendering-application（script_reader。boa はここだけ）"], ["inbound-cli", "adapters（bin・コンポジションルート）", "application と outbound のすべて（main.rs でコンポジションルートを置く。outbound に依存してよいのはここだけ）"]]
tree_html = ('<div class="legend"><span class="crate">crate</span> Cargo.toml を持つフォルダ。依存の単位で、名前は右のラベルのとおり（いま15個）。ラベルの無いフォルダは、まとめるだけのフォルダか、crate の中のモジュール</div>' + '<div class="fig"><div class="tree">' + ''.join(f'<div class="tl"><code>{html.escape(p)}</code><span>' + (f'<span class="crate">crate</span><code class="cn">{html.escape(c)}</code> ' if c else '') + f'{html.escape(n)}</span></div>' for p, n, c in TREE) + '</div></div>')
SCHG = [("Concrete を書くための型", "SAS → Concrete を書く人と AI ・ バリデーター", [("abstract.schema.json", "Abstract の本体。Concrete に書ける x- の4つの値の形（$defs）と、根 ・ 欄 ・ rule の形。JSON Schema で書けることは JSON Schema で書き、書けない3つは SAS の組み込みの検査にする"), ("vocabulary.schema.json", "語彙。入れ子に書ける x- の3つの形と、x- の閉じた一覧（propertyNames）"), ("dialect.schema.json", "方言。2020-12 に SAS の語彙を足す（$dynamicAnchor: meta で入れ子に掛ける）")]),
 ("CLI の出力", "SAS → 呼び出し元", [("command-result.schema.json", "操作の結果。change ・ validation ・ rule-result ・ render ・ session（semantic-agent-schema init）・ operation（semantic-agent-schema log の一覧の1行）・ search ・ error（打ったセッションの ID を持つ）。semantic-agent-schema log --operation は change ・ validation の形で返す")]),
 ("打ったセッションへの手順", "SAS → 打ったセッション", [("procedure.schema.json", "手順。SAS が Concrete の rule と当たる所（検索は範囲の文書）から組み、--semantic で結果の行の代わりに出す（記録しない）。operation_kind は validate ・ write ・ search。command に値を渡すコマンド、answer に値の形（format ・ schema）を持つ。判定の steps はレビューのまとまり（所のまとまり review-place：pointer ・ before ・ current ・ changes ・ rules、文書全体のまとまり review-document）で、rules は id で根の rules を指す。根の review に since（前の判定の操作）と scope（changed ・ all-rules ・ first）。schema は形の本体（書くときは書く元のスキーマ。$ref の先は $id を持ったまま $defs に入れる）")]),
 ("打ったセッションの答え", "打ったセッション → SAS", [("judgment-answer.schema.json", "semantic-agent-schema validate --answer で渡す判定（hash ・ results）。手順の answer.schema に中身が入る。書く値は --value（文書全体）か --patch（JSON Patch）"), ("search-answer.schema.json", "semantic-agent-schema search --answer で渡す答え（answer ・ sources の path ・ pointer ・ hash）。手順の answer.schema に中身が入る")]),
 ("受け取りの型", "使う側 → SAS", [("design-system.schema.json", "デザインシステムの受け取りの型（x-view の種類ごとのテンプレート ・ トークン）"), ("template-language.json", "テンプレートの書き方の契約（Mustache）")]),
 ("作らないと決めたもの", "", [("glossary", "使うステップが無い"), ("command-request", "SAS の入力は CLI の引数で、JSON で依頼するインターフェースが無い。インターフェースを作るときに足す"), ("session-request", "セッションを立てないので持たない"), ("session ・ diff", "command-result の中（$defs）"), ("settings", "使う設定が無い")])]
XTERMS = [("x-prompt", "根と欄", "meaning（値を何として読むか）・ when（いつ、何のために読むか）・ rules（rule の並び。rule は id ・ rule ・ why ・ check ・ bad ・ good ・ related_fields。bad ・ good は例の1つ以上の並びで必須。例は value（当たる所の値）・ related（related_fields にある所の値）・ note（なぜ。bad では必須））。文書の外を見ないと判定できない決まりは rule にせず、欄の description に書く。meaning ・ when ・ related_fields の値は打ったセッションへの手順に入る"),
 ("x-decided-by", "欄", "人が決める欄（human）。セッションの ID がある呼び出しの変更を断る"),
 ("x-published", "欄", "同じ版（$id）のあいだ変えない欄"),
 ("x-view", "根", "描画の種類の名前。デザインシステムのテンプレートの種類と合わせる")]
MAPW = [("CLI の引数（--session ・ --reason ・ --request-id）", "sessions ・ changes（reason）"), ("環境変数（callers.json）・ semantic-agent-schema init", "sessions（id ・ tool ・ folder ・ via）"),
 ("judgment-answer の results（--answer）", "rule_results（by = ai）"), ("command-result の diff", "changes の前後のハッシュ・field_changes"), ("書いた ・ 検証した文書の $schema", "changes.schema ・ validations.schema")]
MAPR = [("sessions", "session（打ったセッションの ID）・ semantic-agent-schema init の結果（id ・ tool ・ folder ・ via）"), ("ビュー operations（新しい順に絞る）＋sessions", "operation（semantic-agent-schema log の一覧の1行）"), ("changes＋field_changes", "diff（change・operation の中）・ 判定の手順のレビューのまとまり（before ・ changes）"),
 ("rule_results", "rule-result（reads_hash）"), ("changes の最後の after_hash", "validation の unrecorded・error"), ("changes＋field_changes ・ validations＋rule_results（1件）と schema", "change ・ validation（semantic-agent-schema log --operation。schema は target の concrete）")]
FACT = [
 ("JSON Schema の出力の形", "json-schema.org Core 2020-12 の12章", "欄名はすべて在った。一覧に absoluteKeywordLocation と error（単数）を足した。出力の形は実装に SHOULD"),
 ("$vocabulary・$dynamicAnchor・$dynamicRef", "Core 2020-12・メタスキーマ", "方言に $dynamicAnchor: meta を置く。true の語彙を知らないバリデーターは方言を断る（MUST）"),
 ("JSON Patch", "RFC 6902", "op・path・value・from と op の6つが在った"),
 ("Design Tokens", "Format Module 2025.10", "版を固定。「群」を原典の group に直した。別名は {group.token}"),
 ("MCP の inputSchema", "MCP 2026-07-28", "根は type: \"object\"、外の $ref は解かれない。ツールごとに1つのスキーマへ束ねる"),
 ("コーディングエージェント5つ", "Claude Code・Codex CLI・Gemini CLI・Copilot CLI・Kiro CLI の公式文書", "セッションの ID を環境変数で取れるのは Claude Code だけ。caller.env を任意にし、無いときは --session か semantic-agent-schema init（2026-11-01 に Codex・Copilot・Kiro で実測、Gemini・Cursor は原文）"),
 ("SQLite", "sqlite.org", "CHECK は NULL を違反にしない。NULL になりうる欄の = ・ IN を IS ・ IS NOT NULL で閉じた（実測でも確かめた）。最低の版 3.8.0・3.38.0、RAISE は文字列のリテラル"),
 ("Git の掃除", "git-scm.com（2.52.0 で最後に更新された頁）", "指されないオブジェクトは gc.pruneExpire（既定2週間）で消える。SAS は版を残さない（Git は読むだけ）ので、この条件は設計に関わらない")]
UNREAD = "Claude Code の sessions.md は 2026-11-01 に取得して照合した（同じセッションを2か所で再開すると記録が混ざる・--fork-session）。取得していない頁：Gemini の automation のチュートリアル、Codex の app-server、Copilot の SDK"
def flow(name, steps):
    ns = [{"id": f"s{i}", "label": f"{t}：{sc}", "role": ("focus" if tb else None)} for i, (t, sc, tb) in enumerate(steps)]
    for n in ns:
        if n["role"] is None: del n["role"]
    return run('figure', name, {"direction": "TB", "nodes": ns, "edges": [{"from": f"s{i}", "to": f"s{i+1}"} for i in range(len(steps) - 1)]})
F['fv'] = flow('fv', [("CLI の入力", "semantic-agent-schema validate の引数（--semantic か --answer）", False), ("セッションを決める", "--session ・ 環境変数 ・ semantic-agent-schema init → sessions", True), ("SAS を通らない変更", "changes の最後の after_hash と比べる", True),
  ("形の検査と組み込みの検査", "バリデーター → rule-result（by: machine）", False), ("--semantic", "変更の記録から作ったレビューのまとまりの手順（procedure。すべては --all-rules）を出して終わる。記録しない", False),
  ("--answer", "judgment-answer の形 → ハッシュ → step ・ rule ・ pointer。違えば invalid_answer か conflict", False), ("記録", "validations・rule_results（1つのトランザクション）", True), ("CLI の最後の行", "command-result の validation（session）", False)])
F['fu'] = flow('fu', [("CLI の入力", "semantic-agent-schema update --semantic --task（--ref）", False), ("セッションを決める", "sessions", True), ("SAS を通らない変更", "違えば直さずに断る", True),
  ("--semantic", "procedure（write）を出して終わる。command は update --patch … --if-match。記録しない", False),
  ("--patch", "打ったセッションが JSON Patch で渡す。ステップの欄だけ", False), ("ハッシュを確かめて書く", "--if-match と違えば conflict。一時ファイルで置き換え、前後の中身から差分を求める", False), ("記録", "changes・field_changes（diff から）", True), ("CLI の最後の行", "command-result の change（session・hint）", False)])
F['overview'] = figman.overview(); F['layers'] = figman.layers(); F['hex'] = figman.hexf(); F['crates'] = figman.cratesf()
F['stages'] = figman.stages(); F['model'] = figman.model(); F['datamap'] = figman.datamap(); F['bnd'] = figman.bnd()
REPORT += [(n, next((x for x in r if '検査1〜4' in x), '')) for n, r in figman.REPORT]
def schg_html():
    out = '<div class="schg">'
    for g, ft, files in SCHG:
        out += f'<div class="sg"><b>{g}</b><span class="hint">{ft}</span>' + ''.join(f'<div class="sf"><code>{html.escape(f)}</code><span>{html.escape(d)}</span></div>' for f, d in files) + '</div>'
    return out + '</div>'
xt_html = '<div class="tw"><table><tr><th>x-</th><th>置く所</th><th>中身</th></tr>' + ''.join(f'<tr><td><code>{html.escape(a)}</code></td><td>{b}</td><td>{c}</td></tr>' for a, b, c in XTERMS) + '</table></div>'
TABLES5 = ["sessions", "changes", "field_changes", "validations", "rule_results"]
MAPG = [("CLI の引数", "W", "W", "", "", ""), ("procedure（出力）", "", "", "", "", ""), ("judgment-answer", "", "", "", "", "W"), ("diff", "", "WR", "WR", "", ""),
        ("session（出力）", "R", "", "", "", ""), ("operation（出力・semantic-agent-schema log）", "R", "R", "", "R", "R"), ("change（出力・semantic-agent-schema log --operation）", "", "R", "R", "", ""), ("rule-result（出力）", "", "", "", "", "WR"), ("validation（出力）", "", "R", "", "R", "R")]
map_grid = (f'<div class="need" style="--n:5"><div></div>' + ''.join(f'<div class="nh">{t}</div>' for t in TABLES5)
  + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if v else ""}">{ {"W":"書く","R":"読む","WR":"書く・読む"}.get(v,"") }</div>' for v in vs) for r, *vs in MAPG) + '</div>')
T_NEW = {
 'domain': ('domain', '事業領域と文脈',
  '<p class="ans">事業領域（AI のエージェントが扱う構造化データを、スキーマに従って書き、検証すること）を分解して業務領域を見つけ、SAS 自身の4つを、意味の境界の内側の広い1つの文脈「スキーマとインスタンス」に置く。一般の4つと使う側は外の文脈。</p>'
  + fig(F['layers'], '事業領域・業務領域・区切られた文脈', '業務領域は見つけるもの、文脈は設計するもの。区分は仮で、確かめるときに実装の方法から逆に判定する。')
  + fig(F['ctxmap'], '文脈の地図', '中核を含む文脈「スキーマとインスタンス」は、ツールにはセッション ID の環境変数（callers.json）だけで従属し、エージェントを起動しない。Git と JSON Schema にも従属する。使う側へは共用サービスとして、公開された言葉（Abstract と CLI の出力）だけを出す。')
  + cards([("意味の境界", "ツールのセッションの寿命と再開・Git の「コミット」・使う側の「目的のセッション」は文脈「スキーマとインスタンス」の語と意味が違う"),
           ("ポートと製品", "一般の能力は、ポートと受け取りの型を SAS が持ち、製品はアダプターとして渡す"),
           ("切り出す契機", "描画が別のデザインシステムや別のチームを持ったら、下流の文脈へ")])
  + det('業務領域（区分・実装の方法・受け持つユースケース・手がかり）', tbl(['業務領域', '区分', '実装の方法', 'ユースケース', 'アクター／外部システム／データ'], SUB))
  + det('同じ言葉の一覧（と、境界の外で意味が違う語）', tbl(['範囲', '語'], WORDS))),
 'model': ('model', 'ドメインモデル',
  '<p class="ans">集約は2つ。Concrete・インスタンスで、書くのモジュール（writing/domain）に置く。検証のモジュールは集約を読んで rule に照らす。上から決め、下から確かめて同じ形になったものだけを domain に置く。</p>'
  + fig(F['model'], '集約・値・サービス', 'writing-domain は何にも依存しない。validation-domain は集約を読む。意味（x-prompt の rule・x-prompt の欄を書く）は検証・書くの中にあり、業務領域にしない。記録・描画・検索はドメインモデルを持たない（業務ロジックは use_cases のトランザクションスクリプト）。点線の箱は use_cases に残すもの。')
  + fig(F['datamap'], 'データとドメインモデルの対応', 'ファイルの中身は Git。記録は集約にせず、追記する行のまま。')
  + cards([("2つの集約", "Concrete・インスタンス。1回に書く集約は1つ。sessions と記録は追記する行"),
           ("業務イベントは作らない", "購読する手続きが無い。現れたら足す"),
           ("SAS を通らない変更", "インスタンスの集約が判別する。最後のハッシュは use_cases が記録から読んで値で渡す")])
  + det('不変条件', tbl(['集約', '不変条件'], INV)) + det('コマンド', tbl(['集約', 'コマンド'], CMD))
  + det('値オブジェクトと業務サービス', '<p class="hint">値：' + VAL + '</p>' + tbl(['業務サービス', 'すること'], SVC))),
 'arch': ('arch', 'アーキテクチャ',
  '<p class="ans">文脈の中をポートとアダプターで作る。内側に domain と5つの application、縁に各モジュールが自分で宣言するポート、外にアダプター6つと入口の CLI。application はほかのモジュールの application に依存せず、domain だけに依存する。依存の向きは crate で強制する。</p>'
  + fig(F['hex'], 'ポートとアダプター', 'ポートは、使うユースケースを持つモジュールの application が宣言し、アダプターが実装する（本では業務ロジック層が定義する。記録と描画はドメインモデルを持たないので application に置く）。同じ形のポートもモジュールごとに宣言し、1つのアダプターが複数のモジュールのポートを実装する。誰が打ったかは入口が決めて、値で渡す。外のライブラリはそれを使う outbound の crate だけが依存する。')
  + fig(F['crates'], 'crate の層', 'crate はいま15（domain 2・application 5・outbound 6・cli 1・test-support 1）。名前は &lt;モジュール&gt;-&lt;層&gt; と &lt;inbound|outbound&gt;-&lt;技術&gt;（文脈の名前は付けない）。application の間に線は無く、どれも domain だけに依存する。outbound に依存してよいのはコンポジションルートを持つ cli だけ。')
  + tree_html
  + cards([("ポートとアダプター", "データアクセスはレイヤードの語で、ここには無い。外の I/O はスクリプトのモジュールもポートを通す"),
           ("crate とモジュール", "向きを強制したい境界（業務領域 × 層）とライブラリを閉じ込める所だけ crate。集約・値・サービス・ユースケースはモジュール"),
           ("ポートは dyn", "どれも dyn で使い、dyn compatible に保つ（型引数のあるメソッド・async fn・関連定数を持たない）"), ("ポートは使う側が宣言する", "モジュールごとに、使うメソッドだけ。記録のポートも表の側から作らない")])
  + det('crate と依存してよい向き', tbl(['crate', '層', '依存してよい crate'], CRATES))
  + det('ポートの一覧', tbl(['ポート（会話の目的）', '宣言するモジュール', 'アダプター（実装する型）', '受け渡すもの'], PORTS))
  + det('外のライブラリを閉じ込める所', tbl(['ライブラリ', 'どの crate が依存するか', '扱い'], LIBS))
  + det('言語に依存しない決まり', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in RULES_ANY) + '</ul>')
  + det('Rust の決まり（一次情報の頁と行）', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in RULES_RS) + '</ul><p class="hint">原文は 2026-10-31 に取得して照合した（The Rust Programming Language・The Cargo Book・The Rust Reference・Rust API Guidelines。識別子33個・引用51件が原文と一致）。crate の数の基準は原文に無い。</p>')),
 'schema': ('schema', 'スキーマと DB',
  '<p class="ans">公開された言葉は published-language/v1 の9つ（Concrete を書く型3 ・ CLI の出力1 ・ 打ったセッションへの手順1 ・ そのセッションが渡す答えの形2 ・ 受け取りの型2）。JSON Schema で書けることは JSON Schema で書き、x- と SAS の rule は書けないことだけにする。DB は5表とビューで公開せず、first_seen のほかのどの欄も CLI の出力の形のどこかに出る。</p>'
  + '<h3>自分の $id を持つリソースにするか、$defs の部品にするか</h3><div class="tw"><table><tr><th>問い</th><th>はい</th><th>いいえ</th></tr><tr><td>1. その形を1つの文書として名指しする相手がいるか（$schema で示す ・ 別の文書の欄から URN で指す ・ 外のツールへ文書で渡す）</td><td>2へ</td><td>使うリソースの $defs に置く。複数で使うなら意味の持ち主の $defs に置き、ほかは $id#/$defs/名前 で指す</td></tr><tr><td>2. すでにあるリソースと、読み手も変わる理由も同じか</td><td>同じリソースに置く</td><td>自分の $id を持つリソース（ファイル）にする</td></tr></table></div><p class="hint">ファイルは運ぶ単位で、線はリソース（$id）にある（JSON Schema 2020-12 Core 8.2.4 ・ 9.3）。手順（procedure）は打ったセッションが行う指示で、結果を返す command-result と役目が違うので別のリソース。Concrete を書く人への作法は Abstract の rule：one-document-kind（1つの Concrete に1種類の文書）・ no-shape-only-concrete（properties に $schema のほかの欄を1つ以上持たせる）・ shape-in-defs（インスタンスの根にならない形は $defs）・ ref-not-copy（ほかの Concrete の形は写さず $ref）。</p>'
  + fig(F['bnd'], 'スキーマの境界', 'JSON で表したい・型で制御したい・意味を持たせたい・決まった型で返したい、のどれかに当たる境界だけをスキーマにする。DB の表と版の管理・バリデーター・テンプレートの処理系はプロセスの外へ JSON が出ないのでスキーマにしない。')
  + schg_html()
  + cards([("Concrete を書く型", "Concrete に書ける x- は4つ（x-prompt ・ x-decided-by ・ x-published ・ x-view）。x- を増やさず、意味の決まりは x-prompt の rule に書く。参照のパスを必要とする Concrete は、普通の欄（format: uri-reference と pattern）として書く"), ("答えの形", "判定の答え（judgment-answer）と検索の答え（search-answer）は、打ったセッションが従って書くので公開された言葉に置く。手順の answer.schema にも中身が入る。値を渡すコマンドは手順の command"), ("公開しない", "DB の DDL。出力では DB の名前を直し、DDL を変えても出力のスキーマは変わらない")])
  + '<h3>Concrete を書くための x- 項目</h3>' + xt_html
  + '<h3>意味の部分を含む操作の流れ（どの段でどのスキーマと表を使うか）</h3><div class="two"><div>' + fig(F['fv'], '検証の流れ', '検証。濃い段は DB を書く・読む段。', 1.2) + '</div><div>' + fig(F['fu'], '更新の流れ', '更新。答えの各項目はステップの番号と JSON ポインタを持ち、SAS はステップの欄だけへ書く。', 1.2) + '</div></div>'
  + '<p class="hint">CLI の出力は1行1つの JSON（JSON Lines）。--semantic を付けた操作は、結果の行の代わりに手順（procedure）を出し、記録しない。手順を行うのは打ったセッションで、command のコマンドで値か判定を渡す。SAS はセッションを立てないので、閉じる処理は無い。</p>'
  + '<h3>スキーマと DB の対応</h3><div class="fig">' + map_grid + '</div><p class="hint">DB にだけある欄は sessions の first_seen だけ。スキーマにだけあるのは、記録しないと決めたもの（描画・検索の入力、当てる前の値、決めきれず返した点、検索の答え、ファイルが正本の設定）だけ。</p>'
  + det('公開された言葉のファイルと、作らないと決めたもの', ''.join(tbl(['ファイル', '中身'], [[f'<code>{html.escape(f)}</code>', d] for f, d in files]) for g, ft, files in SCHG))
  + det('書く向きの対応', tbl(['スキーマ', 'DB'], MAPW)) + det('読む向きの対応（出力で直す名前）', tbl(['DB', '出力'], MAPR))
  + '<h3>原典と照合した結果（2026-10-31）</h3>' + tbl(['項目', '原典（版）', '結果と直した所'], [[f'<span class="tag t-chk">照合済み</span> {a}', b, c] for a, b, c in FACT]) + f'<p class="hint">原文は取得日と sha256 と一緒に残した。{UNREAD}。</p>'),
}


# ---------- 確かめる：上から決めて下から確かめる（版14） ----------
_IA = json.load(open(os.path.join(H, '..', 'advisors', 'impl-ddd.json')))
BOTTOM_FULL = [[wfix(c).replace('ドメインサービス', '業務サービス').replace('schema_checker', 'schema_validator') for c in r] for r in _IA['code'][1]['rows']]
for r in BOTTOM_FULL:
    r[0] = r[0].replace('どの UC も：操作を追記する', '作成・更新・削除・検証・取り込み：操作を追記する')
    r[3] = r[3].replace('再開の決まり6つ', '再開の決まり5段')
    r[2] = r[2].replace('再開の決まり6つ', '再開の決まり5段')
TOPDOWN = [("スキーマへの適合", "ドメインモデル（中核）", "版は閉じた一覧・本体の x- の名前を足さない・id を変えない・導出値の一致・人の欄は人の入口から・ハッシュの一致。作成・更新・削除・取り込みの4つの経路で同じく守る"),
           ("意味の操作", "ドメインモデル（中核）", "セッションの型ごとの不変条件・再開の決まり5段・やり直しの決まり（決めたことの行）"),
           ("操作の記録", "トランザクションスクリプト（補完）", "追記だけ・request_id は1回を DDL・trigger・一意の索引が守る。手続きは行の出し入れ"),
           ("描画", "トランザクションスクリプト（補完）", "テンプレートに値を入れて返すだけで、書かない")]
BU = [("UC1 作成（機械）", "Concrete・インスタンス", "版・URN・ハッシュ", "指す先の実在", "形の検査（ポート）"),
      ("取り込み（--adopt）", "インスタンス（ハッシュ・人の欄）", "差分", "", "$schema を決める・衝突を返す"),
      ("UC2 更新", "同じ2つ（同じ理由で変わる）", "判定の結果", "", "版の移し方"),
      ("UC3 削除", "", "", "", "従うインスタンスがあれば止める"),
      ("UC4 検証", "", "判定の結果（判定の5つの区分）", "", "範囲の選び方"),
      ("意味の手順（版14 の SF1）", "SAS が立てたセッション（版16 でやめた）", "", "", "手順を作って返す・答えを突き合わせる"),
      ("UC5 描画", "", "", "", "描く（判定は使わない）"),
      ("UC6 検索", "", "", "", "探す"),
      ("UC7 読む", "", "差分を使う", "", "記録を読む"),
      ("操作を追記する", "", "", "", "行を足す")]
bu_grid = ('<div class="need" style="--n:4"><div></div>' + ''.join(f'<div class="nh">{h}</div>' for h in ["集約", "値オブジェクト", "業務サービス", "スクリプトのまま"])
    + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if v else ""}" style="font-size:12px">{v}</div>' for v in vs) for r, *vs in BU) + '</div>')
RECON = [("業務領域ごとの実装の方法", "一致", "―"), ("集約（Concrete・インスタンス・SAS が立てたセッション）", "一致", "―"),
         ("差分", "上の見落とし", "値オブジェクトにする（UC2・取り込み・意味の書き込み・UC7 が使う）"),
         ("SAS を通らない変更の検出", "上の置き違い", "操作の記録からインスタンスの集約へ移す（最後のハッシュは値で受け取る）"),
         ("形の検査と x- の判定", "上の見落とし", "形の検査は schema_validator のポートの向こう、x- の判定は集約とサービス"),
         ("業務イベント（events/）", "上の先取り", "作らない。購読する手続きが現れたら足す"),
         ("スキーマへの適合の中のスクリプト", "上の先取り", "検索・版の移し方などは use_cases に残す"),
         ("操作の記録の集約2つ", "上の先取り", "集約にせず、追記する行にする")]
RC = {"一致": "m-keep", "上の見落とし": "m-warn", "上の置き違い": "m-warn", "上の先取り": "m-warn"}
recon_grid = ('<div class="mx" style="grid-template-columns:minmax(0,1.3fr) minmax(0,.7fr) minmax(0,2fr)"><div class="mh">項目</div><div class="mh">判定</div><div class="mh">直す方</div>'
    + ''.join(f'<div class="mr">{a}</div><div class="mc {RC[b]}"><b>{b}</b></div><div class="mc m-none">{c}</div>' for a, b, c in RECON) + '</div>')
T_VERIFY = ('verify', '確かめる',
  '<p class="ans">上から決めた構成を、ユースケースの手続きから下から確かめた（版14 の記録）。業務領域ごとの実装の方法と3つの集約は一致し、構成は妥当。食い違い6件は上からの設計を直した。いまの集約は2つ（Concrete・インスタンス）。</p>'
  + fig(F['stages'], '探索の4つの段階と引き上げの表', '段階3でユースケースの手続きをそのまま書き、段階4で何になるかを判別する。似たものも、同じ意味・同じ理由で変わると確かめるまで引き上げない。')
  + '<h3>上から：ドメインモデルが自明な所と根拠</h3>' + tbl(['業務領域', '判定', '根拠（守る条件がすでに書けている所）'], TOPDOWN)
  + '<h3>下から：ユースケースの手続き → 何になるか</h3><div class="fig">' + bu_grid + '</div><p class="hint">緑のマスがそのユースケースの手続きで見つかったもの。手順とルールの全文は下に畳んだ。</p>'
  + '<h3>突き合わせ</h3>' + recon_grid
  + cards([("食い違いのとき", "部品の置き場所は下を、境界と公開する契約は上を採る。どちらにも実物が無ければスクリプトに残す"),
           ("自明なら上から", "守る条件がすでに決めたことや試験の場面に書けている所は、探りを入れず初めから domain に置く"),
           ("ドメインモデルは育てる", "スクリプトに残したものも、ほかのユースケースが同じ理由で使い始めたら引き上げる")])
  + det('段階3の手続きとルール（全文）', tbl(['ユースケース', '段階3：主な手順', '出てくるルール', '段階4：何になるか'], BOTTOM_FULL))
  + det('名前が同じで別のもの', tbl(['名前', '別々のもの'], [["検証", "形の検査（2020-12、一般）・機械の rule（x- の意味）・意味の rule（UC4 の変形「意味の判定」）"], ["ハッシュ", "対象のファイルのハッシュと、手順のプロンプトの prompt_hash"], ["版", "$id の v&lt;版&gt; と Git のコミット"]])))

# ---------- 決めたこと・承認 ----------

# ---------- 能力のマトリクス（HTML の格子） ----------
CAP = [("作成", "人やツールが値を書く", "x-prompt を指示に値を書く", "残る（create）"),
       ("更新", "人やツールが書き換える", "x-prompt を指示に書き換える", "残る（update）"),
       ("削除", "対象だけを消す（ほかの文書は変えない）", None, "残る（delete）"),
       ("検証", "JSON Schema と組み込みの検査3つ", "x-prompt の rule を判定する", "残る（validate と rule ごとの結果）"),
       ("描画", "テンプレートとトークンを注入し、その場で描く", None, "残らない（コミットしない）"),
       ("検索", "構造で絞る", "意味で探す", "残らない（打ったセッションの行は sessions に残る）")]
cap_grid = '<div class="mx"><div class="mh"></div><div class="mh">機械で</div><div class="mh">意味で（エージェント）</div><div class="mh">操作として</div>' + ''.join(
    f'<div class="mr">{a}</div><div class="mc m-mach">{b}</div><div class="mc {"m-sem" if c else "m-none"}">{c or "―"}</div><div class="mc {"m-keep" if d.startswith("残る") else "m-none"}">{d}</div>' for a, b, c, d in CAP) + '</div>'




# ---------- 必要十分：ユースケースのステップ × 能力 ----------
NCOL = ["セッションを決める", "範囲で絞る", "手順を作る", "答えを突き合わせる", "コミットの中身を読む", "差分を求める", "検査する", "デザインシステム", "記録を読む"]
NROW = [("UC1〜UC6 のはじめ", ["セッションを決める"]),
        ("UC1 作成する（機械）", ["検査する"]),
        ("UC1 作成する（意味）", ["手順を作る", "検査する"]),
        ("UC2 更新する", ["差分を求める", "検査する"]),
        ("UC2 の意味（手順を返し、JSON Patch で受け取る）", ["手順を作る", "差分を求める", "検査する"]),
        ("UC2 取り込み・UC4 通らない変更", ["コミットの中身を読む", "差分を求める"]),
        ("UC3 削除する", ["検査する"]),
        ("UC4 検証する", ["検査する"]),
        ("UC4 の変形：意味の判定", ["手順を作る", "答えを突き合わせる"]),
        ("UC5 描画する", ["デザインシステム"]),
        ("UC6 検索する（構造で絞る）", ["範囲で絞る"]),
        ("UC6 の意味（手順を返し、答えの出どころを確かめる）", ["範囲で絞る", "手順を作る", "答えを突き合わせる", "検査する"]),
        ("UC7 読む（一覧 ・ 1件）", ["記録を読む"])]
need_grid = (f'<div class="need" style="--n:{len(NCOL)}"><div></div>' + ''.join(f'<div class="nh">{c}</div>' for c in NCOL)
    + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if c in cs else ""}">{"●" if c in cs else ""}</div>' for c in NCOL) for r, cs in NROW)
    + '<div class="nr">使う行の数</div>' + ''.join(f'<div class="nc tot">{sum(c in cs for _, cs in NROW)}</div>' for c in NCOL) + '</div>')
NOT_HAVE = ["コミット・PR・ブランチ・履歴の検索", "版を残す・復元（Git に書き込むこと）", "目的のセッションを立てて役割ごとに任せる", "エージェントを起動する・セッションの状態を持つ（開閉・終わり方・回・再開の決まり）", "セッションの会話を複製（fork）して行う", "デザインシステムを作る・既定のデザインシステム", "バリデーターを自作する", "断った操作を記録する"]
T_NEED = ('need', '必要十分',
  '<p class="ans">ユースケースのどのステップも、SAS の契約の能力だけで行える。どの能力もどこかのステップが使い、使わない能力は持たない。</p>'
  + '<div class="fig">' + need_grid + '</div><p class="hint">● はそのステップが使う能力。最後の行が0の列は無い（余分が無い）。どの行も能力が欠けていない（不足が無い）。人が決める欄は能力を足さず、人の判定を UC4 の入力で受ける。UC7 は読むだけなので、セッションを決めず行も足さない。</p>'
  + cards([("余分", "使う行が0の能力は無い"), ("不足", "足りない能力は無い"), ("持たない", "コミット・PR、目的のセッション、エージェントの起動、デザインシステムを作ること")])
  + det('持たない能力', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in NOT_HAVE) + '</ul>'))

# ---------- タブ ----------
T = []
T.append(('overview', '全体像',
  '<p class="ans">JSON Schema を Semantic に拡張し、その型で作成・更新・削除・検証・描画・検索する基盤。どんな Concrete を作るかは使う側が決める。</p>'
  + fig(F['overview'], 'SAS と使う側と一般の製品', 'SAS は能力・記録（動いているセッションを回すため）・公開する形と、一般の能力のポートと受け取りの型まで。製品は作らずに渡す。目的ごとに進める営みは使う側。')
  + cards([("SAS の中", "能力6つ・記録・公開する形・意味としての目的・一般の能力のポートと受け取りの型"), ("使う側", "目的のセッション・承認・まとめ・コミット・PR"), ("一般の製品", "Git・バリデーター・テンプレートの処理系はアダプターとして渡す。コーディングエージェントは semantic-agent-schema を打つ側で、SAS は起動しない")])
  + det('目的の2つの読み', tbl(['目的の読み', '置き場所'], [["意味の一部（rule・欄・Concrete が何のためにあるか、1件の操作の意図）", "SAS の中：rule.why・x-prompt の when・根の description・changes.reason"], ["進め方（目的ごとのセッション・親子・承認・コミット）", "使う側"]]))))
T.append(('types', '型と能力',
  '<p class="ans">Abstract は SAS が決め、Concrete とインスタンスは使う側が決める。6つの能力はどの段にも働き、意味が関わるのは作成・更新・検証・検索。</p>'
  + fig(F['types'], '3段の型') + cap_grid
  + '<p class="hint">緑＝意味の側がある・灰＝機械だけ。「操作として」は手元の DB に残るか。</p>'
  + cards([("意味は4つ", "作成・更新・検証・検索。どれも SAS が打ったセッションに手順を返し、セッションが値か判定を渡す"), ("x-prompt の形", "meaning（値を何として読むか）・ when（いつ読むか）・ rules。場面ではなく中身の種類で分ける"), ("rule の例", "bad ・ good は value ・ related ・ note の並び。値と説明を分け、判定の境目を示す"), ("削除", "参照している所の記載も同じ呼び出しの続きで消す"), ("描画", "その場で描き、コミットしない")])))


SIM_URL = 'https://claude.ai/artifact/PkMt9GJLuAyX1zLAeMivm9'
SESS_CMDS = tbl(['誰が', 'すること', '何で'], [
  ['SAS', '打ったセッションに、意味で書く手順（x-prompt と rule）を返す', '<code>semantic-agent-schema create ・ update --semantic</code>'],
  ['SAS', '検査して書く。断ったら理由を返す', '<code>semantic-agent-schema create --value ・ update --patch</code>'],
  ['SAS', '意味の rule の判定の手順を返す（判定済みの対象は変わった所だけ）／ 判定を受け取って記録する', '<code>semantic-agent-schema validate --semantic（--all-rules）・ --answer</code>'],
  ['SAS', '最初に打ったときに sessions に1行足し、以後の操作の行から指す', 'どのコマンドでも（環境変数か --session）'],
  ['SAS', 'いまのセッションをこのフォルダに登録する（環境変数から ID が取れないツールのため）', '<code>semantic-agent-schema init</code>'],
  ['使う側', '目的を分けるとき、フォルダ（ワークツリーなど）を用意してセッションを1つ立てる', '<code>git worktree add</code> と <code>claude -p</code> など'],
  ['使う側', '同じ目的の続きを依頼する（同じセッションを再開する）', '<code>claude -p --resume</code> など'],
  ['使う側', '様子を見る ・ 画面で開く', 'ツールの出力 ・ <code>semantic-agent-schema log --folder ・ --by</code>（記録を読むだけ）・ <code>claude --resume</code>']])
SESS_ROW = tbl(['sessions の列', '中身'], [
  ['id', 'セッション ID（環境変数か --session）'], ['tool', 'どのツールか（どの環境変数から拾ったか）'],
  ['folder', 'そのセッションが動いているフォルダ（ワークツリーを切っていれば、その根）'], ['first_seen', '初めて記録した時刻']])
T.append(('session', 'セッション',
  '<p class="ans">SAS はエージェントを起動しない。打ったセッションの ID を拾って sessions に1行で記録し、操作の行から指すだけ。最初に semantic-agent-schema を打ったときに、そのフォルダでの記録が始まる（環境変数から ID が取れないツールは <code>semantic-agent-schema init</code> で登録する）。セッションは1つのフォルダに1つで、ふだんはいま動いているセッション1つで進める。セッションを立てる ・ 再開する ・ 様子を見るのは使う側。</p>'
  + cards([("いま動いているセッション", "ふだんはこれ1つ（いまはこのチャット）。SAS は環境変数の ID で記録するだけ"),
           ("ほかのフォルダのセッション", "目的を分けるとき、使う側がフォルダ（ワークツリーなど）ごとに1つ立てる。画面は無くプロセスで動き、そのフォルダで --resume すれば画面で開ける"),
           ("sessions の行", "1つのセッションに1行。親子 ・ 役 ・ 起動の設定は持たない"),
           ("semantic-agent-schema init", "いまのセッションを、このフォルダの記録の相手として登録する。環境変数から ID が取れないツール（Gemini ・ Cursor）では ID を決めてフォルダに残し、以後の semantic-agent-schema はそれを使う")])
  + '<h3>誰が何をするか</h3>' + SESS_CMDS
  + '<h3>sessions の行</h3>' + SESS_ROW
  + f'<p class="hint">動かして確かめる：<a href="{SIM_URL}">目的のセッションのシミュレーション</a>（立てたセッション2つ ・ semantic-agent-schema コマンドが増やしたセッション0）。</p>'))
T.append(('data', 'データ',
  '<p class="ans">手元の DB は5表とビュー。誰が何で動いたか（sessions）と、書いた変更（changes）・検証（validations）。中身はコミットせず、作り直せるデータとして扱う。</p>'
  + fig(F['er'], 'ER 図')
  + cards([("1 対 多", "sessions → changes・validations、changes → field_changes、validations → rule_results"), ("中身は Git", "パスとハッシュで指すだけ。Git は読むだけで、版は残さない"), ("空になる列が無い", "変更と検証を別の表にしたので、どの行もすべての欄が埋まる（作成の事前・削除の事後・原因を除く）。断った操作は持たない"), ("置き場所", "&lt;git-common-dir&gt;/semantic-agent-schema/records.sqlite。作業ツリーの外で、ワークツリーをいくつ作っても1つ。環境変数 SEMANTIC_AGENT_SCHEMA_DATABASE_FILE で指せる。Git のリポジトリの外では記録を作らずに断る"), ("worktree の欄", "changes に、書いたワークツリー（作業ツリーの根）。外の変更は、同じワークツリーで記録した最後の事後と比べる（そのワークツリーでまだ書いていなければ、記録のどれかの中身と比べる）"), ("schema の欄", "changes ・ validations に、書いた ・ 検証した文書が従う $id。semantic-agent-schema log --operation で書いたときと同じ形に戻すため"), ("記録を読む", "semantic-agent-schema log がビュー operations を新しい順に絞って読む（UC7）。読むだけで、何も足さない"), ("field_changes の役割", "欄ごとの前後を残し、次の判定のレビューのまとまり（変わった所 ・ 前の値 ・ 変更の操作）を作る"), ("理由は reason", "changes の reason（--reason）と rule_results の reason"), ("not-judged", "意味の rule に判定が渡されていない区分")])))
T.append(('flow', '意味の部分を含む操作の流れ',
  '<p class="ans">セッションが semantic-agent-schema に手順をもらい、自分で値を作って書かせる。断られたら理由を読んで直す。立てた側が受け取るのは最後の答えだけ。SAS はセッションを作らない。ふだんの1セッションでは、いま動いているセッションが同じことを自分で行う。</p>'
  + fig(F['flow'], '意味の部分を含む操作の流れ', 'ユースケースを意味で作る例。main はいま動いているセッション、別のフォルダは使う側が別のフォルダ（ワークツリーなど）に立てたセッション。ふだんの1セッションでは、main が自分で SAS と同じやりとりをする。検証も同じで、validate --semantic で手順をもらい、--answer で判定を渡す。', 1.8)
  + cards([("セッションは増えない", "どの semantic-agent-schema も、打ったセッションの操作になる"), ("判定はその時のもの", "結果は打ったセッションに返る。コミットしない"), ("CI", "機械の検証だけを毎回行う"),
           ("レビューのまとまり", "判定済みの対象では、前の判定のあとの変更の記録から変わった所を決め、所のまとまり（pointer ・ before ・ current ・ changes ・ rules）と文書全体のまとまりを渡す。入らなかった所は前の判定を引き継ぐ。すべては --all-rules と初めての対象"),
           ("変更の記録の役割", "書いたときの field_changes と --reason の理由が、次の判定のまとまりの before と changes になる。手順の根の review が since と scope を持つ"),
           ("判定する文脈", "手順の constraints に2文：判定は書いたときとは別の文脈（サブエージェントなど）で行う／判定用の会話はキャッシュの有効時間のうち続けて使い、変わった所の手順だけを渡す"),
           ("判定者を区別しない", "サブエージェントのセッション ID は親と同じなので、SAS は判定したのが誰かを区別しない")])))
SCN = {
 'a': ("場面 a：エージェントが U10 を作り、検証する", "semantic-agent-schema create requirements/usecases/U10.json --concrete urn:…:usecase:v1 --semantic --task \"…\"\nsemantic-agent-schema create requirements/usecases/U10.json --concrete urn:…:usecase:v1 --value '{…}'\nsemantic-agent-schema validate requirements/usecases/U10.json --semantic\nsemantic-agent-schema validate requirements/usecases/U10.json --answer judgments.json",
       '{"kind": "procedure", "operation_kind": "write", "command": "semantic-agent-schema create … --value <値の JSON>", …}\n{"kind": "change", "result": "created", "target": {"path": "…/U10.json", "schema": "urn:…:usecase:v1"}, "session": "4401cead…", "operation": "o-0031", …}\n{"kind": "procedure", "operation_kind": "validate", "command": "semantic-agent-schema validate … --answer <ファイル | ->", …}\n{"kind": "validation", "session": "4401cead…", "counts": {"verified": 2, "unverified": 1}, …}',
       [["sessions", "4401cead…（tool claude-code ・ via env）。最初に semantic-agent-schema を打ったときの1行"], ["changes", "o-0031 create U10 ・ session 4401cead…"], ["validations", "o-0032 U10 ・ hash sha256:3f9a… ・ session 4401cead…"], ["rule_results", "goal-phrase ai verified ／ no-design-words machine verified ／ stakeholder-interest ai unverified"]]),
 'b': ("場面 b：エージェントが意味で探す", "semantic-agent-schema search --concrete urn:…:usecase:v1 --semantic --question \"goal はなぜこの句か\"\nsemantic-agent-schema search --concrete urn:…:usecase:v1 --answer answer.json",
       '{"kind": "procedure", "operation_kind": "search", "question": "goal はなぜこの句か", "range": [{"path": "…/U10.json", "hash": "sha256:3f9a…"}, …], "steps": [{"no": 1, "action": "read", "path": "…/U10.json", "hash": "sha256:3f9a…", "current": {…}}, …], "command": "semantic-agent-schema search --concrete urn:…:usecase:v1 --answer <答えの JSON のファイル | ->", …}\n{"kind": "search", "range": [{"path": "…/U10.json", "hash": "sha256:3f9a…"}, …], "answer": "…", "sources": [{"path": "…/U10.json", "pointer": "/goal", "value": "…"}]}\n（出どころが範囲の外か所が無い → invalid_answer、ハッシュが違う → conflict）',
       [["sessions", "打ったセッションの行（初めてのときだけ1つ）"], ["changes・validations", "入らない（検索は記録しない）"]]),
 'c': ("場面 c：SAS を通らない変更があった", "（エディタで U10 の goal を直す）\nsemantic-agent-schema validate requirements/usecases/U10.json\nsemantic-agent-schema update requirements/usecases/U10.json --adopt --reason \"goal を短くした\"",
       '{"kind": "validation", "unrecorded": [{"path": "…/U10.json", "expected": "sha256:3f9a…", "actual": "sha256:91c2…"}], "counts": {"verified": 1, "not-judged": 2}, "hint": "理由を付けて取り込む：semantic-agent-schema update … --adopt --reason \\"…\\""}\n{"result": "adopted", "diff": {"source": "outside"}}',
       [["validations", "o-0042 U10 ・ hash sha256:91c2…"], ["changes", "o-0043 update ・ source outside ・ reason goal を短くした ・ before sha256:3f9a… ・ after sha256:91c2…"]]),
 'd': ("場面 d：U11 を消す（U0 が U11 を指していても U0 は変えない）", "semantic-agent-schema delete requirements/usecases/U11.json --reason \"U0 に統合した\"",
       '{"result": "deleted", "operation": "o-0040", "target": {"kind": "instance", "path": "…/U11.json", "schema": "urn:…:usecase:v1"}}',
       [["changes", "o-0040 delete U11 ・ reason U0 に統合した（事後なし・事前の欄を remove で残す）。U0 の記載は Git の差分とレビューで直す"]]),
 'e': ("場面 e：エージェントが U10 を意味で直し、断られて打ち直し、変わった所を判定させる", "semantic-agent-schema update requirements/usecases/U10.json --semantic --task \"goal を短くする\"\nsemantic-agent-schema update requirements/usecases/U10.json --patch '[…]' --if-match sha256:3f9a…\n（断った理由を読んで直す）\nsemantic-agent-schema update requirements/usecases/U10.json --patch '[…]' --if-match sha256:3f9a… --reason \"goal を短くする\"\nsemantic-agent-schema validate requirements/usecases/U10.json --semantic",
       '{"kind": "procedure", "operation_kind": "write", "command": "semantic-agent-schema update … --patch <JSON Patch> --if-match sha256:3f9a…", …}\n{"kind": "error", "code": "invalid_value", "message": "…", …}\n{"kind": "change", "result": "updated", "session": "4401cead…", "operation": "o-0050", …}\n{"kind": "procedure", "operation_kind": "validate", "target": {"path": "…/U10.json", "schema": "urn:…:usecase:v1", "hash": "sha256:7d1e…"}, "review": {"since": "o-0032", "scope": "changed"}, "rules": [{"id": "goal-phrase", …}], "steps": [{"no": 1, "action": "review-place", "pointer": "/goal", "before": "…", "current": "…", "changes": [{"operation": "o-0050", "reason": "goal を短くする"}], "rules": ["goal-phrase"]}, {"no": 2, "action": "review-document", "pointer": "", "changed": ["/goal"], …}], …}\n（/goal の所のまとまりと文書全体のまとまりだけ。ほかの所は前の判定を引き継ぐ。すべては --all-rules）',
       [["sessions", "増えない（同じセッション）"], ["changes", "o-0050 update U10 ・ reason goal を短くする ・ session 4401cead…（断った操作は記録しないので、記録は1件）"], ["validations", "入らない（手順を返すだけ。判定は --answer で渡したときに記録する）"]]),
}
use_html = ''
for k in 'abcde':
    t, cmd, out, rows = SCN[k]
    body = (f'<p class="lbl">実行するコマンド</p><pre>{html.escape(cmd)}</pre><p class="lbl">SAS が返すもの（抜粋）</p><pre>{html.escape(out)}</pre>'
            + '<p class="lbl">DB に入る行</p>' + tbl(['表', '行'], rows))
    use_html += f'<h3>{html.escape(t)}</h3>' + fig(F['sc' + k], t, '「エージェント」は semantic-agent-schema を打ったセッション（AI のエージェント）。SAS はエージェントを起動しない。', 1.8) + det('コマンド・返すもの・DB に入る行', f'<div class="scn">{body}</div>')
T.append(('use', '使われ方',
  '<p class="ans">5つの場面を、誰が何を依頼し、SAS が何を返し、DB に何が残るかで見る。</p>' + use_html))
LINK_CODE = '<pre>' + html.escape(json.dumps({"$schema": "urn:example:purpose-session:v1", "id": "ps-2026-11-01-01", "purpose": "ユースケースの頁を仕上げる", "parent": None, "status": "進行中",
  "sas": {"session": "4401cead-…", "operations": ["o-0031", "o-0032"]}, "commit": None}, ensure_ascii=False, indent=2)) + '</pre>'
T.append(('link', '使う側とのつなぎ方',
  '<p class="ans">SAS の DB は使う側に共通のモデルとして使わせない。約束はセッションの ID と操作の ID だけ。</p>'
  + fig(F['link'], '使う側とのつなぎ方')
  + cards([("SAS → 使う側", "CLI は打ったセッションの ID と操作の ID を返す"), ("使う側 → SAS", "--session でセッションの ID を渡せる（環境変数から取れるツールでは省ける。取れないツールは semantic-agent-schema init で登録する）"), ("残すべき結びつき", "目的とセッションの結びつきは、使う側が自分のインスタンスに書いて Git に残す。SAS の DB は作り直せる")])
  + det('使う側の Concrete の例', LINK_CODE)))
T.append(('uc', 'ユースケース',
  '<p class="ans">ユースケースは能力ごとの7件と要約1件。意味の部分は UC1・UC2・UC4・UC6 の中にあり、打ったセッションに手順を返す（判定は UC4 の変形「意味の判定」）。</p>'
  + fig(F['uc'], 'ユースケース')
  + det('8件の機械／意味と主アクター', wfix(tables(v5sec('uc'))[0] if tables(v5sec('uc')) else '').replace('UC7 セッションと操作を読む', 'UC7 操作の記録を読む').replace('<tr><td>SF1 SAS が立てたセッションでエージェントに手順で行わせる（サブ機能）</td><td>意味</td><td>呼び出したユースケースと同じ</td></tr>', '').replace('<td>UC4 検証する</td>', '<td>UC4 検証する（変形：意味の判定）</td>'))))

for k, v in T_NEW.items(): T.append(v)
T.append(T_NEED)
T.append(T_VERIFY)
ORDER = ['overview', 'types', 'uc', 'domain', 'model', 'verify', 'arch', 'data', 'session', 'flow', 'use', 'link', 'schema', 'need']
T = sorted(T, key=lambda t: ORDER.index(t[0]))

CSS = open(os.path.join(H, 'base.css')).read()
tabs = ''.join(f'<button type="button" role="tab" id="t-{k}" aria-controls="p-{k}" aria-selected="{"true" if i == 0 else "false"}">{n}</button>' for i, (k, n, _) in enumerate(T))
panels = ''.join(f'<section role="tabpanel" id="p-{k}" aria-labelledby="t-{k}"{"" if i == 0 else " hidden"}><h2>{n}</h2>{b}</section>' for i, (k, n, b) in enumerate(T))
JS = '''<script>
(function(){
  var bs=[].slice.call(document.querySelectorAll('.tabs button'));
  function show(k){bs.forEach(function(b){var on=b.id==='t-'+k;b.setAttribute('aria-selected',on?'true':'false');document.getElementById(b.getAttribute('aria-controls')).hidden=!on;});
    try{localStorage.setItem('sas-tab',k)}catch(e){}}
  bs.forEach(function(b){b.addEventListener('click',function(){show(b.id.slice(2));window.scrollTo({top:0})})});
  var h=(location.hash||'').slice(1); var s=null; try{s=localStorage.getItem('sas-tab')}catch(e){}
  var k=document.getElementById('t-'+h)?h:(s&&document.getElementById('t-'+s)?s:null); if(k) show(k);
})();
</script>'''
page = ('<title>Semantic Agent Schema の概念</title>\n<meta name="claude:viewport-fit-sides" content="cover">\n<style>\n' + CSS + '\n</style>\n<main>'
        '<h1>Semantic Agent Schema の概念</h1><p class="read">Semantic Agent Schema が何をし、何を記録し、どう使われるか。要求を書く（全体像〜ユースケース）→ 見つける（事業領域と文脈）→ 設計する（ドメインモデル〜スキーマと DB）の順に並べた。</p>'
        + f'<nav class="tabs" role="tablist" aria-label="内容">{tabs}</nav>' + panels + '</main>\n' + JS)
page = page.replace('指紋', 'ハッシュ')  # 以前の記録の語も、ふだん使う語で見せる
open(os.path.join(H, 'index.html'), 'w').write(page)
for n, r in REPORT: print(n, r)
print('ok', len(page))
