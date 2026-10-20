# 09. 議論の推移（逐語録ではなく要約）

1. Rustのマシン負荷: コンパイル時と実行時を区別。TokioのTask大量生成による負荷はGoのgoroutineとも共通する問題。
2. Rust vs Go: 一般APIではGoのシンプルさに利点。Rustは所有権・enum・型状態などによる静的保証が魅力。
3. Rustバックエンド: Axum/Tokio/Hyper/Tower/Serde等の成熟したライブラリを用い、低レイヤーHTTPを自作しない。
4. AI時代の技術選定: 開発速度だけでなく、AI生成実装の機械的検証可能性を評価。React/Vueへの懸念とSvelteのコンパイラ設計への関心。
5. JSON Schema: AIによるJSON直接生成とSchema妥当性検証の利点。業務上の全意味までは表現できない。
6. Angular: template checking、DI、フォーム、規約の多さはAIの検証可能性に寄与し得る。
7. TypeSpec: 上位仕様からOpenAPI/JSON Schemaを生成する例。ただし発想の抽象が既に決まっているため、独自Coreの正本には不適。
8. 既存技術と独自技術: Validator/標準/コンパイラは再利用し、意味モデル・Vocabulary・Constraint・Projectionは自設計候補。
9. OpenAPIとRust: Contract-firstでRust DTOやAPI境界へ展開可能。OpenAPIはHTTP特有の契約として位置付ける。
10. Coreの方向: JSONを低依存の共通表現とし、自由度をSchemaの制約内に収める。意味のあるContractを通してAI/Frontend/Backendをつなぐ。
11. バックエンド対象追加: Rust/GoだけでなくTypeScript/Pythonも検証対象。
12. 次テーマ: 「Semanticとは何か」を定義する前に、全構想を一式保存する。
