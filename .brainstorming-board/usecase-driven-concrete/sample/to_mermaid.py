# ユースケースの宣言（JSON）から Mermaid の sequenceDiagram を組む。宣言だけで組めるかを確かめるため
import sys,os; sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
from data3 import D
d=D['UC-1']; sc=d['scenario']; h=d['header']
out=['sequenceDiagram',f'  actor {h["primary_actor"]}','  participant システム']+[f'  participant {a}' for a in sc['supporting_actors']]
def msg(s,ind='  '):
  frm=s['actor']; to=s.get('to') or frm
  out.append(f'{ind}{frm}->>{to}: {s["no"]} {s["name"]}')
  if s.get('reply'): out.append(f'{ind}{to}-->>{frm}: {s["reply"]}')
for s in sc['steps']:
  msg(s)
  for x in s['extensions']:
    out.append(f'  opt {x["label"]} {x["condition"].rstrip("：")}（{x["condition_kind"]}）')
    for t in x['steps']: msg(t,'    ')
    out.append(f'    Note over システム: → {x["ending"]}')
    out.append('  end')
print('\n'.join(out))
