# オリエンテーション・はじめに・教材1の作る材料

オリエンテーション1本、三教材共通のはじめに1本、教材1の本編5本の計7本を、1つの仕組みで作る。7本とも `lesson-0N-*.json` を正本にし、`build_lessons.py` で組む。出来上がりは `dist/` の本ごとのフォルダへ置く（対応は `places.py`）。

| 本 | 題 | 置き場所 |
|---|---|---|
| 00 | オリエンテーション | `dist/orientation/` |
| 01 | はじめに：三教材共通の題材と学習の流れ | `dist/intro/` |
| 02〜06 | 原因を知る ／ 意味を決める ／ 範囲を決める ／ 条件を決める ／ 具体と抽象を振り返る | `dist/lesson-1/01-cause/` 〜 `05-summary/` |

## 作るための材料（`src/lessons/`）

| もの | 役割 |
|---|---|
| `lesson-0N-*.json` | 本ごとの枚と原稿の正本 |
| `ledes.json` | 枚ごとの導入文 |
| `words.json` | 同じものを指す語と、使用しない語 |
| `orientation_current_visuals.py` | 正式版の枚と図の対応 |
| `*_visuals.py` | 枚ごとの図 |
| `examples/` | 図と原稿が使う議事録の例 |
| `narration/` | 読み上げの入力と、合成した音声（`cache-seikabutsu-v1/`）。読みの辞書は `src/shared/lexicon.pls` |
| `previews/` | 描画したスライドの画像。動画と照合の入力が使う |
| `places.py` | 本と、`dist/` の置き場所の対応 |
| `media-check.json` | 動画7本の形式の検証の記録 |
| `work/` | 動画を作る途中の区間ファイル。追跡しない |

## 作り直す手順

`src/lessons/` で実行する。出すものは、`dist/` の本ごとのフォルダに置く。

| 手順 | コマンド | 出すもの |
|---|---|---|
| 1. スライドを作る | `python3 build_lessons.py` | `slides.html` ・ `script.md`。語と図の検査も実行する |
| 2. 図を検査する | `python3 check_figures.py` | 文字の重なり ・ はみ出しの検出 |
| 3. 画像にする | `python3 render_previews.py --browser <ブラウザ> [本の番号…]` | `previews/` |
| 3b. PDFにする | `python3 export_pdf.py --browser <ブラウザ> [本の番号…]`（既定は 00） | `slides-all.html` ・ `slides.pdf` |
| 4. 読み上げの入力を作る | `python3 build_narration.py [00-start …]` | `narration/narration.json` |
| 5. 音声を合成する | `slide-deck-speech synth narration`（slide-deck-speech Skill、`AWS_PROFILE=dev`）。読みの辞書 `../shared/lexicon.pls` は、変えたときに `slide-deck-speech lexicon ../shared/lexicon.pls aiworkshop` で登録し直す | `narration/narration.out.json` と音声 |
| 6. 動画にする | `python3 build_videos.py [00-start …]` | `video.mp4` ・ `video.vtt` ・ `video-timeline.json`。字幕は音声の文の時刻から組み、Noto Sans JP で焼き込む。書体の置き場は `SUB_FONTS`（既定は `~/.local/share/fonts`） |

原稿を持つデッキを1枚ずつ照合するときは、slide-deck Skill の照合の工程に従う。
`python3 review_input.py build <変更前> <review.json> --log ../../review/lessons-notes/review-log.json` で照合の入力を作り、
`slide-deck review` で比較ページを、`slide-deck export` で台本 ・ 確認記録 ・ 観点ごとの結果を出す。

決定と根拠は `.brainstorming-board/material-abstraction/` と `.acdr/` が保持する。

## 経緯

2026年10月11日のスライドと原稿から、オリエンテーション、はじめに、本編5本の音声と動画を作成した。音声はTakumiの日本語ニューラル音声を使用した。

2026年10月13日に、比較ページで確認したオリエンテーションを正式版へ反映した。仕事の目的をAIで達成できるようになる開催目的を示し、仕事の変化、教材、応用、書籍、学習と実践の進め方を説明する。表紙1枚＋本編13枚。2026年10月17日の教材の閲覧と支援の案内の追加後は約16分27秒。決定は [ACDR 0178](../../../../../.acdr/0178-オリエンテーションの正式版改訂/index.html) に記録した。

2026年10月17日に、説明文と表の境界、図の中の情報のまとまりを見直した。既存の配色とカードの形に合わせ、スライド・PDF・オリエンテーションの動画へ反映した。決定はACDR 0184に記録した。

2026年10月17日に、オリエンテーションの原稿の重複と予告の食い違いを修正し、動画の切り替えに間と短いフェードを加えた。決定はACDR 0185に記録した。

2026年10月17日に、教材の閲覧方法と実践中の支援を独立した2枚として追加した。企画書の図を使用し、ポータル、日程、サポート体制の順に説明する。決定はACDR 0187に記録した。

2026年10月25日、三教材共通の「はじめに」を表紙1枚と本文9枚へ改訂した。最新のスライドと原稿は dist/intro/slides.html と script.md。目的・要件、業務モデル・設計、AgentがSkillsを使う作業と確認の手順へつなぐ。表紙の見出し・導入文・原稿はlesson-01-lead.jsonのcoverが持つ。図はfigures/common-intro/を使用する。音声・字幕付き動画も同じ原稿で更新した（全10枚、約22分39秒）。決定はACDR 0205に記録する。

音声・動画の生成はACDR 0206に記録した。Takumiの日本語ニューラル音声と読み辞書を使用し、253件の字幕を下の専用帯へ表示する。

2026年10月26日、オリエンテーションを表紙1枚と本文13枚の全14枚へ改訂した。最新のスライドと原稿は dist/orientation/slides.html と script.md。仕事全体の生産性と品質を高める到達点と、目的と要件の明確化・業務のモデル化と設計・AI活用の実践と改善の三教材を説明する。表紙の見出し・導入文・原稿はlesson-00-start.jsonのcoverが持つ。決定はACDR 0207に記録した。

オリエンテーションの音声・字幕付き動画も同じ原稿で更新した（全14枚、約16分26秒、字幕187件）。生成はACDR 0208に記録した。
