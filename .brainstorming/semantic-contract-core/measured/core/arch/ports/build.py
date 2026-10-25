"""ポートの置き場所とドメインサービスの頁（arch/ports/index.html）を組む。図は ports.svg ・ crates.svg（class で書き design-svg resolve を通したもの）。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>ポートの置き場所</title>')
fig = lambda n: '<div class="fig">' + open(os.path.join(H, n)).read() + '</div>'
def table(head_, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{h}</th>' for h in head_) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
import json
m = json.load(open(os.path.join(C, 'model', 'model.json')))
outs = [[p[0], p[1]] for p in m['ports']['outbound']]
ins = [[p[0], ' ・ '.join(p[1])] for p in m['ports']['inbound']]
body = f'''<main>
<h1>ポートの置き場所とドメインサービス</h1>
<p class="meta">利用者の問い（ポートは domain か application か ・ ドメインサービスは1つの業務領域に属するか ・ その定義）への答え。ddd-advisor と反証役のレビューを、ボードの決着に照らして1つにした。ACDR 0201 で model.json ・ arch.html に入れた</p>
<p class="ans">駆動される側のポート{len(outs)}は application の ports/ に置く。domain のドメインサービス4つは、値を受け取って値を返す関数にする</p>
{fig('ports.svg')}
<h2>1　決め手</h2>
{table(['前提（原文）', '出どころ', 'この答えへの帰結'], [
 ['「同じ入力から同じ結果を返し、別の言語の実装が使える適合の試験を持つ」', 'ボードの決着 Q0 #14（premises/board-answers.json）', 'ドメインサービスが計算の途中で trait を通して入出力を起こすと、結果がアダプターの振る舞いに依る'],
 ['「there is an application on the inside communicating over some number of ports with things on the outside」', 'Cockburn hexagonal-architecture:1498-1500', 'ポートは内側の六角形の境界に在る。この記事は domain と application を分けていない'],
 ['「eliminate any dependency on infrastructure, user interface, or even application logic that is not business logic」', 'Evans DDD Reference p.10 Layered Architecture', '読む ・ 書くの順序を持つのは application。domain は業務の計算だけを持つ'],
])}
<h2>2　ドメインサービスとは</h2>
{table(['問い', '答え', '出どころ'], [
 ['定義', '「When a significant process or transformation in the domain is not a natural responsibility of an entity or value object, add an operation to the model as a standalone interface declared as a service.」', 'Evans DDD Reference p.14 Services'],
 ['何に属するか', '「State these assertions in the ubiquitous language of a specific bounded context.」── 区切られた文脈に属する。業務領域（subdomain）に属するとは書いていない', '同上'],
 ['基盤では', '基盤の内側は1つの区切られた文脈（Q9 #11）。crate は、model.json が各ドメインサービスに付けた業務領域で置く。ドメインサービスは4つ（Resolver ・ Validator ・ Renderer ・ Session Selector）で、ほかは集約の操作 ・ ファクトリ ・ 値オブジェクトが持つ（ドメインサービスの分け直しの頁）', 'premises/board-answers.json:293 ・ model.json domain_services'],
])}
<h2>3　外の情報を使うドメインサービス</h2>
{table(['ドメインサービス', '外の情報の受け取り方', '前提'], [
 ['Semantic Schema Resolver', 'application が文脈のリポジトリの契約 ・ glossary と、閉じた一覧にある公開された言葉の版を先に読んで渡す。Resolver は渡された中だけで解決し、無いものは断る', 'Q4 #9「検証の途中で、$ref が指す外のスキーマを取りに行かない」'],
 ['Semantic Schema Validator', '2段で動く。①依頼の一覧（呼び出し call）を値で返す ②application が AI に semantic-judge の procedure を渡し、返った答えを5つの値にまとめる（割れたら unverified）。rule の答えどうしに連鎖があれば、application が①②を繰り返す', 'I14 ・ C60'],
 ['Session Selector', 'application がトランザクションを取り、手元の索引 ・ AI ツールの答え ・ 設定を読んで渡す。返った「再開する ID」か「新しく立てる（閉じる ID）」を、セッションの集約のコマンドで書いてコミットする', 'boundary「基盤は起動の前に問われて、再開する ID か「新しく立てる」を返すだけ」'],
 ['Semantic Schema Renderer', 'application が描くインスタンス ・ 状態 ・ ビュー（文脈の Concrete のビューか、Abstract に同梱した既定のビュー）を渡す。書き出しは application が「描画したものを書き出す」ポートで行う', 'I26'],
])}
<h2>4　駆動される側のポート（{len(outs)}。application の ports/）</h2>
{table(['ポート', '最初のアダプター'], outs)}
<h2>5　駆動する側の入口（{len(ins)}。inbound）</h2>
{table(['入口', '使う場面'], ins)}
<p class="meta">AI が行う操作は、入口のセマンティックのコマンドが procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返し、AI が生成した入力を機械的なコマンドが適用する。消すコマンドは無い</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body)
print('ok')
