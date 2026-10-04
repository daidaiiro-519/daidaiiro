"""枚ごとの画像と音声を、本ごとの動画1本へ結合する。

音声と尺は slide-deck-speech Skill が作る（narration/narration.out.json）。
ここが保持するのは、枚の並び ・ 画像の置き場 ・ 動画の条件だけである。

  python3 build_videos.py            全9本を組む
  python3 build_videos.py 01-lead    1本だけ組む

字幕は、音声を合成したときの文の時刻（marks）から WebVTT を組み、out/videos/<本>.vtt に置く。
動画には切り替え式の字幕トラック（mov_text）として重ねる ── 映像へ焼き込まない。
"""
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
# 完成した動画は out/videos/ へ、枚ごとの区間は work/ へ置く ── 区間は途中の生成物であり、追跡しない
ROOT = HERE.parent
OUT = ROOT / 'out' / 'videos'
WORK = HERE / 'work' / 'videos'
NARR = HERE / 'narration'
IMAGE = 'jrottenberg/ffmpeg:7.1-alpine'   # 版を固定した公開のイメージを使う
SIZE = '1280x720'
LINE = 22   # 字幕の1行の文字数の目安。2行を超える文は、読点で2つの字幕に分ける
FPS = 30


def ffmpeg(args, workdir):
    """版を固定したイメージで実行する。端末へ導入しない ── 版の差が動画の差になる。"""
    cmd = ['docker', 'run', '--rm', '-v', f'{workdir}:/w', '-w', '/w', IMAGE] + args
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f'ffmpeg が失敗した\n{" ".join(args)}\n{r.stderr[-1500:]}')


def _ts(ms):
    ms = max(0, int(ms))
    return f'{ms // 3600000:02d}:{ms // 60000 % 60:02d}:{ms // 1000 % 60:02d}.{ms % 1000:03d}'


def _cuts(text):
    """改行してよい位置。読点の後と、ひらがなから漢字 ・ カタカナ ・ 英数へ移る位置（語の切れ目の近似）。"""
    hira = lambda c: '\u3041' <= c <= '\u309f'
    out = []
    for i in range(1, len(text)):
        p, c = text[i - 1], text[i]
        if p in '、。' or (hira(p) and not hira(c) and c not in '、。」』）'):
            out.append(i)
    return out


def _wrap(text):
    """1行がおよそ LINE 字に収まるよう、2行のうち長いほうが最も短くなる位置で改行する。"""
    if len(text) <= LINE:
        return [text]
    cuts = _cuts(text) or [len(text) // 2]
    # 読点の後を優先し、行の長さの差が大きくならない範囲で選ぶ
    best = min(cuts, key=lambda i: (max(i, len(text) - i) + (0 if text[i - 1] == '、' else 3)))
    return [text[:best], text[best:]]


def _split(text, start, end):
    """2行に収まらない文は、読点か語の切れ目で分け、時間を文字数で按分する。"""
    if len(text) <= LINE * 2:
        return [(text, start, end)]
    a, b = _wrap(text)
    mid = start + (end - start) * len(a) // len(text)
    return _split(a, start, mid) + _split(b, mid, end)


def subtitles(ids, dur):
    """枚ごとの文の時刻を、通しの時刻へずらして WebVTT にする。"""
    cues, offset = [], 0
    for sid in ids:
        item = dur[sid]
        marks = [json.loads(l) for l in (NARR / item['marks']).read_text().splitlines() if l.strip()]
        sents = [m for m in marks if m['type'] == 'sentence']
        for k, m in enumerate(sents):
            end = sents[k + 1]['time'] if k + 1 < len(sents) else item['durationMs']
            for text, a, b in _split(m['value'], m['time'], end - 80):
                cues.append((offset + a, offset + b, '\n'.join(_wrap(text))))
        offset += item['durationMs']
    body = '\n'.join(f'{n}\n{_ts(a)} --> {_ts(b)}\n{t}\n' for n, (a, b, t) in enumerate(cues, 1))
    return 'WEBVTT\n\n' + body


def decks():
    """再生の順に、本と、その枚の識別子を返す。"""
    for p in sorted(HERE.glob('lesson-0*.json')):
        d = json.loads(p.read_text())
        stem = f'{d["no"]:02d}-{d["key"]}'
        ids = [f'{d["key"].upper()}-COVER'] + [s['id'] for s in d['slides']]
        yield stem, d['no'], ids


def build(stem, no, ids, dur):
    """枚ごとに画像と音声を1つの区間にし、通しで繋ぐ。"""
    work = WORK / stem
    work.mkdir(parents=True, exist_ok=True)
    parts = []
    for i, sid in enumerate(ids, 1):
        if sid not in dur:
            raise SystemExit(f'音声が無い: {sid} ── slide-deck-speech Skill を先に実行する')
        png = HERE / 'previews' / f'{no:02d}-{i:02d}.png'
        if not png.exists():
            raise SystemExit(f'画像が無い: {png}')
        mp3 = NARR / dur[sid]['audio']
        part = work / f'{i:02d}.mp4'
        # 映像の長さは音声に合わせる。-shortest ではなく音声の長さで切る
        ffmpeg(['-y', '-loop', '1', '-i', str(png.relative_to(ROOT)),
                '-i', str(mp3.relative_to(ROOT)),
                '-c:v', 'libx264', '-preset', 'medium', '-crf', '20',
                '-pix_fmt', 'yuv420p', '-r', str(FPS), '-s', SIZE,
                '-c:a', 'aac', '-b:a', '128k', '-ar', '48000', '-ac', '2',
                '-t', f'{dur[sid]["durationMs"] / 1000:.3f}',
                str(part.relative_to(ROOT))], ROOT)
        parts.append(part)
    lst = work / 'parts.txt'
    lst.write_text(''.join(f"file '{p.name}'\n" for p in parts))
    vtt = OUT / f'{stem}.vtt'
    vtt.write_text(subtitles(ids, dur))
    # 通しは再符号化して繋ぐ ── 無変換で繋ぐと、時刻の差が累積する。字幕は切り替え式のトラックとして重ねる
    ffmpeg(['-y', '-f', 'concat', '-safe', '0', '-i', str(lst.relative_to(ROOT)), '-i', str(vtt.relative_to(ROOT)),
            '-map', '0:v', '-map', '0:a', '-map', '1:s',
            '-c:v', 'libx264', '-preset', 'medium', '-crf', '20', '-pix_fmt', 'yuv420p',
            '-c:a', 'aac', '-b:a', '128k', '-ar', '48000', '-ac', '2',
            '-c:s', 'mov_text', '-metadata:s:s:0', 'language=jpn',
            str((OUT / f'{stem}.mp4').relative_to(ROOT))], ROOT)
    total = sum(dur[s]['durationMs'] for s in ids) / 1000
    print(f'{stem}.mp4　{len(ids)}枚　{int(total // 60)}分{int(total % 60):02d}秒')
    return total


if __name__ == '__main__':
    out = NARR / 'narration.out.json'
    if not out.exists():
        raise SystemExit('narration.out.json が無い ── slide-deck-speech Skill を先に実行する')
    dur = {x['id']: x for x in json.loads(out.read_text())['items']}
    OUT.mkdir(parents=True, exist_ok=True)
    want = sys.argv[1:]
    total = 0
    for stem, no, ids in decks():
        if want and stem not in want:
            continue
        total += build(stem, no, ids, dur)
    print(f'合計　{int(total // 60)}分{int(total % 60):02d}秒')
