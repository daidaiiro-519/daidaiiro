import re,json,subprocess,os
exec(open('texts.py').read()); exec(open('new.py').read())
CONV='../../../../.claude/skills/design-svg/tool/business_logic/tests/gallery/class/'
BIN='/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/bin/design-svg'
for d in ('templates','figures','resolved'): os.makedirs(d,exist_ok=True)
def subst(svg,texts,aria):
    it=iter(texts)
    svg=re.sub(r'(<text\b[^>]*>)[^<]*(</text>)',lambda m:m.group(1)+next(it)+m.group(2),svg)
    svg=re.sub(r'aria-label="[^"]*"',f'aria-label="{aria}"',svg,count=1)
    return svg
out={}
for k,(src,t,f) in T.items():
    s=open(CONV+src+'.svg').read()
    open(f'templates/{k}.svg','w').write(subst(s,t,'テンプレート：'+k))
    open(f'figures/{k}.svg','w').write(subst(s,f,f[0] if k!='hub' else '病院の予約の文脈の地図'))
for k,(fn,t,f) in NEW.items():
    open(f'templates/{k}.svg','w').write(fn(t)); open(f'figures/{k}.svg','w').write(fn(f))
res={}
for d in ('templates','figures'):
    for fn_ in sorted(os.listdir(d)):
        r=json.loads(subprocess.run([BIN,'resolve',f'{d}/{fn_}','--json'],capture_output=True,text=True).stdout)
        if not r['ok']: print('ERR',d,fn_,r['findings']); continue
        open(f'resolved/{d[0]}-{fn_}','w').write(r['data']['svg'])
        if r['findings'] or (r['data']['min_font_px'] or 99)<8: print(d,fn_,r['data']['min_font_px'],r['findings'])
        res[f'{d}/{fn_}']=(r['data']['min_font_px'],r['data'].get('lines'))
json.dump(res,open('results.json','w'),ensure_ascii=False,indent=0)
print('done',len(res))
