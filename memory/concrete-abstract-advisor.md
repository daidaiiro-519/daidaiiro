---
name: concrete-abstract-advisor
description: 「具体と抽象」の本を原典にした新しい advisor を作る作業の現在地（2026-10-05 に引き継いだ）
metadata:
  type: project
---

別のセッションが原典の準備まで進めた（控えは `.artifact-backup/advisor-20261004/`：preparation.md ・ manifest.json ・ source.pdf。PDF 262頁、文字層なし、奇数頁と偶数頁の順が入れ替わる見開きがある。書誌は manifest.json に記録：細谷 功『システム開発と「具体と抽象」』技術評論社、ISBN 9784297157906）。頁の画像は scan/p-NNN.jpg（262枚、git の外）。

利用者の決め（2026-10-05）：範囲は原典の全章をそのまま網羅する。本質は「目的のレベルを正しく見極める」ことで、その表れが具体と抽象に出ている（Skill の名前と説明はこれに合わせる。名前は meta-thinking-advisor（書名の「思考のメタ化」から、2026-10-05 決定）。雛形を置き、archive/ に book.pdf ・ scan/ ・ manifest ・ sources を写した（git の外）。頁の索引の指示は scratchpad/INDEX-MT.md（usecase ・ ddd の直しが終わってから起こす。同時に多く起こすと利用の上限に当たる）。

2026-10-06：判断基準65件 ・ 図29件で完成し、ACDR 0127 を承認 ・ コミットした。原典は利用者が全262頁を撮り直した（archive/retake/）。残り：配線表に、この advisor を呼ぶ行を足すかを決める。関連：[[advisor-coverage-gaps]]
