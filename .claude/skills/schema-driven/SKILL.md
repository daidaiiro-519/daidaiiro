---
name: schema-driven
description: AI が扱うデータを、Markdown ではなく JSON Schema とそのインスタンスで持つときに使う基盤の Skill。abstract スキーマ（版ごとに振る舞いを保証する契約）とエンジンを持つ。Skill ・ Agent は、abstract に従う concrete スキーマ ・ インスタンス ・ ページテンプレートを持ち、エンジンの共通ツールで作成 ・ 取得 ・ 更新 ・ 検査 ・ 承認 ・ 描画する。
license: MIT
---

# 構造化データを正本として持つ基盤の Skill：schema-driven

## 目的

AI が扱うデータを、**JSON Schema とそのインスタンスで持つ**ときに使う。Markdown の文書は、書いたものが形を満たしているかを機械で確かめられない。スキーマとインスタンスなら、書いたその場で検証でき、参照と導出値も検査できる。

この Skill は**基盤**である。何のデータを持つかは決めない。

| 誰が | 何を持つか |
|---|---|
| 基盤（この Skill） | abstract スキーマ（`assets/` の契約）と、それを版ごとに守るエンジン（共通ツール9つ） |
| 使う側（基盤の上に作る Skill ・ Agent） | concrete スキーマ ・ インスタンス ・ ページテンプレート ・ 部品のカタログ（`assets/catalog/`）・ 自分の能力を持つ実行ファイル（任意） |

**使う側は、基盤の能力をそのまま使い、自分の能力は自分の実行ファイルとして持つ。** 基盤は使う側の実行ファイルを起動しない。使う側が、自分の実行ファイルを直接呼ぶ。ページの見た目と操作は、使う側の Skill のフォルダにある部品のカタログが持つ。基盤はカタログの中身を持たず、カタログとデザインシステムが守る形だけを持つ。abstract スキーマを書き換えると、エンジンが扱えなくなる。

---

## 役割

- concrete スキーマが abstract スキーマの版に従っているかを検査する
- スキーマからインスタンスを作成し、未記入のプロパティと、その x-prompt を返す
- JMESPath 式で値を取得し、JSON Patch で更新する。更新は、検証エラーが無いときだけ書く
- ディレクトリのインスタンスを検証し、x-ref（参照）と x-derive（導出値）を検査する
- 検査を通ったインスタンスのパスとハッシュ値を、承認記録へ書く
- 検証を通ったインスタンスを、ページテンプレートと、使う側の Skill のフォルダの部品のカタログでページへ描画する

---

## 処理対象と成果物

### 処理対象

concrete スキーマと、それに従うインスタンスのディレクトリ。スキーマは `x-abstract` に従う abstract の版を、`$id` に自分の名前を書く。インスタンスは `$schema` に、そのスキーマの名前を書く。

### 成果物

| 成果物 | どこに書くか |
|---|---|
| インスタンス | 渡したパス |
| 検証と検査の結果 | 標準出力（JSON） |
| 承認記録 | 検査したディレクトリの `approval.json` |
| ページ | 渡した出力のディレクトリ（インスタンス1件につき HTML 1枚） |

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| スキーマ ・ インスタンス ・ ディレクトリのパス | 明示されなければ受け付けない。相対パスは、CLI を起動した場所から解決する |
| ページテンプレートのディレクトリ（render の `--pages`） | 明示されなければ、どの種類にもページテンプレートの指定が無い。インスタンスが1件でもあれば描画しない |
| 読んだ時点のハッシュ値（update の `--hash`） | 明示されなければ、update が読んだ時点のハッシュ値で書く。ほかの更新と重なりうるときは渡す |
| Skill のフォルダ（`--skill`） | 明示されなければ、スキーマの名前はエンジンが持つものだけを引き、描画はできない。渡すと、その Skill のスキーマも名前で引き、その Skill の `assets/catalog/` のカタログで描画する |

---

## 実行手順

ツールは、CLI の実行ファイル `scripts/schema-driven` と、MCP のサーバー `scripts/schema-driven-mcp` の両方から、同じ名前と引数で呼べる。MCP の登録はツールごとに形が違うので、導入するときに行う。一覧は次で取れる。

```
scripts/schema-driven --json
```

### Step 1: concrete スキーマを書き、abstract に従っているかを検査する

concrete スキーマは、abstract スキーマのインスタンスである。インスタンスと同じ共通ツールで作り、書き進める。中身（何を持つか）は、作る側が決める。

```
scripts/schema-driven create --schema schema-driven/abstract@v1 --path <Skill のフォルダ>/assets/<種類>.schema.json
scripts/schema-driven prompt --schema schema-driven/abstract@v1 --property /<欄>
scripts/schema-driven update --path <スキーマ> --patch '<JSON Patch>'
```

create は、根に `$schema` と `x-abstract` を書いて枠を作り、未記入の欄を x-prompt と一緒に返す。根に置く3つの欄は次のとおりで、書き方の決まりは `assets/meta-schema.json` が持つ。

| 欄 | 書くもの |
|---|---|
| `$schema` | JSON Schema の標準の dialect `https://json-schema.org/draft/2020-12/schema` |
| `x-abstract` | 従う abstract の版 `schema-driven/abstract@v1` |
| `$id` | このスキーマの名前「<Skill の名前>/<種類>@<版>」。インスタンスを作るスキーマ（`x-generates` を持つもの）は必ず書く |

`$id` は create のあとに書く。インスタンスを作るスキーマは、根の `properties` に `kind` と `$schema` を置き、`$schema` の `const` に自分の `$id` を書く。

プロパティごとに `description` と `x-prompt`（`read` と `write`）を書き、参照には `x-ref`、導出値には `x-derive` を付ける（`assets/annotations.schema.json`）。

ほかのスキーマの定義を使うときは、`$ref` をファイル名ではなく名前で書く。同じ Skill のスキーマなら、`$id` を基準に JSON Schema の規則で解けるので、「<種類>@<版>」だけでよい（例：`term@v1#/$defs/short`）。ファイル名を変えても、参照は切れない。

```
scripts/schema-driven check-schemas --dir <スキーマのディレクトリ>
```

`findings` が空なら、契約に従っている。版を書いていないスキーマと、エンジンが扱わない版のスキーマは、検査せずに指摘する。

スキーマのフォルダに公開の記録 `published.json`（`assets/published.schema.json`）があれば、check-schemas は公開した版の指紋といまの中身を比べる。

| 状態 | check-schemas の返し方 |
|---|---|
| 公開した版の中身が変わった | `findings` に「公開した版が書き換えられている」 |
| 公開した版が消えた | `findings` に「公開した版が無い」 |
| まだ記録に無い名前 | `unpublished` に名前と指紋。公開するときに、そのまま記録へ足す |

指紋は、キーの順と空白に左右されない SHA-256 である。名前（`$id`）を持つ JSON は、ファイル名が `.schema.json` で終わらなくても対象になる。

### Step 2: インスタンスを作成し、x-prompt を読む

```
scripts/schema-driven create --schema <スキーマ> --path <インスタンス>
scripts/schema-driven prompt --schema <スキーマ> --property /<プロパティ>
```

create は、必須のプロパティを空のままにしてインスタンスを作り、未記入のプロパティを x-prompt と一緒に返す。インスタンスの `$schema` には、スキーマの名前を書く。**x-prompt の `write` の決まりに従って値を決める。**

インスタンスを読むツールは、`$schema` の名前から次の順にスキーマを引く。

| 名前 | 引く先 |
|---|---|
| `schema-driven/…` | エンジンが持つスキーマ（ページテンプレート ・ カタログの枠 ・ デザインシステムなど）。`--skill` を渡さなくても引ける |
| `--skill` の Skill の名前で始まるもの | その Skill の `assets` の下の `*.schema.json` のうち、`$id` が同じもの |
| ほかの Skill の名前で始まるもの | どの Skill のスキーマかを返して止める。その Skill を `--skill` に渡して検査し直す |
| JSON Schema の dialect（concrete スキーマ） | `x-abstract` に書いた abstract スキーマ。エンジンが持つので、`--skill` を渡さなくても引ける |
| 名前の形でない値 | 断って「スキーマの名前でない」を返す。パスでは引かない |

同じ名前のスキーマが2つあれば、どちらも使わずに止める。

### Step 3: JSON Patch で更新する

```
scripts/schema-driven update --path <インスタンス> --patch '<JSON Patch>' [--hash <読んだ時点のハッシュ値>]
```

未記入以外の検証エラーがあれば、書かずに `errors` を返す。直して、もう一度送る。未記入が残っているときは書いて、残りを x-prompt と一緒に返す。

値を読むときは get を使う。

```
scripts/schema-driven get --path <インスタンス> --query '<JMESPath 式>'
```

### Step 4: ディレクトリを検査する

```
scripts/schema-driven check --dir <インスタンスのディレクトリ>
```

**終了コード 0 は、検査を最後まで実行できたことだけを示す。** 合格かどうかは、`instances` の `errors` と `unfilled`、`findings` の `status`（合格 ・ ずれ ・ 確かめ直し など）で見る。

### Step 5: カタログを置き、ページテンプレートを書き、描画する

ページの見た目と操作は、使う側の Skill のフォルダの `assets/catalog/` に置く部品のカタログが持つ。基盤はカタログの中身（部品の名前 ・ HTML ・ CSS ・ JS ・ トークンの値）を持たない。render は、`--skill` に渡した Skill のフォルダから、描画のたびにカタログを読む。

| ファイル | 中身 | 守る形 |
|---|---|---|
| `assets/catalog/catalog.schema.json` | 部品の一覧。`$defs` のキーが部品の名前で、値が入力の形 ・ 説明 ・ 状態ごとの振る舞い（`x-states`） | `assets/catalog.schema.json`（`schema-driven/catalog@v1`） |
| `assets/catalog/parts.html` | 部品の HTML。部品ごとに `<template id="部品の名前">` を1つ置き、ほかにページの `page` と節の `section` を置く | プレースホルダーは `{{名前}}` |
| `assets/catalog/design-system.json` | トークンの値と、コントラストを確かめる色の組 | `assets/design-system.schema.json`（`schema-driven/design-system@v1`） |
| `assets/catalog/catalog.css` | 部品の見た目。無くてもよい | 色の値を直接書かず、`var(--トークンの名前)` で引く |
| `assets/catalog/catalog.js` | 部品の操作。無くてもよい | ― |

基盤は、テンプレートのプレースホルダーへ次の値を入れる。

| テンプレート | 基盤が入れる値 |
|---|---|
| `page` | `{{title}}`（インスタンスの名前）・ `{{tokens}}`（トークンから作った CSS の変数）・ `{{style}}`（catalog.css）・ `{{script}}`（catalog.js）・ `{{sections}}`（節）。ほかのプレースホルダーには、ページテンプレートの `head` の同じ名前の値 |
| `section` | `{{heading}}`（見出し）・ `{{help}}`（説明）・ `{{body}}`（節の要素） |
| 部品 | ページテンプレートに書いた入力の値を、同じ名前のプレースホルダーへ入れる。文字はエスケープする。入力の形の `contentMediaType` が `image/svg+xml` の入力だけは、検査した SVG をそのまま入れる |

デザインシステムについて基盤が確かめるのは、形式 ・ 参照（`{名前}` の指す先があるか）・ 参照が循環しないこと ・ `contrast` に書いた組のコントラスト（文字 4.5:1、部品の境界 3:1。明と暗の両方）と、カタログの CSS に色の値を直接書いていないことだけである。トークンの名前と層の分け方は、使う側が決める。

ページテンプレートは、節の並びと、節ごとに置く部品と、値を取る JMESPath 式だけを書いた JSON である（`assets/page.schema.json`）。部品は、カタログの部品の名前で呼ぶ。**HTML ・ CSS ・ JS は書かない。**

```
scripts/schema-driven --skill <Skill のフォルダ> render --dir <インスタンスのディレクトリ> --pages <ページテンプレートのディレクトリ> --out <出力>
```

| 状態 | render の返し方 |
|---|---|
| カタログが無い | 1ページも書かずに「部品のカタログを読めない」 |
| カタログが枠かデザインシステムの契約を満たさない | 1ページも書かずに「部品のカタログが契約を満たさない」 |
| ページテンプレートに、カタログに無い部品 ・ 入力の欠け ・ `page` に無い `head` の名前がある | 1ページも書かずに「ページテンプレートが契約を通過しない」 |
| ページテンプレートの指定が無い種類がある | 1ページも書かずに「ページテンプレートの指定が無い種類がある」 |
| 検証エラーのあるインスタンスがある | 1ページも書かずに「検証を通過しないインスタンスがある」 |

内容が変わったページだけを書くので、同じ入力から何度描画しても結果は変わらない。インスタンスの値は、HTML を含んでいても文字として出る。

### Step 6: 承認を記録する

```
scripts/schema-driven approve --dir <インスタンスのディレクトリ>
```

検証エラー ・ ずれ ・ 未記入が1件でもあれば、承認記録を書かない。

---

## 出力形式

どのツールも、標準出力へ JSON を1つ書く。

| 終了コード | 意味 | 出力 |
|---|---|---|
| 0 | 実行できた | `{"ok": true, …}`。中身はツールごとに違う |
| 1 | 拒否した ・ 失敗した | `{"ok": false, "reason": …, "detail": …, "errors": […]}` |
| 2 | 使い方が違う | `{"ok": false, "reason": "使い方が違う", "detail": …, "tools": […]}` |

動詞なしの `--json` だけは、ツールの一覧を `{"ok": true, "findings": [], "data": {"tools": […]}}` で返す。

---

## ガードレール

- **abstract スキーマを、使う側で書き換えない**。エンジンは、版ごとの abstract スキーマに対する振る舞いだけを保証する
- **公開した版のスキーマを、消さず書き換えない**。中身を変えるときは、版を上げた新しい名前（`@v2`）で足す。同じ名前で中身が変わると、前に書いたインスタンスが別のスキーマで検査される。公開するときは、その版の指紋を `published.json` に記録し、配る前の検査で確かめる
- **インスタンスの `$schema` に、Skill の置き場所へのパスを書かない**。名前で指す。配った先では、Skill は別の場所に置かれる
- **ファイルを手で書き換えて、update を通さずにインスタンスを直さない**。update を通さないと、書いたその場の検証が行われない。手で直したときは、check で確かめ直す
- **check の終了コードだけで合格と判断しない**。0 は実行できたことだけを示す
- **検証エラーのあるインスタンスを、描画や承認で先へ進めない**。基盤はどちらも止める
- **ページテンプレートに、HTML ・ CSS ・ 色の値を書かない**。見た目と操作は、Skill のフォルダのカタログ（`assets/catalog/`）だけが持つ。ページテンプレートに書くと、カタログを替えても見た目が揃わない
- **部品のカタログは、枠とデザインシステムに従う**。部品ごとに名前 ・ 入力の形 ・ 説明 ・ 状態ごとの振る舞いを書き、CSS に色の値を直接書かず、文字と背景 ・ 部品の境界と背景の組を `contrast` に書く
- **基盤の中心は、ファイルを直接扱わない**。入出力はポートを通す
- **基盤はネットワークに出ない**。依存の版は1か所で決める

---

## 参照

| 場所 | 中身 |
|---|---|
| `assets/meta-schema.json` | abstract スキーマ v1（`schema-driven/abstract@v1`）。concrete スキーマが従う契約で、check-schemas が検査する |
| `assets/annotations.schema.json` | 注釈 x-ref（指される側の数の決まり inverse を含む）と x-derive の仕様 |
| `assets/view.schema.json` | x-view の書き方と、基盤の6つの関数（name ・ label ・ map ・ view ・ part ・ quote） |
| `assets/page.schema.json` | ページテンプレートの形と、描画の文脈 |
| `assets/catalog.schema.json` | 部品のカタログの枠。使う側の部品の一覧（`assets/catalog/catalog.schema.json`）が守る、部品1つの書き方の形 |
| `assets/design-system.schema.json` | デザインシステムの契約（形式 ・ 参照 ・ 循環 ・ コントラストの下限）。描画の前に基盤が検査する |
| `assets/published.schema.json` | 公開の記録（`published.json`）の形。公開した版の名前と指紋 |
