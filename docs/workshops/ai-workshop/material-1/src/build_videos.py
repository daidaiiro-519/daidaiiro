"""枚ごとの画像と音声を、本ごとの動画1本へ結合する。

音声と尺は narration Skill が作る（narration/narration.out.json）。
ここが保持するのは、枚の並び ・ 画像の置き場 ・ 動画の条件だけである。

  python3 build_videos.py            全9本を組む
  python3 build_videos.py 01-lead    1本だけ組む
"""
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / 'videos'
NARR = HERE / 'narration'
IMAGE = 'jrottenberg/ffmpeg:7.1-alpine'   # 版を固定した公開のイメージを使う
SIZE = '1280x720'
FPS = 30


def ffmpeg(args, workdir):
    """版を固定したイメージで実行する。端末へ導入しない ── 版の差が動画の差になる。"""
    cmd = ['docker', 'run', '--rm', '-v', f'{workdir}:/w', '-w', '/w', IMAGE] + args
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f'ffmpeg が失敗した\n{" ".join(args)}\n{r.stderr[-1500:]}')


def decks():
    """再生の順に、本と、その枚の識別子を返す。"""
    for p in sorted(HERE.glob('lesson-0*.json')):
        d = json.loads(p.read_text())
        stem = f'{d["no"]:02d}-{d["key"]}'
        ids = [f'{d["key"].upper()}-COVER'] + [s['id'] for s in d['slides']]
        yield stem, d['no'], ids


def build(stem, no, ids, dur):
    """枚ごとに画像と音声を1つの区間にし、通しで繋ぐ。"""
    work = OUT / stem
    work.mkdir(parents=True, exist_ok=True)
    parts = []
    for i, sid in enumerate(ids, 1):
        if sid not in dur:
            raise SystemExit(f'音声が無い: {sid} ── narration Skill を先に実行する')
        png = HERE / 'previews' / f'{no:02d}-{i:02d}.png'
        if not png.exists():
            raise SystemExit(f'画像が無い: {png}')
        mp3 = NARR / dur[sid]['audio']
        part = work / f'{i:02d}.mp4'
        # 映像の長さは音声に合わせる。-shortest ではなく音声の長さで切る
        ffmpeg(['-y', '-loop', '1', '-i', str(png.relative_to(HERE)),
                '-i', str(mp3.relative_to(HERE)),
                '-c:v', 'libx264', '-preset', 'medium', '-crf', '20',
                '-pix_fmt', 'yuv420p', '-r', str(FPS), '-s', SIZE,
                '-c:a', 'aac', '-b:a', '128k', '-ar', '48000', '-ac', '2',
                '-t', f'{dur[sid]["durationMs"] / 1000:.3f}',
                str(part.relative_to(HERE))], HERE)
        parts.append(part)
    lst = work / 'parts.txt'
    lst.write_text(''.join(f"file '{p.name}'\n" for p in parts))
    # 通しは再符号化して繋ぐ ── 無変換で繋ぐと、時刻の差が累積する
    ffmpeg(['-y', '-f', 'concat', '-safe', '0', '-i', str(lst.relative_to(HERE)),
            '-c:v', 'libx264', '-preset', 'medium', '-crf', '20', '-pix_fmt', 'yuv420p',
            '-c:a', 'aac', '-b:a', '128k', '-ar', '48000', '-ac', '2',
            str((OUT / f'{stem}.mp4').relative_to(HERE))], HERE)
    total = sum(dur[s]['durationMs'] for s in ids) / 1000
    print(f'{stem}.mp4　{len(ids)}枚　{int(total // 60)}分{int(total % 60):02d}秒')
    return total


if __name__ == '__main__':
    out = NARR / 'narration.out.json'
    if not out.exists():
        raise SystemExit('narration.out.json が無い ── narration Skill を先に実行する')
    dur = {x['id']: x for x in json.loads(out.read_text())['items']}
    OUT.mkdir(exist_ok=True)
    want = sys.argv[1:]
    total = 0
    for stem, no, ids in decks():
        if want and stem not in want:
            continue
        total += build(stem, no, ids, dur)
    print(f'合計　{int(total // 60)}分{int(total % 60):02d}秒')
