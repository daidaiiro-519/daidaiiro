---
name: advisor-coverage-gaps
description: advisor の判断基準を、原典の全頁から作り直す作業の現在地（usecase ・ ddd ・ meta-thinking ・ qa は完了、platform ・ ux が残り）
metadata:
  type: project
---

2026-10-06：usecase ・ ddd ・ meta-thinking の3つは完了し、ACDR 0115 ・ 0119 ・ 0127 を承認 ・ コミットした（0118 は 0119 で差し替え済み）。作り方は手順の正本（skills-creator の document.json の段1 ・ 段2、「網羅」「詳しさ」の行）と、索引のスキーマ（references/types/advisor/index.schema.json）にある：頁の画像 → 学習ノートに中身の全部 → 索引 → ノートと同じ詳しさの判断基準 → 図は宣言 JSON から SVG（図の要素は more_figures で関係する図を並べて持てる）。確かめ方は、ノートの抜き取り（30頁を別の担当が画像から読み直す）と、ノートの全行を判断基準に対応づける割り当て。

2026-10-06：qa-advisor も完了（ACDR 0131 承認 ・ コミット 4afcedfd、未 push）。Web の原典（xunitpatterns）は、取得した HTML を chrome で PDF に印刷して頁の画像にした。

次：platform-advisor ・ ux-advisor を同じやり方で作り直す（原典が書籍のスキャンでないもの（Web の原文など）は、頁の画像の代わりに取得した原文で同じ段を踏む）。push はまだしていない（利用者に聞く）。

作業に使った指示書と出力は、このセッションの scratchpad にあった（NOTEPASS ・ CRITPASS ・ LINEMAP ・ SPOT2 ・ FIGS ・ MOREFIG）。セッションが変わると消える。

関連：[[concrete-abstract-advisor]]
