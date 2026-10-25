# Semantic Read ・ Semantic Write と search：下書き

## 利用者の言葉（そのまま）
- 「SemanticReadやSemanticWriteで重要なのはx-promptへのアクセスですよ。ここもないがしろにできる部分ではないですし、ここへはReadとWriteへの参照があって、その項目へのインスタンスの値への操作ができるのと、加えてx-prompt自体への更新や削除、新規作成にも使用できるなどが必要です。そのときはJsonPatchなども使用できると思います。」
- 「Semantic Read と Semantic Write というのはx-promptを読み、aiが読み書きを人間の都度の指示がなくてもループ的にできるようにしたものだと思ってください。」
- 「JMESPathって何ができるの？？」「これってインスタンスの中の情報を検索するときは？」

## 答えの下書き

### 1 Semantic Read（AI が書く前に読む）
- 入力：インスタンスの id ・ kind と、欄の位置（JSON Pointer。Concrete の書く単位）
- 返すもの：その欄の x-prompt（read の means ・ shown ・ parse ・ when と、write の rule の並び）、いまの値、rule の reads が指す欄の値（x-relationship でたどった先も読んだ欄だけ）、その欄の rule ごとのいまの検証結果
- 位置は2つに分ける：スキーマの中の位置（schemaLocation）とインスタンスの中の位置（instanceLocation）。どちらも JSON Pointer

### 2 Semantic Write（AI が書き、基盤が確かめる）
- 入力：JSON Patch（RFC 6902）。path は JSON Pointer。セッション ID と意図を一緒に受け取る（いまの「インスタンスを書く」と同じ）
- 基盤は Patch を当てた後の姿で、その欄の rule（x-prompt の write）と Concrete の JSON Schema を検証し、rule ごとの結果を返す
- 決定権が人の欄は、AI の Patch を断る（I の決定権）
- AI は violated ・ error が無くなるまで Read → Write を繰り返す。決めきれないものは needs で親に返す（人の都度の指示なしに回る）

### 3 x-prompt 自体の編集
- Concrete の文書への JSON Patch で、x-prompt を作る ・ 変える ・ 消す
- 基盤は Patch の後の Concrete を Abstract に照らして検証する（Validator）。いまの「意味を宣言する」に入れる

### 4 search
- 探す単位：種類 ・ 状態と判定の結果 ・ term ・ x-relationship ・ 拡張項目 ・ 欄の値
- 欄の値の条件は JSON Schema で書く（match）。「この JSON Schema に合うインスタンスを探す」。結果の位置は JSON Schema の出力の instanceLocation（JSON Pointer）
- JMESPath ・ JSONPath は基盤の契約に入れない（原則3：出力の形を JSON Schema で公開する、原則2：標準は JSON Schema だけ）
- 結果には探した範囲を付ける（I42）

### 5 位置の決まり
- 欄の位置はすべて JSON Pointer（RFC 6901）。存在しない欄と配列の `-` の扱いを決める：Read で存在しない欄は「値が無い」と返し、Write の Patch の add だけが `-` を使える

### 6 ポートとの対応
- 駆動する側のポート：「インスタンスを書く」を Semantic Write、「探して取り出す ・ 案内する」の案内を Semantic Read に名前を改める。x-prompt の編集は「意味を宣言する」
EOF
