# 共通の view トークン（10キー）と、部品の割り当てから tokens.css を組む。部品は10キーの名前しか指さない
import json,os
H=os.path.dirname(os.path.abspath(__file__))
t=json.load(open(f'{H}/tokens.json')); v=json.load(open(f'{H}/{t["source"]}'))
P=v['palettes'][t['palette']]; KEYS=list(P['light'])
for k,x in t['component'].items():
  if x not in KEYS: raise SystemExit(f'{k} が10キーに無い名前 {x} を指している')
def block(mode): return ''.join(f'--{k}:{P[mode][k]};' for k in KEYS)
comp=''.join(f'--{k}:var(--{x});' for k,x in t['component'].items())
css=(f':root{{{block("light")}{comp}}}\n'
     f'@media (prefers-color-scheme: dark){{:root:not([data-theme="light"]){{{block("dark")}color-scheme:dark}}}}\n'
     f':root[data-theme="dark"]{{{block("dark")}color-scheme:dark}}\n')
open(f'{H}/tokens.css','w').write(css); print(f'色 {len(KEYS)} キー ・ 部品 {len(t["component"])} 件')
