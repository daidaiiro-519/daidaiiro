"""企画書の詳細版の本編（表紙〜まとめ）を、音声と字幕付きの動画1本にする。

  python3 build_video.py narration                       原稿から narration/narration.json を組む
  python3 build_video.py previews --browser <ブラウザ>    dist/proposal/detail/slides.html の本編を previews/NN.png にする
  python3 build_video.py video                           画像と音声を結合し、dist/proposal/detail/ へ置く

音声と尺は slide-deck-speech Skill が作る（narration/narration.out.json）。
枚の識別子は、build_deck.py の枚のラベルをそのまま使う。
字幕の組み方と結合の条件（FFmpeg のイメージ ・ 画面の大きさ ・ 字幕の帯 ・ 書体）は、
教材の動画（src/lessons/build_videos.py）を読み込んで同じものを使う ── 2か所に書かない。
付録は動画にしない ── 質疑への応答用の枚である。
"""
import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]   # FFmpeg に渡す場所。src/ と dist/ の両方を含む
NARR = HERE / 'narration'
PREVIEWS = HERE / 'previews'
OUT = ROOT / 'dist' / 'proposal' / 'detail'
WORK = HERE / 'work' / 'video'
HEAD = {'voice': 'Takumi', 'engine': 'neural', 'format': 'mp3', 'cache': 'cache-proposal-v1', 'lexicons': ['aiworkshop']}  # 辞書は src/shared/lexicon.pls
# 枚の前後の間。表紙の前には置かない
PACING = {'transitionMs': 250, 'beforeMs': 250, 'afterMs': 350}


def videos():
    sys.path.insert(0, str(HERE.parent / 'lessons'))
    import build_videos as V
    V.NARR = NARR
    return V


def main_slides():
    import build_deck as B
    deck = B._deck(B.ORDER)
    return deck[:B.ORDER.index(B.APPENDIX_INDEX)]


def narration():
    items = [{'id': s['label'], 'text': s['notes']} for s in main_slides()]
    if len({i['id'] for i in items}) != len(items):
        raise SystemExit('枚のラベルが重なっている')
    NARR.mkdir(exist_ok=True)
    (NARR / 'narration.json').write_text(json.dumps(dict(HEAD, items=items), ensure_ascii=False, indent=1) + '\n')
    print(f'narration.json　{len(items)}枚')


def previews(browser):
    count = len(main_slides())
    PREVIEWS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        # 枚が切り替わるときの動きを止める ── 動きの途中で撮ると、枚が半透明に写る。
        # めくる操作の案内は動画では使えないので消す。枚番号は PDF と同じ番号のまま残す
        page = Path(tmp) / 'slides.html'
        page.write_text((OUT / 'slides.html').read_text()
                        .replace('</style>', '.slide{animation:none!important}.hint{visibility:hidden}</style>', 1))
        for i in range(1, count + 1):
            shot = PREVIEWS / f'{i:02d}.png'
            shot.unlink(missing_ok=True)
            subprocess.run([browser, '--headless=new', '--no-sandbox', '--hide-scrollbars', '--window-size=1280,720',
                            f'--screenshot={shot}', f'file://{page}#{i}'], capture_output=True)
            if not shot.exists():
                raise SystemExit(f'画像を作れなかった: {shot}')
    print(f'previews　{count}枚')


def video():
    V = videos()
    out = NARR / 'narration.out.json'
    if not out.exists():
        raise SystemExit('narration.out.json が無い ── slide-deck-speech Skill を先に実行する')
    dur = {x['id']: x for x in json.loads(out.read_text())['items']}
    ids = [s['label'] for s in main_slides()]
    pacing = {'transitionMs': PACING['transitionMs'],
              'slides': {sid: {'beforeMs': 0 if n == 1 else PACING['beforeMs'], 'afterMs': PACING['afterMs']}
                         for n, sid in enumerate(ids, 1)}}
    for d in (WORK, OUT):
        d.mkdir(parents=True, exist_ok=True)
    from theme import T
    color = '0x' + T['ground'].lstrip('#')
    fade = pacing['transitionMs'] / 2000
    parts, timeline, offset = [], [], 0
    for i, sid in enumerate(ids, 1):
        if sid not in dur:
            raise SystemExit(f'音声が無い: {sid} ── slide-deck-speech Skill を先に実行する')
        png = PREVIEWS / f'{i:02d}.png'
        if not png.exists():
            raise SystemExit(f'画像が無い: {png}')
        gap = pacing['slides'][sid]
        duration = dur[sid]['durationMs'] + gap['beforeMs'] + gap['afterMs']
        # 無音は音声の前後へ加え、発話そのものは切らない
        filters = ['-af', f"adelay={gap['beforeMs']}:all=1,apad=pad_dur={gap['afterMs'] / 1000:.3f}"]
        fades = ([f'fade=t=in:st=0:d={fade}:color={color}'] if i > 1 else []) + \
                ([f'fade=t=out:st={duration / 1000 - fade}:d={fade}:color={color}'] if i < len(ids) else [])
        filters += ['-vf', ','.join(fades)]
        timeline.append(dict(number=i, id=sid, startSeconds=offset / 1000,
                             narrationStartSeconds=(offset + gap['beforeMs']) / 1000,
                             durationSeconds=duration / 1000,
                             voiceDurationSeconds=dur[sid]['durationMs'] / 1000))
        offset += duration
        part = WORK / f'{i:02d}.mp4'
        # 映像の長さは、音声と明示した間の合計で固定する
        V.ffmpeg(['-y', '-loop', '1', '-i', str(png.relative_to(ROOT)),
                  '-i', str((NARR / dur[sid]['audio']).relative_to(ROOT))] + filters + [
                  '-c:v', 'libx264', '-preset', 'medium', '-crf', '20',
                  '-pix_fmt', 'yuv420p', '-r', str(V.FPS), '-s', V.SIZE,
                  '-c:a', 'aac', '-b:a', '128k', '-ar', '48000', '-ac', '2',
                  '-t', f'{duration / 1000:.3f}', str(part.relative_to(ROOT))], ROOT)
        parts.append(part)
    lst = WORK / 'parts.txt'
    lst.write_text(''.join(f"file '{p.name}'\n" for p in parts))
    vtt = OUT / 'video.vtt'
    vtt.write_text(V.subtitles(ids, dur, pacing))
    w, h = map(int, V.SIZE.split('x'))
    style = ('FontName=Noto Sans JP,FontSize=11,PrimaryColour=&H00FFFFFF,'
             'BorderStyle=1,Outline=0,Shadow=0,MarginV=4,Alignment=2')
    vf = (f"pad={w}:{h + V.BAND}:0:0:color=0x{T['ink'].lstrip('#')},"
          f"subtitles={vtt.relative_to(ROOT)}:original_size={w}x{h + V.BAND}:fontsdir=/fonts:force_style='{style}'")
    # 通しは再符号化して繋ぐ ── 無変換で繋ぐと、時刻の差が累積する。字幕は帯へ焼き込む
    mp4 = OUT / 'video.mp4'
    V.ffmpeg(['-y', '-f', 'concat', '-safe', '0', '-i', str(lst.relative_to(ROOT)), '-vf', vf,
              '-c:v', 'libx264', '-preset', 'medium', '-crf', '20', '-pix_fmt', 'yuv420p',
              '-c:a', 'aac', '-b:a', '128k', '-ar', '48000', '-ac', '2',
              str(mp4.relative_to(ROOT))], ROOT, fonts=True)
    V.ffmpeg(['-y', '-i', str(mp4.relative_to(ROOT)), '-vn', '-c:a', 'libmp3lame', '-b:a', '128k', '-ar', '48000',
              str((OUT / 'audio.mp3').relative_to(ROOT))], ROOT)
    (OUT / 'poster.png').write_bytes((PREVIEWS / '01.png').read_bytes())
    total = offset / 1000
    (OUT / 'video-timeline.json').write_text(json.dumps(dict(seconds=total, slides=timeline), ensure_ascii=False, indent=2) + '\n')
    print(f'video.mp4　{len(ids)}枚　{int(total // 60)}分{int(total % 60):02d}秒')


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('step', choices=['narration', 'previews', 'video'])
    ap.add_argument('--browser')
    a = ap.parse_args()
    sys.path.insert(0, str(HERE))
    if a.step == 'narration':
        narration()
    elif a.step == 'previews':
        if not a.browser:
            raise SystemExit('--browser でブラウザの場所を渡す')
        previews(a.browser)
    else:
        video()
