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
ENTRY = [
 ['ドメインモデルを書く（値オブジェクト ・ 集約 ・ 不変条件）', 'model.json（ボードの measured/core/model/）→ ' + L('LD2wfboxHLbFP6KvybezwE', 'ドメインサービスの分け直し') + ' → ' + L('TPUCAGqmJ23JMhSXWnbAcw', 'ディレクトリ構成と CODING.md')],
 ['Abstract の13ファイルを書く', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体') + '（5節）→ ' + L('BcTSrcczNGLWeezwUitexr', 'コマンドのシミュレーション') + '（procedure ・ query）→ ' + L('XSP4EXyAP9bKRBr6DyGrKv', 'JSON Schema の能力の使い方')],
 ['コマンドとポートを書く', L('7hZXTxh89Ce8Cs4uJnHQAu', 'ポートの置き場所') + ' → ' + L('BcTSrcczNGLWeezwUitexr', 'コマンドのシミュレーション') + ' → ' + L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write') + ' ・ ' + L('J4e6mDbKwUaBoDsmSHjHeV', 'Semantic Search と用語')],
 ['手元の DB（sqlite）を書く', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体') + '（4節の DDL と ER）→ ' + L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション') + ' → ' + L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型')],
 ['ai-cli と agents-contract を書く', L('LTNMGFUXwuXFw9Pr41EgMj', 'AI ツールの調査') + ' → ' + L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型') + ' → ' + L('4AArrwWCGdnujXn4etBpdA', 'Agents 契約のシミュレーション') + '（一部古い）'],
 ['Renderer と既定のビューを書く', L('No2jxMJzWphQXry5mJFgH8', '既定のビューのデザインシステム') + ' → ' + L('8TTxF5B5f2wSVXNPUvYozy', 'x-prompt とビューの完成イメージ') + '（一部古い）'],
 ['試験と追跡の検査を書く', 'CODING.md の7節 ・ 9節 → ' + L('3SeEdiiBwRmYLbrZAa699T', '基盤のテスト計画') + '（一部古い）→ ' + L('GfD1nrpKqmr3EP4RQKbWzh', '基盤の核') + '（場面70）'],
]
REF = [
 [L('TEX6vLna45nG38koC7Tfc1', 'ACDR 0201 Semantic Agent Schema の土台'), '土台の決定（ドメインサービス4 ・ コマンド ・ 記録の置き場所 ・ 消さない ・ 既定のビュー）の変更前と変更後'],
 [L('4XQfGGa6TJEf7N2xW285rA', 'ACDR 0202 ディレクトリ構成とコーディング規則'), 'crate の中の分け方と CODING.md'],
 [L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体'), '全体の入口。ドメインモデル ・ コマンド ・ DDL と ER ・ Abstract の13ファイル'],
 [L('TPUCAGqmJ23JMhSXWnbAcw', 'ディレクトリ構成と CODING.md'), 'ファイル1つずつの置き場所 ・ 実測 ・ 原典の引用'],
 [L('YcHWX4NkaJCXPCgZR5iQaN', '基盤のアーキテクチャ'), '依存の辺（4節）・ 図 ・ 外の情報を使うドメインサービスの分担'],
 [L('LD2wfboxHLbFP6KvybezwE', 'ドメインサービスの分け直し'), 'どの処理が集約の操作 ・ ファクトリ ・ ドメインサービスか'],
 [L('7hZXTxh89Ce8Cs4uJnHQAu', 'ポートの置き場所'), '駆動される側のポート13と入口12'],
 [L('BcTSrcczNGLWeezwUitexr', 'コマンドのシミュレーション'), 'セマンティック7つの procedure と、20段の事前状態 ・ 事後状態'],
 [L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write'), 'read と write の入出力、検査①②、Patch の op'],
 [L('J4e6mDbKwUaBoDsmSHjHeV', 'Semantic Search と用語'), '探す条件（JMESPath）・ 結果 ・ 出どころの型と用語'],
 [L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション'), '手元の DB の記録と、コミットするまとめの流れ'],
 [L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型'), '4つの型 ・ context の列 ・ AI ツールごとの手段とキャッシュの実測'],
 [L('No2jxMJzWphQXry5mJFgH8', '既定のビューのデザインシステム'), '既定のビュー3つの完成イメージ ・ トークン ・ 部品'],
 [L('LTNMGFUXwuXFw9Pr41EgMj', 'AI ツールの調査'), 'ai-cli のアダプターの事実（各 CLI の呼び方）'],
 [L('XSP4EXyAP9bKRBr6DyGrKv', 'JSON Schema の能力の使い方') + ' ・ ' + L('BH7hvc6RCqEzvLT7ru86Fh', '読み方'), 'Abstract の13ファイルを書くとき'],
 [L('PYFjdKyGYcatEdBuSFy9ET', 'ACDR 0200 architecture-lint'), '依存の向きの検査の規則の形とコマンド'],
]
PART = [
 [L('3SeEdiiBwRmYLbrZAa699T', '基盤のテスト計画'), '層ごとの方針は使える', '置き場所 ・ 命名 ・ 追跡 ・ 完了基準は CODING.md の7節 ・ 9節が新しい'],
 [L('GfD1nrpKqmr3EP4RQKbWzh', '基盤の核'), '場面70 ・ 84回の実行の結果', '「変更の記録を確定する」と annotations'],
 [L('FCnpeA6s132k9tKpui15CA', '変更の記録を基盤が持つ理由'), 'なぜ記録するか', '流れの4 ・ 5（記録をコミット ・ その場でまとめる）'],
 [L('WNzbtyyi2xigs1DXzEBp8n', '変更の記録の契約'), '場所の指し方（pointer と find）', '「記録を足したコミットの中で引く」'],
 [L('HpSCtBAVfJMrQXN5scVZWS', '変更の場所の指し方'), '実測の結果', '「承認したらコミット」'],
 [L('SNeEvbxWSN8LNJnokciLV4', '変更の記録の状態'), '前後を受け取るだけで差分を作らない境界', '「消した ID の履歴」'],
 [L('Wso2J5PQSppkqxkh2AtwwH', '変更の記録と判定の記録の完成イメージ'), '判定の記録の形', '変更の記録の置き場所'],
 [L('5Hk6aKs7ApaFZawk1bonhU', 'ACDR の Concrete スキーマ'), 'Concrete の書き方の例', '$schema の名前は確かめてから使う'],
 [L('Jhj2PvMe53KNCHVGHXWR5V', 'ACDR の Concrete を作って使う'), 'U1 ・ U5 の流れの例', 'annotations'],
 [L('GRqKKpvcij9JmhC26VczG1', 'Abstract と Concrete の完成イメージ'), '形の例', 'annotations ・ URN の :core:（いまは :base:、13ファイル）'],
 [L('8TTxF5B5f2wSVXNPUvYozy', 'x-prompt とビューの完成イメージ'), 'x-prompt の形', 'ビューの対象に Abstract のファイルが無い'],
 [L('AjvjUjn2Z46r7V5v39A54s', '任せる先と呼び出しの完成イメージ'), 'x-delegates と呼び出し（call）の形', 'AI の判定の流れ（いまは semantic-judge → validate）'],
 [L('4AArrwWCGdnujXn4etBpdA', 'Agents 契約のシミュレーション') + ' ・ ' + L('3hwMTCMcQVQzKcaFdak1DQ', 'agents の呼び出しと注入'), 'agents-contract の受け渡しの例', '10/20 の版。本文は読んでいない'],
 [L('Xib2xbrqj1aEF562okQ9KU', '決まりの書き方と穴の確かめ方'), 'rule の書き方', '細部はボードの Q7 の最新を正とする'],
 [L('CdRCEdAdiSnUrBSc3svC5j', 'ボード（構想 v0.3 から）'), '論点の一覧', '10/20 の画面の写し。正本は手元の .brainstorming/semantic-contract-core.json'],
]
OLD = [
 [L('QtFMd4TFHwsksNqYAeWNg6', 'search の完成イメージ'), 'Semantic Search と用語'],
 [L('7JPPsCyqFQH4xodTotDKet', 'Semantic Agent Schema のページ一覧'), 'この索引と決めたことの全体'],
 [L('4fh7NtGf8wXyca29S1FeGi', '基盤の核の全容') + ' ・ ' + L('S1m24U6u8LhsU5QR1BJXch', '基盤の核の地図'), '決めたことの全体と model.json'],
 [L('56vhEiFptg1DMZoX88mq3K', 'どこに何を書くか'), '決めたことの全体の5節'],
 [L('KQ4KMPPaa5ZQUHGCFnB8Yv', '何が DB で何がファイルか') + ' ・ ' + L('E1wptU6KDXqMYgdHcB2vqw', '変更と DB の設計の点検'), '決めたことの全体の4節と変更のまとめのシミュレーション'],
 [L('2417p3Tk1mNswHFQMAwfrH', '変更の記録の活用パターン'), '変更のまとめのシミュレーション'],
 [L('ESKi4yMh3Zc7bVUESvaaWu', '変更を見せる（試作）'), '既定のビューのデザインシステム'],
 [L('DwfKjNW9nmPdg9m4GFRjHG', 'セッションの組み方') + ' ・ ' + L('M3zR2UPwF7NFP7219UjvSZ', '目的ごとのセッション'), '子のセッションの型'],
 [L('78cZGovzmkqKvpeCH2izXy', '2つの判断の完成イメージ'), 'ACDR 0201'],
 [L('UcziDsYkZyfS8uhV187QLa', '核の作業の整理') + ' ・ ' + L('DTRhVPR4UVbpdBaoHNVpBa', '核のレビューの統合') + ' ・ ' + L('SF6pDvnF3cH9dFGUqvvZpQ', '第2回') + ' ・ ' + L('GNJL5qY7fVP9qVpWvmunFC', 'I46 の変更前と変更後'), 'model.json の変更一覧に反映済み'],
 [L('6KAjMRraHNv1eLdcLSnvMq', 'Abstract の確かめ') + ' ・ ' + L('LBLn1Jn6QcKXpq3LBkRWjA', '根幹からの整理'), 'ボードの Q0 ・ Q6 と ACDR 0201'],
 [L('X8zEnVa4FTBMvTqNjGDtCT', '判定用の CLI の出どころ') + ' ・ ' + L('ERcJVf6Q5y3iQpceEg7c4i', 'agent の定義の置き場所') + ' ・ ' + L('17qQmq3pBdFUrzJxH7LGqd', '外の検査の置き場所') + ' ・ ' + L('R4vGMFr5cfjXZoUgzjqzRF', 'agents の見出し'), 'ボードの Q3 ・ Q7（x-delegates の judge）'],
 ['Q0 ・ Q1 ・ Q3 ・ Q8 ・ Q9 の変更差分（' + ' ・ '.join(L(i, n) for i, n in [('9SzbnD8ZAozFN3LiJQTFiF','Q0'),('HWR1uvPsPTmpd9qcBuiSRq','Q1'),('UoM9gCcp6BAxTdvdpwNBNa','Q3'),('9kN1GGPKDBdZjYwwuZUHqU','Q8'),('EStF6LYYmMe9vVjQ8EKMFH','Q9')]) + '）', 'ボードの正本に反映済み'],
 [L('T14FitHUWgt2oeJAYpD9NS', '依存の向きの検査ツール（ボード）') + ' ・ ' + L('ARtxyA2ev2Uo7Qr14c4kLU', 'ACDR 0199'), 'ACDR 0200 と architecture-lint'],
 [L('RJgEw765n5vNrLinAbKUod', 'Semantic Agent Schema（10/20）') + ' ・ ' + L('HJizdkQAf2wW9B26ZajHZC', '今の実装との比較を外す') + ' ・ ' + L('PunhBaaRr3zUDsQY2UN9jV', '論点の立て方') + ' ・ ' + L('8T4pVpD5C8uFEFxgEc5oGR', 'ボードの食い違いの一覧') + ' ・ ' + L('B9PgGRPmfd79bdK4RMVKGB', '決定の出どころの洗い直し') + ' ・ ' + L('C8EbF2b44k9h47irRrUUvq', '進み具合'), 'ボードの正本（途中の整理）'],
]
body = f'''<main>
<h1>参照する頁の索引</h1>
<p class="ans">実装は目的ごとに進める。目的ごとの入口から読み始め、「参照する」の頁を正とする。「部分的に参照する」は古い所を避けて使い、「経緯だけ」は読まない</p>
<h2>1　目的ごとの入口</h2>
{table(['目的', '読む順'], ENTRY)}
<h2>2　参照する（最新）</h2>
{table(['頁', '何のために読むか'], REF)}
<h2>3　部分的に参照する</h2>
{table(['頁', '使える所', '古い所'], PART)}
<h2>4　経緯だけ（読まない）</h2>
{table(['頁', '置き換えた先'], OLD)}
<p class="meta">仕分けの基準：ACDR 0201 ・ 0202、決めたことの全体、ボードの正本（.brainstorming/semantic-contract-core.json）、model.json。手元に元のファイルがある頁は本文を確かめた。10/20〜10/22 の手元に無い頁（Agents 契約 ・ Q の差分など）は本文を読まず、題と置き換えの関係で仕分けた。10/17 より前の頁は対象にしていない</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
