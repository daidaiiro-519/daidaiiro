# 06. UIとフロントエンド

## 技術候補
- **Angular**: 統合CLI、DI、ルーティング、フォーム、strict template checking。強い規約が有利な可能性。フレームワーク固有概念の複雑さは懸念。
- **Svelte/SvelteKit**: コンパイラ中心、svelte-checkなど。軽量な独自Rendererを設計する候補。AI生成品質の優位性は未実証。
- **Vue**: SFC/テンプレート型検査、柔軟なエコシステム。Schema-driven UIライブラリと接続可能。
- **React**: TS/JSX、成熟したエコシステム。UI規約は採用構成による。JSON Forms等既存資産。

## UIの3層
1. Data: スキーマに従う値・編集対象。
2. Interaction: 操作、イベント、状態遷移、権限。
3. Presentation: 配置、外観、アニメーション、応答性。
全てのPresentationをJSONだけで表すと独自プログラミング言語を作る危険がある。

## 方式A
通常のAngular/Svelte/Vue/Reactコンポーネントに、生成API型とRuntime Validatorを組み込む。

## 方式B
許されたComponent Registry / Action Registry / Layout語彙を独自UI Schemaで表現し、AIは妥当なJSONインスタンスを生成、Rendererが表示する。フォーム・設定画面などの定型UIでPoCし、任意コード実行を禁止する。

## 既存技術
JSON FormsはJSON Schemaに基づくフォームUIを提供。React/Angular/Vue向け統合がある。一方、Svelteの同等の公式統合を前提にはできない。既存資産との適合度は評価が必要。

## 重要な区別
AIがコードを書いて新規コンポーネントを生成するケースと、AIがSchemaで許可された既存コンポーネントをJSONで組み立てるケースは、保証範囲が異なる。
