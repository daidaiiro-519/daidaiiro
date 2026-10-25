# オリエンテーションの見直し

2026年10月25日、完成した共通の「はじめに」に合わせ、現行14枚を土台にオリエンテーションの見直しを開始した。2026年10月26日、全14枚の図と原稿を作業コピーで修正し、利用者が1枚ずつ確認した。続けて全14枚の通し確認と、受講者（Haiku 5.5）のレビューを1枚ずつ2回と通し読み1回行い、採った指摘を反映した。正本への反映はACDRと利用者の承認を経て行う。企画書は次の工程で扱う。

現行の全画像を受講者Lunaが確認し、原稿を読んだ感想をreview/luna-current-feedback.mdへ保存。助言担当の検討はreview/story-advice.jsonと.html。これらを踏まえた構成案はstory-plan.mdと.json。

変更前はbefore/。現行の入力と生成元を保存し、source-manifest.jsonへ元ファイルのハッシュを記録した。before/slides.htmlは書籍表紙の埋め込み画像だけを既存画像への参照に変えている。元のHTMLと全14枚の画像は.artifact-backup/ai-workshop-orientation-review-20261025/に保存し、公開リポジトリへ追加しない。

変更後の編集元はafter/material-1/src/。canonicalの生成処理を作業コピーで使用する。build_orientation_review.pyがローカルの保管先へ描画用HTMLを生成し、書籍表紙の画像を埋め込まないafter/slides.htmlとcomparison.jsonを出力する。revisions.jsonが反映済みの枚・確認済みの枚と変更理由を持つ。各枚の作り直す前の版はiterations/に、受講者レビューの入力・回答・採否はreview/haiku-review-20261026*/にある。

生成はリポジトリのルートで次の順に実行する。

```bash
python3 docs/workshops/ai-workshop/review/proposal-orientation-20261024/orientation-20261025/build_orientation_review.py
python3 docs/workshops/ai-workshop/review/common-intro-20261019/workspace/build_review.py --publish
```

比較は <http://127.0.0.1:8772/design-review.html?deck=orientation&slide=1#slide-comparison>。共通の比較HTMLを直接編集せず、担当情報のorientation_review_inputとorientation_review_planを介して反映する。正本・音声・動画は変更していない。
