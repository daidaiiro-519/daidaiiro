"""Semantic Agent Schema の実装計画（arch/plan/index.html）。目的ごとの段と、題材 Concrete（ACDR）での受け入れ。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '''main>*{min-width:0;max-width:100%}
.ph{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:12px 14px;display:flex;flex-direction:column;gap:6px}
.ph h3{margin:0;font-size:16px}.ph .uc{font-size:12.5px;color:var(--sub)}.ph table{font-size:13px}
.flow{display:flex;flex-wrap:wrap;gap:6px;align-items:center}.flow span{border:1px solid var(--line);border-radius:999px;padding:2px 10px;font-size:12.5px;background:var(--card)}.flow b{color:var(--accent)}
</style>'''
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>実装計画</title>')
A = 'https://claude.ai/artifact/'
L = lambda i, t: f'<a href="{A}{i}">{t}</a>'
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
# 段：番号, 目的, 使う場面, 作るもの, ACDR での受け入れ（利用者がすること → 期待）, システムテストの場面, 読む頁
P = [
 ('0', '検査の土台を作る', 'U8（CI）', 'cli/tests/system の走らせ方（test-suite の場面を1件ずつ）と trace.rs（追跡の検査）。model.json に状態を導く決定表の列を足す。段1〜3b で使うコマンドの名前と引数の最小の形を決める（ボードの Q4）。題材の置き場所 test-suite/fixtures/acdr/ と、「crates/ に題材の語が0件」の検査', '（基盤の外の準備）', '空の場面で system が走る。trace.json と untested.json の漏れ ・ 重なりを落とす', 'CODING.md 7節 ・ 9節'),
 ('1', '意味を宣言する：Abstract と Concrete', 'U1 ・ U23', '<b>スキーマ</b> semantic-abstract-schema.json ・ glossary.schema.json。<b>domain</b> declaration（Abstract ・ Concrete ・ glossary、rule と任せる先、雛形のファクトリ）と Resolver。<b>コマンド</b> create ・ get ・ patch（JSON Patch。スキーマにもインスタンスにも使う）・ validate。<b>ポート</b> contracts（fs）', '利用者が ACDR の Concrete を雛形から作り、patch で欄に x-prompt を書く → Abstract に照らして検証が通る。x-prompt の形が誤っていれば violated で場所が出る', '雛形を作る ・ Concrete を Abstract で検証する ・ 拡張項目を足す', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体') + ' 5節 ・ ' + L('XSP4EXyAP9bKRBr6DyGrKv', 'JSON Schema')),
 ('2', 'インスタンスを書いて検証する（機械の rule）', 'U5 ・ U6 ・ U7（欄と rule の部分。変更の記録は段3a）', '<b>domain</b> instance（集約とファクトリ）、validation の機械の rule と検査①②。<b>ポート</b> instances（fs）', '利用者が ACDR 0199 を作り、decision と why を patch で書く → 必須の欄が空なら violated、書けば機械の rule が pass。id や $schema を変える Patch は断られ、状態は変わらない', '作る ・ 欄を書く ・ 検査①で断る（unchanged）・ 検査②で violated', L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write')),
 ('3a', '変更を記録する（手元の DB）', 'U5 ・ U6 の記録の部分 ・ U16（人の判定）・ U13', '<b>スキーマ</b> change.schema.json ・ judgment.schema.json。<b>domain</b> change（変更の記録 ・ 判定の記録）。<b>ポート</b> change_records ・ judgments（意図の判定）・ sessions の索引（sqlite）、judgments の rule の判定（fs）。<b>コマンド</b> record ・ judge（人）・ history', '利用者が 0199 を書いたあと history を見る → 変えた欄と意図が手元の記録に出る。人が意図を承認すると判定の記録が残る', '記録 ・ 人の判定 ・ 経緯', L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション') + ' ・ 全体の4節'),
 ('3b', 'まとめをコミットし、ほかの変更を取り込む', 'U25 ・ U18', '<b>スキーマ</b> change-summary.schema.json。<b>domain</b> まとめのファクトリ。<b>ポート</b> changes（git ・ snapshot）・ change_summaries（fs）', '利用者がまとめを作ってコミットする → changes/&lt;id&gt;.json が1件でき、記録に無い書き換えは例外に出る。ほかの人のコミットを取り込むと、取り込んだあとの姿で全インスタンスが検証される', 'まとめ（例外あり ・ なし）・ 記録に無い書き換え ・ 取り込み', L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション')),
 ('4', 'AI と外のツールに rule を判定させる', 'U10 ・ U14 ・ U16（意図の判定 I33）', '<b>domain</b> Validator の2段（依頼の一覧を返す ・ 答えをまとめる）。<b>コマンド</b> semantic-judge。<b>ポート</b> judge（agents-contract ＋ ai-cli）・ tools（外のツールの validator）。システムテストは代役の AI', '利用者が ACDR の Concrete に「why が決定を説明しているか」の AI の rule を足し、0199 を検証する → needs が返り、AI の答えで pass か violated になる。0185 を意図に無く変えると、I33 の判定で例外に出る。x-prompt を変えると前の判定は効かなくなり、取り直せる', 'needs を返す ・ 答えをまとめる（割れた答えは unverified）・ 取り直す ・ 外のツール', L('AjvjUjn2Z46r7V5v39A54s', '任せる先と呼び出し') + '（一部古い）・ ' + L('LTNMGFUXwuXFw9Pr41EgMj', 'AI ツールの調査')),
 ('5', 'AI が手順に従って書く：セマンティックのコマンドと MCP', 'U5 ・ U6 ・ U2（AI から）', '<b>スキーマ</b> procedure.schema.json。<b>コマンド</b> semantic-read ・ write ・ create ・ judge の procedure。<b>入口</b> MCP（src/bin/mcp.rs）', '利用者が Claude Code に「0203 を書いて」と依頼する（CLI と MCP の両方で）→ AI が procedure に従って入力を作り、patch と validate を回して書き上げる。人は意図を承認するだけ', '各 semantic-* が procedure を返す ・ 入力のスキーマに外れた入力を断る ・ CLI と MCP で同じ結果', L('BcTSrcczNGLWeezwUitexr', 'コマンドのシミュレーション')),
 ('6', '既存のファイルを取り込み、探す', 'U17 ・ U28 ・ U26', '<b>スキーマ</b> query.schema.json ・ settings.schema.json。<b>domain</b> search（探す条件 ・ 出どころ ・ 範囲）。<b>コマンド</b> import ・ semantic-import ・ search ・ semantic-search。<b>ポート</b> settings（fs）', '利用者が既存の ACDR 178件を取り込む → 古い版の記録は下書きか violated で並ぶ（取り込みの試験として期待どおり）。「AI の許可を決めた記録と理由は？」と聞く → 出どころ付きで答えが返り、validate で出どころを評価し直せる', '取り込む ・ 下書き ・ 探す（where ・ select）・ 出どころの評価し直し ・ 探す範囲の宣言', L('J4e6mDbKwUaBoDsmSHjHeV', 'Semantic Search と用語')),
 ('7a', '置き換えて残す', 'U11', '<b>domain</b> superseded と x-relationship の検証', '利用者が 0199 を 0200 で置き換える → 0199 は superseded で残り、0199 の判定は効かなくなり、0199 を指す記録は壊れない', '置き換え ・ 指す側が壊れない', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体')),
 ('7b', '版を出す ・ 移す ・ 公開された言葉', 'U3 ・ U4 ・ U24', '<b>スキーマ</b> published.schema.json。<b>コマンド</b> release ・ migrate ・ semantic-migrate。<b>ポート</b> published（fs）', '利用者が Abstract の版を上げる → ACDR の Concrete は機械で直せる所が直り、残りが案内に出る。ほかの文脈の公開された言葉を版で固定して読める', '移す ・ 残りの案内 ・ 版で固定して読む', L('XmAraA9W1dZURaT3Mddx3H', '決めたことの全体')),
 ('8', '人が読む：描画と既定のビュー', 'U12 ・ U25', '<b>スキーマ</b> view.schema.json ・ catalog.schema.json ・ design-tokens.schema.json と views/ の既定のビュー3つ。<b>domain</b> render（ビュー ・ デザインシステム）と Renderer。<b>ポート</b> rendered（fs）', '利用者が 0199 を描く → 状態と rule ごとの結果が内容と一緒に出る。変更のまとめ ・ 判定は既定のビューで読め、ACDR の Concrete のビューがあればそちらで描く', '描く前の検査 ・ 既定のビュー ・ 差し替え', L('No2jxMJzWphQXry5mJFgH8', '既定のビュー')),
 ('9', '目的ごとのセッション', 'U27', '<b>スキーマ</b> session.schema.json。<b>domain</b> session と Session Selector。<b>ポート</b> sessions（sqlite）・ ai_sessions（ai-cli）', '利用者が目的 0203 のセッションを立て、翌日 ID で再開する → 同じ目的 ・ 親 ・ 役割で開いていて有効時間内なら再開、そうでなければ新しく立てる。子のセッションの型が記録される', '再開か新規か ・ 子の型 ・ 閉じる', L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型')),
 ('10', '通しの受け入れ', 'U0', '（足すものは無い）', '利用者が ACDR の新しい記録を、目的のセッションの中で AI に書かせ、検証 ・ 判定 ・ まとめ ・ 描画まで通す → 使っているインスタンスがすべて verified のまま使い続けられる', '全段の場面を逆順でも同じ結果', '—'),
 ('11', '2つ目の題材：Brainstorming のボード', 'U1 〜 U28 の回帰', 'brainstorming の board スキーマを Concrete として書き直す（入れ子 ・ x-prompt 110件 ・ 回と回答の参照）', '利用者がいまのボード5枚を取り込み、論点を書き、回を進める → 入れ子の欄と参照が検証され、既定のビューで読める', '入れ子 ・ 配列 ・ 多い x-prompt ・ 規模', '—'),
]
phases = ''.join(f'''<section class="ph"><h3>段{n}　{t}</h3><p class="uc">使う場面 {u}</p>
{table(['作るもの', 'ACDR での受け入れ（利用者がすること → 期待）', 'システムテストの場面', '読む頁'], [[w, a, s, r]])}</section>''' for n, t, u, w, a, s, r in P)
flow = '<div class="flow">' + ' → '.join(f'<span><b>段{n}</b> {t}</span>' for n, t, *_ in P) + '</div>'
body = f'''<main>
<h1>実装計画</h1>
<p class="ans">Semantic Agent Schema の基盤を、目的ごとの段で作る。各段は1つの目的のセッションで、題材の Concrete（ACDR）を使った受け入れ（利用者が実際に動かす）とシステムテストで終える。ACDR で全段を通したあと、2つ目の題材（Brainstorming のボード）で入れ子と規模を確かめる</p>
<h2>1　題材の Concrete</h2>
{table(['', 'ACDR（最初の題材）', 'Brainstorming のボード（2つ目）'], [
 ['実物', '記録 178件（古い版の記録を含む）。スキーマは20の欄で、配列の入れ子は2段まで。x-prompt は0件', 'ボード5枚。スキーマは入れ子（定義7）。x-prompt 110件'],
 ['試せること', 'Concrete を一から書く（U1 ・ U23）・ 欄の書き換えと検証 ・ AI の rule（理由が決定を説明しているか）・ superseded と置き換えの参照 ・ 実データの取り込みと探す（U17 ・ U28）', '入れ子と配列の深い欄 ・ 多い x-prompt ・ 回と回答の参照 ・ 規模'],
 ['なぜこの順', '形が軽く、宣言 → インスタンス → 記録の順を最初から最後まで通せる。x-prompt が0件なので、Concrete を一から書くことになる。実データがある', '毎日使っていて使い勝手の試験としての価値は高いが、形が重い（入れ子 ・ x-prompt 110件 ・ serve）。ACDR で土台が動いてから確かめる'],
 ['置き場所', 'test-suite/fixtures/acdr/。題材は例であって基盤の中身ではないので、crates/ に題材の語を入れない（grep で検査）', 'test-suite/fixtures/brainstorming/'],
])}
<h2>2　段の並び</h2>
{flow}
<p class="meta">依存の順：宣言（段1）→ インスタンス（段2）→ 記録（段3a）→ まとめ（段3b）→ AI の判定（段4）→ セマンティックと MCP（段5）。段6 〜 9 は段5 のあと、互いに独立（7b は 7a のあと）。段10 は全段のあと、段11 は最後。Abstract の13ファイルとポート13は、どれも作る段が1つ決まっている</p>
<h2>3　段ごとの中身</h2>
{phases}
<h2>4　どの段にも共通の進め方</h2>
{table(['こと', 'どうするか'], [
 ['1段 ＝ 1つの目的', '段ごとに目的のセッションを立てる。段の中で決めることがあれば、その段の ACDR に変更前と変更後を出してから実装する'],
 ['試験から書く', 'domain の値と集約は単体テスト（決定表 ・ 状態遷移）から書く。段の受け入れの流れを test-suite の場面にして、先に落ちることを確かめる'],
 ['終わりの条件', 'CODING.md の8段の検査が0件で通る ・ untested.json の行が減っている ・ 利用者が ACDR での受け入れを動かして確かめた'],
 ['受け入れ（UAT）', '段の終わりに、利用者が実際に CLI（段10 からは MCP も）で題材を動かす。手順と期待はこの頁の表のとおり'],
 ['AI の扱い', 'システムテストでは代役の AI を使い、本物の AI ツールは呼ばない。本物は受け入れのときだけ使う'],
])}
<h2>5　始める前に決まっていないこと</h2>
{table(['こと', 'いつ決めるか'], [
 ['model.json の states に、状態を導く決定表の列が無い', '段0'],
 ['ACDR の Concrete の名前と、x-prompt の中身', '段1（題材の側の決めごと）'],
 ['コマンドの名前と引数', '段0 で段1〜3b の分の最小の形を決め、段ごとに足す'],
])}
<p class="meta">参照：{L('3cTvkRsQ2YEqzywBoxkTgV', '参照する頁の索引')} ・ ACDR 0201 ・ 0202 ・ CODING.md</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
