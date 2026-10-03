# 共通の view トークン（10キー）から tokens.css を組む。CSS と図は、この10キーを直接指す
import json,os
H=os.path.dirname(os.path.abspath(__file__))
t=json.load(open(f'{H}/tokens.json')); v=json.load(open(f'{H}/{t["source"]}'))
P=v['palettes'][t['palette']]; KEYS=list(P['light'])
def block(mode): return ''.join(f'--{k}:{P[mode][k]};' for k in KEYS)
css=(f':root{{{block("light")}}}\n'
     f'@media (prefers-color-scheme: dark){{:root:not([data-theme="light"]){{{block("dark")}color-scheme:dark}}}}\n'
     f':root[data-theme="dark"]{{{block("dark")}color-scheme:dark}}\n')
open(f'{H}/tokens.css','w').write(css); print(f'色 {len(KEYS)} キー')
