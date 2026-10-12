---
name: brainstorming-board-migration
description: 新しい形（基盤 schema-driven の上）へ Skill を移す作業の現在地と、残りの順番
metadata:
  type: project
---

## 現在地（2026-10-12）

新しい形で完成したのは schema-driven（ACDR 0165 ・ 0166）と skills-creator（ACDR 0167）の2つ。0167 まで承認 ・ 適用 ・ コミット済み。push はしていない。0162（acdr の新しい記録の形）は proposed のまま止めてある。

いまはボード `.brainstorming-board/brainstorming-board-migration/`（Artifact https://claude.ai/artifact/PiuRFBbVVjvX8Uinis2kV9）の5回目。

| 論点 | 状態 |
|---|---|
| Q1 判断の目的 ・ Q2 UI と UX ・ Q3 デザインシステム | 決着 |
| Q4 固有のツール | 承認待ち（コミット 81e8bf72） |
| Q5 既にあるボード（33枚を名前の $schema と changes へ書き換えるか） | Q4 のあと |

- Q3 の決着：ボード ・ ACDR ・ アドバイザーの回答の頁 ・ 基盤の文書を、基盤のカタログの1つのデザインシステムにする（主役の色は青緑 #0d5c55、内側の面は #edf2ef）
- Q4 の答え：固有に残すのは advance（回を進め、論点ごとの changes を書く）・ inspect（本文を読む3つの判定）・ serve（直接呼ぶ。宣言しない）の3つ。inspect のうち3つはボードのスキーマ（maxContains ・ if/then）へ移す。init ・ migrate ・ render ・ 部品 board ・ transcribe ・ check-copy は消す
- 試作とアドバイザーの回答は、そのセッションの作業用の領域（proto/ ・ q4b-*.json）にだけある

## 次にすること（順番）

1. Q4 の承認 → Q5 → ボードを決着させる
2. schema-driven の ACDR：カタログに部品を4つ足し、9つ変え、トークンを入れる。check が maxContains の超過を「contains のスキーマに合う要素が無い」と返す誤りも直す
3. advance ・ inspect ・ serve を `src/brainstorming-board/` に Rust で作り、ブレストボードを移す ACDR を出して適用する。移したら `src/skills-creator/tests/baseline-v3/brainstorming-board.json` を消す
4. acdr を新しい形で作り直す（0162 もここで扱う）
5. agents creator（始める前に「Agent」が何を指すかを利用者に聞く）
6. 残りの Skill を移す。アドバイザーは最後に Agent にする
7. 配布物（tools.json の OS ごとの SHA-256 ・ accept と dist ・ release.yml ・ インストールの台本。作業ツリーに未コミットの変更あり）と、スキーマの無い JSON の洗い出し

## 移すときに使うもの

- 移す試験：`cd src/skills-creator && cargo test`。版3 の Skill を移したら、その Skill の baseline を消す（消し忘れると試験が落ちる）
- 古い skills-creator は `src/skills-creator-v3/` に凍結してある
