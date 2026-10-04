# 宣言を、共通の部品（タイル ・ カード ・ 札 ・ 表 ・ 図）とデザイントークンで描画する
import json,html,sys,os,subprocess,itertools,re
HERE=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,HERE)
from data6 import D,RETIRED
import gen6 as g
from desc6 import DESC
import drift6 as drift, spec6 as spec_drift
SPEC=spec_drift.checks(json.load(open(spec_drift.APPROVED,encoding='utf-8')))
CONDS=drift.conditions()
OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'
DSB='/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/tool/target/release/design-svg'
for sub in ('decls','figures'):
  os.makedirs(f'{OUT}/{sub}',exist_ok=True)
  for f in os.listdir(f'{OUT}/{sub}'): os.remove(f'{OUT}/{sub}/{f}')
for k,v in D.items(): json.dump(v,open(f'{OUT}/decls/{k}.json','w'),ensure_ascii=False,indent=1)
json.dump({"conditions":CONDS,"retired":RETIRED},open(f'{OUT}/conditions.json','w'),ensure_ascii=False,indent=1)
E=html.escape; HC=[0]
THEME={"color.box-fill":"var(--paper)","color.box-stroke":"var(--line)","color.ink":"var(--ink)","color.ink-soft":"var(--muted)","color.ink-faint":"var(--muted)",
 "color.line":"var(--muted)","color.accent":"var(--accent)","color.accent-bg":"var(--accent-soft)","color.accent-fg":"var(--accent)","color.warn":"var(--warn)","color.warn-bg":"var(--warn-soft)"}
def svg(name,args,decl):
  decl=dict(decl); decl['theme']={**THEME,**decl.get('theme',{})}
  p=f'{OUT}/figures/{name}.json'; json.dump(decl,open(p,'w'),ensure_ascii=False)
  r=subprocess.run([DSB]+args+[p,'--out',f'{OUT}/figures/{name}.svg'],capture_output=True,text=True)
  if r.returncode: print(name,r.stdout.strip(),r.stderr.strip())
  return f'<figure class="fig">{open(OUT+f"/figures/{name}.svg").read()}</figure>'
figure=lambda n,d: svg(n,['figure'],d)
chart=lambda n,k,d: svg(n,['chart',k],d)
KIND={"domain":"ドメイン（プロダクト）","glossary":"用語集","other_requirements":"その他の要求","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","glossary","other_requirements","use_case","subdomain","context","aggregate","domain_service"]
SIDE={"domain":"要求の側","glossary":"要求の側","other_requirements":"要求の側","use_case":"要求の側"}
CAT={"中核":"t-core","補完":"t-supporting","一般":"t-generic"}
def ctx_of(d):
  if d['kind']=='use_case':
    c=g.contexts_of(d['id']); return c[0] if c else None
  return d['header'].get('context')
def term(ctx,t): return g.word(t)
def dname(i):
  d=D.get(i)
  if not d: return None
  n=d['header']['name']; return g.word(n)
def idt(i): return f'<span class="id">{E(i)}</span>'
def ref(i):
  n=dname(i); return f'<span class="missing">{E(i)}（未作成）</span>' if n is None else f'<a class="ref" href="#{E(i)}">{E(n)}</a>'
def helpbtn(t):
  if t not in DESC: return ''
  HC[0]+=1; i=f'h{HC[0]}'
  return f'<button class="help" popovertarget="{i}" aria-label="{E(t)}の説明">?</button><div class="pop" id="{i}" popover><b>{E(t)}</b><p>{E(DESC[t])}</p></div>'
def lab(t): return f'{E(t)}{helpbtn(t)}'
def pill(t,tone=''): return f'<span class="pill {tone}">{E(t)}</span>'
def pills(ts,tone=''): return ''.join(pill(t,tone) for t in ts)
def tbl(cols,rows):
  # テスト条件の列は表に出さず、隣の欄の後ろに付ける（宣言の欄を表示したときだけ見える）
  if cols and cols[0]=='テスト条件':
    cols=cols[1:]; rows=[[r[1]+' '+r[0]]+list(r[2:]) for r in rows]
  elif cols and cols[-1]=='テスト条件':
    cols=cols[:-1]; rows=[[r[0]+' '+r[-1]]+list(r[1:-1]) for r in rows]
  lab_=[re.sub(r'<[^>]+>','',re.sub(r'<div class="pop".*?</div>','',c)).replace('?','').strip() for c in cols]
  return '<div class="tw"><table class="st"><thead><tr>'+''.join(f'<th>{c}</th>' for c in cols)+'</tr></thead><tbody>'+''.join('<tr>'+''.join(f'<td data-label="{E(lab_[i])}">{c}</td>' for i,c in enumerate(r))+'</tr>' for r in rows)+'</tbody></table></div>'
def block(title,body): return f'<section class="blk"><h2>{E(title)}{helpbtn(title)}</h2>{body}</section>'
def tiles(pairs): return '<div class="tiles">'+''.join(f'<div class="tile"><div class="tl">{lab(k)}</div><div class="tv">{v}</div></div>' for k,v in pairs)+'</div>'
def card(title,body,cls=''): return f'<div class="card {cls}"><h3>{title}</h3>{body}</div>'
def cards(xs): return '<div class="cards">'+''.join(xs)+'</div>'
def item(name,text='',extra=''): return f'<div class="item"><b>{E(name)}</b>'+(f'<span class="txt">{E(text)}</span>' if text else '')+extra+'</div>'
def tchip(cid):
  return f'<span class="tc">{E(cid)}</span>'
def tblock(decl):
  cs=[c for c in CONDS if c['decl']==decl]
  if not cs: return ''
  return '<div class="mx">'+block('テスト条件',tbl(['ID','対象','確かめること','求めるレベル','ハッシュ値'],[[tchip(c['id']),E(c['label']),E(c['checks']),pill(c['required_level']),f'<span class="no">{c["hash"]}</span>'] for c in cs]))+'</div>'
def head(d,badges='',lead=''):
  side=SIDE.get(d['kind'],'設計の側')
  return f'<label class="mt top"><input type="checkbox" class="mtog"> 宣言の欄を表示（ID ・ 参照 ・ テスト条件 ・ JSON）</label><header class="ph"><p class="kind">{pill(side)} {KIND[d["kind"]]}{badges}</p><h1>{E(dname(d["id"]))}{idt(d["id"])}</h1></header>'+(f'<p class="lead">{E(lead)}</p>' if lead else '')
def yn(v): return '<span class="yes">はい</span>' if v else '<span class="no-ans">いいえ</span>'
def gen(t): return f'{E(t)}'
def raw(d): return f'<details class="fold mx"><summary>{lab("宣言の JSON")}</summary><div class="fbody"><pre class="code">{E(json.dumps(d,ensure_ascii=False,indent=1))}</pre></div></details>'
def sv(x): return E(g.show(x))
def ex_rows(decl,cmd,exs):
  """例を、前の状態 ・ 引数 ・ 後の状態 ・ 業務イベントの表にする"""
  rows=[]
  for o in exs:
    bf='、'.join(f'{E(g.qname(k,decl,cmd))}＝{sv(v)}' for k,v in o['before'].items())
    ag='、'.join(f'{E(g.word(g.find(cmd["args"],k)["name"]))}＝{sv(v)}' for k,v in o.get('args',{}).items()) or '―'
    af=g.after(cmd,o)
    at='、'.join(f'{E(g.qname(k,decl,cmd))}＝{sv(v)}' for k,v in af.items())
    ev=' '.join(pill(g.word(e['name']),'t-accent') for e in cmd['emits']) or '―'
    rows.append([tchip(f'{decl}.{cmd["id"]}.{o["id"]}'),bf,ag,at,ev])
  return rows
# ── ドメイン
def p_domain(d):
  h=d['header']; vp=d['vision']
  uc={}
  for k,v in D.items():
    for sc in v.get('contributes_to',[]): uc.setdefault(sc,[]).append(k)
  SDS=[k for k,v in D.items() if v['kind']=='subdomain']
  b=head(d)+f'<section class="blk"><h2>{lab("ビジョン記述")}</h2>'+tiles([('課題',E(vp['problem']))])
  vals=[card(E(v['name'])+(pill('★ 競合との違い','t-accent') if v['differentiator'] else ''),f'<span class="txt">{E(v["text"])}</span>'+''.join(f'<span class="ln"><span class="k">担う</span>{ref(s)} {pill(D[s]["classification"]["category"],CAT[D[s]["classification"]["category"]])}</span>' for s in SDS if v['id'] in D[s]['serves_values']),'left-accent' if v['differentiator'] else 'left-neutral') for v in vp['values']]
  vals+=[card('競合 ・ '+E(c['name']),f'<span class="txt">{gen(g.competitor_text(c))}</span>','left-neutral') for c in vp['competitors']]
  b+=f'<h3 class="sub">{lab("提供価値")}</h3>'+cards(vals)
  def scrow(x):
    src='業務イベント' if x['source']=='event' else 'システムの外'
    return [f'<b>{E(x["name"])}</b>',gen(g.sc_text(x)),pill(src),pill(g.thr(x['threshold'])),pill(g.thr(x['ratio'])) if x.get('ratio') else '―',E(x['window']),' '.join(ref(u) for u in uc.get(x['id'],[])) or '<span class="missing">なし</span>']
  b+=f'<h3 class="sub">{lab("達成の基準")}</h3>'+tbl(['基準','組んだ文','測る対象','閾値','割合','期間','寄与するユースケース'],[scrow(x) for x in vp['success_criteria']])+'<p class="txt">運用で測る。テスト条件にはしない。</p></section>'
  ucs=[(k,v) for k,v in D.items() if v['kind']=='use_case']
  inc=''.join(f'<div class="item"><b>{ref(k)}</b><span class="txt">{E(v["header"]["primary_actor"])}の目的 ・ {E(v["header"]["level"])}</span></div>' for k,v in ucs if v['header']['level']=='ユーザー目的')
  summ=' '.join(ref(k) for k,v in ucs if v['header']['level']=='要約')
  outc=''.join(f'<div class="item"><b>{E(x)}</b></div>' for x in d['scope']['out'])
  b+=block('範囲',cards([card('作るもの（In）',inc+(f'<span class="ln"><span class="k">まとめる要約</span>{summ}</span>' if summ else ''),'left-accent'),card('作らないもの（Out）',outc,'left-neutral')])+'<p class="txt">作るものは、ユーザー目的のユースケースの一覧から道具が組む（どのユースケースがあるかを決めることが、作るものを決めることになる）。</p>')
  sh={x['id']:x['who'] for x in d['stakeholders']}
  nm=lambda x: E(sh.get(x) or next((y['name'] for y in d['design_scopes'] if y['id']==x),x))
  def realize(s):
    bcs=[k for k,v in D.items() if v['kind']=='context' and s in v['header']['subdomains']]
    ext=[(k,x) for k,v in D.items() if v['kind']=='context' for x in v['context_map']['relations'] if x.get('fulfills')==s]
    where=' '.join(ref(b2) for b2 in bcs)+' '.join(f'{E(x["external"])}（{ref(k)}の文脈の地図）' for k,x in ext)
    ucs=list(D[s].get('use_cases',[]))
    return where,ucs
  rows=[]
  for s in SDS:
    c=D[s]['classification']; w,u=realize(s)
    rows.append([ref(s)+f'<span class="what">{E(D[s]["header"]["description"])}</span>',pill(c['category'],CAT[c['category']]),E(c.get('sourcing','―')),w or '<span class="missing">なし</span>',E(g.impl_method(D[s]['business_logic'])),' '.join(ref(x) for x in u) or '―'])
  b+=block('サブドメインの組み立て（設計の側。道具が組む）',tbl(['サブドメイン','カテゴリー','調達','どこで','実装方法（導く）','使うユースケース'],rows)+'<p class="txt">道具が、サブドメイン ・ 区切られた文脈 ・ 文脈の地図の欄から組む。宣言の重さは、カテゴリーではなく実装方法で決まる。</p>')
  b+=block('設計スコープ',tbl(['高さ','名前','内側','外側'],[[pill(x['level']),f'<b>{E(x["name"])}</b>','、'.join(nm(y) for y in x.get('inside',[])) or '―','、'.join(nm(y) for y in x['outside'])] for x in d['design_scopes']])+'<p class="txt">ユースケースは、システムの高さのスコープを対象に書く。アプリ ・ 画面 ・ サービスへの分け方はスコープの内側のことで、実装の定義が持つ。</p>')
  b+=block('利害関係者',tbl(['利害関係者','関心'],[[f'<b>{E(x["who"])}</b>',f'<span class="txt">{E(x["interest"])}</span>'] for x in d['stakeholders']]))
  return b+raw(d)
def p_sd(d):
  c=d['classification']; bl=d['business_logic']; dc=g.derived_category(c)
  b=head(d,' '+pill(c['category'],CAT[c['category']]),d['header']['description'])
  SP=g.SD_SCHEMA['properties']
  def qa(group,ans):
    pr=SP[group]['properties']; ks=[k for r in SP[group]['x-derive']['rules'] for k in r['when']]
    ks=list(dict.fromkeys(ks))
    return tbl(['問い（どのサブドメインでも同じ）','このサブドメインの答え'],[[E(pr[k]['title']),yn(ans[k])] for k in ks])
  ok=c['category']==dc
  b+=block('カテゴリー',tiles([(SP['classification']['x-derive']['title'],pill(dc,CAT[dc])),('カテゴリー（宣言）',pill(c['category'],CAT[c['category']])+('' if ok else ' <span class="missing">導いたカテゴリーと違う</span>'))]+([('調達',E(c['sourcing']))] if c.get('sourcing') else []))+qa('classification',c))
  def rule_text(r):
    p=r.split('.'); dd=D.get(p[0])
    if dd is None: return E(r)
    if dd['kind']=='aggregate':
      inv=[x for x in dd['invariants'] if x['id']==p[1]]
      return E(g.cond(inv[0]['condition'],p[0])) if inv else E(r)
    if dd['kind']=='domain_service':
      op=[o for o in dd['operations'] if o['id']==p[1]][0]; res=[x for x in op['results'] if x['id']==p[2]][0]
      return E(dname(p[0])+'の結果：'+g.cond(res['condition'],p[0]))
    return E(r)
  b+=block('業務ロジックの性質',tiles([(SP['business_logic']['x-derive']['title'],f'<b>{E(g.impl_method(bl))}</b>')])+qa('business_logic',bl)+(('<h3 class="sub">'+E(SP['business_logic']['properties']['rule_conditions']['title'])+'</h3>'+tbl(['根拠の条件'],[[rule_text(r)+f'<span class="ln mx"><span class="no">{E(r)}</span></span>'] for r in bl['rule_conditions']])) if bl['rule_conditions'] else ''))
  if d['serves_values']: b+=block('担う提供価値',cards([card(E(v['name']),f'<span class="txt">{E(v["text"])}</span>','left-accent') for v in D['DOM-1']['vision']['values'] if v['id'] in d['serves_values']]))
  b+=block('束ねるユースケース',(' '.join(ref(u) for u in d.get('use_cases',[])) or '<span class="txt">なし（外部のサービスで満たす。使うユースケースはドメインの頁の組み立てに出る）</span>'))
  return b+raw(d)
def p_bc(d):
  h=d['header']; b=head(d)
  sds=' '.join(ref(s)+pill(D[s]['classification']['category'],CAT[D[s]['classification']['category']]) for s in h['subdomains']) or '<span class="txt">なし（このプロダクトの範囲の外）</span>'
  bd=d.get('boundary',{})
  b+=tiles([('モデルの目的',E(h['purpose'])),('配置の単位',E(bd.get('kind','―'))),('所有するチーム',E(bd.get('owner','―'))),('担うサブドメイン',sds)])
  mem=[k for k,v in D.items() if ctx_of(v)==d['id'] and v['kind'] in ('aggregate','value_object','domain_service')]
  ucs=[k for k,v in D.items() if ctx_of(v)==d['id'] and v['kind']=='use_case']
  if mem: b+=block('この文脈のモデル',tbl(['種類','名前'],[[KIND[kk],' ・ '.join(ref(m) for m in mem if D[m]['kind']==kk)] for kk in dict.fromkeys(D[m]['kind'] for m in mem)])+(f'<p class="ln"><span class="k">この文脈を使うユースケース</span>{" ".join(ref(u) for u in ucs)}</p>' if ucs else ''))
  T=g.terms(); ts=[dict(T[u['term']],**{"definition":[m for m in T[u['term']]['meanings'] if m['id']==u['meaning']][0]['definition'],"kind":[m for m in T[u['term']]['meanings'] if m['id']==u['meaning']][0]['kind']}) for u in d['uses'] if u['term'] in T]
  if ts:
    GEN={'動作','情報の別名','失敗の種類'}
    tcard=lambda xs,k: card(E(k),''.join(item(x['word'],x['definition'],(f'<span class="ln"><span class="k">使わない語</span>{" ".join(f"<span class=avoid>{E(a)}</span>" for a in x["avoid"])}</span>' if x['avoid'] else '')) for x in xs if x['kind']==k))
    bk=list(dict.fromkeys(x['kind'] for x in ts if x['kind'] not in GEN)); gk=list(dict.fromkeys(x['kind'] for x in ts if x['kind'] in GEN))
    gn=len([x for x in ts if x['kind'] in GEN])
    b+=block('用語集',f'<p class="txt">この文脈で使う、{ref("GLO-1")}の語と意味。同じ言葉が通用するのは、この文脈の内側だけ。</p>'+cards([tcard(ts,k) for k in bk])+(f'<details class="fold"><summary>文を組むための語（{gn}語：{" ・ ".join(gk)}）</summary><div class="fbody">'+cards([tcard(ts,k) for k in gk])+'</div></details>' if gk else ''))
  rel=d['context_map']['relations']
  if rel:
    nodes=[{"id":d['id'],"label":h['name'],"role":"focus"}]; edges=[]
    for n,rr in enumerate(rel,1):
      nodes.append({"id":f"x{n}","label":rr['external']}); edges.append({"from":d['id'],"to":f"x{n}","label":rr['pattern']})
    f=figure('context-'+d['id'],{"direction":"LR","nodes":nodes,"edges":edges})
    def how(rr):
      if rr.get('translates'): return '<br>'.join(f'<code>{E(t["theirs"])}</code> → {E(g.word(t["ours"]))}' for t in rr['translates'])
      if rr.get('uses'): return 'そのまま使う：'+'、'.join(E(g.word(t)) for t in rr['uses'])
      return '―'
    ops=[]
    for rr in rel:
      for o in rr.get('operations',[]):
        ops.append([f'<b>{E(rr["external"])}</b>',E(g.word(o['name'])),'、'.join(E(g.word(t)) for t in o['sends']),'、'.join(E(g.word(t)) for t in o['receives']),'<br>'.join(f'<span class="no">{E(f["id"])}</span> {E(g.word(f["name"]))}' for f in o['failures']),('してよい（'+E(g.word(o['retry']['key']))+'で同じ依頼と分かる）') if o['retry']['safe'] else 'してはいけない',ref(rr['fulfills']) if rr.get('fulfills') else '―'])
    opt=block('外部の操作（こちらから見た約束）',tbl(['相手','使う操作','渡すもの','受け取るもの','失敗の種類','再実行','満たすサブドメイン'],ops)) if ops else ''
    b+=block('文脈の地図',f+tbl(['相手（このプロダクトの外）','持ち主','連係方法','この文脈の側','こちらでの変換'],[[f'<b>{E(rr["external"])}</b>',E(rr['owner']),pill(rr['pattern']),pill(rr['direction']),how(rr)] for rr in rel])+'<p class="txt">相手の中身は、このプロダクトでは決めず、管理もしない。書くのは、こちらの側でどう変換するか、どの語をそのまま使うかだけ。</p>')+opt
  if d.get('business_rules'): b+=block('業務ルール',tbl(['ID','組んだ文',lab('実装するビジネスルール')],[[f'<span class="no">{E(x["id"])}</span>',gen(g.cond(x['condition'])),(ref('REQ-1')+'「'+E(g.item(x['implements'])[0]['name'])+'」') if x.get('implements') else '―'] for x in d['business_rules']]))
  return b+raw(d)
def p_agg(d):
  h=d['header']; st=d['structure']; k=d['id']
  ents={e['id']:e for e in st['entities']}
  b=head(d,f' ・ {ref(h["context"])}')+tiles([('守る一貫性',gen(g.agg_consistency(d)))])
  # 構造の表。エンティティの中の状態は、同じ表の行として親の行の下に並べ、親の行のボタンで開閉する（列をそろえる）
  cols=['状態','型','個数']
  def tr(cells,cls='',attr=''):
    return f'<tr class="{cls}"{attr}>'+''.join(f'<td data-label="{E(cols[i])}">{c}</td>' for i,c in enumerate(cells))+'</tr>'
  body=''
  for s in st['state']:
    t=s['type']; mx=tchip(f'{k}.{s["id"]}.MAX') if s['multiplicity']['max'] not in (None,1) else ''
    if t in ents:
      inner=ents[t]['state']; gid=f'{k}-{s["id"]}'
      btn=f'<button type="button" class="tg" aria-expanded="false" aria-controls="{gid}"><b>{E(g.word(s["name"]))}</b><span class="txt">中の状態 {len(inner)}つ</span></button>'
      body+=tr([btn+' '+mx,pill('エンティティ'),pill(g.mul_text(s['multiplicity']))],'ent-row')
      for x in inner:
        body+=tr([E(g.word(x['name'])),ref(x['type']) if x['type'] in D else E(x['type']),pill('1つ')],'ent-sub',f' data-grp="{gid}" hidden')
    else:
      ty=ref(t) if t in D else E(t)
      body+=tr([f'<b>{E(g.word(s["name"]))}</b> '+mx,ty,pill(g.mul_text(s['multiplicity']))])
  b+=block('構造','<div class="tw"><table class="st"><thead><tr>'+''.join(f'<th>{c}</th>' for c in cols)+'</tr></thead><tbody>'+body+'</tbody></table></div>')
  if d['invariants']:
    ir=[]
    for i in d['invariants']:
      ve=g.violation_example(d,i)
      ir.append([tchip(f'{k}.{i["id"]}'),gen(g.cond(i['condition'],k)),'、'.join(f'{E(g.qname(a,k))}＝{sv(v)}' for a,v in ve['before'].items()),' ・ '.join(E(g.word([c for c in d['commands'] if c['id']==x][0]['name'])) for x in ve['via']) or '<span class="missing">なし</span>'])
    b+=block('不変条件',tbl(['テスト条件','組んだ文','違反する状態（道具が組む）','至る操作'],ir))
  crow=[[f'<b>{E(g.word(c["name"]))}</b>','<br>'.join(E(g.post_text(p['condition'],k,c)) for p in c['state_changes']),' '.join(pill(g.word(p['reject'])) for p in c['business_rules']) or '―',' '.join(pill(g.word(e['name']),'t-accent') for e in c['emits']) or '―'] for c in d['commands']]
  b+=block('コマンド',tbl(['コマンド','状態の変更'+helpbtn('状態の変更'),'拒否の理由','業務イベント'],crow)+'<p class="txt">コマンドごとの業務ルールと例は、下で開く。</p>')
  for c in d['commands']:
    cname=g.word(c['name'])
    args=' '.join(f'{E(g.word(x["name"]))}（{ref(x["type"])}）' for x in c['args']) or 'なし'
    evs=' '.join(pill(g.word(e['name']),'t-accent')+'<span class="txt">'+' ・ '.join(E(g.field_name(d,f)) for f in e['fields'])+'</span>' for e in c['emits'])
    pr=[]
    for p in c['business_rules']:
      ex=g.reject_example(d,c,p)
      exs='、'.join(f'{E(g.qname(a,k,c))}＝{sv(v)}' for a,v in list(ex['before'].items())+list(ex.get('args',{}).items())) if ex else '<span class="missing">組めない</span>'
      pr.append([tchip(f'{k}.{c["id"]}.{p["id"]}'),gen(g.cond(p['condition'],k,c)),pill(g.word(p['reject'])),exs+(' '+pill('手で書いた例') if ex and ex.get('manual') else ' '+pill('道具が組む'))])
    pre=[p for p in c['business_rules'] if p['condition']['op']=='eq']; post=[p for p in c['state_changes'] if p['condition']['op']=='eq']
    nodes=[];edges=[];seen=set()
    for a in pre:
      for z in post:
        if a['condition']['target']==z['condition']['target'] and str(a['condition']['value']).startswith('TERM-') and str(z['condition']['value']).startswith('TERM-'):
          s1=g.word(a['condition']['value']); s2=g.word(z['condition']['value'])
          for x in (s1,s2):
            if x not in seen: seen.add(x); nodes.append({"id":x,"label":x})
          edges.append({"from":s1,"to":s2,"label":cname})
          for ev in c['emits']: nodes.append({"id":ev['id'],"label":g.word(ev['name']),"role":"muted"}); edges.append({"from":s2,"to":ev['id'],"label":"業務イベント","dashed":True})
    fig=figure('agg-'+k+'-'+c['id'],{"direction":"LR","nodes":nodes,"edges":edges}) if nodes else ''
    sec=f'<details class="fold"><summary>コマンド「{E(cname)}」の業務ルールと例（業務ルール{len(c["business_rules"])} ・ 状態の変更{len(c["state_changes"])} ・ 例{len(c.get("accept_examples",[]))}）</summary><div class="fbody">'+fig+tiles([('引数',args)]+([('業務イベント（項目は from から組む）',evs)] if evs else []))
    if pr: sec+=f'<h3 class="sub">業務ルールと拒否の例{helpbtn("業務ルール")}</h3>'+tbl(['テスト条件','業務ルール','拒否の理由','拒否の例'],pr)
    sec+=f'<h3 class="sub">状態の変更</h3>'+tbl(['状態の変更'],[[gen(g.post_text(p['condition'],k,c))+f'<span class="ln mx"><span class="no">{E(p["id"])}</span></span>'] for p in c['state_changes']])
    if c.get('accept_examples'): sec+=f'<h3 class="sub">受け付ける例{helpbtn("例")}</h3>'+tbl(['テスト条件','前の状態','引数','後の状態（道具が導く）','業務イベント'],ex_rows(k,c,c['accept_examples']))
    b+=sec+'</div></details>'
  b+=tblock(k)
  return b+raw(d)
def p_vo(d):
  ctx=d['header']['context']; k=d['id']
  gdef=[g.terms()[d['header']['name']]['definition']]
  b=head(d,f' ・ {ref(ctx)}',gdef[0] if gdef else '')
  def shape(c):
    if c.get('values'): return '値の一覧：'+' ・ '.join(g.word(v) for v in c['values'])
    parts=[]
    if c.get('digits'): parts.append(f'{c["digits"]}桁')
    if c.get('precision'): parts.append(c['precision'])
    return ' ・ '.join(parts) or '―'
  b+=block('構成する値',tbl(['名前','値の種類','桁 ・ 精度 ・ 値','単位'],[[f'<b>{E(c["name"])}</b>',pill(c['kind']),E(shape(c)),E(c.get('unit','―'))] for c in d['components']]))
  inv=[[tchip(f'{k}.{i["id"]}'),gen(g.cond(i['condition'],k)),pill(str(g.impossible(i['condition']))),' ・ '.join(f'{x}（{kk}）' for x,kk in g.bounds(i['condition']))] for c in d['components'] for i in c['invariants']]
  if inv: b+=block('不変条件',tbl(['テスト条件','組んだ文','作れない値（道具が組む）',f'境界値（道具が導く）'],inv))
  if d['operations']:
    b+=block('操作',tbl(['テスト条件','操作','引数','結果','例'],[[tchip(f'{k}.{o["id"]}.{x["id"]}'),f'<b>{E(g.word(o["name"]))}</b>',' ・ '.join(ref(a) for a in o['args']),ref(o['result']),E(f'{x["self"]} を {x["args"][0]} で「{g.word(o["name"])}」→ {x["result"]}')] for o in d['operations'] for x in o['accept_examples']]))
  b+=tblock(k)
  return b+raw(d)
def p_ds(d):
  h=d['header']; ctx=h['context']; k=d['id']
  b=head(d,' '+pill(h['reason'],'t-accent')+f' ・ {ref(ctx)}')+tiles([('置く理由（組んだ文）',gen(g.ds_reason(d))),('参照する集約（変更しない）',' ・ '.join(ref(x) for x in d['reads']))])
  for o in d['operations']:
    on=g.word(o['name'])
    nodes=[{"id":"op","label":on,"role":"focus"},{"id":"out","label":dname(o['output'])}]; at={"op":["b","r1"],"out":["c","r1"]}; edges=[{"from":"op","to":"out"}]
    for i,x in enumerate(o['inputs']): nodes.append({"id":f"in{i}","label":g.input_name(x)}); at[f"in{i}"]=["a",f"r{i}"]; edges.append({"from":f"in{i}","to":"op"})
    f=figure('ds-'+k+'-'+o['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","c"],"rows":sorted({v[1] for v in at.values()}),"at":at}})
    b+=f'<section class="blk"><h2>操作「{E(on)}」{helpbtn("操作")}</h2>{f}'
    b+='<h3 class="sub">入力</h3>'+tbl(['ID','組んだ名前','元にする状態','型'],[[f'<span class="no">{E(x["id"])}</span>',gen(g.input_name(x)),f'<span class="no">{E(x["from"]["target"])}</span>'+(' '+pill('合計') if x['from'].get('agg')=='sum' else ''),ref(x['type'])] for x in o['inputs']])
    b+='<h3 class="sub">結果</h3>'+tbl(['テスト条件','結果'],[[tchip(f'{k}.{o["id"]}.{p["id"]}'),gen(g.cond(p['condition'],k))] for p in o['results']])+'</section>'
  b+=tblock(k)
  return b+raw(d)
def estb(e):
  if not e: return '<span class="txt">（ユースケースを始める前から、注文は下書きである）</span>' if False else '<span class="txt">注文を作ったときから成り立つ</span>'
  if '.X-' in e:
    x,_,_=g.item(e); return f'{E(x["external"])}<span class="txt">（このプロダクトの外）</span>'
  return ref(e)
def p_uc(d):
  h=d['header']; sc=d['scenario']; ctx=h['scope'].get('context'); k=d['id']; nums=g.number(d)
  ctxl=''
  role={h['primary_actor']:'primary','システム':'system'}; role.update({a:'supporting' for a in sc['supporting_actors']})
  RL={'primary':'主','supporting':'支援','system':''}
  act=lambda a: f'<span class="act {role.get(a,"system")}">'+(f'<span class="ar">{RL[role.get(a,"system")]}</span>' if RL[role.get(a,'system')] else '')+f'{E(a)}</span>'
  dom={x['id']:x for x in D['DOM-1']['stakeholders']}
  sh={x['id']:x for x in d['stakeholders']}; gu=d['guarantees']
  who=lambda i: dom[sh[i]['who'].split('.')[-1]]['who']
  whos=lambda ids: pills([who(i) for i in ids])
  def pref(rr):
    x,decl,cmd=g.item(rr); a=rr.split('.')[0]
    if D[a]['kind']=='context':
      return f'{ref(a)}の外部の操作の失敗「{E(g.word(x["name"]))}」<span class="no">{E(rr)}</span>'
    return f'{ref(a)}「{E(g.word(cmd["name"]))}」の{"業務ルール" if x["id"].startswith("BR") else "状態の変更"}<span class="no">{E(x["id"])}</span>'
  QS={g.item(q)[0]['target'].split('.',1)[1]:q for q in d.get('links',{}).get('quality',[])}
  def calls_label(c):
    h=c.split('.')[0]
    if D[h]['kind']=='context':
      x,_,rel=g.item(c); return f'{E(rel["external"])}の「{E(g.word(x["name"]))}」'
    return E(g.cmd_label(c))
  def links(s):
    o=''
    if s.get('reply'): o+=f'<span class="ln"><span class="k">戻りメッセージ</span>{E(g.reply_text(s))}</span>'
    if s.get('calls'): o+=f'<span class="ln"><span class="k">呼ぶユースケース</span>{ref(s["calls"])}</span>'
    if s.get('checks'): o+=f'<span class="ln"><span class="k">確かめるビジネスルール</span>'+' '.join(f'{ref(r.split(".")[0])}「{E(g.item(r)[0]["name"])}」' for r in s['checks'])+'</span>'
    if s.get('keeps'): o+=f'<span class="ln"><span class="k">守る最低保証</span>{pills([m["name"] for m in gu["minimal"] if m["id"] in s["keeps"]],"t-minimal")}</span>'
    if s['id'] in QS:
      q=QS[s['id']]; o+=f'<span class="ln"><span class="k">品質の要求</span>{E(g.qr_text(g.item(q)[0]))} {tchip(f"{k}.{s[chr(105)+chr(100)]}.{q.split(chr(46))[-1]}")}</span>'
    return o
  parts=list(dict.fromkeys([h['primary_actor'],'システム']+sc['supporting_actors']+[t['actor'] for t in g.all_steps(d)])); msgs=[]; groups=[]
  def add(s):
    frm=s['actor']; to=s.get('to') or ('システム' if s.get('calls') else frm)
    msgs.append({"from":frm,"to":to,"label":f"{nums[s['id']]} {g.step_short(s)}"})
    if s.get('reply'): msgs.append({"from":to,"to":frm,"label":g.reply_text(s),"kind":"return"})
  for s in sc['steps']: add(s)
  TH={"font.size-small":14,"font.size":15,"chart.exchange-col-w":220,"chart.pad":6,"chart.exchange-row-h":40}
  seqm=chart('uc-'+k+'-main','exchange',{"participants":parts,"steps":list(msgs),"groups":[],"theme":TH})
  msgs.clear()
  for s in sc['steps']:
    add(s)
    for x in s['extensions']:
      st0=len(msgs)
      for t in x['steps']: add(t)
      et=g.ending_text(x,nums)
      groups.append({"label":'break' if x['ending']=='失敗' else 'opt',"cases":[{"name":f"{nums[x['id']]} → {et}" if et else nums[x['id']],"span":[st0,len(msgs)-1]}]})
  seq=chart('uc-'+k,'exchange',{"participants":parts,"steps":msgs,"groups":groups,"theme":{"font.size-small":14,"font.size":15,"chart.exchange-col-w":220,"chart.pad":6,"chart.exchange-row-h":40}})
  scs=[x for x in D['DOM-1']['vision']['success_criteria'] if x['id'] in d['contributes_to']]
  trig=[s for s in sc['steps'] if s['id']==h['trigger_step']][0]
  sys_=[x for x in D['DOM-1']['design_scopes'] if 'DOM-1.'+x['id']==h['scope']['system']][0]
  b=head(d,' '+pill(h['level'])+f' ・ スコープ <a class="ref" href="#DOM-1">{E(sys_["name"])}</a>（{E(sys_["level"])}）'+ctxl)
  b+=tiles([('主アクター',act(h['primary_actor'])),('支援アクター',''.join(act(a) for a in sc['supporting_actors'])),('トリガー',gen(g.step_text(trig))),('寄与する達成の基準',' '.join(f'<a class="ref" href="#DOM-1">{E(x["name"])}</a>' for x in scs))])
  sline=lambda x: ''
  sgc=card(lab('成功時保証'),''.join(item(x['name'],g.sg_text(x),f'<span class="ln">{whos(x["satisfies"])}</span>'+sline(x)) for x in gu['success']),'top-success')
  mgc=card(lab('最低保証'),''.join(item(x['name'],g.mg_text(x),f'<span class="ln">{whos(x["protects"])}</span>') for x in gu['minimal']),'top-minimal')
  b+='<div style="height:.75rem"></div>'+cards([sgc,mgc])
  if d['preconditions']: b+=block('事前条件',tbl(['事前条件','成り立たせるもの'],[[gen(g.pre_text(x)),estb(x.get('established_by'))] for x in d['preconditions']]))
  def who_to(t): return act(t['actor'])
  def body(t):
    s=g.step_text(t); p=t['actor']+'は、'
    return s[len(p):] if s.startswith(p) else s
  def meta(t,extra=''):
    return f'<div class="meta"><span class="no">{E(t["id"])} ・ {E(t["kind"])}</span>{links(t)}{extra}</div>'
  def line(t,no,cls='step'):
    rep=f'<span class="reply">← {E(g.reply_text(t))}</span>' if t.get('reply') else ''
    return f'<li class="{cls}"><span class="sn">{E(no)}</span><div class="sb"><div class="sh">{who_to(t)}<b>{gen(body(t))}</b>{rep}</div>{meta(t)}</div></li>'
  fl=''
  for s in sc['steps']:
    fl+=line(s,nums[s['id']])
    for x in s['extensions']:
      fail=x['ending']=='失敗'
      sub=''.join(line(t,nums[t['id']],'sub') for t in x['steps'])
      end=(f'<li class="end fail"><span class="sn">✕</span><div class="sb"><b>失敗で終わる</b><span class="note">最低保証が成り立つ</span></div></li>' if fail
           else '' if x['ending']=='成功'
           else f'<li class="end back"><span class="sn">✓</span><div class="sb"><b>ユースケースは終了する</b></div></li>' if x['ending']=='終了'
           else f'<li class="end back"><span class="sn">↩</span><div class="sb"><b>手順{E(nums[x["ending"]])}へ戻る</b></div></li>')
      xm=f'<div class="meta"><span class="no">{E(x["id"])} ・ {E(x["condition_kind"])}</span>'+f'<span class="ln">{tchip(k+"."+x["id"])}</span></div>'
      fl+=f'<li class="ext"><div class="xh"><span class="xl">{E(nums[x["id"]])}</span><b>{gen(g.ext_text(x))}</b></div>{xm}<ol class="xs">{sub}{end}</ol></li>'
  fl+=f'<li class="end ok"><span class="sn">✓</span><div class="sb"><b>成功で終わる</b><span class="note">成功時保証が成り立つ</span><div class="meta">{tchip(k+".M")}</div></div></li>'
  b+=f'<section class="blk"><h2>主成功シナリオと拡張{helpbtn("主成功シナリオ")}</h2><ol class="flow">{fl}</ol></section>'
  b+=f'<details class="fold near"><summary>主成功シナリオのシーケンス図{helpbtn("シーケンス図")}</summary><div class="fbody">{seqm}</div></details>'
  b+=f'<details class="fold near"><summary>拡張を含むシーケンス図</summary><div class="fbody">{seq}</div></details>'
  srows=[[f'<b>{E(who(i))}</b>',f'<span class="txt">{E(x["interest"])}</span>','、'.join(nums[s['id']] for s in sc['steps'] if i in s.get('serves',[])) or '<span class="missing">なし</span>',pills([m['name'] for m in gu['success'] if i in m['satisfies']],'t-success')+pills([m['name'] for m in gu['minimal'] if i in m['protects']],'t-minimal')] for i,x in sh.items()]
  b+=block('利害関係者と利益',tbl(['利害関係者','利益','守る手順','守る保証'],srows))
  vr=[[nums[s['id']],E(v['varies']),pills(v['values'])] for s in sc['steps'] for v in s.get('variations',[])]
  if vr: b+=f'<details class="fold"><summary>技術およびデータのバリエーション（{len(vr)}件）{helpbtn("技術およびデータのバリエーション")}</summary><div class="fbody">{tbl(["手順","違い","値"],vr)}</div></details>'
  lk=d.get('links',{})
  lrows=[[E({'business_rules':'ビジネスルール','quality':'品質の要求','technology':'使われる技術','data':'データ要求'}[kk]),'、'.join(ref(r.split('.')[0])+'「'+E(g.item(r)[0].get('name') or g.item(r)[0].get('system') or g.item(r)[0].get('measure'))+'」' for r in v)] for kk,v in lk.items() if v]
  if lrows: b+=block('関連情報',tbl(['その他の要求','結ぶもの'],lrows))
  if h['level']!='要約':
    sds=g.subdomains_of(k); bcs=g.contexts_of(k)
    b+=block('束ねる設計の側',(f'<p class="txt">このユースケースを束ねるサブドメイン：{" ".join(ref(x) for x in sds)}。そのサブドメインを対象とする区切られた文脈：{" ".join(ref(x) for x in bcs) or "なし"}。ユースケースは設計の側を指さないので、道具が設計の側の従属関係から逆向きに引いて見せる。</p>') if sds else '<p class="missing">このユースケースを束ねるサブドメインが無い</p>')
  if d.get('open_issues'): b+=f'<details class="fold"><summary>未決定事項（{len(d["open_issues"])}件）{helpbtn("未決定事項")}</summary><div class="fbody">{tbl(["未決定事項"],[[E(x)] for x in d["open_issues"]])}</div></details>'
  b+=tblock(k)
  return b+raw(d)
def p_glossary(d):
  b=head(d,'',f'プロダクトで使う語と、その意味（{len(d["terms"])}語）。区切られた文脈は、ここからどの語のどの意味を使うかを指す。')
  users={}
  for k,v in D.items():
    if v['kind']=='context':
      for u in v['uses']: users.setdefault((u['term'],u['meaning']),[]).append(k)
  GEN={'動作','情報の別名','失敗の種類'}
  rows=[];grows=[]
  for t in d['terms']:
    for m in t['meanings']:
      r=[f'<b>{E(t["word"])}</b>'+(f' <span class="no">{E(m["id"])}</span>' if len(t['meanings'])>1 else ''),f'<span class="txt">{E(m["definition"])}</span>',pill(m['kind']),' '.join(ref(c) for c in users.get((t['id'],m['id']),[])) or '<span class="txt">使う文脈は無い</span>',' '.join(f'<span class="avoid">{E(a)}</span>' for a in t['avoid']) or '―']
      (grows if m['kind'] in GEN else rows).append(r)
  cols=['語','意味','種類','使う文脈','使わない語']
  b+=block('語と意味',tbl(cols,rows)+f'<details class="fold"><summary>文を組むための語（{len(grows)}語）</summary><div class="fbody">{tbl(cols,grows)}</div></details>')
  return b+raw(d)
def p_req(d):
  b=head(d,'','ユースケースの外に書く要求。ユースケースから関連情報で結ぶ。')
  uses=lambda r: ' '.join(ref(k) for k,v in D.items() if v['kind']=='use_case' and r in [x for xs in v.get('links',{}).values() for x in xs])
  impl=lambda r: ' '.join(ref(k) for k,v in D.items() if v['kind']=='context' for x in v.get('business_rules',[]) if x.get('implements')==r)
  sh={x['id']:x['who'] for x in D['DOM-1']['stakeholders']}
  b+=block('ビジネスルール',tbl(['ルール','組んだ文','守る利害関係者','結ぶユースケース','実装する文脈'],[[f'<b>{E(x["name"])}</b>',gen(g.cond(x['condition'])),pills([sh[p.split('.')[-1]] for p in x['protects']]),uses(f'{d["id"]}.{x["id"]}'),impl(f'{d["id"]}.{x["id"]}') or '<span class="missing">なし</span>'] for x in d['business_rules']]))
  b+=block('品質の要求',tbl(['組んだ文','閾値','割合','条件','非機能要求グレード','測り方'],[[gen(g.qr_text(q)),pill(g.thr(q['threshold'])),pill(g.thr(q['ratio'])),E(q['condition']['period']+' ・ '+str(q['condition']['load']['value'])+q['condition']['load']['unit']),E(f'{q["grade"]["item"]} ・ レベル{q["grade"]["level"]}'),pill(q['method'])] for q in d['quality']]))
  b+=block('使われる技術',tbl(['相互作用するシステム','要求'],[[f'<b>{E(x["system"])}</b>',f'<span class="txt">{E(x["text"])}</span>'] for x in d['technology']]))
  if d.get('data'): b+=block('データ要求',tbl(['項目','組んだ文','結ぶユースケース'],[[f'<b>{E(x["name"])}</b>',gen(g.cond(x['condition'])),uses(f'{d["id"]}.{x["id"]}')] for x in d['data']]))
  if d['open_issues']: b+=block('未決定事項',tbl(['未決定事項'],[[E(x)] for x in d['open_issues']]))
  return b+raw(d)
def p_drift():
  import collections
  b='<header class="ph"><p class="kind">テスト</p><h1>テスト条件と宣言どうしの検査</h1></header><p class="lead">宣言から道具が出すもの。テストの合否はここに無い（テストの実行器が判定し、保存しない）。</p>'
  scnt=collections.Counter(x['status'] for x in SPEC)
  b+='<h2 class="sec">宣言どうし</h2>'+tiles([(k,f'<b style="font-size:1.3rem">{scnt.get(k,0)}</b>') for k in ['ずれ','欠け','確かめ直し','レビュー','合格']])
  CAT={'structure':'構造で検査','change':'上流の変更','review':'人のレビュー'}
  SST={'合格':'','ずれ':'t-warn','欠け':'t-warn','確かめ直し':'','レビュー':''}
  def at(i):
    if ':' in i: a,r=i.split(':',1); return ref(a)+' → '+at(r)
    if i.startswith('TERM-'): return f'「{E(g.word(i))}」'
    return ref(i.split('.')[0])+(f' <span class="no">{E(".".join(i.split(".")[1:]))}</span>' if '.' in i else '') if i and i.split('.')[0] in D else E(i)
  srow=[[pill(x['status'],SST[x['status']]),E(CAT[x['category']]),E(x['check']),at(x['from']),'、'.join(at(t) for t in x['to'].split('・')) if x['to'] else '',E(x['text'])] for x in sorted(SPEC,key=lambda x:(x['status']=='合格',list(CAT).index(x['category'])))]
  b+=block('宣言どうしの検査',tbl(['状態','分け方','検査','参照元','参照先','内容'],srow))
  b+='<h2 class="sec">宣言とテスト</h2>'
  b+=block('テスト条件',tbl(['ID','宣言','対象','確かめること','求めるレベル','ハッシュ値'],[[tchip(c['id']),ref(c['decl']),E(c['label']),E(c['checks']),pill(c['required_level']),f'<span class="no">{c["hash"]}</span>'] for c in CONDS]))
  b+=block('突き合わせ',tbl(['何を','どこで見られるか'],[
    ['テストは走ったときに、上の ID ・ ハッシュ値 ・ レベルを記録ファイルへ1行ずつ追記する（記録の契約）。道具は宣言と記録だけを照らし、欠け ・ 余り ・ 古い ・ レベル違いを出す','<a class="ref" href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">テスト条件 ID の突き合わせ</a>'],
    ['Python と Go のテストを実際に実行し、9手のコミットごとに CI を流した記録','<a class="ref" href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録</a>']]))
  return b
# ── 書き方（スキーマ）。各欄の description と x-prompt.write を、そのまま並べる
SCH={os.path.basename(f)[:-12]:json.load(open(f,encoding='utf-8')) for f in __import__('glob').glob(os.path.join(HERE,'schema','*.schema.json'))}
def sch_rows(props):
  return [[f'<b>{E(v.get("description",""))}</b>',f'<code>{E(k)}</code>',f'<span class="txt">{E(v.get("x-prompt",{}).get("write",""))}</span>'] for k,v in props.items() if k not in ('$schema',)]
def p_sch(k):
  sc=SCH[k]; b=f'<header class="ph"><p class="kind">{pill("書き方")} スキーマ</p><h1>{E(sc["title"])}{idt(k+".schema.json")}</h1></header><p class="lead">{E(sc["description"])}</p>'
  if k=='common':
    return b+block('共通の形',tbl(['項目','名前','書くこと'],sch_rows(sc['$defs'])))
  b+=block('欄と書くこと',tbl(['項目','キー','書くこと'],sch_rows(sc['properties'])))
  for n,d in sc.get('$defs',{}).items():
    if n=='sub_step': continue
    b+=f'<section class="blk"><h2>{E(d["description"])}</h2><p class="txt">{E(d["x-prompt"]["write"])}</p>{tbl(["項目","キー","書くこと"],sch_rows(d.get("properties",{})))}</section>'
  return b
R={"domain":p_domain,"glossary":p_glossary,"other_requirements":p_req,"subdomain":p_sd,"context":p_bc,"aggregate":p_agg,"value_object":p_vo,"domain_service":p_ds,"use_case":p_uc}
SCH_ORDER=[k for k in ORDER+['value_object']]+['common']
nav_sch='<div class="ng"><span class="nk">書き方（スキーマ）</span>'+''.join(f'<a href="#SCH-{k}" data-id="SCH-{k}">{E(SCH[k]["title"])}</a>' for k in SCH_ORDER)+'</div>'
nav='<div class="ng"><span class="nk">テスト</span><a href="#DRIFT" data-id="DRIFT">テスト条件と検査</a><a href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">突き合わせ ↗</a><a href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録 ↗</a></div>'+''.join(f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i,v in D.items() if v['kind']==k)+'</div>' for k in ORDER)+nav_sch
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())+f'<article class="page" id="DRIFT">{p_drift()}</article>'+''.join(f'<article class="page" id="SCH-{k}">{p_sch(k)}</article>' for k in SCH_ORDER)
css=open(os.path.join(HERE,'tokens.css')).read()+open(os.path.join(HERE,'sample4.css')).read()
page=f'''<title>モバイルオーダーの宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">{E(D["DOM-1"]["header"]["name"])}</p><details class="navd" open><summary>目次を開く ・ 閉じる</summary><nav class="nav">{nav}</nav></details></aside>
<main class="main">{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);const hit=[...document.querySelectorAll('.page')].some(p=>p.id===id);const cur=hit?id:'DOM-1';document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==cur);document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===cur));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
document.querySelectorAll('.tg').forEach(b=>b.addEventListener('click',()=>{{const o=b.getAttribute('aria-expanded')!=='true';b.setAttribute('aria-expanded',o);document.querySelectorAll('tr[data-grp="'+b.getAttribute('aria-controls')+'"]').forEach(r=>r.hidden=!o)}}));
if(innerWidth<=760){{const n=document.querySelector('.navd');if(n)n.open=false;document.querySelectorAll('.nav a').forEach(a=>a.addEventListener('click',()=>{{n.open=false}}))}}
const go=n=>{{document.querySelectorAll('.simstep').forEach(s=>s.hidden=s.dataset.no!=n);document.querySelectorAll('.stepper .sbtn').forEach(b=>b.classList.toggle('on',b.dataset.go==n));const st=document.querySelector('.stepper');if(st&&window.scrollY>st.offsetTop)st.scrollIntoView();}};
document.querySelectorAll('.sbtn').forEach(b=>b.addEventListener('click',()=>{{go(b.dataset.go);document.querySelector('.stepper').scrollIntoView({{behavior:'smooth'}})}}));go('0');
</script>'''
open(f'{OUT}/viewer.html','w').write(page); print(len(page))
