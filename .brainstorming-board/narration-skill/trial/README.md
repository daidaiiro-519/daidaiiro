# 試作：原稿から音声付きスライド動画まで

ブレストボード「学習教材の音声を作るSkill」の5件の決着を、実際に通して検証した記録である。
2026-09-22に実行した。

## 何を検証したか

| 論点 | 検証した内容 | 結果 |
|---|---|---|
| 受け持ち範囲 | 原稿から、枚ごとの音声と尺までを出力する | `narration.out.json` に音声・尺・長さが揃う |
| 実行の場所 | 結合の道具を手元へ導入せずに動画を作成する | コンテナの中の ffmpeg で `lesson.mp4` を出力した |
| 継ぎ目の形 | 枚のidだけで、画像と音声を突き合わせる | `slide-01` で `previews/slide-01.png` と対応した |
| 作り直しの単位 | 鍵が在る枚を、再合成せずに取り出す | 2回目の実行で3枚とも「取り出す」になった |
| 声と読み方 | ニューラルの日本語の声を比較する | Takumi ・ Kazuha ・ Tomoko の3つを同じ文で合成した |

## 置いてあるもの

| 場所 | 中身 |
|---|---|
| `narration.json` | 入力。発表原稿の1〜3枚目から作成した。枚のidと読み上げる文を持つ |
| `narration.out.json` | 出力。枚のidごとに、鍵・音声・尺・長さを持つ |
| `synth.sh` | 合成の手順。鍵を作成し、在れば取り出し、無ければ合成する |
| `cache/` | 鍵の名前が付いた音声と尺。名前が鍵そのものである |
| `movie/lesson.mp4` | 結合した動画（80秒）。1〜3枚目を順に再生する |
| `voice-*.mp3` | 声の比較。同じ文（1枚目）を3つの声で合成した |

## 実行の方法

```
aws sso login --profile dev
AWS_PROFILE=dev AWS_REGION=ap-northeast-1 ./synth.sh Takumi
docker run --rm -v "$PWD:/w" -w /w jrottenberg/ffmpeg:7.1-alpine \
  -y -loop 1 -i slide-01.png -i cache/<鍵>.mp3 \
  -c:v libx264 -tune stillimage -c:a aac -pix_fmt yuv420p -shortest movie/slide-01.mp4
```

## 判明したこと

1枚あたりの原稿は75〜237文字で、同期の合成の上限（3000課金文字）に対して余裕がある。
尺は音声と同じ操作から取得でき、`durationMs` として出力へ含められる。
手元に ffmpeg が無い状態でも、コンテナの中で結合できる。

## まだ決まっていないこと

どの声を採用するかは、聞いて決定する。社内でECRとCodeBuildを作成できるかは未確認である。
読みの補正（辞書）は、この試作では適用していない。
