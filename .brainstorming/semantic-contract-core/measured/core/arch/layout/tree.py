"""確定するディレクトリ構成の正本（頁と雛形の両方がここから作る）。
節点：(名前, 説明, 子の並び or None, 出どころ, 種類)
出どころ：ok＝承認済み（ACDR 0201 の arch.html ・ ボード）／ rev＝見直し（承認済みから変える）／ new＝新しく決める
種類：agg 集約 ・ ent エンティティ ・ fac ファクトリ ・ svc ドメインサービス ・ val 値 ・ port ポート ・ uc 使う場面 ・ ad アダプター"""
OK, REV, NEW = 'ok', 'rev', 'new'
def F(n, d='', o=OK, k=None): return (n, d, None, o, k)
def D(n, d, kids, o=OK, k=None): return (n, d, kids, o, k)
P = 'semantic-agent-schema-'

# 依存の辺（arch.html 4節。session → declaration だけ見直しで足す）
EDGES = {
 'declaration': [], 'instance': ['declaration'], 'change': ['declaration'], 'resolver': ['instance', 'declaration'],
 'validation': ['resolver', 'change', 'declaration'], 'render': ['resolver', 'change', 'declaration'],
 'search': ['change', 'instance', 'declaration'], 'session': ['declaration'],
}
OUTBOUND = ['agents-contract', 'ai-cli', 'git', 'fs', 'tools', 'sqlite']

def crate(name, desc, src, o=OK, extra=None):
    kids = [F('Cargo.toml', f'crate の名前 {P}{name}', OK), D('src/', 'ソース', [F('lib.rs', 'モジュールを宣言し、公開するものを pub use で並べる', OK)] + src, OK)]
    if extra: kids += extra
    return D(f'{name}/', desc, kids, o)

# domain の crate の中：values/（値オブジェクト）・ aggregates/（集約。entities/ ・ factories/ を下に持つ）・ services/（ドメインサービス）
# エンティティかファクトリを持つ集約は、集約の名前のフォルダ（aggregates/<集約>/）に entities/ ・ factories/ を置き、欄を pub(in crate::aggregates::<集約>) にする
def values(items, o=None):
    return [D('values/', '値オブジェクト', [F(f, d, oo, 'val') for f, d, oo in items], REV)]
def aggs(items, shared_entities=None):
    """items：(ファイル, 説明, エンティティの並び, ファクトリの並び)。エンティティかファクトリを持つ集約だけ、集約の名前のフォルダを作る"""
    kids = []
    for f, d, ents, facs in items:
        name = f[:-3]
        kids.append(F(f, d + f'。欄は pub(in crate::aggregates::{name})' if (ents or facs) else d, OK, 'agg'))
        if ents or facs:
            sub = []
            if ents: sub.append(D('entities/', 'この集約のエンティティ', [F(ef, ed, NEW, 'ent') for ef, ed in ents], NEW))
            if facs: sub.append(D('factories/', 'この集約のファクトリ（欄で作る）', [F(ff, fd, REV, 'fac') for ff, fd in facs], REV))
            kids.append(D(f'{name}/', f'{d} の境界（このフォルダの外からは欄に触れない）', sub, REV))
    if shared_entities:
        kids.append(D('entities/', '2つ以上の集約が持つエンティティ（pub(in crate::aggregates)）', [F(ef, ed, NEW, 'ent') for ef, ed in shared_entities], NEW))
    return [D('aggregates/', '集約', kids, OK)]
def svcs(items):
    return [D('services/', 'ドメインサービス', [F(f, d, OK, 'svc') for f, d in items], OK)]
DOMAIN = [
 crate('declaration', '意味の宣言', [
   *values([('urn.rs', '名前の型（URN）', OK), ('target.rs', '対象（id と kind か、パス）', OK), ('procedure.rs', 'procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）', OK), ('purpose_ref.rs', '目的の参照（session と change が使う）', REV)]),
   *aggs([('abstract_schema.rs', 'Semantic Abstract Schema', [], []), ('concrete_schema.rs', 'Semantic Concrete Schema', [('rule.rs', 'rule')], [('concrete_template.rs', 'Concrete の雛形')]), ('glossary.rs', 'glossary', [], [])],
         shared_entities=[('delegate.rs', '任せる先（x-delegates。Abstract と Concrete の両方が持つ）')])],
   extra=[F('build.rs', 'schemas/v1/ を取り込み、$id で引く', OK)]),
 crate('instance', 'インスタンスの作成と変更', [*aggs([('instance.rs', 'インスタンス', [], [('instance_factory.rs', 'インスタンス（作る ・ 取り込む）')])])]),
 crate('change', '変更の確かめと例外の判断', [
   *values([('verdict.rs', '検証結果（5つの値 ・ 根拠 ・ 判定した者）', OK), ('instance_state.rs', 'インスタンスの状態の4つ', OK), ('change_id.rs', '変更の記録の id（ULID）', OK)]),
   *aggs([('judgment.rs', '判定の記録', [], []), ('change_record.rs', '変更の記録', [], [('change_summary.rs', '変更のまとめ（記録から機械的に作る）')])])]),
 crate('resolver', '意味の解決', [*svcs([('resolver.rs', 'Semantic Schema Resolver（セマンティックスキーマリゾルバー）')])]),
 crate('validation', '意味の検証', [*values([('call.rs', '依頼の一覧（呼び出し call）', OK)]), *svcs([('validator.rs', 'Semantic Schema Validator（セマンティックスキーマバリデーター）')])]),
 crate('render', '描画', [*aggs([('view.rs', 'ビュー', [], []), ('design_system.rs', 'デザインシステム', [], [])]), *svcs([('renderer.rs', 'Semantic Schema Renderer（セマンティックスキーマレンダラー）')])]),
 crate('search', '探す', [*values([('query.rs', '探す条件（JMESPath の where ・ select）', OK), ('provenance.rs', '出どころ', OK), ('scope.rs', '探した範囲', OK)])]),
 crate('session', '目的ごとのセッションの管理', [*values([('session_id.rs', 'セッション ID', OK), ('child_kind.rs', '子のセッションの型', OK)]), *aggs([('session.rs', 'セッション', [], [])]), *svcs([('selector.rs', 'Session Selector（セッションセレクター）')])]),
]

APP = [('semantic.rs', 'セマンティック7つ。procedure を返す'), ('search.rs', 'search ・ get'), ('create.rs', 'create'), ('patch.rs', 'patch（検査①：適用する前に入力を見る）'), ('import.rs', 'import'),
       ('validate.rs', 'validate（検査②。依頼の一覧を返し、答えをまとめる）'), ('judge.rs', 'judge（人の判定だけ）'), ('render.rs', 'render'),
       ('record.rs', 'record ・ まとめを作る'), ('history.rs', 'history'), ('release.rs', 'release ・ migrate'), ('session.rs', 'session（立てる ・ 再開する ・ 閉じる）')]
PORTS = [('contracts.rs', '契約を読む ・ 書く'), ('instances.rs', 'インスタンスを読む ・ 書く'), ('judgments.rs', '判定の記録を読む ・ 書く'), ('change_records.rs', '変更の記録を読む ・ 書く'),
         ('sessions.rs', 'セッションと手元のセッションの索引（索引の型もここ）'), ('change_summaries.rs', '変更のまとめを読む ・ 書く'), ('changes.rs', '変更の仕組みとやりとりする'), ('published.rs', '公開された言葉を取りに行く'),
         ('judge.rs', 'rule の判定を依頼する（AI）'), ('rendered.rs', '描画したものを書き出す'), ('tools.rs', '外のツールを呼ぶ'), ('ai_sessions.rs', 'AI ツールのセッションを問う'), ('settings.rs', '設定を読む')]
AD = [('agents-contract', '腐敗防止層（Agents 契約）。judge を実装する', [('call.rs', '依頼の一覧を Agents 契約の形へ直す'), ('answer.rs', '答えの形を読む')]),
      ('ai-cli', 'AI ツールの CLI。ai_sessions を実装する', [('claude_code.rs', ''), ('codex.rs', ''), ('copilot.rs', ''), ('gemini.rs', ''), ('kiro.rs', '')]),
      ('git', '変更の仕組み（Git の CLI）。changes を実装する', [('changes.rs', '前後 ・ 単位の中身 ・ 確定する')]),
      ('fs', 'ファイル', [('files.rs', 'contracts ・ instances ・ judgments（rule の判定）・ change_summaries'), ('settings.rs', 'settings'), ('rendered.rs', 'rendered'), ('published.rs', 'published'), ('snapshot.rs', 'changes の別の実装（前と後のディレクトリ）')]),
      ('tools', '外のツール。tools を実装する', [('run.rs', '名前と版で起動する')]),
      ('sqlite', '手元の DB。sessions ・ change_records ・ judgments（意図の判定）を実装する', [('sessions_store.rs', 'セッションと手元の索引'), ('change_store.rs', '変更の記録と意図の判定'), ('schema.sql', 'DDL')])]

BOARD_NAMED = {'change', 'judgment', 'session', 'view', 'catalog', 'design-tokens', 'glossary', 'published'}  # ボード Q6 に名前がある
SCHEMAS = [('semantic-abstract-schema.json', '本体（メタスキーマ）', OK)] + [(f'{n}.schema.json', d, OK if n in BOARD_NAMED else NEW) for n, d in [
  ('change', '変更の記録'), ('change-summary', '変更のまとめ'), ('judgment', '判定の記録'), ('session', 'セッション'), ('procedure', 'セマンティックのコマンドが返す手順'),
  ('query', '探す条件 ・ 結果 ・ 出どころ'), ('settings', '設定'), ('view', 'ビュー'), ('catalog', '部品のカタログ'), ('design-tokens', 'デザイントークン'), ('glossary', '文脈の言葉'), ('published', '公開された言葉の版')]]

TREE = D('semantic-agent-schema/', 'リポジトリ直下。ワークスペースの根', [
 F('Cargo.toml', 'ワークスペース（virtual manifest）。members ・ resolver ・ [workspace.package]（rust-version 1.97）・ [workspace.dependencies] ・ [workspace.lints]', OK),
 F('Cargo.lock', '依存の版の1か所', OK),
 F('rustfmt.toml', 'style_edition だけ', NEW), D('.cargo/', '', [F('config.toml', 'build.warnings = "deny"（警告で止める）', NEW)], NEW),
 F('clippy.toml', 'lint の設定値（試験のコードで unwrap ・ expect ・ panic を許す）', NEW),
 F('architecture.json', 'architecture-lint の規則（crate の間で許す依存の辺）', OK),
 F('CODING.md', 'コーディング規則', OK), F('README.md', '', NEW),
 D('schemas/', '公開された言葉', [D('v1/', 'Abstract の13ファイル（出した版は変えない）', [F(n, d, o) for n, d, o in SCHEMAS] + [
   D('views/', '基盤の既定のビュー', [F('session.json', '', NEW), F('change-summary.json', '', NEW), F('judgment.json', '', NEW)], OK)], OK)], OK),
 D('test-suite/', 'システムテストの入力と期待', [D('scenarios/', '場面1つ1ファイル', [F('*.json', 'given ・ when ・ after ・ then ・ unchanged', NEW)], REV),
   F('trace.json', '不変条件 ・ 使う場面 ・ ポートの id と名前の一覧（追跡の検査が読む）', NEW), F('untested.json', '試験がまだ無い行（why ・ move_when）', NEW)], OK),
 D('crates/', 'コードだけを置く。下のフォルダが依存の向きの層', [
   D('domain/', '業務領域ごとの crate。値を受け取り値を返す。入出力をしない ・ ポートを持たない', [F('clippy.toml', '入出力を禁じる（disallowed-methods ・ disallowed-types）＋ 根の設定値の写し', NEW)] + DOMAIN, OK),
   crate('application', '使う場面とポート', [D('use_cases/', '使う場面（コマンドの組ごとに1ファイル）', [F(f, d, OK, 'uc') for f, d in APP], OK),
       D('ports/', '駆動される側のポート13（trait。outbound が実装する）', [F(f, d, OK, 'port') for f, d in PORTS], OK)],
       extra=[D('tests/', '結合テスト', [D('use_cases/', '', [F('main.rs', '使う場面ごとのモジュール', NEW), D('doubles/', '代役（ポート1つ1ファイル。FakeXxx ・ StubXxx）', [F('…', '', NEW)], NEW)], NEW)], NEW)]),
   D('outbound/', '駆動される側のアダプター。application の ports/ を実装する', [
       crate(n, d, [F(f, dd, OK, 'ad') for f, dd in fs], extra=[D('tests/', '結合テスト', [D(f'{n}/', '', [F('main.rs', '', NEW)], NEW)], NEW)]) for n, d, fs in AD], OK),
   D('inbound/', '駆動する側のアダプター（入口）', [crate('cli', '入口。lib ＋ 実行ファイル2つ（CLI ・ MCP）', [F('main.rs', 'CLI の実行ファイル。引数を受け取って compose を呼ぶだけ', OK), D('commands/', 'セマンティック7つ ・ 機械的なコマンドの引数を読む', [F('semantic.rs', '', NEW), F('mechanical.rs', '', NEW)], OK),
       F('compose.rs', 'outbound を application へ渡す1か所。時刻と id もここから値で渡す', OK),
       D('bin/', '2つ目の実行ファイル', [F('mcp.rs', 'MCP の入口（semantic-agent-schema-mcp）。compose.rs を同じく使う', REV)], REV)],
       extra=[D('tests/', 'システムテスト', [D('system/', '1つの結合テストの crate（harness = false）', [F('main.rs', 'test-suite の場面を1件ずつ走らせる', REV), F('trace.rs', '追跡の検査（trace.json ・ syn ・ grep）', NEW)], REV)], OK)])], OK),
 ], OK),
], OK)

# 承認済みの形から変えたもの（変更前 → 変更後 ・ 理由）
REVISIONS = [
 ('ファクトリの置き場所', 'crate の直下の factories/', 'aggregates/<集約>/factories/ に移し、集約の欄を pub(in crate::aggregates::<集約>) にする', '直下の factories/ からは、欄を pub(crate) にしないと集約を作れない（実測 E0451）。pub(crate) だと crate のどこからでも書き換えられる。集約のフォルダの下に置けば、欄をその集約のフォルダの中にだけ見せて作れる。別の集約 ・ services/ からの書き換えは E0616 で止まる（実測）'),
 ('値オブジェクトの置き場所', 'crate の直下', 'values/', '利用者「そっちの方が分かりやすい」。値オブジェクトは自分の new で作られ、ほかから中身に触らないので、分けても失うものが無い'),
 ('エンティティの置き場所', '（決めていなかった）', 'aggregates/<集約>/entities/。2つ以上の集約が持つもの（任せる先）だけ aggregates/entities/', '集約の外から勝手に作れないようにする（実測 E0624）。利用者「①でお願いします」'),
 ('session の依存', 'どこにも依存しない', 'session → declaration（目的の参照 purpose_ref.rs を declaration に置く）', 'session と change の両方が目的の参照を持つ。declaration は中心で、ほかの domain の crate も依存している'),
 ('MCP の入口', '無い（ボード Q3 は「駆動する側は CLI ・ MCP」なのに、arch.html に無かった）', 'cli の crate に2つ目の実行ファイル src/bin/mcp.rs を足し、compose.rs は lib から2つで使う', 'ボード Q3 に合わせる。起動時の結合が1か所のまま。原典：「Other executables can be placed in src/bin/」（Cargo Book Package Layout）'),
 ('システムテストの置き場所', 'tests/system.rs', 'tests/system/main.rs ＋ trace.rs', '1つの結合テストの crate の中をモジュールに分け、追跡の検査を同じ crate に置く'),
 ('test-suite/ の中', '約21場面の JSON', 'scenarios/*.json ・ trace.json ・ untested.json', 'qa-advisor の追跡の持ち方'),
]
