"""参照の実装のアダプター。基盤（base.py）はこのモジュールの名前を知らず、ポートの形だけで呼ぶ。"""
import json, os, shutil, sqlite3, subprocess

SKIP_DIRS = {'.git'}


def walk(root):
    out = {}
    for d, dirs, files in os.walk(root):
        dirs[:] = [x for x in dirs if x not in SKIP_DIRS]
        for f in files:
            p = os.path.join(d, f); out[os.path.relpath(p, root)] = p
    return out


def read(p):
    if p is None or not os.path.exists(p): return None
    b = open(p, 'rb').read()
    try: return b.decode('utf-8')
    except UnicodeDecodeError: return '\0binary'


class Unreachable(Exception):
    pass


# ---------------- 変更の仕組み：Git
class GitChanges:
    name = 'git'

    def __init__(self, work):
        self.w = work

    def _g(self, *a, ok=False, inp=None):
        r = subprocess.run(['git', '-C', self.w, *a], capture_output=True, text=True, input=inp)
        if r.returncode and not ok: raise RuntimeError(r.stderr)
        return r

    def init(self, files):
        self._g('init', '-q'); self._g('config', 'user.name', 'tester'); self._g('config', 'user.email', 't@example.invalid')
        for p, c in files.items():
            fp = os.path.join(self.w, p); os.makedirs(os.path.dirname(fp), exist_ok=True); open(fp, 'w').write(c)
        self._g('add', '-A'); self._g('commit', '-q', '--allow-empty', '-m', 'base')

    def last_confirmed(self):
        return {'ref': self._g('rev-parse', 'HEAD').stdout.strip()}

    def _tree(self, ref):
        r = self._g('ls-tree', '-r', '--name-only', ref, ok=True)
        if r.returncode: raise Unreachable(ref)
        return [x for x in r.stdout.split('\n') if x]

    def _show(self, ref, p):
        r = subprocess.run(['git', '-C', self.w, 'show', f'{ref}:{p}'], capture_output=True)
        if r.returncode: return None
        try: return r.stdout.decode('utf-8')
        except UnicodeDecodeError: return '\0binary'

    def before_after(self, before, after=None):
        try:
            bt = self._tree(before)
            at = self._tree(after) if after else None
        except Unreachable as e:
            return {'unreachable': str(e)}
        names = sorted(set(bt) | (set(at) if at is not None else set(walk(self.w))))
        items = []
        for n in names:
            b = self._show(before, n) if n in bt else None
            a = (self._show(after, n) if n in at else None) if at is not None else read(os.path.join(self.w, n))
            if b != a: items.append({'target': n, 'before': b, 'after': a})
        return {'items': items}

    def unit(self, ref):
        r = self._g('log', '-1', '--format=%P%n%an%n%aI', ref).stdout.split('\n')
        return {'before': (r[0].split() or [None])[0], 'after': ref,
                'author': {'value': r[1], 'source': 'self-declared'}, 'time': {'value': r[2], 'source': 'self-declared'}}

    def deleted_ids(self):
        return {'ids': [], 'searched': 'all'}

    def confirm(self, paths, message):
        exist = [p for p in paths if os.path.exists(os.path.join(self.w, p))]
        tracked = set(self._tree('HEAD'))
        gone = [p for p in paths if p not in exist and p in tracked]
        if exist: self._g('add', '--', *exist)
        if gone: self._g('rm', '-q', '--cached', '--', *gone)
        targets = exist + gone
        if not targets or not self._g('diff', '--cached', '--name-only', '--', *targets).stdout.strip():
            return {'nothing_changed': True}
        self._g('commit', '-q', '--only', '-m', message, '--', *targets)
        return {'ref': self._g('rev-parse', 'HEAD').stdout.strip()}


# ---------------- 変更の仕組みが無い場所：スナップショット（前と後のディレクトリ）
class SnapshotChanges:
    name = 'snapshot'

    def __init__(self, work, store):
        self.w, self.s = work, store

    def init(self, files):
        for p, c in files.items():
            fp = os.path.join(self.w, p); os.makedirs(os.path.dirname(fp), exist_ok=True); open(fp, 'w').write(c)
        shutil.copytree(self.w, os.path.join(self.s, 's0'))

    def _refs(self):
        return sorted((d for d in os.listdir(self.s) if d.startswith('s')), key=lambda d: int(d[1:]))

    def last_confirmed(self):
        return {'ref': self._refs()[-1]}

    def before_after(self, before, after=None):
        bd = os.path.join(self.s, before)
        if not os.path.isdir(bd): return {'unreachable': before}
        ad = os.path.join(self.s, after) if after else self.w
        if after and not os.path.isdir(ad): return {'unreachable': after}
        bf, af = walk(bd), walk(ad)
        items = []
        for n in sorted(set(bf) | set(af)):
            b, a = read(bf.get(n)), read(af.get(n))
            if b != a: items.append({'target': n, 'before': b, 'after': a})
        return {'items': items}

    def unit(self, ref):
        refs = self._refs(); i = refs.index(ref)
        return {'before': refs[i - 1] if i else None, 'after': ref,
                'author': {'value': '', 'source': 'none'}, 'time': {'value': '', 'source': 'none'}}

    def deleted_ids(self):
        return {'ids': [], 'searched': 'all'}

    def confirm(self, paths, message):
        last = self._refs()[-1]; new = f's{int(last[1:]) + 1}'
        nd = os.path.join(self.s, new); shutil.copytree(os.path.join(self.s, last), nd)
        changed = False
        for p in paths:
            src, dst = os.path.join(self.w, p), os.path.join(nd, p)
            if os.path.exists(src):
                if read(src) != read(dst if os.path.exists(dst) else None):
                    os.makedirs(os.path.dirname(dst), exist_ok=True); shutil.copy(src, dst); changed = True
            elif os.path.exists(dst):
                os.remove(dst); changed = True
        if not changed:
            shutil.rmtree(nd); return {'nothing_changed': True}
        return {'ref': new}


# ---------------- 手元の DB：SQLite
class SqliteSessions:
    def __init__(self, path, ddl):
        self.path = path
        c = self.conn(); c.executescript(open(ddl).read()); c.close()

    def conn(self, fk=True):
        c = sqlite3.connect(self.path, timeout=5, isolation_level=None)
        if fk: c.execute('PRAGMA foreign_keys=ON')
        return c

    @staticmethod
    def row(r):
        if r is None: return None
        tool, id_, pur, pt, pid, role, state, last = r
        d = {'session': {'tool': tool, 'id': id_}, 'state': state, 'last_used_at': last}
        if pur is not None: d['purpose_ref'] = pur
        if pid is not None: d['parent'] = {'tool': pt, 'id': pid}
        if role is not None: d['role'] = role
        return d

    def slot_where(self, purpose, parent, role):
        if purpose is None: return 'purpose_ref IS NULL', []
        if role is None: return 'purpose_ref = ? AND role IS NULL', [purpose]
        return 'parent_tool = ? AND parent_id = ? AND role = ?', [parent['tool'], parent['id'], role]

    def open_in_slot(self, purpose, parent, role, c=None):
        c = c or self.conn(); w, a = self.slot_where(purpose, parent, role)
        return self.row(c.execute(f"SELECT * FROM sessions WHERE state='open' AND {w}", a).fetchone())

    def register(self, new, close, expected):
        c = self.conn(); c.execute('BEGIN IMMEDIATE')
        try:
            now_open = self.open_in_slot(new.get('purpose_ref'), new.get('parent'), new.get('role'), c)
            exp = expected and expected['session']
            if (now_open and now_open['session']) != exp:
                c.execute('ROLLBACK'); return {'switched_to': now_open}
            for s in close:
                c.execute("UPDATE sessions SET state='closed' WHERE tool=? AND id=?", (s['tool'], s['id']))
            p = new.get('parent') or {}
            c.execute('INSERT INTO sessions VALUES (?,?,?,?,?,?,?,?)', (new['session']['tool'], new['session']['id'], new.get('purpose_ref'),
                      p.get('tool'), p.get('id'), new.get('role'), 'open', new['last_used_at']))
            c.execute('COMMIT'); return {'registered': new}
        except sqlite3.DatabaseError as e:
            c.execute('ROLLBACK'); return {'refused': str(e)}

    def close(self, s):
        self.conn().execute("UPDATE sessions SET state='closed' WHERE tool=? AND id=?", (s['tool'], s['id']))

    def add_index(self, s, change_id):
        self.conn().execute('INSERT INTO session_changes VALUES (?,?,?)', (s['tool'], s['id'], change_id))

    def index_of(self, s):
        return [r[0] for r in self.conn().execute('SELECT change_id FROM session_changes WHERE tool=? AND id=? ORDER BY change_id', (s['tool'], s['id']))]

    def get(self, s):
        return self.row(self.conn().execute('SELECT * FROM sessions WHERE tool=? AND id=?', (s['tool'], s['id'])).fetchone())

    def all(self):
        return [self.row(r) for r in self.conn().execute('SELECT * FROM sessions ORDER BY tool, id')]

    def all_index(self):
        return [{'session': {'tool': r[0], 'id': r[1]}, 'change_id': r[2]} for r in self.conn().execute('SELECT * FROM session_changes ORDER BY 1,2,3')]


# ---------------- AI ツールのセッション（場面が答えを決める）
class FakeAiSessions:
    def __init__(self, table):
        self.t = table

    def inspect(self, tool, id):
        return self.t.get(f'{tool}/{id}', {'resumable': False, 'cacheLifetime': 'unknown', 'lastUsedAt': '', 'historyPercent': 0})


# ---------------- 判定させる AI（場面が答えを決める。本物の AI は呼ばない）
class StubJudge:
    def __init__(self, answers):
        self.a, self.calls = answers, []

    def ask(self, call):
        self.calls.append(call)
        res = self.a.get(call['inputs']['intent'], 'verified')
        out = {'result': res, 'grounds': [{'at': '/intent', 'text': f'意図と変更の照らし：{res}'}],
               'meaning': {'whole': call['inputs']['intent'], 'per_target': [{'target': t, 'text': '変更', 'at': []} for t in call['inputs']['targets']]}}
        if res == 'unverified': out['question'] = '何が分かれば決められるか'
        return out


# ---------------- 設定のファイル
class FileSettings:
    def __init__(self, user_path, work):
        self.paths = [('user', user_path), ('project', os.path.join(work, '.semantic-agent-schema', 'settings.json')),
                      ('local', os.path.join(work, '.semantic-agent-schema', 'settings.local.json'))]

    def read(self):
        merged = {}
        for _, p in self.paths:
            if os.path.exists(p): merged.update(json.load(open(p)).get('session', {}))
        return merged
