"""アーキテクチャのページ（arch.html）の5節：ディレクトリ構成を作る。
domain の各 crate の中は、集約（aggregates/）・ ドメインサービス（services/）・ 値オブジェクトに分けて見せる。ポートは application の ports/ に置く。"""
import json, os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
m = json.load(open(os.path.join(C, 'model', 'model.json')))
L = []
def line(path, d='', cls=''): L.append((path, d, cls))
P = lambda x: 'semantic-agent-schema-' + x
# 業務領域の crate ごとに：集約（ファイル名, モデルの集約名）・ ドメインサービス（ファイル名, モデルのドメインサービス名）・ ポート
# 業務領域の crate ごとに：(名前, 業務領域, 集約, ドメインサービス, 値オブジェクト, ファクトリ)
DOMAIN = [
 ('declaration', '意味の宣言', [('abstract_schema.rs', 'Semantic Abstract Schema'), ('concrete_schema.rs', 'Semantic Concrete Schema'), ('glossary.rs', 'glossary')], [],
  [('urn.rs', '名前の型（URN）'), ('target.rs', '対象（id と kind か、パス）'), ('procedure.rs', 'procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）')], [('concrete_template.rs', 'Concrete の雛形')]),
 ('instance', 'インスタンスの作成と変更', [('instance.rs', 'インスタンス')], [], [], [('instance_factory.rs', 'インスタンス（作る ・ 取り込む）')]),
 ('change', '変更の確かめと例外の判断', [('judgment.rs', '判定の記録'), ('change_record.rs', '変更の記録')], [],
  [('verdict.rs', '検証結果（5つの値 ・ 根拠 ・ 判定した者）'), ('instance_state.rs', 'インスタンスの状態の4つ'), ('change_id.rs', '変更の記録の id（ULID）')], [('change_summary.rs', '変更のまとめ（記録から機械的に作り、コミットする）')]),
 ('resolver', '意味の解決', [], [('resolver.rs', 'Semantic Schema Resolver', 'セマンティック スキーマ リゾルバー')], [], []),
 ('validation', '意味の検証', [], [('validator.rs', 'Semantic Schema Validator', 'セマンティック スキーマ バリデーター')], [('call.rs', '依頼の一覧（呼び出し call）')], []),
 ('render', '描画', [('view.rs', 'ビュー'), ('design_system.rs', 'デザインシステム')], [('renderer.rs', 'Semantic Schema Renderer', 'セマンティック スキーマ レンダラー')], [], []),
 ('search', '探す', [], [], [('query.rs', '探す条件（JMESPath の where ・ select）'), ('provenance.rs', '出どころ（id ・ kind ・ 版 ・ where ・ select）'), ('scope.rs', '探した範囲')], []),
 ('session', '目的ごとのセッションの管理', [('session.rs', 'セッション')], [('selector.rs', 'Session Selector', 'セッション セレクター')], [('session_id.rs', 'セッション ID'), ('child_kind.rs', '子のセッションの型（単発 ・ 継続 × 引き継ぐ ・ 引き継がない）')], []),
]
# モデルと突き合わせる：集約9 ・ ドメインサービス4 が1回ずつ現れること
aggs = [a['name'] for a in m['aggregates']]
got_a = [n for _, _, A, _, _, _ in DOMAIN for _, n in A]; got_s = [x[1] for _, _, _, S, _, _ in DOMAIN for x in S]
assert sorted(got_a) == sorted(aggs), (set(aggs) ^ set(got_a))
assert sorted(got_s) == sorted(d[0] for d in m['domain_services']), (set(got_s) ^ set(d[0] for d in m['domain_services']))

line('semantic-agent-schema/', 'リポジトリ直下。ワークスペースの根')
line('  Cargo.toml', 'ワークスペース。members は crates/domain/* ・ crates/application ・ crates/outbound/* ・ crates/inbound/*')
line('  Cargo.lock'); line('  CODING.md', 'この構成と検査の規則'); line('  architecture.json', '許す辺 ・ 辺の種類 ・ crate ごとに許す外の crate（検査の正本）')
line('  schemas/', '公開された言葉。集約 Semantic Abstract Schema の中身'); line('    v1/', 'Abstract の13ファイル（10 ＋ query ・ procedure ・ change-summary。出した版は変えない。declaration が build.rs で取り込む）'); line('      views/', '基盤の既定のビュー（session ・ change-summary ・ judgment。文脈の Concrete のビューがあればそちらを使う）'); line('  test-suite/', 'システムテストの約21場面（入力と期待結果の JSON）')
line('  crates/', 'コード（crate）だけを置く。下のフォルダが依存の向きの層')
line('    domain/', '業務領域ごとの crate。値を受け取り値を返す。入出力をしない ・ ポートを持たない')
for name, desc, A, S, V, F in DOMAIN:
    line(f'      {name}/', f'{P(name)}　{desc}', 'cr'); line('        Cargo.toml'); line('        src/'); line('          lib.rs')
    for f, d in V: line(f'          {f}', d)
    if A:
        line('          aggregates/', '集約', 'ag')
        for f, n in A: line(f'            {f}', f'集約 {n}')
    if F:
        line('          factories/', 'ファクトリ', 'fc')
        for f, n in F: line(f'            {f}', f'ファクトリ {n}')
    if S:
        line('          services/', 'ドメインサービス', 'sv')
        for f, n, k in S: line(f'            {f}', f'ドメインサービス {n}（{k}）')
    if name == 'declaration': line('        build.rs', 'schemas/ を取り込み、$id で引く')
line('    application/', f'{P("application")}　使う場面（入口のコマンドごと）', 'cr'); line('      Cargo.toml'); line('      src/'); line('        lib.rs')
for p_ in m['ports']['inbound']:
    pass
APP = [('semantic.rs', 'セマンティック7つ（search ・ read ・ create ・ write ・ judge ・ import ・ migrate）。procedure を返す'),
       ('search.rs', 'search ・ get'), ('create.rs', 'create'), ('patch.rs', 'patch（検査①：適用する前に入力を見る）'), ('import.rs', 'import'),
       ('validate.rs', 'validate（検査②。依頼の一覧を返し、答えをまとめる）'), ('judge.rs', 'judge（人の判定だけ）'), ('render.rs', 'render（既定のビューと Concrete のビュー）'),
       ('record.rs', 'record ・ まとめを作る（コミットの前。記録と変更の前後を両方向で突き合わせる）'), ('history.rs', 'history（経緯をたどる）'),
       ('release.rs', 'release ・ migrate（版を出す ・ 移す）'), ('session.rs', 'session（立てる ・ 再開する ・ 閉じる）')]
line('        use_cases/', '使う場面（コマンドの組ごとに1ファイル）', 'sv')
for f, d in APP: line(f'          {f}', d)
PORTS = [('contracts.rs', '契約を読む ・ 書く'), ('instances.rs', 'インスタンスを読む ・ 書く'), ('judgments.rs', '判定の記録を読む ・ 書く（rule の判定はファイル、意図の判定は手元の DB）'), ('change_records.rs', '変更の記録を読む ・ 書く（手元の DB）'),
         ('sessions.rs', 'セッションと手元のセッションの索引を読む ・ 書く'), ('change_summaries.rs', '変更のまとめを読む ・ 書く（ファイル。コミットごとに1件）'), ('changes.rs', '変更の仕組みとやりとりする（前後 ・ 単位の中身 ・ 確定する）'), ('published.rs', '公開された言葉を取りに行く'),
         ('judge.rs', 'rule の判定を依頼する（AI）'), ('rendered.rs', '描画したものを書き出す'), ('tools.rs', '外のツールを呼ぶ'), ('ai_sessions.rs', 'AI ツールのセッションを問う'), ('settings.rs', '設定を読む')]
assert len(PORTS) == len(m['ports']['outbound'])
line('        ports/', f'駆動される側のポート{len(PORTS)}（trait。outbound が実装する）', 'pt')
for f, d in PORTS: line(f'          {f}', d)
line('    outbound/', '駆動される側のアダプター。application の ports/ を実装する')
AD = [('agents-contract', '腐敗防止層（Agents 契約）。judge を実装する', [('call.rs', '依頼の一覧を Agents 契約の呼び出しの形へ直す'), ('answer.rs', '答えの形を読む（5つの値への判定は validation）')]),
      ('ai-cli', 'AI ツールの CLI。ai_sessions を実装し、agents-contract が起動に使う', [('claude_code.rs', ''), ('codex.rs', ''), ('copilot.rs', ''), ('gemini.rs', ''), ('kiro.rs', '')]),
      ('git', '変更の仕組み（Git の CLI を呼ぶ）。changes を実装する', [('changes.rs', '前後 ・ 単位の中身 ・ 確定する')]),
      ('fs', 'ファイル', [('files.rs', 'contracts ・ instances ・ judgments（rule の判定）・ change_summaries'), ('settings.rs', 'settings（3段を重ねる）'), ('rendered.rs', 'rendered'), ('published.rs', 'published（閉じた一覧の $id だけ）'), ('snapshot.rs', 'changes の別の実装（前と後のディレクトリ）')]),
      ('tools', '外のツール（Concrete が宣言した validator）。tools を実装する', [('run.rs', '名前と版で起動する')]),
      ('sqlite', '手元の DB。sessions ・ change_records ・ judgments（意図の判定）を実装する', [('sessions_store.rs', 'セッションと手元の索引'), ('change_store.rs', '変更の記録と意図の判定'), ('schema.sql', 'DDL（表 ・ 一意の索引 ・ トリガー。sessions に context）')])]
for n, d, fs in AD:
    line(f'      {n}/', f'{P(n)}　{d}', 'cr'); line('        Cargo.toml'); line('        src/'); line('          lib.rs')
    for f, dd in fs: line(f'          {f}', dd)
line('    inbound/', '駆動する側のアダプター（入口）'); line('      cli/', f'{P("cli")}　入口', 'cr'); line('        Cargo.toml'); line('        src/')
line('          main.rs', '起動するだけ'); line('          commands/', 'セマンティック7つ ・ 機械的なコマンドの引数を読む'); line('          compose.rs', 'outbound を application へ渡す1か所。時刻と id もここから値で渡す'); line('        tests/system.rs', 'システムテスト（test-suite/ の約21場面をバイナリで回す）')
rows = ''.join(f'<span class="{c}">{p}{"  <i>" + d + "</i>" if d else ""}</span>' for p, d, c in L)
svc_rows = ''.join(f'<tr><td>{n}</td><td>{k}</td><td>crates/domain/{name}/src/services/{f}</td></tr>' for name, _, _, S, _, _ in DOMAIN for f, n, k in S)
sec = f'''<h2>5　ディレクトリ構成（crate と、その中のモジュール）</h2>
<p class="meta">コードは crates/ にまとめ、その下のフォルダで層を見せる（利用者が案 C を選んだ）。domain の各 crate の中は、集約（aggregates/）・ ドメインサービス（services/）・ 値オブジェクトに分ける。ポートは application の ports/ に置く（arch/ports の承認）。集約は model.json の9つと、作るときに突き合わせて全部が1回ずつ現れることを確かめている。ドメインサービスは決めた4つで、ほかの処理は集約の操作とファクトリ3つが持つ</p>
<p class="meta lg"><span class="cr">crate</span> <span class="ag">集約</span> <span class="fc">ファクトリ</span> <span class="sv">ドメインサービス ・ 使う場面</span> <span class="pt">ポート</span></p>
<pre class="v">{rows}</pre>
<h3>ドメインサービス4つの置き場所</h3>
<div class="tw"><table><tr><th>ドメインサービス</th><th>カタカナ</th><th>ファイル</th></tr>{svc_rows}</table></div>
<div class="tw"><table><tr><th>決まり</th><th>中身</th></tr>
<tr><td>根</td><td>コード（crates/）と、契約 ・ 場面 ・ 規則（schemas/ ・ test-suite/ ・ CODING.md ・ architecture.json）を分ける</td></tr>
<tr><td>crate ＝ フォルダ</td><td>crates/domain/ ・ crates/outbound/ ・ crates/inbound/ の下のフォルダと crates/application/ が、crate 1つずつ。フォルダの並びが依存の向きの層。inbound ・ outbound の名前は Q5 #9</td></tr>
<tr><td>domain の crate の中</td><td>aggregates/（集約1つに1ファイル。集約の操作もここ）・ factories/（ファクトリ1つに1ファイル）・ services/（ドメインサービス1つに1ファイル）・ 値オブジェクトは src/ の直下。ポートを持たない。値オブジェクトは、その語を持つ業務領域の crate に置く</td></tr>
<tr><td>application の中</td><td>use_cases/（入口のコマンドの組ごとに1ファイル。読む → 集約の操作 ・ ドメインサービス → 書く）・ ports/（駆動される側のポート1つに1ファイル）</td></tr>
<tr><td>識別子</td><td>crate ・ ファイル ・ 型の名前は英語（ASCII）。業務領域の語との対訳を CODING.md の表に持つ</td></tr>
</table></div>
'''
p = os.path.join(C, 'arch.html'); s = open(p).read()
i = s.index('<h2>5　ディレクトリ構成'); j = s.index('<h2>6　決定的にする検査</h2>')
s = s[:i] + sec + s[j:]
if 'pre.v .ag' not in s:
    s = s.replace('</style>', 'pre.v .ag{background:var(--hot-bg);color:var(--hot)}pre.v .sv{background:var(--code);color:var(--accent)}pre.v .pt{color:var(--bad)}pre.v .fc{background:var(--accent-bg);color:var(--ink)}.lg .fc{background:var(--accent-bg);color:var(--ink)}.lg span{border-radius:6px;padding:1px 8px;margin-right:6px;font-size:12px}.lg .cr{background:var(--accent-bg);color:var(--accent)}.lg .ag{background:var(--hot-bg);color:var(--hot)}.lg .sv{background:var(--code);color:var(--accent)}.lg .pt{color:var(--bad)}\n</style>', 1)
open(p, 'w').write(s)
print('ok', len(got_a), len(got_s))
