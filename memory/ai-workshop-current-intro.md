---
name: ai-workshop-current-intro
description: 三教材共通のはじめに。7枚構成を再検討中。6枚目の言葉の意味と設計判断への修正を利用者が確認する段階。
metadata:
  node_type: memory
  type: project
---

# 共通の「はじめに」の現在地

最新の作業コピーは `docs/workshops/ai-workshop/review/common-intro-20261019/workspace/`。進捗の記録は [progress.json](../docs/workshops/ai-workshop/review/common-intro-20261019/workspace/seven-slide-revision-20261019/progress.json)にある。

2026年10月24日、企画書・オリエンテーションの別担当との共有を開始した。[共同制作のメモリ](ai-workshop-shared.md)と[担当情報](../docs/workshops/ai-workshop/review/shared/tracks/intro.json)を追加した。

直近の修正では、3〜4枚目で、不具合修正後も残った設計から追加変更と確認負担へ進む接続を変更した。5枚目はレビュー中心から、設計・実装の目的から必要な作業を考える説明へ変更し、利用者確認待ち。6〜7枚目は最新の再構成方針への修正前である。とくに教材3を仕様参照中心に説明する7枚目を、そのまま採用済みとしない。

比較は <http://127.0.0.1:8772/design-review.html?slide=5#lead>。2026年10月24日にサーバーを起動してHTTP 200を確認した。プロセスは後のセッションで停止している可能性がある。

図と原稿の修正はレビュー用。正本・音声・動画は未変更。全編に対する最新のLunaレビューも未実施。過去の経緯は[旧記録](ai-workshop.md)とGit履歴、生成方法は[引き継ぎ](../docs/workshops/ai-workshop/review/common-intro-20261019/HANDOFF.md)を参照できる。古い引き継ぎに含まれる枚数や次の作業は、現在のprogress.jsonと異なる場合がある。

2026年10月24日、利用者の依頼で比較HTMLの操作と差分表示を改善した。図は並列・重ね合わせ、原稿は文単位の追加・削除と全文並列を切替可能。変更理由は独立表示。生成元はworkspace/review_ui.pyとreview-ui-shell.html、review-ui.css、review-ui.js。スライド・原稿自体と5枚目の確認待ちは変更していない。確認画像と結果はworkspace/ui-review-20261024/。

2026年10月24日、利用者の依頼で画面遷移図を作成し、メイン画面から比較・閲覧・制作情報へ進む導線を整理した。ルートはメイン、閲覧はmaterials.html、方針と進捗はworkflow.html、図はnavigation.html。build_review.pyがreview_navigation.pyから生成する。比較と閲覧は資料と枚を維持して往復できる。古いルートの資料リンクは閲覧へ移動する。図・原稿の入力は未変更で、5枚目の確認待ちは変わらない。

2026年10月24日、利用者がこのセッションは「はじめに」の修正担当であると再確認し、続きから完成するよう依頼した。6枚目を言葉の意味から設計に必要な区別を考える説明へ修正し、比較を更新した。最新の確認待ちは6枚目、次は7枚目。5枚目の個別承認は明示されていない。比較は <http://127.0.0.1:8772/design-review.html?slide=6#lead>。7枚目は以前の説明が残る。

2026年10月24日、利用者が事例と三教材の対応を指摘し、修正を依頼した。内容4〜7枚目を、設計の前提を見直さない追加変更でレビュー負担が増える問題から、目的・業務の意味・AIが実行する判断手順へ接続した。7枚目も修正済みとなったが、4〜7枚目は利用者の確認待ち。最新全編のLunaレビューと正式反映はまだ行っていない。比較は <http://127.0.0.1:8772/design-review.html?slide=4#lead>。

2026年10月24日、テスト工程を修正。マージ前に単体・既存の自動テストとレビューを行い、後の結合テストで業務シナリオを確認して不具合が発覚する設定へ変更した。2・3枚目の図と原稿、案件設定、共通方針 revision 2026-10-24-02 を更新。現在の描画は7枚で、8枚への構成案はまだ提案段階。比較は <http://127.0.0.1:8772/design-review.html?slide=2#lead>。
