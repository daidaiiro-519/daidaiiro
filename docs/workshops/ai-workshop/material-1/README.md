# 教材1 ── オリエンテーションと目的の把握

受講者向けの動画教材。オリエンテーション1本、教材1のはじめに1本、本編5本の計7本である。


2026年10月17日に、説明文と表の境界、図の中の情報のまとまりを見直した。既存の配色とカードの形に合わせ、スライド・PDF・オリエンテーションの動画へ反映した。決定はACDR 0184に記録した。



2026年10月17日に、オリエンテーションの原稿の重複と予告の食い違いを修正し、動画の切り替えに間と短いフェードを加えた。決定はACDR 0185に記録した。


## 見るもの（`out/`）

`out/index.html` から、7本の動画と音声を順に再生できる。

| フォルダ | 中身 |
|---|---|
| `out/audio/` | 本ごとの音声（MP3） |
| `out/videos/` | 本ごとの動画（mp4）と字幕（WebVTT）。動画は枚ごとの画像と読み上げの音声を結合し、スライドの下の帯に字幕を焼き込む（1280×816）。mp4 だけで、どのプレーヤーでも字幕が出る |
| `out/slides/` | 本ごとのスライド（HTML）。ブラウザで開き、矢印キーでめくる |
| `out/scripts/` | 本ごとの読み上げの原稿（Markdown） |
| `out/pdf/` | 全枚を縦に並べたスライド（HTML）と、そのPDF。配る・印刷するときに使う |

| 本 | 題 |
|---|---|
| 00 | オリエンテーション ── AIを、自分の仕事で使い続けるために |
| 01 | 目的の把握（教材1のはじめに） |
| 02〜06 | 原因を知る ／ 意味を決める ／ 範囲を決める ／ 条件を決める ／ 具体と抽象を振り返る |

2026年10月11日のスライドと原稿から、オリエンテーション、はじめに、本編5本の音声と動画を作成した。音声はTakumiの日本語ニューラル音声を使用した。

2026年10月13日に、比較ページで確認したオリエンテーションを正式版へ反映した。仕事の目的をAIで達成できるようになる開催目的を示し、仕事の変化、教材、応用、書籍、学習と実践の進め方を説明する。表紙1枚＋本編11枚。2026年10月17日の原稿と遷移の改訂後は約14分24秒。決定は [ACDR 0178](../../../../.acdr/0178-オリエンテーションの正式版改訂/index.html) に記録した。

書籍の表紙を含むスライドHTML、PDF、動画と該当するプレビュー画像は、ローカルで生成して確認する。公開リポジトリへのコミットには含めない。

## 作るための材料（`src/`）

| もの | 役割 |
|---|---|
| `lesson-0N-*.json` | 本ごとの枚と原稿の正本 |
| `ledes.json` | 枚ごとの導入文 |
| `words.json` | 同じものを指す語と、使用しない語 |
| `orientation_current_visuals.py` | 正式版の枚と図の対応 |
| `*_visuals.py` | 枚ごとの図 |
| `examples/` | 図と原稿が使う議事録の例 |
| `narration/` | 読み上げの入力と、合成した音声（`cache/`） |
| `previews/` | 描画したスライドの画像。動画と照合の入力が使う |
| `review-log.json` | 1枚ずつ照合したときの、観点の群ごとの確認の結果 |
| `promotion-table.md` | 教材1の語と、ドメイン駆動設計の用語の対応 |
| `work/` | 動画を作る途中の区間ファイル。追跡しない |

## 作り直す手順

`src/` で実行する。

| 手順 | コマンド | 出すもの |
|---|---|---|
| 1. スライドを作る | `python3 build_lessons.py` | `out/slides/` ・ `out/scripts/`。語と図の検査も実行する |
| 2. 図を検査する | `python3 check_figures.py` | 文字の重なり ・ はみ出しの検出 |
| 3. 画像にする | `python3 render_previews.py --browser <ブラウザ> [本の番号…]` | `previews/` |
| 3b. PDFにする | `python3 export_pdf.py --browser <ブラウザ> [本の番号…]`（既定は 00） | `out/pdf/` |
| 4. 読み上げの入力を作る | `python3 build_narration.py [00-start …]` | `narration/narration.json` |
| 5. 音声を合成する | `slide-deck-speech synth narration`（slide-deck-speech Skill、`AWS_PROFILE=dev`）。読みの辞書 `narration/lexicon.pls` は、変えたときに `slide-deck-speech lexicon narration/lexicon.pls aiworkshop` で登録し直す | `narration/narration.out.json` と音声 |
| 6. 動画にする | `python3 build_videos.py [00-start …]` | `out/videos/` の動画と字幕。字幕は音声の文の時刻から組み、Noto Sans JP で焼き込む。書体の置き場は `SUB_FONTS`（既定は `~/.local/share/fonts`） |

原稿を持つデッキを1枚ずつ照合するときは、slide-deck Skill の照合の工程に従う。
`python3 review_input.py build <変更前> <review.json> --log review-log.json` で照合の入力を作り、
`slide-deck review` で比較ページを、`slide-deck export` で台本 ・ 確認記録 ・ 観点ごとの結果を出す。

決定と根拠は `.brainstorming-board/material-abstraction/` と `.acdr/` が保持する。
