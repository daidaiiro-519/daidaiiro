# オリエンテーションの正式反映

利用者が2026年10月26日、ACDR 0207（オリエンテーションの正式版改訂）を「承認します、正本へ反映してください」と承認した。全14枚のスライドと原稿を正本へ反映した。

## 反映した対象

正本docs/workshops/ai-workshop/material-1/src/の15ファイル（build_lessons.py、derived_visuals.py、ledes.json、lesson-00-start.json、orientation_boundary_visuals.py、orientation_support_visuals.py、reference_slide_02.py〜reference_slide_10.py）と、そこから生成したout/slides/lesson-00-start.htmlとout/scripts/lesson-00-start-script.md。生成したスライドと原稿は、利用者が比較ページで確認した版と一致する（書籍の表紙の参照方法を除く）。

共用の生成元から共通の「はじめに」（lesson-01-lead）を生成し直しても、出力は変わらないことを確認した。企画書の入力、教材本編、音声・動画は変更していない。

## 記録

ACDR 0207は.acdr/0207-オリエンテーションの正式版改訂/にあり、状態はaccepted。対象の文書のsha256を封印した。閲覧用の複製は http://127.0.0.1:8772/decisions/0207.html 。反映する差分の一覧はproposal-orientation-20261024/orientation-20261025/promotion-plan.md。

## 別の資料への影響と次の作業

音声と字幕付き動画は、正本化した版から作る。企画書には旧教材名、ポータルの図の「言葉の定義」、定員15名とKiroの費用が残るため、企画書の見直しで合わせる。

ACDRの表示で、画面の高さを基準にするスライドのHTMLを面に置くと、面の高さが伸び続ける不具合がある（ACDR 0205でも同じ）。acdrスキルの修正は別の決定として扱う。
