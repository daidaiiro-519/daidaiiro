# 04. アーキテクチャと機械検証

## 参照構造
```
Human / AI
    ↓
Semantic Contract Core (identity, relations, constraints, vocabulary)
    ├── JSON Schema (data)
    ├── OpenAPI (HTTP)
    ├── UI Contract (presentation/interaction)
    └── Architecture Contract (allowed dependencies)
    ↓
Contract Engine (validate, resolve, project, diagnose)
    ├── Backend Adapters: Rust / Go / TypeScript / Python
    └── Frontend Adapters: Angular / Svelte / Vue / React
    ↓
Compilers + Validators + DB constraints + Tests
    ↓
Structured evidence → AI repair loop / human approval
```

## 各保証の境界
| 保証対象 | 代表的手段 | 限界 |
|---|---|---|
| JSON構造・値 | JSON Schema validator | Schemaの意図や業務妥当性は証明しない |
| HTTP入出力 | OpenAPI生成型 + 契約テスト | 実際の動作は別途検証 |
| Rust型・網羅性 | rustc / cargo check / clippy | 業務ルール全般の証明ではない |
| TS型 | tsc, UI checker | 型消去後の入力は実行時検証 |
| Python | 型チェッカー + Pydantic | 型ヒントは実行時保証ではない |
| Go | go build, go vet, tests | 状態表現の静的保証には設計上の限界 |
| 依存方向 | crate/package境界 + import検査 | 意図と責務分担は別途レビュー |
| 一意性・整合性 | DB unique / FK / transaction | 競合や運用上の条件もテスト |
| 業務仕様 | property tests / integration tests / review | テストにない条件は残る |

## 検証Evidence
候補ステータス: `verified`, `violated`, `unverified`, `unsupported`。必ず `verifier`, `scope`, `artifact`, `timestamp`, `diagnostic` と共に扱う。「verified」は実行された検査範囲での合格に過ぎない。

## AIの変更権限
- JSONインスタンス生成: Schema検証の範囲で自動化できる。
- UI設定変更: 許可されたComponent/Action Registryと契約検証の範囲で自動化可能。
- 実装変更: 型検査等を経て自動化を検討。
- 契約の意味変更: 原則、提案→差分検査→承認。
- 不変条件の変更: 人が責任を持って確認。

## 同期方針
双方向の完全同期は前提にしない。Contractを正本にしてDTOやAPI型等を一方向生成し、手書きUseCase/Domain/UIの契約適合性を検査する。実装からの意味抽出は変更提案として扱う。
