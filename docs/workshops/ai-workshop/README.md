# 社内AI人材育成ワークショップ

部長向けの説明資料と、開催条件を詰めるための運営案。

| ファイル | 用途 |
|---|---|
| [director-deck.html](director-deck.html) | 表紙1枚＋本編14枚＋付録12枚のスライド。ブラウザで開き、矢印キー・画面下のボタン・スワイプで操作 |
| [director-deck.pdf](director-deck.pdf) | 共有・配布用のPDF |
| [speaker-notes.md](speaker-notes.md) | 各スライドに対応する部長向け発表原稿 |
| [workshop-plan.md](workshop-plan.md) | テーマ、概念教材、半年間の進行、支援、評価、工数の運営案 |
| [survey-and-evaluation.md](survey-and-evaluation.md) | 共通・PM／PL・開発者向けの設問、段階評価、試用・投票、成果報告の設計 |
| [previews/overview.png](previews/overview.png) | 全スライドの一覧画像 |
| [quality-check-2026-09-13.md](quality-check-2026-09-13.md) | 最新版の描画・文章・独立レビューの確認記録 |

主催者指定の条件と今回の提案は、運営設計案の最初の表で区別している。受講者向けの音声動画は、構成案までを含み、動画そのものはまだ制作していない。

## 再生成

`build_deck.py` がスライド本文と発表原稿を保持する。同梱の `skills/ai-workshop-slide-deck/assets/deck-template.html` を基に、HTMLと発表原稿を生成する。Python 3の標準ライブラリだけで実行できる。

```bash
cd ai-workshop # ZIPの展開先。リポジトリでは docs/workshops/ai-workshop
python3 build_deck.py
```

HTMLは外部ファイルの読み込みなしで表示できる。フォントは端末にある日本語フォントを使い、端末によって字形が変わる。PDFは作成時の表示を保持する。HTMLを変更した場合、PDFとプレビューは別途再出力する。

## 同梱Skill

[ai-workshop-slide-deck](skills/ai-workshop-slide-deck/SKILL.md) に作成・編集・描画確認の手順とテンプレートを同梱している。
[デザイントークン](skills/ai-workshop-slide-deck/references/design-tokens.md) は現在のスライドの配色・書体・寸法を基にしている。

AIへの依頼例：

> skills/ai-workshop-slide-deck/SKILL.md を読み、この資料の配色とレイアウトを保ってスライドを編集してください。変更内容：……

Skillを別の環境へ登録する場合は `ai-workshop-slide-deck` フォルダをその環境のSkill配置先へコピーし、資料の展開先も伝える。再生成に必要なので、資料側の同梱Skillフォルダも保持する。
