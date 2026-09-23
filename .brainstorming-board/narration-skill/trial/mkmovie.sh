#!/usr/bin/env bash
# 結合：音声の実長で画面を切る。尺は ffprobe で測り、通しも再符号化して繋ぐ。
set -euo pipefail
img="jrottenberg/ffmpeg:7.1-alpine"
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$here/movie"
docker run --rm -v "$here:/w" -w /w --entrypoint sh "$img" -c '
set -e
: > movie/list.txt
python_json() { :; }
for row in $(cat plan.tsv | cut -f1,2 | tr "\t" ":"); do
  id="${row%%:*}"; key="${row##*:}"
  dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "cache/$key.mp3")
  ffmpeg -y -loglevel error -loop 1 -framerate 30 -i "$id.png" -i "cache/$key.mp3" \
    -t "$dur" -c:v libx264 -preset veryfast -pix_fmt yuv420p -r 30 \
    -c:a aac -ar 48000 -ac 2 -vsync cfr "movie/$id.mp4"
  echo "file '"'"'$id.mp4'"'"'" >> movie/list.txt
  echo "$id 音声 ${dur}s"
done
cd movie && ffmpeg -y -loglevel error -f concat -safe 0 -i list.txt \
  -c:v libx264 -preset veryfast -pix_fmt yuv420p -r 30 -c:a aac -ar 48000 -ac 2 lesson.mp4
echo "--- 検査 ---"
ffprobe -v error -select_streams v -show_entries stream=duration -of csv=p=0 lesson.mp4 | sed "s/^/映像 /"
ffprobe -v error -select_streams a -show_entries stream=duration -of csv=p=0 lesson.mp4 | sed "s/^/音声 /"
'
