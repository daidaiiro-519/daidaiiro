# 共通の「はじめに」の正式反映

利用者が2026年10月25日に「この内容で正本化」と承認した、表紙1枚と本文9枚を正本へ反映する。対象は共通の「はじめに」のスライドと原稿であり、教材1本編、企画書、オリエンテーション、音声・動画の制作は含まない。

現行の正本は参照漏れを扱う旧題材である。確認済みの題材は、要件の解釈、不具合と再作業、動作修正後に残る設計、追加変更で広がり得る確認範囲から三教材の学びへつなぐ。

## 反映する差分

| 対象 | 理由 |
|---|---|
| `docs/workshops/ai-workshop/material-1/src/build_lessons.py` | 表紙の専用入力を使用し、表紙の原稿を台本へ含める。図の対応表は本文9枚へ合わせる。 |
| `docs/workshops/ai-workshop/material-1/src/build_narration.py` | 次回の音声制作で表紙の確認済み原稿を使用し、題名だけの読み上げへ戻らないようにする。 |
| `docs/workshops/ai-workshop/material-1/src/common_intro_visuals.py` | 本文と表紙の確認済み図を同じ表示classで読み込む。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/cover.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/cover.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-01.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-01.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-02.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-02.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-03.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-03.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-04.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-04.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-05.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-05.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-06.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-06.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-07.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-07.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-08.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-08.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-09.source.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-09.svg` | 同じ比較画面で確認済みの図を、本文の表示順に対応させて正本へ移す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/theme.json` | レビューで確認済みのwarm-paperの図の配色を使用する。 |
| `docs/workshops/ai-workshop/material-1/src/ledes.json` | 確認済みの導入文を本文9枚と対応させ、不要になった旧10枚目の項目を外す。 |
| `docs/workshops/ai-workshop/material-1/src/lesson-01-lead.json` | 確認済みの表紙と本文9枚の見出し・原稿へ改訂し、開発全体の改善から三教材へつなぐ。 |
| `docs/workshops/ai-workshop/material-1/src/words.json` | 既存のタスク説明の例外と同じ用途で、担当する変更の説明を指す二つの表現を許容する。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-10.source.svg` | 本文を9枚へ再構成したため、旧10枚目の図を外す。 |
| `docs/workshops/ai-workshop/material-1/src/figures/common-intro/slide-10.svg` | 本文を9枚へ再構成したため、旧10枚目の図を外す。 |
| `docs/workshops/ai-workshop/material-1/README.md` | 三教材共通の導入の位置づけと全10枚、入力と出力、旧音声・動画の状態を明示する。 |
| `docs/workshops/ai-workshop/material-1/out/index.html` | 最新のスライドと原稿へリンクし、動画と音声が改訂前の内容であることを明示する。 |

## 確認したこと

表紙と本文の見出し・導入文・原稿、および本文9枚の図がレビュー入力と一致することを照合した。表紙を含む音声制作用の入力も10枚となり、表紙の専用原稿を使用する。図の配色は確認済みテーマを使用する。

作業コピーから全10枚を描画し、見出しと図のはみ出しを検査した。旧音声・動画はそのままであり、正式反映後も改訂前と明記する。新しい動画の制作は別工程とする。

## 文書の10判定

1. 指し先：共通導入の入力と出力を明示。
2. 型：変更するファイルと理由を表で表示。
3. 数：表紙1枚と本文9枚の全10枚を照合。
4. 見出し：正式反映の対象を記載。
5. 結論：利用者が承認した反映対象を先に説明。
6. 形：既存の線画とwarm-paperのテーマを使用。
7. 用語：要件・モデル・設計・AI作業の役割を区別。
8. 読み手：旧動画と新しいスライドを混同させない。
9. 量：音声や動画の制作は別工程と明示。
10. 比喩：承認済み入力と正式反映の差分を直接記述。
