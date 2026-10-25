# オリエンテーションの音声・字幕付き動画の更新

ACDR 0207で正本化したオリエンテーション全14枚（表紙1枚と本文13枚）から、音声と字幕付き動画を作り直す。旧版の音声と動画は、改訂前のスライドと原稿のままである。

## 生成した条件

| 項目 | 内容 |
|---|---|
| 原稿 | 正本のlesson-00-start.json（表紙専用の原稿と本文13枚）。短縮も読み仮名の追加もしていない |
| 音声 | slide-deck-speechで合成。声はTakumi、エンジンはneural、AWS_PROFILEはdev。Pollyに登録された読み辞書aiworkshopがリポジトリのlexicon.plsと一致することを確認した |
| 画像 | 正本のスライドから描画したpreviews/00-01.png〜00-14.png |
| 動画 | build_videos.pyで、版を固定したFFmpegのイメージ（jrottenberg/ffmpeg:7.1-alpine）を使用。枚ごとの間はvideo-pacing.jsonの既存の設定のまま |
| 音声のみ | 動画の音声を128kbps・48kHzのMP3へ書き出した |
| ポスター | 表紙のスライド画像previews/00-01.png（共通の「はじめに」と同じ方法） |

## 検証の結果

| 項目 | 結果 |
|---|---|
| 枚数と並び | 14枚。動画の枚の並び、合成した音声の並び、原稿が一致する |
| 長さ | 動画986.344秒（約16分26秒）、音声のみ986.400秒 |
| 形式 | 1280×816、映像H.264、音声AAC |
| 字幕 | 187件。すべて動画の長さの中にあり、重ならない |
| 全編のデコード | エラーなし |
| 映像と元のスライド | 各枚の読み上げ開始から2.5秒後のフレームと元の画像の差（RMS）が最大1.676で、基準の8未満 |
| 目視 | 14枚のフレームで、字幕がスライドの下の帯に2行以内で収まり、図を隠さないことを確認した |
| 再生 | 共通の閲覧ページで、内容ハッシュ付きURLの動画が読み込まれ、再生時間が進むことを確認した |

全音声の聴取と、全187件の字幕の目視確認はしていない。検証の記録はverification.json、フレームはframe-01.png〜frame-14.pngとcontact.png。

## 反映する出力

out/videos/00-start.mp4、00-start.vtt、00-start-timeline.json、00-start-release.json、out/audio/00-start.mp3、out/videos/posters/00-start.png、src/previews/00-01.png〜00-14.png、src/narration/のnarration.json・narration.full.json・narration.out.jsonと、合成した音声の新しいキャッシュ14枚。材料のページout/index.htmlの長さ、README、共通の閲覧ページの生成元review_navigation.pyも更新する。
