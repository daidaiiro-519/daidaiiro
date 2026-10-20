# 07. PoC設計・検証計画

## フェーズ
1. **Contract MVP**: JSONメタモデル（Contract/Schema/Relation/Constraint）、識別子、バージョン、参照解決、Schema検証。
2. **Projection**: JSON Schema/OpenAPI、Rust/Go/TS/Pythonのうち最低2言語のDTO等、Frontend API型。
3. **Verification**: cargo/go/tsc/pyright等とSchema validator、アーキテクチャ依存検査、診断結果をJSON化。
4. **AI Loop**: AIの変更→自動検証→修正→収束測定。Semantic変更は承認ゲート。
5. **UI**: Schema-drivenなフォーム設定画面と通常コンポーネント方式の比較。

## 検証シナリオ
- Requestに必須フィールドを追加。
- Enumに状態を追加し既存分岐修正を漏らす。
- 数値の範囲を変更する。
- Responseを互換性を壊す形で変更する。
- Domain→Infrastructureの依存を故意に挿入。
- 既存DB一意制約を破るデータを生成。
- UIで未定義ActionやComponentを指定する。
- Schemaは通るが業務計算が誤るケースを入れる。

## 比較条件
同一の契約、同一要求、同等のテストセットを準備し、AIモデル・試行回数・予算を揃える。単にコンパイルエラーが少ない言語を勝者にせず、最終残存欠陥・必要な検証コストと開発時間を含める。

## 主要リスク
独自Coreの過度な一般化、Schema→言語型の制約欠落、生成器バージョンの不整合、正本の二重化、UI DSL肥大化、検証が通るだけで正しいと錯覚すること。
