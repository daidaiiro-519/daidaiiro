#!/usr/bin/env python3
"""concrete：宣言の検査と、テストの記録との突き合わせ。終了コード 0 正常 ／ 1 検出あり ／ 2 誤用
道具が読むのは、宣言（spec/）と、テストが走ったときに書いた記録ファイルの2つだけである。
テストのソースも、テストの結果も読まない。テストが通るかは、テストの実行器が判定する。

  concrete contract             記録の契約（どの言語のテストも、これだけを守る）
  concrete conditions           テスト条件の一覧（ID ・ ハッシュ値 ・ 求めるレベル ・ 確かめること）
  concrete check                宣言どうしの検査
  concrete match <記録>          宣言と記録を突き合わせる（欠け ・ 余り ・ 古い ・ レベル違い）
  concrete patch <JSON Patch>    宣言を JSON Patch（RFC 6902）で更新する。検証を通るときだけ書く
  concrete approve              宣言どうしの対応を、いまの姿で承認の記録にする（spec/confirmed.json）
"""
import sys,os,json,collections,copy
sys.path.insert(0,os.path.dirname(__file__))
import gen5, drift5, spec5
from data5 import ROOT,RETIRED,D
REC=os.path.join(ROOT,'spec/confirmed.json')
CONTRACT="""記録の契約（concrete。言語によらない）
1. テストを始めるときに、環境変数 CONCRETE_TRACE を読む。設定されていなければ何もしない。
2. 設定されていれば、そのファイルの末尾へ、JSON を1行追記する（改行で終える）。
   {"condition": テスト条件の ID, "hash": テスト条件のハッシュ値, "level": テストレベル, "test": テストの名前（任意）}
3. 値は concrete conditions が出したものを、そのまま書く。ハッシュ値は、テストを書いたときに読んだ値である。
4. 追記する処理は、その言語の標準ライブラリだけで補助の関数として1つ作り、テストの間で共有する。
5. 1つのテストが確かめるテスト条件ごとに1行書く。確かめないテスト条件の ID は書かない。
6. skip するテストは、書く前に skip する。
7. テストの合否はここに書かない。合否はテストの実行器が出す。
8. 記録は、テストを実際に実行したときにだけ書かれる。実行器が前回の結果を再利用する仕組み（キャッシュ）は切って流す。"""
def conds(): return {c['id']:c for c in drift5.conditions()}
def read_trace(p):
  rows=[]
  for i,l in enumerate(open(p,encoding='utf-8'),1):
    l=l.strip()
    if not l: continue
    x=json.loads(l)
    for k in ('condition','hash','level'):
      if k not in x: raise SystemExit(f'記録の {i} 行目に {k} が無い')
    rows.append(x)
  return rows
def main(a):
  cmd=a[0] if a else ''
  if cmd=='contract': print(CONTRACT); return 0
  if cmd=='conditions':
    for c in drift5.conditions(): print(f'{c["id"]:<20} {c["hash"]}  {c["required_level"]:<9} {c["checks"]}')
    return 0
  if cmd=='check':
    rec=json.load(open(REC)) if os.path.exists(REC) else {}
    allr=spec5.checks(rec); r=[x for x in allr if x['status']!='合格']
    print('宣言どうし：'+' ・ '.join(f'{k} {v}' for k,v in collections.Counter(x['status'] for x in allr).items()))
    for x in r: print(f'  {x["status"]}  {x["check"]}  {x["from"]} → {x["to"] or "―"}  {x["text"]}')
    return 1 if r else 0
  if cmd=='match':
    if len(a)<2 or not os.path.exists(a[1]): print('記録ファイルが無い（テストが1件も記録していない）'); rows=[]
    else: rows=read_trace(a[1])
    cs=conds(); out=[]
    for cid,c in cs.items():
      rs=[r for r in rows if r['condition']==cid]
      if not rs: out.append(('欠け',cid,f'記録が無い。今のハッシュ値 {c["hash"]} ・ 求めるレベル {c["required_level"]}'))
      elif any(r['hash']!=c['hash'] for r in rs): out.append(('古い',cid,f'記録のハッシュ値 {rs[0]["hash"]} ／ 今 {c["hash"]}　'+(rs[0].get('test') or '')))
      elif not any(r['level']==c['required_level'] for r in rs): out.append(('レベル違い',cid,f'記録は {rs[0]["level"]} ／ 求めるのは {c["required_level"]}'))
    for r in rows:
      if r['condition'] not in cs: out.append(('余り',r['condition'],('廃止した条件　' if r['condition'] in RETIRED else '仕様に無い ID　')+(r.get('test') or '')))
    n=collections.Counter(k for k,_,_ in out)
    print(f'突き合わせ：テスト条件 {len(cs)} 件 ・ 記録 {len(rows)} 行'+('' if not out else ' ・ '+' ・ '.join(f'{k} {v}' for k,v in n.items())))
    for k,cid,t in out: print(f'  {k}  {cid}  {t}')
    return 1 if out else 0
  if cmd=='patch':
    import sim5
    ops=json.load(open(a[1])); state=copy.deepcopy(D); sim5.apply(state,ops)
    for k,v in state.items(): json.dump(v,open(os.path.join(ROOT,'spec/decls',k+'.json'),'w'),ensure_ascii=False,indent=1)
    print(f'宣言を更新した：{len(ops)} 件の操作'); return 0
  if cmd=='approve':
    rec=spec5.record(); json.dump(rec,open(REC,'w'),ensure_ascii=False,indent=1,sort_keys=True)
    print(f'承認の記録：{len(rec)} 件の対応のハッシュ値を spec/confirmed.json に書いた'); return 0
  print(__doc__); return 2
if __name__=='__main__': sys.exit(main(sys.argv[1:]))
