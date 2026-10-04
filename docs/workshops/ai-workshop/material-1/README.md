# 教材1 ── オリエンテーションと課題の把握

受講者向けの動画教材。オリエンテーション1本、教材1のはじめに1本、本編6本、まとめ1本の計9本である。

## 見るもの（`out/`）

| フォルダ | 中身 |
|---|---|
| `out/videos/` | 本ごとの動画（mp4）と字幕（WebVTT）。動画は枚ごとの画像と読み上げの音声を結合し、字幕を切り替え式のトラックとして持つ |
| `out/slides/` | 本ごとのスライド（HTML）。ブラウザで開き、矢印キーでめくる |
| `out/scripts/` | 本ごとの読み上げの原稿（Markdown） |
| `out/pdf/` | 全枚を縦に並べたスライド（HTML）と、そのPDF。配る・印刷するときに使う |

| 本 | 題 |
|---|---|
| 00 | オリエンテーション ── 目的をAIで達成できるようになる |
| 01 | 課題の把握（教材1のはじめに） |
| 02〜07 | 原因を知る ／ 意味を決める ／ 範囲を決める ／ 条件を決める ／ 揺らぎを直す ／ 抽象の高さを合わせる |
| 08 | 教材1のまとめ ── 次の教材へ |

## 作るための材料（`src/`）

| もの | 役割 |
|---|---|
| `lesson-0N-*.json` | 本ごとの枚と原稿の正本 |
| `ledes.json` | 枚ごとの導入文 |
| `words.json` | 同じものを指す語と、使用しない語 |
| `*_visuals.py` | 枚ごとの図 |
| `examples/` | 図と原稿が使う議事録の例 |
| `narration/` | 読み上げの入力と、合成した音声（`cache/`） |
| `previews/` | 描画したスライドの画像。動画と照合の入力が使う |
| `review-log.json` | 1枚ずつ照合したときの、観点の群ごとの確認の結果 |
| `promotion-table.md` | 教材1の語と、ドメイン駆動設計の用語の対応 |
| `work/` | 動画を組む途中の区間ファイル。追跡しない |

## 作り直す手順

`src/` で実行する。

| 段 | コマンド | 出すもの |
|---|---|---|
| 1. スライドを組む | `python3 build_lessons.py` | `out/slides/` ・ `out/scripts/`。語と図の検査も実行する |
| 2. 図を検査する | `python3 check_figures.py` | 文字の重なり ・ はみ出しの検出 |
| 3. 画像にする | `python3 render_previews.py --browser <ブラウザ> [本の番号…]` | `previews/` |
| 3b. PDFにする | `python3 export_pdf.py --browser <ブラウザ> [本の番号…]`（既定は 00） | `out/pdf/` |
| 4. 読み上げの入力を組む | `python3 build_narration.py [00-start …]` | `narration/narration.json` |
| 5. 音声を合成する | `slide-deck-speech synth narration`（slide-deck-speech Skill、`AWS_PROFILE=dev`）。読みの辞書 `narration/lexicon.pls` は、変えたときに `slide-deck-speech lexicon narration/lexicon.pls aiworkshop` で登録し直す | `narration/narration.out.json` と音声 |
| 6. 動画にする | `python3 build_videos.py [00-start …]` | `out/videos/` の動画と字幕。字幕は、音声の文の時刻から組む |

原稿を持つデッキを1枚ずつ照合するときは、slide-deck Skill の照合の工程に従う。
`python3 review_input.py build <変更前> <review.json> --log review-log.json` で照合の入力を組み、
`slide-deck review` で比較ページを、`slide-deck export` で台本 ・ 確認記録 ・ 観点ごとの結果を出す。

決定と根拠は `.brainstorming-board/material-abstraction/` と `.acdr/` が保持する。
