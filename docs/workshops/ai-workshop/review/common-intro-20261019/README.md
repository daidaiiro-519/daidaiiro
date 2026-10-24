# 共通の「はじめに」：確認用の作業場所

共同制作は[共通の入口](../shared/README.md)から確認できます。企画書・オリエンテーションの別担当も、同じ方針と制作ガイドを参照します。

2026年10月19日、編集用ファイルとローカル閲覧ページをリポジトリ内へ移しました。

- [引き継ぎプロンプト](HANDOFF.md)：次の担当へ本文を入力してください。
- [現在地](workspace/seven-slide-revision-20261019/progress.json)：前の展開では1〜4枚目の図を反映済みです。最新の展開を4・5枚目の図と原稿に反映しました。次は6枚目です。
- [最新の案件設定](workspace/seven-slide-revision-20261019/case-setting.json)：承認段階の取り違えと、マージ後CIでの発覚を扱います。
- [7枚の構成・原稿案](workspace/seven-slide-revision-20261019/slide-plan.md)を確認できます。
- [参照ファイル一覧](reference-files.json)：元の保存先も含む一覧です。
- [移行の記録](migration.json)：書籍の表紙を含む表示用ファイルは公開履歴から除外しています。

生成は `python3 workspace/build_review.py --publish`、閲覧はリポジトリのルートから次を実行します。

```bash
python3 -m http.server 8772 --bind 127.0.0.1 --directory docs/workshops/ai-workshop/review/common-intro-20261019/viewer
```

比較は <http://127.0.0.1:8772/design-review.html?slide=5#lead> です。スライドの編集元は `workspace/`、公開するファイルは `viewer/`、確認用スクリプトは `tools/` にあります。生成と閲覧に `/tmp` は使用しません。正本と音声・動画は今回更新していません。

## 比較ページのUI

図と原稿の内容を変えず、資料・枚の切替、重ね合わせ、原稿の文単位の追加・削除、変更理由を確認できます。UIの生成元はworkspace/review_ui.pyとreview-ui-shell.html、review-ui.css、review-ui.jsです。build_review.py --publishで同じ比較ページを再生成します。

操作の確認は `node tools/check_review_ui_20261024.cjs`。確認画像と結果はworkspace/ui-review-20261024/に保存しています。旧原稿が保存されていない資料は、変更後のみ表示します。

## メイン画面と画面遷移

8772番ポートのルートは作業の入口です。比較はdesign-review.html、通しの閲覧はmaterials.html、共通方針と進捗はworkflow.html、画面遷移図はnavigation.htmlで確認できます。従来のルートの資料リンクも閲覧画面へ移動します。

生成元はworkspace/review_navigation.pyとreview-navigation.css。materials-source.htmlは以前の閲覧画面の元の入力です。はじめにの図と原稿は現在の制作元から反映し、古い動画は変更前と表示します。build_review.py --publishで同時に生成するため、公開HTMLだけを直接編集しないでください。

確認は `node tools/check_review_navigation_20261024.cjs`。画面遷移図の入力はworkspace/navigation.source.svgとnavigation.mmd、確認画像と結果はworkspace/navigation-review-20261024/です。
