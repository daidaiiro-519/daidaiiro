# 共同制作の閲覧画面と生成元

2026年10月24日、利用者が画面遷移図とメイン画面からの導線を確認し、別セッションも使えるよう共有資料へ追加するよう依頼した。これは画面構成の確認であり、教材の内容や動画の承認とは別である。

## 作業別の入口

| 確認すること | ローカルサーバーのURL |
|---|---|
| 作業を選ぶメイン画面 | <http://127.0.0.1:8772/> |
| 図・原稿の変更前後と変更理由 | <http://127.0.0.1:8772/design-review.html?slide=5#lead> |
| 資料を通して見る | <http://127.0.0.1:8772/materials.html#lead> |
| 制作方針と担当別進捗 | <http://127.0.0.1:8772/workflow.html> |
| 画面遷移図と操作の順序 | <http://127.0.0.1:8772/navigation.html> |

上の比較URLの5は、共有した時点の確認対象である。以後の確認対象は担当情報とメイン画面から読む。資料の選択はdeck、枚の選択はslideでURLに記録する。資料のIDはbrief、detail、orientation、leadである。

メイン画面には、比較、通しの閲覧、制作情報の三つの入口がある。企画書・オリエンテーション・共通の「はじめに」の資料名からも移動できる。以前のルートの#lead、#orientation、#brief、#detailは、移動後の資料閲覧画面へ進む。

2026年10月24日、共通の「はじめに」は確認用の8枚構成になった。表示順はworkspace/review-layout.jsonが持ち、review_deck.pyを比較と閲覧の生成処理が共通で使う。新5枚目はslide-05-growth.json、6〜8枚目の入力名は従来のslide-05.json〜slide-07.jsonである。ファイル名の番号だけで表示位置を判断しない。

新規追加の枚には変更前の図と原稿が存在しない。比較ではその旨を表示し、原稿差分を作らない。移動した枚は、対応する旧版を同じ位置に表示して比較する。

## 比較と閲覧の往復

比較画面の「この資料を通して見る」から、選択した資料と枚を引き継いで閲覧できる。閲覧画面の「この資料の変更を確認」で、同じ資料と枚の比較へ戻る。各画面にはメイン画面へのリンクがある。

比較内の並列・重ね合わせ、原稿差分・全文、変更理由と構成の確認は、同じ画面内の切替と移動である。ページの移動とは区別する。図の拡大、PDF・原稿ファイルの表示は別タブで行う。遷移の詳細は画面遷移図にある。

## 編集元と生成先

以下のパスは `docs/workshops/ai-workshop/review/common-intro-20261019/` からの相対パスである。

| 対象 | 編集する入力・生成元 | 生成先 |
|---|---|---|
| 比較する図・原稿・変更理由 | workspace/review-layout.json、各JSONとSVG、build_review.py、保存した変更前とログ | viewer/common-intro-review/、viewer/design-review.html |
| 比較ページのUI | workspace/review_ui.py、review-ui-shell.html、review-ui.css、review-ui.js | viewer/design-review.html |
| メイン、閲覧、制作情報の導線 | workspace/review_navigation.py、review-navigation.css | viewer/index.html、materials.html、workflow.html、review-navigation.css |
| 通しの閲覧の元 | workspace/materials-source.html。はじめにの図・原稿は現在の制作元から反映 | viewer/materials.html |
| 画面遷移図 | workspace/navigation.source.svg、navigation.svg、navigation.mmd | viewer/navigation.html |
| 制作方針と進捗の表示 | review/shared/の資料と担当情報、はじめにのprogress.json | viewer/workflow.htmlとメインの状態表示 |

最後の行のreview/shared/は、ai-workshop配下の共有資料を指す。共有資料を更新しても、公開HTMLが即時に変わるわけではない。生成処理を実行して表示を更新する。

リポジトリのルートから、次のコマンドで比較とメイン画面を再生成する。

```bash
python3 docs/workshops/ai-workshop/review/common-intro-20261019/workspace/build_review.py --publish
```

スライドのSVGも修正した場合は、先にdesign-svgで解決・検査し、描画して確認する。生成後のHTMLだけを直接編集しない。共通の公開先と生成処理は「はじめに」担当が更新する。担当範囲の一覧は[担当情報](tracks/intro.json)のgenerated_files_ownerにある。

## 別セッションが修正を追加するとき

1. 自分の作業コピーに、図・原稿・保存した変更前・変更理由・生成元を持つ。
2. 自分の比較の公開URLと、生成方法・編集元を[担当情報](tracks/proposal-orientation.json)へ記録する。
3. 共通のメインや閲覧画面に反映したい内容、公開URL、元ファイルと検査結果を、新しい[変更記録](changes/)へ保存する。
4. 共通の公開先を更新する担当が、生成元へ導線を追加して再生成する。表示用HTMLの一部分だけを他担当が書き換えない。

URLを担当情報へ記録するだけで、メイン画面の全リンクが自動登録される仕組みではない。現在のリンクはreview_navigation.pyが生成する。新しい公開先を追加するときは、この生成元も更新する。

比較のデータでは、beforeNotesが保存した変更前、afterNotesが変更後、reasonsが変更理由を表す。現在のreview_ui.pyは既存の比較テンプレートの構造を使うため、別形式のHTMLへそのまま適用できるとは限らない。再利用前に入力と必要な要素を確認する。

## 版と確認状態

図・原稿の新版を確認することと、動画・音声の新版を確認することを分ける。現在の「はじめに」の閲覧は修正中の7枚を使い、動画・音声は変更前と表示する。旧原稿が保存されていない資料は、原稿差分を推測で作らず、変更後のみと明示する。変更理由が未記録の枚も、その旨を示す。

画面構成への同意を、スライドや正本への適用承認として記録しない。題材と教材1〜3の方針は[共通方針](direction.md)、個々の確認待ちは担当情報が参照する進捗から読む。

## 操作と表示の確認

共通の公開先を更新した担当が、次を実行する。

```bash
node docs/workshops/ai-workshop/review/common-intro-20261019/tools/check_review_ui_20261024.cjs
node docs/workshops/ai-workshop/review/common-intro-20261019/tools/check_review_navigation_20261024.cjs
```

原稿差分の確認スクリプトは教材1の紹介がある6枚目、導線の確認スクリプトは新規追加の5枚目を対象とする。8枚全体の対応はtools/check_intro_eight_slide_layout_20261024.cjsで確認する。対象を移した場合は入力も見直す。スクリプトの通過だけで判断せず、生成した画面と図の画像を確認する。既存の確認画像と結果はworkspace/ui-review-20261024/とworkspace/navigation-review-20261024/にある。

サーバーが停止していた場合の起動方法は[共通導入のREADME](../common-intro-20261019/README.md)にある。作業用のデータを/tmpだけに保存しない。
