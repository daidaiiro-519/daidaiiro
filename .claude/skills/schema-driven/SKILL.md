---
name: schema-driven
description: AI が扱うデータを、Markdown ではなく JSON Schema とそのインスタンスで持つときに使う基盤の Skill。スキーマからインスタンスを作成し、x-prompt を受け取り、JMESPath 式で取得し、JSON Patch で更新し、注釈 x-ref と x-derive で参照と導出値を検査し、承認を記録し、ページを描画する。具体の Skill（acdr ・ ブレストボードなど）は、この基盤の crate を転写して、自分のスキーマとデザインで使う。
version: 0.1.0
---

# 構造化データを正本として持つ基盤の Skill：schema-driven

## 目的

AI が扱うデータを、**JSON Schema とそのインスタンスで持つ**ときに使う。Markdown の文書は、書いたものが形を満たしているかを機械で確かめられない。スキーマとインスタンスなら、書いたその場で検証でき、参照と導出値も検査できる。

この Skill は**基盤**である。何のデータを持つかは決めない。

| 誰が | 何を持つか |
|---|---|
| 基盤（この Skill） | 作成 ・ 取得 ・ 更新 ・ 検査 ・ 承認 ・ 描画 ・ 転写の仕組みと、具体のスキーマが従う契約（`references/meta-schema.json` ・ `references/annotations.schema.json` ・ `references/view.schema.json`） |
| 具体（基盤の上に作る Skill） | 自分のスキーマ ・ ページテンプレート ・ デザイン ・ 足したい関数とツール |

具体のデザインが無くても、**文書**だけは描画できる。基盤が文書の契約（`references/document.schema.json`）と決まったデザイン（`references/document-design/`）を持つからである。

---

## 役割

- スキーマからインスタンスを作成し、未記入のプロパティと、その x-prompt を返す
- JMESPath 式で値を取得し、JSON Patch で更新する。更新は、検証エラーが無いときだけ書く
- ディレクトリのインスタンスを検証し、x-ref（参照）と x-derive（導出値）を検査する
- 検査を通ったインスタンスのパスとハッシュ値を、承認記録へ書く
- 検証を通ったインスタンスを、ページへ描画する
- 具体のスキーマが、基盤の契約に従っているかを検査する
- 基盤の crate を具体の Skill へ転写し、基盤の新しい版へ更新する

---

## 処理対象と成果物

### 処理対象

JSON Schema と、それに従うインスタンスのディレクトリ。インスタンスは `$schema` で、自分のファイルからの相対パスでスキーマを指す。

### 成果物

| 成果物 | どこに書くか |
|---|---|
| インスタンス | 渡したパス |
| 検証と検査の結果 | 標準出力（JSON） |
| 承認記録 | 検査したディレクトリの `approval.json` |
| ページ | 渡した出力のディレクトリ（インスタンス1件につき HTML 1枚） |
| 転写した複製 | 具体の Skill の `tool/schema-driven/` と、転写の記録 `tool/schema-driven/transcription.json` |

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| スキーマ ・ インスタンス ・ ディレクトリのパス | 明示されなければ受け付けない。相対パスは、CLI を起動した場所から解決する |
| ページテンプレートのディレクトリ（render の `--pages`） | 明示されなければ、kind が document のインスタンスだけを描画できる |
| 読んだ時点のハッシュ値（update の `--hash`） | 明示されなければ、update が読んだ時点のハッシュ値で書く。ほかの更新と重なりうるときは渡す |

---

## 実行手順

ツールは、CLI `bin/schema-driven` と MCP `bin/schema-driven-mcp` の両方から、同じ名前と引数で呼べる。一覧は次で取れる。

```
schema-driven --json
```

### Step 1: スキーマを書き、契約に従っているかを検査する

スキーマの書き方の決まりは、`references/meta-schema.json` が持つ。プロパティごとに `x-prompt`（`read` と `write`）を書き、参照には `x-ref`、導出値には `x-derive` を付ける（`references/annotations.schema.json`）。

```
schema-driven check-schemas --dir <スキーマのディレクトリ>
```

`findings` が空なら、契約に従っている。

### Step 2: インスタンスを作成し、x-prompt を読む

```
schema-driven create --schema <スキーマ> --path <インスタンス>
schema-driven prompt --schema <スキーマ> --property /<プロパティ>
```

create は、必須のプロパティを空のままにしてインスタンスを作り、未記入のプロパティを x-prompt と一緒に返す。**x-prompt の `write` に従って値を決める。**

### Step 3: JSON Patch で更新する

```
schema-driven update --path <インスタンス> --patch '<JSON Patch>' [--hash <読んだ時点のハッシュ値>]
```

未記入以外の検証エラーがあれば、書かずに `errors` を返す。直して、もう一度送る。

値を読むときは get を使う。

```
schema-driven get --path <インスタンス> --query '<JMESPath 式>'
```

### Step 4: ディレクトリを検査する

```
schema-driven check --dir <インスタンスのディレクトリ>
```

**終了コード 0 は、検査を最後まで実行できたことだけを示す。** 合格かどうかは、`instances` の `errors` と `unfilled`、`findings` の `status`（合格 ・ ずれ ・ 確かめ直し など）で見る。

### Step 5: 描画して読む

```
schema-driven render --dir <インスタンスのディレクトリ> --out <出力> [--pages <ページテンプレートのディレクトリ>]
```

| インスタンスの kind | 描画のしかた |
|---|---|
| `document` | 基盤の文書の契約と決まったデザイン |
| 具体の種類 | 具体のページテンプレート（`--pages`）とデザイン |
| どちらでもない | 1ページも書かずに失敗する |

検証エラーが1件でもあれば描画しない。内容が変わったページだけを書くので、同じ入力から何度描画しても結果は変わらない。

### Step 6: 承認を記録する

```
schema-driven approve --dir <インスタンスのディレクトリ>
```

検証エラー ・ ずれ ・ 未記入が1件でもあれば、承認記録を書かない。

### Step 7: 具体の Skill を作るとき、基盤を転写する

```
schema-driven transcribe --to <具体の Skill のディレクトリ>
schema-driven check-copy --to <具体の Skill のディレクトリ>
```

基盤の `tool/core` ・ `tool/adapters` ・ `references` を、具体の Skill の `tool/schema-driven/` の下へ同じ並びで写す。**写した複製は具体の持ち物である。** 2回目からは基盤の新しい版への更新になり、具体が変えたファイルは上書きしない。基盤と具体の両方が変えたファイルは、基盤の新しい版を隣（`<ファイル名>.schema-driven-new`）に置くので、見比べて合わせ、終わったら消す。

具体のコードが使ってよいのは、crate が外へ出しているものだけである（ACDR 0144）。

| crate | 使ってよいもの |
|---|---|
| `schema-driven-core` | `ports`（`Design` ・ `Renderer` ・ `Functions` ・ `Files` など）、`application` の4つのユースケース（`instances` ・ `checks` ・ `renders` ・ `transcriptions`）とその結果、`domain` の直下の値（`Finding` ・ `Html` ・ `Hash` など） |
| `schema-driven-adapters` | `Toolbox` と `ExtraTools`（具体のツールを足す）、CLI の `run_in`、ファイルシステム ・ JMESPath ・ 文書のデザインの実装 |

---

## 出力形式

どのツールも、標準出力へ JSON を1つ書く。

| 終了コード | 意味 | 出力 |
|---|---|---|
| 0 | 実行できた | `{"ok": true, …}`。中身はツールごとに違う |
| 1 | 拒否した ・ 失敗した | `{"ok": false, "reason": …, "detail": …, "errors": […]}` |
| 2 | 使い方が違う | `{"ok": false, "reason": "使い方が違う", "detail": …, "tools": […]}` |

動詞なしの `--json` だけは、ツールの一覧を `{"ok": true, "findings": [], "data": {"tools": […], "skill_root": …}}` で返す。

---

## ガードレール

- **ファイルを手で書き換えて、update を通さずにインスタンスを直さない**。update を通さないと、書いたその場の検証が行われない。手で直したときは、check で確かめ直す
- **check の終了コードだけで合格と判断しない**。0 は実行できたことだけを示す
- **検証エラーのあるインスタンスを、描画や承認で先へ進めない**。基盤はどちらも止める
- **具体の種類を、基盤の文書の契約へ押し込まない**。決まったデザインがあるものは、具体がスキーマとページテンプレートとデザインを持つ
- **具体のコードから、基盤の内部（feature `internals` で開くもの）を使わない**。基盤を更新したときに、具体のビルドが壊れる
- **core ではファイルを直接扱わない**。入出力は ports の trait を通し、`tool/core/clippy.toml` が `std::fs` を禁じる（ACDR 0121）
- **版は `tool/Cargo.lock` の1か所で決める**。基盤はネットワークに出ない（ACDR 0122）

---

## 参照

| 場所 | 中身 |
|---|---|
| `references/meta-schema.json` | 具体のスキーマが従う契約。check-schemas が検査する |
| `references/annotations.schema.json` | 注釈 x-ref（指される側の数の決まり inverse を含む）と x-derive の仕様 |
| `references/view.schema.json` | x-view の書き方と、基盤の6つの関数（name ・ label ・ map ・ view ・ part ・ quote） |
| `references/page.schema.json` | ページテンプレートの形と、描画の文脈 |
| `references/document.schema.json` | 基盤の文書の契約（14種類のブロック） |
| `references/document-design/` | 文書の決まったデザイン（`parts.html` ・ `document.css`） |
| `tool/core/` | domain ・ application ・ ports。何にも依存しない |
| `tool/adapters/` | 入ってくる側（CLI と MCP の受け口、ツールの一覧）と、出ていく側（ファイルシステム ・ JMESPath ・ 文書のデザイン） |
| `tool/cli/` ・ `tool/mcp/` | 実行ファイル。アダプタとユースケースをつなぐ |
| `tool/cli/tests/` ・ `tool/core/tests/` | テスト。`cargo test --manifest-path tool/Cargo.toml` で実行する |
