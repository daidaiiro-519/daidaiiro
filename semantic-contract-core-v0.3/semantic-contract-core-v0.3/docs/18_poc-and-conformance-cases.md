# 18. 最小PoCとConformance検証例

## 目的

「新しい表現ができること」「成熟技術による検証が可能なこと」「標準が対応しない意味が消えないこと」「Rust以外で実装したInterpreterでも同じプロトコルを使えること」を検証する。

## P0 — 仕様だけのPoC

- Vocabulary: `DurationSeconds`、`connectedTo`、`temporal-transition`。
- Concrete Schema: センサー計測間隔（許容値10/15/60秒）とGateway関連。
- Instance: 適合データ、不適合データ、未対応意味を含むデータ。
- Structured diagnostics: path、constraint、sourceVocabulary、capability、status。

## P1 — Rust Engineの実装

- JSON読み込みと基礎構造検証。
- Vocabulary IDと参照解決。
- JSON Schema 2020-12による構造検証。
- Interpreter Capabilityマッチング。
- `notRepresented`を含むProjection診断。

## P2 — Graph検証

- JSONデータから関係の一部をGraphに射影。
- Node・Relationへの基本制約をSHACL等で検証。
- Graph化されない情報が元JSONに残り、検証結果がpartial扱いになることを確認。

## P3 — 別言語と開発基盤

- 追加InterpreterをTypeScriptかPythonで作り、Rust EngineとJSON経由で相互運用。
- Concrete Development SchemaでOpenAPIとRust/TSの型生成、コンパイル検査。
- AIにContract変更や誤実装をさせ、診断から修正させる。

## 評価指標の例

| ものさし | 観測例 |
|---|---|
| 表現力 | 要求した意味が欠損せずContractに保持された率 |
| 検証力 | 作り込んだ違反を検出した割合と誤検知率 |
| 透明性 | 未対応Capabilityを成功として報告した件数（目標0） |
| 可搬性 | Rustと別言語Interpreterの同一Conformance Suite適合 |
| 射影品質 | lossless / partial / unsupported分類の妥当性 |
| AI修正収束 | 検証失敗から修正成功までの平均反復数 |

未着手の提案であり、実証結果ではない。
