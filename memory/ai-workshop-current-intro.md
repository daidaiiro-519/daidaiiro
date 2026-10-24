---
name: ai-workshop-current-intro
description: 三教材共通のはじめに。要件と設計を軸に8枚の構成を改訂。1枚目を図・原稿へ反映し確認待ち。
metadata:
  node_type: memory
  type: project
---

# 共通の「はじめに」の現在地

最新の作業コピーは `docs/workshops/ai-workshop/review/common-intro-20261019/workspace/`。進捗の記録は [progress.json](../docs/workshops/ai-workshop/review/common-intro-20261019/workspace/progress.json)にある。

2026年10月24日、企画書・オリエンテーションの別担当との共有を開始した。[共同制作のメモリ](ai-workshop-shared.md)と[担当情報](../docs/workshops/ai-workshop/review/shared/tracks/intro.json)を追加した。

現在は、要件と設計を軸に8枚の構成を改訂した段階。workspace/slide-structure.jsonが見出し・導入文・原稿案・図の役割と改訂前を持つ。review_structure.pyが同じ比較HTMLへ表示する。1枚目は図と原稿へ反映済みで利用者確認待ち。2〜8枚目は構成案で、図の改修は一枚ずつ進める。

比較は <http://127.0.0.1:8772/design-review.html?slide=5#lead>。2026年10月24日にサーバーを起動してHTTP 200を確認した。プロセスは後のセッションで停止している可能性がある。

図と原稿の修正はレビュー用。正本・音声・動画は未変更。全編に対する最新のLunaレビューも未実施。過去の経緯は[旧記録](ai-workshop.md)とGit履歴、生成方法は[引き継ぎ](../docs/workshops/ai-workshop/review/common-intro-20261019/HANDOFF.md)を参照できる。古い引き継ぎに含まれる枚数や次の作業は、現在のprogress.jsonと異なる場合がある。

2026年10月24日、利用者の依頼で比較HTMLの操作と差分表示を改善した。図は並列・重ね合わせ、原稿は文単位の追加・削除と全文並列を切替可能。変更理由は独立表示。生成元はworkspace/review_ui.pyとreview-ui-shell.html、review-ui.css、review-ui.js。スライド・原稿自体と5枚目の確認待ちは変更していない。確認画像と結果はworkspace/ui-review-20261024/。

2026年10月24日、利用者の依頼で画面遷移図を作成し、メイン画面から比較・閲覧・制作情報へ進む導線を整理した。ルートはメイン、閲覧はmaterials.html、方針と進捗はworkflow.html、図はnavigation.html。build_review.pyがreview_navigation.pyから生成する。比較と閲覧は資料と枚を維持して往復できる。古いルートの資料リンクは閲覧へ移動する。図・原稿の入力は未変更で、5枚目の確認待ちは変わらない。

2026年10月24日、利用者がこのセッションは「はじめに」の修正担当であると再確認し、続きから完成するよう依頼した。6枚目を言葉の意味から設計に必要な区別を考える説明へ修正し、比較を更新した。最新の確認待ちは6枚目、次は7枚目。5枚目の個別承認は明示されていない。比較は <http://127.0.0.1:8772/design-review.html?slide=6#lead>。7枚目は以前の説明が残る。

2026年10月24日、利用者が事例と三教材の対応を指摘し、修正を依頼した。内容4〜7枚目を、設計の前提を見直さない追加変更でレビュー負担が増える問題から、目的・業務の意味・AIが実行する判断手順へ接続した。7枚目も修正済みとなったが、4〜7枚目は利用者の確認待ち。最新全編のLunaレビューと正式反映はまだ行っていない。比較は <http://127.0.0.1:8772/design-review.html?slide=4#lead>。

2026年10月24日、テスト工程を修正。マージ前に単体・既存の自動テストとレビューを行い、後の結合テストで業務シナリオを確認して不具合が発覚する設定へ変更した。2・3枚目の図と原稿、案件設定、共通方針 revision 2026-10-24-02 を更新。現在の描画は7枚で、8枚への構成案はまだ提案段階。比較は <http://127.0.0.1:8772/design-review.html?slide=2#lead>。

2026年10月24日、8枚構成への続きの作成依頼を受け、新5枚目を追加した。表示順はworkspace/review-layout.jsonが管理する。新5枚目はslide-05-growth.json、教材紹介の新6〜8枚目は従来のslide-05〜07.jsonを入力として使う。旧承認済み8枚目は上書きしない。共通方針revision 2026-10-24-03に、設計を見直さず変更を重ねた場合に想定される展開と説明上の区別を共有した。新5枚目は利用者確認待ち。比較は <http://127.0.0.1:8772/design-review.html?slide=5#lead>。


2026年10月24日、利用者が、設計・実装時に誤解を防ぐことと、レビューで誤りを拾うことの両方が必要と確認した。教材3の見出し・図・原稿に両方を明記し、AIによるレビューと人によるレビューの双方を含めた。比較は <http://127.0.0.1:8772/design-review.html?slide=8#lead>。新5枚目と8枚目を確認待ち。詳細は[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-24-11-intro-review-and-design.md)と[担当情報](../docs/workshops/ai-workshop/review/shared/tracks/intro.json)を参照。


2026年10月24日、利用者が「はじめに」の狙いを補足。AIで実装が速くても、業務の意味・設計を扱いきれず、AIと人のレビューも同じ前提を見逃すと、変更が重なって確認負担が増える。そのため実装を軽視せず、設計が業務理解を実装・レビューへ導く理由を伏線にし、教材でアーキテクチャとDDDの有用性、AIへの具体化まで回収したい意向。アドバイザー確認では流れは妥当だが、DDD・アーキテクチャを実際に教材で教える範囲の照合が必要。具体的なスライド案は未確定。共有記録: [伏線の狙い](../docs/workshops/ai-workshop/review/shared/changes/2026-10-24-12-intro-design-ddd-foreshadowing.md)。アドバイザーHTMLはworkspace/story-advisory-20261024/。


2026年10月24日、利用者がアーキテクチャの狙いを補足。各方式の詳細ではなく、依存関係・境界・責務を適切に設計し、業務モデルと実装を保つ考え方が主題。ここを誤ると、実装が設計から乖離し得る。AIの速い実装に対して、設計を実装上の制約とAI・人のレビューの照合基準にする伏線を置き、教材でDDDとAgents・Skillsへの具体化を回収したい。方式の採用やAIにDDDが必須という主張はしない。共通方針 revision 2026-10-24-05 と[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-24-13-intro-architecture-principle.md)を参照。


2026年10月24日、利用者がAI時代の教材で要件と設計の双方を重視し、三冊を基盤にしたいと補足。具体と抽象は目的・要件の高さを見極める考え方、ユースケースは目的・範囲・シナリオを通じた要件整理、DDDは業務モデルを設計・責務・境界・依存関係へ反映する支えとして相互補完する。要件が無かったと決めつけず、既存仕様と今回の作業要件・設計・実装・テスト・レビューの結びつきを見る。8枚の構成案を作成したが既存スライドは未変更。詳細は[共有記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-24-14-intro-requirements-and-design.md)、[ストーリー案](../docs/workshops/ai-workshop/review/common-intro-20261019/workspace/story-advisory-20261024/requirements-design-story-map.md)。


2026年10月24日、利用者の依頼で要件と設計を軸に8枚の構成を改訂。比較HTMLの上部に構成案を表示し、各枚の見出し・導入文・原稿案・図の役割を改訂前と比較できる。表示URLは <http://127.0.0.1:8772/design-review.html?deck=lead&structure=1#lead>。1枚目の図と原稿にも反映し、warm-paperで描画確認した。独立した確認者の指摘で、既存仕様がある設定、7枚目の重複、状態数を設計の正誤と同一視する説明を修正した。これは全編Lunaレビューではない。次は利用者確認後に2枚目。
