# 宣言どうしのずれ（論点5を論点7で改めた形）。構造で検査 ・ 対応の欄で検査 ・ 上流の変更 ・ 人のレビューに分ける。
# 対応は設計の側（アプリケーション層の操作）にだけ書かれ、ユースケースの側への向きは道具が計算する。自由文は読まない
import gen6 as g
from drift6 import fp,up
D=g.D
import json,os
APPROVED=os.path.join(os.path.dirname(os.path.abspath(__file__)),'approved-record.json')
def use(state):
  global D
  D=state; g.use(state)
KIND={"aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス","application_operation":"アプリケーション層の操作"}
CHANGE_STEPS=('内部の状態変化',)
EXT_FROM_DESIGN=('業務ルールの拒否','支援アクターの失敗')
def links():
  """設計の側が要求の側を指す対応（参照元は設計の側）"""
  out=[]
  for ak,A in [(k,v) for k,v in D.items() if v['kind']=='application_operation']:
    for u in A['satisfies']: out.append((ak,'satisfies',u))
    for s in A['steps']: out.append((f'{ak}:{s["step"]}','calls',s['calls']))
    for x in A['extensions']:
      for r in x['raised_by']: out.append((f'{ak}:{x["extension"]}','raised_by',r))
    for p in A['preconditions']:
      for r in p['prevents']: out.append((f'{ak}:{p["precondition"]}','prevents',r))
    for q in A['guarantees']:
      for r in q['established_by']: out.append((f'{ak}:{q["guarantee"]}','established_by',r))
  for k,v in D.items():
    if v['kind']=='context':
      for b in v.get('business_rules',[]):
        if b.get('implements'): out.append((f'{k}.{b["id"]}','implements',b['implements']))
  return out
def record(): return {f'{a}→{r}':fp(up(r)) for a,f,r in links() if f not in ('satisfies','calls')}
def checks(rec=None):
  """rec は承認した時点の記録。渡さなければ、どの対応もまだ承認していないものとして扱う（自分自身と比べない）"""
  R=[]
  def put(cat,name,src,dst,text,st): R.append({"category":cat,"check":name,"from":src,"to":dst,"text":text,"status":st})
  T=g.terms()
  def kind_is(t,k,src,what):
    x=T.get(t)
    if x is None: put('structure','名前が用語を指す',src,t,f'{what}が用語集の語を指していない','ずれ')
    elif k and x['kind'] not in (k if isinstance(k,tuple) else (k,)): put('structure','名前が用語を指す',src,t,f'{what}「{x["word"]}」の語の種類が {x["kind"]}','ずれ')
  rejects={}
  # ── 設計の側の構造（集約 ・ ドメインサービス ・ サブドメイン。論点5と同じ）
  for k,d in D.items():
    if d['kind'] in KIND: kind_is(d['header']['name'],KIND[d['kind']],k,'名前')
    if d['kind']=='aggregate':
      for s in d['structure']['state']: kind_is(s['name'],None,f'{k}.{s["id"]}','状態の名前')
      for e in d['structure']['entities']:
        for s in e['state']: kind_is(s['name'],None,f'{k}.{e["id"]}.{s["id"]}','状態の名前')
      posts=[]
      for c in d['commands']:
        kind_is(c['name'],'コマンド',f'{k}.{c["id"]}','コマンドの名前')
        for p in c['business_rules']:
          kind_is(p['reject'],'拒否の理由',f'{k}.{c["id"]}.{p["id"]}','拒否の理由')
          rejects.setdefault(p['reject'],set()).add(k)
          if g.reject_example(d,c,p) is None: put('structure','拒否の例が組める',f'{k}.{c["id"]}.{p["id"]}','','条件から拒否の例を組めず、宣言にも例が無い','ずれ')
        for e in c['emits']: kind_is(e['name'],'業務イベント',f'{k}.{c["id"]}.{e["id"]}','業務イベントの名前')
        posts.append((c['id'],str(c['state_changes'])))
      for i,(a,pa) in enumerate(posts):
        for b,pb in posts[i+1:]:
          if pa==pb: put('structure','別のコマンドの状態の変更が同じ',f'{k}.{a}',f'{k}.{b}','名前の違うコマンドが同じ状態の変更を持つ','ずれ')
      for inv in d['invariants']:
        if not inv.get('via'): put('structure','違反する例に至る操作がある',f'{k}.{inv["id"]}','','違反する状態に至る操作（via）が無い','ずれ')
    if d['kind']=='subdomain':
      c=d['classification']; dc=g.derived_category(c)
      put('structure','カテゴリーが判断基準の答えと合う',k,'',f'判断基準の答えから導いたカテゴリーは {dc}、宣言は {c["category"]}','合格' if dc==c['category'] else 'ずれ')
      m=g.impl_method(d['business_logic'])
      bad=(c['category']=='中核' and 'ドメインモデル' not in m) or (c['category']!='中核' and 'ドメインモデル' in m)
      put('review' if bad else 'structure','カテゴリーと実装方法',k,'',f'{c["category"]}で、業務ロジックの性質から導いた実装方法は{m}'+('。判断を再検討する機会' if bad else ''),'レビュー' if bad else '合格')
  for r,owners in rejects.items():
    if len(owners)>1: put('structure','拒否の理由を1つの集約だけが使う','・'.join(sorted(owners)),r,f'同じ拒否の理由を {"・".join(sorted(owners))} が使っている','ずれ')
  # ── 用語集：1つの文脈が同じ語の2つの意味を指さない
  for k,v in D.items():
    if v['kind']!='context': continue
    by={}
    for u in v['uses']: by.setdefault(u['term'],set()).add(u['meaning'])
    two=[t for t,ms in by.items() if len(ms)>1]
    for t in two: put('structure','1つの文脈が1つの語に1つの意味',k,t,f'「{g.word(t)}」の意味を2つ使っている。語を2つに分ける','ずれ')
    if not two: put('structure','1つの文脈が1つの語に1つの意味',k,'',f'{len(by)}語とも、使う意味は1つ','合格')
    unk=[u['term'] for u in v['uses'] if u['term'] not in T]
    for t in unk: put('structure','使う意味が用語集にある',k,t,'用語集に無い語を指している','ずれ')
  # ── その他の要求：1件のビジネスルールを2つの文脈で実装しない
  impl={}
  for k,v in D.items():
    if v['kind']=='context':
      for b in v.get('business_rules',[]):
        if b.get('implements'): impl.setdefault(b['implements'],[]).append(k)
  for k,v in D.items():
    if v['kind']!='other_requirements': continue
    for b in v['business_rules']:
      r=f'{k}.{b["id"]}'; who=impl.get(r,[])
      if len(who)>1: put('structure','ビジネスルールを実装する文脈は1つ',r,'・'.join(who),f'「{b["name"]}」を{len(who)}つの文脈で実装している','ずれ')
      elif who: put('structure','ビジネスルールを実装する文脈は1つ',r,who[0],f'「{b["name"]}」は {g.dname(who[0])} が実装する','合格')
      else: put('link','ビジネスルールを実装する文脈は1つ',r,'',f'「{b["name"]}」を実装する文脈が無い','対応の欠け')
  # ── ユースケースの側（内部を指していないか ・ 語が用語集にあるか）
  APPS=g.apps()
  for k,d in D.items():
    if d['kind']!='use_case': continue
    nums=g.number(d)
    bad=[x for x in ('context',) if x in d['header']['scope']]
    put('structure','ユースケースが内部を指さない',k,'',('スコープが区切られた文脈を指している' if bad else 'スコープはプロダクトだけを指し、手順に内部の ID が無い'),'ずれ' if bad else '合格')
    for s in g.all_steps(d):
      src=f'{k}.{s["id"]}'
      for f in ('invokes','via','handles','ensures'):
        if s.get(f): put('structure','ユースケースが内部を指さない',src,s[f],f'手順{nums[s["id"]]}が {f} で内部を指している','ずれ')
      if s['kind']=='相互作用':
        for t in s['data']: kind_is(t,('情報の別名','値オブジェクト','識別子'),src,'渡す情報')
        kind_is(s['verb'],'動作',src,'述語')
      if s['kind']=='内部の状態変化':
        kind_is(s['object'],None,src,'対象'); kind_is(s['verb'],None,src,'述語')
      if s['kind']=='妥当性確認':
        f=[r for x in s.get('extensions',[]) for r in x.get('fails',[])]
        for r in s['checks']:
          put('link','確かめた条件の失敗を扱う',src,r,'拡張が扱っている' if r in f else f'手順{nums[s["id"]]}で確かめる条件が成り立たないときの拡張が無い','合格' if r in f else '対応の欠け')
    for x in d['guarantees']['success']+d['guarantees']['minimal']+d['preconditions']:
      for t in cond_targets(x['condition']):
        if g.resolve_term_path(t) is None and t.count('.')>0:
          put('structure','条件の語が設計の側で解ける',f'{k}.{x["id"]}',t,f'「{g.qname(t)}」に当たる集約の状態が無い','ずれ')
  # ── ユースケース → 設計（どの操作にも満たされていないものは無いか）
  for k,d in D.items():
    if d['kind']!='use_case': continue
    nums=g.number(d); mine=g.apps_of(k)
    if d['header']['level']=='要約':
      put('link','ユースケースを満たす設計',k,'','要約のユースケース。手順は下のユースケースの呼び出しなので、操作は持たない','合格'); continue
    put('link','ユースケースを満たす設計',k,'・'.join(a['id'] for a in mine),('満たす操作がある' if mine else f'「{d["header"]["name"]}」を満たすアプリケーション層の操作が無い'),'合格' if mine else '対応の欠け')
    mapped_steps={s['step'] for a in mine for s in a['steps']}
    mapped_ext={x['extension'] for a in mine for x in a['extensions']}
    for s in g.all_steps(d):
      if s['kind'] in CHANGE_STEPS:
        r=f'{k}.{s["id"]}'
        put('link','内部の状態変化を実現する操作',r,'','実現する操作がある' if r in mapped_steps else f'手順{nums[s["id"]]}「{g.step_short(s)}」を実現する操作が無い','合格' if r in mapped_steps else '対応の欠け')
    for s in d['scenario']['steps']:
      for x in s['extensions']:
        if x['condition_kind'] not in EXT_FROM_DESIGN: continue
        r=f'{k}.{x["id"]}'
        put('link','拡張を起こす拒否 ・ 失敗',r,'','対応する拒否か失敗がある' if r in mapped_ext else f'拡張{nums[x["id"]]}（{g.ext_text(x).rstrip("：")}）が、どの拒否 ・ 失敗で起きるかが無い','合格' if r in mapped_ext else '対応の欠け')
    gm={q['guarantee']:q['established_by'] for a in mine for q in a['guarantees']}
    for x in d['guarantees']['success']:
      r=f'{k}.{x["id"]}'; est=gm.get(r,[])
      if not est: put('link','成功時保証を成り立たせる状態の変更',r,'',f'「{x["name"]}」を成り立たせる状態の変更が無い','対応の欠け'); continue
      want=g.resolve_term_path(x['condition']['target'])
      got=[f'{c.split(".")[0]}.{g.item(c)[0]["condition"]["target"]}' for c in est]
      ok=want in got
      put('link','成功時保証を成り立たせる状態の変更',r,'・'.join(est),('状態の変更が、保証の語と同じ状態を変える' if ok else f'「{x["name"]}」の語（{g.qname(x["condition"]["target"])}）と、指した状態の変更の状態が違う'),'合格' if ok else 'ずれ')
    kept={m for t in g.all_steps(d) for m in t.get('keeps',[])}
    for m in d['guarantees']['minimal']:
      put('link','最低保証を守る手順',f'{k}.{m["id"]}','','守る手順がある' if m['id'] in kept else f'「{m["name"]}」を守る手順が無い','合格' if m['id'] in kept else '対応の欠け')
  # ── 設計 → ユースケース（呼ぶものの拒否 ・ 失敗が、拡張か事前条件に対応しているか）
  for A in APPS:
    ak=A['id']
    for u in A['satisfies']:
      if u not in D: put('structure','満たすユースケースがある',ak,u,'指したユースケースが無い','ずれ')
    covered={r for x in A['extensions'] for r in x['raised_by']}|{r for p in A['preconditions'] for r in p['prevents']}
    for x in A['extensions']:
      u,eid=x['extension'].split('.',1); ext=g.uc_part(D[u],eid)
      for r in x['raised_by']:
        it,_,_=g.item(r); word_=it.get('reject') or it.get('name')
        ok=word_ in ext.get('reasons',[])
        put('link','拡張の条件と拒否の語が合う',f'{ak}:{x["extension"]}',r,('拡張の条件が、拒否 ・ 失敗と同じ語で書かれている' if ok else f'拡張の条件の語と「{g.word(word_)}」が合わない'),'合格' if ok else 'ずれ')
    touched=set()
    for s in A['steps']:
      c=s['calls']; head=c.split('.')[0]
      if D[head]['kind']=='aggregate':
        cm,_,_=g.item(c)
        if cm['state_changes']: touched.add(head)
        for p in cm['business_rules']:
          r=f'{c}.{p["id"]}'
          put('link','呼ぶコマンドの拒否を扱う',f'{ak}:{s["step"]}',r,('拡張か事前条件に対応している' if r in covered else f'{g.cmd_label(c)}ときの拒否「{g.word(p["reject"])}」が、拡張にも事前条件にも対応していない'),'合格' if r in covered else '対応の欠け')
      if D[head]['kind']=='context' and '.X-' in c:
        op,_,_=g.item(c)
        for f in op['failures']:
          r=f'{c}.{f["id"]}'
          put('link','外部の操作の失敗を扱う',f'{ak}:{s["step"]}',r,('拡張に対応している' if r in covered else f'外部の操作の失敗「{g.word(f["name"])}」が、どの拡張にも対応していない'),'合格' if r in covered else '対応の欠け')
    if len(touched)>1:
      put('review','複数の集約を変える',ak,'・'.join(sorted(touched)),f'{" ・ ".join(g.dname(t) for t in sorted(touched))}を変える。1つのトランザクションで変える集約は1つなので、サーガかプロセスマネージャーで実装する（実装の定義）','レビュー')
  # ── 同じデータを操作するユースケースを、2つの文脈に分けていないか
  ctx_of_agg={}
  for A in APPS:
    for s in A['steps']:
      h=s['calls'].split('.')[0]
      if D[h]['kind']=='aggregate': ctx_of_agg.setdefault(h,set()).add(A['header']['context'])
  for a,cs in ctx_of_agg.items():
    put('structure','同じデータのユースケースを1つの文脈に置く',a,'・'.join(sorted(cs)),(f'{g.dname(a)}を変えるユースケースは、すべて {g.dname(list(cs)[0])} にある' if len(cs)==1 else f'{g.dname(a)}を変えるユースケースが {len(cs)} つの文脈に分かれている'),'合格' if len(cs)==1 else 'ずれ')
  # ── サブドメインが束ねるユースケース
  for k,v in D.items():
    if v['kind']!='subdomain': continue
    for u in v.get('use_cases',[]):
      if u not in D: put('structure','束ねるユースケースがある',k,u,'指したユースケースが無い','ずれ')
  # ── 上流の変更（承認した時点のハッシュ値と比べる）
  Cr={} if rec is None else rec
  for a,f,r in links():
    if f in ('satisfies','calls'): continue
    key=f'{a}→{r}'; now=fp(up(r))
    if key not in Cr: put('change',f'上流の変更（{f}）',a,r,'まだ承認していない対応。PR の承認で記録する','確かめ直し')
    elif Cr[key]!=now: put('change',f'上流の変更（{f}）',a,r,'承認のあとで参照先が変わった。参照元を確かめ直す','確かめ直し')
    else: put('change',f'上流の変更（{f}）',a,r,'承認した時点から変わっていない','合格')
  return R
def cond_targets(c):
  out=[c['target']]
  if c.get('if'): out.append(c['if']['target'])
  return [t for t in out if isinstance(t,str) and t.startswith('TERM-')]
if __name__=='__main__':
  import collections
  r=checks(json.load(open(APPROVED,encoding='utf-8'))); print(collections.Counter(x['status'] for x in r))
  for x in r:
    if x['status']!='合格': print(x)
