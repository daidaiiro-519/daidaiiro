"""ポートの置き場所とドメインサービスの頁（arch/ports/index.html）を組む。図は ports.svg ・ crates.svg（class で書き design-svg resolve を通したもの）。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>ポートの置き場所</title>')
fig = lambda n: '<div class="fig">' + open(os.path.join(H, n)).read() + '</div>'
def table(head_, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{h}</th>' for h in head_) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
body = f'''<main>
<h1>ポートの置き場所とドメインサービス</h1>
<p class="meta">利用者の問い（ポートは domain か application か ・ ドメインサービスは1つの業務領域に属するか ・ その定義 ・ この構成にした見解）への答え。ddd-advisor と反証役（結論を渡していない）のレビューを、ボードの決着に照らして1つにした。arch.html はまだ直していない</p>
<p class="ans">駆動される側のポート12は application に置く。domain のドメインサービスは、値を受け取って値を返す関数にする</p>
{fig('ports.svg')}
<h2>1　決め手</h2>
{table(['前提（原文）', '出どころ', 'この答えへの帰結'], [
 ['「outbound adapter：いまはファイル。クラウドでは保存先のアダプターを足す。core は入出力に触らないので変えない（Q3）」', 'ボードの決着（premises/board-answers.json:160）', 'trait を通しても、ドメインサービスが計算の途中で入出力を起こすなら「触らない」にならない'],
 ['「there is an application on the inside communicating over some number of ports with things on the outside」', 'Cockburn hexagonal-architecture:1498-1500', 'ポートは内側の六角形の境界に在る。この記事は domain と application を分けていない'],
 ['「eliminate any dependency on infrastructure, user interface, or even application logic that is not business logic」', 'Evans DDD Reference p.10 Layered Architecture', '読む ・ 書くの順序を持つのは application。domain は業務の計算だけを持つ'],
])}
<h2>2　ドメインサービスとは</h2>
{table(['問い', '答え', '出どころ'], [
 ['定義', '「When a significant process or transformation in the domain is not a natural responsibility of an entity or value object, add an operation to the model as a standalone interface declared as a service.」', 'Evans DDD Reference p.14 Services'],
 ['何に属するか', '「State these assertions in the ubiquitous language of a specific bounded context.」── 区切られた文脈に属する。業務領域（subdomain）に属するとは書いていない', '同上'],
 ['基盤では', '基盤の内側は1つの区切られた文脈（Q9 #11）。だから業務領域をまたぐドメインサービスも許される。crate は、model.json が各ドメインサービスに付けた業務領域で置く', 'premises/board-answers.json:293 ・ model.json domain_services'],
 ['「変更を見せる」の中身', '並べる順（例外 → 目的 → 意図 → …）と例外の判定がドメインサービス。前後の取得 ・ 全件の検証 ・ 描画の呼び出しは使う場面の処理なので application へ移す', '反証役の指摘 s ・ Evans p.14「a significant process or transformation in the domain」'],
])}
<h2>3　計算の途中で外の情報を使う3つ</h2>
{table(['ドメインサービス', '変更前', '変更後', '前提'], [
 ['Semantic Schema Resolver', '途中で公開された言葉を取りに行く', 'application が文脈のリポジトリの契約 ・ glossary と、閉じた一覧にある公開された言葉の版を先に読んで渡す。Resolver は渡された中だけで解決し、無いものは断る', 'Q4 #9「検証の途中で、$ref が指す外のスキーマを取りに行かない」'],
 ['Semantic Schema Validator', '途中で AI と外のツールを呼ぶ', '2段にする。① 依頼の一覧（呼び出し call）を値で返す ② application が依頼する ③ 答えを5つの値にまとめる（割れたら unverified）。rule の答えどうしに連鎖があれば、application が①〜③を繰り返す', 'I14 ・ 呼び出し（call）の形は protocol が持つ'],
 ['再開か新規かを決める', '途中で手元の索引と AI ツールに問う', 'application がトランザクションを取り、索引 ・ AI ツール ・ 設定を読んで渡す。返った決定を索引へ書いてコミットする', 'boundary「基盤は起動の前に問われて、再開する ID か「新しく立てる」を返すだけ」'],
])}
<h2>4　比べた案</h2>
{table(['案', '判定', '理由'], [
 ['いまの案：各 domain の crate に ports.rs', '採らない', 'trait を通した入出力が domain の中で起きる。clippy の forbid は std の入出力しか止めない。Q3 の1行と食い違う'],
 ['混在：読むポート3つ（契約 ・ インスタンス ・ 判定の記録を読む）だけ domain（ddd-advisor の案）', '採らない', '読む先が中身で決まる読み出しを途中で行う理由は、全部を先に読むと遅いことだけ。前提は「関係の索引は、遅いと測れてから」なので、それまでは先に読んで渡せる。遅いと測れたら、そのときに置き場所を決め直す'],
 ['application の ports/（この答え）', '採る', 'domain は値の関数だけになり、Q0 #14「同じ入力から同じ結果」を型の上で守れる'],
])}
<h2>5　あわせて直す crate の依存</h2>
<p class="meta">反証役が、いまの辺の表は決着済みの前提に違反すると指摘した。ポートを application へ移すと、次のとおり循環を作らずに直せる</p>
{fig('crates.svg')}
{table(['直すこと', '変更前', '変更後', '前提'], [
 ['判定の記録の業務領域', '意味の検証へ移す（8節の問い1）', '変更の確かめと例外の判断のまま。8節の問い1は取り下げる', 'Q9 #6「本人の承認は中核「変更の確かめと例外の判断」の中にあり、判定の記録として残す」'],
 ['validation と change', 'change → validation（validation → change は禁止）', 'validation → change。Validator は判定の記録の使い直しと I33（変更の記録）に change の型を使う。change は検証を呼ばない', 'I33 ・ Validator の定義'],
 ['render', 'declaration ・ values だけ', 'validation ・ change ・ declaration に依存する', 'Q5 #1「Resolver は Validator と描画の両方が使う」'],
 ['values の crate', '循環を切るために置いた', '無くす。検証結果 ・ 状態 ・ 変更の記録の id は change、名前の型 ・ 対象は declaration（Abstract が決める形）、セッション ID は session', 'Evans p.15 Modules「Give the modules names that become part of the ubiquitous language」・ boundary「セッション ID はセッションの集約と手元のセッションの索引にだけ置き」'],
 ['instance → validation', '許す', '外す。作成のあとに検証するのは application の順序', 'Evans p.10'],
 ['答えを5つの値へ直す（I14）', 'アダプター agents-contract', 'validation（Validator の③）', 'Q9 #16「Agents 契約を腐敗防止層として基盤の側に置き、中核のモデルを守る」'],
 ['フォルダ名', 'crates/adapters ・ crates/entry', 'crates/outbound ・ crates/inbound', 'Q5 #9「英語はコードのフォルダ名 inbound ・ outbound と同じ」'],
 ['実装が無いポート3つ', '無い', '公開された言葉を取りに行く ・ 外のツールを呼ぶ ・ 描画したものを書き出す のアダプターを足す', 'model.json ports.outbound'],
])}
<p class="meta">業務領域1つに crate 1つは変えない。反証役は search ・ migration ・ publish をモジュールへ畳めると挙げたが、Q9 #11 はどちらでも満たし、「業務領域1つ ＝ crate 1つ」の1つの規則のほうが検査の表が1つで済む</p>
<h2>6　決めてほしいこと</h2>
{table(['決めてほしいこと', '私の見解'], [
 ['駆動される側のポート12を application に置き、domain を値の関数にする', '置く（この頁の答え）'],
 ['Validator の定義の文を、①依頼の一覧を返す ・ ③答えをまとめる の2段に書き直す（model.json の変更）', '書き直す。ドメインサービスの数は13のまま'],
])}
<p class="meta">承認されたら arch.html の4〜8節をこの頁の内容で直す。8節の残りの問い（スキーマの置き場所 ・ Q3 の置き換え）はそのまま残る</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body)
print('ok')
