"""完成動画の全スライドと音声・字幕・時刻の対応を確認する。"""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = next(p for p in Path(__file__).resolve().parents if (p / 'CLAUDE.md').exists())
SRC = ROOT / 'docs/workshops/ai-workshop/material-1/src'
sys.path.insert(0, str(SRC))
import build_videos as V
import math

OUT = Path(__file__).resolve().parents[1] / 'workspace/video-release-20261025'
OUT.mkdir(exist_ok=True)
media = V.ROOT / 'out/videos/01-lead.mp4'
probe = subprocess.run(['docker','run','--rm','-v',str(V.ROOT)+':/w','-w','/w','--entrypoint','ffprobe',V.IMAGE,'-v','error','-show_format','-show_streams','-of','json','out/videos/01-lead.mp4'],check=True,capture_output=True,text=True)
info = json.loads(probe.stdout)
video = next(x for x in info['streams'] if x['codec_type']=='video')
audio = next(x for x in info['streams'] if x['codec_type']=='audio')
assert (video['width'],video['height'])==(1280,816)
assert video['codec_name']=='h264' and audio['codec_name']=='aac'
timeline = json.loads((V.OUT/'01-lead-timeline.json').read_text())
lesson = json.loads((SRC/'lesson-01-lead.json').read_text())
ids = ['LEAD-COVER']+[s['id'] for s in lesson['slides']]
assert [s['id'] for s in timeline['slides']]==ids and len(ids)==10
seconds = float(info['format']['duration'])
assert abs(seconds-timeline['seconds'])<1
narr = json.loads((V.NARR/'narration.out.json').read_text())
assert [x['id'] for x in narr['items']]==ids
input_ = json.loads((V.NARR/'narration.json').read_text())
assert [x['text'] for x in input_['items']]==[lesson['cover']['narration']]+[s['narration'] for s in lesson['slides']]
frames=[]
for n,s in enumerate(timeline['slides'],1):
    frame=SRC/'work/intro-video-20261025'/f'frame-{n:02d}.png'
    V.ffmpeg(['-y','-ss',str(s['narrationStartSeconds']+2.5),'-i','out/videos/01-lead.mp4','-frames:v','1',str(frame.relative_to(V.ROOT))],V.ROOT)
    stats=SRC/'work/intro-video-20261025'/f'psnr-{n:02d}.txt'
    V.ffmpeg(['-y','-i',str(frame.relative_to(V.ROOT)),'-i',f'src/previews/01-{n:02d}.png','-filter_complex',f'[0:v]crop=1280:720:0:0,format=yuv420p[a];[1:v]format=yuv420p[b];[a][b]psnr=stats_file={stats.relative_to(V.ROOT)}','-frames:v','1','-f','null','-'],V.ROOT)
    rms=math.sqrt(float(re.search(r'mse_avg:([\d.]+)',stats.read_text())[1]))
    assert rms<8,(n,rms)
    saved=OUT/f'frame-{n:02d}.png';saved.write_bytes(frame.read_bytes())
    frames.append(dict(slide=n,id=s['id'],time=s['narrationStartSeconds']+2.5,slide_rms=round(rms,3)))
vtt=(V.OUT/'01-lead.vtt').read_text()
def timestamp(value):
    h,m,s=value.split(':');return int(h)*3600+int(m)*60+float(s)
cues=[(timestamp(a),timestamp(b)) for a,b in re.findall(r'(\d\d:\d\d:\d\d\.\d\d\d) --> (\d\d:\d\d:\d\d\.\d\d\d)',vtt)]
assert cues and all(0<=a<b<=seconds for a,b in cues)
assert all(cues[i][0]>=cues[i-1][1] for i in range(1,len(cues)))
V.ffmpeg(['-v','error','-i','out/videos/01-lead.mp4','-f','null','-'],V.ROOT)
result=dict(date='2026-10-25',slides=10,seconds=seconds,voice=narr['voice'],engine=narr['engine'],video_codec=video['codec_name'],audio_codec=audio['codec_name'],width=1280,height=816,subtitle_cues=len(cues),full_decode='passed',frames=frames,video_sha256=hashlib.sha256(media.read_bytes()).hexdigest(),source_sha256=hashlib.sha256((SRC/'lesson-01-lead.json').read_bytes()).hexdigest(),lexicon_sha256=hashlib.sha256((V.NARR/'lexicon.pls').read_bytes()).hexdigest())
(OUT/'verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
(V.OUT/'01-lead-release.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['slides','seconds','subtitle_cues','full_decode','width','height']},ensure_ascii=False))
