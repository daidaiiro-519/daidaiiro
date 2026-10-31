"""Semantic Agent Schema の概念（版6）。土台・設計のページと同じ作り（タブ・結論の1文・図・カード・details の表）で、版5 の決定を図を主役に組み直す。
図は design-svg の figure と chart exchange で作り、ER 図と3段の型は版5（../sas-role4/）の検査済みの図を使う。"""
import os, re, json, subprocess, html
H = os.path.dirname(os.path.abspath(__file__))
R4 = os.path.join(H, 'src')  # 版5 の検査済みの図と頁（3段の型・木・版5 の頁）
DS = os.path.normpath(os.path.join(H, '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg'))
os.makedirs(os.path.join(H, 'fig'), exist_ok=True)
REPORT = []
import figman

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
F['er'] = open(os.path.join(H, 'fig', 'er.svg')).read()
F['tree'] = open(os.path.join(R4, 'f5.svg')).read()
F['fail'] = run('figure', 'fail', {"direction": "TB", "nodes": [
    {"id": "st", "label": "セッションを立てる（request_id・attempt）", "role": "focus"},
    {"id": "ask", "label": "エージェントが答える"},
    {"id": "to", "label": "timeout で閉じる → 新しいセッションでやり直す（2回まで）", "role": "warn"},
    {"id": "shape", "label": "答えの形を確かめる"},
    {"id": "inv", "label": "誤り → 同じセッションで直させる（2回まで）。超えたら invalid_output で閉じる", "role": "warn"},
    {"id": "hash", "label": "対象の指紋を確かめる"},
    {"id": "cf", "label": "違う → conflict で閉じる（書かない）", "role": "warn"},
    {"id": "ok", "label": "SAS が書く・completed", "role": "focus"}],
    "edges": [{"from": "st", "to": "ask"}, {"from": "ask", "to": "to", "label": "答えない"}, {"from": "ask", "to": "shape", "label": "答えた"},
              {"from": "shape", "to": "inv", "label": "誤り"}, {"from": "shape", "to": "hash", "label": "合う"},
              {"from": "hash", "to": "cf", "label": "違う"}, {"from": "hash", "to": "ok", "label": "一致"}]})
F['flow'] = run('exchange', 'flow', {"label": "意味の操作の流れ", "participants": ["呼び出し元", "SAS", "SAS のセッション"], "steps": [
    {"from": "呼び出し元", "to": "SAS", "label": "検証を頼む"},
    {"from": "SAS", "to": "SAS", "label": "記録・機械の検証"},
    {"from": "SAS", "to": "SAS のセッション", "label": "単発で立て手順を渡す"},
    {"from": "SAS のセッション", "to": "SAS", "label": "rule ごとの判定"},
    {"from": "SAS", "to": "SAS", "label": "確かめて DB へ"},
    {"from": "SAS", "to": "呼び出し元", "label": "結果とセッション ID"}]})
SC = {
 'a': ("エージェント", [("エージェント", "SAS", "U10 を作る"), ("SAS", "SAS", "DB へ o-0031"), ("SAS", "エージェント", "s-0007"),
       ("エージェント", "SAS", "U10 を検証"), ("SAS", "SAS のセッション", "s-0008 で手順"), ("SAS のセッション", "SAS", "rule ごとの判定"),
       ("SAS", "SAS", "DB へ o-0032"), ("SAS", "エージェント", "結果と s-0008")]),
 'b': ("人", [("人", "SAS", "goal はなぜ？"), ("SAS", "SAS のセッション", "s-0011 で探す"),
       ("SAS のセッション", "SAS", "答えと出どころ"), ("SAS", "人", "答え")]),
 'c': ("人", [("人", "人", "直接書き換え"), ("人", "SAS", "U10 を検証"), ("SAS", "SAS", "指紋が違う"),
       ("SAS", "人", "通らない変更"), ("人", "SAS", "--adopt で取り込む"), ("SAS", "SAS", "DB へ o-0043")]),
 'd': ("人", [("人", "SAS", "U11 を削除"), ("SAS", "SAS", "参照元 U0 を探す"), ("SAS", "SAS", "DB へ 2件"),
       ("SAS", "人", "o-0040・o-0041")]),
 'e': ("エージェント", [("エージェント", "SAS", "U10 を更新"), ("SAS", "SAS のセッション", "s-0012（1回目）"),
       ("SAS のセッション", "SAS", "形が誤り"), ("SAS", "SAS", "invalid_output"), ("SAS", "SAS のセッション", "s-0013（2回目）"),
       ("SAS", "SAS", "timeout"), ("SAS", "SAS のセッション", "s-0014（3回目）"), ("SAS のセッション", "SAS", "決めた形の JSON"),
       ("SAS", "SAS", "書く・o-0050"), ("SAS", "エージェント", "updated")]),
}
for k, (who, steps) in SC.items():
    parts = [who, "SAS"] + (["SAS のセッション"] if any("SAS のセッション" in (a, b) for a, b, _ in steps) else [])
    F['sc' + k] = run('exchange', 'sc' + k, {"label": "場面 " + k, "participants": parts, "steps": [{"from": a, "to": b, "label": l} for a, b, l in steps]})
F['link'] = run('figure', 'link', {"direction": "LR", "nodes": [
    {"id": "u", "label": "使う側の Concrete（例：目的のセッション）：目的・親子・状態・コミット", "role": "kind-2"},
    {"id": "cli", "label": "SAS の CLI"},
    {"id": "s", "label": "SAS のセッション（手元の DB）", "role": "focus"}],
    "edges": [{"from": "u", "to": "cli", "label": "--caller で呼び出し元の参照を渡す"}, {"from": "cli", "to": "s", "label": "記録する"},
              {"from": "s", "to": "u", "label": "セッション ID を sas_sessions に記録する", "dashed": True}]})
F['uc'] = run('figure', 'uc', {"direction": "TB", "nodes": [
    {"id": "u0", "label": "UC0 その型で成果物を作り、検証された状態で使い続ける（要約・使う側）", "role": "kind-2"},
    {"id": "u1", "label": "UC1 作成する"}, {"id": "u2", "label": "UC2 更新する"}, {"id": "u3", "label": "UC3 削除する"},
    {"id": "u4", "label": "UC4 検証する"}, {"id": "u5", "label": "UC5 描画する"}, {"id": "u6", "label": "UC6 検索する"},
    {"id": "u7", "label": "UC7 セッションと操作を読む"},
    {"id": "sf", "label": "SF1 SAS が立てたセッションで手順を行わせる（意味）", "role": "focus"}],
    "edges": [{"from": "u0", "to": x} for x in ["u1", "u2", "u3", "u4", "u5", "u6", "u7"]] +
             [{"from": x, "to": "sf", "dashed": True} for x in ["u1", "u2", "u4", "u6"]]})


F['resume'] = run('figure', 'resume', {"direction": "TB", "nodes": [
    {"id": "q", "label": "続けて頼む（継続のセッション）", "role": "focus"},
    {"id": "r1", "label": "① 単発か、閉じている → 新しく立てる", "role": "warn"},
    {"id": "r2", "label": "② アダプターが再開を持たないか、ツールの側の期限切れ・消えた → 新しく立て continued_from で指す", "role": "warn"},
    {"id": "r3", "label": "③ 文脈の量が上限の8割を超えた → 新しく立て、意図と決めたことの文から brief を組む", "role": "warn"},
    {"id": "r4", "label": "④ 前の回のあとに対象の指紋が変わった → 再開し、差分を手順のプロンプトの先頭へ（差分が取れなければ新規）"},
    {"id": "r5", "label": "⑤ ほか → 再開する", "role": "focus"}],
    "edges": [{"from": "q", "to": "r1", "label": "上から当てる"}, {"from": "r1", "to": "r2", "label": "当たらない"}, {"from": "r2", "to": "r3", "label": "当たらない"}, {"from": "r3", "to": "r4", "label": "当たらない"}, {"from": "r4", "to": "r5", "label": "当たらない"}]})
F['unsup'] = run('figure', 'unsup', {"direction": "LR", "nodes": [
    {"id": "req", "label": "型を指定して頼む（例：単発・引き継ぐ）", "role": "focus"},
    {"id": "cap", "label": "接続の定義の能力の宣言を読む"},
    {"id": "ok", "label": "能力がある → 立てる", "role": "focus"},
    {"id": "ng", "label": "能力が無い → 立てずに unsupported_capability（足りない能力と代わりの指定：引き継がない＋--brief）", "role": "warn"}],
    "edges": [{"from": "req", "to": "cap"}, {"from": "cap", "to": "ok", "label": "ある"}, {"from": "cap", "to": "ng", "label": "無い"}]})

# ---------- 版5 から details に入れる表 ----------
v5 = open(os.path.join(R4, 'index.html')).read()
def v5sec(i):
    a = v5.find(f'id="{i}"'); b = v5.find('<section', a + 5); return v5[a:b if b > 0 else len(v5)]
def tables(frag):
    return re.findall(r'<div class="tw">.*?</table></div>', frag, re.S)
sess = v5sec('session'); sess_tables = tables(sess)
rules_ol = re.search(r'<ol.*?</ol>', sess, re.S); rules_ol = rules_ol.group(0) if rules_ol else ''
use = v5sec('use'); scns = re.findall(r'<div class="scn"><h3>(.*?)</h3>(.*?)</div>(?=<div class="scn">|</section>|\s*$)', use, re.S)
scn_html = {}
for m in re.finditer(r'<div class="scn"><h3>(.*?)</h3>', use):
    start = m.end(); nxt = use.find('<div class="scn">', start); body = use[start:nxt if nxt > 0 else use.rfind('</section>')]
    scn_html[re.search(r'場面 ([a-e])', m.group(1)).group(1)] = (re.sub('<[^>]+>', '', m.group(1)), body.rstrip()[:-6] if body.rstrip().endswith('</div>') else body)
link_code = re.search(r'<pre>.*?</pre>', v5sec('link'), re.S).group(0)

def tbl(head, rows):
    return '<div class="tw"><table><thead><tr>' + ''.join(f'<th>{h}</th>' for h in head) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</tbody></table></div>'
def cards(items):
    return '<div class="cards">' + ''.join(f'<div class="card"><b>{t}</b><span>{d}</span></div>' for t, d in items) + '</div>'
def det(summary, body):
    return f'<details><summary>{summary}</summary>{body}</details>'


# ---------- 版7：見つける・設計する（事業領域・文脈・ドメインモデル・アーキテクチャ・スキーマ） ----------
def rd(n): return open(os.path.join(H, 'fig', n + '.svg')).read()
for n in ['ctxmap']: F[n] = rd(n)
def jd(n): return json.load(open(os.path.join(H, 'fig', 'base', n + '.json')))
def nl(d, i, label=None, role=None):
    for x in d['nodes']:
        if x['id'] == i:
            if label: x['label'] = label
            if role is not None: x['role'] = role
SUB = [("意味の操作", "中核", "ドメインモデル", "UC1・UC2・UC4・UC6 の意味の側、SF1", "SAS が立てたエージェント／コーディングエージェント／SAS が立てたセッション・手順のプロンプト・答え"),
       ("契約への適合", "中核", "ドメインモデル", "UC1〜UC4 の機械の側、取り込み、UC3 の参照元、UC6 の構造で絞る", "人・AI のエージェント／JSON Schema の検査・Git（読むだけ）／Abstract・Concrete・インスタンスのファイル"),
       ("操作の記録", "補完", "トランザクションスクリプト", "UC1〜UC4 の記録、UC7", "人・AI のエージェント・レビューする人／ツールのセッション ID／呼び出し元のセッション・操作・変わった欄・rule の結果"),
       ("描画", "補完", "トランザクションスクリプト", "UC5", "読み手／デザインシステム／書かない"),
       ("版の管理・JSON Schema の検査・エージェントの実行・描画の部品", "一般", "製品は作らずに使い、アダプターで差し込む。能力の契約（ポートと SAS の型）は持ち主の業務領域が SAS の中に持つ", "―", "Git・検査器・コーディングエージェント・テンプレートの処理系")]
WORDS = [("同じ意味（文脈「契約とインスタンス」）", "セッション（呼び出し元・SAS が立てた）・操作・変わった欄・rule の結果・指紋・意図・決めたことの文・依頼と回数・手順のプロンプト・エージェントの答え・終わり方"),
         ("意味が違う（境界の外）", "ツールの「セッション」（会話の続き。SAS では tool_ref として持つだけ）・Git の「コミット」・使う側の「目的のセッション」（目的・親子・承認・コミットを持つ）")]
CTXNO = [("描画を別の文脈にする", "1つの変更要求で2つの文脈を直すことになる。いまは描画が SAS の語をそのまま使う"),
         ("業務領域ごとに1つの文脈にする", "意味の操作と操作の記録が同じデータ（sessions・operations）を扱い、閉じたセッションに足さない規則を2か所で守ることになる"),
         ("事業領域全体を1つの文脈とみなす", "土台の決まりに反する。今回は語で境界を引き、一般の製品と使う側を外の文脈にした"),
         ("前の2つの文脈（検証の文脈・変更の確かめの文脈）", "前は「判定」の2つの意味で2つの文脈に分けていた。意図の判定を使う側へ出したので意味が1つに戻り、1つになった")]
INV = [("集約：SAS が立てたセッション（意味の操作）", "① 親は必ず呼び出し元のセッション（下に子は立たない）② 単発は1件の依頼で閉じ、再開しない ③ 閉じたら回を足さない ④ 能力の写しに無い操作（再開・fork）をしない ⑤ inherit は引き継いだ元を必ず持つ ⑥ validate の依頼を持つセッションは単発・引き継がない ⑦ 形を直させるのは2回まで、超えたら invalid_output"),
       ("集約：Concrete（契約への適合）", "① 申告する Abstract の版は閉じた一覧にある ② 本体の x- と同じ名前を足さない ③ 公開した版の中身を変えない"),
       ("集約：インスタンス（契約への適合）", "① id を変えない ② x-derived-from の宣言値と導出値が一致する ③ x-decided-by が human の欄は人の入口からだけ変える ④ 読んだ時の指紋といまの指紋が一致したときだけ書く ⑤ 形の確認・機械の検証・指紋の一致がそろったときだけ書く ⑥ 最後の操作の事後の指紋（値で受け取る）と違えば SAS を通らない変更として知らせる"),
       ("操作の記録の行（集約にしない）", "追記だけ・request_id は1回・続き番号はセッションごとに1つずつ（DDL・trigger・一意の索引が守る）")]
CMD = [("SAS が立てたセッション", "立てる（型を指定）／続ける（再開の決まり）／fork／状態を読む／答えを受け取る（形を直させる）／閉じる（完了・時間切れ・失敗・取り消し）"),
       ("Concrete", "作る／版を変える（公開した版は変えない）"),
       ("インスタンス", "作る／直す（指紋を確かめる）／消す（参照元は use_cases が同じセッションで1件ずつ直す）／取り込む（--adopt）")]
VAL = "契約への適合：指紋・差分（欄ごとの前後）・URN と版・インスタンスの id・判定の結果（5つの値と、何を verified に数えるか）。意味の操作：セッションの型（寿命・文脈）・能力の宣言。業務イベントは作らない（購読する手続きが無い）"
SVC = [("x-relationship（契約への適合）", "指す先が在るかを確かめる・参照元を探す。ほかのインスタンスにまたがる"),
       ("再開の決まり（意味の操作）", "上から5段：単発か閉じた→新規／再開できない（能力・期限・消えた）→新規／文脈の量が8割超→新規／対象が変わった→再開して差分を先頭へ／ほか→再開。当たった段を記録に残す"),
       ("やり直しの決まり（意味の操作）", "時間切れは同じ依頼・次の回数で新しいセッションを2回まで、間隔を広げる")]
MNO = [("上からだけ（版13：events/・操作の記録の集約2つ・契約への適合をアクティブレコード）", "購読者の無い業務イベントと、守る条件の無い集約が残る。x- の規則が作成・更新・削除・取り込みの4本のスクリプトに重複する"),
       ("下からだけ（すべてスクリプトで書いてから引き上げる）", "守る条件がすでに書けている契約への適合と SAS が立てたセッションまで、いったんスクリプトに重複して書く"),
       ("業務領域ごとに技術方式を分ける（スクリプトはレイヤード）", "1つの文脈に依存の向きが2つでき、スクリプトから引き上げるたびに向きを付け替える"),
       ("domain を1つの crate にする", "1つの crate の中では集約の間の向きを止められない（Book ch07-03:296）"),
       ("イベント履歴式にする", "中身の情報源が Git と履歴の2つになり、SAS を通らない変更で食い違う")]
PORTS = [["agent_runtime", "semantic-operations", "outbound/agents", "dyn。設定でツールを選ぶので実行時に型が決まる"], ["keep_prompt", "semantic-operations", "outbound/files", "dyn。組み立てと試験の差し替えを1つの書き方にそろえる"], ["agent_session_store", "semantic-operations", "outbound/sqlite", "dyn。同上"], ["document_store", "conformance", "outbound/files", "dyn。同上。書くのは SAS の use_cases だけ"], ["version_reader", "conformance", "outbound/git", "dyn。同上。コミット済みの版を読むだけ"], ["schema_checker", "conformance", "outbound/jsonschema", "dyn。同上。2020-12 の形の検査。x- の判定は domain"], ["identify_caller", "operation-recording", "outbound/agents", "dyn。同上。いちばん下に置き、どのモジュールからも使える"], ["operation_log", "operation-recording", "outbound/sqlite", "dyn。同上。呼び出し元のセッションと操作の行を足す・読む"], ["template_renderer", "rendering", "outbound/templates", "dyn。同上"]]
LIBS = [["rusqlite", "outbound/sqlite の crate だけが依存", "行の型で変換する（orphan rule）"], ["2020-12 の検査器のライブラリ", "outbound/jsonschema の crate だけが依存", "schema_checker を実装。$ref はネットワークから取らない。x- の判定は domain"], ["テンプレートの処理系のライブラリ", "outbound/templates の crate だけが依存", "template_renderer を実装。種類がそろうかは application が確かめる"]]
RULES_ANY = ["リポジトリの根は事業領域 Semantic Agent Schema の置き場で、文脈の根にしない。文脈の根は、文脈の名前のフォルダ（リポジトリの根の直下。ここでは contracts-and-instances/）。文脈の名前はその文脈の言葉（契約・インスタンス）から付け、事業領域や製品の名前を使わない。文脈全体を1つのヘキサゴナルにし、外とのやりとりはすべて application の ports を adapters/outbound が実装する", "業務ロジックの置き方は、上から決めて下から確かめる。守る条件がすでに決定・Abstract の rule・試験の場面に書けている所は初めから domain に置き、それ以外は use_cases にトランザクションスクリプトで書く。引き上げの表で見分け、突き合わせで確かめたものだけを domain に置く", "domain の中は部品の種類でフォルダを分ける：aggregates/（集約ごと）・values/・services/。購読する手続きが無いうちは events/ を作らない", "application の中は use_cases/ と ports/。use_cases/ のファイルは集約か手順のまとまりごとに分け、ユースケースごとのファイルやフォルダは作らない（土台の境界6）", "domain を持つのは、突き合わせで集約・値・サービスが確かめられた業務領域だけ（意味の操作・契約への適合）。操作の記録と描画は application だけ", "ポートは、それを使う手順を持つモジュールの application/ports に置く。共有の置き場を作らない。呼び出し元を見分けるポートは、いちばん下の操作の記録が持つ", "モジュールの間の向きは 意味の操作 → 契約への適合 → 操作の記録、描画 → 契約への適合 の一方向。同じ文脈の別のモジュールはポートを挟まず application を直接呼ぶ", "外のライブラリ（rusqlite・2020-12 の検査器・テンプレートの処理系）は、それを使う outbound の crate だけが依存する", "組み立て（アダプターをポートへ渡す）は入口の1か所だけ", "公開された言葉（schemas/&lt;版&gt;/）は文脈の根に置き、出した版は変えない"]
RULES_RS = ["依存の向きを強制したい境界と、外のライブラリを閉じ込めたい所だけを crate にし、それ以外はモジュールで分ける。crate の数の基準は原典に無い<span class=\"src\">（Book ch07-01:186・ch14-03:187（基準の記述が無い）・ch07-03:296・Reference visibility:476）</span>", "domain は業務領域ごとに crate を分ける（conformance-domain と semantic-operations-domain）。1つの crate の中では子のモジュールが祖先の項目を使えるので、集約の間の向きはモジュールでは止められない<span class=\"src\">（Book ch07-03:296・ch14-03:299）</span>", "根の Cargo.toml は virtual manifest。[workspace] に members と resolver = \"3\" を書く<span class=\"src\">（Cargo workspaces:241・258、Book ch14-03:207〜211）</span>", "外のライブラリの版は [workspace.dependencies] に置き、各 crate は workspace = true で継承する（optional にはできない）。SAS の中の crate どうしは path dependencies<span class=\"src\">（Cargo workspaces:359・363、specifying-dependencies:425）</span>", "crate が使えるのは自分の Cargo.toml に書いた crate だけ。これで domain の無依存と、ライブラリの閉じ込めを強制する<span class=\"src\">（Book ch14-03:299・390）</span>", "lib.rs は pub mod aggregates; pub mod values; pub mod services;（events は作らない）。crate の中だけで使う部品は pub(crate)<span class=\"src\">（Book ch07-02:273、Reference visibility:390・476）</span>", "mod.rs を使わない。src/aggregates.rs と src/aggregates/concrete.rs・instance.rs・agent_session.rs、src/aggregates/agent_session/turn.rs の形にする<span class=\"src\">（Book ch07-05:251・265・268）</span>", "application の ports.rs が、ポートで使う domain の型を pub use で再公開する。adapters は application だけに依存する<span class=\"src\">（Book ch07-04:392、Reference visibility:564）</span>", "組み立ては cli の main.rs。outbound の crate に依存してよいのは cli だけ<span class=\"src\">（Book ch14-03:299（推論：実装の型の名前を書くので依存が要る））</span>", "ポートは trait で、9つとも dyn で使う。dyn compatible に保つ：型引数のあるメソッド・async fn・関連定数を持たず、Self を receiver 以外で使わない。sealed にしない<span class=\"src\">（Reference traits:412・429・445・459、API Guidelines C-OBJECT:322・324・321、C-SEALED:169）</span>", "集約の欄は private、メソッドだけ pub。ID・指紋は newtype で区別する<span class=\"src\">（Book ch07-03:477、API Guidelines C-STRUCT-PRIVATE:218・C-NEWTYPE:169）</span>", "adapters で外の trait（rusqlite・serde など）を domain の型に実装しない（orphan rule）。sqlite の crate は自分の行の型を持って変換する<span class=\"src\">（Book ch10-02:302・309・313、Reference implementations:658・659・669）</span>", "依存のグラフは循環させない<span class=\"src\">（Cargo resolver:568・585）</span>"]
TREE = [["semantic-agent-schema/", "リポジトリ ＝ 事業領域 Semantic Agent Schema（文脈ではない）。直下に文脈の名前のフォルダと、事業領域全体のものだけを置く", ""], ["├── Cargo.toml", "［事業領域全体］virtual manifest：[workspace] members に下の crate のパス（札の付いたフォルダ）が13並ぶ・resolver = \"3\"・[workspace.dependencies]（rusqlite・検査器・テンプレートの処理系の版）", ""], ["├── architecture.json", "［事業領域全体］crate の間で許す依存の向き（検査に使う）", ""], ["├── docs/", "［事業領域全体］設計の文書（土台・設計のページ）", ""], ["└── contracts-and-instances/", "文脈「契約とインスタンス」（区切られた文脈）＝ 1つのヘキサゴナル。まとめるだけのフォルダ", ""], ["    ├── schemas/v1/", "公開された言葉（14のファイル）", ""], ["    ├── conformance/", "業務領域「契約への適合」（中核・ドメインモデル）。まとめるだけのフォルダ", ""], ["    │   ├── domain/", "依存：無し", "contracts-and-instances-conformance-domain"], ["    │   │   ├── Cargo.toml", "", ""], ["    │   │   └── src/lib.rs", "pub mod aggregates; pub mod values; pub mod services;", ""], ["    │   │       ├── aggregates.rs", "pub mod concrete; pub mod instance;", ""], ["    │   │       ├── aggregates/concrete.rs", "集約「Concrete」：版は閉じた一覧・本体の x- の名前を足さない・公開した版を変えない", ""], ["    │   │       ├── aggregates/instance.rs", "集約「インスタンス」：id を変えない・導出値が一致・人の欄は人の入口から・指紋が一致したときだけ書く・SAS を通らない変更を見分ける", ""], ["    │   │       ├── values.rs ＋ values/", "値：fingerprint・diff・urn（URN と版）・instance_id・verdict（5つの値と数え方）", ""], ["    │   │       └── services.rs ＋ services/", "サービス：relationship（指す先の実在・参照元を探す）", ""], ["    │   └── application/", "依存：conformance-domain・operation-recording-application", "contracts-and-instances-conformance-application"], ["    │       ├── Cargo.toml", "", ""], ["    │       └── src/", "use_cases/documents.rs（作る・直す・消す・取り込む・機械で検証する）・use_cases/search.rs・ports.rs（document_store・version_reader・schema_checker。domain の型を pub use）", ""], ["    ├── semantic-operations/", "業務領域「意味の操作」（中核・ドメインモデル）。まとめるだけのフォルダ", ""], ["    │   ├── domain/", "依存：conformance-domain", "contracts-and-instances-semantic-operations-domain"], ["    │   │   ├── Cargo.toml", "", ""], ["    │   │   └── src/", "aggregates.rs・aggregates/agent_session.rs（集約「SAS が立てたセッション」：型ごとの不変条件）・aggregates/agent_session/turn.rs（回）・values/（session_type・capabilities）・services/（resume_rule 5段・retry_rule）", ""], ["    │   └── application/", "依存：semantic-operations-domain・conformance-domain・conformance-application・operation-recording-application", "contracts-and-instances-semantic-operations-application"], ["    │       ├── Cargo.toml", "", ""], ["    │       └── src/", "use_cases/agent_sessions.rs（SF1：手順のプロンプトを組む・呼ぶ・答えを確かめて契約への適合へ渡す）・ports.rs（agent_runtime・keep_prompt・agent_session_store）", ""], ["    ├── operation-recording/", "業務領域「操作の記録」（補完・トランザクションスクリプト。domain は無い）。まとめるだけのフォルダ", ""], ["    │   └── application/", "依存：conformance-domain（値だけ）", "contracts-and-instances-operation-recording-application"], ["    │       ├── Cargo.toml", "", ""], ["    │       └── src/", "use_cases/history.rs（追記する・UC7 を読む）・ports.rs（identify_caller・operation_log）", ""], ["    ├── rendering/", "業務領域「描画」（補完・トランザクションスクリプト。domain は無い）。まとめるだけのフォルダ", ""], ["    │   └── application/", "依存：conformance-domain・conformance-application", "contracts-and-instances-rendering-application"], ["    │       ├── Cargo.toml", "", ""], ["    │       └── src/", "use_cases/render.rs・ports.rs（template_renderer）", ""], ["    └── adapters/", "ヘキサゴナルの外側。まとめるだけのフォルダ", ""], ["        ├── inbound/cli/", "依存：4つの application と outbound の6つ。[[bin]] の名前は sas（製品の名前はここにだけ残す）。src/main.rs で組み立てる。outbound に依存してよいのはここだけ", "contracts-and-instances-adapters-inbound-cli"], ["        │   └── Cargo.toml", "", ""], ["        └── outbound/", "まとめるだけのフォルダ", ""], ["            ├── agents/", "agent_runtime・identify_caller を実装", "contracts-and-instances-adapters-outbound-agents"], ["            │   └── Cargo.toml", "", ""], ["            ├── files/", "keep_prompt・document_store を実装", "contracts-and-instances-adapters-outbound-files"], ["            │   └── Cargo.toml", "", ""], ["            ├── git/", "version_reader を実装（読むだけ）", "contracts-and-instances-adapters-outbound-git"], ["            │   └── Cargo.toml", "", ""], ["            ├── jsonschema/", "schema_checker を実装（2020-12 の検査器を閉じ込める）", "contracts-and-instances-adapters-outbound-jsonschema"], ["            │   └── Cargo.toml", "", ""], ["            ├── sqlite/", "agent_session_store・operation_log を実装。migrations/。行の型で変換（orphan rule）", "contracts-and-instances-adapters-outbound-sqlite"], ["            │   └── Cargo.toml", "", ""], ["            └── templates/", "template_renderer を実装（テンプレートの処理系を閉じ込める）", "contracts-and-instances-adapters-outbound-templates"], ["                └── Cargo.toml", "", ""]]
CRATES = [["conformance-domain", "domain", "無し"], ["semantic-operations-domain", "domain", "conformance-domain"], ["operation-recording-application", "application", "conformance-domain（値だけ）"], ["conformance-application", "application", "conformance-domain・operation-recording-application"], ["semantic-operations-application", "application", "semantic-operations-domain・conformance-domain・conformance-application・operation-recording-application"], ["rendering-application", "application", "conformance-domain・conformance-application"], ["adapters-outbound-agents", "adapters", "semantic-operations-application・operation-recording-application（agent_runtime・identify_caller を実装）"], ["adapters-outbound-files", "adapters", "semantic-operations-application・conformance-application（keep_prompt・document_store）"], ["adapters-outbound-git", "adapters", "conformance-application（version_reader。読むだけ）"], ["adapters-outbound-jsonschema", "adapters", "conformance-application（schema_checker。2020-12 の検査器のライブラリはここだけ）"], ["adapters-outbound-sqlite", "adapters", "semantic-operations-application・operation-recording-application（agent_session_store・operation_log。rusqlite はここだけ）"], ["adapters-outbound-templates", "adapters", "rendering-application（template_renderer。テンプレートの処理系はここだけ）"], ["adapters-inbound-cli", "adapters（bin・組み立て）", "4つの application と outbound の6つ（main.rs で組み立てる。outbound に依存してよいのはここだけ）"]]
tree_html = ('<div class="legend"><span class="crate">crate</span> Cargo.toml を持つフォルダ。依存の単位で、名前は右の札のとおり（13個）。札の無いフォルダは、まとめるだけのフォルダか、crate の中のモジュール</div>' + '<div class="fig"><div class="tree">' + ''.join(f'<div class="tl"><code>{html.escape(p)}</code><span>' + (f'<span class="crate">crate</span><code class="cn">{html.escape(c)}</code> ' if c else '') + f'{html.escape(n)}</span></div>' for p, n, c in TREE) + '</div></div>')
SCHG = [("Concrete を書くための型", "SAS → Concrete を書く人と AI・検査器", [("semantic-abstract-schema.json", "Abstract の本体。x- の10の値の形、欄・rule・読み方の形"), ("vocabulary.schema.json", "語彙。入れ子に掛かる x-（x-view を足す。x-prompt は在る）"), ("dialect.schema.json", "方言。2020-12 に SAS の語彙を足す（$dynamicAnchor: meta で入れ子に掛ける）"), ("glossary.schema.json", "用語集の型。中身は使う側")]),
 ("CLI の入出力", "呼び出し元 ⇄ CLI", [("operation-request.schema.json", "操作の依頼。create・update・delete・validate・render・search・read・session-close・session-status と共通の session-spec"), ("operation-result.schema.json", "操作の結果。procedure・change・validation・rule-result・render・search・session・operation・history・error")]),
 ("意味の操作", "SAS ⇄ アダプター ⇄ SAS が立てたセッション", [("procedure-prompt.schema.json", "手順のプロンプトの構造化された部分（ステップ・読む欄・答えの形）"), ("session-request.schema.json", "セッションの依頼（start・continue・fork・close・status）とアダプターの返り（reply）"), ("agent-answer.schema.json", "エージェントの答え（field-values・judgments・found・grounds）")]),
 ("共通の値", "2つ以上の境界から参照", [("session.schema.json", "セッションの姿（4つの型・回・終わり方・トークン）"), ("diff.schema.json", "差分（欄ごとの前後・指紋）")]),
 ("差し込む物の契約", "使う側 → SAS", [("agent-connector.schema.json", "エージェントの接続の定義（起動の仕方・能力の宣言・呼び出し元の見分け方・同時に動かせる数）"), ("design-system.schema.json", "デザインシステムの受け取りの型（x-view の種類ごとのテンプレート・トークン）"), ("settings.schema.json", "設定（アダプター・上限・Concrete と手順のプロンプトの置き場所）")])]
XTERMS = [("x-instance-path・x-instance-label", "根", "インスタンスの置き場所と、指されたときの名前", ""), ("x-delegates", "根", "rule を任せる先の立場と種類（judging・working）。ツールの名前は書かない", ""),
 ("x-extensions", "根", "Concrete が足す x- の定義（schema・title・description・prompt・published）", ""), ("x-prompt", "根と欄", "読み方（shown・interpret・when）と書き方の rule。いまの vocab-meta に在り、方言を通して入れ子でも検査される", ""),
 ("x-relationship・x-derived-from・x-decided-by・x-published", "欄", "参照先・導く決まり・人が決める欄・公開するか", ""), ("x-view", "根と欄", "描画の種類の名前。デザインシステムのテンプレートの種類と合わせる", "足した")]
GONE2 = [("judgment.schema.json", "rule_results（DB）と operation-result の rule-result"), ("change.schema.json", "operations・field_changes（DB）と diff"), ("change-summary.schema.json", "使う側（まとめ・承認・コミット）。SAS は読む形で材料を渡す"),
 ("call.schema.json", "役目は残す。rules・inputs・task → procedure-prompt、delegate・handoff → session-request"), ("Abstract の output-judging・output-working", "agent-answer"), ("vocab-meta.schema.json", "vocabulary（改名。x-view を足す）")]
MAPW = [("operation-request（intent・request_id・caller・session-spec）", "operations・sessions・session_turns"), ("procedure-prompt（ファイル）", "session_turns.prompt_path・prompt_hash"), ("agent-connector の capabilities・caller", "sessions.capabilities・tool_ref"),
 ("session-request の reply", "session_turns.outcome・tokens・answer_hash"), ("agent-answer の field-values", "ファイルへ書き、差分を求めて field_changes"), ("agent-answer の judgments", "rule_results（by = ai）"), ("diff", "operations の前後の指紋・field_changes")]
MAPR = [("sessions＋session_turns", "session（parent_id → parent、prompt_path・prompt_hash → prompt）"), ("operations", "operation（session_id → session、turn_seq → turn、cause_id → cause）"), ("operations＋field_changes", "diff（change・operation の中）"),
 ("rule_results", "rule-result（reads_hash → reads_fingerprint）"), ("operations の最後の after_hash", "validation の untracked・error"), ("operations（パスごと）", "history")]
FACT = [
 ("JSON Schema の出力の形", "json-schema.org Core 2020-12 の12章", "欄名はすべて在った。一覧に absoluteKeywordLocation と error（単数）を足した。出力の形は実装に SHOULD"),
 ("$vocabulary・$dynamicAnchor・$dynamicRef", "Core 2020-12・メタスキーマ", "方言に $dynamicAnchor: meta を置く。true の語彙を知らない検査器は方言を断る（MUST）"),
 ("JSON Patch", "RFC 6902", "op・path・value・from と op の6つが在った"),
 ("Design Tokens", "Format Module 2025.10", "版を固定。「群」を原典の group に直した。別名は {group.token}"),
 ("MCP の inputSchema", "MCP 2026-07-28", "根は type: \"object\"、外の $ref は解かれない。道具ごとに1つのスキーマへ束ねる"),
 ("コーディングエージェント5つ", "Claude Code・Codex CLI・Gemini CLI・Copilot CLI・Kiro CLI の公式文書", "環境変数で ID を取れるのは Claude Code だけ。caller.env を任意に、tool_ref.from に assigned を足し、close・status を省略できる形に"),
 ("SQLite", "sqlite.org", "CHECK は NULL を違反にしない。NULL になりうる欄の = ・ IN を IS ・ IS NOT NULL で閉じた（実測でも確かめた）。最低の版 3.8.0・3.38.0、RAISE は文字列のリテラル"),
 ("Git の掃除", "git-scm.com（2.52.0 で最後に更新された頁）", "指されないオブジェクトは gc.pruneExpire（既定2週間）で消える。SAS は版を残さない（Git は読むだけ）ので、この条件は設計に関わらない")]
UNREAD = "取得していない頁：Claude Code の sessions.md、Gemini の automation のチュートリアル、Codex の app-server、Copilot の SDK。ここに環境変数や欄が在る可能性は残る"
def flow(name, steps):
    ns = [{"id": f"s{i}", "label": f"{t}：{sc}", "role": ("focus" if tb else None)} for i, (t, sc, tb) in enumerate(steps)]
    for n in ns:
        if n["role"] is None: del n["role"]
    return run('figure', name, {"direction": "TB", "nodes": ns, "edges": [{"from": f"s{i}", "to": f"s{i+1}"} for i in range(len(steps) - 1)]})
F['fv'] = flow('fv', [("CLI の入力", "operation-request の validate", False), ("呼び出し元を見分ける", "agent-connector の caller → sessions", True), ("SAS を通らない変更", "operations の最後の after_hash と比べる", True),
  ("機械と人の rule", "検査器 → rule-result（by: human は unverified）", False), ("手順のプロンプト", "procedure-prompt（judge）→ sessions・session_turns", True), ("セッションの依頼", "session-request の start", False),
  ("答え", "agent-answer の judgments → session_turns", True), ("指紋を確かめる", "違えば書かずに conflict", False), ("記録", "operations・rule_results（1つのトランザクション）", True), ("閉じる", "sessions.ended_as（別のトランザクション）", True), ("CLI の最後の行", "operation-result の validation", False)])
F['fu'] = flow('fu', [("CLI の入力", "operation-request の update（intent・brief・session-spec）", False), ("呼び出し元を見分ける", "sessions（caller）", True), ("SAS を通らない変更", "違えば直さずに断る", True),
  ("セッションを決める", "再開の決まり → sessions か session_turns", True), ("手順のプロンプト", "procedure-prompt（write。再開なら diff を先頭）", True), ("セッションの依頼", "session-request の start・continue・fork", False),
  ("答え", "agent-answer の field-values（ステップの欄だけ）", True), ("指紋を確かめて書く", "一時ファイルで置き換え、書く前と後の中身から差分を求める", False), ("記録", "operations・field_changes（diff から）", True), ("閉じる", "単発なら sessions.ended_as（別のトランザクション）", True), ("CLI の最後の行", "operation-result の change", False)])
F['overview'] = figman.overview(); F['layers'] = figman.layers(); F['hex'] = figman.hexf(); F['crates'] = figman.cratesf()
F['stages'] = figman.stages(); F['model'] = figman.model(); F['datamap'] = figman.datamap(); F['bnd'] = figman.bnd(); F['gone'] = figman.gone()
REPORT += [(n, next((x for x in r if '検査1〜4' in x), '')) for n, r in figman.REPORT]
def schg_html():
    out = '<div class="schg">'
    for g, ft, files in SCHG:
        out += f'<div class="sg"><b>{g}</b><span class="hint">{ft}</span>' + ''.join(f'<div class="sf"><code>{html.escape(f)}</code><span>{html.escape(d)}</span></div>' for f, d in files) + '</div>'
    return out + '</div>'
xt_html = '<div class="tw"><table><tr><th>x-</th><th>置く所</th><th>中身</th><th></th></tr>' + ''.join(f'<tr><td><code>{html.escape(a)}</code></td><td>{b}</td><td>{c}</td><td>{"<span class=\"tag t-new2\">"+d+"</span>" if d else ""}</td></tr>' for a, b, c, d in XTERMS) + '</table></div>'
TABLES5 = ["sessions", "session_turns", "operations", "field_changes", "rule_results"]
MAPG = [("operation-request", "W", "W", "W", "", ""), ("procedure-prompt", "", "W", "", "", ""), ("session-request（reply）", "W", "W", "", "", ""), ("agent-answer", "", "W", "", "W", "W"), ("diff", "", "", "WR", "WR", ""),
        ("session（出力）", "R", "R", "", "", ""), ("operation・history（出力）", "", "", "R", "", ""), ("rule-result（出力）", "", "", "", "", "WR"), ("validation（出力）", "", "", "R", "", "R")]
map_grid = (f'<div class="need" style="--n:5"><div></div>' + ''.join(f'<div class="nh">{t}</div>' for t in TABLES5)
  + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if v else ""}">{ {"W":"書く","R":"読む","WR":"書く・読む"}.get(v,"") }</div>' for v in vs) for r, *vs in MAPG) + '</div>')
T_NEW = {
 'domain': ('domain', '事業領域と文脈',
  '<p class="ans">事業領域 SAS を分解して業務領域を見つけ、SAS 自身の4つを、意味の境界の内側の広い1つの文脈「契約とインスタンス」に置く。一般の4つと使う側は外の文脈。</p>'
  + fig(F['layers'], '事業領域・業務領域・区切られた文脈', '業務領域は見つけるもの、文脈は設計するもの。区分は仮で、確かめるときに実装の方法から逆に判定する。')
  + fig(F['ctxmap'], '文脈の地図', '中核を含む文脈「契約とインスタンス」は、変わりやすいツールと Git にモデル変換装置を置く。使う側へは公開された言葉（Abstract と CLI の出力）だけを出す。')
  + cards([("意味の境界", "ツールの「セッション」・Git の「コミット」・使う側の「目的のセッション」は文脈「契約とインスタンス」の語と意味が違う"),
           ("契約と製品", "一般の能力は、契約（ポートと SAS の型）を SAS が持ち、製品はアダプターで差し込む"),
           ("切り出す契機", "描画が別のデザインシステムや別のチームを持ったら、下流の文脈へ")])
  + det('業務領域（区分・実装の方法・受け持つユースケース・手がかり）', tbl(['業務領域', '区分', '実装の方法', 'ユースケース', 'アクター／外部システム／データ'], SUB))
  + det('用語集（同じ意味の語と、意味が違う語）', tbl(['範囲', '語'], WORDS))
  + det('外した案と理由', tbl(['外した案', '理由'], CTXNO))),
 'model': ('model', 'ドメインモデル',
  '<p class="ans">集約は3つ。Concrete・インスタンス（契約への適合）と、SAS が立てたセッション（意味の操作）。上から決め、下から確かめて同じ形になったものだけを domain に置く。</p>'
  + fig(F['model'], '集約・値・サービス', '契約への適合の domain は何にも依存しない。意味の操作の domain はその値を使う。操作の記録と描画は domain を持たない（スクリプト）。点線の箱は use_cases に残すもの。')
  + fig(F['datamap'], 'データとドメインモデルの対応', 'ファイルの中身は Git。操作の記録は集約にせず、追記する行のまま。')
  + cards([("3つの集約", "Concrete・インスタンス・SAS が立てたセッション。1回に書く集約は1つ"),
           ("業務イベントは作らない", "購読する手続きが無い。現れたら足す"),
           ("SAS を通らない変更", "インスタンスの集約が見分ける。最後の指紋は use_cases が操作の記録から読んで値で渡す")])
  + det('不変条件', tbl(['集約', '不変条件'], INV)) + det('コマンド', tbl(['集約', 'コマンド'], CMD))
  + det('値オブジェクトとドメインサービス', '<p class="hint">値：' + VAL + '</p>' + tbl(['ドメインサービス', 'すること'], SVC))
  + det('外した案と理由', tbl(['外した案', '理由'], MNO))),
 'arch': ('arch', 'アーキテクチャ',
  '<p class="ans">文脈全体を1つのヘキサゴナルにする。内側に domain と4つの application、縁にポート9つ、外にアダプター6つと入口の CLI。依存の向きは crate で強制する。</p>'
  + fig(F['hex'], '1つのヘキサゴナル', 'ポートは application が宣言し、outbound のアダプターが実装する。外のライブラリはそれを使う outbound の crate だけが依存する。同じ文脈のモジュールどうしは application を直接呼ぶ。')
  + fig(F['crates'], 'crate の層', 'crate は13：domain 2・application 4・outbound 6・cli 1。名前は contracts-and-instances-&lt;業務領域&gt;-&lt;層&gt;。outbound に依存してよいのは組み立てを持つ cli だけ。')
  + tree_html
  + cards([("ヘキサゴナル1つ", "データアクセスはレイヤードの語で、ここには無い。外の I/O はスクリプトのモジュールもポートを通す"),
           ("crate とモジュール", "向きを強制したい境界（業務領域 × 層）とライブラリを閉じ込める所だけ crate。集約・値・サービス・ユースケースはモジュール"),
           ("ポートは dyn", "9つとも dyn で使い、dyn compatible に保つ（型引数のあるメソッド・async fn・関連定数を持たない）")])
  + det('crate と依存してよい向き', tbl(['crate（contracts-and-instances- を省略）', '層', '依存してよい crate'], CRATES))
  + det('ポートの一覧', tbl(['ポート（会話の目的）', '持ち主のモジュール', 'アダプター', '受け渡すもの'], PORTS))
  + det('外のライブラリを閉じ込める所', tbl(['ライブラリ', 'どの crate が依存するか', '扱い'], LIBS))
  + det('言語に依存しない決まり', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in RULES_ANY) + '</ul>')
  + det('Rust の決まり（一次情報の頁と行）', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in RULES_RS) + '</ul><p class="hint">原文は 2026-10-31 に取得して照合した（The Rust Programming Language・The Cargo Book・The Rust Reference・Rust API Guidelines。識別子33個・引用51件が原文と一致）。crate の数の基準は原文に無い。</p>')),
 'schema': ('schema', 'スキーマと DB',
  '<p class="ans">境界（誰が書き、誰が読むか）ごとに1つのスキーマを立て、schemas/v1 は14。DB は5表で公開せず、どの欄も CLI の出力の形のどこかに出る。</p>'
  + fig(F['bnd'], 'スキーマの境界', 'JSON で表したい・型で制御したい・意味を持たせたい・決まった型で返したい、のどれかに当たる境界だけをスキーマにする。DB の表と版の管理・検査器・テンプレートの処理系はプロセスの外へ JSON が出ないのでスキーマにしない。')
  + schg_html()
  + cards([("Concrete を書く型", "SAS は Concrete を作るところまで持つので、x- 項目10個と用語集の型を持つ"), ("意味の操作", "手順のプロンプト・セッションの依頼・エージェントの答えの3つで、頼む側と答える側を対で持つ"), ("公開しない", "DB の DDL。出力では DB の名前を直し、DDL を変えても出力のスキーマは変わらない")])
  + '<h3>Concrete を書くための x- 項目</h3>' + xt_html
  + '<h3>意味の操作の流れ（どの段でどのスキーマと表を使うか）</h3><div class="two"><div>' + fig(F['fv'], '検証の流れ', '検証。濃い段は DB を書く・読む段。', 1.2) + '</div><div>' + fig(F['fu'], '更新の流れ', '更新。答えの各項目はステップの番号と JSON ポインタを持ち、SAS はステップの欄だけへ書く。', 1.2) + '</div></div>'
  + '<p class="hint">CLI の出力は1行1つの JSON（JSON Lines）。意味の操作は1行目が手順のプロンプト（procedure）、最後の行が結果。手順を行うのは SAS が立てたセッション。操作の記録とセッションを閉じるのは別のトランザクション（1回に書く集約は1つ）。</p>'
  + '<h3>スキーマと DB の対応</h3><div class="fig">' + map_grid + '</div><p class="hint">DB にだけある欄は無い。スキーマにだけあるのは、記録しないと決めたもの（描画・検索の入力、当てる前の値、決めきれず返した点、検索の答え、ファイルが正本の設定）だけ。</p>'
  + fig(F['gone'], '外したものと引き継ぐ先', 'vocab-meta は vocabulary へ改名（x-view を足す。x-prompt はいまも在る）。SAS の版のタグが無いので未公開と見る。公開していたら v2 として作る。')
  + det('14のファイルと主な $defs', ''.join(tbl(['ファイル', '中身'], [[f'<code>{html.escape(f)}</code>', d] for f, d in files]) for g, ft, files in SCHG))
  + det('書く向きの対応', tbl(['スキーマ', 'DB'], MAPW)) + det('読む向きの対応（出力で直す名前）', tbl(['DB', '出力'], MAPR))
  + det('外したものの行き先', tbl(['外したもの', '行き先'], GONE2))
  + '<h3>原典と照合した結果（2026-10-31）</h3>' + tbl(['項目', '原典（版）', '結果と直した所'], [[f'<span class="tag t-chk">照合済み</span> {a}', b, c] for a, b, c in FACT]) + f'<p class="hint">原文は取得日と sha256 と一緒に残した。{UNREAD}。</p>'),
}
UC_FIX = tbl(['ユースケース', '直した保証・足した拡張'], [["UC3 削除する", "対象と、参照している所の記載が、同じセッションの操作として消える"], ["UC6 検索する", "検索は操作として残さない（立てたセッションは残る）。絞り込みは継続のセッションで続けられる"],
  ["UC1 作成する", "意味の作成でセッションの型を指定できる（既定は単発・引き継がない）。拡張：アダプターが能力を持たない → unsupported_capability（足りない能力と代わりの指定）"],
  ["UC2 更新する", "型を指定できる。拡張：続けて直す（継続・引き継ぐ）、能力が無い → unsupported_capability"],
  ["UC6 検索する（型）", "型を指定できる。拡張：能力が無い → unsupported_capability"]])


# ---------- 確かめる：上から決めて下から確かめる（版14） ----------
_IA = json.load(open(os.path.join(H, '..', 'advisors', 'impl-ddd.json')))
BOTTOM_FULL = [r for r in _IA['code'][1]['rows']]
for r in BOTTOM_FULL:
    r[0] = r[0].replace('どの UC も：操作を追記する', '作成・更新・削除・検証・取り込み：操作を追記する')
    r[3] = r[3].replace('再開の決まり6つ', '再開の決まり5段')
    r[2] = r[2].replace('再開の決まり6つ', '再開の決まり5段')
TOPDOWN = [("契約への適合", "ドメインモデル（中核）", "版は閉じた一覧・本体の x- の名前を足さない・id を変えない・導出値の一致・人の欄は人の入口から・指紋の一致。作成・更新・削除・取り込みの4つの経路で同じく守る"),
           ("意味の操作", "ドメインモデル（中核）", "セッションの型ごとの不変条件・再開の決まり5段・やり直しの決まり（決めたことの行）"),
           ("操作の記録", "トランザクションスクリプト（補完）", "追記だけ・request_id は1回を DDL・trigger・一意の索引が守る。手続きは行の出し入れ"),
           ("描画", "トランザクションスクリプト（補完）", "テンプレートに値を入れて返すだけで、書かない")]
BU = [("UC1 作成（機械）", "Concrete・インスタンス", "版・URN・指紋", "指す先の実在", "形の検査（ポート）"),
      ("取り込み（--adopt）", "インスタンス（指紋・人の欄）", "差分", "", "$schema を決める・衝突を返す"),
      ("UC2 更新", "同じ2つ（同じ理由で変わる）", "判定の結果", "", "版の移し方"),
      ("UC3 削除", "", "", "参照元を探す", "参照元を1件ずつ消して回る"),
      ("UC4 検証", "", "判定の結果（5つの値）", "", "範囲の選び方"),
      ("SF1 手順を行わせる", "SAS が立てたセッション", "型・能力", "再開・やり直し", "プロンプトを組む・呼ぶ"),
      ("UC5 描画", "", "判定の結果を使う", "", "描く"),
      ("UC6 検索", "", "", "", "探す"),
      ("UC7 読む", "", "差分を使う", "", "記録を読む"),
      ("操作を追記する", "", "", "", "行を足す")]
bu_grid = ('<div class="need" style="--n:4"><div></div>' + ''.join(f'<div class="nh">{h}</div>' for h in ["集約", "値オブジェクト", "ドメインサービス", "スクリプトのまま"])
    + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if v else ""}" style="font-size:12px">{v}</div>' for v in vs) for r, *vs in BU) + '</div>')
RECON = [("業務領域ごとの実装の方法", "一致", "―"), ("集約（Concrete・インスタンス・SAS が立てたセッション）", "一致", "―"),
         ("差分", "上の見落とし", "値オブジェクトにする（UC2・取り込み・SF1・UC7 が使う）"),
         ("SAS を通らない変更の検出", "上の置き違い", "操作の記録からインスタンスの集約へ移す（最後の指紋は値で受け取る）"),
         ("形の検査と x- の判定", "上の見落とし", "形の検査は schema_checker のポートの向こう、x- の判定は集約とサービス"),
         ("業務イベント（events/）", "上の先取り", "作らない。購読する手続きが現れたら足す"),
         ("契約への適合の中のスクリプト", "上の先取り", "検索・参照元を消して回る・版の移し方などは use_cases に残す"),
         ("操作の記録の集約2つ", "上の先取り", "集約にせず、追記する行にする")]
RC = {"一致": "m-keep", "上の見落とし": "m-warn", "上の置き違い": "m-warn", "上の先取り": "m-warn"}
recon_grid = ('<div class="mx" style="grid-template-columns:minmax(0,1.3fr) minmax(0,.7fr) minmax(0,2fr)"><div class="mh">項目</div><div class="mh">判定</div><div class="mh">直す方</div>'
    + ''.join(f'<div class="mr">{a}</div><div class="mc {RC[b]}"><b>{b}</b></div><div class="mc m-none">{c}</div>' for a, b, c in RECON) + '</div>')
T_VERIFY = ('verify', '確かめる',
  '<p class="ans">上から決めた構成を、ユースケースの手続きから下から確かめた。業務領域ごとの実装の方法と3つの集約は一致し、構成は妥当。食い違い6件は上からの設計を直した。</p>'
  + fig(F['stages'], '探索の4つの段階と引き上げの表', '段階3でユースケースの手続きをそのまま書き、段階4で何になるかを見分ける。似たものも、同じ意味・同じ理由で変わると確かめるまで引き上げない。')
  + '<h3>上から：ドメインモデルが自明な所と根拠</h3>' + tbl(['業務領域', '判定', '根拠（守る条件がすでに書けている所）'], TOPDOWN)
  + '<h3>下から：ユースケースの手続き → 何になるか</h3><div class="fig">' + bu_grid + '</div><p class="hint">緑のマスがそのユースケースの手続きで見つかったもの。手順とルールの全文は下に畳んだ。</p>'
  + '<h3>突き合わせ</h3>' + recon_grid
  + cards([("食い違いのとき", "部品の置き場所は下を、境界と公開する契約は上を採る。どちらにも実物が無ければスクリプトに残す"),
           ("自明なら上から", "守る条件がすでに決めたことや試験の場面に書けている所は、探りを入れず初めから domain に置く"),
           ("ドメインモデルは育てる", "スクリプトに残したものも、ほかのユースケースが同じ理由で使い始めたら引き上げる")])
  + det('段階3の手続きとルール（全文）', tbl(['ユースケース', '段階3：主な手順', '出てくるルール', '段階4：何になるか'], BOTTOM_FULL))
  + det('名前が同じで別のもの', tbl(['名前', '別々のもの'], [["検証", "形の検査（2020-12、一般）・機械の rule（x- の意味）・意味の rule（SF1）"], ["指紋", "対象のファイルの指紋と、手順のプロンプトの prompt_hash"], ["セッション", "SAS が立てた（状態を持つ集約）と、呼び出し元（見分けた記録の行）"], ["版", "$id の v&lt;版&gt; と Git の版"]])))

# ---------- 決めたこと・承認 ----------
DECIDED = [
 ("変更", "業務領域は4つと一般", "意味の操作・契約への適合（中核・ドメインモデル）、操作の記録・描画（補完・トランザクションスクリプト）、一般の4つは作らずに使う（版14 で契約への適合と操作の記録の区分を入れ替えた）"),
 ("足した", "文脈は「契約とインスタンス」の1つ（SAS は事業領域で、文脈ではない）", "意味が違う語（ツールのセッション・コミット・目的のセッション）の手前に境界。一般と使う側は外の文脈"),
 ("変更", "集約は3つ", "Concrete・インスタンス・SAS が立てたセッション。呼び出し元のセッションと操作は集約にせず追記する行（版14）。1回に書く集約は1つ"),
 ("足した", "ディレクトリ", "（旧）リポジトリの根に業務領域ごとのモジュールを置いていた。版15 で、リポジトリの根は事業領域、その直下に文脈の名前のフォルダ（contracts-and-instances/）、その中に業務領域のモジュールと adapters/ に直した"),
 ("変更", "スキーマは14、1ファイル1境界", "Concrete を書く型4・CLI の入出力2・意味の操作3・共通の値2・差し込む物の契約3。call と用語集は外さず、call は手順のプロンプトとセッションの依頼に分ける"),
 ("足した", "DB は5表、公開しない", "sessions・session_turns・operations・field_changes・rule_results。どの欄も出力のどこかに出る。出力では DB の名前を直す"),
 ("照合", "x- は10個（足すのは x-view だけ）", "x-prompt はいまの vocab-meta に在った。方言の $dynamicAnchor: meta で語彙が入れ子に掛かる（Core 2020-12 で照合）"),
 ("足した", "意味の操作の CLI の出力", "JSON Lines。1行目が手順のプロンプト、最後の行が結果。手順を行うのは SAS が立てたセッション"),
 ("足した", "記録と閉じるは別のトランザクション", "操作の記録とセッションを閉じることを分ける。間で止まったら状態を読む操作が閉じ直す（反証で直した）"),
 ("変更", "既定は単発・引き継がない", "意味の作成・更新・検索の既定、検証は固定。ほかの型は指定とアダプターの能力で使う。決めたことは --brief の文で渡す"),
 ("足した", "書くのは SAS だけ", "エージェントは決めた形の JSON を返す。機械の検証と指紋の確認を通ったときだけ SAS が書く"),
 ("変更", "セッションの4つの型と再開の決まりは SAS が持つ", "単発・継続 × 引き継ぐ・引き継がないと、立てる・続ける・fork・閉じる・状態を読むを基盤の能力として持つ。目的ごとの営みは使う側（前は「型と再開の規則は使う側」）"),
 ("足した", "一般は契約と製品に分ける", "能力の契約（ポートと SAS の型）は SAS の中、製品（コーディングエージェント・Git・検査器・テンプレートの処理系）はアダプターで差し込む"),
 ("変更", "差分は SAS の型で表す・Git は読むだけ", "SAS が書いた操作は書く前後の中身から、SAS を通らない変更と継続で対象が変わったときは Git のコミット済みの版を読んで、変わった欄を SAS の差分の型で出す。版を残す仕組み（refs/sas/ で指す・復元）は持たない"),
 ("変更", "crate と DDD の部品をディレクトリで表す", "業務領域 × 層ごとに crate（13）。domain は契約への適合と意味の操作だけで aggregates・values・services（events は作らない）、application は use_cases と ports"),
 ("変更", "DB の役目は動いているセッションを回すこと", "再開・回・形の直し・やり直しのために持つ。DB の中身はコミットしない。終わったあとの記録は分析に使える程度で、人のチャットまでは集めない"),
 ("足した", "描画はデザインシステムを受け取る契約", "種類がそろうかを確かめてから描く。SAS は既定のデザインシステムを持たない"),
 ("足した", "必要十分", "ユースケースのステップ × 能力の格子で、余分も不足も無い"),
 ("変更", "セッションを立てずに行う操作は無い", "SAS が立てたセッションが無ければ、呼び出し元のセッションを記録する"),
 ("変更", "検証は残す・検索は残さない", "検証には結果がある。検索は残す使い道が無い"),
 ("足した", "SAS を通らない変更を良しとしない", "描画したものはコミットしない。ファイルが SAS を通らず変わったら検証で知らせる"),
 ("変更", "削除は参照している所も消す", "参照元の書き換えも、同じセッションの操作として残る"),
 ("変更", "PR・コミットは SAS が担わない", "すべてローカル PC で動く。何を PR に添えるかは使う側"),
 ("照合", "CHECK の NULL のすり抜けを閉じた", "SQLite の CHECK は式が NULL のとき違反にしない。resume の resume_decision と validate の前後の指紋を IS・IS NOT NULL で書き直し、断ることを SQLite 3.46.1 で確かめた"),
 ("照合", "エージェントの接続の定義を原文に合わせた", "環境変数で ID を取れるのは Claude Code だけなので caller.env を任意にし、無いときは --caller。SAS が ID を作って渡す assigned を足す。閉じる・状態を読むコマンドはどのツールにも無いので省略できる"),
 ("照合", "Design Tokens と MCP の版を固定", "Design Tokens Format Module 2025.10、MCP 2026-07-28。「群」は原典の group に直した"),
 ("変更", "文脈全体を1つのヘキサゴナル", "外の I/O はスクリプトのモジュールもポートを通す。データアクセスはレイヤードの語でここには無い。active-record・data-access の crate は無い（版13 のモジュールごとの技術方式を取り消した）"),
 ("照合", "Rust の一次情報に合わせた構成", "pub mod と pub(crate)、mod.rs を使わない、application が domain の型を再公開、組み立ては cli の main.rs、resolver = \"3\" と workspace.dependencies、ポート7つは dyn（dyn compatible に保つ）、集約の欄は private、orphan rule のため sqlite は行の型で変換"),
 ("版2", "判定の結果はコミットしない", "OK かを決めるのは呼んだ人。CI は機械の検証だけ"),
 ("版2", "意味の作成・更新・検索も同じ手順", "手順のプロンプトを返し、SAS が立てたセッションで行わせる"),
 ("版2", "人の判定は呼び出し元のセッション", "SAS は by: human の rule を「人が決める」と返すだけ"),
 ("版2", "DB はセッションと操作", "1つのセッションに操作が複数。操作は意図と、変わった欄の前後を持つ"),
 ("版2", "CLI は必ずセッション ID を返す", "使う側はそれを自分の Concrete に記録する"),
]
DECIDED += [
 ("足した", "上から決めて下から確かめる", "自明な所は上から domain に置き、ユースケースの手続きを段階3で書いて段階4の表で見分け、突き合わせて確かめたものだけを domain に置く。一致：実装の方法と3つの集約。食い違い6件は上を直した"),
 ("足した", "ポートは9つ", "agent_runtime・keep_prompt・agent_session_store（意味の操作）、document_store・version_reader・schema_checker（契約への適合）、identify_caller・operation_log（操作の記録）、template_renderer（描画）。呼び出し元を見分けるのはいちばん下の操作の記録（循環を避ける）"),
 ("足した", "domain は2つの crate", "conformance-domain と semantic-operations-domain（前者に依存）。1つの crate の中では集約の間の向きを止められないため（Book ch07-03:296）"),
]
DECIDED += [
 ("変更", "SAS は事業領域で、文脈ではない", "リポジトリの根 semantic-agent-schema/ は事業領域の置き場。直下に文脈の名前のフォルダ（いまは contracts-and-instances/ の1つ）と、Cargo.toml・architecture.json・docs/ など事業領域全体のものだけを置く。旧版の「文脈の根＝リポジトリの根」はやめた"),
 ("変更", "文脈の名前は「契約とインスタンス」", "文脈の言葉の実物（ユースケースの「いまの契約のすべての rule」「契約・インスタンス・記録」）から付けた。「文書の管理」はユースケースと Abstract にほとんど出ない語なので採らなかった"),
 ("変更", "業務領域の名前を活動の名前に", "契約への適合（conformance。旧名：型とインスタンス）・意味の操作（semantic-operations）・操作の記録（operation-recording。旧名：操作の経緯）・描画（rendering）"),
 ("変更", "crate の名前", "contracts-and-instances-&lt;業務領域&gt;-&lt;層&gt;。製品の名前は CLI のコマンド名（[[bin]]）にだけ残す"),
]
APPROVE = [
 ("土台の文の案（ディレクトリと文脈）", "置き方の決まり：「文脈の根は、文脈の名前のフォルダ（リポジトリの根の直下）。リポジトリの根は事業領域の置き場で、文脈の根にしない。文脈の名前はその文脈の言葉から付け、事業領域や製品の名前を使わない」。ディレクトリの図の注記と、文脈の型1・型2（いまの型2 の「業務領域をフォルダの名前にする」書き方も直す）を同じ形に。土台は正本なので ACDR で直す"),
 ("スキーマ14と DB 5表", "スキーマと DB のタブのとおり。見解付きで決めた3点：CLI の返り値の読み方（SAS が立てたセッションが手順を行う）、v1 の改名（未公開と見る。公開していたら v2）、ended_as を9つ（closed・idle を足す）"),
 ("trigger（確かめる）", "見解：不変条件の多くを trigger で守る。本文はスキーマ定義を書くステップで DDL と一緒に書き、違反の行を断ることを試す"),
 ("業務領域・文脈・集約・ディレクトリ", "事業領域と文脈・ドメインモデル・確かめる・アーキテクチャのタブのとおり（契約への適合は中核・ドメインモデル、操作の記録は補完・スクリプト、集約は3つ、crate は13、ポートは9つ）"),
 ("記録が抜けたときの知らせ方（確かめる）", "見解：DB の記録が失敗した変更も、次の検証で SAS を通らない変更として知らせ、--adopt で直す。新しい仕組みは足さない"),
 ("Git に前の版が無いときの取り込み（確かめる）", "見解：欄ごとの前の値を空にして取り込み、取れなかったことを返す"),
 ("セッションの能力", "4つの型と5つの操作を SAS が持つ。既定は単発・引き継がない、検証は固定。再開の決まりは5段。能力が無ければ unsupported_capability"),
 ("一般の能力の契約", "エージェントを動かす・版の中身を読む・JSON Schema で検査する・デザインシステムを読む。製品はアダプターで差し込む"),
 ("sessions と operations に足す欄", "sessions：lifetime・context・inherited_from・continued_from・capabilities。operations：request_id・source。再開の決まりの段は session_turns"),
 ("書くのは SAS だけ", "形の誤りは同じセッションで2回まで直させ、時間切れは新しいセッションで2回までやり直す"),
 ("終わり方", "ended_as は sas が completed・closed・timeout・invalid_output・conflict・failed・cancelled・gone、caller が closed・idle"),
 ("SAS を通らない変更の見つけ方と取り込み方", "インスタンスの集約が、いまの指紋と最後の操作の事後の指紋を比べて見分ける。取り込みは --adopt と意図。前の値は Git のコミット済みの版を読んで取る。無ければ空にして、取れなかったことを返す"),
 ("人のセッションをいつ立て、いつ閉じるか", "人が CLI を初めて実行したときに立てる。sas session end か、一定の時間 CLI が実行されなかったときに閉じる"),
 ("呼び出し元のセッション ID の取り方", "identify_caller のアダプターがツールのセッション ID を取る。取れなければ --session。どちらも無ければ人のセッション"),
 ("土台の「中の形」の文を直す（ACDR が要る）", "見解：「中の形は、文脈ごとに1つのヘキサゴナルにする。外とのやりとりはすべて application の ports を adapters が実装する。業務ロジックの置き方は、上から決めて下から確かめる。上から：守る条件がすでに決定や試験の場面に書けている所は、初めから集約・値・ドメインサービスを domain に置く。下から：ユースケースの手続きをトランザクションスクリプトとして書き、引き上げの表で見分ける。2つを突き合わせ、部品の置き場所は下を、境界と公開する契約は上を採る。補完はスクリプトで止めてよい」に直す。土台は正本なので ACDR で差分を見せてから直す"),
 ("ガイドの確かめる段階（ボード Q5）に手順を足す", "見解：① 上からの判定のうち自明とした所と根拠を表にする ② ユースケースごとに段階3の手続きとルールを表にする ③ 段階4の表で何になるかを見分ける ④ 突き合わせの表（一致・上の先取り・上の見落とし）を作り、直す方を書く ⑤ 直した結果で domain の中身を確定する。ボードは次の回で直す"),
 ("crate は13", "見解：domain 2・application 4・outbound 6・cli 1。向きを強制したい境界とライブラリを閉じ込める所だけを crate にした。原典に数の基準は無い"),
]
TAGC = {"足した": "t-new", "変更": "t-chg", "照合": "t-chk", "版2": "t-old"}
def tagged(items):
    return '<div class="dec">' + ''.join(f'<div class="dc {TAGC[t]}"><span class="tag">{"版2 のまま" if t == "版2" else t}</span><b>{h}</b><span>{d}</span></div>' for t, h, d in items) + '</div>'

# ---------- 能力のマトリクス（HTML の格子） ----------
CAP = [("作成", "人や道具が値を書く", "x-prompt を指示に値を書く", "残る（create）"),
       ("更新", "人や道具が書き換える", "x-prompt を指示に書き換える", "残る（update）"),
       ("削除", "参照元の記載も一緒に消す", None, "残る（delete と参照元の update）"),
       ("検証", "JSON Schema と by: machine の rule", "by: ai の rule を判定する", "残る（validate と rule ごとの結果）"),
       ("描画", "テンプレートとトークンを注入し、その場で描く", None, "残らない（コミットしない）"),
       ("検索", "構造で絞る", "意味で探す（中核）", "残らない（立てたセッションは残る）")]
cap_grid = '<div class="mx"><div class="mh"></div><div class="mh">機械で</div><div class="mh">意味で（エージェント）</div><div class="mh">操作として</div>' + ''.join(
    f'<div class="mr">{a}</div><div class="mc m-mach">{b}</div><div class="mc {"m-sem" if c else "m-none"}">{c or "―"}</div><div class="mc {"m-keep" if d.startswith("残る") else "m-none"}">{d}</div>' for a, b, c, d in CAP) + '</div>'

# 4つの型の格子（SAS が4つとも持つ）
q11 = ('<div class="q11"><div></div><div class="mh">引き継がない</div><div class="mh">引き継ぐ（fork）</div>'
       '<div class="mr">単発</div><div class="mc m-sem"><b>既定</b>：意味の作成・更新・検索。<b>検証はこれに固定</b></div><div class="mc m-keep">呼び出し元の決めたことが --brief に収まらない作成・更新</div>'
       '<div class="mr">継続</div><div class="mc m-keep">検索の絞り込み・同じ Concrete のインスタンスを続けて直す</div><div class="mc m-keep">Concrete を変えた考えを持ったまま violated を続けて直す（UC2 の拡張）</div></div>'
       '<p class="hint">寿命 × 文脈の4つの型を、SAS のセッションの能力として4つとも持つ。どの型が使えるかは、差し込んだアダプターの能力の宣言で決まる。目的ごとの営み（目的のセッション・役割・承認）は使う側。</p>')
OPS = ('<div class="mx" style="grid-template-columns:repeat(5,minmax(0,1fr))">' + ''.join(f'<div class="mc m-sem"><b>{a}</b><br>{b}</div>' for a, b in [
       ("立てる", "型（寿命・文脈）を指定"), ("続ける", "再開の決まりで再開か新規"), ("fork", "引き継いで分ける"), ("閉じる", "完了・取り消し"), ("状態を読む", "ツールの側で消えたら closed")]) + '</div>')

MAP27 = [("残す", "U12 人が読む形で読んで判断する", "UC5 描画する"),
 ("直す", "U0 成果物を確かめた状態で使い続ける", "UC0"), ("直す", "U1 Concrete を作る・U5 インスタンスを作る", "UC1"),
 ("直す", "U2 Concrete を変える・U6 インスタンスを変える", "UC2"), ("直す", "U3 Abstract の新しい版に合わせる", "UC2 の拡張"),
 ("直す", "U8 変更のたびにすべてを検証する", "UC4（主アクター CI、全件）"), ("直す", "U10 効かなくなった判定を取り直す", "UC4（入力が変わった rule だけ）"),
 ("直す", "S1 検証する", "UC4 へ上げる"), ("直す", "U11 置き換えて残す", "UC3 と UC2"), ("直す", "U13 経緯をあとから確かめる", "UC7"),
 ("直す", "U16 人が決める欄を承認する", "UC4 の拡張（人が決める → 呼び出し元のセッション）"), ("直す", "U17 既存のファイルを取り込む", "UC1 の変形"),
 ("直す", "U26 探す範囲を宣言する", "UC6 の事前条件"), ("直す", "U28 出どころ付きの答えを返す", "UC6 の意味で探す"),
 ("直す", "U23 拡張項目・U24 x-relationship で読む", "UC1・UC2・UC4 の中身"), ("直す", "S2 変更を記録する", "すべての UC の「操作を残す」ステップ"),
 ("直す", "S4 セマンティックの手順を返す", "SF1"),
 ("外す", "U4 Abstract の新しい版を出す", "SAS を作る側の作業"), ("外す", "U14 validator をつなぐ", "UC4 の事前条件「アダプターが設定されている」"),
 ("外す", "U18 ほかの変更を取り込む", "取り込みは Git、取り込んだ後は UC4"), ("外す", "U25 変更を確かめる・S3 変更のまとめを作る", "使う側"),
 ("外す", "U27 目的のセッション・N3 子のセッションに任せる", "使う側のオーケストレーター")]


# ---------- 必要十分：ユースケースのステップ × 能力 ----------
NCOL = ["見分ける", "能力を読む", "立てる", "続ける", "fork", "閉じる", "状態を読む", "版の中身を読む", "差分を求める", "検査する", "デザインシステム", "記録を読む"]
NROW = [("UC1〜UC7 のはじめ", ["見分ける"]),
        ("UC1 作成する（機械）", ["検査する"]),
        ("UC1 作成する（意味）", ["能力を読む", "立てる", "続ける", "閉じる"]),
        ("UC2 更新する", ["差分を求める", "検査する"]),
        ("UC2 の拡張（続けて直す）", ["能力を読む", "続ける", "fork", "状態を読む", "版の中身を読む"]),
        ("UC2 取り込み・UC4 通らない変更", ["版の中身を読む", "差分を求める"]),
        ("UC3 削除する", ["検査する"]),
        ("UC4 検証する", ["検査する", "能力を読む", "立てる", "続ける", "閉じる"]),
        ("UC5 描画する", ["デザインシステム"]),
        ("UC6 検索する", ["能力を読む", "立てる", "続ける", "閉じる"]),
        ("UC7 読む", ["記録を読む", "差分を求める"]),
        ("SF1 手順を行わせる", ["能力を読む", "立てる", "続ける", "閉じる", "状態を読む"])]
need_grid = (f'<div class="need" style="--n:{len(NCOL)}"><div></div>' + ''.join(f'<div class="nh">{c}</div>' for c in NCOL)
    + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if c in cs else ""}">{"●" if c in cs else ""}</div>' for c in NCOL) for r, cs in NROW)
    + '<div class="nr">使う行の数</div>' + ''.join(f'<div class="nc tot">{sum(c in cs for _, cs in NROW)}</div>' for c in NCOL) + '</div>')
NOT_HAVE = ["コミット・PR・ブランチ・履歴の検索", "版を残す・復元（Git に書き込むこと）", "目的のセッションを立てて役割ごとに任せる", "セッションをまとめる・会話を書き換える", "デザインシステムを作る・既定のデザインシステム", "検査器を自作する", "fork の代わりに会話を文字にして渡す"]
T_NEED = ('need', '必要十分',
  '<p class="ans">ユースケースのどのステップも、SAS の契約の能力だけで行える。どの能力もどこかのステップが使い、使わない能力は持たない。</p>'
  + '<div class="fig">' + need_grid + '</div><p class="hint">● はそのステップが使う能力。最後の行が0の列は無い（余分が無い）。どの行も能力が欠けていない（不足が無い）。人が決める欄は能力を足さず、人の判定を UC4 の入力で受ける。</p>'
  + cards([("余分", "使う行が0の能力は無い"), ("不足", "足りない能力は無い"), ("持たない", "コミット・PR、目的のセッション、デザインシステムを作ること")])
  + det('持たない能力', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in NOT_HAVE) + '</ul>'))

# ---------- タブ ----------
T = []
T.append(('overview', '全体像',
  '<p class="ans">JSON Schema を Semantic に拡張し、その型で作成・更新・削除・検証・描画・検索する基盤。どんな Concrete を作るかは使う側が決める。</p>'
  + fig(F['overview'], 'SAS と使う側と一般の製品', 'SAS は能力・記録（動いているセッションを回すため）・公開する形と、一般の能力の契約まで。製品は作らずに差し込む。目的ごとに進める営みは使う側。')
  + cards([("SAS の中", "能力6つ・記録・公開する形・意味としての目的・一般の能力の契約"), ("使う側", "目的のセッション・承認・まとめ・コミット・PR"), ("一般の製品", "Git・コーディングエージェント・検査器・テンプレートの処理系。アダプターで差し込む")])
  + det('目的の2つの読み', tbl(['目的の読み', '置き場所'], [["意味の一部（rule・欄・Concrete が何のためにあるか、1件の操作の意図）", "SAS の中：rule.why・reading.when・根の description・operations.intent"], ["進め方（目的ごとのセッション・親子・承認・コミット）", "使う側"]]))))
T.append(('types', '型と能力',
  '<p class="ans">Abstract は SAS が決め、Concrete とインスタンスは使う側が決める。6つの能力はどの段にも働き、意味が関わるのは作成・更新・検証・検索。</p>'
  + fig(F['types'], '3段の型') + cap_grid
  + '<p class="hint">緑＝意味の側がある・灰＝機械だけ。「操作として」は手元の DB に残るか。</p>'
  + cards([("意味は4つ", "作成・更新・検証・検索。どれも SAS が立てたセッションで手順を行わせる"), ("削除", "参照している所の記載も同じセッションで消す"), ("描画", "その場で描き、コミットしない")])))

TOOLS = ["Claude Code", "Codex CLI", "Gemini CLI", "Copilot CLI", "Kiro CLI"]
TOOLCAP = [
 ("非対話の実行", ["-p", "codex exec", "-p", "-p", "--no-interactive"]),
 ("ID を指定して再開", ["--resume", "codex exec resume &lt;SESSION_ID&gt;", "--resume &lt;UUID&gt;", "--resume= ・ --session-id", "--resume-id"]),
 ("非対話で fork", ["--fork-session", "codex exec fork", None, None, None]),
 ("セッション ID の取り方", ["json の session_id ・ 環境変数 CLAUDE_CODE_SESSION_ID", "--json の thread.started の thread_id", "stream-json の init（キー名は原文に無い）", "形式は原文に無い", "終了時に文で出す"]),
 ("呼ぶ側が ID を決めて渡す", ["--session-id（UUID）", None, None, "--session-id", None]),
 ("答えの形を JSON Schema で指定", ["--json-schema", "--output-schema", None, None, None]),
 ("閉じる・状態を読むコマンド", [None, None, None, None, None]),
]
def toolcell(row, v):
    if v: return f'<div class="mc m-sem"><code>{v}</code></div>'
    if row == "非対話で fork": return '<div class="mc m-warn">無い → fork を指定すると <b>unsupported_capability</b>（代わり：引き継がない＋--brief）</div>'
    if row == "閉じる・状態を読むコマンド": return '<div class="mc m-none">無い → SAS の側で閉じたことにし、状態は回の記録から導く</div>'
    if row == "答えの形を JSON Schema で指定": return '<div class="mc m-none">無い → 文から JSON を読む（形の直しが増える）</div>'
    if row == "呼ぶ側が ID を決めて渡す": return '<div class="mc m-none">無い → 出力から取る</div>'
    return '<div class="mc m-none">無い</div>'
tool_grid = ('<div class="tw"><div class="tg"><div class="mh"></div>' + ''.join(f'<div class="mh">{t}</div>' for t in TOOLS)
  + ''.join(f'<div class="mr">{r}</div>' + ''.join(toolcell(r, v) for v in vals) for r, vals in TOOLCAP) + '</div></div>'
  + '<p class="hint">各社の公式文書と照合した（2026-10-31）。「無い」は取得した頁の読めた範囲に無いという意味。オプションの名前は接続の定義のファイルに書き、SAS の中には書かない。fork を指定したとき非対話の fork を持たないツールは、立てる前に unsupported_capability を返す。</p>')

T.append(('session', 'セッション',
  '<p class="ans">SAS はセッションの4つの型（単発・継続 × 引き継ぐ・引き継がない）と5つの操作を基盤の能力として持つ。既定は単発・引き継がないで、検証はこれに固定する。</p>'
  + OPS + q11
  + fig(F['resume'], '再開の決まり', '継続のセッションに続けて頼むとき、上から当てる。当たった段は記録に残る。使う側は --new で新規を強制できる。')
  + fig(F['unsup'], 'アダプターが能力を持たないとき', '会話を文字にして fork の代わりにしたり、黙って型を落としたりしない。記録と実際がずれるため。')
  + '<h3>照合したツールごとの能力</h3>' + tool_grid
  + fig(F['tree'], 'セッションと操作の親子', '場面 a と d で DB に入るもの。呼び出し元のセッションが、SAS が立てたセッション（子）と操作を持つ。')
  + fig(F['fail'], '失敗したときの扱い', 'どの型でも書くのは SAS だけ。エージェントは決めた形の JSON を返す。同じ request_id の操作は1回だけ残る。')
  + cards([("差し込む", "コーディングエージェントごとのアダプターか、接続の定義（agent-connector）で動く汎用のアダプター"), ("既定の理由", "エージェントが見たものが記録だけになり、記録だけで判定の根拠を説明できる。検証が作った側の考えに引きずられない"), ("同時に動かす数", "接続の定義に上限を持たせ、超える依頼は待たせる")])
  + det('2種類の比べ', sess_tables[0] if sess_tables else '')
  + det('場面ごとの扱い（8つ）', sess_tables[1] if len(sess_tables) > 1 else '')
  + det('外した案と理由', tbl(['外した案', '理由'], [["SAS は単発・引き継がないだけを持ち、ほかの型は使う側（前の案）", "fork や継続が要る使う側が SAS を通らずツールを直接起動し、そのセッションが記録に残らない"], ["能力を自前で作る", "一般の製品を自社で作ることになる"], ["どんな目的のセッションでも立てられるサービスとして出す", "使う側の目的のセッションと意味がぶつかり、どのユースケースも要らない"], ["fork が無いツールで会話を文字にして渡す・黙って型を落とす", "記録と実際がずれる"]]))
  + det('sessions の決まり', tbl(['集約', '決まり'], INV[:1]) + '<p class="hint">費用：手順のプロンプトの決まった前置き（Concrete・rule）を先頭に置けば、前置きのキャッシュが毎回効く。前置きに効くことは、ツールごとに実測で確かめる。各ツールの再開・fork の有無は公式の文書と照合した（上の格子）。</p>')))
T.append(('data', 'データ',
  '<p class="ans">手元の DB は5表。役目は動いているセッションを回すこと（再開・回・形の直し・やり直し）。中身はコミットしない。終わったあとの記録は分析に使える程度。</p>'
  + fig(F['er'], 'ER 図')
  + cards([("1 対 多", "sessions → session_turns・operations → field_changes・rule_results"), ("中身は Git", "パスと指紋で指すだけ。Git は読むだけで、版は残さない"), ("回を分ける", "継続のセッションは手順を何度も受け、形の直しも1回ずつ。だから session_turns を足す")])))
T.append(('flow', '意味の操作の流れ',
  '<p class="ans">呼び出し元が頼み、SAS が単発のセッションを立てて手順を渡し、確かめてから記録して返す。OK かは呼び出し元が決める。</p>'
  + fig(F['flow'], '意味の操作の流れ', '検証の例。意味の作成・更新・検索も同じ手順。「SAS のセッション」は SAS が立てたセッション。', 1.8)
  + cards([("判定はその時のもの", "結果は呼び出し元に返る。コミットしない"), ("by: human", "「人が決める」と返し、判断は呼び出し元のセッション"), ("CI", "機械の検証だけを毎回行う")])))
sc_titles = {k: v[0] for k, v in scn_html.items()}
use_html = ''
for k in 'abcde':
    t, body = scn_html.get(k, (f'場面 {k}', ''))
    use_html += f'<h3>{html.escape(t)}</h3>' + fig(F['sc' + k], t, '「SAS のセッション」は SAS が立てたセッション、「エージェント」は呼び出し元の AI のエージェント。', 1.8) + det('コマンド・返すもの・DB に入る行', f'<div class="scn">{body}</div>')
T.append(('use', '使われ方',
  '<p class="ans">5つの場面を、誰が何を頼み、SAS が何を返し、DB に何が残るかで見る。</p>' + use_html))
T.append(('link', '使う側とのつなぎ方',
  '<p class="ans">SAS の DB は使う側に共通のモデルとして使わせない。約束はセッション ID と呼び出し元の参照の2つだけ。</p>'
  + fig(F['link'], '使う側とのつなぎ方')
  + cards([("SAS → 使う側", "CLI は必ずセッション ID を返す"), ("使う側 → SAS", "--caller で呼び出し元の参照を渡せる。SAS は解釈しない"), ("引ける範囲", "DB は手元にあるので、同じリポジトリの手元で引く")])
  + det('使う側の Concrete の例', link_code)))
T.append(('uc', 'ユースケース',
  '<p class="ans">ユースケースは能力ごとの7件と要約1件、意味の操作のサブ機能1件。</p>'
  + fig(F['uc'], 'ユースケース')
  + det('8件の機械／意味と主アクター', tables(v5sec('uc'))[0] if tables(v5sec('uc')) else '')
  + det('いまの27件との対応', tbl(['扱い', 'いまのユースケース', '移す先'], MAP27))
  + det('直した保証と足した拡張（UC1・UC2・UC3・UC6）', UC_FIX)))
T.append(('decided', '決めたことと承認',
  '<p class="ans">決めたことと承認してほしい決定（「確かめる」は見解付き）。</p><h3>決めたこと</h3>' + tagged(DECIDED)
  + '<h3>承認してほしいこと</h3><div class="dec">' + ''.join(f'<div class="dc t-ask"><span class="tag">承認待ち</span><b>{h}</b><span>{d}</span></div>' for h, d in APPROVE) + '</div>'))

for k, v in T_NEW.items(): T.append(v)
T.append(T_NEED)
T.append(T_VERIFY)
ORDER = ['overview', 'types', 'uc', 'domain', 'model', 'verify', 'arch', 'data', 'session', 'flow', 'use', 'link', 'schema', 'need', 'decided']
T = sorted(T, key=lambda t: ORDER.index(t[0]))
STATUS = [("決まった", "s-ok", "全体像・型と能力・ユースケース・データ・流れ・使われ方・つなぎ方"), ("承認待ち", "s-wait", "事業領域と文脈・ドメインモデル・確かめる・アーキテクチャ・スキーマと DB・セッションの能力（4つの型）・一般の能力の契約・必要十分・sessions の欄・SAS を通らない変更・人のセッション・呼び出し元の ID")]

CSS = open(os.path.join(H, 'base.css')).read()
tabs = ''.join(f'<button type="button" role="tab" id="t-{k}" aria-controls="p-{k}" aria-selected="{"true" if i == 0 else "false"}">{n}</button>' for i, (k, n, _) in enumerate(T))
panels = ''.join(f'<section role="tabpanel" id="p-{k}" aria-labelledby="t-{k}"{"" if i == 0 else " hidden"}><h2>{n}</h2>{b}</section>' for i, (k, n, b) in enumerate(T))
status = '<div class="status">' + ''.join(f'<div class="st {c}"><span class="tag">{t}</span><span>{d}</span></div>' for t, c, d in STATUS) + '</div>'
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
        '<h1>Semantic Agent Schema の概念</h1><p class="read">SAS が何をし、何を記録し、どう使われるか。2026-10-31 版15（SAS は事業領域で文脈ではないことをディレクトリで表し、文脈「契約とインスタンス」を名前のフォルダにした。業務領域を活動の名前に直した）。要求を書く（全体像〜ユースケース）→ 見つける（事業領域と文脈）→ 設計する（ドメインモデル〜スキーマと DB）の順に並べた。</p>'
        + status + f'<nav class="tabs" role="tablist" aria-label="内容">{tabs}</nav>' + panels + '</main>\n' + JS)
open(os.path.join(H, 'index.html'), 'w').write(page)
for n, r in REPORT: print(n, r)
print('ok', len(page))
