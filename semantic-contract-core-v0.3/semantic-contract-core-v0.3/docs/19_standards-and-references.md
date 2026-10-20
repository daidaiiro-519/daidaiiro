# 19. 既存標準と公式の参照先

本文の技術的説明の参照候補。各規格の**概念**を理解するための資料であり、Foundationの抽象仕様がこれらに従属することを意味しない。

- JSON Schema公式Specification（Core, Validation, Meta-Schemas, Vocabulary）: https://json-schema.org/specification
- JSON-LD 1.1（W3C Recommendation）: https://www.w3.org/TR/json-ld11/
- JSON-LD 1.1 Processing Algorithms and API: https://www.w3.org/TR/json-ld11-api/
- RDF 1.1 Concepts: https://www.w3.org/TR/rdf11-concepts/
- SHACL（W3C Recommendation）: https://www.w3.org/TR/shacl/
- OpenAPI仕様: https://spec.openapis.org/oas/latest.html
- Rust serde: https://serde.rs/
- Rust jsonschema: https://docs.rs/jsonschema/latest/jsonschema/
- Rust Axum: https://docs.rs/axum/latest/axum/
- WIT / WebAssembly Component Model: https://component-model.bytecodealliance.org/design/wit.html

### 読み取り上の注意

- JSON-LDのContextは識別子の対応を提供するが、業務意味の正しさを直接証明しない。
- SHACLはRDFグラフに対するShape制約。JSON Schemaの単純な置換ではない。
- OpenAPIはHTTP APIの具体契約であり、Foundationの抽象的な意味モデルではない。
- ライブラリやGeneratorの対応範囲は実際にPoCで検証する。すべてのSchema制約がRust等の静的型に完全反映されるわけではない。
