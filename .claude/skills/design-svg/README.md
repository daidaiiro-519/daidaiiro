# design-svg の描画エンジン

宣言（節点・辺・囲み）から、その場でSVGを組み立てる描画エンジン。

**配置アルゴリズムも図形の重なりも自前で解く。** 依存する crate は、JSON の読み書き（`serde` ・
`serde_json`）・正規表現（`regex`）・要約（`sha1`）だけである。

## 使う

```
cd rs && cargo build --release
bin/design-svg figure 宣言.json --out 図.svg
```

`宣言.json` は次の形である。

```json
{"nodes": [{"id": "a", "label": "受付"}, {"id": "b", "label": "検証", "role": "focus"}],
 "edges": [{"from": "a", "to": "b", "label": "渡す"}]}
```

量を描く部品は `chart`、画布へ直に置くものは `canvas`、描いた SVG の検査は `verify` である。

## 何を受け取れるか

```
bin/design-svg catalog   # 目録をJSONで出す
```

目録は、どの部品があり、それぞれがどんな値を読み、どんなトークンで見た目が決まり、
どの配置戦略が選べるかを持つ。**部品が読む値の表は、部品の関数の本文と試験で突き合わせる**
── 部品を直して表を直し忘れれば、`cargo test` が失敗する。

利用側の変換器を書く人は、これだけを参照すれば済む。**この目録が答えるのは「何を受け取れるか」
だけで、「何を表せるか」ではない。** 言い分の名前と、それをどの部品でどう組むかの対応表は
利用側の持ち物である。

## 見た目を変える

3層で解決する。テーマの既定値 → 役割による上書き → その場の上書き。既定値の正本は
`references/theme.json` である。役割の中身もテーマが `role.<名前>.<トークン名>` という平らな名前で
持つので、**新しい役割はテーマへ行を足すだけで増える**（エンジンには触れない）。

焼き上がったSVGは、生成後の外部CSSでも上書きできる。

## 配置

層状・環状・放射の木・格子の4つが同じ契約を返し、宣言の `layout` で差し替える。
層状は Graphviz の `dot` と同じ系統（網状単体法で辺の長さの総和を最小化し、
等調回帰で層の中を詰める）。同じ図を `dot` にも解かせて突き合わせる計測の道具を
`tool/parts/examples/bench_layout.rs` に持つ。

## 開発

```
cd rs
cargo test                                          # 試験（移す前の出力を固定した事例を含む）
cargo clippy --all-targets -- -Dwarnings            # 静的検査
cargo run -q -p ds_parts --example bench_layout     # 本家との突き合わせ
```

規律は3条 ── 下から上を呼ばない／層ごとに型が変わり前の層を飛ばせない／
契約は中立が所有し実装が所有しない。詳しくは design-svg Skill の
`references/knowledge/svg-engine-discipline.md` を読む。層の規約は `tool/parts/tests/contracts.rs` が検査する。
