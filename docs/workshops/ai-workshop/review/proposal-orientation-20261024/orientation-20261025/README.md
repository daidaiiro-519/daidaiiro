# オリエンテーションの見直し

2026年10月25日、完成した共通の「はじめに」に合わせ、現行14枚を土台にオリエンテーションの見直しを開始した。構成案は利用者の確認待ち。スライドと原稿はまだ修正していない。企画書は次の工程で扱う。

現行の全画像を受講者Lunaが確認し、原稿を読んだ感想をreview/luna-current-feedback.mdへ保存。助言担当の検討はreview/story-advice.jsonと.html。これらを踏まえた構成案はstory-plan.mdと.json。

変更前はbefore/。現行の入力と生成元を保存し、source-manifest.jsonへ元ファイルのハッシュを記録した。before/slides.htmlは書籍表紙の埋め込み画像だけを既存画像への参照に変えている。元のHTMLと全14枚の画像は.artifact-backup/ai-workshop-orientation-review-20261025/に保存し、公開リポジトリへ追加しない。

変更後の編集元はafter/material-1/src/。canonicalの生成処理を作業コピーで使用する。build_orientation_review.pyがローカルの保管先へ描画用HTMLを生成し、書籍表紙の画像を埋め込まないafter/slides.htmlとcomparison.jsonを出力する。revisions.jsonが反映済みの枚と今回の変更理由を持つ。現在の反映済みは空で、変更後は未修正と表示する。

生成はリポジトリのルートで次の順に実行する。

```bash
python3 docs/workshops/ai-workshop/review/proposal-orientation-20261024/orientation-20261025/build_orientation_review.py
python3 docs/workshops/ai-workshop/review/common-intro-20261019/workspace/build_review.py --publish
```

比較は <http://127.0.0.1:8772/design-review.html?deck=orientation&slide=1#orientation-plan>。共通の比較HTMLを直接編集せず、担当情報のorientation_review_inputとorientation_review_planを介して反映する。正本・音声・動画は変更していない。
