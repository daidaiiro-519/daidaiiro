# 依存の向きの検査ツール：論点と答えの下書き

材料：research.md（7ツールの原文。引用は Q 番号）・ reviews/meta-thinking.json（論点の立て方）

## 目的（前提として置く）

ビルドツールが「書いていない依存を使う」と「循環」しか止めない言語で、リポジトリの開発者（人と AI ツール）と CI が、規則の表に無い依存の辺を、言語やアーキテクチャに依らない1つの規則の形で検出して止められるようにする。

最初の利用者：semantic-agent-schema の Rust 実装（.brainstorming/semantic-contract-core/measured/core/arch.html 4節の依存の表 ・ 6節の検査）。

## Q1 規則の形

問い：依存の規則を、どの形で書くか。

答え：要素を名前の付いた集まり（group）にまとめ、集まりの間で許す辺だけを書く許可リスト型にする。書いていない辺はすべて禁止する。

- group は名前と、要素の名前のパターン（完全一致か末尾の `*`）の並びを持つ
- allow は `from`（group）と `to`（group の並び）を持つ。同じ group の中の辺も、書かなければ禁止
- 辺の種類（Rust の normal ・ dev ・ build など）は区別せず、どの種類の辺も allow に無ければ違反（dev の辺の循環も止まる）
- 外の依存も要素として扱う。最初の利用者の「crate ごとに許す外の crate」は、外の要素の group への allow で書く
- どの group にも入らない要素が辺の元か先にあれば違反（import-linter `exhaustive`・deptrac `--fail-on-uncovered` と同じ考え）
- 規則そのものの検査：allow の group の間に循環があれば、規則が読めないとして断る
- 形は JSON Schema で公開する
- 既知の違反を一時的に許す仕組みは持たない（範囲の外）

採らなかった案：
- 層の並び（上が下に依存してよい。import-linter `layers`・ArchUnit layeredArchitecture）だけで書く：最初の利用者の domain の中は並びにならない（search は change を使ってよいが validation を使ってはいけない、session はどれにも依存しない）。並びで書くと search → validation を許してしまう
- 禁止リスト型（forbidden）：書き忘れた辺が通る。最初の利用者は「書いていない組はすべて禁止」と決めている
- パスの正規表現を規則に直接書く（dependency-cruiser）：言語のパスの形が規則に入り、言語に依らない形にならない
- 辺の種類で規則を絞る（dependency-cruiser `dependencyTypes`）：最初の利用者は dev の辺の循環を止めたいので、種類を問わず止める。絞りが要る利用者が現れたら足す
- タグの AND（Nx）・`+` による推移的な許可（deptrac）：最初の利用者は使わない

## Q2 言語との境目

問い：判定の中心は何を受け取り、言語ごとの読み取りとどう分けるか。

答え：判定の中心は「依存グラフ」の JSON（要素の並びと辺の並び）と規則だけを受け取る。言語ごとの読み取りは、そのグラフの JSON を出す別の部品にする。

- グラフの形：`nodes`（`id`・`external`）と `edges`（`from`・`to`・`kind`）。`kind` は読み取りが付ける文字列で、中心は意味を解釈しない（報告に載せるだけ）
- グラフの形も JSON Schema で公開する。ほかの言語の読み取りは、その形の JSON を出すプログラムなら何で書いてもよい（標準入力かファイルで渡す）
- 最初の版の読み取りは Rust だけ：公式の `cargo metadata --format-version 1` を起動し、`workspace_members` を内の要素、`resolve.nodes` の `deps` を辺、`dep_kinds` の `kind`（"dev"・"build"・null→"normal"）を辺の種類にする。辺の元は内の要素だけ（外の要素どうしの辺は持たない）
- 紙の上で当てる2つ目：Python のモジュール（import の辺。kind は "runtime" と "type-checking"）。規則と中心を変えずに書けることを確かめる

採らなかった案：
- 中心が cargo metadata を直接読む：中心に Rust の語（packages・resolve・dep_kinds）が入る
- 言語ごとに中心を持つ：規則の形が言語ごとに分かれる
- 読み取りを中心と同じ実行ファイルのプラグインにする（動的ロード）：言語ごとの読み取りを Rust で書くことを強いる。JSON を渡す形なら言語を問わない

## Q3 出力と終了コード

問い：違反をどう報告し、終了コードを何通りにするか。

答え：標準出力に JSON で違反の一覧を出し、終了コードは 0（違反なし）・ 1（違反あり）・ 2（規則かグラフが読めない ・ 使い方の誤り）の3つにする。

- 違反1件：`from`・`to`・`kind`・`from_group`・`to_group`・`reason`（`not-allowed`＝allow に無い ／ `ungrouped`＝どの group にも入らない）
- 人が読む形は `--format text` で同じ中身を1行1件で出す
- 規則の誤り（知らない group を指す ・ allow の循環 ・ 形の違反）は違反と分けて 2
- 規則に書いたのに一度も使われなかった allow は報告に載せる（違反にはしない）

採らなかった案：
- 違反の数を終了コードにする（dependency-cruiser）：256件を超えると0に戻る環境がある（推測）。合否だけで足りる
- 検査ごとのビット集合（cargo-deny）：検査は1種類なので要らない
- 「違反あり」と「読めない」を同じ 1 にする：CI で、規則の書き損じと本物の違反が区別できない

## Q4 置き場所・配り方・名前

問い：ツールをどこに置き、どう配り、何と呼ぶか。

答え：リポジトリ直下の `dependency-rules/` に Rust の workspace として置き、実行ファイル `dependency-rules` を配る。利用者のリポジトリは規則のファイルを持ち、CI で `dependency-rules check --rules <規則> --cargo <ワークスペースの根>` を呼ぶ。

- 中の構成：判定の中心（グラフと規則の型 ・ 判定。入出力をしない）・ 読み取り（cargo metadata）・ 入口（CLI）。ツール自身の規則ファイルを持ち、自分を検査する
- 規則ファイルの場所と名前は利用者が決め、引数で渡す（既定の名前は持たない）
- 最初の利用者は semantic-agent-schema/architecture.json を規則ファイルにする
- 名前は目的（依存の規則）を表し、Rust の語も最初の利用者の名前も含めない

採らなかった案：
- semantic-agent-schema の中の crate にする：アーキテクチャに寄らない道具が、1つの利用者の中に置かれる（利用者が案 A を選んだ）
- Skill として .claude/skills/ の下に置く：特定の AI ツールの置き場所に入る。CI から呼ぶ道具なので Skill にしない
- cargo のサブコマンド（cargo-xxx）として配る：名前と呼び方が Rust に寄る
