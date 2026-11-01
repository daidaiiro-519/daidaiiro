"""Semantic Agent Schema の概念（版29）。土台・設計のページと同じ作り（タブ・結論の1文・図・カード・details の表）で、版5 の決定を図を主役に組み直す。
図は design-svg の figure と chart exchange で作り、ER 図と3段の型は版5（../sas-role4/）の検査済みの図を使う。"""
import os, re, json, subprocess, html
H = os.path.dirname(os.path.abspath(__file__))
R4 = os.path.join(H, 'src')  # 版5 の検査済みの図と頁（3段の型・木・版5 の頁）
DS = os.path.normpath(os.path.join(H, '..', '..', '..', '..', '.claude', 'skills', 'design-svg', 'bin', 'design-svg'))
os.makedirs(os.path.join(H, 'fig'), exist_ok=True)
REPORT = []
import figman
WORDFIX = [('バリデーター', 'バリデーター'), ('差し込む物の契約', '受け取りの型'), ('アダプターで差し込む', 'アダプターとして渡す'), ('差し込んだ', '渡した'), ('差し込む', '渡す'), ('組み立てる', 'コンポジションルートを置く'), ('組み立て', 'コンポジションルート'), ('見分けた', '判別した'), ('見分ける', '判別する'), ('見分け', '判別し'), ('頼み、', '依頼し、'), ('頼む', '依頼する'), ('組む', '作る'), ('道具', 'ツール'), ('依存が要る', '依存が必要'), ('要ると確かめたら', '必要と確かめたら'), ('要らない', '必要ない'), ('札の付いた', 'ラベルの付いた'), ('右の札の', '右のラベルの')]
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
F['fail'] = run('figure', 'fail', {"direction": "TB", "nodes": [
    {"id": "st", "label": "子セッションを起動するか、渡された ID の子セッションへ続けて送る", "role": "focus"},
    {"id": "ask", "label": "エージェントが答える"},
    {"id": "to", "label": "時間切れ → 新しい子セッションでやり直す（2回まで）", "role": "warn"},
    {"id": "shape", "label": "答えの形を確かめる"},
    {"id": "inv", "label": "誤り → 同じ子セッションで直させる（2回まで）。超えたら invalid_output", "role": "warn"},
    {"id": "hash", "label": "対象のハッシュを確かめる"},
    {"id": "cf", "label": "違う → conflict（書かない）", "role": "warn"},
    {"id": "ok", "label": "SAS が書いて記録する", "role": "focus"}],
    "edges": [{"from": "st", "to": "ask"}, {"from": "ask", "to": "to", "label": "答えない"}, {"from": "ask", "to": "shape", "label": "答えた"},
              {"from": "shape", "to": "inv", "label": "誤り"}, {"from": "shape", "to": "hash", "label": "合う"},
              {"from": "hash", "to": "cf", "label": "違う"}, {"from": "hash", "to": "ok", "label": "一致"}]})
F['flow'] = run('exchange', 'flow', {"label": "意味の部分を含む操作の流れ", "participants": ["main", "別のフォルダ", "SAS"], "steps": [
    {"from": "main", "to": "別のフォルダ", "label": "立てて目的を渡す（使う側）"},
    {"from": "別のフォルダ", "to": "SAS", "label": "create --semantic"},
    {"from": "SAS", "to": "別のフォルダ", "label": "手順（x-prompt と rule）"},
    {"from": "別のフォルダ", "to": "SAS", "label": "create --value"},
    {"from": "SAS", "to": "別のフォルダ", "label": "断った理由か created"},
    {"from": "別のフォルダ", "to": "main", "label": "最後の答え"},
    {"from": "main", "to": "別のフォルダ", "label": "続き（再開する）"}]})
SC = {
 'a': ("エージェント", [("エージェント", "SAS", "U10 を作る（--semantic）"), ("SAS", "エージェント", "書く手順（記録しない）"), ("エージェント", "SAS", "create --value で欄の値"),
       ("SAS", "SAS", "検査して書く・DB へ o-0031"), ("SAS", "エージェント", "created とセッションの ID"), ("エージェント", "SAS", "U10 を検証（--semantic）"),
       ("SAS", "エージェント", "判定の手順（記録しない）"), ("エージェント", "SAS", "--answer で判定"), ("SAS", "SAS", "突き合わせて DB へ o-0032"), ("SAS", "エージェント", "結果")]),
 'b': ("エージェント", [("エージェント", "SAS", "範囲と問い（--semantic）"), ("SAS", "エージェント", "探す手順（記録しない）"),
       ("エージェント", "エージェント", "範囲の文書を読んで答える"), ("エージェント", "SAS", "--answer で答えと出どころ"),
       ("SAS", "SAS", "出どころを確かめる"), ("SAS", "エージェント", "答えと出どころの値")]),
 'c': ("人", [("人", "人", "直接書き換え"), ("人", "SAS", "U10 を検証"), ("SAS", "SAS", "ハッシュが違う"),
       ("SAS", "人", "通らない変更と案内"), ("人", "SAS", "--adopt で取り込む"), ("SAS", "SAS", "DB へ o-0043")]),
 'd': ("人", [("人", "SAS", "U11 を削除"), ("SAS", "SAS", "従うインスタンスが無いかを見る"), ("SAS", "SAS", "DB へ o-0040"),
       ("SAS", "人", "o-0040")]),
 'e': ("エージェント", [("エージェント", "SAS", "U10 を更新（--semantic）"), ("SAS", "エージェント", "書く手順（記録しない）"),
       ("エージェント", "SAS", "update --patch"), ("SAS", "エージェント", "断った理由（形が誤り）"), ("エージェント", "SAS", "直して打ち直す"),
       ("SAS", "SAS", "書く・o-0050"), ("SAS", "エージェント", "updated")]),
}
for k, (who, steps) in SC.items():
    parts = [who, "SAS"]
    F['sc' + k] = run('exchange', 'sc' + k, {"label": "場面 " + k, "participants": parts, "steps": [{"from": a, "to": b, "label": l} for a, b, l in steps]})
F['link'] = run('figure', 'link', {"direction": "LR", "nodes": [
    {"id": "u", "label": "使う側の Concrete（例：目的のセッション）：目的・親子・状態・コミット", "role": "kind-2"},
    {"id": "cli", "label": "SAS の CLI"},
    {"id": "s", "label": "SAS の記録（手元の DB・作り直せる）", "role": "focus"}],
    "edges": [{"from": "u", "to": "cli", "label": "セッションの ID（--session ・ 環境変数 ・ sas init）"}, {"from": "cli", "to": "s", "label": "記録する"},
              {"from": "s", "to": "u", "label": "セッションと操作の ID を返し、使う側が自分のインスタンスに書く", "dashed": True}]})
F['uc'] = run('figure', 'uc', {"direction": "TB", "nodes": [
    {"id": "u0", "label": "UC0 その型で成果物を作り、検証された状態で使い続ける（要約・使う側）", "role": "kind-2"},
    {"id": "u1", "label": "UC1 作成する"}, {"id": "u2", "label": "UC2 更新する"}, {"id": "u3", "label": "UC3 削除する"},
    {"id": "u4", "label": "UC4 検証する（変形：意味の判定）"}, {"id": "u5", "label": "UC5 描画する"}, {"id": "u6", "label": "UC6 検索する"},
    {"id": "u7", "label": "UC7 操作の記録を読む"}],
    "edges": [{"from": "u0", "to": x} for x in ["u1", "u2", "u3", "u4", "u5", "u6", "u7"]]})


F['child'] = run('figure', 'child', {"direction": "TB", "nodes": [
    {"id": "q", "label": "意味の部分を含む操作を依頼する（親セッションの ID は --session か環境変数）", "role": "focus"},
    {"id": "c1", "label": "--continue がある → 役割を確かめ、sessions.launch の設定のまま同じ子セッションへ送る"},
    {"id": "b1", "label": "役割が違う（書き込み用の子セッションを検証に）→ writer_as_checker で断る", "role": "warn"},
    {"id": "b2", "label": "同じ子セッションが処理中 → busy（送らない）", "role": "warn"},
    {"id": "c2", "label": "--new か、この親セッションに同じ役割の子セッションが無い → 起動して ID を返す", "role": "focus"},
    {"id": "c3", "label": "この親セッションに直前の同じ役割の子セッションがある → 起動せずに child_exists で案内", "role": "warn"}],
    "edges": [{"from": "q", "to": "c1", "label": "上から当てる"}, {"from": "c1", "to": "b1", "label": "役割"}, {"from": "c1", "to": "b2", "label": "ロック"},
              {"from": "q", "to": "c2", "label": "--continue が無い"}, {"from": "c2", "to": "c3", "label": "直前の子セッションがある"}]})
F['ids'] = run('figure', 'ids', {"direction": "LR", "nodes": [
    {"id": "p", "label": "親：sas を打つ（環境変数 CLAUDE_CODE_SESSION_ID に親セッションの ID）"},
    {"id": "sas", "label": "SAS：親セッションの ID を決め、初めてなら sessions に caller の行", "role": "focus"},
    {"id": "tool", "label": "ツール：claude -p … --output-format json"},
    {"id": "rec", "label": "SAS：出力の session_id を子セッションの ID として sessions に writer の行（model・launch）", "role": "focus"}],
    "edges": [{"from": "p", "to": "sas", "label": "--brief・--ref"}, {"from": "sas", "to": "tool", "label": "起動する"},
              {"from": "tool", "to": "rec", "label": "JSON を返す"}, {"from": "rec", "to": "p", "label": "親セッションと子セッションの ID を返す", "dashed": True}]})
F['unsup'] = run('figure', 'unsup', {"direction": "LR", "nodes": [
    {"id": "req", "label": "--continue で続けて依頼する", "role": "focus"},
    {"id": "cap", "label": "接続の定義の能力（resume）を読む"},
    {"id": "ok", "label": "ある → 同じ子セッションへ送る", "role": "focus"},
    {"id": "ng", "label": "無い → 送らずに unsupported_capability（毎回新しく起動する打ち方を案内）", "role": "warn"}],
    "edges": [{"from": "req", "to": "cap"}, {"from": "cap", "to": "ok", "label": "ある"}, {"from": "cap", "to": "ng", "label": "無い"}]})
F['tree'] = run('figure', 'ptree', {"direction": "TB", "nodes": [
    {"id": "p", "label": "親セッション（caller）4401cead…", "role": "focus"},
    {"id": "w", "label": "書き込み用の子セッション（writer）31cca313…"}, {"id": "c", "label": "検証用の子セッション（checker）3ff0f03a…"},
    {"id": "o1", "label": "changes o-0001・o-0002"}, {"id": "o3", "label": "validations o-0003"}],
    "edges": [{"from": "p", "to": "w", "label": "parent_id"}, {"from": "p", "to": "c", "label": "parent_id"},
              {"from": "w", "to": "o1", "label": "child_session_id"}, {"from": "c", "to": "o3", "label": "child_session_id"}]})
F['er'] = run('figure', 'er16', {"direction": "TB", "nodes": [
    {"id": "s", "label": "sessions：id・tool・folder・via・first_seen", "role": "focus"},
    {"id": "ch", "label": "changes：id・at・kind・target_path・前後のハッシュ・intent・request_id・source・session_id・schema"},
    {"id": "fc", "label": "field_changes：change_id・pointer・change・before・after"},
    {"id": "v", "label": "validations：id・at・target_path・hash・session_id・schema"},
    {"id": "rr", "label": "rule_results：validation_id・rule・pointer・by・result・reads_hash"}],
    "edges": [{"from": "s", "to": "ch", "label": "session_id"}, {"from": "s", "to": "v", "label": "session_id"},
              {"from": "ch", "to": "fc", "label": "1 対 多"}, {"from": "v", "to": "rr", "label": "1 対 多"}]})

# ---------- 版5 から details に入れる表 ----------
v5 = open(os.path.join(R4, 'index.html')).read()
def v5sec(i):
    a = v5.find(f'id="{i}"'); b = v5.find('<section', a + 5); return v5[a:b if b > 0 else len(v5)]
def tables(frag):
    return re.findall(r'<div class="tw">.*?</table></div>', frag, re.S)
sess = v5sec('session'); sess_tables = [wfix(t) for t in tables(sess)]
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
SUB = [("書いて検証する（モジュール writing・validation）", "中核", "ドメインモデル（集約 Concrete・インスタンスは writing、集約を読んで rule に照らす業務サービスと判定の区分は validation）", "UC1 作成・UC2 更新・UC3 削除・取り込み・UC4 検証・入力が変わった rule だけ。意味の部分は x-prompt の欄を書くこと（ステップ7）と by: ai の rule を判定すること（ステップ6）", "人・AI のエージェント（打ったセッション）／Git（読むだけ）・JSON Schema のバリデーター／Abstract・Concrete・インスタンスのファイルと判定の結果"),
       ("記録（モジュール recording）", "補完", "トランザクションスクリプト", "ほかの操作の追記、UC7 操作の記録を読む（sas log。読むだけ）", "人・AI のエージェント・レビューする人／ツールのセッション ID／sessions・changes・field_changes・validations・rule_results"),
       ("検索（モジュール search）", "補完", "トランザクションスクリプト", "UC6（範囲を宣言して、構造で絞る ・ 意味で探す。--answer で出どころを確かめる。記録しない）", "人・AI のエージェント・オーケストレーター／Abstract・Concrete・インスタンスのファイル／書かない"),
       ("描画（モジュール rendering）", "補完", "トランザクションスクリプト", "UC5（ステップ5。デザインシステムとインスタンスの値を受け取り、テンプレートが値を置き、トークンが見た目を決める。同じ入力から同じページ）", "読み手／デザインシステム／書かない"),
       ("版の管理・JSON Schema の検証・描画の部品", "一般", "製品は作らずに使い、アダプターとして渡す。ポートと受け取りの型は、使うモジュールが SAS の中に持つ", "―", "Git・バリデーター・テンプレートの処理系")]
WORDS = [("同じ言葉（文脈「スキーマとインスタンス」）", "セッション（sas を打ったツールのセッション ID。sessions の1行で指す。状態は持たない）・ハッシュ（中身の sha256。記録の列 before_hash・after_hash の値）・書いた変更・検証・変わった欄・rule の結果・ハッシュ・意図・決めたことの文（brief）・依頼・手順のプロンプト・エージェントの答え"),
         ("意味が違う（境界の外）", "ツールの「セッション」の寿命と再開（ツールと使う側が持つ。SAS は ID を記録するだけ）・Git の「コミット」・使う側の「目的のセッション」（目的・親子・承認・コミットを持つ）")]
CTXNO = [("「セッション」で文脈を2つに分ける（9519cffa）", "版16 で SAS が子セッションの状態を持たないことにしたので、セッションは誰が何で動いたかの1つのモデルになり、分ける根拠が無くなった"),
         ("描画を別の文脈にする", "1つの変更要求で2つの文脈を直すことになる。いまは描画が SAS の語をそのまま使う"),
         ("業務領域ごとに1つの文脈にする", "書く・検証・描画・検索がどれも同じ Concrete・インスタンスのモデルを読み、言葉のずれが無い。分けるとモデルを重ねて持つことになる"),
         ("事業領域全体を1つの文脈とみなす", "土台の決まりに反する。今回は語で境界を引き、一般の製品と使う側を外の文脈にした"),
         ("前の2つの文脈（検証の文脈・変更の確かめの文脈）", "前は「判定」の2つの意味で2つの文脈に分けていた。意図の判定を使う側へ出したので意味が1つに戻り、1つになった")]
INV = [("集約 Concrete（書くのモジュール）", "① 申告する Abstract の版は閉じた一覧にある（Abstract の enum）② x-published の欄を同じ版のまま変えない"),
       ("集約 Instance（書くのモジュール）", "① id を変えない ② x-decided-by が human の欄は人の入口からだけ変える（導く値は、Concrete の JSON Schema の if ・ then ・ else が形の確認で見る）"),
       ("組み込みの検査（検証のモジュールの業務サービスが判定）", "JSON Schema で書けない Concrete の決まり2つ：$schema の const と $id が同じ（instance-schema-name）・rule の id が重ならない（unique-id）。x-prompt の rule ではなく、Concrete の検証のたびに SAS が確かめる。x-prompt の rule はどれも打ったセッションが判定して --answer で渡す（判定できなければ unverified）。人だけが変える欄は書く前に断る。x- の閉じた一覧は語彙の propertyNames（JSON Schema）が見る"),
       ("書く手順の条件（書くのモジュールの use_cases）", "読んだ時のハッシュといまのハッシュが一致したときだけ書く・形の確認・機械の検証・ハッシュの一致がそろったときだけ書く・最後の操作の事後のハッシュ（値で受け取る）と違えば SAS を通らない変更として知らせる"),
       ("sessions の行（集約ではない）", "追記だけ・1つのセッションに1行・via は env ・ flag ・ init のどれか（DDL・trigger）"),
       ("記録の行（集約ではない）", "追記だけ・request_id は1回（DDL・trigger・一意の索引）")]
CMD = [("Concrete", "作る／版を変える（公開した版は変えない）"),
       ("インスタンス", "作る／直す（ハッシュを確かめる）／消す（ほかの文書は変えない。Concrete は従うインスタンスがあれば止める）／取り込む（--adopt）"),
       ("意味の手順（集約ではない。use_cases の手順）", "組んで返す（記録しない）／渡された値と判定を、形 ・ ハッシュ ・ step ・ rule ・ pointer で確かめる")]
VAL = "書くのモジュール：ハッシュ・差分（欄ごとの前後）・書く条件の外れ。検証のモジュール：判定の結果（判定の5つの区分と、何を verified に数えるか）・誰が判定したか（形の検査と組み込みの検査は machine、rule は ai）。sessions のツール ・ フォルダ ・ ID の出どころ（via）は記録の行の値。業務イベントは作らない（購読する手続きが無い）"
SVC = [("当たる rule・Abstract の rule（検証のモジュール）", "集約を読んで rule に照らす。書くのモジュールはサービスを持たない（参照を見分けない）")]
MNO = [("上からだけ（版13：events/・操作の記録の集約2つ・スキーマへの適合をアクティブレコード）", "購読者の無い業務イベントと、守る条件の無い集約が残る。x- の規則が作成・更新・削除・取り込みの4本のスクリプトに重複する"),
       ("下からだけ（すべてスクリプトで書いてから引き上げる）", "守る条件がすでに書けているスキーマへの適合と SAS が立てたセッションまで、いったんスクリプトに重複して書く"),
       ("業務領域ごとに技術方式を分ける（スクリプトはレイヤード）", "1つの文脈に依存の向きが2つでき、スクリプトから引き上げるたびに向きを付け替える"),
       ("domain を1つの crate にする", "1つの crate の中では集約の間の向きを止められない（Book ch07-03:296）"),
       ("イベント履歴式にする", "中身の情報源が Git と履歴の2つになり、SAS を通らない変更で食い違う")]
PORTS = [["document_store", "validation", "outbound/filesystem", "dyn。コンポジションルートと試験の差し替えを1つの書き方にそろえる。書くのは書くのモジュールの use_cases だけ"], ["schema_validator", "validation", "outbound/jsonschema", "dyn。同上。2020-12 の形の検査。x- の判定は validation-domain"], ["commit_reader", "writing", "outbound/git", "dyn。同上。コミットにある中身を読むだけ（ステップ3 で version_reader から改名）"], ["identify_caller", "recording", "inbound/cli", "dyn。同上。打ったセッションの ID を --session → 環境変数（callers.json）→ このフォルダで sas init が登録したセッションの順で決める。いちばん下に置き、どのモジュールからも使える"], ["operation_log", "recording", "outbound/sqlite", "dyn。同上。sessions（打ったセッション）と changes・validations の行を足す・読む"], ["operation_reader", "recording", "outbound/sqlite", "dyn。同上。記録にある操作を新しい順に絞って読む・1件の中身を読む（UC7。読むだけ。ステップ9 で足した）"], ["template_renderer", "rendering", "outbound/templates", "dyn。Mustache（mustache/spec の必須の部分）の処理系。値を置く・繰り返す・有無で出し分けるだけで、ロジックを持たない。描かずに木を読み、検証に使う"], ["script_reader", "rendering", "outbound/javascript", "dyn。JS の構文解析器 boa。テンプレートの JS が値の入口 sas-data から読む欄を取り出す（ステップ5 で足した）"], ["page_writer", "rendering", "outbound/filesystem", "dyn。描いたページを書き出す。入力と実体の経路で重なれば断り、1つでも置けなければどれも置かない（ステップ5 で足した）"]]
LIBS = [["rusqlite", "outbound/sqlite の crate だけが依存", "行の型で変換する（orphan rule）"], ["2020-12 のバリデーターのライブラリ", "outbound/jsonschema の crate だけが依存", "schema_validator を実装。$ref はネットワークから取らない。x- の判定は domain"], ["テンプレートの処理系のライブラリ", "outbound/templates の crate だけが依存", "template_renderer を実装。種類がそろうかは application が確かめる"]]
RULES_ANY = ["リポジトリの根は製品 Semantic Agent Schema の置き場で、文脈の根にしない。文脈の根は、文脈の名前のフォルダ（リポジトリの根の直下。ここでは schemas-and-instances/）。その中は業務領域を作るモジュール（writing・validation・recording・rendering・search）と adapters で、文脈の根に domain を置かない。名前はその文脈の言葉（スキーマ・インスタンス・書く・検証・記録）から付け、製品の名前を使わない。文脈の中はポートとアダプターで作り、外とのやりとりはすべて application の ports を adapters が実装する（呼び出し元を決めるポートは inbound/cli、ほかは outbound）。本ではポートは業務ロジック層が定義するが、記録と描画はドメインモデルを持たないので application に置く", "業務ロジックの置き方は、上から決めて下から確かめる。守る条件がすでに決定・Abstract の rule・試験の場面に書けている所は初めから domain に置き、それ以外は use_cases にトランザクションスクリプトで書く。引き上げの表で判別し、突き合わせで確かめたものだけを domain に置く", "domain の中は部品の種類でフォルダを分ける：aggregates/（集約ごと）・values/・services/。購読する手続きが無いうちは events/ を作らない", "application の中は use_cases/ と ports/。use_cases/ のファイルは集約か手順のまとまりごとに分け、ユースケースごとのファイルやフォルダは作らない（土台の境界6）", "domain を持つのは、突き合わせで集約・値・サービスが確かめられたモジュールだけ（いまは書く・検証）。集約は書くのモジュール、検証のモジュールは集約を読んで rule に照らすサービスと判定の値を持つ。記録・描画・検索は application だけ", "ポートは、それを使う手順を持つモジュールの application/ports に置く。共有の置き場を作らない。書く・検証・検索が共に使うポートは、書く・検索が依存する検証のモジュールが持つ。呼び出し元を判別するポートは、いちばん下の記録が持つ", "モジュールの間の向きは 書く → 検証 → 書くの domain、書く・検証・描画・検索 → 記録、描画・検索 → 検証 の一方向。同じ文脈の別のモジュールはポートを挟まず application を直接呼ぶ", "外のライブラリ（rusqlite・2020-12 のバリデーター・テンプレートの処理系）は、それを使う outbound の crate だけが依存する", "コンポジションルート（アダプターをポートへ渡す）は入口の1か所だけ", "公開された言葉（Abstract の本体・語彙・方言と CLI の入出力）は文脈の根の published-language/&lt;版&gt;/ に置き、出した版は変えない"]
RULES_RS = ["依存の向きを強制したい境界と、外のライブラリを閉じ込めたい所だけを crate にし、それ以外はモジュールで分ける。crate の数の基準は原典に無い<span class=\"src\">（Book ch07-01:186・ch14-03:187（基準の記述が無い）・ch07-03:296・Reference visibility:476）</span>", "domain はモジュールごとに crate を分ける（いまは writing-domain と validation-domain。検証が集約を読む向きを crate で強制する）。1つの crate の中では子モジュールが祖先の項目を使えるので、集約の間の向きはモジュールでは止められない<span class=\"src\">（Book ch07-03:296・ch14-03:299）</span>", "根の Cargo.toml は virtual manifest。[workspace] に members と resolver = \"3\" を書く<span class=\"src\">（Cargo workspaces:241・258、Book ch14-03:207〜211）</span>", "外のライブラリの版は [workspace.dependencies] に置き、各 crate は workspace = true で継承する（optional にはできない）。SAS の中の crate どうしは path dependencies<span class=\"src\">（Cargo workspaces:359・363、specifying-dependencies:425）</span>", "crate が使えるのは自分の Cargo.toml に書いた crate だけ。これで domain の無依存と、ライブラリの閉じ込めを強制する<span class=\"src\">（Book ch14-03:299・390）</span>", "domain の lib.rs は pub mod aggregates; pub mod values; pub mod services;（集約の無い validation-domain は values と services だけ。events は作らない）。crate の中だけで使う部品は pub(crate)<span class=\"src\">（Book ch07-02:273、Reference visibility:390・476）</span>", "mod.rs を使わない。src/aggregates.rs と src/aggregates/concrete.rs・instance.rs の形にする<span class=\"src\">（Book ch07-05:251・265・268）</span>", "application の ports.rs が、ポートで使う domain の型を pub use で再公開する。adapters は application だけに依存する<span class=\"src\">（Book ch07-04:392、Reference visibility:564）</span>", "コンポジションルートは cli の main.rs。outbound の crate に依存してよいのは cli だけ<span class=\"src\">（Book ch14-03:299（推論：実装の型の名前を書くので依存が必要））</span>", "ポートは trait で、9つとも dyn で使う。dyn compatible に保つ：型引数のあるメソッド・async fn・関連定数を持たず、Self を receiver 以外で使わない。sealed にしない<span class=\"src\">（Reference traits:412・429・445・459、API Guidelines C-OBJECT:322・324・321、C-SEALED:169）</span>", "集約の欄は private、メソッドだけ pub。ID・ハッシュは newtype で区別する<span class=\"src\">（Book ch07-03:477、API Guidelines C-STRUCT-PRIVATE:218・C-NEWTYPE:169）</span>", "adapters で外の trait（rusqlite・serde など）を domain の型に実装しない（orphan rule）。sqlite の crate は自分の行の型を持って変換する<span class=\"src\">（Book ch10-02:302・309・313、Reference implementations:658・659・669）</span>", "依存のグラフは循環させない<span class=\"src\">（Cargo resolver:568・585）</span>"]
TREE = [["semantic-agent-schema/", "リポジトリ ＝ 製品 Semantic Agent Schema（文脈ではない）。直下に文脈の名前のフォルダと、製品全体のものだけを置く", ""], ["├── Cargo.toml", "［製品全体］virtual manifest：[workspace] members に下の crate のパス（ラベルの付いたフォルダ。いま15）・resolver = \"3\"・[workspace.dependencies]", ""], ["├── architecture.json", "［製品全体］crate の間で許す依存の向きと crate の名前の決め方（検査に使う）", ""], ["├── docs/", "［製品全体］設計の文書（土台・設計のページ）", ""], ["└── schemas-and-instances/", "文脈「スキーマとインスタンス」（区切られた文脈）。中の作り方はポートとアダプター。中は業務領域を作るモジュールと adapters", ""], ["    ├── published-language/v1/", "公開された言葉：Concrete を書く型（abstract ・ vocabulary ・ dialect）・ CLI の出力（command-result）・ 打ったセッションへの手順（procedure）・ 受け取りの型（design-system ・ template-language）の7つ。操作の依頼の型は JSON で依頼する口を作るときに足す", ""], ["    ├── writing/", "業務領域「書いて検証する」の書くモジュール", ""], ["    │   ├── domain/", "依存：無し。aggregates/（Concrete・インスタンスと書く条件）・values/（ハッシュ・差分・書く条件の外れ）", "writing-domain"], ["    │   └── application/", "use_cases/write.rs（作る・直す・取り込む）・write/delete.rs・write/patch.rs・ports.rs（commit_reader）", "writing-application"], ["    ├── validation/", "業務領域「書いて検証する」の検証のモジュール", ""], ["    │   ├── domain/", "依存：writing-domain。values/（判定の5つの区分・rule の結果・by）・services/（当たる rule・Abstract の rule）", "validation-domain"], ["    │   └── application/", "use_cases/validate.rs（機械で検証する・入力が変わった rule だけ）・meaning.rs（意味の判定の手順を組む・判定を突き合わせる）・ports.rs（document_store・schema_validator）", "validation-application"], ["    ├── recording/", "業務領域「記録」のモジュール（トランザクションスクリプト。domain は無い）", ""], ["    │   └── application/", "use_cases/history.rs（打ったセッションを決める・追記する・UC7 を読む）・ports.rs（identify_caller・operation_log・operation_reader）", "recording-application"], ["    ├── rendering/", "業務領域「描画」のモジュール（domain は無い）", ""], ["    │   └── application/", "use_cases/render.rs・design.rs・ports.rs（template_renderer・script_reader・page_writer）", "rendering-application"], ["    ├── search/", "業務領域「検索」（補完）のモジュール（domain は無い。記録しない）", ""], ["    │   └── application/", "use_cases/search.rs（範囲を読む・構造で絞る・意味で探す手順を作る・答えの出どころを確かめる）・search-answer.schema.json（答えの形。公開しない）", "search-application"], ["    ├── adapters/", "アダプターの置き場", ""], ["    │   ├── inbound/cli/", "[[bin]] の名前は sas（製品の名前はここにだけ残す）。main.rs がコンポジションルート。callers.json（セッションの ID の環境変数）。outbound に依存してよいのはここだけ", "inbound-cli"], ["    │   └── outbound/", "filesystem（document_store・page_writer）・git（commit_reader。読むだけ）・jsonschema（schema_validator）・sqlite（operation_log・operation_reader。migrations/）・templates（template_renderer。ステップ5）・javascript（script_reader。ステップ5）", "outbound-&lt;技術&gt;"], ["    ├── system-tests/", "システムテストの場面（scenarios/）・材料（fixtures/）・trace.json・untested.json", ""], ["    └── test-support/", "試験の部品。dev-dependencies からだけ", "test-support"]]
CRATES = [["writing-domain", "domain", "無し"], ["validation-domain", "domain", "writing-domain（集約を読む）"], ["recording-application", "application", "無し"], ["validation-application", "application", "writing-domain・validation-domain・recording-application"], ["writing-application", "application", "writing-domain・validation-domain・validation-application・recording-application"], ["rendering-application", "application", "writing-domain・validation-domain・validation-application・recording-application"], ["search-application", "application", "writing-domain・validation-application・recording-application（identify_caller）"], ["outbound-filesystem", "adapters", "validation-application（document_store）・ rendering-application（page_writer）"], ["outbound-git", "adapters", "writing-application（commit_reader。読むだけ）"], ["outbound-jsonschema", "adapters", "validation-application（schema_validator。2020-12 のバリデーターのライブラリはここだけ）"], ["outbound-sqlite", "adapters", "recording-application（operation_log・operation_reader。rusqlite はここだけ）"], ["outbound-templates", "adapters", "rendering-application（template_renderer。Mustache の処理系）"], ["outbound-javascript", "adapters", "rendering-application（script_reader。boa はここだけ）"], ["inbound-cli", "adapters（bin・コンポジションルート）", "application と outbound のすべて（main.rs でコンポジションルートを置く。outbound に依存してよいのはここだけ）"]]
tree_html = ('<div class="legend"><span class="crate">crate</span> Cargo.toml を持つフォルダ。依存の単位で、名前は右のラベルのとおり（いま15個）。ラベルの無いフォルダは、まとめるだけのフォルダか、crate の中のモジュール</div>' + '<div class="fig"><div class="tree">' + ''.join(f'<div class="tl"><code>{html.escape(p)}</code><span>' + (f'<span class="crate">crate</span><code class="cn">{html.escape(c)}</code> ' if c else '') + f'{html.escape(n)}</span></div>' for p, n, c in TREE) + '</div></div>')
SCHG = [("Concrete を書くための型", "SAS → Concrete を書く人と AI ・ バリデーター", [("abstract.schema.json", "Abstract の本体。Concrete に書ける x- の4つの値の形（$defs）と、根 ・ 欄 ・ rule の形。JSON Schema で書けることは JSON Schema で書き、書けない2つは SAS の組み込みの検査にする"), ("vocabulary.schema.json", "語彙。入れ子に書ける x- の3つの形と、x- の閉じた一覧（propertyNames）"), ("dialect.schema.json", "方言。2020-12 に SAS の語彙を足す（$dynamicAnchor: meta で入れ子に掛ける）")]),
 ("CLI の出力", "SAS → 呼び出し元", [("command-result.schema.json", "操作の結果。change ・ validation ・ rule-result ・ render ・ session（sas init）・ operation（sas log の一覧の1行）・ search ・ error（打ったセッションの ID を持つ）。sas log --operation は change ・ validation の形で返す")]),
 ("打ったセッションへの手順", "SAS → 打ったセッション", [("procedure.schema.json", "手順。SAS が Concrete の rule と当たる所（検索は範囲の文書）から組み、--semantic で結果の行の代わりに出す（記録しない）。operation_kind は validate ・ write ・ search。command に値を渡すコマンド、answer に値の形（format ・ schema）を持つ。schema は形の本体（書くときは書く元のスキーマ。$ref の先は $id を持ったまま $defs に入れる）")]),
 ("受け取りの型", "使う側 → SAS", [("design-system.schema.json", "デザインシステムの受け取りの型（x-view の種類ごとのテンプレート ・ トークン）"), ("template-language.json", "テンプレートの書き方の契約（Mustache）")]),
 ("判定と検索の答え（公開しない）", "打ったセッション → SAS", [("agent-answer.schema.json（validation/application）", "--answer で渡す判定（hash ・ results）。手順の answer.schema に中身が入る。書く値は --value（文書全体）か --patch（JSON Patch）"), ("search-answer.schema.json（search/application）", "sas search --answer で渡す答え（answer ・ sources の path ・ pointer ・ hash）。手順の answer.schema に中身が入る")]),
 ("作らないと決めたもの", "版21", [("glossary", "使うステップが無い"), ("command-request", "SAS の入力は CLI の引数で、JSON で依頼する口が無い。口を作るときに足す"), ("session-request", "SAS はセッションを立てない（版28）"), ("session ・ diff", "command-result の中（$defs）"), ("settings", "使う設定が無い")])]
XTERMS = [("x-prompt", "根と欄", "書き方の rule（write）と読み方（read：interpret ・ when）。read は打ったセッションへの手順に入る", ""),
 ("x-decided-by", "欄", "人が決める欄（human）。セッションの ID がある呼び出しの変更を断る", ""),
 ("x-published", "欄", "同じ版（$id）のあいだ変えない欄", "書き直した"),
 ("x-view", "根", "描画の種類の名前。デザインシステムのテンプレートの種類と合わせる", ""),
 ("x-relationship（版22）・ x-instance-path ・ x-instance-label ・ x-delegates ・ x-derived-from ・ Concrete の x-extensions", "―", "SAS が振る舞いを持たないか、JSON Schema で書けるので外した。x- を増やさず、意味の決まりは x-prompt の rule に書く", "外した")]
GONE2 = [("judgment.schema.json", "rule_results（DB）と command-result の rule-result"), ("change.schema.json", "changes・field_changes（DB）と diff"), ("change-summary.schema.json", "使う側（まとめ・承認・コミット）。SAS は読む形で材料を渡す"),
 ("call.schema.json", "役目は残す。手順と値を渡すコマンドは procedure.schema.json"), ("Abstract の output-judging・output-working", "agent-answer（validation/application。公開しない）"), ("x-derived-from", "Concrete の JSON Schema（if ・ then ・ else と const）"), ("x-delegates ・ rule の agent", "外した（版28 で SAS はエージェントを起動しない）"), ("x-instance-path ・ x-instance-label", "外した（検索の範囲は呼ぶ側が経路 ・ --concrete ・ --all で渡す）"), ("Concrete の x-extensions", "外した（x- を増やさない）"), ("vocab-meta.schema.json", "vocabulary（改名。x-view を足す）")]
MAPW = [("CLI の引数（--session ・ --intent ・ --request-id）", "sessions ・ changes"), ("環境変数（callers.json）・ sas init", "sessions（id ・ tool ・ folder ・ via）"),
 ("agent-answer の results（--answer）", "rule_results（by = ai）"), ("command-result の diff", "changes の前後のハッシュ・field_changes"), ("書いた ・ 検証した文書の $schema", "changes.schema ・ validations.schema（0005）")]
MAPR = [("sessions", "session（打ったセッションの ID）・ sas init の結果（id ・ tool ・ folder ・ via）"), ("ビュー operations（新しい順に絞る）＋sessions", "operation（sas log の一覧の1行）"), ("changes＋field_changes", "diff（change・operation の中）"),
 ("rule_results", "rule-result（reads_hash）"), ("changes の最後の after_hash", "validation の untracked・error"), ("changes＋field_changes ・ validations＋rule_results（1件）と schema", "change ・ validation（sas log --operation。schema は target の concrete）")]
FACT = [
 ("JSON Schema の出力の形", "json-schema.org Core 2020-12 の12章", "欄名はすべて在った。一覧に absoluteKeywordLocation と error（単数）を足した。出力の形は実装に SHOULD"),
 ("$vocabulary・$dynamicAnchor・$dynamicRef", "Core 2020-12・メタスキーマ", "方言に $dynamicAnchor: meta を置く。true の語彙を知らないバリデーターは方言を断る（MUST）"),
 ("JSON Patch", "RFC 6902", "op・path・value・from と op の6つが在った"),
 ("Design Tokens", "Format Module 2025.10", "版を固定。「群」を原典の group に直した。別名は {group.token}"),
 ("MCP の inputSchema", "MCP 2026-07-28", "根は type: \"object\"、外の $ref は解かれない。ツールごとに1つのスキーマへ束ねる"),
 ("コーディングエージェント5つ", "Claude Code・Codex CLI・Gemini CLI・Copilot CLI・Kiro CLI の公式文書", "セッションの ID を環境変数で取れるのは Claude Code だけ。caller.env を任意にし、無いときは --session か sas init（2026-11-01 に Codex・Copilot・Kiro で実測、Gemini・Cursor は原文）"),
 ("SQLite", "sqlite.org", "CHECK は NULL を違反にしない。NULL になりうる欄の = ・ IN を IS ・ IS NOT NULL で閉じた（実測でも確かめた）。最低の版 3.8.0・3.38.0、RAISE は文字列のリテラル"),
 ("Git の掃除", "git-scm.com（2.52.0 で最後に更新された頁）", "指されないオブジェクトは gc.pruneExpire（既定2週間）で消える。SAS は版を残さない（Git は読むだけ）ので、この条件は設計に関わらない")]
UNREAD = "Claude Code の sessions.md は 2026-11-01 に取得して照合した（同じセッションを2か所で再開すると記録が混ざる・--fork-session）。取得していない頁：Gemini の automation のチュートリアル、Codex の app-server、Copilot の SDK"
def flow(name, steps):
    ns = [{"id": f"s{i}", "label": f"{t}：{sc}", "role": ("focus" if tb else None)} for i, (t, sc, tb) in enumerate(steps)]
    for n in ns:
        if n["role"] is None: del n["role"]
    return run('figure', name, {"direction": "TB", "nodes": ns, "edges": [{"from": f"s{i}", "to": f"s{i+1}"} for i in range(len(steps) - 1)]})
F['fv'] = flow('fv', [("CLI の入力", "sas validate の引数（--semantic か --answer）", False), ("セッションを決める", "--session ・ 環境変数 ・ sas init → sessions", True), ("SAS を通らない変更", "changes の最後の after_hash と比べる", True),
  ("形の検査と組み込みの検査", "バリデーター → rule-result（by: machine）", False), ("--semantic", "判定の手順（procedure）を出して終わる。記録しない", False),
  ("--answer", "agent-answer の形 → ハッシュ → step ・ rule ・ pointer。違えば invalid_answer か conflict", False), ("記録", "validations・rule_results（1つのトランザクション）", True), ("CLI の最後の行", "command-result の validation（session）", False)])
F['fu'] = flow('fu', [("CLI の入力", "sas update --semantic --brief（--ref）", False), ("セッションを決める", "sessions", True), ("SAS を通らない変更", "違えば直さずに断る", True),
  ("--semantic", "procedure（write）を出して終わる。command は update --patch … --if-match。記録しない", False),
  ("--patch", "打ったセッションが JSON Patch で渡す。ステップの欄だけ", False), ("ハッシュを確かめて書く", "--if-match と違えば conflict。一時ファイルで置き換え、前後の中身から差分を求める", False), ("記録", "changes・field_changes（diff から）", True), ("CLI の最後の行", "command-result の change（session・hint）", False)])
F['overview'] = figman.overview(); F['layers'] = figman.layers(); F['hex'] = figman.hexf(); F['crates'] = figman.cratesf()
F['stages'] = figman.stages(); F['model'] = figman.model(); F['datamap'] = figman.datamap(); F['bnd'] = figman.bnd(); F['gone'] = figman.gone()
REPORT += [(n, next((x for x in r if '検査1〜4' in x), '')) for n, r in figman.REPORT]
def schg_html():
    out = '<div class="schg">'
    for g, ft, files in SCHG:
        out += f'<div class="sg"><b>{g}</b><span class="hint">{ft}</span>' + ''.join(f'<div class="sf"><code>{html.escape(f)}</code><span>{html.escape(d)}</span></div>' for f, d in files) + '</div>'
    return out + '</div>'
xt_html = '<div class="tw"><table><tr><th>x-</th><th>置く所</th><th>中身</th><th></th></tr>' + ''.join(f'<tr><td><code>{html.escape(a)}</code></td><td>{b}</td><td>{c}</td><td>{"<span class=\"tag t-new2\">"+d+"</span>" if d else ""}</td></tr>' for a, b, c, d in XTERMS) + '</table></div>'
TABLES5 = ["sessions", "changes", "field_changes", "validations", "rule_results"]
MAPG = [("CLI の引数", "W", "W", "", "", ""), ("procedure（出力）", "", "", "", "", ""), ("agent-answer", "", "", "", "", "W"), ("diff", "", "WR", "WR", "", ""),
        ("session（出力）", "R", "", "", "", ""), ("operation（出力・sas log）", "R", "R", "", "R", "R"), ("change（出力・sas log --operation）", "", "R", "R", "", ""), ("rule-result（出力）", "", "", "", "", "WR"), ("validation（出力）", "", "R", "", "R", "R")]
map_grid = (f'<div class="need" style="--n:5"><div></div>' + ''.join(f'<div class="nh">{t}</div>' for t in TABLES5)
  + ''.join(f'<div class="nr">{r}</div>' + ''.join(f'<div class="nc{" on" if v else ""}">{ {"W":"書く","R":"読む","WR":"書く・読む"}.get(v,"") }</div>' for v in vs) for r, *vs in MAPG) + '</div>')
T_NEW = {
 'domain': ('domain', '事業領域と文脈',
  '<p class="ans">事業領域（AI のエージェントが扱う構造化データを、スキーマに従って書き、検証すること）を分解して業務領域を見つけ、SAS 自身の4つを、意味の境界の内側の広い1つの文脈「スキーマとインスタンス」に置く。一般の4つと使う側は外の文脈。9519cffa で「セッション」を理由に2つに分けたが、版16 で1つに戻した。</p>'
  + fig(F['layers'], '事業領域・業務領域・区切られた文脈', '業務領域は見つけるもの、文脈は設計するもの。区分は仮で、確かめるときに実装の方法から逆に判定する。')
  + fig(F['ctxmap'], '文脈の地図', '中核を含む文脈「スキーマとインスタンス」は、ツールにはセッション ID の環境変数（callers.json）だけで従属し、エージェントを起動しない。Git と JSON Schema にも従属する。使う側へは共用サービスとして、公開された言葉（Abstract と CLI の出力）だけを出す。')
  + cards([("意味の境界", "ツールのセッションの寿命と再開・Git の「コミット」・使う側の「目的のセッション」は文脈「スキーマとインスタンス」の語と意味が違う"),
           ("ポートと製品", "一般の能力は、ポートと受け取りの型を SAS が持ち、製品はアダプターとして渡す"),
           ("切り出す契機", "描画が別のデザインシステムや別のチームを持ったら、下流の文脈へ")])
  + det('業務領域（区分・実装の方法・受け持つユースケース・手がかり）', tbl(['業務領域', '区分', '実装の方法', 'ユースケース', 'アクター／外部システム／データ'], SUB))
  + det('同じ言葉の一覧（と、境界の外で意味が違う語）', tbl(['範囲', '語'], WORDS))
  + det('外した案と理由', tbl(['外した案', '理由'], CTXNO))),
 'model': ('model', 'ドメインモデル',
  '<p class="ans">集約は2つ。Concrete・インスタンスで、書くのモジュール（writing/domain）に置く。検証のモジュールは集約を読んで rule に照らす。SAS が立てたセッションの集約は版16 でやめた（版28 で SAS はセッションを立てない）。上から決め、下から確かめて同じ形になったものだけを domain に置く。</p>'
  + fig(F['model'], '集約・値・サービス', 'writing-domain は何にも依存しない。validation-domain は集約を読む。意味（x-prompt の rule・x-prompt の欄を書く）は検証・書くの中にあり、業務領域にしない。記録・描画・検索はドメインモデルを持たない（業務ロジックは use_cases のトランザクションスクリプト）。点線の箱は use_cases に残すもの。')
  + fig(F['datamap'], 'データとドメインモデルの対応', 'ファイルの中身は Git。記録は集約にせず、追記する行のまま。')
  + cards([("2つの集約", "Concrete・インスタンス。1回に書く集約は1つ。sessions と記録は追記する行"),
           ("業務イベントは作らない", "購読する手続きが無い。現れたら足す"),
           ("SAS を通らない変更", "インスタンスの集約が判別する。最後のハッシュは use_cases が記録から読んで値で渡す")])
  + det('不変条件', tbl(['集約', '不変条件'], INV)) + det('コマンド', tbl(['集約', 'コマンド'], CMD))
  + det('値オブジェクトと業務サービス', '<p class="hint">値：' + VAL + '</p>' + tbl(['業務サービス', 'すること'], SVC))
  + det('外した案と理由', tbl(['外した案', '理由'], MNO))),
 'arch': ('arch', 'アーキテクチャ',
  '<p class="ans">文脈の中をポートとアダプターで作る。内側に domain と5つの application、縁にポート9、外にアダプター6つと入口の CLI。依存の向きは crate で強制する。</p>'
  + fig(F['hex'], 'ポートとアダプター', 'ポートは application が宣言し、アダプターが実装する（本では業務ロジック層が定義する。記録と描画はドメインモデルを持たないので application に置く）。外のライブラリはそれを使う outbound の crate だけが依存する。同じ文脈のモジュールどうしは application を直接呼ぶ。')
  + fig(F['crates'], 'crate の層', 'crate はいま15（domain 2・application 5・outbound 6・cli 1・test-support 1）。名前は &lt;モジュール&gt;-&lt;層&gt; と &lt;inbound|outbound&gt;-&lt;技術&gt;（文脈の名前は付けない）。outbound に依存してよいのはコンポジションルートを持つ cli だけ。')
  + tree_html
  + cards([("ポートとアダプター", "データアクセスはレイヤードの語で、ここには無い。外の I/O はスクリプトのモジュールもポートを通す"),
           ("crate とモジュール", "向きを強制したい境界（業務領域 × 層）とライブラリを閉じ込める所だけ crate。集約・値・サービス・ユースケースはモジュール"),
           ("ポートは dyn", "どれも dyn で使い、dyn compatible に保つ（型引数のあるメソッド・async fn・関連定数を持たない）")])
  + det('crate と依存してよい向き', tbl(['crate', '層', '依存してよい crate'], CRATES))
  + det('ポートの一覧', tbl(['ポート（会話の目的）', '持ち主のモジュール', 'アダプター', '受け渡すもの'], PORTS))
  + det('外のライブラリを閉じ込める所', tbl(['ライブラリ', 'どの crate が依存するか', '扱い'], LIBS))
  + det('言語に依存しない決まり', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in RULES_ANY) + '</ul>')
  + det('Rust の決まり（一次情報の頁と行）', '<ul class="plain">' + ''.join(f'<li>{x}</li>' for x in RULES_RS) + '</ul><p class="hint">原文は 2026-10-31 に取得して照合した（The Rust Programming Language・The Cargo Book・The Rust Reference・Rust API Guidelines。識別子33個・引用51件が原文と一致）。crate の数の基準は原文に無い。</p>')),
 'schema': ('schema', 'スキーマと DB',
  '<p class="ans">公開された言葉は published-language/v1 の7つ（Concrete を書く型3 ・ CLI の出力1 ・ 打ったセッションへの手順1 ・ 受け取りの型2）。JSON Schema で書けることは JSON Schema で書き、x- と SAS の rule は書けないことだけにする。判定と検索の答えの形（agent-answer ・ search-answer）は公開しない。DB は5表とビューで公開せず、first_seen のほかのどの欄も CLI の出力の形のどこかに出る。</p>'
  + '<h3>自分の $id を持つリソースにするか、$defs の部品にするか</h3><div class="tw"><table><tr><th>問い</th><th>はい</th><th>いいえ</th></tr><tr><td>1. その形を1つの文書として名指しする相手がいるか（$schema で名乗る ・ 別の文書の欄から URN で指す ・ 外の道具へ文書で渡す）</td><td>2へ</td><td>使うリソースの $defs に置く。複数で使うなら意味の持ち主の $defs に置き、ほかは $id#/$defs/名前 で指す</td></tr><tr><td>2. すでにあるリソースと、読み手も変わる理由も同じか</td><td>同じリソースに置く</td><td>自分の $id を持つリソース（ファイル）にする</td></tr></table></div><p class="hint">ファイルは運ぶ単位で、線はリソース（$id）にある（JSON Schema 2020-12 Core 8.2.4 ・ 9.3）。手順（procedure）は打ったセッションが行う指示で、結果を返す command-result と役目が違うので別のリソース。Concrete を書く人への作法は Abstract の rule：one-document-kind（1つの Concrete に1種類の文書）・ no-shape-only-concrete（形だけの Concrete を作らない）・ shape-in-defs（インスタンスの根にならない形は $defs）・ ref-not-copy（ほかの Concrete の形は写さず $ref）。</p>'
  + fig(F['bnd'], 'スキーマの境界', 'JSON で表したい・型で制御したい・意味を持たせたい・決まった型で返したい、のどれかに当たる境界だけをスキーマにする。DB の表と版の管理・バリデーター・テンプレートの処理系はプロセスの外へ JSON が出ないのでスキーマにしない。')
  + schg_html()
  + cards([("Concrete を書く型", "Concrete に書ける x- は4つ（x-prompt ・ x-decided-by ・ x-published ・ x-view）。x- を増やさず、意味の決まりは x-prompt の rule に書く。参照のパスが要る Concrete は、普通の欄（format: uri-reference と pattern）として書く"), ("答えの形", "判定の答え（agent-answer）は検証のモジュール、検索の答え（search-answer）は検索のモジュールが持ち、手順の answer.schema で渡す。値を渡すコマンドは手順の command"), ("公開しない", "DB の DDL。出力では DB の名前を直し、DDL を変えても出力のスキーマは変わらない")])
  + '<h3>Concrete を書くための x- 項目</h3>' + xt_html
  + '<h3>意味の部分を含む操作の流れ（どの段でどのスキーマと表を使うか）</h3><div class="two"><div>' + fig(F['fv'], '検証の流れ', '検証。濃い段は DB を書く・読む段。', 1.2) + '</div><div>' + fig(F['fu'], '更新の流れ', '更新。答えの各項目はステップの番号と JSON ポインタを持ち、SAS はステップの欄だけへ書く。', 1.2) + '</div></div>'
  + '<p class="hint">CLI の出力は1行1つの JSON（JSON Lines）。--semantic を付けた操作は、結果の行の代わりに手順（procedure）を出し、記録しない。手順を行うのは打ったセッションで、command のコマンドで値か判定を渡す。SAS はセッションを立てないので、閉じる処理は無い。</p>'
  + '<h3>スキーマと DB の対応</h3><div class="fig">' + map_grid + '</div><p class="hint">DB にだけある欄は sessions の first_seen だけ。スキーマにだけあるのは、記録しないと決めたもの（描画・検索の入力、当てる前の値、決めきれず返した点、検索の答え、ファイルが正本の設定）だけ。</p>'
  + fig(F['gone'], '外したものと引き継ぐ先', 'vocab-meta は vocabulary へ改名（x-view を足す。x-prompt はいまも在る）。SAS の版のタグが無いので未公開と見る。公開していたら v2 として作る。')
  + det('公開された言葉のファイルと、作らないと決めたもの', ''.join(tbl(['ファイル', '中身'], [[f'<code>{html.escape(f)}</code>', d] for f, d in files]) for g, ft, files in SCHG))
  + det('書く向きの対応', tbl(['スキーマ', 'DB'], MAPW)) + det('読む向きの対応（出力で直す名前）', tbl(['DB', '出力'], MAPR))
  + det('外したものの行き先', tbl(['外したもの', '行き先'], GONE2))
  + '<h3>原典と照合した結果（2026-10-31）</h3>' + tbl(['項目', '原典（版）', '結果と直した所'], [[f'<span class="tag t-chk">照合済み</span> {a}', b, c] for a, b, c in FACT]) + f'<p class="hint">原文は取得日と sha256 と一緒に残した。{UNREAD}。</p>'),
}
UC_FIX = tbl(['ユースケース', '直した保証・足した拡張'], [["UC3 削除する", "対象と、参照している所の記載が、同じ呼び出しの続きの更新（原因は削除）として消える"], ["UC6 検索する", "範囲（経路 ・ --concrete ・ --all）を必ず渡し、結果に範囲を返す。検索は記録しない。構造で絞るのは --where の一致だけ。意味で探すときは打ったセッションに手順を返し、--answer の出どころが範囲の中か ・ ハッシュ ・ 所を確かめる。拡張：範囲の外か所が無い → invalid_answer、ハッシュが違う → conflict、範囲の中身が 64KB を超える → range_too_large（切り詰めない）"], ["UC7 操作の記録を読む", "読むだけで、セッションの行も足さない。一覧は新しい順の要約（operation）、1件は書いたとき ・ 検証したときと同じ change ・ validation の形。拡張：無い ID → not_found"],
  ["UC1 作成する", "意味の作成は打ったセッションが行う。--semantic（--brief ・ --ref）で手順を返し、セッションが create --value で書く。最低保証は「理由が返る」（断った操作は記録しない）。手順の answer.schema は書く元のスキーマの本体で、形の決まりを書く前に読める。拡張：断られた → 理由を読んで直し、打ち直す"],
  ["UC2 更新する", "UC1 と同じ。手順の command は update --patch … --if-match <手順のハッシュ>。中身が変わっていれば conflict"],
  ["UC4 検証する", "変形「意味の判定」：--semantic で判定の手順を返し、打ったセッションが --answer で判定を渡す。拡張：答えの形か突き合わせが違う → invalid_answer、ハッシュが違う → conflict（手順の取り直しを案内）"]])


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
      ("意味の手順（版14 の SF1）", "SAS が立てたセッション（版16 でやめた）", "", "", "手順を組んで返す・答えを突き合わせる"),
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
  '<p class="ans">上から決めた構成を、ユースケースの手続きから下から確かめた（版14 の記録）。業務領域ごとの実装の方法と3つの集約は一致し、構成は妥当。食い違い6件は上からの設計を直した。版16 で SAS が立てたセッションの集約はやめたので、いまの集約は2つ。</p>'
  + fig(F['stages'], '探索の4つの段階と引き上げの表', '段階3でユースケースの手続きをそのまま書き、段階4で何になるかを判別する。似たものも、同じ意味・同じ理由で変わると確かめるまで引き上げない。')
  + '<h3>上から：ドメインモデルが自明な所と根拠</h3>' + tbl(['業務領域', '判定', '根拠（守る条件がすでに書けている所）'], TOPDOWN)
  + '<h3>下から：ユースケースの手続き → 何になるか</h3><div class="fig">' + bu_grid + '</div><p class="hint">緑のマスがそのユースケースの手続きで見つかったもの。手順とルールの全文は下に畳んだ。</p>'
  + '<h3>突き合わせ</h3>' + recon_grid
  + cards([("食い違いのとき", "部品の置き場所は下を、境界と公開する契約は上を採る。どちらにも実物が無ければスクリプトに残す"),
           ("自明なら上から", "守る条件がすでに決めたことや試験の場面に書けている所は、探りを入れず初めから domain に置く"),
           ("ドメインモデルは育てる", "スクリプトに残したものも、ほかのユースケースが同じ理由で使い始めたら引き上げる")])
  + det('段階3の手続きとルール（全文）', tbl(['ユースケース', '段階3：主な手順', '出てくるルール', '段階4：何になるか'], BOTTOM_FULL))
  + det('名前が同じで別のもの', tbl(['名前', '別々のもの'], [["検証", "形の検査（2020-12、一般）・機械の rule（x- の意味）・意味の rule（UC4 の変形「意味の判定」）"], ["ハッシュ", "対象のファイルのハッシュと、手順のプロンプトの prompt_hash"], ["セッション", "版15 までは SAS が立てた（状態を持つ集約）と呼び出し元（記録の行）の2つ。版16 で1つの意味（誰が何で動いたか）になり、版28 で打ったセッションだけになった"], ["版", "$id の v&lt;版&gt; と Git のコミット（ステップ3 で呼び分けた）"]])))

# ---------- 決めたこと・承認 ----------
DECIDED = [
 ("版29", "記録を読む ・ 検索 ・ 手順に形を入れる ・ 完了の確認（ステップ8〜10）", "sas log（UC7）は記録にある操作を新しい順に要約（operation）で返し、--operation で1件を書いたとき ・ 検証したときと同じ change ・ validation の形で返す（無い ID は not_found）。絞り方は経路 ・ --by ・ --folder ・ --since ・ --limit。読むだけで、changes ・ validations に schema の欄を足した（0005）。ポート operation_reader を足した。sas search（UC6）は範囲（経路 ・ --concrete ・ --all）を必ず渡し、--where で構造で絞るか、--semantic で範囲の文書ごとの手順を返す。--answer の出どころは範囲の中か ・ ハッシュ ・ 所を SAS が確かめる（U28）。範囲の中身が 64KB を超えたら range_too_large。検索は記録しない。検索は業務領域「検索」（補完）のモジュール（search-application。application だけ）。意味で書く手順の answer.schema に、書く元のスキーマの本体を入れた。公開された言葉7つと UC0〜UC7 のどれにも場面があることを試験で確かめる（場面128本。UC0 は S-UC0-USE の1本）"),
 ("版28", "SAS はエージェントを起動しない（b2ae6ad7）", "意味の操作は、打ったセッションに手順（procedure、command 欄つき）を返すだけにした。子セッション ・ 接続の定義 ・ outbound/agents ・ --agent ・ --continue を外し、sessions は id ・ tool ・ folder ・ via ・ first_seen にした。sas init は環境変数から ID が取れないツールのため"),
 ("版27", "ステップ7 意味で書く", "sas create ・ update --semantic --brief --ref --agent。Concrete の rule から SAS が書く手順（procedure の write）を組み、書き込み用の子セッションに欄を書かせ、答えの形と Concrete の形に照らしてから SAS が書く。人の欄は書かせない。書く順は rule の reads、読み合えば reads_cycle。--ref は SAS が読んで中身を渡す（64KiB まで）。判定は書いたあと sas validate --agent。その前にステップ1〜6 をいまのスキーマ定義と照らして点検し、61件を直した（$ref をたどる rule の当て方 ・ --agent と --stale ・ reads の実装など）"),
 ("版26", "スキーマの分け方の作法", "外に出すかの線はファイルではなくリソース（$id）。名指しする相手がいて、読み手か変わる理由が違うときだけ自分の $id を持つ。Concrete を書く人への作法は Abstract の rule 4つ（one-document-kind ・ no-shape-only-concrete ・ shape-in-defs ・ ref-not-copy）にした。中身の sha256 を指す語を、ふだん使う「ハッシュ」にそろえた"),
 ("版25", "operation-result を command-result に", "SAS のコマンドの出力の抽象の型なので、名前をコマンドにそろえた（記録の操作 operation と取り違えない）。入力は CLI の引数で、command-request は JSON で依頼する口を作るときに足す"),
 ("版24", "手順のプロンプトを独立したスキーマに", "手順は SAS が子セッションに渡す指示で、CLI の結果（operation-result）とは相手も役目も違うので、operation-result の $defs から published-language/v1/procedure.schema.json に分けた。公開された言葉は8つ。版21 で「手順は operation-result の中で足りる」とした判断を改めた"),
 ("版23", "rule の by をなくした", "x-prompt の rule は、書く AI への指示であり、検証では子セッションが判定するレビューの観点である。by: machine は SAS に実行する手段が無くいつも unsupported、by: human は unverified を返すだけで、どちらも空の約束だった。値だけで決まる決まりは JSON Schema、JSON Schema で書けない Concrete の決まり2つ（instance-schema-name ・ unique-id）は SAS の組み込みの検査、承認は x-decided-by の欄、子セッションが決められないときは unverified。人だけが変える欄の rule human-entry-only も外した"),
 ("版22", "スキーマの形の未決5つを決めた", "x-extensions を一覧ごと外し、各 x- の説明と rule を値の形（$defs）へ移した。x-view は根だけに書ける（深い所は $dynamicAnchor meta で断る）。x-decided-by ・ x-published は Concrete のどこに書いても守る。x-relationship を外した：Abstract に参照パスが要るかは Concrete ごとに違うので、Abstract の約束にしていたのは責務過多だった。参照のパスが要る Concrete は普通の欄（format: uri-reference と pattern）として書き、SAS は参照を見分けない。削除はほかの文書を変えず、参照元の書き換え ・ cause_id ・ 参照先の検査 ・ id_changed を外した。reads はそのインスタンスの中だけ。Concrete に書ける x- は4つ"),
 ("版21", "Abstract を導き直した", "Abstract は以前の基盤のスキーマの写し（rule 52件のうち51件が同じ）だったので、SAS の能力から Concrete を書く人への約束を導き直した。JSON Schema 2020-12 で書けることは JSON Schema で書き（$schema の enum ・ $id の pattern と not ・ 欄の required ・ x- の閉じた一覧の propertyNames ・ 導く値の if ・ then ・ else）、x- と SAS の rule は書けないことだけにした。Concrete に書ける x- は5つ（x-prompt ・ x-relationship ・ x-decided-by ・ x-published ・ x-view）で、x- を増やさない。x-delegates ・ x-instance-path ・ x-instance-label ・ x-derived-from ・ Concrete の x-extensions ・ output-judging ・ output-working を外した。SAS の機械の rule は4つ（$schema の const と $id ・ rule の id ・ 参照先 ・ 人だけが変える欄）で、どれも判定する。Abstract の $defs の中の rule も Concrete に当てる。x-published の欄があっても削除は止めない。公開された言葉は7つ（作らないと決めたもの：glossary ・ operation-request ・ procedure-prompt ・ session-request ・ session ・ diff ・ settings）"),
 ("版20", "検証用の子セッション（ステップ6）", "SF1 と UC4 の意味の rule（by: ai）を作った。sas validate --agent <接続の定義> で、rule × 当たる所ごとの手順のプロンプトを SAS が起動した検証用の子セッションに渡し、答えを step ・ rule ・ pointer で突き合わせて記録する。子セッションの起動 ・ 続け方 ・ 答えと子セッションの ID の取り出し方は接続の定義（agent-connector）が持ち、SAS はツールの名前で分岐しない。--continue で記録にある検証用の子セッションを同じ接続の定義で続ける。前の判定は渡さない。ポート keep_prompt は作らず、ポートは9つ"),
 ("版19", "描画（ステップ5）", "UC5 を作った。SAS が持つのは、JSON の値をテンプレートに埋め込んで描く契約と、その契約の検証だけ。テンプレートが表現するロジックと UI ・ UX はテンプレートの JS が持ち、SAS は持たない（テンプレートが増えても SAS の描画は変わらない）。テンプレートの書き方は Mustache（mustache/spec の必須の部分）を契約にし（template-language.json）、渡すのは value ・ json ・ tokens ・ css。sas validate でデザインシステムを契約に照らし、テンプレートが置く名前と JS が値の入口 sas-data から読む欄が Concrete にあるかを見る。外れたデザインシステムでは描かない。同じ入力からはバイト列まで同じページ（利用者の要件 2026-11-01）。ポート page_writer ・ script_reader を足した"),
 ("版18", "言葉を本の定義にそろえる", "『ドメイン駆動設計をはじめよう』の定義と照らして点検した（ddd-advisor と反証役。2026-11-01 承認：言葉の点検のページ 9C9nmWLvzKT6VZoG5UbPTr 版5）"),
 ("版18", "公開された言葉は1か所", "Abstract の本体（abstract.schema.json）・語彙・方言と CLI の出力は、どれも使う側が従い読む通信規約なので published-language/v1 にまとめた。URN も urn:semantic-agent-schema:published-language:… にした。子セッションとのやりとりは公開された言葉に入れない"),
 ("版18", "書くと検証で1つの業務領域", "同じ業務領域のユースケースは同じアクターが関与し密接に関係するデータを操作する（本15頁）。書くと検証はアクターもデータも同じなので、業務領域「書いて検証する」（中核）を writing と validation の2つのモジュールで作る"),
 ("版18", "リポジトリの根は製品", "事業領域は問題の側の言葉（本4頁）。リポジトリの根は製品 Semantic Agent Schema の置き場で、事業領域は「AI のエージェントが扱う構造化データを、スキーマに従って書き、検証すること」"),
 ("版18", "ポートとアダプター", "本の名前にそろえた（ヘキサゴナルは別名）。ポートは本では業務ロジック層が定義するが、記録と描画はドメインモデルを持たないので application に置く。ポートは adapters が実装する（呼び出し元を決めるポートは inbound/cli）"),
 ("版18", "語を本の訳語と1語1義に", "業務サービス（ドメインサービス）・従属（順応者）・判定の5つの区分（5つの値）・バリデーター（検査器。ポートは SchemaValidator）・コンポジションルート・同じ言葉・受け取りの型（アダプターの契約）。集約の外で守るものは不変条件と呼ばず、検証の rule と書く手順の条件に分けた"),
 ("版17", "文脈の中は業務領域を作るモジュール", "writing（書く）・validation（検証）・recording（記録）、ステップ5 で rendering（描画）、ステップ8 で search（検索）と adapters。文脈の根に domain を置かない（2026-11-01 名前の見直し。9C9nmWLvzKT6VZoG5UbPTr）"),
 ("版17", "集約は書くのモジュールに置く", "Concrete・インスタンスは作成・更新・削除のコマンドを受けて不変条件を守るので writing/domain。検証は集約を読んで rule に照らすサービスと判定の値を validation/domain に持つ。依存は 書く → 検証 → 書くの domain"),
 ("版17", "意味の操作は業務領域にしない", "意味（自然言語の rule に照らして中身の意味を書く・判定する・探すこと）は書く・検証・検索のそれぞれの中にある性質。AI の子セッションは手段なので outbound/agents に置き、ポートは read_meaning（検証のモジュール）"),
 ("版17", "スキーマの置き場所を2つに", "schemas/v1 を abstract/v1（Concrete が従う型）と published-language/v1（SAS の入出力の形）に分けた。「スキーマ」の指し先を1つにする"),
 ("版17", "名前の付け方", "業務領域の名前は実際に使う言葉から（検証 48・書く 23・記録 52 回。適合は 0 回）。crate は &lt;モジュール&gt;-&lt;層&gt;・&lt;inbound|outbound&gt;-&lt;技術&gt;。files は filesystem、test-suite は文脈の中の system-tests"),
 ("版16", "SAS は子セッションの状態を管理しない", "開閉・終わり方・回・再開の決まりを持たない。次にどの子セッションへ送るかは親セッションが決め、SAS は案内するだけ（2026-11-01 承認。承認のページ XQmeBmywzzsZy3AUgcSK4B 版4）"),
 ("版16", "薄い sessions（7列）", "id・role（caller・writer・checker）・parent_id・tool・model・launch・first_seen。追記だけ。launch は --continue で同じ設定を渡すため（Codex の exec resume は -s が無く -c で渡す）"),
 ("版16", "書き込み用の子セッションと検証用の子セッションをそれぞれ続けて使う", "1回目は起動して ID を返し、2回目からは親セッションが --continue で渡す。brief と参照は1回目だけ。検証用の子セッションには前の判定・書き込み用の子セッションの答え・brief を渡さない。実測で2回目以降の費用は約1/20（Claude Code）"),
 ("版16", "fork は使わない", "親セッションとモデルや設定が違えば親セッションの全履歴を書き直し、同じでも親セッションの履歴の分を毎回読む。非対話の fork は Claude Code と Codex だけ"),
 ("版16", "親セッションと子セッションの ID の決め方", "親：--session → 接続の定義の環境変数（Claude Code は CLAUDE_CODE_SESSION_ID）→ 人なら NULL。子：ツールの出力の session_id（Codex は thread_id、Copilot・Kiro は sessionId）"),
 ("版16", "記録は changes と validations に分ける", "operations 1表をやめ、書いた変更（＋field_changes）と検証（＋rule_results）にする。UC7 はビュー operations。断った操作は持たず、理由はその場で返す（最低保証は「理由が返る」）"),
 ("版16", "文脈は「スキーマとインスタンス」1つに戻す", "9519cffa で「セッション」を2つのモデルで持つことを理由に分けたが、セッションは1つの意味になった。launched-sessions の crate はやめた"),
 ("版16", "DB の置き場所", "<git-common-dir>/sas/sas.db（作業ツリーの外・ワークツリーで1つ・コミットしない）。作り直せるデータとして扱う"),
 ("版16", "ステップ4b を足す", "セッションを外して記録を分ける。ステップ5 の前に入れ、ステップ1〜4 の振る舞いは変えない"),
 ("変更", "業務領域は4つと一般（版18）", "書いて検証する（中核・ドメインモデル。モジュールは書く・検証の2つ）、記録・描画（補完・トランザクションスクリプト）、検索（ステップ8 で確かめる）。一般の4つは作らずに使う。意味の操作は業務領域にしない"),
 ("足した", "文脈は「スキーマとインスタンス」の1つ（SAS は事業領域で、文脈ではない）", "意味が違う語（ツールのセッション・コミット・目的のセッション）の手前に境界。一般と使う側は外の文脈"),
 ("変更", "集約は2つ（版16）", "Concrete・インスタンス。版14 の3つ目（SAS が立てたセッション）は版16 でやめた。sessions と記録は集約にせず追記する行。1回に書く集約は1つ"),
 ("足した", "ディレクトリ", "（旧）リポジトリの根に業務領域ごとのモジュールを置いていた。版15 で、リポジトリの根は事業領域、その直下に文脈の名前のフォルダ（schemas-and-instances/）、その中に業務領域のモジュールと adapters/ に直した"),
 ("変更", "スキーマは14、1ファイル1境界", "Concrete を書く型4・CLI の入出力2・意味の部分3・共通の値2・受け取りの型3。call と用語集は外さず、call は手順のプロンプトとセッションの依頼に分ける"),
 ("足した", "DB は5表、公開しない", "（版16）sessions・changes・field_changes・validations・rule_results とビュー operations。launch のほかのどの欄も出力のどこかに出る。出力では DB の名前を直す"),
 ("照合", "x- は10個（足すのは x-view だけ）", "x-prompt はいまの vocab-meta に在った。方言の $dynamicAnchor: meta で語彙が入れ子に掛かる（Core 2020-12 で照合）"),
 ("足した", "意味の部分を含む操作の CLI の出力", "JSON Lines。1行目が手順のプロンプト、最後の行が結果。手順を行うのは SAS が起動した子セッション"),
 ("足した", "書くのは SAS だけ", "エージェントは決めた形の JSON を返す。機械の検証とハッシュの確認を通ったときだけ SAS が書く"),
 ("足した", "一般は契約と製品に分ける", "能力の契約（ポートと SAS の型）は SAS の中、製品（コーディングエージェント・Git・バリデーター・テンプレートの処理系）はアダプターとして渡す"),
 ("変更", "差分は SAS の型で表す・Git は読むだけ", "SAS が書いた操作は書く前後の中身から、SAS を通らない変更は Git のコミット済みの中身を読んで、変わった欄を SAS の差分の型で出す。版を残す仕組み（refs/sas/ で指す・復元）は持たない"),
 ("変更", "crate と DDD の部品をディレクトリで表す", "（版17）業務領域を作るモジュール × 層ごとに crate（いま11）。domain は書く（aggregates・values・services）と検証（values・services）。events は作らない。application は use_cases と ports"),
 ("変更", "DB の役目", "（版16）コミット前の、誰が・なぜ・何を変えたかを持つ（外の変更の検出・上書きを止める・間に入った変更・同じ依頼を2回処理しない・変わった rule だけ判定し直す）。中身はコミットしない"),
 ("足した", "描画はデザインシステムを受け取る契約", "種類がそろうかを確かめてから描く。SAS は既定のデザインシステムを持たない"),
 ("足した", "必要十分", "ユースケースのステップ × 能力の格子で、余分も不足も無い"),
 ("変更", "機械だけの操作", "（版16）子セッションを起動しない操作は親セッションの ID だけを記録する（人なら NULL）"),
 ("変更", "検証は残す・検索は残さない", "検証には結果がある。検索は残す使い道が無い"),
 ("足した", "SAS を通らない変更を良しとしない", "描画したものはコミットしない。ファイルが SAS を通らず変わったら検証で知らせる"),
 ("変更", "削除は参照している所も消す", "参照元の書き換えも、同じ呼び出しの続きの更新（原因は削除）として残る"),
 ("変更", "PR・コミットは SAS が担わない", "すべてローカル PC で動く。何を PR に添えるかは使う側"),
 ("照合", "CHECK の NULL のすり抜けを閉じた", "SQLite の CHECK は式が NULL のとき違反にしない。NULL になりうる欄の比べを IS・IS NOT NULL で書き、断ることを確かめた"),
 ("照合", "エージェントの接続の定義を原文に合わせた", "親セッションの ID を環境変数で取れるのは Claude Code だけなので caller.env を任意にし、無いときは --session。閉じる・状態を読むコマンドはどのツールにも無く、版16 で持たないことにした"),
 ("照合", "Design Tokens と MCP の版を固定", "Design Tokens Format Module 2025.10、MCP 2026-07-28。「群」は原典の group に直した"),
 ("変更", "文脈の中をポートとアダプターで作る", "外の I/O はスクリプトのモジュールもポートを通す。データアクセスはレイヤードの語でここには無い。active-record・data-access の crate は無い（版13 のモジュールごとの技術方式を取り消した）"),
 ("照合", "Rust の一次情報に合わせた構成", "pub mod と pub(crate)、mod.rs を使わない、application が domain の型を再公開、コンポジションルートは cli の main.rs、resolver = \"3\" と workspace.dependencies、ポートは dyn（dyn compatible に保つ）、集約の欄は private、orphan rule のため sqlite は行の型で変換"),
 ("版2", "判定の結果はコミットしない", "OK かを決めるのは呼んだ人。CI は機械の検証だけ"),
 ("版2", "意味の作成・更新・検索も同じ手順", "手順のプロンプトを返し、SAS が起動した子セッションで行わせる"),
 ("版2", "人の判定は呼び出し元のセッション", "SAS は by: human の rule を「人が決める」と返すだけ"),
 ("版2", "DB はセッションと操作", "1つの親セッションに操作が複数。変更は意図と、変わった欄の前後を持つ"),
 ("版2", "CLI はセッション ID を返す", "（版16）親セッションと子セッションの ID。使う側はそれを自分の Concrete に記録する"),
]
DECIDED += [
 ("足した", "上から決めて下から確かめる", "自明な所は上から domain に置き、ユースケースの手続きを段階3で書いて段階4の表で判別し、突き合わせて確かめたものだけを domain に置く。一致：実装の方法と3つの集約（版14 の時点。版16 で2つ）。食い違い6件は上を直した"),
 ("足した", "ポートは8つ（版16）", "（版17）read_meaning・keep_prompt・document_store・schema_validator（検証）、commit_reader（書く）、identify_caller・operation_log（記録）、template_renderer（描画）。agent_session_store は外し、agent_runtime は read_meaning に改めた"),
 ("変更", "domain の crate", "（版17）writing-domain（集約）と validation-domain（判定の値・サービス）の2つ。文脈の根に domain を置かない"),
]
DECIDED += [
 ("変更", "SAS は事業領域で、文脈ではない", "リポジトリの根 semantic-agent-schema/ は事業領域の置き場。直下に文脈の名前のフォルダ（schemas-and-instances/ の1つ）と、Cargo.toml・architecture.json・docs/ など事業領域全体のものだけを置く。旧版の「文脈の根＝リポジトリの根」はやめた"),
 ("変更", "文脈の名前は「スキーマとインスタンス」", "9519cffa で「契約とインスタンス」から改めた。「契約」は手続きに出ず指し先が2つあったため。業務領域「契約への適合」も「スキーマへの適合」にした"),
 ("変更", "業務領域の名前を活動の名前に", "（版17）書く（writing）・検証（validation）・記録（recording。旧名：操作の記録・操作の経緯）・描画（rendering）・検索（search）。旧名のスキーマへの適合（conformance）は書くと検証に分けた"),
 ("変更", "crate の名前", "（版17）&lt;モジュール&gt;-&lt;層&gt;・&lt;inbound|outbound&gt;-&lt;技術&gt;・test-support。文脈は1つなので文脈の名前を付けない。製品の名前は CLI のコマンド名（[[bin]]）にだけ残す"),
]
APPROVE = [
 ("土台の文の案（ディレクトリと文脈・中の形）", "見解：「文脈の根は、文脈の名前のフォルダ（リポジトリの根の直下）」と「中の形は文脈ごとに中をポートとアダプターで作る、上から決めて下から確かめる」に直す。土台は正本なので ACDR 0217 で差分を見せてから直す（0216 を置き換える）"),
 ("ガイドの確かめる段階（ボード Q5）と Q3 に手順を足す", "見解：Q5 に上からと下からの突き合わせの5つの手順、Q3 に「同じ物を2つの目的で別のモデルにしているか」の問いを足す。ボードは次の回で直す"),
]
TAGC = {"足した": "t-new", "変更": "t-chg", "照合": "t-chk", "版2": "t-old", "版16": "t-new2", "版17": "t-new2", "版18": "t-new2", "版19": "t-new2", "版20": "t-new2", "版21": "t-new2", "版22": "t-new2", "版23": "t-new2", "版24": "t-new2", "版25": "t-new2", "版26": "t-new2", "版27": "t-new2", "版28": "t-new2", "版29": "t-new2"}
def tagged(items):
    return '<div class="dec">' + ''.join(f'<div class="dc {TAGC[t]}"><span class="tag">{"版2 のまま" if t == "版2" else t}</span><b>{h}</b><span>{d}</span></div>' for t, h, d in items) + '</div>'

# ---------- 能力のマトリクス（HTML の格子） ----------
CAP = [("作成", "人やツールが値を書く", "x-prompt を指示に値を書く", "残る（create）"),
       ("更新", "人やツールが書き換える", "x-prompt を指示に書き換える", "残る（update）"),
       ("削除", "対象だけを消す（ほかの文書は変えない）", None, "残る（delete）"),
       ("検証", "JSON Schema と組み込みの検査2つ", "x-prompt の rule を判定する", "残る（validate と rule ごとの結果）"),
       ("描画", "テンプレートとトークンを注入し、その場で描く", None, "残らない（コミットしない）"),
       ("検索", "構造で絞る", "意味で探す", "残らない（打ったセッションの行は sessions に残る）")]
cap_grid = '<div class="mx"><div class="mh"></div><div class="mh">機械で</div><div class="mh">意味で（エージェント）</div><div class="mh">操作として</div>' + ''.join(
    f'<div class="mr">{a}</div><div class="mc m-mach">{b}</div><div class="mc {"m-sem" if c else "m-none"}">{c or "―"}</div><div class="mc {"m-keep" if d.startswith("残る") else "m-none"}">{d}</div>' for a, b, c, d in CAP) + '</div>'

# 子セッションの形（版16）
q11 = ('<div class="q11"><div></div><div class="mh">書き込み用の子セッション（writer）</div><div class="mh">検証用の子セッション（checker）</div>'
       '<div class="mr">1回目</div><div class="mc m-sem">SAS が起動して ID を返す。<b>--brief（必須）と --ref</b> を渡す</div><div class="mc m-sem">SAS が起動して ID を返す。rule と参照を渡す（brief は渡さない）</div>'
       '<div class="mr">2回目から</div><div class="mc m-keep">親セッションが <b>--continue</b> で渡す。差分の指示だけを送る</div><div class="mc m-keep">親セッションが <b>--continue</b> で渡す。いまの中身だけを送り、前の判定は渡さない</div></div>'
       '<p class="hint">SAS は子セッションの状態（開閉・終わり方・回）を持たない。続けるか新しくするかは親セッションが決める。子セッションの役割は1つで変わらない。</p>')
OPS = ('<div class="mx" style="grid-template-columns:repeat(auto-fit,minmax(150px,1fr))">' + ''.join(f'<div class="mc m-sem"><b>{a}</b><br>{b}</div>' for a, b in [
       ("親セッションを決める", "--session → 環境変数 → 人"), ("起動する", "ID をツールの出力から取る"), ("続ける", "--continue と記録した起動の設定"), ("案内する", "child_exists・busy・新しい子セッションの目安")]) + '</div>')

MAP27 = [("残す", "U12 人が読む形で読んで判断する", "UC5 描画する"),
 ("直す", "U0 成果物を確かめた状態で使い続ける", "UC0"), ("直す", "U1 Concrete を作る・U5 インスタンスを作る", "UC1"),
 ("直す", "U2 Concrete を変える・U6 インスタンスを変える", "UC2"), ("直す", "U3 Abstract の新しい版に合わせる", "UC2 の拡張"),
 ("直す", "U8 変更のたびにすべてを検証する", "UC4（主アクター CI、全件）"), ("直す", "U10 効かなくなった判定を取り直す", "UC4（入力が変わった rule だけ）"),
 ("直す", "S1 検証する", "UC4 へ上げる"), ("直す", "U11 置き換えて残す", "UC3 と UC2"), ("直す", "U13 経緯をあとから確かめる", "UC7 操作の記録を読む"),
 ("直す", "U16 人が決める欄を承認する", "UC4 の拡張（人が決める → 呼び出し元のセッション）"), ("直す", "U17 既存のファイルを取り込む", "UC1 の変形"),
 ("直す", "U26 探す範囲を宣言する", "UC6 の事前条件"), ("直す", "U28 出どころ付きの答えを返す", "UC6 の意味で探す"),
 ("直す", "U23 拡張項目", "UC1・UC2・UC4 の中身"), ("外す", "U24 x-relationship で読む", "版22 で参照を外した。参照は Concrete の普通の欄"), ("直す", "S2 変更を記録する", "すべての UC の「操作を残す」ステップ"),
 ("直す", "S4 セマンティックの手順を返す", "UC1・UC2・UC4・UC6 の意味の部分（手順を返す）"),
 ("外す", "U4 Abstract の新しい版を出す", "SAS を作る側の作業"), ("外す", "U14 validator をつなぐ", "UC4 の事前条件「アダプターが設定されている」"),
 ("外す", "U18 ほかの変更を取り込む", "取り込みは Git、取り込んだ後は UC4"), ("外す", "U25 変更を確かめる・S3 変更のまとめを作る", "使う側"),
 ("外す", "U27 目的のセッション・N3 子セッションに任せる", "使う側のオーケストレーター")]


# ---------- 必要十分：ユースケースのステップ × 能力 ----------
NCOL = ["セッションを決める", "範囲で絞る", "手順を組む", "答えを突き合わせる", "コミットの中身を読む", "差分を求める", "検査する", "デザインシステム", "記録を読む"]
NROW = [("UC1〜UC6 のはじめ", ["セッションを決める"]),
        ("UC1 作成する（機械）", ["検査する"]),
        ("UC1 作成する（意味）", ["手順を組む", "検査する"]),
        ("UC2 更新する", ["差分を求める", "検査する"]),
        ("UC2 の意味（手順を返し、JSON Patch で受け取る）", ["手順を組む", "差分を求める", "検査する"]),
        ("UC2 取り込み・UC4 通らない変更", ["コミットの中身を読む", "差分を求める"]),
        ("UC3 削除する", ["検査する"]),
        ("UC4 検証する", ["検査する"]),
        ("UC4 の変形：意味の判定", ["手順を組む", "答えを突き合わせる"]),
        ("UC5 描画する", ["デザインシステム"]),
        ("UC6 検索する（構造で絞る）", ["範囲で絞る"]),
        ("UC6 の意味（手順を返し、答えの出どころを確かめる）", ["範囲で絞る", "手順を組む", "答えを突き合わせる", "検査する"]),
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
  + cards([("SAS の中", "能力6つ・記録・公開する形・意味としての目的・一般の能力のポートと受け取りの型"), ("使う側", "目的のセッション・承認・まとめ・コミット・PR"), ("一般の製品", "Git・バリデーター・テンプレートの処理系はアダプターとして渡す。コーディングエージェントは sas を打つ側で、SAS は起動しない")])
  + det('目的の2つの読み', tbl(['目的の読み', '置き場所'], [["意味の一部（rule・欄・Concrete が何のためにあるか、1件の操作の意図）", "SAS の中：rule.why・reading.when・根の description・changes.intent"], ["進め方（目的ごとのセッション・親子・承認・コミット）", "使う側"]]))))
T.append(('types', '型と能力',
  '<p class="ans">Abstract は SAS が決め、Concrete とインスタンスは使う側が決める。6つの能力はどの段にも働き、意味が関わるのは作成・更新・検証・検索。</p>'
  + fig(F['types'], '3段の型') + cap_grid
  + '<p class="hint">緑＝意味の側がある・灰＝機械だけ。「操作として」は手元の DB に残るか。</p>'
  + cards([("意味は4つ", "作成・更新・検証・検索。どれも SAS が打ったセッションに手順を返し、セッションが値か判定を渡す"), ("削除", "参照している所の記載も同じ呼び出しの続きで消す"), ("描画", "その場で描き、コミットしない")])))

TOOLS = ["Claude Code", "Codex CLI", "Copilot CLI", "Kiro CLI", "Gemini CLI", "Cursor CLI"]
TOOLCAP = [
 ("非対話で1回送る", ["claude -p --output-format json", "codex exec --json", "copilot -p --output-format json", "kiro-cli chat --no-interactive", "gemini -p --output-format json", "cursor-agent -p --output-format json"]),
 ("子セッションの ID の取り方", ["出力の session_id", "thread.started の thread_id", "result の sessionId（--session-id で先に決めても良い）", "stream-json の sessionId", "JSON の session_id", "JSON の session_id"]),
 ("ID を指定して続ける", ["--resume &lt;ID&gt;", "codex exec resume &lt;ID&gt;", "--resume=&lt;ID&gt;", "--resume-id &lt;ID&gt;", "-r &lt;ID&gt;", "--resume &lt;ID&gt;"]),
 ("同じ子セッションへ同時に送ると", ["記録が混ざる（原典）", "2つ目が失敗（実測）", "記録が混ざる（実測）", "2つ目が失敗（原典・実測）", None, None]),
 ("続けたときの費用（2回目以降 ÷ 毎回起動、実測）", ["約5%", "約10%（キャッシュに当たらなかった入力）", "約90%", "約66%（credit）", None, None]),
 ("非対話で fork", ["--fork-session", "codex exec fork", None, None, None, None]),
 ("親セッションの ID の環境変数", ["CLAUDE_CODE_SESSION_ID", None, None, None, None, None]),
]
def toolcell(row, v):
    if v: return f'<div class="mc m-sem"><code>{v}</code></div>' if not row.startswith(("同じ子セッション", "続けた")) else f'<div class="mc m-sem">{v}</div>'
    if row == "非対話で fork": return '<div class="mc m-none">無い（使わないので必要ない）</div>'
    if row == "親セッションの ID の環境変数": return '<div class="mc m-none">無い → 親セッションが --session で渡す</div>'
    return '<div class="mc m-none">原文に無い・実測していない</div>'
tool_grid = ('<div class="tw"><div class="tg" style="grid-template-columns:minmax(120px,1.2fr) repeat(6,minmax(110px,1fr))"><div class="mh"></div>' + ''.join(f'<div class="mh">{t}</div>' for t in TOOLS)
  + ''.join(f'<div class="mr">{r}</div>' + ''.join(toolcell(r, v) for v in vals) for r, vals in TOOLCAP) + '</div></div>'
  + '<p class="hint">各社の公式文書と照合し、手元にある4つは実測した（2026-11-01、ツール6つの調査 KbNCDxxdVA5CipaYG8aFJ5）。Gemini と Cursor は原文だけ。オプションの名前は接続の定義のファイルに書き、SAS の中には書かない。同じ子セッションへ同時に送らない決まりは SAS の側で持つ（子セッションごとのロック）。</p>')
COST = tbl(['形（Claude Code・sonnet・参照 98KB）', '1回目', '2回目以降'], [["毎回新しく起動", "$0.198", "$0.198（参照を毎回読み直す）"], ["同じ子セッションを --continue で続ける", "$0.196", "約 $0.009（キャッシュから読む）"], ["親セッションの会話を複製（fork）", "親セッションの分", "約 $0.006。親セッションとモデルが違うと全履歴を書き直す"]])

OLD_SESSION = (
  '<p class="ans">SAS は子セッションの状態を管理しない。親セッションの ID を決め、意味の部分を含む操作では書き込み用の子セッション・検証用の子セッションを起動して ID を返し、親セッションが --continue で渡したら記録した設定のままその子セッションへ続けて送る。</p>'
  + OPS + q11
  + fig(F['ids'], 'ID の流れ', '親セッションの ID は --session か環境変数から、子セッションの ID はツールの JSON 出力の session_id から取る。どちらも sessions に1行ずつ、追記だけで書く（本物の ID で動かした：シミュレーション VYeEarjweXihDbJTNxnGhW）。')
  + fig(F['child'], '子セッションを決める', '上から当てる。続けるか新しくするかを決めるのは親セッションで、SAS は止めて案内するだけ。')
  + fig(F['unsup'], 'ツールに続ける機能が無いとき', '黙って新しく起動しない。記録と実際がずれるため。')
  + '<h3>照合したツールごとの能力</h3>' + tool_grid
  + '<h3>費用（実測）</h3>' + COST
  + fig(F['tree'], '親セッションと子セッションと記録', 'シミュレーションで DB に入ったもの。親セッションが子セッションを持ち、子セッションが変更か検証を持つ。')
  + fig(F['fail'], '失敗したときの扱い', 'どの子セッションでも書くのは SAS だけ。エージェントは決めた形の JSON を返す。同じ request_id の変更は1回だけ残る。')
  + cards([("アダプター", "コーディングエージェントごとのアダプターか、接続の定義（agent-connector）で動く汎用のアダプター"), ("検証用の子セッションを分ける理由", "作った人が確かめるのは独立性が無い（qa-advisor）。前の判定に引っぱられないかは、順を変えた判定と照らし合わせる場面で確かめる"), ("新しい子セッションの目安", "前の操作から接続の定義の cache_ttl_s（Claude Code 1時間・Copilot 30分）を過ぎたか、同じ子セッションで10回を超えたら案内する")])
  + det('外した案と理由', tbl(['外した案', '理由'], [["SAS がセッションの4つの型・再開の決まり・終わり方を持つ（版15）", "寿命と再開はツールが持つ一般の仕組みで、SAS が持つと要件を複雑にする。続けるかは親セッションが決めれば足りる"], ["親セッションの会話を複製（fork）して行う", "親セッションとモデルや設定が違えば全履歴を書き直し、同じでも親セッションの履歴の分を毎回読む。非対話で使えるのは Claude Code と Codex だけ"], ["親セッションそのものへ送る", "記録が混ざる（sessions.md:217）。動いているバックグラウンドのセッションには送れない"], ["毎回新しく起動して brief と参照で引き継ぐ", "参照を毎回読み直す。2回目以降の費用は続けると約1/20"], ["検証を書き込み用の子セッションで行う", "独立性が無い"], ["sessions を持たず、子セッションの ID を記録の行に値で持つ", "役割・親セッション・ツール・モデル・起動の設定は子セッションの性質。--continue で同じ設定を渡すには1か所に持つ必要がある"]])))
SIM_URL = 'https://claude.ai/artifact/PkMt9GJLuAyX1zLAeMivm9'
SESS_CMDS = tbl(['誰が', 'すること', '何で'], [
  ['SAS', '打ったセッションに、意味で書く手順（x-prompt と rule）を返す', '<code>sas create ・ update --semantic</code>'],
  ['SAS', '検査して書く。断ったら理由を返す', '<code>sas create --value ・ update --patch</code>'],
  ['SAS', '意味の rule の判定の手順を返す ／ 判定を受け取って記録する', '<code>sas validate --semantic ・ --answer</code>'],
  ['SAS', '最初に打ったときに sessions に1行足し、以後の操作の行から指す', 'どのコマンドでも（環境変数か --session）'],
  ['SAS', 'いまのセッションをこのフォルダに登録する（環境変数から ID が取れないツールのため）', '<code>sas init</code>'],
  ['使う側', '目的を分けるとき、フォルダ（ワークツリーなど）を用意してセッションを1つ立てる', '<code>git worktree add</code> と <code>claude -p</code> など'],
  ['使う側', '同じ目的の続きを頼む（同じセッションを再開する）', '<code>claude -p --resume</code> など'],
  ['使う側', '様子を見る ・ 画面で開く', 'ツールの出力 ・ <code>sas log --folder ・ --by</code>（記録を読むだけ）・ <code>claude --resume</code>']])
SESS_ROW = tbl(['sessions の列', '中身'], [
  ['id', 'セッション ID（環境変数か --session）'], ['tool', 'どのツールか（どの環境変数から拾ったか）'],
  ['folder', 'そのセッションが動いているフォルダ（ワークツリーを切っていれば、その根）'], ['first_seen', '初めて記録した時刻']])
SESS_FIX = tbl(['承認のあとに直す所', '中身'], [
  ['ユースケース', 'SF1（子セッションで手順を行わせる）を外し、UC1・UC2・UC4・UC6 の意味の部分を「打ったセッションに手順を返す」にする'],
  ['ドメインモデル ・ データ', 'sessions から parent ・ role（writer と checker）・ launch を外し、tool ・ folder を持たせる'],
  ['使われ方 ・ 使う側とのつなぎ方', '場面と JSON の例から子セッションを外し、打ったセッションの ID だけにする'],
  ['実装', 'エージェントを起動する仕組みをすべて外す：接続の定義（agent-connector）・ outbound/agents ・ 子セッションの鍵 ・ 中継と直させる（converse）・ writer-answer.schema.json ・ --continue ・ validate --agent']])
T.append(('session', 'セッション',
  '<p class="ans">SAS はエージェントを起動しない。打ったセッションの ID を拾って sessions に1行で記録し、操作の行から指すだけ。最初に sas を打ったときに、そのフォルダでの記録が始まる（環境変数から ID が取れないツールは <code>sas init</code> で登録する）。セッションは1つのフォルダに1つで、ふだんはいま動いているセッション1つで進める。セッションを立てる ・ 再開する ・ 様子を見るのは使う側。</p>'
  + cards([("いま動いているセッション", "ふだんはこれ1つ（いまはこのチャット）。SAS は環境変数の ID で記録するだけ"),
           ("ほかのフォルダのセッション", "目的を分けるとき、使う側がフォルダ（ワークツリーなど）ごとに1つ立てる。画面は無くプロセスで動き、そのフォルダで --resume すれば画面で開ける"),
           ("sessions の行", "1つのセッションに1行。親子 ・ 役 ・ 起動の設定は持たない"),
           ("sas init", "いまのセッションを、このフォルダの記録の相手として登録する。環境変数から ID が取れないツール（Gemini ・ Cursor）では ID を決めてフォルダに残し、以後の sas はそれを使う")])
  + '<h3>誰が何をするか</h3>' + SESS_CMDS
  + '<h3>sessions の行</h3>' + SESS_ROW
  + f'<p class="hint">動かして確かめる：<a href="{SIM_URL}">目的のセッションのシミュレーション</a>（立てたセッション2つ ・ sas コマンドが増やしたセッション0）。</p>'
  + '<h3>承認のあとに直す所</h3>' + SESS_FIX
  + det('外した形と理由', tbl(['外した形', '理由'], [
      ["意味の操作のたびに、書き込み用 ・ 検証用の子セッションを <code>claude -p</code> で起動する（版16〜27）", "セッションは目的ごとに1つでよい。操作ごとに起動すると、画面の無いプロセスが増え、何が起きているか誰にも見えない。ツールを持たない子セッションが、ツールの呼び出しを文字で763回くり返して10分で時間切れになった（実測 2026-11-01）"],
      ["SAS が子セッションの答えの JSON を受け取り、形が違えば直させる中継をする", "セッションが sas の出力を読み、自分で直して打ち直せば足りる"],
      ["子セッションにツールを渡さない", "判定の根拠を渡したものだけにするための仮の置き方だった。セッションは親と同じ条件で動き、書き込みは必ず sas を通る"],
      ["SAS が sas init でワークツリーとセッションを立て、send ・ watch で再開する ・ 見る（版28 の途中の案）", "目的のセッションは使う側の概念と決めてある（SAS がしないこと）。ワークツリーとセッションの寿命はツールと Git が持つ一般の仕組みで、SAS が包む理由が無い"],
      ["sessions の表を外し、操作の行に ID だけを持たせる", "どのセッションが、どのツールで、どのフォルダで動いたかを1か所に持てなくなる。DB はワークツリーをまたいで1つなので、見分ける行が要る"]]))
  + det('版27 までのセッションのタブ', OLD_SESSION)))
T.append(('data', 'データ',
  '<p class="ans">手元の DB は5表とビュー。誰が何で動いたか（sessions）と、書いた変更（changes）・検証（validations）。中身はコミットせず、作り直せるデータとして扱う。</p>'
  + fig(F['er'], 'ER 図')
  + cards([("1 対 多", "sessions → changes・validations、changes → field_changes、validations → rule_results"), ("中身は Git", "パスとハッシュで指すだけ。Git は読むだけで、版は残さない"), ("空になる列が無い", "変更と検証を別の表にしたので、どの行もすべての欄が埋まる（作成の事前・削除の事後・原因を除く）。断った操作は持たない"), ("置き場所", "&lt;git-common-dir&gt;/sas/sas.db。作業ツリーの外で、ワークツリーをいくつ作っても1つ"), ("schema の欄（0005）", "changes ・ validations に、書いた ・ 検証した文書が従う $id。sas log --operation で書いたときと同じ形に戻すため。0005 より前の行は空"), ("記録を読む", "sas log がビュー operations を新しい順に絞って読む（UC7）。読むだけで、何も足さない")])))
T.append(('flow', '意味の部分を含む操作の流れ',
  '<p class="ans">セッションが sas に手順をもらい、自分で値を作って書かせる。断られたら理由を読んで直す。立てた側が受け取るのは最後の答えだけ。SAS はセッションを作らない。ふだんの1セッションでは、いま動いているセッションが同じことを自分で行う。</p>'
  + fig(F['flow'], '意味の部分を含む操作の流れ', 'ユースケースを意味で作る例。main はいま動いているセッション、別のフォルダは使う側が別のフォルダ（ワークツリーなど）に立てたセッション。ふだんの1セッションでは、main が自分で SAS と同じやりとりをする。検証も同じで、validate --semantic で手順をもらい、--answer で判定を渡す。', 1.8)
  + cards([("セッションは増えない", "どの sas も、打ったセッションの操作になる"), ("判定はその時のもの", "結果は打ったセッションに返る。コミットしない"), ("CI", "機械の検証だけを毎回行う")])))
SCN = {
 'a': ("場面 a：エージェントが U10 を作り、検証する", "sas create requirements/usecases/U10.json --concrete urn:…:usecase:v1 --semantic --brief \"…\"\nsas create requirements/usecases/U10.json --concrete urn:…:usecase:v1 --value '{…}'\nsas validate requirements/usecases/U10.json --semantic\nsas validate requirements/usecases/U10.json --answer judgments.json",
       '{"kind": "procedure", "operation_kind": "write", "command": "sas create … --value <値の JSON>", …}\n{"kind": "change", "result": "created", "session": "4401cead…", "operation": "o-0031", …}\n{"kind": "procedure", "operation_kind": "validate", "command": "sas validate … --answer <ファイル | ->", …}\n{"kind": "validation", "session": "4401cead…", "counts": {"verified": 2, "unverified": 1}, …}',
       [["sessions", "4401cead…（tool claude-code ・ via env）。最初に sas を打ったときの1行"], ["changes", "o-0031 create U10 ・ session 4401cead…"], ["validations", "o-0032 U10 ・ hash sha256:3f9a… ・ session 4401cead…"], ["rule_results", "goal-phrase ai verified ／ no-design-words machine verified ／ stakeholder-interest human unverified"]]),
 'b': ("場面 b：エージェントが意味で探す", "sas search --concrete urn:…:usecase:v1 --semantic --question \"goal はなぜこの句か\"\nsas search --concrete urn:…:usecase:v1 --answer answer.json",
       '{"kind": "procedure", "operation_kind": "search", "question": "goal はなぜこの句か", "range": [{"path": "…/U10.json", "hash": "sha256:3f9a…"}, …], "steps": [{"no": 1, "action": "read", "path": "…/U10.json", "hash": "sha256:3f9a…", "current": {…}}, …], "command": "sas search --concrete urn:…:usecase:v1 --answer <答えの JSON のファイル | ->", …}\n{"kind": "search", "range": [{"path": "…/U10.json", "hash": "sha256:3f9a…"}, …], "answer": "…", "sources": [{"path": "…/U10.json", "pointer": "/goal", "value": "…"}]}\n（出どころが範囲の外か所が無い → invalid_answer、ハッシュが違う → conflict）',
       [["sessions", "打ったセッションの行（初めてのときだけ1つ）"], ["changes・validations", "入らない（検索は記録しない）"]]),
 'c': ("場面 c：SAS を通らない変更があった", "（エディタで U10 の goal を直す）\nsas validate requirements/usecases/U10.json\nsas update requirements/usecases/U10.json --adopt --intent \"goal を短くした\"",
       '{"kind": "validation", "untracked": [{"path": "…/U10.json", "expected": "sha256:3f9a…", "actual": "sha256:91c2…"}], "hint": "意図を付けて取り込む：sas update … --adopt --intent \\"…\\""}\n{"result": "adopted", "diff": {"source": "outside"}}',
       [["validations", "o-0042 U10 ・ hash sha256:91c2…"], ["changes", "o-0043 update ・ source outside ・ before sha256:3f9a… ・ after sha256:91c2…"]]),
 'd': ("場面 d：U11 を消す（U0 が U11 を指していても U0 は変えない）", "sas delete requirements/usecases/U11.json --intent \"U0 に統合した\"",
       '{"result": "deleted", "operation": "o-0040", "target": {"kind": "instance", "path": "…/U11.json"}}',
       [["changes", "o-0040 delete U11（事後なし・事前の欄を remove で残す）。U0 の記載は Git の差分とレビューで直す"]]),
 'e': ("場面 e：エージェントが U10 を意味で直し、断られて打ち直す", "sas update requirements/usecases/U10.json --semantic --brief \"goal を短くする\"\nsas update requirements/usecases/U10.json --patch '[…]' --if-match sha256:3f9a…\n（断った理由を読んで直す）\nsas update requirements/usecases/U10.json --patch '[…]' --if-match sha256:3f9a…",
       '{"kind": "procedure", "operation_kind": "write", "command": "sas update … --patch <JSON Patch> --if-match sha256:3f9a…", …}\n{"kind": "error", "code": "invalid_value", "message": "…", …}\n{"kind": "change", "result": "updated", "session": "4401cead…", "operation": "o-0050", …}',
       [["sessions", "増えない（同じセッション）"], ["changes", "o-0050 update U10 ・ session 4401cead…（断った操作は記録しないので、記録は1件）"]]),
}
use_html = ''
for k in 'abcde':
    t, cmd, out, rows = SCN[k]
    body = (f'<p class="lbl">実行するコマンド</p><pre>{html.escape(cmd)}</pre><p class="lbl">SAS が返すもの（抜粋）</p><pre>{html.escape(out)}</pre>'
            + '<p class="lbl">DB に入る行</p>' + tbl(['表', '行'], rows))
    use_html += f'<h3>{html.escape(t)}</h3>' + fig(F['sc' + k], t, '「エージェント」は sas を打ったセッション（AI のエージェント）。SAS はエージェントを起動しない。', 1.8) + det('コマンド・返すもの・DB に入る行', f'<div class="scn">{body}</div>')
T.append(('use', '使われ方',
  '<p class="ans">5つの場面を、誰が何を依頼し、SAS が何を返し、DB に何が残るかで見る。</p>' + use_html))
LINK_CODE = '<pre>' + html.escape(json.dumps({"$schema": "urn:example:purpose-session:v1", "id": "ps-2026-11-01-01", "purpose": "ユースケースの頁を仕上げる", "parent": None, "status": "進行中",
  "sas": {"session": "4401cead-…", "operations": ["o-0031", "o-0032"]}, "commit": None}, ensure_ascii=False, indent=2)) + '</pre>'
T.append(('link', '使う側とのつなぎ方',
  '<p class="ans">SAS の DB は使う側に共通のモデルとして使わせない。約束はセッションの ID と操作の ID だけ。</p>'
  + fig(F['link'], '使う側とのつなぎ方')
  + cards([("SAS → 使う側", "CLI は打ったセッションの ID と操作の ID を返す"), ("使う側 → SAS", "--session でセッションの ID を渡せる（環境変数から取れるツールでは省ける。取れないツールは sas init で登録する）"), ("残すべき結びつき", "目的とセッションの結びつきは、使う側が自分のインスタンスに書いて Git に残す。SAS の DB は作り直せる")])
  + det('使う側の Concrete の例', LINK_CODE)))
T.append(('uc', 'ユースケース',
  '<p class="ans">ユースケースは能力ごとの7件と要約1件。意味の部分は UC1・UC2・UC4・UC6 の中にあり、打ったセッションに手順を返す（判定は UC4 の変形「意味の判定」）。</p>'
  + fig(F['uc'], 'ユースケース')
  + det('8件の機械／意味と主アクター', wfix(tables(v5sec('uc'))[0] if tables(v5sec('uc')) else '').replace('UC7 セッションと操作を読む', 'UC7 操作の記録を読む').replace('<tr><td>SF1 SAS が立てたセッションでエージェントに手順で行わせる（サブ機能）</td><td>意味</td><td>呼び出したユースケースと同じ</td></tr>', '').replace('<td>UC4 検証する</td>', '<td>UC4 検証する（変形：意味の判定）</td>'))
  + det('いまの27件との対応', tbl(['扱い', 'いまのユースケース', '移す先'], MAP27))
  + det('直した保証と足した拡張（UC1・UC2・UC3・UC4・UC6・UC7）', UC_FIX)))
T.append(('decided', '決めたことと承認',
  '<p class="ans">決めたことと承認してほしい決定（「確かめる」は見解付き）。</p><h3>決めたこと</h3>' + tagged(DECIDED)
  + '<h3>承認してほしいこと</h3><div class="dec">' + ''.join(f'<div class="dc t-ask"><span class="tag">承認待ち</span><b>{h}</b><span>{d}</span></div>' for h, d in APPROVE) + '</div>'))

for k, v in T_NEW.items(): T.append(v)
T.append(T_NEED)
T.append(T_VERIFY)
ORDER = ['overview', 'types', 'uc', 'domain', 'model', 'verify', 'arch', 'data', 'session', 'flow', 'use', 'link', 'schema', 'need', 'decided']
T = sorted(T, key=lambda t: ORDER.index(t[0]))
STATUS = [("決まった", "s-ok", "全体像〜スキーマと DB の全部（計画の承認 2026-10-31、版16 の承認 2026-11-01、版28 はコミット済み、版29 は完成イメージの承認 2026-11-01）"), ("承認待ち", "s-wait", "土台の文（ACDR 0217）・ガイドのボードへ足す手順")]

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
        '<h1>Semantic Agent Schema の概念</h1><p class="read">SAS が何をし、何を記録し、どう使われるか。2026-11-01 版29（記録を読む sas log ・ 検索 sas search ・ 意味で書く手順に形の本体を入れる ・ 完了の確認。版28：SAS はエージェントを起動しない。打ったセッションを sessions に記録するだけで、子セッションの概念を外した。版27：ステップ7 意味で書く。版26：スキーマの分け方の作法。版25：operation-result を command-result に改名。版24：手順のプロンプトを procedure.schema.json に分けた。版23：rule の by をなくし、rule はどれも子セッションが判定する。版22：スキーマの形の未決5つ：x-relationship を外し、参照は Concrete の普通の欄に。Concrete に書ける x- は4つ。版21：Abstract を導き直した：JSON Schema で書けることは JSON Schema で。版20：ステップ6 検証用の子セッション。版19：描画。版18：言葉を本の定義にそろえた：公開された言葉を1か所に、書くと検証で1つの業務領域、リポジトリの根は製品、ポートとアダプター。版17：文脈の中を業務領域を作るモジュール（書く・検証・記録）にし、集約は書くのモジュールへ。意味の操作は業務領域にしない。版16：SAS は子セッションの状態を管理しない。書き込み用の子セッションと検証用の子セッションを --continue で続けて使い、fork は使わない。薄い sessions と、changes・validations に分けた記録。文脈は1つに戻した）。要求を書く（全体像〜ユースケース）→ 見つける（事業領域と文脈）→ 設計する（ドメインモデル〜スキーマと DB）の順に並べた。</p>'
        + status + f'<nav class="tabs" role="tablist" aria-label="内容">{tabs}</nav>' + panels + '</main>\n' + JS)
page = page.replace('指紋', 'ハッシュ')  # 以前の記録の語も、ふだん使う語で見せる
open(os.path.join(H, 'index.html'), 'w').write(page)
for n, r in REPORT: print(n, r)
print('ok', len(page))
