---
name: "skills-creator"
description: "新しいSkillsを作成したいとき、Skillのテンプレートが欲しいとき、「スキルを作って」「Skills作成」「新しいスキル」と言われたとき、またはベストプラクティスに従ったSkillフォルダ構成を生成したいときに使う。"
---

# 型と言語を選んで新しい Skill の一式を生み、契約を検査するのを担当する Skill：skills-creator

## 目的

新しいSkillsを作成したいとき、Skillのテンプレートが欲しいとき、「スキルを作って」「Skills作成」「新しいスキル」と言われたとき、またはベストプラクティスに従ったSkillフォルダ構成を生成したいときに使う。

---

## 役割

- 新しい Skill の要件（名前 ・ 使う場面 ・ 型 ・ 道具の要否 ・ 言語）を確認する
- 道具を持つ Skill は、`skills-creator scaffold` で生む。Rust ならリファレンス実装から生成し、ほかの言語なら枠を置いてリファレンス実装を移植する
- SKILL.md の各 `{{…}}` を、そこに書かれた指示に従って記入する
- `skills-creator check`（助言型は `accept` も）で契約を検査し、検出を0件にしてから渡す
- 作ったものの使い方（CLI と MCP の呼び方）を利用者に伝える

---

## 処理対象と成果物

### 処理対象

新しい Skill の要件。名前 ・ 使う場面 ・ 型 ・ 道具の要否 ・ 道具を書く言語が決まっていない状態も含む。

### 成果物

`.claude/skills/<名前>/` に置いた新しい Skill。SKILL.md と、道具を持つなら道具の一式（`tool/` ・ `tool.json` ・ `mcp.json` ・ `references/`）を持ち、`check` の検出が0件である。

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| Skill の名前 | 英小文字とハイフンにする（例：`release-notes`）。明示されなければ、用途から案を出して確認する |
| 使う場面と、何を作るか | 明示されなければ作成を始めず、先に確認する |
| 型 | 下の「Skill の型」の表で決める。明示されなければ、何を作る Skill かから判定して確認する |
| 道具の要否 | 検査 ・ 生成 ・ 変換を道具にするなら必要とする。手順と知識だけの Skill なら不要である |
| 道具を書く言語 | `rust` ・ `python` ・ `typescript` ・ `csharp` ・ `go` のどれか。明示されなければ、利用者が普段使う言語を確認する。既定は `rust` である |

---

## 実行手順

### Step 1: 要件を確認する

名前 ・ 使う場面 ・ 型 ・ 道具の要否 ・ 言語を確認する。不明な項目は利用者に質問する。**要件が決まるまで作成を始めない。**

### Step 2: 一式を置く

道具を持つ Skill は、`scaffold` で型の一式を置く。

```
skills-creator scaffold <名前> --type <work|advisor> [--language <言語>]
```

`--language` を省くか `rust` を渡すと、Rust のリファレンス実装（`references/sample/rust/`）から道具まで生成する。
ほかの言語（`go` ・ `python` など）を渡すと、言語に依存しない枠（SKILL.md ・ references ・ `tool.json` ・ `mcp.json`）だけを置き、
道具はリファレンス実装を移植して書く（手順は `skills-creator view --kind document --id porting` で読む）。
置いたファイルの一覧と、次に書くもの ・ 組み立てのコマンド（または移植の案内）が出る。**既に在るファイルは上書きしない。**

道具を保持しない Skill は、`references/skill-template.md.tmpl` を `.claude/skills/<名前>/SKILL.md` として置くだけにする。

### Step 3: 組み立てる

`tool.json` の `build` のコマンドを、Skill のフォルダで実行する（`scaffold` が「組む ──」として出す）。その言語の処理系を必要とする。**処理系が無い環境では組み立てられない** ── 利用者に導入を頼む。

### Step 4: 中身を書く

- SKILL.md の各 `{{…}}` を、そこに書かれた指示に従って記入する。**指示を読まずに推測で記入しない**
- 作業型は、見本の道具 `hello` を、この Skill の道具に書き換える。道具の一覧（能力の正本）と、そこから組む CLI と MCP の形は変えない
- 助言型は、`references/document.json` の手順（id「procedure」）の9段で判断基準を作る

### Step 5: 検査する

```
skills-creator check <Skill のフォルダ>     # 振る舞い ・ ソース ・ 文書の3つ
skills-creator accept <Skill のフォルダ>    # 助言型だけ。受け入れの検査（機械の7件）
```

**検出が0件になるまで Step 4 へ戻る。** 生んだ直後の `check` は、SKILL.md の未記入の差し込み場所を検出する ── 記入が完了していないという印である。

### Step 6: 使い方を伝える

作ったファイルの一覧と、呼び方を利用者に伝える。

- CLI：`tool.json` の `cli` に書いた起動のコマンド
- MCP：`mcp.json` に書いた起動のコマンドを、プロジェクトの MCP の登録へ足す（Claude Code では `.mcp.json` の `mcpServers` に足すか、`claude mcp add --scope project` で登録する）

---

## 出力形式

作ったファイルの一覧 ・ `check` の結果（検出0件） ・ CLI と MCP の呼び方を、利用者に伝える。

---

## Skill の型

**skills-creator は、Skill を型ごとに生む**（ACDR 0060 ・ 0061 ・ 0097）。型は、その Skill を使うときに誰が何を作るかで決まる。

| 型 | 定義 | 生み方 |
|---|---|---|
| 作業型（`work`） | エージェントが成果物を作る作業の手順を持ち、途中で検査や生成の道具を使う | 共通の一式 ・ 見本の道具 `hello` ・ SKILL.md の雛形 |
| 生成型（`generate`） | 他から入力を渡され、道具が成果物を組んで返す。作業の手順は保持しない | **準備中**（リファレンス実装が雛形を持たず、`scaffold` は断る）。それまでは作業型で生む |
| 助言型（`advisor`） | 相談を受け、原典の判断基準に照らして答える | 雛形を丸ごと固定する（判断基準と回答のスキーマ ・ 回答の道具 ・ 試験） |

**助言型は、`references/document.json` の手順（id「procedure」。`skills-creator view --kind document --id procedure` で読む）の9段で作り、`skills-creator accept <フォルダ>` の受け入れの検査（機械の7件）を実行する。** 判断基準は、原典が名前を付けて立てている概念を1件の単位にする。語彙は原典の語のまま使い、説明は学習ノートを読んでまとめた言葉で書く（ノートの文を複製しない）。

---

## 道具を伴う Skill

**道具を持つ Skill は、道具の契約に従わせる。**
契約は `references/document.json`（tool-contract） が規定する ── 呼び方を1つに固定し、
**能力をサービス層の道具の一覧に1度だけ書かせて、CLI と MCP をその一覧から組ませる**。

**契約が規定するのは、呼ぶ側から観察できるものだけである**（ACDR 0036）。
道具を書く言語と、配布するかどうかは、利用者に委ねる。

| 置く先 | 中身 | 文書 |
|---|---|---|
| 契約 | CLI と MCP の2つのプレゼンテーション層と起動のコマンド（`tool.json` ・ `mcp.json`）・ CLI の規約 ・ 1つの道具の一覧 ・ Skill のフォルダの求め方 ・ 外部の道具 | `references/document.json`（tool-contract） |
| リファレンス実装 | Rust の実装1組 ── 実行ファイルの置き場所 ・ 雛形とその構成（推奨） ・ 2段目の検査。作業型と助言型を生む。ほかの言語の Skill は、これを移植する | `references/document.json`（sample-rust ・ porting） |
| 配布（任意） | 配布物 ・ 導入スクリプト ・ 組み立ての定義。全言語で1つを共有する | `references/document.json`（distribution） |

```
skills-creator scaffold <Skill の名前> [--type work|advisor] [--language <言語>]   # Rust は道具まで、ほかの言語は枠だけを置く
skills-creator check <Skill のフォルダ>  # 2段の検査と、文書の検査（節の構成 ・ 未記入の差し込み場所）
skills-creator check <Skill のフォルダ> --cases 1  # 加えて、契約のテストケースを Skill の実行コマンドで実行する
skills-creator accept <advisor のフォルダ>  # 助言型の受け入れの検査（機械の7件）
skills-creator dist --repo <所有者/リポジトリ>  # 配布するときだけ。導入スクリプトと組み立ての定義を置く
```

**check は2段で検査する。** 1段目は実行ファイルを起動して振る舞いを確認する ── どの言語でも同じである。
2段目はソースを読む検査で、リファレンス実装と同じ言語（Rust）の Skill だけに当てる。**ほかの言語では「実行しない」と出し、合格とは扱わない** ── 共通ツールの振る舞いは、どの言語でもテストケースで確かめる。

**共通ツール（get ・ validate ・ view ・ import）の振る舞いは、テストケースで決まる**（ACDR 0096）。
テストケースは `references/contract/cases/` に1件1ファイルで置き、呼び出しと期待値（終了コードと `--json` の出力）を持つ。
`check --cases 1` は、検証する Skill の `tool.json` の実行コマンドで全件を実行し、JSON の値として比較する ── どの言語で書いた Skill にも、同じテストケースを実行できる。
テストケースは、Rust のリファレンス実装のテスト1件に1件以上を対応させる。

**references は JSON Schema と JSON で持つ**（契約の版2、ACDR 0043）。`tool.json` に `"contract": 2` を書いた
Skill は、雛形の references の実装で get ・ validate ・ view ・ import を持ち、Markdown は SKILL.md だけにする。
**道具を持つ Skill は、どれも版2 に従う**（ACDR 0075）── `tool.json` に `"contract": 2` が無ければ、`check` が検出する。**SKILL.md がコードの記法で指す Skill の中のファイルは、実在しなければ検出する。**

**外部の道具は、外から注入する。** `tool.json` の `external` に名前 ・ 起動するコマンド ・ 理由を書き、
サービス層が業務ロジック層へ渡す ── 業務ロジック層は名前を直書きしない。利用者は `tool.json` を書き換えるだけで差し替えられる。

**生んだものが、そのまま契約を満たす。** 満たさないと、新しい Skill は必ず不合格の
状態で生まれる ── 実測で、契約だけを変えて雛形を差し替えなかったとき、そうなった。

**MCP の実装が無い環境では、MCP サーバーは立たず、CLI だけが動く** ──
呼び出し方が欠けても、能力は欠けない。

---

## ガードレール

- **節の構成を、対応する雛形と一致させる**。助言型の Skill は `references/types/advisor/skill-template.md.tmpl`、それ以外は
  `references/skill-template.md.tmpl` を満たす ── **どちらを適用するかは節の有無で決まる**（名前で分岐すると、Skill が
  増えるたびに検査を直すことになる）
- **道具を持つ Skill には、道具の契約を適用させる**（`references/document.json`（tool-contract））── 呼び出し方の形が道具ごとに違うと、呼ぶ側は呼ぶたびに本文を読み直すことになる
- **同じ概念の仕組みは、同じ実装の形にさせる**。入力の契約（スキーマ）・トークンの正本・入力の検査・置き場所を、Skill ごとに違う形で実装させない ── 概念が同じで形が違うと、**1つを読んで得た理解が、次の Skill で通用しない**
- **能力を2か所へ記述させない**。CLI と MCP は、サービス層の道具の一覧から組ませる ── 同じ能力を2度書くと、片方だけが古くなる
- **業務ロジック層の関数に呼び出し方を付けさせない**。業務ロジック層の関数は道具の一覧に載せない ── 載せると、同じ能力に呼び方が2つできる
- **道具の中の構成を、契約として強制しない**（ACDR 0059）。層 ・ 依存の向き ・ 入出力の置き場所は Skill ごとの設計である ── Skill ごとに実現したいことが違うので、中身まで指定すると、契約が守らせたいもの（呼び方と成果物の形）を越える。雛形はサービス層を挟んだレイヤードアーキテクチャの4層で生むが、check は既定ではこれを検査しない（`--layout 1` で検査する）
- **道具に印字させない**。戻り値は `{ok, findings, data}` とし、印字と終了コードはプレゼンテーション層が持つ ── 道具が印字すると、MCP から呼んだときに戻り値が空になる
- **ガードレールと手順を、不変条件で記述させる**。1回の出来事・特定の語・特定の形を規則にしない ── **規則が事例の形を取ると、その形だけが規制され、他の形は通過する**。規則が禁じるのは性質であって、その性質が現れた1つの形ではない
- **例を規則本体へ混ぜさせない**。例を添えるなら「例」と明示し、規則の文とは分離する ── 混ざると、読み手は例の範囲を規則の範囲として解釈する
- SKILL.md は必ずスキルフォルダのルートに置く。サブフォルダには置かない
- テンプレートファイルは assets/ ではなく references/ に置く
- 不要なフォルダは作らない。使うものだけ作る
- 差し込み場所は、`references/skill-template.md.tmpl` の各 `{{…}}` に書かれた指示に従って記入する。指示を読まずに推測で記入しない
- スキル名は英小文字・ハイフン区切りに統一する（スペース・アンダースコア不可）
- 利用者が要件を明確にしていない場合は作成を開始しない。必ず確認を先に完了させる
- **一式を手で書き起こさない**。道具を持つ Skill は `scaffold` で生む ── 手で書くと、契約（道具の一覧 ・ CLI の規約 ・ MCP ・ references の4つの道具）のどれかが欠落し、`check` が不合格になる
- **`check` の検出を残したまま渡さない**。0件にしてから渡す
- 新しいSkillに、特定の外部ツール・システム（特定の CLI ・ MCP など）の存在を自ら判定して振る舞いを変えるロジックを持たせない。汎用のSkillが具体的な実装の有無を参照するのは依存性の方向違反であり、正しい向きは「具体的なシステムの側が汎用Skillを自分の中に注入・統合する」（composition-rootの原則）。特定システムとの統合が必要な場合は、そのSkill自体は単一の環境非依存な実装のままにし、統合はSkillの外側（呼び出し側・Orchestrator側）に置く設計を選ぶ

---

## 参照

この Skill のフォルダに在るものだけを並べる。道具のソース（`tool/`）は配布物に入らない。

- `bin/skills-creator` ・ `bin/skills-creator-mcp`: この Skill の CLI と MCP サーバー。`scaffold` ・ `check` ・ `accept` ・ `dist` を持つ ── **この Skill も、同じ契約に従う**
- `references/skill-template.md.tmpl`: SKILL.md の雛形（作業型）。各 `{{…}}` が、記入のしかたの指示を持つ
- `references/document.json`（folder-structure）: Skill のフォルダのミニマム構成とフル構成
- `references/view/`: **描画の見た目の正本**（ボード view-design-tokens）── 色のトークン（パレット × 明暗）・ トークンのスキーマ ・ 規則（view.css）・ 頁の型。scaffold が各 Skill の references/ へ複製し、check が複製と正本の差 ・ 色の直値 ・ 定まらない変数 ・ 文字と地の比を検査する（`tool/business_logic/src/view.rs`）
- `references/document.json`（tool-contract）: 道具の契約。**言語に依存しない**。2つのプレゼンテーション層 ・ 戻り値 ・ 終了コード ・
  外部の道具 ・ 1段目の検査 ・ 雛形が採る構成（推奨 ・ 契約ではない） ・ **MCP サーバーの規約**（標準出力 ・ 誤りの返し方 ・
  引数の型 ・ 子プロセスの規律）を規定する ── MCP の規約は**原典の引用と行番号つき**である
- `references/contract/`: **言語に依存しない契約**（ACDR 0096）── Skill の構成（`structure.json`）・ ツールの CLI ・ 出力 ・ 終了コード ・ サブコマンド ・ `tool.json` の欄（`tools.json`）・ リファレンス実装からしか読み取れない実装上の前提（`assumptions.json`）。それぞれのスキーマを並べて置く
- `references/contract/cases/`: 共通ツールのテストケースと、その入力データ（`fixtures/`）。対応するテストを持たない理由は `exempt.json` が持つ
- `references/types/`: Skill の型（作業型 ・ 生成型 ・ 助言型）の定義
- `references/types/advisor/`: 助言型の正本 ── 判断基準と回答のスキーマ ・ SKILL.md の雛形 ・ 9段の手順は `references/document.json` の id「procedure」が持つ
- `references/sample/rust/`: **リファレンス実装**（Rust の実装1組）。`sample.json` が何をどこへ置くか ・ ソースの拡張子 ・ 外部の道具の起動の書き方を持つ。説明は `references/document.json`（sample-rust）
- `references/types/skeleton.json` ・ `references/types/common/`: どの言語の Skill にも置く枠（`tool.json` ・ `mcp.json` ・ `.gitignore` ・ `references/document.schema.json`）
- `references/document.json`（porting）: ほかの言語へリファレンス実装を移植する手順
- `references/document.json`（distribution） ・ `references/distribution/`: 配布（任意）。導入スクリプトと組み立ての定義の雛形
