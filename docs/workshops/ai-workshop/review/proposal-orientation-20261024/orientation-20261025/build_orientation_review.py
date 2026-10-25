"""作業コピーの入力からオリエンテーションの比較用データを生成する。"""
from pathlib import Path
import hashlib
import html
import importlib.util
import json
import re
import sys
import shutil
import markdown

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'CLAUDE.md').exists())
PUBLIC=ROOT/'docs/workshops/ai-workshop/review/common-intro-20261019/viewer/orientation-review-20261025'
CACHE=ROOT/'.artifact-backup/ai-workshop-orientation-review-20261025/generated'
SRC=HERE/'after/material-1/src'
CACHE.mkdir(parents=True,exist_ok=True)
PUBLIC.mkdir(parents=True,exist_ok=True)
sys.path.insert(0,str(SRC))
spec=importlib.util.spec_from_file_location('orientation_build',SRC/'build_lessons.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
module.SLIDES=CACHE;module.SCRIPTS=CACHE
module.check_ids()
no,stem=module.build(SRC/'lesson-00-start.json');module.retitle(no,stem)
import check_words
assert not check_words.check([CACHE/(stem+'.html'),CACHE/(stem+'-script.md')])
sys.path.insert(0,str(ROOT/'docs/workshops/ai-workshop/proposal/src'))
import visuals
raw=(CACHE/(stem+'.html')).read_text();matches=list(re.finditer(r'href="data:image/[^;]+;base64,[^"]+"',raw))
assert len(matches)==3
for match,book in reversed(list(zip(matches,visuals.BOOKS))):
 raw=raw[:match.start()]+f'href="/workshop/books/{book[0]}.jpg"'+raw[match.end():]
(HERE/'after/slides.html').write_text(raw)
shutil.copy2(HERE/'before/slides.html',PUBLIC/'before.html');shutil.copy2(HERE/'after/slides.html',PUBLIC/'after.html')
before=json.loads((HERE/'current.json').read_text());lesson=json.loads((SRC/'lesson-00-start.json').read_text());ledes=json.loads((SRC/'ledes.json').read_text());after=[dict(before[0])]+[dict(number=n+2,id=s['id'],heading=s['heading'],lede=ledes.get(s['id'],''),narration=s['narration']) for n,s in enumerate(lesson['slides'])]
(HERE/'after/current.json').write_text(json.dumps(after,ensure_ascii=False,indent=2)+'\n')
def note(s):
 return dict(number=s['number'],title=s['heading'],body='<h4>見出し</h4><p>'+html.escape(s['heading'])+'</p><h4>導入文</h4><p>'+html.escape(s['lede'])+'</p><h4>原稿</h4>'+markdown.markdown(s.get('narration') or s['narration_status']))
revision=json.loads((HERE/'revisions.json').read_text()) if (HERE/'revisions.json').exists() else {}
applied=revision.get('applied_slides',[])
payload=dict(id='orientation',name='オリエンテーション',before='orientation-review-20261025/before.html?v='+hashlib.sha256((PUBLIC/'before.html').read_bytes()).hexdigest()[:12],after='orientation-review-20261025/after.html?v='+hashlib.sha256((PUBLIC/'after.html').read_bytes()).hexdigest()[:12],notes=[note(s) for s in after],beforeNotes=[note(s) for s in before],afterNotes=[note(s) for s in after],reasons={str(n):revision.get('reasons',{}).get(str(n),'今回は未修正です。現行の図と原稿を保存し、構成案を検討しています。') for n in range(1,15)},structureAppliedSlides=applied,structureApprovedSlides=revision.get('approved_slides',[]),addedSlides=[])
(HERE/'comparison.json').write_text(json.dumps(payload,ensure_ascii=False,indent=2)+'\n')
if (HERE/'story-plan.md').exists():(PUBLIC/'story-plan.html').write_text('<!doctype html><html lang="ja"><meta charset="utf-8"><link rel="stylesheet" href="../review-navigation.css"><main>'+markdown.markdown((HERE/'story-plan.md').read_text(),extensions=['tables'])+'</main></html>')
print('比較入力を生成:',len(after),'枚。変更反映済み:',applied)
