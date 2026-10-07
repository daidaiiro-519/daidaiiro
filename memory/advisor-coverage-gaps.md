---
name: advisor-coverage-gaps
description: advisor の判断基準を、原典の全頁から作り直す作業の現在地（usecase ・ ddd ・ meta-thinking ・ qa ・ platform は完了、ux が残り）
metadata:
  type: project
---

2026-10-06：usecase ・ ddd ・ meta-thinking の3つは完了し、ACDR 0115 ・ 0119 ・ 0127 を承認 ・ コミットした（0118 は 0119 で差し替え済み）。作り方は手順の正本（skills-creator の document.json の段1 ・ 段2、「網羅」「詳しさ」の行）と、索引のスキーマ（references/types/advisor/index.schema.json）にある：頁の画像 → 学習ノートに中身の全部 → 索引 → ノートと同じ詳しさの判断基準 → 図は宣言 JSON から SVG（図の要素は more_figures で関係する図を並べて持てる）。確かめ方は、ノートの抜き取り（30頁を別の担当が画像から読み直す）と、ノートの全行を判断基準に対応づける割り当て。

2026-10-06：qa-advisor も完了（ACDR 0131 承認 ・ コミット 4afcedfd、未 push）。Web の原典（xunitpatterns）は、取得した HTML を chrome で PDF に印刷して頁の画像にした。

2026-10-06：platform-advisor も完了（ACDR 0134 承認）。WAF の柱の件6つとフレームワークの件を足して64件、図38枚。meta-thinking-advisor を配線表に2行足した（ACDR 0135）。

既知の欠陥（未着手）：ゲート1が advisor の判断基準の頁（view の HTML）で廃語を検出しなかった（platform で「要る」約300か所を見逃した）。

2026-10-07（現在地）：ux の作り直しの前に、advisor 全体の回答の形をボード .brainstorming-board/advisor-answer/（Artifact 9ek8LMJvRsQs1PHk9jWVWB、4回目）で決めている。論点2・3決着。論点1は4回目の答え（共通にするのは Orchestrator が読む欄＝結論・根拠・ai_cautions と、頁の構造と表示の部品だけ。完成イメージの中身は advisor ごとに SKILL.md）で、承認＝UI の合意、まだ承認されていない。論点4（雛形への入れ方の5段）は待ち。
- ACDR 0150（全 advisor 共通の samples と共通の描画）は利用者が「前の方が百倍よかった」と差し戻し、採らずに作業ツリーを戻した（控えは消えるので無い）。教訓は brainstorming-board / Skill 側に置く
- UI の見本：ux 設計相談 5upZ4JUY45p9XauQU2HuKS ・ ddd 判断相談 EMW8bu9L88cXeYRDa7g1EM ・ 6つの advisor の完成イメージ37種類 EWrHCJe3RLBy3Hg3QtDcKH（利用者「いいね。これ図がいいね」）。ソースはボードの ui-samples/ に写した（gallery は assemble.py で組む）
- ux-advisor-purpose のボード（9回目）：論点1〜4決着。論点2の「共通の欄 samples」という言い方は UI の合意のあとで直す
- design-svg の作り直しは 2026-10-08 に完了（ボード design-svg-rework 論点1〜5 決着、ACDR 0157〜0160 承認・コミット）。作成者が class で SVG を書き、design-svg が解決と検査をする。テンプレート17種類は design-svg の references/exemplars。残り：8つの Skill の refs.rs の写し直しの ACDR（準備中）、ブレストボード側の figures の検査の削除はその作り直しの担当へ申し送り
- 次：advisor-answer の論点1（UI の合意）の承認から再開する

次（段1〜5のあと）：ux-advisor を同じやり方で作り直す（原典が書籍のスキャンでないもの（Web の原文など）は、頁の画像の代わりに取得した原文で同じ段を踏む）。push はまだしていない（利用者に聞く）。

作業に使った指示書と出力は、このセッションの scratchpad にあった（NOTEPASS ・ CRITPASS ・ LINEMAP ・ SPOT2 ・ FIGS ・ MOREFIG）。セッションが変わると消える。

関連：[[concrete-abstract-advisor]]
