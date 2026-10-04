---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、AI が扱うデータを構造化して持つための基盤（開発に限らない）として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-05、16回目、未 push）：
- 論点1（基盤の7つの契約）は決着。ACDR 0093 は適用済み
- 10回目：論点1を B⁗（開発に限らない AI の構造化データ管理の基盤。正本は構造化データ、Markdown と HTML は描画の結果。7つの契約 ＋ 注釈 x-ref ・ x-derive を読む道具）、論点2を B‴（x-test-spec とテストの記録の突き合わせは開発の concrete）に開き直し、承認済み（11回目で決着）。見本の道具は base7.py（基盤）と concrete7.py（開発）に分けた
- 14回目：論点3 B″ を決着（基盤は描画の部品と頁の型を読んで組む道具、concrete は頁の型をデータで書く。見本の全35頁が頁の型と部品で前と同じ HTML。頁の型のキーが約50に増えたので page.schema.json で絞るのが要求事項）。16回目：論点4 A を決着（基盤は Skill「schema-driven」。正本を持ち、作るものへ複製を転写し、check が複製と正本の差を報告する。concrete は use-case-driven-development の一式）
- 論点1〜4がすべて決着し、ボードで答えられる問いは尽きた。次は外の作業：①schema-driven をプロダクトとして宣言 ②サブドメインごとに実装の型を導く ③アーキテクチャ（外枠はポートとアダプタ、中の型は文脈ごと）と技術スタック（Rust の crate）を決める ④Rust で作る ⑤use-case-driven の Skill へ転写し差の検査を確かめる（論点4の試し）。③はいまの宣言の種類に置き場所が無く、usecase-modeling-coding の実装の側で扱う
- 2026-10-05 次にすること1を済ませた：`.brainstorming-board/schema-driven-base/product/`（build.py → decls 21件、run.py で検査、view.py で描画。Artifact https://claude.ai/artifact/GPSBWs5TVjxN8wKNtXG4EL 、75d0b1f2）。サブドメイン4（インスタンスの読み書き=一般 ・ 参照と導出値=中核 ・ 描画 ・ 転写=補完）、文脈3（インスタンスの操作と検査 ・ 描画 ・ 転写）、ユースケース11。欠け1件＝中核の文脈に集約が無い。2も済ませた（設計の側：集約 インスタンス ・ 承認記録、ドメインサービス 検査する。欠け0件、実装の型は 参照と導出値=ドメインモデル、ほか3つ=トランザクションスクリプト）。利用者に聞いている未決：作成直後の未記入をどう扱うか（未記入を検証エラーと分ける案）。次は3：アーキテクチャと技術スタック。3つの未決は利用者が承認（ずれが残る集合は承認しない ・ 手の変更がある複製へは転写しない ・ 書く途中で失敗しても前の内容のまま）。ACDR 0117（用語集の語に出どころ、仕様の用語を訳さない）承認 ・ 適用済み（84979096）。残り：skills-creator の document.json と no-more-spaghetti の schema-meta の「実体 ・ 案内 ・ 置き場所」を直す（別セッションの未コミットの変更が片付いてから）
- 前提に足した：スキーマ駆動は新しく定義する概念で、すべての基盤として一から作る。いまの Skill は基盤より前の試しで、根拠にしない（基盤のあとで移す）
- 論点4の前提：基盤から作るものには、基盤の能力の複製を転写する（skills-creator の雛形の契約と同じ形）
- 試作 `trial/spike/` は契約の動作確認だけに使う
