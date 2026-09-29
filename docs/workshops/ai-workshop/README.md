# 下期AI活用ワークショップ

部長向けの企画書と、受講者向けの教材を置く。**見るものは各フォルダの `out/`、作るための材料は `src/` にある。**

## 見るもの

### 企画書（部長向け）

| もの | 中身 |
|---|---|
| [要点版のスライド](proposal/out/director-deck-brief.html) ・ [PDF](proposal/out/director-deck-brief.pdf) | 表紙1枚＋本編10枚＋付録27枚。ブラウザで開き、矢印キーでめくる |
| [詳細版のスライド](proposal/out/director-deck.html) ・ [PDF](proposal/out/director-deck.pdf) | 表紙1枚＋本編23枚＋付録14枚 |
| [要点版の1ページ版](proposal/out/director-deck-brief-all.html) ・ [詳細版の1ページ版](proposal/out/director-deck-all.html) | 全枚を縦に並べた版。通し読みに使う |
| [要点版の発表原稿](proposal/out/speaker-notes-brief.md) ・ [詳細版の発表原稿](proposal/out/speaker-notes.md) | 各枚に対応する原稿 |
| [運営案](proposal/out/workshop-plan.md) | テーマ ・ 教材 ・ 半年間の進行 ・ 支援 ・ 評価 ・ 工数 |
| [アンケート・効果測定案](proposal/out/survey-and-evaluation.md) | 設問 ・ 尺度 ・ KPI ・ 試用記録と、Microsoft Forms での実装 |
| [Forms の Copilot に渡すプロンプト](proposal/out/forms-copilot-prompts.md) | 事前 ・ 中間 ・ 終了時の3本のアンケートを生成するプロンプトと、生成後に手で設定する分岐 ・ 設定 |
| [全枚の一覧画像](proposal/out/overview.png) | 詳細版の全枚を1枚に並べた画像 |

`proposal/out/` には常に最新版を置く。前の版は `proposal/out/versions/<版>/` に、同じファイル名のまま残す。

| 版 | 置き場所 | 日付 | 内容 | 記録 |
|---|---|---|---|---|
| v2（最新） | `proposal/out/` | 2026-09-29 | 身につける力を目的をAIで達成する力へ、主KPIを目的の達成へ改め、次の期の扱いを目指す姿と組織への浸透へ置き換えた | [ACDR 0032](../../../.acdr/0032-企画書を目的の軸へ改める/) |
| v1 | `proposal/out/versions/v1/` | 2026-09-27 | 課題をAIで解決する力を軸にした版 | ── |

### 教材1（オリエンテーション＋目的の把握）

| もの | 中身 |
|---|---|
| [動画](material-1/out/videos/) | 本ごとの動画（mp4）。いまはオリエンテーション（`00-start.mp4`、v1 の原稿）と、教材1のはじめに（`01-lead.mp4`、旧原稿）の2本 |
| [スライド](material-1/out/slides/) | 本ごとのスライド（HTML）。9本 |
| [原稿](material-1/out/scripts/) | 本ごとの読み上げの原稿（Markdown） |

`material-1/out/` には常に最新版を置く。前の版は `material-1/out/versions/<版>/` に、同じフォルダ構成のまま残す。

| 版 | 置き場所 | 日付 | 内容 | 記録 |
|---|---|---|---|---|
| v2（最新） | `material-1/out/` | 2026-09-29 | オリエンテーションを目的をAIで達成する力の軸へ改め、3つの教材の名前を目的の把握 ・ 言葉の定義 ・ 仕組みの構築にした。動画はまだ作り直していない | [ACDR 0034](../../../.acdr/0034-オリエンテーションを目的の軸へ改める/) |
| v1 | `material-1/out/versions/v1/` | 2026-09-27 | 課題をAIで解決する力を軸にしたオリエンテーション（スライド ・ 原稿 ・ 動画）と、教材1のはじめに ・ まとめの表紙 | ── |

教材1の中身と、作り直す手順は [material-1/README.md](material-1/README.md) にある。

### 教材2（試作）

`material-2/` に、ドメイン駆動設計編の試作（3本ぶんのスライドと図）を置いている。

## 作り直す

| 何を | どこで | 手順 |
|---|---|---|
| 企画書 | `proposal/src/` | `python3 build_deck.py`。HTML を `proposal/out/` へ出す。PDF と一覧画像は別に描画して出す |
| 教材1 | `material-1/src/` | [material-1/README.md](material-1/README.md) を参照 |

企画書の組み立ては、同梱の Skill（`proposal/src/skills/ai-workshop-slide-deck/`）の型を使う。教材1の組み立ても、この型と `proposal/src/visuals.py` の部品を借りる。

変更の経緯は [.acdr](../../../.acdr/) の記録が保持する。
