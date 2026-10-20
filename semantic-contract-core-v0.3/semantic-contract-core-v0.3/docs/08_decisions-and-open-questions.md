# 08. 合意・仮説・未決論点

## 会話から明確な方向性（正式な実装決定ではない）
- AIによる誤実装の機械的な検出可能性を重要な技術選定軸にする。
- JSONベースでAIと各技術が依存度低く通信できるインターフェースを目指す。
- JSON Schema/OpenAPIといった成熟した標準は活用したい。
- TypeSpecの抽象をCoreの正本に直接採用せず、設計思想を参考にする。
- バックエンド比較にRust・Go・TypeScript・Pythonを含める。
- UIはAngular・Svelte・Vue・Reactを比較対象とし、Schema駆動を研究する。

## 提案だが未確定
- CoreをContract + Relation + Constraint + Interpretationにする。
- Core正本とJSON Schema等の標準語彙を分離する。
- Vocabulary/Capability Modelを設ける。
- Contract→生成物の一方向投影を基本とする。
- Semantic変更に承認ゲートを設ける。
- PoCでRust + SvelteKit、別案Go/TS/Python等を比較する。

## 次にブレストする問い
1. **Semanticとは何か？** データ意味・操作意味・不変条件・関係・目的をどこまで含むのか。
2. Contractの存在論的な最小単位は何か。Contractを上位に置きすぎないか。
3. JSON SchemaをCoreそのものにするのか、標準として内包/参照するのか。
4. 意味をどう検証可能な宣言へ落とすのか。形式検証可能・テスト可能・人間判断をどう区分するか。
5. UIの表現空間をどこまで閉じ、拡張可能にするか。
6. 意味モデルの更新と実装の一致をどこまで確認できるか。
7. Adapterごとの静的/動的保証差をどう可視化するか。
8. 正本の決定権、AI自動変更の権限モデルをどう設計するか。

## 次回セッション冒頭に使えるプロンプト
> Semantic Contract Core v0.1のREADMEと08を前提にブレストを再開したい。技術選定より先に「Semanticとは何か」を定義し、既存JSON Schemaの意味論、独自Vocabulary、検証可能性の境界を検討してほしい。決定済みと仮説は区別し、過度な抽象化への反証も示してほしい。
