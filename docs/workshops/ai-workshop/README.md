# 下期AI活用ワークショップ

部長向けの企画書と、受講者向けの教材を置く。**見るものは各フォルダの `out/`、作るための材料は `src/` にある。**

## 見るもの

### 企画書（部長向け）

| もの | 中身 |
|---|---|
| [要点版のスライド](proposal/out/director-deck-brief.html) ・ [PDF](proposal/out/director-deck-brief.pdf) | 表紙1枚＋本編10枚＋付録28枚。ブラウザで開き、矢印キーでめくる |
| [詳細版のスライド](proposal/out/director-deck.html) ・ [PDF](proposal/out/director-deck.pdf) | 表紙1枚＋本編24枚＋付録14枚 |
| [要点版の1ページ版](proposal/out/director-deck-brief-all.html) ・ [詳細版の1ページ版](proposal/out/director-deck-all.html) | 全枚を縦に並べた版。通し読みに使う |
| [要点版の発表原稿](proposal/out/speaker-notes-brief.md) ・ [詳細版の発表原稿](proposal/out/speaker-notes.md) | 各枚に対応する原稿 |
| [運営案](proposal/out/workshop-plan.md) | テーマ ・ 教材 ・ 半年間の進行 ・ 支援 ・ 評価 ・ 工数 |
| [アンケート・効果測定案](proposal/out/survey-and-evaluation.md) | 設問 ・ 尺度 ・ KPI ・ 試用記録と、Microsoft Forms での実装 |
| [全枚の一覧画像](proposal/out/overview.png) | 詳細版の全枚を1枚に並べた画像 |

### 教材1（オリエンテーション＋課題の把握）

| もの | 中身 |
|---|---|
| [動画](material-1/out/videos/) | 本ごとの動画（mp4）。いまはオリエンテーション（`00-start.mp4`、14分44秒）と、教材1のはじめに（`01-lead.mp4`、旧原稿）の2本 |
| [スライド](material-1/out/slides/) | 本ごとのスライド（HTML）。9本 |
| [原稿](material-1/out/scripts/) | 本ごとの読み上げの原稿（Markdown） |

教材1の中身と、作り直す手順は [material-1/README.md](material-1/README.md) にある。

### 教材2（試作）

`material-2/` に、ドメイン駆動設計編の試作（3本ぶんのスライドと図）を置いている。まだ追跡していない。

## 作り直す

| 何を | どこで | 手順 |
|---|---|---|
| 企画書 | `proposal/src/` | `python3 build_deck.py`。HTML を `proposal/out/` へ出す。PDF と一覧画像は別に描画して出す |
| 教材1 | `material-1/src/` | [material-1/README.md](material-1/README.md) を参照 |

企画書の組み立ては、同梱の Skill（`proposal/src/skills/ai-workshop-slide-deck/`）の型を使う。教材1の組み立ても、この型と `proposal/src/visuals.py` の部品を借りる。

変更の経緯は [.acdr](../../../.acdr/) の記録が保持する。
