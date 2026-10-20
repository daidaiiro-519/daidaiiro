# 共通の「はじめに」：確認用の作業場所

2026年10月19日、編集用ファイルとローカル閲覧ページをリポジトリ内へ移しました。

- [引き継ぎプロンプト](HANDOFF.md)：次の担当へ本文を入力してください。
- [現在地](workspace/seven-slide-revision-20261019/progress.json)：1〜3枚目の図を反映し、次は4枚目を制作します。
- [最新の案件設定](workspace/seven-slide-revision-20261019/case-setting.json)：承認段階の取り違えと、マージ後CIでの発覚を扱います。
- [7枚の構成・原稿案](workspace/seven-slide-revision-20261019/slide-plan.md)を確認できます。
- [参照ファイル一覧](reference-files.json)：元の保存先も含む一覧です。
- [移行の記録](migration.json)：書籍の表紙を含む表示用ファイルは公開履歴から除外しています。

生成は `python3 workspace/build_review.py --publish`、閲覧はリポジトリのルートから次を実行します。

```bash
python3 -m http.server 8772 --bind 127.0.0.1 --directory docs/workshops/ai-workshop/review/common-intro-20261019/viewer
```

比較は <http://127.0.0.1:8772/design-review.html?slide=3#lead> です。スライドの編集元は `workspace/`、公開するファイルは `viewer/`、確認用スクリプトは `tools/` にあります。生成と閲覧に `/tmp` は使用しません。正本と音声・動画は今回更新していません。
