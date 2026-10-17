---
name: brainstorming
description: 決めることがある問い（方針 ・ 設計 ・ 構成をまだ決めていないとき）に論点を立て、案を出して反証し、残った1つを完成イメージ付きの答えにして、利用者が承認 ・ 差し戻し ・ 書き足しだけで決められるボードを作るときに使う。ボードは JSON のインスタンスで持ち、画面はサイドバー ・ ダッシュボード ・ AI に聞くチャットを持つ。
license: MIT
---

# 反証で絞った答えを、承認だけで決められるボードにする Skill：brainstorming

## 目的

決めることがある問いに論点を立て、案を出し、**提示の前に自分で反証して1つに絞り**、利用者が承認だけで決められるボードを作る。利用者の判断の目的は、AI が作ったもののうち自分の意図からずれた所だけを早く見つけて正すことである。承認する人の問いは「この答えを、このまま進めてよいか」の1つだけにする。

**進み具合は決着の数で測る。** 案の数でも論点の数でもない。

## 役割

- 論点を3〜5件立て、依存の順を自分で持ち、いま見る論点を示す
- 案を複数出し、提示の前に反証して1つに絞る。外した案は理由と一緒に残す
- 答えの直下に完成イメージ（図 ・ 画面 ・ コード ・ 差分 ・ フォルダ構成 ・ 実行例）を置く
- 事実は自分で調べ、根拠に出どころを付ける。利用者に渡すのは決定だけである
- 回答を受けて回を進め、前の回から変わったところを理由と一緒に示す

## 入力の想定

| 受け取る情報 | 解釈 ・ 既定値 |
|---|---|
| テーマ | 1〜2文で確かめてから始める。論点はこちらで立てる |
| 回答（承認 ・ 差し戻し ・ 書き足し） | 画面から送られ、`.brainstorming/<ボード>/answers/` に1件ずつ残る。チャットで「回答した」と言われたら読む |
| データの置き場所 | リポジトリの根の `.brainstorming/`。ボードの本体は `.brainstorming/<ボード>.json` |

## 実行手順

使うのは、この Skill のフォルダの実行ファイル `scripts/bin/brainstorming` だけである。ボードの JSON は手で書き換えず、`update` で直す（書いたその場でスキーマの検査が走る）。どの操作も JSON を1つ返し、終了コードは 0 正常 ／ 1 不合格 ・ 失敗 ／ 2 使い方の誤りである。

```
scripts/bin/brainstorming            操作の一覧
scripts/bin/brainstorming --version  版
```

### Step 1: ボードを作る

ボードの名前（小文字の英数字とハイフン）を決め、スキーマから作る。

```
scripts/bin/brainstorming create --schema brainstorming/board@v1 --path .brainstorming/<ボード>.json
```

未記入の欄が x-prompt と一緒に返る。欄の書き方は `prompt` で読み、その決まりに従って値を決める。

```
scripts/bin/brainstorming prompt --schema brainstorming/board@v1 --property /topics
```

### Step 2: 論点を立て、案を反証して答えを1つにする

論点ごとに、問い ・ 答え（1文と詳しい決まり）・ 完成イメージ ・ 置いた前提 ・ 確かめてほしい箇所（3件まで）・ 根拠 ・ 採らなかった案を書く。前提にした論点は `depends` に書く。書き手の検証の記録（通過した案 ・ 扱わない範囲 ・ 外の作業）は `verification` に書く。画面には出ない。

```
scripts/bin/brainstorming update --path .brainstorming/<ボード>.json --patch '<JSON Patch>'
```

- 案が最初から1つなら、論点の立て方が誤っている。複数出して反証し、残った1つを答えにする
- 根拠には、答えのどこを支えるか ・ もとにしたこと ・ 出どころの3つを書く。原典の引用は原文のまま書く
- 完成イメージは、在る種類だけがタブになる。表は入れない。変化の要約は差分か図で見せる
- 図の SVG は、書いた側で検査してから入れる

### Step 3: 出す前に検査する

```
scripts/bin/brainstorming inspect --board .brainstorming/<ボード>.json
scripts/bin/brainstorming check --dir .brainstorming
```

inspect は形では見られない決まり（1文は120字まで ・ 答えが指す論点は depends に書く ・ いま答える論点には完成イメージと試す行き先がある ・ いま見る論点は前提が決着している ・ 一度に開く論点は5件まで）を見る。check は形と参照を見る。どちらも0件にしてから出す。check の終了コード 0 は実行できたことだけを示すので、`instances` の `errors` と `findings` で合否を見る。

### Step 4: 回として出し、画面で見せる

```
scripts/bin/brainstorming advance --board .brainstorming/<ボード>.json
scripts/bin/brainstorming render --dir .brainstorming
scripts/bin/brainstorming serve --dir .brainstorming --port 0
```

advance は回の記録 `rounds/<回>.json` を残す（書き換えない）。2回目からは、前の回と欄ごとに比べて「この回で変わったところ」を書き、差し戻しの言葉を理由に入れる。前の回で承認された論点は決着になる。render は全部のボードとダッシュボードを `.brainstorming/site/` に描く。serve は出力の `url` を利用者に渡す（合言葉付きの URL で、最初の1回だけ使う）。

### Step 5: 回答を受けて、直して、次の回を出す

回答は `.brainstorming/<ボード>/answers/` にあり、いまの回への回答を読む。

| 回答 | すること |
|---|---|
| 承認（approve） | その論点の `status` を `settled` にする。前提がすべて決着した論点を `open` にし、`queue` に、なぜいま開いたかと一緒に並べる |
| 差し戻し（return） | まず既存の答えを直せるかを確かめ、「どこが違うか」の所だけを直す。答えが動いていない欄は書き直さない |
| まだ答えない（hold） | その論点は変えない |

直したら Step 3 から繰り返す。advance は、承認と差し戻しの言葉を読んで、この回で変わったところに理由を付ける。

### Step 6: 決着させる

答えが下流の論点か外の作業で使われたら決着にしてよい。答えられる問いが尽きたら、ボードの `status` を `done` にする。止めるなら `hold` にする。保留は正当な着地点である。

## 出力形式

ボード1枚（`.brainstorming/<ボード>.json`）と、描いた画面（`.brainstorming/site/<ボード>.html` ・ `dashboard.html`）。利用者には次を伝える。

| 伝えること | 書き方 |
|---|---|
| 画面の場所 | serve の `url`、または `site/<ボード>.html` |
| いま答える論点 | 論点の id と名前、なぜいま開いたか |
| この回で変わったところ | 何か所か。中身は画面が示すので、チャットに並べない |
| 検査 | inspect と check が0件だったこと |

## ガードレール

- **案を並べて選ばせない**。選ばせるのは判断を利用者に移すことで、承認ではない
- **事実を利用者に聞かない**。調べれば分かることは調べて根拠に書く
- **外した案を黙って消さない**。`rejected` に案と何が壊れるかを残す
- **ボードの JSON を手で書き換えない**。`update` で直す。手で直すと、書いたその場の検査が行われない
- **回の記録 `rounds/` と回答 `answers/` を書き換えない**。比べる相手と利用者の言葉が変わる
- **ボードのフォルダの外を書き換えない**。画面の「AI に聞く」から呼ばれたときは、serve が作業の前後を比べて、外の変更を利用者に知らせる
- **AI の「検査した」で合格としない**。inspect と check の結果で決める
- **扱わない範囲に扱いを付けずに承認を求めない**。`verification.out_of_scope` に、対象外 ・ 後で決める ・ 解消済みのどれかを付ける

## 参照

| 場所 | 中身 |
|---|---|
| `assets/board.schema.json` | ボードの形（brainstorming/board@v1）。論点 ・ 完成イメージ6種類 ・ 変わったところ ・ 最近の動き |
| `assets/answer.schema.json` | 回答の形（brainstorming/answer@v1）。差し戻しは「どこが違うか」が必須 |
| `assets/round.schema.json` | 回の記録の形（brainstorming/round@v1） |
| `assets/dashboard.schema.json` | ダッシュボードの印の形（brainstorming/dashboard@v1） |
| `assets/chat.schema.json` | チャットの記録の形（brainstorming/chat@v1）。serve だけが書く |
| `assets/tokens.schema.json` | デザイントークンの正本の形（Design Tokens 形式） |
| `assets/pages/` | ボードとダッシュボードのページテンプレート |
| `assets/catalog/` | 部品のカタログ（parts.html ・ catalog.schema.json ・ catalog.css ・ catalog.js）と、トークンの正本（light.tokens.json ・ dark.tokens.json）から作った design-system.json |
| `scripts/bin/brainstorming` | この Skill の実行ファイル（advance ・ inspect ・ render ・ serve ・ tokens と、基盤の操作の受け渡し） |
| `scripts/bin/schema-driven` | 照合して入れた基盤のエンジン |
