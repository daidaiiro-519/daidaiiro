# 下期AI活用ワークショップ

部長向けの企画書と、受講者向けの教材を置く。最上位のフォルダは4つである。

| フォルダ | 中身 |
|---|---|
| `dist/` | **配るもの。** 本ごとに1フォルダで、HTML ・ PDF ・ 原稿 ・ 動画 ・ 字幕 ・ 音声が同じフォルダにある。常に最新版だけを置く |
| `src/` | 作るための材料。`src/shared/` が配色 ・ 図の部品 ・ スライドの型 ・ 書籍の表紙 ・ 読み辞書を持つ |
| `archive/` | 前の版。版ごとのフォルダに、当時のファイル名のまま残す |
| `review/` | 見直しの作業コピーと検証の記録。配るものではない |

## 配るもの（`dist/`）

どの本のフォルダも、中の名前は同じである。

| ファイル | 中身 |
|---|---|
| `slides.html` | スライド。ブラウザで開き、矢印キーでめくる |
| `slides-all.html` ・ `slides.pdf` | 全枚を縦に並べた版と、そのPDF。配る ・ 印刷するときに使う |
| `script.md` | 枚ごとの読み上げの原稿 |
| `video.mp4` ・ `video.vtt` | 動画（字幕を焼き込み済み、1280×816）と、字幕だけのファイル |
| `audio.mp3` | 動画の音声だけ |
| `poster.png` | 動画の表紙の画像 |
| `video-timeline.json` ・ `video-release.json` | 枚ごとの時刻と、動画の検証の記録 |

[dist/index.html](dist/index.html) から、オリエンテーション ・ はじめに ・ 教材1の本編5本の動画と音声を順に再生できる。

### 企画書（部長向け）　`dist/proposal/`

| もの | 中身 |
|---|---|
| [詳細版](dist/proposal/detail/slides.html) ・ [PDF](dist/proposal/detail/slides.pdf) ・ [原稿](dist/proposal/detail/script.md) | 表紙1枚＋本編28枚＋付録9枚 |
| [詳細版の動画](dist/proposal/detail/video.mp4) | 本編29枚（表紙〜まとめ）を、原稿のとおりに読み上げた動画。約41分。付録は含まない |
| [要点版](dist/proposal/brief/slides.html) ・ [PDF](dist/proposal/brief/slides.pdf) ・ [原稿](dist/proposal/brief/script.md) | 表紙1枚＋本編14枚＋付録23枚 |
| [運営案](dist/proposal/workshop-plan.md) | テーマ ・ 教材 ・ 半年間の進行 ・ 支援 ・ 評価 ・ 工数 |
| [アンケート・効果測定案](dist/proposal/survey-and-evaluation.md) | 設問 ・ 尺度 ・ KPI ・ 答えやすさの方針と、Microsoft Forms での実装 |
| [Forms の Copilot に渡すプロンプト](dist/proposal/forms-copilot-prompts.md) | 4本のアンケートを生成するプロンプトと、生成後に手で設定する分岐 ・ 設定 |
| [全枚の一覧画像](dist/proposal/overview.png) | 詳細版の全枚を1枚に並べた画像 |

### オリエンテーション　`dist/orientation/`

全14枚、動画は約16分26秒。2026-10-26改訂（ACDR 0207 ・ 0208）。

### はじめに（三教材共通）　`dist/intro/`

全10枚、動画は約22分39秒。2026-10-25改訂（ACDR 0205 ・ 0206）。

### 教材1　`dist/lesson-1/`

本編5本。`01-cause`（原因を知る）・ `02-word`（意味を決める）・ `03-scope`（範囲を決める）・ `04-rule`（条件を決める）・ `05-summary`（具体と抽象を振り返る）。2026-10-07の版（v3）のままで、いまの教材名「目的と要件の明確化」への改訂はまだである。

### 教材2（試作）　`dist/lesson-2/`

ドメイン駆動設計編の試作3本（`00-intro` ・ `02-context` ・ `09-compare`）のスライドと原稿。

## 前の版（`archive/`）

| 版 | 置き場所 | 日付 | 内容 | 記録 |
|---|---|---|---|---|
| 企画書 v4（最新） | `dist/proposal/` | 2026-10-28 | 企画の目的を「AIの活用を、仕事全体の生産性と品質の改善につなげる」に改め、教材名と三教材の役割を共通方針と正式版のオリエンテーションに合わせた。主KPIを目的と要件の明確化率と改善の達成率にし、アンケートを事前7問・参加時は改善1件で8問に絞った。定員を15〜20名にした | [ACDR 0209](../../../.acdr/0209-企画書の正式版改訂/) |
| 企画書 v3 | `archive/proposal/v3/` | 2026-10-07 | ワークショップの目的を「目的を正しく言葉にしてAIに伝え、達成できるようになる」にした | [ACDR 0142](../../../.acdr/0142-企画書とオリエンテーションを言語化の軸へ改める/) |
| 企画書 v2 | `archive/proposal/v2/` | 2026-09-29 | 身につける力を目的をAIで達成する力へ、主KPIを目的の達成へ改めた | [ACDR 0032](../../../.acdr/0032-企画書を目的の軸へ改める/) |
| 企画書 v1 | `archive/proposal/v1/` | 2026-09-27 | 課題をAIで解決する力を軸にした版 | ── |
| 教材 v2 | `archive/lessons/v2/` | 2026-09-29 | オリエンテーションを目的をAIで達成する力の軸へ改め、3つの教材の名前を目的の把握 ・ 言葉の定義 ・ 仕組みの構築にした | [ACDR 0034](../../../.acdr/0034-オリエンテーションを目的の軸へ改める/) |
| 教材 v1 | `archive/lessons/v1/` | 2026-09-27 | 課題をAIで解決する力を軸にしたオリエンテーション（スライド ・ 原稿 ・ 動画）と、教材1のはじめに ・ まとめの表紙 | ── |

## 作り直す（`src/`）

| 何を | どこで | 手順 |
|---|---|---|
| 企画書 | `src/proposal/` | `python3 build_deck.py` で `dist/proposal/detail/` と `brief/` へ出す。`python3 build_survey_docs.py` でアンケートの設計書2本を出す。PDF と一覧画像は別に描画して出す |
| 企画書の動画 | `src/proposal/` | `python3 build_video.py narration` で原稿を音声の入力にし、slide-deck-speech Skill で合成する。`python3 build_video.py previews --browser <ブラウザ>` で枚の画像を撮り、`python3 build_video.py video` で `dist/proposal/detail/` へ出す |
| オリエンテーション ・ はじめに ・ 教材1 | `src/lessons/` | 7本を1つの仕組みで作る。手順は [src/lessons/README.md](src/lessons/README.md) を参照 |
| 教材2（試作） | `src/lesson-2/` | `python3 build_lessons.py` で `dist/lesson-2/` へ出す |

スライドの作成には、同梱の Skill（`src/shared/skills/ai-workshop-slide-deck/`）の型と、`src/shared/visuals.py` の部品を使う。教材のスライドは、企画書の `src/proposal/build_deck.py` から型と CSS を借りる。

変更の経緯は [.acdr](../../../.acdr/) の記録が保持する。フォルダ構成は ACDR 0214 で今の形にした。それより前の記録が指すパス（`proposal/out/` ・ `material-1/` など）は、当時の場所である。
