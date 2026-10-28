"""企画書の動画の、枚と音声・字幕・時刻の対応を確認する。作業コピー after/proposal を検査する。"""
from pathlib import Path
import hashlib, json, math, re, subprocess, sys

ROOT = next(p for p in Path(__file__).resolve().parents if (p / 'CLAUDE.md').exists())
sys.path.insert(0, str(ROOT / 'docs/workshops/ai-workshop/material-1/src'))
import build_videos as V

HERE = Path(__file__).resolve().parent
BASE = HERE.parent / 'after/proposal'
SRC = BASE / 'src'
STEM = 'director-deck'
WORK = SRC / 'work/verify'
WORK.mkdir(parents=True, exist_ok=True)


def probe(rel, *args):
    r = subprocess.run(['docker', 'run', '--rm', '-v', f'{BASE}:/w', '-w', '/w', '--entrypoint', 'ffprobe', V.IMAGE,
                        '-v', 'error', *args, rel], check=True, capture_output=True, text=True)
    return r.stdout


info = json.loads(probe(f'out/videos/{STEM}.mp4', '-show_format', '-show_streams', '-of', 'json'))
video = next(x for x in info['streams'] if x['codec_type'] == 'video')
audio = next(x for x in info['streams'] if x['codec_type'] == 'audio')
assert (video['width'], video['height']) == (1280, 816)
assert video['codec_name'] == 'h264' and audio['codec_name'] == 'aac'
seconds = float(info['format']['duration'])
timeline = json.loads((BASE / f'out/videos/{STEM}-timeline.json').read_text())
narr_in = json.loads((SRC / 'narration/narration.json').read_text())
narr = json.loads((SRC / 'narration/narration.out.json').read_text())
ids = [x['id'] for x in narr_in['items']]
assert len(ids) == 29 and ids[-1] == 'まとめ'
assert [s['id'] for s in timeline['slides']] == ids == [x['id'] for x in narr['items']]
assert abs(seconds - timeline['seconds']) < 1
# 原稿は、正本の発表原稿の本編と1文字も変わらない
notes = (BASE / 'out/speaker-notes.md').read_text()
secs = re.split(r'\n## (\d\d)　[^\n]*\n\n', notes)[1:]
body = [secs[i + 1].strip() for i in range(0, len(secs), 2)][:29]
assert [x['text'] for x in narr_in['items']] == body
frames = []
for n, s in enumerate(timeline['slides'], 1):
    frame = WORK / f'frame-{n:02d}.png'
    stats = WORK / f'psnr-{n:02d}.txt'
    V.ffmpeg(['-y', '-ss', str(s['narrationStartSeconds'] + 2.5), '-i', f'out/videos/{STEM}.mp4', '-frames:v', '1',
              str(frame.relative_to(BASE))], BASE)
    V.ffmpeg(['-y', '-i', str(frame.relative_to(BASE)), '-i', f'src/previews/{n:02d}.png', '-filter_complex',
              f'[0:v]crop=1280:720:0:0,format=yuv420p[a];[1:v]format=yuv420p[b];[a][b]psnr=stats_file={stats.relative_to(BASE)}',
              '-frames:v', '1', '-f', 'null', '-'], BASE)
    rms = math.sqrt(float(re.search(r'mse_avg:([\d.]+)', stats.read_text())[1]))
    assert rms < 8, (n, rms)
    (HERE / f'frame-{n:02d}.png').write_bytes(frame.read_bytes())
    frames.append(dict(slide=n, id=s['id'], time=round(s['narrationStartSeconds'] + 2.5, 3), slide_rms=round(rms, 3)))
vtt = (BASE / f'out/videos/{STEM}.vtt').read_text()
ts = lambda v: (lambda h, m, s: int(h) * 3600 + int(m) * 60 + float(s))(*v.split(':'))
cues = [(ts(a), ts(b)) for a, b in re.findall(r'(\d\d:\d\d:\d\d\.\d\d\d) --> (\d\d:\d\d:\d\d\.\d\d\d)', vtt)]
assert cues and all(0 <= a < b <= seconds for a, b in cues)
assert all(cues[i][0] >= cues[i - 1][1] for i in range(1, len(cues)))
lines = [l for l in vtt.split('\n\n')[1:] if l.strip()]
assert all(len(c.strip().split('\n')) <= 4 for c in lines)   # 番号 ・ 時刻 ・ 2行まで
V.ffmpeg(['-v', 'error', '-i', f'out/videos/{STEM}.mp4', '-f', 'null', '-'], BASE)
mp3_seconds = float(probe(f'out/audio/{STEM}.mp3', '-show_entries', 'format=duration', '-of', 'csv=p=0').strip())
assert abs(mp3_seconds - seconds) < 0.5
assert (BASE / f'out/videos/posters/{STEM}.png').read_bytes() == (SRC / 'previews/01.png').read_bytes()
lex = ROOT / 'docs/workshops/ai-workshop/review/proposal-orientation-20261024/proposal-20261026/after/material-1/src/narration/lexicon.pls'
result = dict(date='2026-10-28', slides=29, seconds=seconds, audio_seconds=mp3_seconds, voice=narr['voice'], engine=narr['engine'],
              video_codec=video['codec_name'], audio_codec=audio['codec_name'], width=1280, height=816,
              subtitle_cues=len(cues), full_decode='passed', frames=frames,
              video_sha256=hashlib.sha256((BASE / f'out/videos/{STEM}.mp4').read_bytes()).hexdigest(),
              notes_sha256=hashlib.sha256((BASE / 'out/speaker-notes.md').read_bytes()).hexdigest(),
              lexicon_sha256=hashlib.sha256(lex.read_bytes()).hexdigest())
(HERE / 'verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
(BASE / f'out/videos/{STEM}-release.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['slides', 'seconds', 'audio_seconds', 'subtitle_cues', 'full_decode']}, ensure_ascii=False),
      'max_rms', max(f['slide_rms'] for f in frames))
