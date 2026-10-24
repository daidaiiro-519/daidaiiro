"""アーキテクチャのページ（arch.html）の5節：ディレクトリ構成を作る。
domain の各 crate の中は、集約（aggregates/）・ ドメインサービス（services/）・ ポート（ports.rs）に分けて見せる。"""
import json, os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
m = json.load(open(os.path.join(C, 'model', 'model.json')))
L = []
def line(path, d='', cls=''): L.append((path, d, cls))
P = lambda x: 'semantic-agent-schema-' + x
# 業務領域の crate ごとに：集約（ファイル名, モデルの集約名）・ ドメインサービス（ファイル名, モデルのドメインサービス名）・ ポート
DOMAIN = [
 ('values', '2つ以上の領域が使う値', [], [], [('verdict.rs', '検証の5つの値（enum）'), ('instance_state.rs', 'インスタンスの状態の4つ'), ('urn.rs', '名前の型（URN）'), ('target.rs', '対象（id と kind か、パス）'), ('ids.rs', '変更の記録の id（ULID）・ セッション ID')], None),
 ('declaration', '意味の宣言', [('abstract_schema.rs', 'Semantic Abstract Schema'), ('concrete_schema.rs', 'Semantic Concrete Schema'), ('glossary.rs', 'glossary')],
  [('create.rs', '作成する'), ('conform.rs', 'Concrete を照らす')], [], '契約を読む ・ 書く'),
 ('validation', '意味の検証', [('judgment.rs', '判定の記録')], [('resolver.rs', 'Semantic Schema Resolver'), ('validator.rs', 'Semantic Schema Validator')], [],
  '判定の依頼 ・ 外のツールを呼ぶ ・ 公開された言葉を取りに行く ・ 判定の記録を読む書く ・ インスタンスを読む'),
 ('change', '変更の確かめと例外の判断', [('change_record.rs', '変更の記録')], [('derive_state.rs', '状態を導く'), ('show_changes.rs', '変更を見せる'), ('exceptions.rs', '例外をまとめる')], [],
  '変更の仕組みとやりとりする ・ 変更の記録を読む書く'),
 ('search', '意味で探す', [], [('search.rs', '意味で探して取り出す')], [], None),
 ('session', '目的ごとのセッションの管理', [('session.rs', 'セッション')], [('resume_or_new.rs', '再開か新規かを決める')], [], 'セッションと手元の索引 ・ AI ツールのセッションを問う ・ 設定を読む'),
 ('render', '描画', [('view.rs', 'ビュー'), ('design_system.rs', 'デザインシステム')], [('renderer.rs', 'Semantic Schema Renderer')], [], '描画したものを書き出す'),
 ('instance', 'インスタンスの作成と変更', [('instance.rs', 'インスタンス')], [('import.rs', '取り込む')], [], 'インスタンスを書く'),
 ('migration', 'マイグレーション', [], [('migrate.rs', '移す')], [], None),
 ('publish', 'スキーマの保存と公開', [], [('publish.rs', '公開された言葉の版を出す')], [], None),
]
# モデルと突き合わせる：集約9 ・ ドメインサービス13 が1回ずつ現れること
aggs = [a['name'] for a in m['aggregates']]; svcs = [d[0] for d in m['domain_services']]
got_a = [n for _, _, A, _, _, _ in DOMAIN for _, n in A]; got_s = [n for _, _, _, S, _, _ in DOMAIN for _, n in S]
assert sorted(got_a) == sorted(aggs), (set(aggs) ^ set(got_a))
assert sorted(got_s) == sorted(svcs), (set(svcs) ^ set(got_s))

line('semantic-agent-schema/', 'リポジトリ直下。ワークスペースの根')
line('  Cargo.toml', 'ワークスペース。members は crates/domain/* ・ crates/application ・ crates/adapters/* ・ crates/entry/*')
line('  Cargo.lock'); line('  CODING.md', 'この構成と検査の規則'); line('  architecture.json', '許す辺 ・ 辺の種類 ・ crate ごとに許す外の crate（検査の正本）')
line('  schemas/', 'Abstract の10ファイル（declaration が build.rs で取り込む）'); line('  spec/', '受け入れの場面 behaviors/ と対応表 coverage.json')
line('  crates/', 'コード（crate）だけを置く。下のフォルダが依存の向きの層')
line('    domain/', '業務領域ごとの crate。入出力をしない（clippy の forbid）')
for name, desc, A, S, V, port in DOMAIN:
    line(f'      {name}/', f'{P(name)}　{desc}', 'cr'); line('        Cargo.toml'); line('        src/'); line('          lib.rs')
    for f, d in V: line(f'          {f}', d)
    if A:
        line('          aggregates/', '集約', 'ag')
        for f, n in A: line(f'            {f}', f'集約 {n}')
    if S:
        line('          services/', 'ドメインサービス', 'sv')
        for f, n in S: line(f'            {f}', f'ドメインサービス {n}')
    if port: line('          ports.rs', f'ポート：{port}', 'pt')
    if name == 'declaration': line('        build.rs', 'schemas/ を取り込み、$id で引く')
line('    application/', f'{P("application")}　使う場面（駆動する側のポート10）', 'cr'); line('      Cargo.toml'); line('      src/'); line('        lib.rs')
for p_ in m['ports']['inbound']:
    pass
APP = [('declare.rs', '意味を宣言する'), ('write_instance.rs', 'インスタンスを書く'), ('record_change.rs', '変更を記録する'), ('validate.rs', '検証する ・ 変更を見せる'), ('judge.rs', '判定する（人の入口だけ）'),
       ('search.rs', '探して取り出す ・ 案内する'), ('render.rs', '描画する'), ('release.rs', 'Abstract の版を出す ・ 移す（基盤を作る側だけ）'), ('trace.rs', '経緯をたどる'), ('session.rs', 'セッションを決める ・ 立てる ・ 閉じる')]
line('        use_cases/', '使う場面（1つの使う場面に1ファイル）', 'sv')
for f, d in APP: line(f'          {f}', d)
line('        ports.rs', 'ポート：時刻と id を受け取る', 'pt')
line('    adapters/', '外の技術。ポートを実装する')
AD = [('agents-contract', '腐敗防止層（Agents 契約）', [('call.rs', '呼び出し（call）を作る'), ('answer.rs', '答えを検査して5つの値へ直す')]),
      ('ai-cli', 'AI ツールの CLI', [('claude_code.rs', ''), ('codex.rs', ''), ('copilot.rs', ''), ('gemini.rs', ''), ('kiro.rs', '')]),
      ('git', '変更の仕組み（Git の CLI を呼ぶ）', [('changes.rs', '前後 ・ 単位の中身 ・ 消した ID ・ 確定する')]),
      ('fs', 'ファイル', [('files.rs', '契約 ・ インスタンス ・ 記録を読む書く'), ('settings.rs', '設定を読む'), ('snapshot.rs', '変更の仕組み（前と後のディレクトリ）'), ('clock.rs', '時刻と id')]),
      ('sqlite', '手元の DB', [('sessions_store.rs', 'セッションと手元の索引'), ('schema.sql', 'DDL（表 ・ 一意の索引 ・ トリガー）')])]
for n, d, fs in AD:
    line(f'      {n}/', f'{P(n)}　{d}', 'cr'); line('        Cargo.toml'); line('        src/'); line('          lib.rs')
    for f, dd in fs: line(f'          {f}', dd)
line('    entry/', '入口'); line('      cli/', f'{P("cli")}　入口', 'cr'); line('        Cargo.toml'); line('        src/')
line('          main.rs', '起動するだけ'); line('          compose.rs', 'アダプターを application へ渡す1か所'); line('        tests/acceptance.rs', '受け入れの試験（spec/behaviors を全アダプターで回す）')
rows = ''.join(f'<span class="{c}">{p}{"  <i>" + d + "</i>" if d else ""}</span>' for p, d, c in L)
svc_rows = ''.join(f'<tr><td>{n}</td><td>crates/domain/{name}/src/services/{f}</td></tr>' for name, _, _, S, _, _ in DOMAIN for f, n in S)
sec = f'''<h2>5　ディレクトリ構成（crate と、その中のモジュール）</h2>
<p class="meta">コードは crates/ にまとめ、その下のフォルダで層を見せる（利用者が案 C を選んだ）。domain の各 crate の中は、集約（aggregates/）・ ドメインサービス（services/）・ ポート（ports.rs）に分ける。並びはモデル（model.json）の集約9 ・ ドメインサービス13と、作るときに突き合わせて全部が1回ずつ現れることを確かめている</p>
<p class="meta lg"><span class="cr">crate</span> <span class="ag">集約</span> <span class="sv">ドメインサービス ・ 使う場面</span> <span class="pt">ポート</span></p>
<pre class="v">{rows}</pre>
<h3>ドメインサービス13の置き場所</h3>
<div class="tw"><table><tr><th>ドメインサービス（モデル）</th><th>ファイル</th></tr>{svc_rows}</table></div>
<div class="tw"><table><tr><th>決まり</th><th>中身</th></tr>
<tr><td>根</td><td>コード（crates/）と、契約 ・ 場面 ・ 規則（schemas/ ・ spec/ ・ CODING.md ・ architecture.json）を分ける</td></tr>
<tr><td>crate ＝ フォルダ</td><td>crates/domain/ ・ crates/adapters/ ・ crates/entry/ の下のフォルダと crates/application/ が、crate 1つずつ。フォルダの並びが依存の向きの層</td></tr>
<tr><td>domain の crate の中</td><td>aggregates/（集約1つに1ファイル）・ services/（ドメインサービス1つに1ファイル）・ ports.rs（その crate が呼ぶポート）。2つ以上の領域が使う値は values の crate</td></tr>
<tr><td>application の中</td><td>use_cases/（駆動する側のポート1つに1ファイル）・ ports.rs</td></tr>
<tr><td>識別子</td><td>crate ・ ファイル ・ 型の名前は英語（ASCII）。業務領域の語との対訳を CODING.md の表に持つ</td></tr>
</table></div>
'''
p = os.path.join(C, 'arch.html'); s = open(p).read()
i = s.index('<h2>5　ディレクトリ構成'); j = s.index('<h2>6　決定的にする検査</h2>')
s = s[:i] + sec + s[j:]
if 'pre.v .ag' not in s:
    s = s.replace('</style>', 'pre.v .ag{background:var(--hot-bg);color:var(--hot)}pre.v .sv{background:var(--code);color:var(--accent)}pre.v .pt{color:var(--bad)}.lg span{border-radius:6px;padding:1px 8px;margin-right:6px;font-size:12px}.lg .cr{background:var(--accent-bg);color:var(--accent)}.lg .ag{background:var(--hot-bg);color:var(--hot)}.lg .sv{background:var(--code);color:var(--accent)}.lg .pt{color:var(--bad)}\n</style>', 1)
open(p, 'w').write(s)
print('ok', len(got_a), len(got_s))
