# 16. 用途別Concrete Schemaと利用するPlatform／Runtime

## 設計境界

Abstract Schemaという「形式・規則」→各用途Concrete Schemaという「定義」→開発基盤／Runtimeという「使用ソフトウェア」。

| Concrete用途 | Schemaの例 | 利用システム | 実装技術の候補 |
|---|---|---|---|
| Development | Artifact / Dependency / Verification / API | Schema駆動開発基盤（AI修正Loop、Generator、Test） | Rust / Go / TS / Python |
| UI | Component / Field / Action / State | Renderer・Form Runtime | Angular / Svelte / Vue / React |
| Workflow | State / Transition / Guard | Workflow Runtime | 任意のEngine |
| Data Integration | Mapping / Relation / Ingestion | ETL / Adapter | HTTP / Batch / Event |
| AI Agent | Tool Permission / Output / Capability | Agent Runtime | 任意のAgent基盤 |

## 開発基盤の責務

- Concrete Development Schemaに従い、コード・型・API・UIのGeneratorを呼ぶ。
- 外部標準へ射影する（OpenAPI、JSON Schemaなど）。
- Rustの`cargo check`等の実装言語固有チェック、TSの型検査、PythonのPydantic検証等を利用。
- 構造・型の適合と、意味的正しさ（業務ロジック）が区別されていることを診断する。
- AIは構造化診断を利用して修正する。意味的契約変更は確定権限を区別する。

## UIについての注意

定型的なフォームや管理画面はSchema駆動Rendererに向く。高度なインタラクションや独自表現はフレームワークに保持してよい。UIすべてをJSON化することを目標にしない。

## 技術選定の留保

RustはFoundation Engineの現時点の第一候補であり、**Development Platformのバックエンドが必ずRustになるとは限らない**。Rust / Go / TS / Python比較は継続。Angular / Svelteも検証器・Renderer・チーム特性で評価し、未決。
