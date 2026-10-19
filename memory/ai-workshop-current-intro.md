---
name: ai-workshop-current-intro
description: 三教材共通のはじめにを7枚構成へ変更中。マージ後CIと承認段階の取り違えが最新設定。
metadata:
  node_type: memory
  type: project
---

# AIワークショップ：共通のはじめにの現在地

2026年10月19日、作業コピーとローカル閲覧ページ、確認用スクリプトを `docs/workshops/ai-workshop/review/common-intro-20261019/` へ移した。旧 `.artifact-backup/ai-workshop-common-intro-story-20261018/` と `/tmp/ai-workshop-viewer-20261017/` は移行前の履歴である。

引き継ぎは [HANDOFF.md](../docs/workshops/ai-workshop/review/common-intro-20261019/HANDOFF.md) にある。現在の編集先はその下の `workspace/`、公開先は `viewer/`、確認用スクリプトは `tools/` である。最新の設定と現在地は `workspace/seven-slide-revision-20261019/case-setting.json` と `progress.json` にある。

経費精算システム刷新の架空案件で、上長承認後から変更不可という本来の仕様を、経理承認後からと理解した設定へ変更した。開発者は資料を確認していたが、解釈を照合しきれず、AIの実装・テストとAI・人のレビューにも取り違えが残る。レビュー承認後にマージし、マージ後のCIで既存の承認フローを自動テストして不具合が発覚する。

新しい構成は表紙を除いて7枚である。図と原稿を反映済みなのは内容1・2枚目だけで、3枚目以降は構成・原稿案を更新した段階である。比較の選択肢はまだ旧10枚を含む。次は2枚目の利用者の確認を受け、新しい内容3枚目の図と原稿を作る。

比較は <http://127.0.0.1:8772/design-review.html?slide=2#lead>。リポジトリ内のviewerを公開するサーバーへ切り替えて確認した。サーバーは再開時に停止している可能性がある。起動方法は引き継ぎに記載した。

正本、音声、動画は今回変更していない。旧 `agreed-story.json`・`sequence.json`・10枚全編レビュー・以前のLunaレビューは最新7枚案への評価ではない。以前のACDR 0196も今回の案への承認として扱う段階ではない。
