# AI活用ワークショップ「はじめに」の引き継ぎプロンプト

以下を、次の担当への最初の入力として使用してください。記録日は2026年10月19日です。

## あなたへの依頼

AI活用ワークショップの、教材1〜3に共通する「はじめに」の修正を引き継いでください。利用者への応答は日本語にしてください。オリエンテーションや教材1本編の修正を始めるのではなく、まず以下の最新状態と入力を読んでください。

作業内容は、経費精算システム刷新の事例を使った7枚構成への変更です。最新の図ができているのは内容1・2枚目で、内容3枚目以降は構成・原稿案だけ更新しています。次の図の制作対象は内容3枚目です。ただし、利用者が内容2枚目を確認中なので、新しい返事や修正依頼があれば先に反映してください。

## 1. 最初に読むファイル

リポジトリのルートは `/home/daidaiiro/workspace/daidaiiro` です。以下の `P` はパスの省略表記です。

```text
P=/home/daidaiiro/workspace/daidaiiro/docs/workshops/ai-workshop/review/common-intro-20261019
A=P/workspace
R=A/seven-slide-revision-20261019
V=P/viewer
```

上の表記は、この文書のパスを読むためのものです。シェルで使用するときは実際の値を設定してください。

読む順序は次のとおりです。

1. この `HANDOFF.md` を読んでください。
2. `R/progress.json` で現在地と次の枚を確認してください。
3. `R/case-setting.json` で最新の案件設定を確認してください。マージ後CIを含みます。
4. `R/slide-plan.json` と `R/slide-plan.md` で7枚の意図、図で表す関係、次への接続、原稿案を確認してください。
5. `R/narration-drafts.json` で7枚分の原稿案を確認してください。ただし1・2枚目の現在の表示は `A/slide-01.json` と `A/slide-02.json` でも照合してください。
6. `A/slide-01.source.svg` と `A/slide-02.source.svg`、解決後の同名 `.svg`、それぞれの `.json` と描画画像を確認してください。
7. `A/build_review.py` で比較ページの生成処理を確認してください。
8. リポジトリの `CLAUDE.md` と `skill-wiring.json`、今回使用するSkillの `SKILL.md` を読んでください。
9. `memory/ai-workshop-current-intro.md` を読んでください。`memory/ai-workshop.md` は古い経緯も含むため、今回の設定については新しいファイルを優先してください。

## 2. 利用者と決めた最新のストーリー

### 案件と目的

経費精算システムを刷新する架空の案件です。既存システムを知るメンバーに、新しい開発者が加わりました。開発担当者、業務担当者、プロジェクトマネージャー・プロジェクトリーダーがいて、実装・レビュー・テストでAIを活用します。

AI活用で目指すのは、必要な品質を満たし、開発完了までの時間を短くすることです。コードを作る速さやレビュー承認までの速さと、修正・確認を含む開発全体の速さを区別してください。

これは三教材全体で目指すことです。教材1だけでこの目的を解決するという説明にはしません。「業務ルールに沿って経費を精算できる」は開発するシステムの目的であり、AI活用の目的と混同しないでください。

### 今回の不具合

| 項目 | 最新の設定 |
|---|---|
| 承認の流れ | 申請 → 上長承認 → 経理承認 |
| 本来の仕様 | 上長承認後は、申請者による申請内容の直接変更を禁止 |
| 担当タスクの説明 | 申請内容の変更機能を刷新する。「承認後は変更不可」と記載 |
| 開発者の理解 | 経理承認後から変更を禁止すればよいと解釈 |
| 資料 | 仕様書・既存コード・開発手順は存在し、新しい開発者も確認 |
| 判断材料 | 承認フローと既存コードの状態判定を照合すれば、上長承認後からの制限と分かる |
| AIへの条件 | 開発者が理解した条件をAIに伝えて実装・テスト・説明を作成 |
| 単体テスト | 作成したケースは通過。経理承認後の変更不可は確認したが、上長承認後・経理承認前のケースが不足 |
| レビュー | AIと人が確認したが、元の業務ルールとの照合が不十分で、同じ取り違えを見逃した |
| 負荷 | コード・テスト・説明が増え、どの条件を何で確かめたか把握しにくく、人の確認もAIの指摘確認に偏った |
| 工程 | レビュー承認 → マージ → マージ後のCIで既存の承認フローの自動テストを実行 |
| 発覚 | 上長承認後・経理承認前の申請を変更できてしまう |
| その後 | 修正・再テスト・再レビュー。修正をマージした後のCIでも解消を確認 |

この不具合は、単体テストやレビューでも検出できるものです。結合テストでしか検出できない種類の不具合とは説明しません。AIが独力で誤解した話でも、資料がないため確認できなかった話でもありません。

実装とテストで同じ取り違えが残っても、作成したテストは通り、AIレビューでも指摘が出ない場合がある、という事例です。AIを使うと必ず品質が下がる・手戻りが増えるという一般論や、実測した効果として説明しないでください。

最後に利用者が指定したのは「マージ後のCIでいいんじゃないか」です。この指定を反映済みです。古い「レビュー承認後もマージ前」「ブランチをテスト環境に配置してからマージ」という設定には戻さないでください。

## 3. 合意した7枚構成

表紙を除く内容の枚数です。既存の表紙は今回変更していません。

| 枚 | 役割 | 現在の制作状態 |
|---|---|---|
| 1 | 案件・チーム・参照資料と、AI活用の目的を紹介 | 図・原稿を比較HTMLに反映済み |
| 2 | 条件をAIに伝え、実装・単体テスト・AIと人のレビューを進め、短時間でレビュー承認。マージ後CIへ接続 | 最新の図・原稿を反映済み。利用者の確認待ち |
| 3 | CIのテストで、上長承認後も変更できる不具合が発覚。修正と再確認が必要になる | 構成・原稿案あり。次に図を作成 |
| 4 | 「承認後」を取り違え、その理解が実装・テスト・レビューに残った過程を振り返る | 構成・原稿案あり。図は未制作 |
| 5 | 教材1：作業の目的から必要な内容を考える | 同上 |
| 6 | 教材2：条件を表す言葉の意味と範囲を明確にする | 同上 |
| 7 | 教材3：条件と手順をAIが使える仕組みにする。最後に教材1へ導入 | 同上 |

以前提案した「別の機能でも試す」「自分の担当業務で考える」という独立した8・9枚目は採用しません。「はじめに」は題材の紹介と教材1への導入です。

教材1では、例えばレビューの目的を、AIの指摘確認だけではなく、業務上必要な条件を満たすか確かめることとして考えます。目的から、業務条件・実装・テスト結果の対応が必要だと考える流れです。

教材1の終わりに「承認後は変更不可」という条件を挙げても、同じ意味で受け取るとは限らない、という問いを残します。教材2では、どの承認段階、誰の操作、どのデータ・操作範囲を指すかを確かめます。教材3では、その条件を共通の参照情報と役割別の手順へ反映し、AIの出力を人が元の資料や実行結果と照合する仕組みを扱います。

目的は大きくするほどよいとは説明しません。どの作業で再利用したいかに合わせて考えます。図の具体例を単にまとめれば、仕事の目的が出てくるという表現にも戻さないでください。

## 4. 最新の比較ページと、表示上の重要な注意

- 比較：`http://127.0.0.1:8772/design-review.html?slide=2#lead`
- 資料全体：`http://127.0.0.1:8772/`
- 過去の画像・SVG一覧：`http://127.0.0.1:8772/asset-gallery/`

比較ページの「新しい7枚の構成・原稿案を開く」には、最新の構成・原稿案を掲載しています。

ただし、比較のスライド選択肢と左右のデッキは、制作途中のためまだ10枚です。新しい図になっているのは内容1・2枚目だけです。内容3〜10枚目は従来のストーリーが残っています。構成案を更新したことを、全図に反映したこととして報告しないでください。

`?slide=2#lead` の2は内容の番号です。表紙を含む通し番号ではありません。比較用の `A/after.html#1` は内容1枚目ですが、正本の `lesson-01-lead.html#1` は表紙です。

従来の内容3枚目と新しい内容3枚目は役割が異なります。旧番号のまま一括でテキスト置換しないでください。新しい各枚を作りながら、旧図のどの部品を使うか判断してください。

## 5. リポジトリへ移行したファイルと役割

`P/workspace/` が現在の編集先です。以前の `.artifact-backup/ai-workshop-common-intro-story-20261018/` は、移行前の履歴として残しています。以後、両方を並行して編集しないでください。

| 保存先 | 内容 |
|---|---|
| `A/slide-NN.json` | その内容スライドの見出し・導入文・ナレーション |
| `A/slide-NN.source.svg` | 図の編集元。classでスタイルを指定 |
| `A/slide-NN.svg` | design-svgによる解決・検査後の図 |
| `A/theme.json` | warm-paperに対応する図の色トークン |
| `A/build_review.py` | 既存のデッキの型と図を使い、比較を生成・公開 |
| `A/baseline-after.html` | 既存デッキの型。生成処理がsectionを差し替える基準 |
| `A/baseline-design-review.html` | 比較ページの基準。`id=data` のJSONを生成処理が更新 |
| `A/after.html`・`A/before.html` | 生成した比較用デッキ |
| `A/comparison.html` | 生成した比較ページの保存用コピー |
| `A/script.md` | 現在表示する10枚の原稿。3枚目以降は旧版を含む |
| `A/log-course.json` | 枚ごとの修正記録。再生成時にbuilderが作成するため、追加記録はbuilderにも反映 |
| `A/approved-slide-NN-hashes.json` | 以前承認したファイルの変更検知。3〜9枚目などの旧版を対象とする記録 |
| `R/previous-approved-slide-01-hashes.json`・`02` | 今回の変更前の承認記録。新案の承認済みという意味ではない |
| `R/before/` | 7枚構成へ変更する直前の1枚目・builder・比較など |
| `R/before-slide-02/` | 2枚目を新構成に変える前の吹き出し図と原稿 |
| `R/before-post-merge-ci/` | マージ後CIへ変更する直前の2枚目・構成案・原稿案など |
| `R/before-approval-story/` | 承認段階の取り違えに変更する前の構成案 |
| `A/full-review/` | 旧10枚構成を通しで確認した記録。今回の7枚を確認した結果ではない |
| `A/after.png`・`dark.png`・`mobile.png` | 内容1枚目の描画 |
| `A/slide-02-after.png`・`dark.png`・`mobile.png` | 最新の内容2枚目の描画 |
| `A/slide-02-comparison-desktop.png`・`mobile.png` | 比較ページの表示確認 |
| `P/tools/` | `/tmp`から保存し、編集先を新しいworkspaceへ変更した確認用スクリプト |
| `P/viewer/` | `/tmp`から移した資料閲覧・比較ページと表示用ファイル |
| `P/reference-files.json` | 旧作業場所、題材の旧版、閲覧ページ、教材正本、画像一覧の5302件の参照一覧 |
| `P/migration.json` | 移行元・移行先・コピーしたスクリプト・公開履歴に含めない書籍関連表示の一覧 |

`A/script-slide-NN.md` と古い `review-slide-NN.md` は、今回のJSONと更新時点が違うものがあります。現在の原稿はJSONと生成した `A/script.md`、将来の7枚の案は `R/narration-drafts.json` を見てください。

`A/agreed-story.json` と `A/sequence.json` は旧10枚構成の記録です。承認後の制限自体が抜ける設定、マージ前テストなどが残っています。最新設定の根拠にしないでください。最新は `R/case-setting.json` と `R/progress.json` です。

Git管理対象から除外された `.artifact-backup/` の代わりに、通常のリポジトリ配下へ配置しています。コミットの状態は `git status --short -- docs/workshops/ai-workshop/review/common-intro-20261019 memory` で確認してください。書籍の表紙を含むHTML・過去の画面画像はローカルに残し、`P/.gitignore` で公開履歴の対象から除外しています。新たに書籍の画像や書き写しをコミットしないでください。

## 6. 生成・描画・公開手順

以下はリポジトリのルートから実行してください。

```bash
P=docs/workshops/ai-workshop/review/common-intro-20261019
A="$P/workspace"

# 図を編集した場合
.claude/skills/design-svg/bin/design-svg resolve "$A/slide-02.source.svg" --theme "$A/theme.json" --out "$A/slide-02.svg"

# 比較用デッキを生成。まだ公開用ファイルへは書かない
python3 "$A/build_review.py"

# 通常表示・ダーク設定・幅358pxで描画。最後の2が内容の番号
node "$P/tools/check_intro_story_20261018.cjs" 2

# 原稿と人が読む文書を検査
.claude/skills/doc-writing-skills/bin/doc-writing-skills check "$A/script.md" "$A/seven-slide-revision-20261019/slide-plan.md"

# PNGを実際に見てから、同じ比較ページに公開
python3 "$A/build_review.py" --publish

# サーバー起動後、比較の原稿とiframeを確認
node "$P/tools/check_intro_comparison_20261018.cjs"
```

`check_intro_comparison_20261018.cjs` は現在、内容2枚目用の確認です。後の枚に進むときは対象番号と期待する原稿を変更してください。選択肢の往復確認は限定的であり、全10枚の整合や全リンクを検証するものではありません。

`full_intro_review_20261018.cjs` は全10枚と正本オリエンテーションを画像化する旧確認用です。途中の7枚案全体が完成したと判断するためには使わないでください。

確認用スクリプトの現在の依存先です。

```text
Node.js
/home/daidaiiro/.npm/_npx/705bc6b22212b352/node_modules/playwright
/home/daidaiiro/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome
Python 3 と markdown モジュール
```

ブラウザーの起動やローカルポートへの接続には、環境によってsandbox外での実行が必要です。許可済みのコマンドか確認し、必要ならツールの実行許可を使用してください。保存先が変わったため、以前の `/tmp` のコマンドに対する許可と、新しいコマンドに対する許可は別の場合があります。

### ローカルサーバー

現在はリポジトリ内の `P/viewer/` を公開するサーバーへ切り替えています。停止していた場合は、同じポートを他のプロセスが使っていないことを確認してから起動してください。

```bash
python3 -m http.server 8772 --bind 127.0.0.1 --directory /home/daidaiiro/workspace/daidaiiro/docs/workshops/ai-workshop/review/common-intro-20261019/viewer
```

サーバープロセスは次のセッションで生きているとは限りません。再開時にURLを確認してください。生成・公開処理、編集元、確認用スクリプトは `/tmp` を必要としません。引き継いだPython・JavaScriptのスクリプトには絶対パスの実行環境依存があるので、別マシンでは上記の依存先を変更してください。

### builderの変更時の注意

現在のbuilderは、10個のsection、1から始まる配列位置、既存の比較用JSONに依存します。7枚の図ができる前に、残り3枚を削除して配列長だけ変更すると、比較の原稿と表示が対応しなくなるおそれがあります。全7枚ができた時点で、section・LABELS・比較の枚数・beforeNotes・afterNotes・notes・理由の番号を一緒に確認してください。

承認記録がある枚を修正すると、ハッシュ検査が失敗します。利用者が今回の変更を承認したことを記録し、変更前のファイルとハッシュをその改訂のbeforeフォルダへ保存してから、新案の確認待ちとして扱ってください。単にハッシュを上書きし、新案を承認済みと扱わないでください。

比較の変更前は、直前に確認していた版を固定してください。今の2枚目の変更前は `R/before-post-merge-ci/` です。1枚目は `R/before/`、旧3・8・9・10枚目は `A/full-review/before/` などをbuilderが参照しています。古いbeforeフォルダを上書きしないでください。

## 7. デザイン・原稿と確認の範囲

利用者は、既存の人物、重ねた成果物、画面、状態、縦方向の具体と抽象、境界のある図の表現を重視しています。内容の変更を理由に、一律の文字カードへ置き換えないでください。1枚ずつ、見出し・導入文・図・原稿を一緒に確認します。

図ではclassを使い、既存のwarm-paperの色を読みます。

```text
.claude/skills/slide-deck/SKILL.md
.claude/skills/slide-deck/references/document.json の review・render-check
.claude/skills/slide-deck/references/themes/warm-paper.css
.claude/skills/design-svg/SKILL.md
.claude/skills/design-svg/references/theme.json
.claude/skills/doc-writing-skills/SKILL.md
.doc-writing/retired-words.json
```

色の新しい直書きはしません。強調する枠に濃い塗りを追加しないでください。現在の `theme.json` では、強調枠もpaper色にしてあります。矢印の接続先に境界があり、線や矢じりが枠・文字と接触していないか、描画画像で確認してください。成果物の量を重なりで表現する場合、同じ大きさの図形を少しずつ移動します。

ナレーションは聞き手が一度で理解できる敬体にします。図の左・中央・右と原稿の案内を対応させ、前の結びが次の始まりへつながるようにしてください。略語の発音は、PMはプロジェクトマネージャー、PLはプロジェクトリーダー、PRはプルリクエスト、Agentsはエージェント、Skillsはスキルです。音声制作時には「成果物」「一人」などの読みも再確認してください。

今回実施済みなのは、1・2枚目のSVG検査、原稿・構成案の文書検査、通常表示・ダーク設定・スマートフォン幅での描画と目視、比較の表示確認です。移行後の比較ページで、マージ後CIへつながる最新原稿も確認しました。

最新の7枚構成に対するLuna受講者レビュー、音声聴取、動画確認はまだ行っていません。以前のLunaレビューを今回の新設定への評価として扱わないでください。利用者から再度レビュー依頼があれば、まず画像と原稿がそろった対象を明確にして実施してください。

## 8. 正本・旧版・関連資料の参照先

| 参照先 | 用途と状態 |
|---|---|
| `docs/workshops/ai-workshop/README.md` | ワークショップ全体の説明 |
| `docs/workshops/ai-workshop/proposal/` | 企画書。今回の変更対象ではない |
| `docs/workshops/ai-workshop/material-1/src/lesson-00-start.json` | 現行オリエンテーションの原稿。今回の変更対象ではない |
| `docs/workshops/ai-workshop/material-1/out/slides/lesson-00-start.html` | 現行オリエンテーション。表紙含む14枚 |
| `docs/workshops/ai-workshop/material-1/src/lesson-01-lead.json` | 正本のはじめに。今回の新案と同一とは限らない |
| `docs/workshops/ai-workshop/material-1/src/ledes.json` | 正本の導入文。今回未変更 |
| `docs/workshops/ai-workshop/material-1/src/figures/` | 正本の図。今回未変更 |
| `docs/workshops/ai-workshop/material-1/out/` | 既存のPDF・スライド・動画など。今回の新案の動画ではない |
| `.artifact-backup/ai-workshop-common-intro-20261017/` | 既存の図解、旧題材、過去レビュー、正式反映用の旧作業コピー |
| 同上の `formal-work/` と `formal-source-files.json` | 以前の正式反映案。今回の最新7枚案ではない |
| `.artifact-backup/ai-workshop-common-intro-story-20261018/` | 今回移行する前の作業場所。以後は履歴として参照 |
| `.artifact-backup/ai-workshop-asset-gallery-20261017/index.html` と `manifest.json` | 過去の画像・SVG・session由来の資料を探す索引 |
| `.artifact-backup/ai-workshop-video-feedback-20261011/` | 古い動画レビュー・Lunaレビュー・修正版の履歴 |
| `/home/daidaiiro/workspace/ai-workshop-wip-2026-10-10/README.md` | 最初の教材1作業コピーと比較の説明。最新編集先ではない |
| `.acdr/0196-三教材共通の導入と用語説明/` | 以前の正式反映に関する記録。今回の新案の承認記録として扱わない |
| `/home/daidaiiro/workspace/spec-correspondence/00-background`・`07-appendix` | 利用者の研究資料。今回の導入には研究説明を追加しない方針 |

QA・開発工程について以前確認した参照先です。新しい主張を追加する場合は原文を改めて確認してください。

- [Microsoft：CI/CDの構成例](https://learn.microsoft.com/en-us/azure/devops/pipelines/architectures/devops-pipelines-baseline-architecture?view=azure-devops)
- [GitLab：マージ結果のテスト](https://docs.gitlab.com/ci/pipelines/merged_results_pipelines/)
- [Google：コードレビューの確認観点](https://google.github.io/eng-practices/review/reviewer/looking-for.html)
- [Google：変更を小さくする理由](https://google.github.io/eng-practices/review/developer/small-cls.html)

オリエンテーションを修正し直したり、研究紹介を導入へ戻したりする依頼ではありません。

## 9. 正本への反映と制作の境界

今回の1・2枚目と7枚の構成案は、レビュー用です。正本の図・原稿、音声・動画は今回変更していません。新案を既に正式反映・動画化したものとして案内しないでください。

利用者は内容を確認しながら進めたい意向です。全編の確認後、正本へ入れる段階では、リポジトリのACDR手順で差分と理由を提示します。

```text
.claude/skills/acdr/SKILL.md
.claude/skills/acdr/bin/acdr new|validate|render
```

正本へ入れる前に承認を得てください。承認後は、正本のbuild_lessons.py・check_figures.py・render_previews.pyなど、その時点の制作手順で生成・検査します。対象パスを明示してstageし、無関係の作業を含めないでください。現在のレビュー入力を正本の全編へ一括適用する段階ではありません。AWSログインやプロファイルは今回確認していないため、音声制作の際に過去のログイン状態を前提にしないでください。

## 10. 再開時に実施すること

1. 最新の利用者の返事と、この引き継ぎの作業対象が一致しているか確認します。
2. `R/case-setting.json` と現在の2枚目を読み、レビュー承認後にマージし、CIで不具合が発覚する前提を確認します。
3. 比較ページを開きます。停止していればリポジトリ内のviewerを公開します。
4. 2枚目への追加修正がなければ、新しい内容3枚目を作ります。上長承認と経理承認の状態、制限すべき範囲、実際に制限した範囲が図で分かるようにしてください。
5. 既存の図から使える部品を選び、画像・全文原稿・前後の接続を一緒に確認します。
6. 検査し、実際の描画画像を見て、同じ比較HTMLへ変更前後を反映します。
7. どの枚の図と原稿を変更したか、未変更の範囲はどこかを報告し、利用者の確認後に次の枚へ進みます。

この引き継ぎを読んだだけで、全7枚の図を一括制作したり、音声・動画を生成したりしないでください。
