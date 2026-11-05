"""開発シミュレーターの材料を集める（作業用。コミットしない）。

test-suite/scenarios/ の場面を、完成した段の順に、カバレッジを取る実行ファイルで1コマンドずつ動かす。
コマンドごとに、作業ディレクトリの前後の写し（JSON Schema ・ インスタンス ・ DB）、入力 ・ 出力 ・ 終了コード、
通った関数と行を sim.json に書く。
"""
import glob, json, os, re, shutil, sqlite3, subprocess, sys, tempfile

H = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(H, '../../../../../..'))
WS = os.path.join(REPO, 'semantic-agent-schema')
TARGET = os.path.join(WS, 'target', 'sim')
BIN = os.path.join(TARGET, 'debug', 'semantic-agent-schema')
SYSROOT = subprocess.run(['rustc', '--print', 'sysroot'], capture_output=True, text=True, cwd=WS).stdout.strip()
LLVM = glob.glob(os.path.join(SYSROOT, 'lib/rustlib/*/bin'))[0]

# 場面の id の頭 → 段（計画の段の名前）。段が増えたらここに1行足す
# (id の頭の並び, ステップ, 中身, 状態 done ・ doing)
STAGES = [
    (('S-DECL',), 'ステップ1', 'Abstract と Concrete の宣言', 'done'),
    (('S-RULE', 'S-INST'), 'ステップ2', 'rule ごとの検証とインスタンス', 'done'),
    (('S-CHANGE',), 'ステップ3a', '変更の記録と人の承認（手元の DB）', 'done'),
    (('S-SUMMARY', 'S-MERGE'), 'ステップ3b', '変更のまとめとコミット ・ 取り込み', 'done'),
    ((), 'ステップ4', 'AI と外のツールに rule を判定させる', 'todo'),
]
# 目次に並べる順（話が流れる順）。ここに無いシナリオは id の順で後ろに付く
ORDER = ['S-DECL-CREATE', 'S-DECL-CREATE-EMPTY', 'S-DECL-PATCH', 'S-DECL-VALID', 'S-DECL-BADRULE', 'S-DECL-REFUSE-ID',
         'S-DECL-EXT', 'S-DECL-EXT-CLASH', 'S-DECL-GLOSSARY', 'S-DECL-GLOSSARY-EMPTY', 'S-DECL-TERM', 'S-DECL-TERM-MISSING', 'S-DECL-GLOSSARY-TERM-SHAPE', 'S-DECL-NESTED-XPROMPT',
         'S-RULE-RESULTS', 'S-RULE-OUTPUT-INLINE', 'S-RULE-LEVEL1',
         'S-INST-CREATE', 'S-INST-REQUIRED', 'S-INST-EMPTY', 'S-INST-PATH', 'S-INST-NO-CONCRETE',
         'S-INST-ETAG', 'S-INST-PATCH', 'S-INST-STALE', 'S-INST-REFUSE-ID', 'S-INST-ACCEPTED',
         'S-CHANGE-RECORD', 'S-CHANGE-NO-INTENT', 'S-CHANGE-HUMAN-WAIT', 'S-CHANGE-APPROVE', 'S-CHANGE-NO-RECORD',
         'S-SUMMARY-CLEAN', 'S-SUMMARY-UNRECORDED', 'S-SUMMARY-NO-CHANGE', 'S-SUMMARY-WORKTREE', 'S-SUMMARY-STASH', 'S-SUMMARY-BRANCH-RENAME', 'S-MERGE-VALIDATE']
LAYERS = [('inbound', 'crates/inbound/'), ('application', 'crates/application/'), ('domain', 'crates/domain/'), ('outbound', 'crates/outbound/')]


def build():
    env = dict(os.environ, RUSTFLAGS='-C instrument-coverage', CARGO_TARGET_DIR=TARGET,
               LLVM_PROFILE_FILE=os.path.join(tempfile.gettempdir(), 'sas-build-%p.profraw'))
    subprocess.run(['cargo', 'build', '-q', '-p', 'semantic-agent-schema-cli', '--bin', 'semantic-agent-schema'], cwd=WS, env=env, check=True)


def kind_of(doc):
    s = doc.get('$schema', '') if isinstance(doc, dict) else ''
    if s.endswith(':semantic-abstract-schema:v1'):
        return 'schema'
    if s.endswith(':glossary:v1'):
        return 'glossary'
    if s.endswith(':change-summary:v1'):
        return 'summary'
    return 'instance'


def snapshot(work):
    """作業ディレクトリの写し：JSON の文書と DB の表"""
    files, dbs = {}, {}
    for dp, dn, fn in os.walk(work):
        # Git の中は手元の DB（.git/semantic-agent-schema/）だけを見る
        dn[:] = [d for d in dn if d != '.git' or os.path.relpath(dp, work) != '.']
        if os.path.isdir(os.path.join(dp, '.git', 'semantic-agent-schema')) and dp == work:
            fn = list(fn) + [os.path.join('.git', 'semantic-agent-schema', x) for x in os.listdir(os.path.join(dp, '.git', 'semantic-agent-schema'))]
        for f in sorted(fn):
            p = os.path.join(dp, f)
            rel = os.path.relpath(p, work)
            if f.endswith('.profraw'):
                continue
            if f.endswith(('.db', '.sqlite', '.sqlite3')):
                con = sqlite3.connect(p)
                tables = {}
                for (t,) in con.execute("select name from sqlite_master where type='table' order by name"):
                    cur = con.execute(f'select * from "{t}"')
                    cols = [c[0] for c in cur.description]
                    tables[t] = {'columns': cols, 'rows': [list(r) for r in cur.fetchall()]}
                con.close()
                dbs[rel] = tables
            elif f.endswith('.json'):
                text = open(p).read()
                try:
                    doc = json.loads(text)
                except ValueError:
                    doc = None
                files[rel] = {'kind': kind_of(doc) if doc is not None else 'broken', 'text': text}
    return {'files': files, 'dbs': dbs}


def run_cmd(work, args, prof, tool=None):
    env = dict(os.environ, HOME=os.path.join(work, '.home'), XDG_CONFIG_HOME=os.path.join(work, '.home/config'),
               XDG_DATA_HOME=os.path.join(work, '.home/data'), XDG_STATE_HOME=os.path.join(work, '.home/state'),
               XDG_CACHE_HOME=os.path.join(work, '.home/cache'), LLVM_PROFILE_FILE=prof, GIT_CONFIG_NOSYSTEM='1',
               GIT_AUTHOR_NAME='t', GIT_AUTHOR_EMAIL='t@example.com', GIT_COMMITTER_NAME='t', GIT_COMMITTER_EMAIL='t@example.com')
    r = subprocess.run([tool or BIN] + args, cwd=work, env=env, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return r.returncode, r.stdout, r.stderr


def coverage(prof):
    """通った関数（crates/ の中だけ）と、通った行。クロージャは、それを含む関数にまとめる"""
    data = prof.replace('.profraw', '.profdata')
    subprocess.run([os.path.join(LLVM, 'llvm-profdata'), 'merge', '-sparse', prof, '-o', data], check=True)
    out = subprocess.run([os.path.join(LLVM, 'llvm-cov'), 'export', '--format=text', '--instr-profile', data, BIN],
                         capture_output=True, text=True, check=True).stdout
    named, closures = [], []
    for fn in json.loads(out)['data'][0]['functions']:
        files = [f for f in fn['filenames'] if '/semantic-agent-schema/crates/' in f]
        if not files:
            continue
        path = os.path.relpath(files[0], WS)
        if '/tests/' in path:
            continue
        lines = {ln for r in fn['regions'] if r[4] > 0 and r[7] == 0 for ln in range(r[0], r[2] + 1)}
        item = {'file': path, 'start': min(r[0] for r in fn['regions']), 'end': max(r[2] for r in fn['regions']),
                'count': fn['count'], 'lines': lines}
        (closures if fn['name'].startswith('_RNC') else named).append(item)
    hit = {}
    for f in named:
        k = (f['file'], f['start'])
        if k in hit:
            hit[k]['count'] += f['count']; hit[k]['lines'] |= f['lines']
        else:
            hit[k] = dict(f)
    for c in closures:
        if c['count'] == 0:
            continue
        for f in hit.values():
            if f['file'] == c['file'] and f['start'] <= c['start'] and c['end'] <= f['end']:
                f['lines'] |= c['lines']
    res = []
    for f in hit.values():
        if f['count'] == 0:
            continue
        f['name'] = fn_name(f['file'], f['start'])
        f['lines'] = sorted(f['lines'])
        res.append(f)
    res.sort(key=lambda h: (layer_rank(h['file']), h['file'], h['start']))
    return res


_SRC = {}
def fn_name(path, start):
    """開始行から fn の名前と、囲む impl の型を読む"""
    if path not in _SRC:
        _SRC[path] = open(os.path.join(WS, path)).read().split('\n')
    src = _SRC[path]
    name = '?'
    for i in range(start - 1, max(start - 6, -1), -1):
        m = re.search(r'\bfn\s+(\w+)', src[i])
        if m:
            name = m.group(1)
            line = src[i]
            break
    else:
        return name
    indent = len(line) - len(line.lstrip())
    if indent == 0:
        return name
    for j in range(i - 1, -1, -1):
        t = src[j]
        if t.strip() and len(t) - len(t.lstrip()) < indent:
            m = re.match(r'impl(?:<[^>]*>)?\s+(?:([\w:]+)(?:<[^>]*>)?\s+for\s+)?([\w:]+)', t.strip())
            if m:
                trait, ty = m.group(1), m.group(2)
                return f'{ty}::{name}' + (f'（{trait}）' if trait else '')
            return name
    return name


def layer_rank(path):
    for i, (_, pre) in enumerate(LAYERS):
        if path.startswith(pre):
            return i
    return len(LAYERS)


def stage_of(sid):
    for pres, no, title, _ in STAGES:
        if any(sid.startswith(p) for p in pres):
            return no, title
    return 'ステップ?', ''


def main():
    build()
    scenarios = []
    for path in sorted(glob.glob(os.path.join(WS, 'test-suite/scenarios/*.json'))):
        s = json.load(open(path))
        no, title = stage_of(s['id'])
        work = tempfile.mkdtemp(prefix='sas-sim-')
        os.makedirs(os.path.join(work, '.home'))
        for rel, body in s.get('given', {}).get('files', {}).items():
            src = body[len('@fixtures/'):] if body.startswith('@fixtures/') else None
            dest = os.path.join(work, rel)
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            text = open(os.path.join(WS, 'test-suite/fixtures', src)).read() if src else body
            open(dest, 'w').write(text)
        if s.get('given', {}).get('repo'):
            run_cmd(work, ['init', '-q', '-b', 'main'], os.devnull, 'git')
        steps = []
        phases = [('given', c) for c in s.get('given', {}).get('steps', [])] + [('when', s['when'])] + [('after', c) for c in s.get('after', [])]
        prof_dir = tempfile.mkdtemp(prefix='sas-prof-')
        for n, (phase, cmd) in enumerate(phases):
            before = snapshot(work)
            prof = os.path.join(prof_dir, f'{n}.profraw')
            is_git = 'git' in cmd
            args = ['git'] + cmd['git'] if is_git else cmd['args']
            code, out, err = run_cmd(os.path.join(work, cmd.get('cwd', '.')), cmd['git'] if is_git else cmd['args'], prof, 'git' if is_git else None)
            after = snapshot(work)
            try:
                parsed = json.loads(out)
            except ValueError:
                parsed = None
            steps.append({'phase': phase, 'args': args, 'cwd': cmd.get('cwd'), 'exit': code, 'stdout': out, 'stderr': err, 'output': parsed,
                          'before': before, 'after': after, 'code': [] if is_git else coverage(prof)})
        shutil.rmtree(work)
        shutil.rmtree(prof_dir)
        scenarios.append({'id': s['id'], 'name': s.get('name', s['id']), 'title': s['title'], 'why': s.get('why', ''), 'traces': s['traces'], 'stage': no, 'stage_title': title,
                          'given_files': sorted(s.get('given', {}).get('files', {}).keys()), 'repo': bool(s.get('given', {}).get('repo')), 'then': s['then'],
                          'unchanged': s.get('unchanged', []), 'steps': steps})
    order = {no: i for i, (_, no, _, _) in enumerate(STAGES)}
    scenarios.sort(key=lambda x: (order.get(x['stage'], 99), ORDER.index(x['id']) if x['id'] in ORDER else 999, x['id']))
    # 通った関数のソース（関数ごとに1回だけ持つ）
    sources = {}
    for sc in scenarios:
        for st in sc['steps']:
            for h in st['code']:
                if h['file'] not in sources:
                    sources[h['file']] = open(os.path.join(WS, h['file'])).read().split('\n')
    # 根拠：使う場面と不変条件の名前（trace.json）と、Abstract の rule の本文と理由
    trace = json.load(open(os.path.join(WS, 'test-suite/trace.json')))
    names = {u['id']: u['name'] for u in trace['use_cases']}
    names.update({i['id']: i['name'] for i in trace['invariants']})
    rules = {}
    def collect(o):
        if isinstance(o, dict):
            for k, v in o.items():
                if k in ('x-prompt', 'prompt') and isinstance(v, dict):
                    for r in v.get('write', []):
                        rules[r['id']] = {'rule': r['rule'], 'why': r['why'], 'by': r['by']}
                collect(v)
        elif isinstance(o, list):
            for v in o:
                collect(v)
    collect(json.load(open(os.path.join(WS, 'schemas/v1/semantic-abstract-schema.json'))))
    commit = subprocess.run(['git', 'log', '-1', '--format=%h %s'], cwd=WS, capture_output=True, text=True).stdout.strip()
    json.dump({'commit': commit, 'names': names, 'rules': rules, 'stages': [{'no': no, 'title': t, 'status': st} for _, no, t, st in STAGES], 'scenarios': scenarios, 'sources': sources},
              open(os.path.join(H, 'sim.json'), 'w'), ensure_ascii=False)
    print('scenarios', len(scenarios), 'steps', sum(len(s['steps']) for s in scenarios), 'files', len(sources))


if __name__ == '__main__':
    sys.exit(main())
