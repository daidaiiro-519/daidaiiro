# 作成者が記述した SVG を、図のデザインシステムで統一して返すSkill：design-svg

## 目的

作成者が記述した SVG を、図のデザインシステムで統一し、検査して返す。

作成者（AI）が SVG を座標まで記述し、スタイルは値ではなく class で指定する。design-svg は class を図のデザインシステムの値へ解決し、検査1〜4を通した SVG を返す。配置 ・ 図の種類 ・ 図形の組み合わせ ・ 図のサイズは、作成者が決める。

図のデザインシステムは、デザイントークン（色 ・ フォントサイズ ・ ストローク幅 ・ 角丸の半径 ・ 破線のパターン）と、class ごとの意味の2つを持つ。すべての図がこれを使う状態を「統一されている」とする。正本は `references/theme.json` である。

この Skill の実体は `tool/business_logic/` の描画エンジンである。CLI は1つの実行ファイル `design-svg` で、MCP サーバーは `design-svg-mcp` である。

```
cargo install --force --path tool/cli --root . --target-dir tool/target   # bin/design-svg ができる
cargo install --force --path tool/mcp --root . --target-dir tool/target   # bin/design-svg-mcp ができる
```

以下の `design-svg` は、この実行ファイルを指す。

---

## 役割

- class を、図のデザインシステムの値へ解決する。class は削除せずに残す
- 返す前に、検査1〜4を通す
- class の選択の誤りを目視で確かめるために、class ごとのラベルの一覧と、複数行のラベルの一覧を返す
- 辺の多い関係図と量のグラフは、figure と chart で生成してもよい。どちらも同じ解決と検査を通す
- **描画した画像を必ず目視で確認する**。検査を通ることと、図として成立していることは別である

---

## 処理対象と成果物

### 処理対象

作成者が記述した SVG。座標を持ち、スタイルは class だけで指定する。figure と chart を使うときは、構造化データの宣言である。

### 成果物

値を属性に持つ SVG の文字列。class は残る。外部ホストにも実行時のライブラリにも依存しないので、ブレストボード ・ ACDR ・ スライド ・ advisor のページ ・ 単独の .svg ファイルのどこに埋め込んでも同じに表示される。

| 足すもの | 中身 |
|---|---|
| 属性の値 | class が決めた色 ・ ストローク幅 ・ 角丸の半径 ・ 破線のパターン ・ フォントサイズ |
| 矢じり | `flow` の線に付ける `marker`。形はどの図でも同じである |
| ダークモードの `style` | class ごとの CSS ルール。OS の設定と、埋め込み先のページの `data-theme` に従う |

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| 作成者が記述した SVG | `viewBox` と座標を持つ。スタイルは class だけで指定し、値と `marker` と `style` を記述しない |
| 色のトークンの上書き | 明示されなければ既定のテーマを使う。上書きしてよいのは色（`color.`）と部品の選択（`parts.`）だけで、渡すとダークモードの `style` を出さない |
| 置き場所（ファイルへ書くか、埋め込むか） | 明示されなければ文字列として返し、書き出しは呼び出し側が決める |

---

## 実行手順

### Step 1: 図のデザインシステムを確認する

```
design-svg get --kind theme
```

`classes` が class の一覧（図の表記法）で、class ごとに意味と値を持つ。`scale` がデザイントークンの段階、`forbidden` が禁止した組み合わせ、`checks` が検査4のしきい値である。

| 種類 | class |
|---|---|
| 図形 | `box` ・ `boundary` ・ `area` ・ `badge` ・ `start` ・ `field` ・ `set` ・ `swatch` ・ `series-1`〜`3` |
| 線 | `flow` ・ `link` ・ `grid` |
| テキスト | `title` ・ `label` ・ `note` |
| 修飾 | `focus` ・ `warn` ・ `kind-1`〜`3` ・ `async` ・ `small` |

1つの要素は、図形 ・ 線 ・ テキストの class を1つだけ持ち、修飾を0個以上持つ。カテゴリを分けたいときは `kind-1`〜`3` を使い、`focus` や `warn` を流用しない。

### Step 2: 近いテンプレートを選んで複製する

図の種類ごとのテンプレートが17種類ある。索引で用途を読み、描きたい図に近いものを1つ選んで、その SVG を複製して編集する。

```
design-svg get --kind exemplars                      # 索引（id ・ 名前 ・ 用途 ・ ファイル ・ 使う class）
design-svg get --kind exemplars --id state-transition  # 1件だけ
```

SVG は `references/exemplars/<id>.svg` にある。中身は一般の語（要素A ・ 状態1 など）で書いてあるので、座標 ・ 図形 ・ 文字を描きたい内容に合わせて変える。

- テンプレートは出発点であって、使うことを求めない。近いものが無ければ、class の一覧から直接記述してよい
- 新しい種類をテンプレートとして足すときは、検査1〜4と目視を通し、ACDR で承認を得てから references へ置く

### Step 3: SVG を記述する

座標は作成者が決める。スタイルは class で指定し、値を記述しない。

```xml
<svg viewBox="0 0 360 100" style="min-width:300px">
  <rect class="box focus" x="10" y="30" width="130" height="40"/>
  <text class="label focus" x="75" y="54" text-anchor="middle">注文を確定する</text>
  <rect class="boundary" x="220" y="30" width="130" height="40"/>
  <text class="label" x="285" y="54" text-anchor="middle">決済代行（外部）</text>
  <path class="flow" d="M140,50 H218"/>
</svg>
```

- 色 ・ ストローク幅 ・ 角丸の半径 ・ 破線のパターン ・ フォントサイズを記述しない。`fill="none"` と `stroke="none"` は値ではないので記述してよい
- コンテンツとしての色（色見本など）だけは、`swatch` の要素自身の `fill` ・ `stroke` ・ `stop-color` に記述してよい
- 矢じりの `marker` を記述しない。`flow` の線に design-svg が付ける
- 集合（`set`）は `kind-1`〜`3` と一緒に指定し、重なりの上の文字は `label` で書く。`note` の色は重なりの上でコントラスト比が足りない
- `style` 要素を記述しない
- 親の `g` の class は子を満たさない。図形とテキストに1つずつ class を指定する
- 横に長い図は、ルートの `style` に `min-width` を書く。スマホ幅で縮小されると、検査4が最小のフォントサイズを検出する

### Step 4: 解決して検査する

```
design-svg resolve 図.svg --out 出力.svg
design-svg resolve 図.svg --theme 色.json --out 出力.svg   # 色のトークンを上書きする
```

検査1〜3は解決の前の SVG に、検査4は解決のあとの SVG に適用する。

| 検査 | 検出するもの |
|---|---|
| 検査1 | class を持たない図形とテキスト ・ 一覧に無い class ・ 作成者が記述した `marker` |
| 検査2 | 作成者が記述した値（表示属性 ・ `style` 属性 ・ `style` 要素） |
| 検査3 | 禁止した組み合わせ ・ 図形 ・ 線 ・ テキストの class を2つ持つ要素 ・ 要素の種類と合わない class |
| 検査4 | テキストどうしの重なり ・ 線の上の文字 ・ キャンバスやボックスからのはみ出し ・ スマホ幅で描画したときの最小のフォントサイズ |

検査4の「線の上の文字」は、線 ・ 枠線 ・ 円の輪郭が文字の上を通るものを検出する。線の上に置いたバッジ（`badge`）の中の文字と、塗りのある図形の中の文字（図形より先に描いた線は塗りに隠れる）は検出しない。

終了コードは `0` 検出なし ／ `1` 検出あり ／ `2` 誤用（読めない SVG ・ 知らないトークン ・ 上書きできないトークン）である。

### Step 5: 一覧で class の選択を確かめる

class の選択の誤り（イベントに `warn` を指定するなど）は機械では検査できない。`resolve` は class ごとのラベルの一覧を返すので、`warn：注文が確定された` のように意味と合わない行が無いかを読む。複数行のラベルの一覧では、単語の途中で改行していないかを読む。

### Step 6: 描画して目視で確認する

ライトモード ・ ダークモード ・ 幅 358px の3通りで描画して目視で確認する。検査が見ないもの（配置の良し悪し ・ 強調の偏り ・ カテゴリ色どうしの差 ・ 単語の途中の改行）は、目視でしか分からない。

### Step 7: figure と chart を使うとき

辺の多い関係図と量のグラフは、宣言から生成してもよい。どちらも class を出力し、作成者が記述した SVG と同じ解決と検査1〜4を通す。

```
design-svg catalog                               # 部品 ・ トークン ・ 配置戦略の目録
design-svg figure 宣言.json --out 図.svg          # 節点 ・ 辺 ・ 囲みから配置する
design-svg chart bars データ.json --out 図.svg    # 量のグラフを1つ描く
```

```json
{"nodes": [{"id": "a", "label": "受付"}, {"id": "b", "label": "検証", "role": "focus"}],
 "edges": [{"from": "a", "to": "b", "label": "渡す"}],
 "groups": [{"label": "束ね", "members": ["a", "b"]}],
 "direction": "TB", "layout": "graph"}
```

- 節点の `role` は、修飾の class の名前（`focus` ・ `warn` ・ `kind-1`〜`3`）か `muted`（`boundary` で描く）である
- 配置戦略は `graph`（層状）・ `radial`（輪）・ `tree`（放射の木）・ `grid`（宣言の座標）の4つで、宣言の `layout` に書く
- 宣言の `theme` で上書きしてよいのは色のトークンだけである
- 縮小するとしきい値を下回る図には、design-svg が `min-width` を足す

### Step 8: 部品が足りなければ足す

量のグラフの部品は、`tool/business_logic/src/shapes*.rs` の関数1つと、そのファイルの `register()` への1行で増える。部品は値と class の両方を出力し、解決の前に class が持つ値が外れる。class が持たない値を出力すると、検査2が検出する。足すときの規約は `references/document.json` の `svg-engine-discipline` にある（`design-svg get --kind document --id svg-engine-discipline`）。目録の表（`catalog.rs` の `PARTS`）にも1行足す。

---

## 出力形式

**値を属性に持つ SVG の文字列を返す。** ファイルへの書き出しは、呼ぶ側が決める。

| 返すもの | 形 |
|---|---|
| 図 | `<svg>` から始まる単体の文字列。class は残る |
| 検出 | 検査1〜4の検出の一覧。0件なら空である |
| class ごとのラベル | class から、その class を持つテキストと、その class を持つ図形の中のテキストへ |
| 複数行のラベル | 同じボックスの中の行を、上から `｜` でつないだもの |
| 最小のフォントサイズ | スマホ幅で描画したときの値（px） |

---

## ガードレール

- **描画した画像を目視で確認する前に提示してはならない**。最優先。例外なし
- 色 ・ ストローク幅 ・ 角丸の半径 ・ 破線のパターン ・ フォントサイズの値を記述しない。class で指定する
- class の一覧に無い class を使わない。新しい class が必要なときは、新しい種類のテンプレートと一緒に ACDR で追加する
- 同じ class を、どの図でも同じ意味で使う。`warn` は問題 ・ 失敗 ・ 変更前、`kind-1`〜`3` は同じ図の中のカテゴリである
- サイズのトークンを上書きしない。図を大きく表示するときは、SVG の表示幅を広げて viewBox ごと拡大する
- 検査を通すために配置を変えない。検出は、class の選択か `min-width` で直す
- 勘で置いた数値を残さない。コードに現れる数は、トークンから注入するか、データから毎回計算する
- 部品は SVG の断片だけを返す。`<svg>` ルートを作るのは最上位の合成だけである
- 部品は決定的であること。乱数 ・ 現在時刻を使わない
- 呼び出し元の語彙をエンジンへ持ち込まない。このSkillが知ってよいのは節点 ・ 辺 ・ 囲みという一般名詞と、部品 ・ トークン ・ class ・ 配置戦略だけである

---

## 参照

- `README.md`: エンジンの入口（使い方 ・ 目録 ・ 配置 ・ 開発）
- `references/theme.json`: 図のデザインシステムの正本 ── トークン ・ 段階 ・ class の一覧 ・ 禁止した組み合わせ ・ ダークモードの値 ・ 矢じり ・ 検査のしきい値。形は `references/theme.schema.json` が規定し、`design-svg validate` が検査する
- `references/exemplars.json` ・ `references/exemplars/*.svg`: 図の種類ごとのテンプレート17種類と、その索引。形は `references/exemplars.schema.json` が規定する
- `references/document.json`: 手引きの文書。`svg-engine-discipline`（エンジンが遵守する規律）と `svg-engine-layout-algorithms`（配置アルゴリズム）を持つ
- `tool/cli/`: 唯一の CLI `design-svg`。`catalog` ・ `figure` ・ `chart` ・ `resolve` ・ `verify` ・ `lint` と、references のツール `get` ・ `validate` ・ `view` ・ `import` を持つ。どれも `--json` で機械が読む形が出る
- `tool/service/`: サービス層のツールの一覧。能力の正本であり、CLI と MCP はここから作る
- `tool/mcp/` ・ `mcp.json`: MCP サーバー `design-svg-mcp`
- `tool/business_logic/`: 描画エンジン（業務ロジック層）。`classes.rs` が class の一覧を読み、`resolve.rs` が値へ解決し、`checks.rs` が検査1〜4を持ち、`publish.rs` が作成者の SVG と figure ・ chart の出力を同じ順に通す
- `tool/data_access/`: データアクセス層。ファイルの入出力だけを持つ
- `tool/business_logic/tests/`: 契約 ・ 解決 ・ 検査1〜4の事例、figure と chart の出力を固定した事例（`golden/`）、受け入れの事例（`gallery/`）。`cd tool && cargo test`
- `tool/business_logic/examples/bench_layout.rs`: 層状配置を Graphviz の `dot` と同じ宣言で測る計測のツール
