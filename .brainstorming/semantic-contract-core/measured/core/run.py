"""核のランナー。behaviors/*.json の場面を、書いたアダプターごとに実行し、期待値と比べる。
完了基準 K1〜K8 を判定し、1つでも不合格なら0以外で終わる。人が読み比べる出力は出さない（report.json に書く）。
使い方：python3 run.py [--tmp <作業の場所>] [--reverse]"""
import copy, glob, json, os, sqlite3, sys, tempfile, shutil
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, os.path.join(HERE, 'ref')); sys.path.insert(0, os.path.join(HERE, 'check'))
import adapters as A, base as B, schemas as SC
from jsonschema import Draft202012Validator
from referencing import Resource
from referencing.jsonschema import DRAFT202012

DDL = os.path.join(HERE, 'adapters', 'sqlite.sql')
DOCS = SC.load_all()


def ptr(doc, p):
    if p in ('', '/'): return doc
    cur = doc
    for k in p.lstrip('/').split('/'):
        k = k.replace('~1', '/').replace('~0', '~')
        if isinstance(cur, list): cur = cur[int(k)]
        else: cur = cur[k]
    return cur


def setup(sc, adapter, tmp):
    g = sc.get('given', {})
    root = tempfile.mkdtemp(dir=tmp); w = os.path.join(root, 'work'); os.makedirs(w)
    if adapter == 'git': ch = A.GitChanges(w)
    else: snap = os.path.join(root, 'snap'); os.makedirs(snap); ch = A.SnapshotChanges(w, snap)
    ch.init(g.get('files', {}))
    st = A.SqliteSessions(os.path.join(root, 'local.db'), DDL)
    c = st.conn()
    for r in g.get('sessions', []):
        p = r.get('parent') or {}
        c.execute('INSERT INTO sessions VALUES (?,?,?,?,?,?,?,?)', (r['session']['tool'], r['session']['id'], r.get('purpose_ref'), p.get('tool'), p.get('id'), r.get('role'), r['state'], r['last_used_at']))
    for r in g.get('index', []): c.execute('INSERT INTO session_changes VALUES (?,?,?)', (r['session']['tool'], r['session']['id'], r['change_id']))
    sset = g.get('settings', {}); user = os.path.join(root, 'user-settings.json')
    for scope, path in [('user', user), ('project', os.path.join(w, '.semantic-agent-schema', 'settings.json')), ('local', os.path.join(w, '.semantic-agent-schema', 'settings.local.json'))]:
        if scope in sset:
            os.makedirs(os.path.dirname(path), exist_ok=True)
            json.dump({'$schema': 'urn:semantic-agent-schema:base:settings:v1', 'kind': 'settings', 'id': scope, 'session': sset[scope]}, open(path, 'w'))
    if 'project' in sset: ch.confirm(['.semantic-agent-schema/settings.json'], 'settings')
    judge = A.StubJudge(g.get('judge', {}))
    base = B.Base(w, ch, st, A.FakeAiSessions(g.get('ai', {})), judge, A.FileSettings(user, w), g.get('ids', []), g.get('now', '2026-10-24T10:00:00Z'))
    ctx = {'base': base, 'w': w, 'root': root, 'refs': {'start': ch.last_confirmed()['ref']}}
    for step in g.get('steps', []): do(ctx, step)
    for p, content in g.get('pending', {}).items():
        fp = os.path.join(w, p)
        if content is None: os.remove(fp)
        else: os.makedirs(os.path.dirname(fp), exist_ok=True); open(fp, 'w').write(content if isinstance(content, str) else json.dumps(content, ensure_ascii=False, indent=1))
    ctx['refs']['before_when'] = ch.last_confirmed()['ref']
    return ctx


def subst(x, ctx):
    if isinstance(x, str) and x.startswith('$ref:'): return ctx['refs'][x[5:]]
    if isinstance(x, dict): return {k: subst(v, ctx) for k, v in x.items()}
    if isinstance(x, list): return [subst(v, ctx) for v in x]
    return x


def do(ctx, step):
    op, args = step['op'], subst(step.get('args', {}), ctx)
    base = ctx['base']
    if op == 'edit':
        for p, content in args.items():
            fp = os.path.join(ctx['w'], p)
            if content is None: os.remove(fp)
            else: os.makedirs(os.path.dirname(fp), exist_ok=True); open(fp, 'w').write(content if isinstance(content, str) else json.dumps(content, ensure_ascii=False, indent=1))
        return {}
    if op == 'sql':
        c = base.st.conn(fk=args.get('foreign_keys', True))
        try:
            c.execute(args['sql'], args.get('params', [])); return {'ok': True}
        except sqlite3.DatabaseError as e:
            return {'refused': str(e)}
    if op == 'concurrent_register':
        d1 = base.decide(**args['decide']); d2 = base.decide(**args['decide'])
        r1 = base.register(d1, args['first'], **args.get('slot', {})); r2 = base.register(d2, args['second'], **args.get('slot', {}))
        return {'first': r1, 'second': r2}
    if op == 'decide_and_register':
        d = base.decide(**args['decide'])
        r = base.register(d, args['new'], **args.get('slot', {})) if d['action'] == 'new' else None
        return {'decision': d, 'register': r}
    if op == 'mark':
        ctx['refs'][args['name']] = base.ch.last_confirmed()['ref']; return {}
    res = getattr(base, op)(**args)
    if isinstance(res, dict) and 'ref' in res and step.get('save_ref'): ctx['refs'][step['save_ref']] = res['ref']
    return res


def state(ctx):
    base = ctx['base']
    recdir = os.path.join(ctx['w'], '.semantic-agent-schema', 'changes')
    jd = os.path.join(ctx['w'], '.semantic-agent-schema', 'judgments')
    juds = sorted(os.path.relpath(os.path.join(d, f), jd) for d, _, fs in os.walk(jd) for f in fs) if os.path.isdir(jd) else []
    return {'sessions': base.st.all(), 'index': base.st.all_index(), 'judgments': juds,
            'pending': [i['target'] for i in base._pending()],
            'records': sorted(os.listdir(recdir)) if os.path.isdir(recdir) else []}


def check(sc, res, st0, st1):
    doc = {'result': res, 'state': st1}; fails = []
    for t in sc.get('then', []):
        try:
            v = ptr(doc, t['at'])
            ok = (('equals' not in t or v == t['equals']) and ('contains' not in t or t['contains'] in v) and
                  ('count' not in t or len(v) == t['count']) and ('exists' not in t or t['exists']))
        except (KeyError, IndexError, TypeError):
            v, ok = None, ('exists' in t and t['exists'] is False)
        if not ok: fails.append({'id': t['id'], 'message': t['message'], 'got': v})
    for k in sc.get('unchanged', []):
        if st0[k] != st1[k]: fails.append({'id': 'unchanged:' + k, 'message': f'{k} が変わった', 'got': st1[k]})
    if 'result_schema' in sc and isinstance(res, dict) and not ({'not_shown', 'exception', 'refused'} & set(res)):
        reg = SC.registry(DOCS); sch = {'$ref': sc['result_schema']}
        errs = list(Draft202012Validator(sch, registry=reg).iter_errors(ptr(res, sc.get('result_schema_at', ''))))
        if errs: fails.append({'id': 'result-schema', 'message': '出力が Abstract のファイルの形に合わない', 'got': errs[0].message[:200]})
    return fails


def run_schema_case(sc):
    g = sc['given']; sid = g['schema']; doc = g['doc']
    reg = SC.registry(DOCS)
    if 'concrete' in g:
        c = json.load(open(os.path.join(HERE, g['concrete']))); reg = reg.with_resource(c['$id'], Resource.from_contents(c, default_specification=DRAFT202012))
        sch = c if sid == c['$id'] else {'$ref': sid}
    else:
        sch = {'$ref': sid}
    if isinstance(doc, str): doc = json.load(open(os.path.join(HERE, doc)))
    for patch in g.get('patch', []):
        cur = doc
        ks = patch['at'].lstrip('/').split('/')
        for k in ks[:-1]: cur = cur[int(k)] if isinstance(cur, list) else cur[k]
        if patch.get('remove'): cur.pop(ks[-1])
        else: cur[ks[-1]] = patch['value']
    errs = list(Draft202012Validator(sch, registry=reg).iter_errors(doc))
    return {'valid': not errs, 'errors': [e.message[:160] for e in errs[:3]]}


def normalize(x, ctx):
    s = json.dumps(x, ensure_ascii=False, sort_keys=True)
    for k, v in ctx['refs'].items(): s = s.replace(v, '<' + k + '>')
    return json.loads(s)


def main():
    tmp = sys.argv[sys.argv.index('--tmp') + 1] if '--tmp' in sys.argv else None
    files = sorted(glob.glob(os.path.join(HERE, 'behaviors', '*.json')))
    if '--reverse' in sys.argv: files = files[::-1]
    scenarios = [s for f in files for s in json.load(open(f))]
    results, outputs = [], {}
    for sc in scenarios:
        if sc['when']['op'] == 'schema':
            res = run_schema_case(sc); fails = check(sc, res, {}, {})
            results.append({'id': sc['id'], 'adapter': 'static', 'pass': not fails, 'fails': fails}); continue
        for ad in sc.get('adapters', ['snapshot']):
            ctx = setup(sc, ad, tmp); st0 = state(ctx)
            try: res = do(ctx, sc['when'])
            except Exception as e: res = {'exception': repr(e)}
            if isinstance(res, dict) and isinstance(res.get('ref'), str): ctx['refs']['when'] = res['ref']
            st1 = state(ctx); fails = check(sc, res, st0, st1)
            outputs.setdefault(sc['id'], {})[ad] = normalize(res, ctx)
            results.append({'id': sc['id'], 'adapter': ad, 'pass': not fails, 'fails': fails})
            shutil.rmtree(ctx['root'], ignore_errors=True)
    # K5：Git とスナップショットで成型した出力が一致する
    k5 = [sid for sid, o in outputs.items() if len(o) > 1 and len({json.dumps(v, sort_keys=True) for v in o.values()}) > 1]
    # K1 ・ K2：対応表
    cov = json.load(open(os.path.join(HERE, 'coverage.json')))
    model = json.load(open(os.path.join(HERE, 'model', 'model.json')))
    ids = [f"{u[0]}:{k}" for u in model['use_cases'] for k in ('事後', '失敗時')] + [i[0] for i in model['invariants']]
    unclassified = [i for i in ids if i not in cov['rows']]
    bad_reason = [i for i, r in cov['rows'].items() if r['class'] in ('確かめない', '一部') and not (r.get('why') and r.get('move_when'))]
    traced = {t for sc in scenarios for t in sc.get('traces', [])}
    untraced = [i for i, r in cov['rows'].items() if r['class'] in ('場面', '一部', 'アダプター', '静的') and i not in traced]
    no_trace = [sc['id'] for sc in scenarios if not sc.get('traces')]
    failed = [r for r in results if not r['pass']]
    report = {'scenarios': len(scenarios), 'runs': len(results), 'failed': failed,
              'K1_unclassified': unclassified, 'K1_missing_reason': bad_reason, 'K2_untraced_ids': untraced, 'K2_scenarios_without_trace': no_trace,
              'K5_adapter_mismatch': k5, 'classes': {c: sum(1 for r in cov['rows'].values() if r['class'] == c) for c in ('場面', '静的', 'アダプター', '一部', '確かめない')}}
    json.dump(report, open(os.path.join(HERE, 'report.json'), 'w'), ensure_ascii=False, indent=1)
    json.dump(outputs, open(os.path.join(HERE, 'outputs.json'), 'w'), ensure_ascii=False, indent=1)
    bad = failed or unclassified or bad_reason or untraced or no_trace or k5
    print(f"場面 {len(scenarios)} ・ 実行 {len(results)} ・ 不合格 {len(failed)} ・ 区分なし {len(unclassified)} ・ 理由なし {len(bad_reason)} ・ 場面の無い id {len(untraced)} ・ アダプターの不一致 {len(k5)}")
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
