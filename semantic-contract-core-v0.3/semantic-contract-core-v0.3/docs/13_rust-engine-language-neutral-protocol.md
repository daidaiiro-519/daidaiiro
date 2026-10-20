# 13. Rust Engineと、言語非依存の意味仕様を両立する

## 方向性

**Rust-native implementation, Language-agnostic semantics.**

RustはFoundation Engineの第一候補。ただし **Abstract Schema / Vocabulary / Concrete Schema / Interpreter Protocol / 診断仕様** をRust固有の型・trait・crateに従属させない。

## 独立させる3つの境界

1. **Semantic Specification:** 宣言形式、識別子、参照解決、制約の意味、適合性、Capabilities、診断の意味を言語非依存に仕様化。
2. **Engine Implementation:** RustによるResolver / Validator / Relation Engine / Registry / Interpreter Manager等の実装。内部ではenumやtrait等を使ってよい。
3. **External Interpreter Protocol:** JSON I/Oを標準の交換形式とし、CLI / IPCをPoCの通信手段にする。必要に応じてHTTP、WASM Component + WIT等もAdapterに追加。

## Rust crate分割案（実装上の仮説）

```text
engine/
  crates/
    semantic-model/       # 内部型（外部仕様ではない）
    semantic-resolver/    # IDとVocabularyの解決
    semantic-validator/   # 構造検証・制約検証
    semantic-relations/   # 関係・Graph射影
    semantic-interpreter/# Capability照合と実行調停
    semantic-protocol/   # JSON I/O / Diagnostics
  bin/semantic-cli/
```

## Interpreter Protocolの最低限必要な概念

- protocol version, operation, vocabulary references, contract reference, instance payload
- required capabilities, interpreter implementation / version
- outcome: `conformant`, `nonconformant`, `unsupported`, `unverified`, `error`（状態遷移・定義は**未決**）
- structured diagnostics: focus, path, constraint, severity, provenance
- 安全性: 外部Interpreterへのアクセス権、ネットワーク解決可否、計算量と実行時間制約

**注意:** 次の `examples/protocol` はあくまで互換性検討用の提案例。実行可能なプロトコルとして確定していない。

## 言語間互換性を保証するためのConformance Suite

- 同じContract+Instanceで別実装が同じ検証結果になるか。
- 参照解決／バージョン／文字列正規化／整数・小数精度／日付時刻／未対応語彙／循環参照などを検証。
- 旧実装と新実装で同じ意味となることをテスト。単なる同じJSON形状だけでは不足。
- 必須Capabilityを欠くときには**fail-closed**（成功とみなさない）。

## Rustで検討する既存技術

- `serde`, `serde_json`: JSON表現とRust型の橋渡し。
- `jsonschema`: JSON Schema検証。
- `json-ld`: Linked Dataの処理。具体的な互換・成熟度はPoCで確認。
- `oxigraph`: 必要な場合のRDF/SPARQL用候補。
- SHACL処理: Rustライブラリまたは外部の成熟実装をAdapter越しに活用（全仕様対応を前提としない）。

既存ライブラリの選択は内部実装の問題。**言語非依存の公開仕様を変更する理由にしてはならない。**
