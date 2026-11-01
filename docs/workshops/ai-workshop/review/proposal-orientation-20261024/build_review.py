"""保存した現行入力と、担当の修正入力から同じ比較HTMLを再生成する。"""
import json
from pathlib import Path
from html import escape
ROOT = Path(__file__).resolve().parent

def read(name):
    return json.loads((ROOT / name).read_text())

data = {}
for deck in ['brief', 'detail', 'orientation']:
    current = read(f'{deck}-current.json')
    sources = read(f'{deck}-source.json') if deck != 'orientation' else None
    orientation = read('before/material-1/src/lesson-00-start.json')['slides'] if deck == 'orientation' else None
    slides = []
    for i, slide in enumerate(current):
        notes = sources[i]['notes'] if sources else (orientation[i-1]['narration'] if i else '未記録（表紙専用の原稿）。現行生成処理は本文最初の原稿を共用する。')
        slides.append(dict(id=f'{deck}-{i+1:02}', heading=slide['heading'], intro=slide['intro'], beforeImage=f'{deck}-{i+1:02}.png', beforeNotes=notes, historicalReasons='未記録', **read('revisions.json').get(f'{deck}-{i+1:02}', {})))
    data[deck] = slides
payload = json.dumps(data, ensure_ascii=False).replace('<', '\\u003c')
report = escape((ROOT/'assessment.md').read_text())
page = '''<!doctype html><html lang="ja"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>企画書・オリエンテーションの確認</title><style>
body{margin:24px auto;padding:0 20px;max-width:1500px;background:#f3efe7;color:#252b29;font-family:sans-serif;line-height:1.7}select,button{font:inherit;max-width:100%;padding:8px}nav{display:flex;gap:12px;flex-wrap:wrap;position:sticky;top:0;background:#f3efe7;padding:12px 0}.columns{display:grid;grid-template-columns:1fr 1fr;gap:20px}article{padding:16px;border:1px solid #6f7a63;background:#fffaf2}img{width:100%;height:auto}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:inherit}#status{font-weight:bold;color:#ad3b1e}a{color:inherit}@media(max-width:700px){.columns{grid-template-columns:1fr}}@media(prefers-color-scheme:dark){body,nav{background:#202620;color:#f3efe7}article{background:#293029}#status{color:#ffb49d}}
</style><h1>企画書・オリエンテーションの現状と構成案</h1><p id="status">構成案の確認前。スライドの変更後は未作成です。</p><p>共通方針 revision 2026-10-24-01。変更前は2026年10月24日に保存した現行版です。</p><p><a href="assessment.md">構成案の原稿</a> ／ <a href="source-manifest.json">版と照合結果</a></p><details><summary>差分整理と構成案を読む</summary><pre>REPORT</pre></details>
<nav><label>資料 <select id="deck"><option value="brief">企画書・要点版</option><option value="detail">企画書・詳細版</option><option value="orientation">オリエンテーション</option></select></label><label>枚 <select id="slide"></select></label></nav><h2 id="heading"></h2><p id="proposal"></p><div class="columns"><article><h3>変更前：保存した現行版</h3><a id="imageLink" target="_blank"><img id="before" alt="変更前の図とスライド"></a><p id="intro"></p><h4>原稿</h4><pre id="beforeNotes"></pre><h4>過去の変更理由</h4><p id="history"></p></article><article><h3>変更後</h3><div id="afterImage"></div><h4 id="afterHeading"></h4><p id="afterIntro"></p><h4>原稿</h4><pre id="afterNotes"></pre><h4>今回の変更理由</h4><pre id="reasons"></pre></article></div><script>
const data=PAYLOAD;const deck=document.querySelector('#deck'),slide=document.querySelector('#slide');const params=new URLSearchParams(location.search);deck.value=data[params.get('deck')]?params.get('deck'):'brief';
function fill(){slide.replaceChildren(...data[deck.value].map((s,i)=>{let o=document.createElement('option');o.value=i;o.textContent=(i+1)+'：'+s.heading;return o}));}
function render(){const s=data[deck.value][Number(slide.value)];for(const [id,value] of Object.entries({heading:s.heading,proposal:s.proposal||'個別の修正案は構成案を参照してください。',intro:s.intro,beforeNotes:s.beforeNotes,history:s.historicalReasons,afterHeading:s.afterHeading||'未作成',afterIntro:s.afterIntro||'',afterNotes:s.afterNotes||'未作成',reasons:s.reasons||'未記録（スライドは未修正）。'}))document.getElementById(id).textContent=value;document.querySelector('#before').src=s.beforeImage;document.querySelector('#imageLink').href=s.beforeImage;const a=document.querySelector('#afterImage');a.replaceChildren();if(s.afterImage){const im=document.createElement('img');im.src=s.afterImage;im.alt='変更後の図とスライド';a.append(im)}else a.textContent='変更後の図は未作成。構成案だけを検討中です。';history.replaceState(null,'','?deck='+deck.value+'&slide='+(Number(slide.value)+1));}
deck.onchange=()=>{fill();render()};slide.onchange=render;fill();slide.value=Math.max(0,Math.min(data[deck.value].length-1,Number(params.get('slide')||9)-1));render();
</script></html>'''
(ROOT/'index.html').write_text(page.replace('REPORT',report).replace('PAYLOAD',payload))
print('Generated', ROOT/'index.html')
