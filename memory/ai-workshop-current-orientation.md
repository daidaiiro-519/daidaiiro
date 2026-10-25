---
name: ai-workshop-current-orientation
description: オリエンテーション見直し。現行14枚のLunaレビューと助言を受けた構成案が確認待ち。図と原稿は未修正。
metadata:
  node_type: memory
  type: project
---

# オリエンテーションの現在地

作業記録：ec12a63c（main）。構成案は未承認。

利用者が完成した共通の「はじめに」と同じ進め方で見直すことを依頼。企画書は次の工程。共通方針revisionは2026-10-25-13。

作業先はdocs/workshops/ai-workshop/review/proposal-orientation-20261024/orientation-20261025/。現行の全14枚を新たに描画し、Lunaと助言担当が確認。story-plan.mdと.jsonの構成案を利用者へ提示した。図と原稿は未修正、正本と音声動画は変更していない。

比較は <http://127.0.0.1:8772/design-review.html?deck=orientation&slide=1#orientation-plan>。比較入力comparison.json、構成案story-plan.mdを担当情報から共通生成元へ読み込ませる。HTMLを直接編集しない。生成はbuild_orientation_review.py、その後common-intro-20261019/workspace/build_review.py --publish。

次は構成案の確認を受け、1枚目の表紙から図・見出し・導入文・専用原稿を修正する。現行表紙の専用原稿は未記録なので、原稿差分を推測しない。

到達像は仕事全体の生産性と品質の改善。6枚目の問い、7枚目のAIの使い方、8枚目の三教材、9枚目の応用、10枚目の書籍の理由を区別する。一般AI初心者向けへ広げず、PM・PLへ不自然な実装作業を割り当てない。Lunaの感想の採否はstory-plan.mdにある。

書籍表紙を含む元HTMLと画像は.artifact-backup/ai-workshop-orientation-review-20261025/に保存し公開コミット対象外。比較用HTMLは既存画像への参照とし、source-manifest.jsonに原本ハッシュを記録した。
