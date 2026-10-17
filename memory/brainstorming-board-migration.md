---
name: brainstorming-board-migration
description: 新しい形（基盤 schema-driven の上）へ Skill を移す作業の現在地と、残りの順番
metadata:
  type: project
---

## 現在地（2026-10-12）

利用者の目標：schema-driven ・ brainstorming-board ・ skills-creator ・ agents creator ・ acdr の5つを、まず仕上げる。

ACDR 0169（2026-10-12、1712d68c）で schema-driven を新しい契約だけの基盤にした。正本は src/schema-driven/（schemas/ ・ catalog/ ・ コード ・ cli と mcp ・ 試験）、Skill は SKILL.md ・ LICENSE と、配布物の段で入る assets/ ・ scripts/（git で追跡しない。いまは手で写している）。schema-driven は skills-creator が作る Skill ではないので tool.json を持たない。skills-creator は構成は新しいが、tools.json と配る手段が無い。ほかの15の Skill は版3。

いまはボード `.brainstorming-board/brainstorming-board-migration/`（Artifact https://claude.ai/artifact/PiuRFBbVVjvX8Uinis2kV9）の5回目。

| 論点 | 状態 |
|---|---|
| Q1 判断の目的 ・ Q2 UI と UX ・ Q3 デザインシステム | 決着 |
| Q4 固有のツール | 決着（2026-10-12） |
| Q5 既にあるボード | 承認待ち（6回目）。答え：既にあるボードは写さず、古いスキーマも残す。新しい形は agents creator のボードを新しく1枚作って確かめる |

- Q3 の決着：ボード ・ ACDR ・ アドバイザーの回答の頁 ・ 基盤の文書を、基盤のカタログの1つのデザインシステムにする（主役の色は青緑 #0d5c55、内側の面は #edf2ef）
- Q4 の決着：固有に残すのは advance（回を進め、論点ごとの changes を書く）・ inspect（本文を読む3つの判定）・ serve（直接呼ぶ。宣言しない）の3つ。inspect のうち3つはボードのスキーマ（maxContains ・ if/then）へ移す。init ・ migrate ・ render ・ 部品 board ・ transcribe ・ check-copy は消す
- 試作とアドバイザーの回答は、そのセッションの作業用の領域（proto/ ・ q4b-*.json）にだけある

## いまの作業（2026-10-13）

- ACDR 0174（CODING.md）・ 0172（検査の文と案内）・ 0175（CODING.md に従わせる。crates/core と crates/adapters、宣言したツールの仕組みを基盤から消した）は承認 ・ コミット済み。0173 は取り下げて消した。
- 決めたこと：Skill の能力は Skill が自分の実行ファイルとして持ち、SKILL.md の手順から直接呼ぶ。JSON ・ スキーマの操作は scripts/bin/schema-driven を使う。
- ボード page-design-ownership（Artifact https://claude.ai/artifact/K2wt1s8HphJVBqrnWasdLo）Q1〜Q4 決着（2026-10-13）：Q1 基盤は部品1つの形 ・ トークンの形とコントラストの下限 ・ 描画の仕組み ・ 検査だけ、部品の一覧と見た目とトークンの値は concrete、文書の決まった見た目は持たない。Q2 そろえる役はデザインシステムだけを持つ1つの Skill（名前と design-svg とまとめるかは後で）。Q3 skills-creator の配布物の手順が入口 ・ エンジンの写し ・ 共有デザインシステムの写しを各 Skill へ入れ、dist.json に版と SHA-256、全 Skill で版が1つかを検査。Windows を含む3 OS は公開の作業へ。Q4 各 Skill は入口 scripts/<Skill名> を1つ持ち SKILL.md はそれだけを呼ぶ、Skill 固有の能力は入口の中、エンジンは別の実行ファイル（入口の言語を選べるように）。
- ACDR 0176（カタログの中身を基盤から外す）・ 0177（7af0c675、基盤が自分で Skill のフォルダを作り公開する）承認 ・ コミット済み。ボード schema-driven-release（Artifact https://claude.ai/artifact/VDq8apjsXH3XW2bAXp7MjD）Q0〜Q4 決着。基盤の Skill のフォルダは `cd src/schema-driven && cargo xtask skill --out ../../.claude/skills/schema-driven` で作る（実行ファイルは scripts/bin/）。公開は .github/workflows/schema-driven.yml（tag schema-driven-v<版>）。古い release.yml ・ install スクリプトは消した（未コミットだった署名の変更は scratchpad の rel-old/ に退避、セッションが終わると消える）。
- schema-driven は完成（2026-10-13）：tag schema-driven-v0.1.0 で 3 OS の build ・ verify ・ publish ・ receive が通り、Release https://github.com/daidaiiro-519/daidaiiro/releases/tag/schema-driven-v0.1.0 に配布物3つ ・ SHA256SUMS ・ 証明がある。
- ボード skills-creator-on-base（Artifact https://claude.ai/artifact/ApJveJNsRA9PMZrvPeEiMD）Q0〜Q4 決着（2026-10-13）。古い skills-creator のボードは参照しない。要点：作られる Skill はソースを scripts/ に持ち、配る仕組みは持たない。入口は scripts/<Skill名>（拡張子なし、1行目で実行のしかた）と scripts/<Skill名>.cmd。基盤のエンジンだけ skills-creator の scaffold ・ engine が版を指して Release から取り、SHA-256 と証明を照合して scripts/bin/ に入れる（記録のファイルは置かない）。skills-creator 自身は基盤と同じ形（CODING.md ・ core と adapters ・ xtask）で作り直し、scripts/bin/skills-creator として置き、tag skills-creator-v<版> で公開。tool.json ・ engine.json は無し。
- skills-creator は完成（2026-10-16）：ACDR 0180（作り直し）・ 0181（基盤の Windows の不具合を直して schema-driven 0.1.1）。tag skills-creator-v0.1.0 で 3 OS の build ・ verify ・ publish ・ receive が通り、Release に配布物3つ ・ SHA256SUMS ・ 証明がある。Claude Code と Codex で SKILL.md の手順どおりに動き、Windows の PowerShell ・ cmd ・ Git Bash で入口（scripts/<名前> と .cmd）が動くことを実測した。
- 残りの小さなこと：利用者の Windows に python3 が無いと、Git Bash から入口の1行目（#!/usr/bin/env python3）が動かない。入口の雛形の注意書きをどうするかは未決。手元の gh は 2.46 で attestation が無い（scratchpad に 2.102.0 を取って使った）。
- 次の順：①brainstorming ②agents creator ③acdr。共有のデザインシステムの Skill は作らない（ux-advisor が完成イメージとしてデザインシステムを出し、各 Skill が持つ。2026-10-16 利用者の決定）。

## brainstorming の現在地（2026-10-17）

- 名前を brainstorming に変え、新しいボードは `.brainstorming/` に新しい形だけで作る。古い `.brainstorming-board/` は移さない（2026-10-16 利用者の決定）。
- ボード `.brainstorming-board/brainstorming-on-base/`（古いツールで作った。Artifact https://claude.ai/artifact/48KWG81WUhsam5WpiirDjS、ローカル 8752）の1回目。Q0 完成の条件（18件）・ Q1 ボードの中身 ・ Q2 画面とデザインシステム ・ Q3 Skill の作り ・ Q4 AI のチャットの内蔵、の5つが回答待ち。
- アドバイザーの回答はボードの advisors/、CLI の実測は measured/、原文の取り直し方（meta.json）と調べのまとめは sources/ にある（原文そのものは追跡しない）。
- ボードの本体は `.brainstorming/<ボード>.json`、回の記録 ・ 回答 ・ 図は `.brainstorming/<ボード>/`（基盤の check と render が直下しか読まないため。2026-10-17 利用者の選択）。
- 試作の画面：アプリ https://claude.ai/artifact/U4UvC4GPvBhxjVrWY4qEz1 ・ 論点 https://claude.ai/artifact/M73AEfCEP3GTi1u1e3211V（生成の台本はセッションの作業用の領域にだけあり、HTML はボードの Q2 に入っている）。
- brainstorming は完成（2026-10-17）：ACDR 0183 承認（19debd24）、tag brainstorming-v0.1.0 で 3 OS の build ・ verify ・ publish ・ receive が通った。受け入れの記録はボード brainstorming-on-base の measured/acceptance/。
- 利用者の提案（2026-10-17）：AI の CLI を呼ぶ能力（adapter）を基盤 schema-driven に持たせ、ACDR やほかの Skill でも使う。基盤の検証を「スキーマで機械的に見るもの」と「意味の検証」に分ける考え。ボードで決める予定（まだ立てていない）。
- 残る別の目的：ワークフローを3本から共通の1本にまとめる（skills-creator.yml の注記「私たちが管理するものが3つになったら」）。古い brainstorming-board の Skill と .mcp.json の登録の扱い。
