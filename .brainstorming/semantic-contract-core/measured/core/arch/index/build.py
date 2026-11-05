"""実装で参照するアーティファクトの索引（arch/index/index.html）。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + 'main>*{min-width:0;max-width:100%}td a{overflow-wrap:anywhere}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>参照する頁の索引</title>')
A = 'https://claude.ai/artifact/'
L = lambda i, t: f'<a href="{A}{i}">{t}</a>'
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
IMPL = 'semantic-agent-schema/'
C_ = lambda s: f'<code>{s}</code>'
# ① 拠り所の順（上が下と食い違えば、上を正とする）
ORDER = [
 ['1', '実装：' + C_(IMPL) + ' のコード（' + C_('crates/') + ' ・ ' + C_('schemas/v1/') + '）・ シナリオ（' + C_('test-suite/scenarios/') + ' の39場面 ・ ' + C_('trace.json') + ' ・ ' + C_('untested.json') + '）・ ' + C_('CODING.md') + ' ・ ' + C_('architecture.json'), '実際の振る舞い。コマンドの引数は実装の --help を正とする。各ステップのコミットで、いつ何を決めたかが読める'],
 ['2', L('JWgrBUZPWX7Deg5CXPYRoz', '実装計画'), 'ステップの並び ・ ステップごとの受け入れ ・ 済んだステップのコミット'],
 ['3', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体'), 'ドメインモデル ・ コマンド ・ 手元の DB の DDL と ER ・ Abstract の13ファイル'],
 ['4', 'ボードの ' + C_('measured/core/model/model.json') + ' と ' + C_('measured/core/schemas/'), '業務領域 ・ 使う場面 ・ 不変条件 ・ ポートの正本'],
]
# ② 実装の拠り所：ステップごとに読む頁（監査で「そのまま使える」か「直せば使える」とした頁だけ）
EVERY = [
 ['どのステップにも', L('YcHWX4NkaJCXPCgZR5iQaN', '基盤のアーキテクチャ') + ' ・ ' + L('7hZXTxh89Ce8Cs4uJnHQAu', 'ポートの置き場所') + ' ・ ' + L('LD2wfboxHLbFP6KvybezwE', 'ドメインサービスの分け直し'), '依存の辺と図 ・ 駆動される側のポート13と入口12 ・ どの処理が集約の操作 ・ ファクトリ ・ ドメインサービスか（分け直しの頁は1節だけ。フォルダの木は実装の ' + C_('crates/domain/') + ' と CODING.md 3節）'],
]
STEPS = [
 ['0　検査の土台（済）', 'CODING.md 7節 ・ 9節 → ' + C_('test-suite/trace.json') + ' ・ ' + C_('scenario.schema.json') + ' → ' + L('PYFjdKyGYcatEdBuSFy9ET', 'ACDR 0200 architecture-lint'), 'ACDR 0200 は検査ツールの名前とコマンドだけ。規則の形は ' + C_('architecture-lint/README.md') + ' と ' + C_('rules.schema.json')],
 ['1　意味を宣言する（済）', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体') + '（5節）→ ' + L('XSP4EXyAP9bKRBr6DyGrKv', 'JSON Schema の能力の使い方') + ' ・ ' + L('BH7hvc6RCqEzvLT7ru86Fh', '読み方') + ' → ' + L('5Hk6aKs7ApaFZawk1bonhU', 'ACDR の Concrete スキーマ'), 'Abstract の13ファイルの形（実装では本体が方言 ' + C_('dialect.schema.json') + ' と語彙のメタスキーマ ' + C_('vocab-meta.schema.json') + ' に従う）・ Concrete の書き方の例（欄は title ・ description ・ x-prompt。x-prompt.read は shown ・ interpret ・ when）'],
 ['2　インスタンスを書いて検証する（済）', L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write') + ' → ' + L('Xib2xbrqj1aEF562okQ9KU', '決まりの書き方と穴の確かめ方'), '検査①② ・ Patch の op ・ ETag と --if-match ・ rule の書き方'],
 ['3a　変更を記録する（済）', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体') + '（4節の DDL と ER）→ ' + L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション'), '手元の DB の表と、記録の流れ'],
 ['3b　まとめをコミットする（済）', L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション'), 'record --summary ・ まとめの形 ・ worktree と branch での絞り込み'],
 ['4　AI と外のツールに判定させる（coding-agents と agents-contract を書く）', L('AjvjUjn2Z46r7V5v39A54s', '任せる先と呼び出しの完成イメージ') + ' → ' + L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型') + ' → ' + L('LTNMGFUXwuXFw9Pr41EgMj', 'コーディングエージェントの調査') + ' → ' + L('Xib2xbrqj1aEF562okQ9KU', '決まりの書き方と穴の確かめ方'), 'x-delegates（run.kind は judging と working。CLI の任せる先は argv。working だけ run.session。output は出力の details の形）・ 出力の形（Abstract の ' + C_('$defs/output-judging') + ' ・ ' + C_('output-working') + '）・ 判定ごとに新しいセッションで起動する型 ・ coding-agents のセッションの事実（再開 ・ 有効時間 ・ 履歴の割合）'],
 ['5　セマンティックのコマンドと MCP', L('BcTSrcczNGLWeezwUitexr', 'コマンドのシミュレーション') + ' → ' + L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write') + ' → ' + L('8TTxF5B5f2wSVXNPUvYozy', 'x-prompt とビューの完成イメージ'), 'セマンティック7つの procedure ・ 事前状態と事後状態 ・ x-prompt の形'],
 ['6　取り込み、探す', L('J4e6mDbKwUaBoDsmSHjHeV', 'Semantic Search と用語'), '探す条件（JMESPath）・ 結果 ・ 出どころの型と用語'],
 ['7a ・ 7b　置き換えて残す ・ 版を出す', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体'), 'superseded と x-relationship ・ published'],
 ['8　描画と既定のビュー', L('No2jxMJzWphQXry5mJFgH8', '既定のビューのデザインシステム') + ' → ' + L('8TTxF5B5f2wSVXNPUvYozy', 'x-prompt とビューの完成イメージ'), '既定のビュー3つ ・ トークン ・ 部品'],
 ['9　目的ごとのセッション', L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型'), '4つの型 ・ context の列 ・ コーディングエージェントごとの手段'],
 ['10 ・ 11　通しの受け入れ ・ 2つ目の題材', L('JWgrBUZPWX7Deg5CXPYRoz', '実装計画') + 'の受け入れの表', '──'],
]
# ③ 経緯（実装では読まない）：監査で「経緯として残す」とした頁
HIST = [
 [L('CdRCEdAdiSnUrBSc3svC5j', 'ボード（構想 v0.3 から）'), '論点の一覧と、決めるまでの回の記録。第96回の Q3 ・ Q0 ・ Q12 の書き換えは保留中で、正しさの根拠にしない', '実装 ・ 実装計画 ・ model.json'],
 [L('TEX6vLna45nG38koC7Tfc1', 'ACDR 0201 Semantic Agent Schema の土台'), '10/25 時点の土台の決定（ドメインサービス4 ・ コマンド ・ 記録の置き場所）となぜこの形か', '以後の変更は実装のコミット'],
 [L('4XQfGGa6TJEf7N2xW285rA', 'ACDR 0202 ディレクトリ構成とコーディング規則'), 'crate の中の分け方となぜこの形か', '実装の ' + C_('crates/') + ' と CODING.md'],
 [L('TPUCAGqmJ23JMhSXWnbAcw', 'ディレクトリ構成と CODING.md'), 'ファイルの置き場所の検討と実測（CODING.md は承認前の写し）', '実装の ' + C_('crates/') + ' と CODING.md'],
 [L('3SeEdiiBwRmYLbrZAa699T', '基盤のテスト計画'), '層ごとのテストの方針の検討', 'CODING.md 7節 ・ 9節と ' + C_('test-suite/')],
 [L('GfD1nrpKqmr3EP4RQKbWzh', '基盤の核'), '参照の実装（measured/core/ref）の場面70と実行の結果。「変更の記録を確定する」と DB の形は当時のもの', C_('test-suite/scenarios/') + ' と trace.json'],
 [L('GRqKKpvcij9JmhC26VczG1', 'Abstract と Concrete の完成イメージ') + ' ・ ' + L('Jhj2PvMe53KNCHVGHXWR5V', 'ACDR の Concrete を作って使う'), '形と使い方の例（annotations ・ URN の :core: は古い）', '実装の ' + C_('schemas/v1/') + ' と変更のまとめのシミュレーション'],
 [L('FCnpeA6s132k9tKpui15CA', '変更の記録を基盤が持つ理由') + ' ・ ' + L('WNzbtyyi2xigs1DXzEBp8n', '変更の記録の契約') + ' ・ ' + L('HpSCtBAVfJMrQXN5scVZWS', '変更の場所の指し方') + ' ・ ' + L('SNeEvbxWSN8LNJnokciLV4', '変更の記録の状態') + ' ・ ' + L('Wso2J5PQSppkqxkh2AtwwH', '変更の記録と判定の記録の完成イメージ') + ' ・ ' + L('2417p3Tk1mNswHFQMAwfrH', '変更の記録の活用パターン'), '変更の記録をファイルに置く案 ・ find と hunk ・ 判定を最後の1件だけ残す案', '手元の DB（' + C_('change_store.rs') + '）・ ' + C_('change.schema.json') + ' ・ ' + C_('judgment.schema.json') + ' ・ 変更のまとめのシミュレーション'],
 [L('78cZGovzmkqKvpeCH2izXy', '2つの判断の完成イメージ'), 'セッションと記録の置き場所の2つの判断（3節は案のまま終わった）', 'ACDR 0201 と子のセッションの型'],
 [L('4AArrwWCGdnujXn4etBpdA', 'Agents 契約のシミュレーション') + ' ・ ' + L('R4vGMFr5cfjXZoUgzjqzRF', 'agents の見出し'), '10/21 の Agents 契約の案（$defs/agents ・ model など外れた案を持つ）', '任せる先と呼び出しの完成イメージ'],
 [L('ERcJVf6Q5y3iQpceEg7c4i', 'agent の定義の置き場所') + ' ・ ' + L('17qQmq3pBdFUrzJxH7LGqd', '外の検査の置き場所'), '任せる先の置き場所の検討', '任せる先と呼び出しの完成イメージ'],
 [L('X8zEnVa4FTBMvTqNjGDtCT', '判定用の CLI の出どころ'), 'CLI の任せる先を残す結論。のちに x-delegates の argv として実装された', '実装の ' + C_('semantic-abstract-schema.json')],
 [L('M3zR2UPwF7NFP7219UjvSZ', '目的ごとのセッション') + ' ・ ' + L('DwfKjNW9nmPdg9m4GFRjHG', 'セッションの組み方'), 'セッションの分け方の検討', '子のセッションの型'],
 [L('QtFMd4TFHwsksNqYAeWNg6', 'search の完成イメージ'), '探すことの古い案', 'Semantic Search と用語'],
 [L('UoM9gCcp6BAxTdvdpwNBNa', 'Q3 の差分'), 'ボードの Q3 の第7版の案。ボードには当てず、後の決定（ACDR 0201 ・ ボード Q3）が置き換えた', 'ACDR 0201'],
 [L('T14FitHUWgt2oeJAYpD9NS', '依存の向きの検査ツール（ボード）') + ' ・ ' + L('ARtxyA2ev2Uo7Qr14c4kLU', 'ACDR 0199'), '検査ツールを内作する決定まで', '規則の正本は ' + C_('architecture.json') + '、ツールは ACDR 0200'],
 [L('DTRhVPR4UVbpdBaoHNVpBa', '核のレビューの統合') + ' ・ ' + L('SF6pDvnF3cH9dFGUqvvZpQ', '第2回') + ' ・ ' + L('LBLn1Jn6QcKXpq3LBkRWjA', '根幹からの整理') + ' ・ ' + L('HJizdkQAf2wW9B26ZajHZC', '今の実装との比較を外す') + ' ・ ' + L('PunhBaaRr3zUDsQY2UN9jV', '論点の立て方'), 'レビューと論点の立て方', 'model.json の変更一覧とボード'],
]
body = f'''<main>
<h1>参照する頁の索引</h1>
<p class="ans">実装の拠り所は、上から 実装 → 実装計画 → 決めたことの全体 → model.json と schemas の順。上と下が食い違えば上を正とする。各ステップは開発シミュレーターで実行結果を確かめ、利用者の OK でコミットする。2節の頁をステップごとに読み、3節の頁は読まない</p>
<h2>1　拠り所の順と、確かめる所</h2>
{table(['順', '拠り所', '何を決めているか'], ORDER)}
<div class="tw"><table><tr><th>確かめる所</th><th>何をするか</th></tr><tr><td>{L('9xBWifdT2tcV8LzzkeuG91', '開発シミュレーター')}</td><td>ステップごとに実際の実行結果を見せ、利用者の OK でコミットする。ステップごとに ACDR は作らない</td></tr></table></div>
<h2>2　実装の拠り所（ステップごとに読む頁）</h2>
{table(['ステップ', '読む頁（読む順）', '読む所'], EVERY + STEPS)}
<p class="meta">ステップの名前と済んだかは実装計画の5節による。この節には、監査で「そのまま使える」か「直せば使える」とした頁だけを置いた。頁の記述が1節の拠り所と違えば、拠り所を正とする</p>
<h2>3　経緯（実装では読まない）</h2>
{table(['頁', '何の経緯か', '今の決定の在りか'], HIST)}
<p class="meta">3節の頁には、すべて先頭に「実装の拠り所ではない」と注記した。今の決定と違う点がある頁は、その点を注記に1文で添えた。監査で「捨ててよい」とした頁（基盤の核の全容 ・ ページ一覧 ・ Q の変更差分など）は索引から外した。頁そのものは消していない</p>
<p class="meta">仕分けの基準：2026-10-26 の監査（実装 ・ 実装計画 ・ model.json と各頁の突き合わせ）</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
