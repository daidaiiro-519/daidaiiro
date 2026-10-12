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
- 次の順：①skills-creator を作り直す ACDR（試作 → 完成の条件：練習用の Skill ・ 壊した例 ・ Claude Code と Codex ・ Windows の PowerShell で .cmd ・ 3 OS の公開）②デザインシステムの Skill ③ブレストボードを移す（15の Skill を src/ 側にするか scripts/ 側にするかもここで決める）。手元の gh は 2.46 で attestation が無いので上げる必要がある。

## 次にすること（順番）

1. Q5 の承認 → ボードを決着させる
2. ボード concrete-schema-roles（Artifact https://claude.ai/artifact/5BwJynkVP75zgTFBrU1chE、1回目）の Q1（3つの層と②の契約と責務）・ Q2（②を③と同じ道筋で扱う＝案 A、規則1つ）と Q3（creator は Agent Skills ・ Agent の形の雛形と検査だけを持つ）まで全部決着（2026-10-12）。ACDR 0170（規則と abstract の決まり）・ 0171（取り込むファイルを crate files の build.rs がフォルダから一覧にし、名前で引く。相対パスをなくした）承認・コミット済み。次は skills-creator の ACDR（雛形3つ・scaffold の --kind 等を外す・試験名から版を外す）。決着したら schema-driven の ACDR（規則と abstract の決まり）。そのあと skills-creator を仕上げる ACDR（tools.json、type_v1.rs と data/skill-v1 の名前から版を外す＝ボード skills-creator-rebuild 論点3の改め直し）。そのあと schema-driven のカタログの ACDR：カタログに部品を4つ足し、9つ変え、トークンを入れる。check が maxContains の超過を「contains のスキーマに合う要素が無い」と返す誤りも直す
3. advance ・ inspect ・ serve を `src/brainstorming-board/` に Rust で作り、ブレストボードを移す ACDR を出して適用する。移したら `src/skills-creator/tests/baseline-v3/brainstorming-board.json` を消す
4. acdr を新しい形で作り直す（0162 もここで扱う）
5. agents creator：新しい形のブレストボードを試す1枚目として作る（最初の論点は「Agent」が何を指すか）
6. 残りの Skill を移す。アドバイザーは最後に Agent にする
7. 配布物（tools.json の OS ごとの SHA-256 ・ accept と dist ・ release.yml ・ インストールの台本。作業ツリーに未コミットの変更あり）と、スキーマの無い JSON の洗い出し

## 移すときに使うもの

- 移す試験：`cd src/skills-creator && cargo test`。版3 の Skill を移したら、その Skill の baseline を消す（消し忘れると試験が落ちる）
- 古い skills-creator は `src/skills-creator-v3/` に凍結してある
