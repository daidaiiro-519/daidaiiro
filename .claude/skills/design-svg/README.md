# design-svg の描画エンジン

作成者が記述した SVG の class を、図のデザインシステムの値へ解決し、検査1〜4を通して返す描画エンジン。
辺の多い関係図と量のグラフは、宣言（節点・辺・囲み）から生成することもできる。

**配置アルゴリズムも図形の重なりも自前で解く。** 依存する crate は、JSON の読み書き（`serde` ・
`serde_json`）・正規表現（`regex`）・要約（`sha1`）だけである。

## 使う

```
cargo install --force --path tool/cli --root . --target-dir tool/target
bin/design-svg resolve 図.svg --out 出力.svg
bin/design-svg figure 宣言.json --out 図.svg
```

`図.svg` は座標を持ち、スタイルを class だけで指定する（class の一覧は `references/theme.json` の `classes`）。

`宣言.json` は次の形である。

```json
{"nodes": [{"id": "a", "label": "受付"}, {"id": "b", "label": "検証", "role": "focus"}],
 "edges": [{"from": "a", "to": "b", "label": "渡す"}]}
```

量を描く部品は `chart`、描いた SVG の幾何の検査は `verify` である。

## 何を受け取れるか

```
bin/design-svg catalog   # 目録をJSONで出す
```

目録は、どの部品があり、それぞれがどんな値を読み、どんなトークンで見た目が決まり、
どの配置戦略が選べるかを持つ。**部品が読む値の表は、部品の関数の本文と試験で突き合わせる**
── 部品を直して表を直し忘れれば、`cargo test` が失敗する。

利用側の変換のコードを書く人は、これだけを参照すれば済む。**この目録が答えるのは「何を受け取れるか」
だけで、「何を表せるか」ではない。** 言い分の名前と、それをどの部品でどう描くかの対応表は
利用側の持ち物である。

## 見た目を変える

見た目は class が決める。class の値と、ダークモードの値の正本は `references/theme.json` である。
呼び出し元が上書きしてよいのは色のトークンだけで、上書きしたときはダークモードの `style` を出さない。

## 配置

層状・環状・放射の木・格子の4つが同じ契約を返し、宣言の `layout` で差し替える。
層状は Graphviz の `dot` と同じ系統（網状単体法で辺の長さの総和を最小化し、
等調回帰で層の中を詰める）。同じ図を `dot` にも解かせて突き合わせる計測のツールを
`tool/business_logic/examples/bench_layout.rs` に持つ。

## 開発

```
cd tool
cargo test                                          # 試験（移す前の出力を固定した事例を含む）
cargo clippy --all-targets -- -Dwarnings            # 静的検査
cargo run -q -p ds_business_logic --example bench_layout     # 本家との突き合わせ
```

規律は3条 ── 下から上を呼ばない／層ごとに型が変わり前の層を飛ばせない／
契約は中立が所有し実装が所有しない。詳しくは design-svg Skill の
`references/document.json` の `svg-engine-discipline` を読む（`design-svg get --kind document --id svg-engine-discipline`）。層の規約は `tool/business_logic/tests/contracts.rs` が検査する。
