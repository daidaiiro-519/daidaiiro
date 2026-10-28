# 企画書の音声・字幕付き動画の作成

ACDR 0209 で正本にした企画書の詳細版から、本編29枚（表紙〜まとめ）の音声と字幕付き動画を作った。ACDR 0212 の承認を受けて、正本へ置いた。

## 置いたもの

| 対象 | 内容 |
|---|---|
| `proposal/src/` | build_video.py（新規）、narration/（入力 ・ 出力 ・ 29枚の音声と時刻）、previews/01.png〜29.png |
| `proposal/out/videos/` | director-deck.mp4 ・ .vtt ・ -timeline.json ・ -release.json ・ posters/director-deck.png |
| `proposal/out/audio/` | director-deck.mp3 |
| `material-1/src/narration/lexicon.pls` | SIer ・ GitHub ・ Confluence の読みを追加。Polly の aiworkshop にも登録した |
| `README.md` | 見るものの表と、作り直す手順の表 |

パスは docs/workshops/ai-workshop/ からの相対パス。

## 確認

動画は41分07秒、字幕は494件である。枚の並び、原稿、長さ、字幕の時刻、全編のデコード、元のスライドとの差（最大1.99）を検証した。29枚のフレームを目視した。全音声の聴取はしていない。正本で原稿から生成した音声の入力は、作業コピーのものと1バイトも変わらない。

記録は .acdr/0212-企画書の動画を作る/ と review/proposal-orientation-20261024/proposal-20261026/video-20261028/ にある。
