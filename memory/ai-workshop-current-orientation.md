---
name: ai-workshop-current-orientation
description: オリエンテーション見直し。1〜3枚目を修正。2・3枚目の構図を再設計し、図と原稿が利用者確認待ち。
metadata:
  node_type: memory
  type: project
---

# オリエンテーションの現在地

基準の作業記録：ec12a63c（main）。最新の反映状況は担当情報と末尾の記録を参照する。

利用者が完成した共通の「はじめに」と同じ進め方で見直すことを依頼。企画書は次の工程。共通方針revisionは2026-10-25-13。

作業先はdocs/workshops/ai-workshop/review/proposal-orientation-20261024/orientation-20261025/。現行の全14枚を新たに描画し、Lunaと助言担当が確認。story-plan.mdと.jsonの構成案を利用者へ提示した。現在は1〜3枚目の図・原稿を作業コピーで修正。正本と音声動画は変更していない。

比較は <http://127.0.0.1:8772/design-review.html?deck=orientation&slide=1#orientation-plan>。比較入力comparison.json、構成案story-plan.mdを担当情報から共通生成元へ読み込ませる。HTMLを直接編集しない。生成はbuild_orientation_review.py、その後common-intro-20261019/workspace/build_review.py --publish。

次は2・3枚目の再修正の利用者確認を受けてから、4枚目へ進む。現行表紙の専用原稿は未記録なので、原稿差分を推測しない。

到達像は仕事全体の生産性と品質の改善。6枚目の問い、7枚目のAIの使い方、8枚目の三教材、9枚目の応用、10枚目の書籍の理由を区別する。一般AI初心者向けへ広げず、PM・PLへ不自然な実装作業を割り当てない。Lunaの感想の採否はstory-plan.mdにある。

書籍表紙を含む元HTMLと画像は.artifact-backup/ai-workshop-orientation-review-20261025/に保存し公開コミット対象外。比較用HTMLは既存画像への参照とし、source-manifest.jsonに原本ハッシュを記録した。

2026-10-25：構成案が読みにくいとの指摘を受け、比較ページ冒頭へ移動した。横長の表をやめ、三つの流れと各枚の要点を縦に表示する。図の方針・修正点は折り畳む。PCとスマートフォン幅で描画を確認し、横にはみ出さないことを確認した。構成案は確認待ち。スライド・原稿は未修正。

2026-10-25：利用者の指示で表紙を修正。三教材の図とアイコンを維持し、教材名と説明を最新の「はじめに」に合わせた。見出しは「AIで、仕事全体の生産性と品質を高める」。専用ナレーションを追加した。変更前の専用原稿は未記録と明示。本文13枚のHTMLが変更前と一致することを検査した。比較URLは担当情報を参照。表紙の確認後に2枚目へ進む。

2026-10-25：利用者の指示で2枚目の背景・位置づけを修正。既存のアイコンと図の関係を維持し、到達像を必要な品質と確認・修正を含む仕事全体の改善に変更。表紙と3〜14枚目のHTMLに変更がないことを照合。図の検査は変更前0件・変更後0件。描画画像と別担当による確認を実施した。1枚目は次の枚への指示を受けて確認済み、2枚目は確認待ち。

2026-10-26：利用者の指示で3枚目を修正。既存の三つのカードとアイコンを活かし、作りやすさと速さの効果から、確認・修正を含む工数・期間・品質へ視野を広げる。効果が出ていないとは決めつけず、受講者の仕事の変化を問う。図の検査は変更前0件・変更後0件。他の13枚のHTMLに変更はない。2枚目は次の枚への指示を受けて確認済み、3枚目は確認待ち。

2026-10-26：利用者の指摘を受け、2・3枚目は構図も再修正。2枚目は背景→ワークショップの学習と実践→目指す改善を接続。3枚目は仕事全体の外枠の中に作成・確認修正・完了を置き、部分と全体を区別する。直前版を作業先のiterations/20261026-before-redesign/へ保存。両方の原稿を図に合わせた。図の検査は各0件。描画画像と別担当の確認を実施。3枚目の下部余白も調整した。2・3枚目とも確認待ちに戻し、4枚目へは未着手。

2026-10-26：2枚目右端のチェックとカレンダーが説明文に対して左へ偏っていたため、双方を説明文の中央に配置した。図の検査0件、描画画像で確認。他13枚と原稿は変更なし。

2026-10-26：利用者が2枚目の見出し案を了承。「AIを仕事の改善につなげる考え方を学び、実践するワークショップです」を反映し、描画と文章の検査を実施。図と原稿の本文は変更していない。

2026-10-26：利用者の指示で、全枚上部の資料名を旧題から表紙の見出し「AIで、仕事全体の生産性と品質を高める」へ、2枚目の導入文を受講後にできること（学んだ考え方をSkillsにまとめAgentsで仕事を進める）へ修正。4〜14枚目は資料名以外変更なし。正本narration.full.jsonの旧題読み上げは音声の作り直しで合わせる。[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-04-orientation-deck-title-and-lede.md)。2・3枚目は確認待ち。

2026-10-26：利用者が2・3枚目の内容の重なりを指摘し、原稿も合わせて直すよう指示。2枚目を「このワークショップで行うこと」（学ぶ→Skillsにまとめる→自分の業務で試す、人が確かめて改善）の図と原稿へ作り直し、背景と確認・修正は3枚目に任せた。直前版はiterations/20261026-before-role-split/。[変更記録05](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-05-orientation-slide-02-03-roles.md)。2・3枚目は確認待ち。

2026-10-26：利用者が見直し開始時の2枚目の構成（開催の背景→位置づけ→目指す姿の帯）の方が背景も分かってよいと指摘。3つの流れの図をやめて元の構成へ戻し、文言を更新。目指す姿は「自分の仕事に合ったSkillsを作り、Agentsで仕事を進められる」。原稿は伝わりにくいとの指摘で1文1内容に分けた。[変更記録06](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-06-orientation-slide-02-original-layout.md)。2・3枚目は確認待ち。

2026-10-26：利用者が2枚目を了承（「いいですね」）し、3枚目も同じ観点で見直すよう指示。図の構成は残し、2・5枚目と重なる原稿と、導入文と帯の重なりを外した。原稿は1文1内容。[変更記録07](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-07-orientation-slide-03-overlap.md)。3枚目確認待ち。

2026-10-26：利用者が見直しの観点を確認し、共通導入との整合、図の見せ方での負担軽減、論・日本語・言い換え・Slopの確認を追加（担当情報のreview_criteria）。3枚目は確認済み。4枚目を修正し確認待ち。[変更記録08](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-08-orientation-slide-04.md)。

2026-10-26：利用者の指示で2・3枚目を追加の観点で再確認。2枚目は教材で学ぶに三教材を含め、スキルの説明を共通導入と同じ「何を参照し、どう判断し、何を確かめるか」に。3枚目は「仕事全体」の3回の繰り返しと和語・「ようやく」を修正。[変更記録09](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-09-orientation-slides-02-03-recheck.md)。2〜4枚目確認待ち。

2026-10-26：2〜4枚目は次の枚への指示で確認済み。5枚目を修正（見出しを4枚目の問いを受ける形、左側の職種の発言を成果物が増える箱へ、帯を結論へ）。[変更記録10](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-10-orientation-slide-05.md)。5枚目確認待ち。

2026-10-26：5枚目は次の枚への指示で確認済み。6枚目を修正（導入文・右の見出し・帯の3重の重なりを分け、帯で7枚目のAIへの説明を予告）。[変更記録11](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-11-orientation-slide-06.md)。6枚目確認待ち。

2026-10-26：6枚目は次の枚への指示で確認済み。7枚目を修正（Skillを「記録」から判断の観点・確認と照合の手順へ、Agentの照合と人の根拠確認を追加、表記を「はじめに」のAgent・Skillsへ）。[変更記録12](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-12-orientation-slide-07.md)。7枚目確認待ち。

2026-10-26：7枚目は次の枚への指示で確認済み。8枚目を修正（旧教材名を共通導入の名称へ、教材2をモデル化と設計へ、Skillsを判断の観点と確認の手順へ、共通の題材で学ぶ）。[変更記録13](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-13-orientation-slide-08.md)。8枚目確認待ち。 囲み枠（塗りなし）は利用者の指摘でやめ、カードの列にそろえたSkills・Agentへ組み直した。

2026-10-26：8枚目は次の枚への指示で確認済み。9枚目を修正（左を進捗の整理の例、右に業務のルールとの照合と設計の見直し、帯を仕事に合わせて考える方法へ）。[変更記録14](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-14-orientation-slide-09.md)。9枚目確認待ち。

2026-10-26：9枚目は次の枚への指示で確認済み。10枚目を修正（教材名、DDDをモデルと設計へ、ユースケースを目的レベルと範囲へ、読了不要を導入文に）。[変更記録15](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-15-orientation-slide-10.md)。10枚目確認待ち。

2026-10-26：10枚目は次の枚への指示で確認済み。11枚目を修正（教材名、仕組み→Skills、載せる項目、運営条件は維持）。企画書のportal図に旧名が残る点を担当情報に記録。[変更記録16](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-16-orientation-slide-11.md)。11枚目確認待ち。

2026-10-26：11枚目は次の枚への指示で確認済み。12枚目を修正（目的の達成→何が改善したか、確認・修正の振り返り、Skills・Agent、Kiroは13枚目へ）。[変更記録17](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-17-orientation-slide-12.md)。12枚目確認待ち。

2026-10-26：12枚目は次の枚への指示で確認済み。13枚目を修正（下段のフッター接触を1行の帯で解消、導入文を資料なしで相談できることへ）。[変更記録18](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-18-orientation-slide-13.md)。13枚目確認待ち。 利用者の再確認で、見出しを図の順の三つに、Teamsを双方向に、帯の「の」の連続と原稿の飛んだ一文を修正。

2026-10-26：13枚目は次の枚への指示で確認済み。14枚目を修正（教材名、円を表紙と同じ到達点、導入文を改善したいことを考える呼びかけ、下段の高さ）。[変更記録19](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-19-orientation-slide-14.md)。14枚目確認待ち。次は全14枚の通し確認とLunaレビュー。

2026-10-26：利用者が参加者の定員を15名から20名に変更。オリエンテーション12・13枚目の図と原稿に反映。企画書（Kiroの費用 月$300→$400など）は未反映で、企画書の見直しで扱う。[変更記録20](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-20-workshop-capacity-20.md)。

2026-10-26：定員を先着15〜20名に変更（12・13枚目の図と原稿）。13枚目の参加者と運営を枠で囲んだ。企画書の費用は月$300〜$400で未反映。

2026-10-26：14枚目と12・13枚目を利用者が確認。全14枚が確認済み。一つのエージェントはAgent、組み合わせはAgents・Skillsと「はじめに」に合わせた（1・2枚目）。[変更記録21](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-21-orientation-agent-notation.md)。次は全14枚の通し確認。

2026-10-26：全14枚の通し確認。枚の境目の重なり5か所（2枚目の三教材の重複説明、7→8、9→10、10の「まず」、12→13）を原稿で修正、図は不変。[変更記録22](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-22-orientation-through-check.md)。確認後にLunaレビュー。

2026-10-26：受講者Lunaの代わりにHaiku 5.5を1枚1体（新しい文脈）で14体レビュー。採否はreview/haiku-review-20261026/README.md。採った修正を反映（図は1・2・4・7・11枚目、他は原稿）。[変更記録23](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-23-orientation-learner-review.md)。確認待ち。

2026-10-26：受講者レビュー2回目（同条件、ワークフローで14体）。指摘は0件にならず。1回目の直し漏れ（9枚目の図の二重否定）と、2回続いた指摘（8枚目の教材3の矢印、5枚目の文、進め方、12枚目の公開時期）などを反映。採否はreview/haiku-review-20261026-round2/README.md。[変更記録24](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-24-orientation-learner-review-round2.md)。

2026-10-26：全14枚の通し読みをHaiku 5.5の1体で確認。目的・学び方・目標・次にすることは正しく読み取れた。教材カード見出しの不統一、6・9枚目の欄名、11枚目の「成果物」、7枚目の（例）などを修正。採否はreview/haiku-review-20261026-whole/README.md。[変更記録25](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-25-orientation-whole-read.md)。機密情報の扱いは利用者が「今回のテーマではない」と判断し、扱わない。

2026-10-26：作業記録をfa721f56でコミット。ACDR 0207（オリエンテーションの正式版改訂）を提案中。正本material-1/srcの15ファイルと出力2ファイルへ作業ツリー上で反映済み・未コミット（承認後にコミット、差し戻しならgit checkoutで戻す）。公開 http://127.0.0.1:8772/decisions/0207.html 。acdrのiframeが画面高さ基準のHTMLで伸び続ける不具合あり（0205も同じ）、別件。

2026-10-26：利用者がACDR 0207を承認。オリエンテーション全14枚のスライドと原稿を正本へ反映しコミット。音声・動画は未作成。次は音声・動画、企画書の見直し。[変更記録26](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-26-orientation-canonical-release.md)。

2026-10-26：オリエンテーションの音声・字幕付き動画を生成（Takumi neural、AWS_PROFILE=dev、全14枚、986.344秒、字幕187件、検証通過）。共通の閲覧ページの生成元review_navigation.pyを、オリエンテーションも正本の出力を指すよう修正。ACDR 0208を提案中、出力は未コミット。

2026-10-26：利用者の「コミットしてプッシュ」でACDR 0208を承認として反映。オリエンテーションのスライド・原稿・音声・動画がすべて正式版。次は企画書の見直し。[変更記録27](../docs/workshops/ai-workshop/review/shared/changes/2026-10-26-27-orientation-video-release.md)。

2026-10-28：利用者がACDR 0209を承認。企画書（詳細版38枚・要点版）とアンケートの設計書を正本へ反映しコミット（1a72433e、未プッシュ）。作業コピーは review/proposal-orientation-20261024/proposal-20261026、レビュー用アーティファクト https://claude.ai/artifact/9m4N9T3BrjKed4Gddm2x3W 。残り：運営設計案 workshop-plan.md の見直し。[変更記録](../docs/workshops/ai-workshop/review/shared/changes/2026-10-28-01-proposal-canonical-release.md)。
