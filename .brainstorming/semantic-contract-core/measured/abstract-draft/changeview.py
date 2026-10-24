"""変更を見せる（試作）。基盤は「変更の前後を受け取る」ポートだけを通して差分を受け取り、成型して描く。

使い方：
  python3 changeview.py git <作業ツリー> [変更の単位（既定 HEAD）] > out.html
  python3 changeview.py snapshot <前のディレクトリ> <後のディレクトリ> > out.html
変更の記録は .semantic-agent-schema/changes/*.json。同じ変更の単位に入った記録の意図を、その単位の変更すべてに付ける（1単位＝1意図）。
"""
import difflib, html, json, os, subprocess, sys

REC_DIR = '.semantic-agent-schema/changes/'


# ── 駆動される側のポート：変更の前後を受け取る ──────────────────────
# before(ref) は変更の単位 ref の、対象ごとの前の姿と後の姿を返す。ref は基盤が解釈しない文字列。
# 返す形：[{'target': パス, 'before': 本文 or None, 'after': 本文 or None}]（None は「無い」）

class GitAdapter:
    def __init__(self, repo):
        self.repo = repo

    def _git(self, *a, ok=False):
        r = subprocess.run(['git', '-C', self.repo, *a], capture_output=True, text=True)
        return r.stdout if r.returncode == 0 else (None if ok else sys.exit(r.stderr))

    def changes(self, ref):
        names = [n for n in self._git('diff', '--name-only', ref + '^', ref).split('\n') if n]
        return [{'target': n, 'before': self._git('show', f'{ref}^:{n}', ok=True),
                 'after': self._git('show', f'{ref}:{n}', ok=True)} for n in names]


class SnapshotAdapter:
    """変更の仕組みを持たない場所：前と後のディレクトリを比べる"""
    def __init__(self, before_dir, after_dir):
        self.dirs = (before_dir, after_dir)

    def _files(self, d):
        return {os.path.relpath(os.path.join(r, f), d) for r, _, fs in os.walk(d) for f in fs}

    def _read(self, d, n):
        p = os.path.join(d, n)
        return open(p, encoding='utf-8').read() if os.path.exists(p) else None

    def changes(self, ref):
        b, a = self.dirs
        out = []
        for n in sorted(self._files(b) | self._files(a)):
            before, after = self._read(b, n), self._read(a, n)
            if before != after:
                out.append({'target': n, 'before': before, 'after': after})
        return out


# ── 基盤：受け取った前後を成型する（アダプターを知らない） ─────────────

def text_rows(before, after):
    """行の並び [(印, 本文)] と、まとまりの数"""
    a, b = (before or '').splitlines(), (after or '').splitlines()
    rows, n = [], 0
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b).get_opcodes():
        if tag == 'equal':
            rows += [(' ', l) for l in b[j1:j2]]
        else:
            rows += [('-', l) for l in a[i1:i2]] + [('+', l) for l in b[j1:j2]] + [('@', None)]
            n += 1
    return rows, n


def json_fields(before, after, at=''):
    """JSON の欄の単位の変更 [(pointer, 前, 後)]"""
    if isinstance(before, dict) and isinstance(after, dict):
        out = []
        for k in list(before) + [k for k in after if k not in before]:
            out += json_fields(before.get(k), after.get(k), f'{at}/{k}')
        return out
    return [] if before == after else [(at or '/', before, after)]


def shape(changes):
    recs = [json.loads(c['after']) for c in changes if c['target'].startswith(REC_DIR) and c['after']]
    targets = [c for c in changes if not c['target'].startswith(REC_DIR) and not c['target'].endswith('.html')]
    intent = ' ・ '.join(r['intent'] for r in recs) or None
    return recs, targets, intent


def render(port, ref):
    recs, targets, intent = shape(port.changes(ref))
    css = ('body{font:14px/1.6 sans-serif;margin:16px;max-width:980px}pre{font:12px/1.5 monospace;border:1px solid #ccc;border-radius:8px;padding:6px 0;overflow-x:auto}'
           '.l{display:block;padding:0 10px;white-space:pre}.d{background:#fbe3e3}.a{background:#ddf3e4}.m{background:#fff1a8;font-family:sans-serif}'
           '.e{background:#f8e1e1;font-family:sans-serif}table{border-collapse:collapse}td,th{border:1px solid #ccc;padding:2px 8px;font:12px monospace}')
    out = [f'<meta charset="utf-8"><title>変更を見せる（試作）</title><style>{css}</style>',
           f'<h1>変更を見せる（試作）</h1><p>アダプター {type(port).__name__} ・ 変更の単位 {html.escape(ref)} ・ 記録 {len(recs)} 件 ・ 対象 {len(targets)} 件</p>',
           '<h2>目的 → 意図</h2><ul>' + ''.join(f'<li>{html.escape(r["purpose_ref"])} → {html.escape(r["intent"])}</li>' for r in recs) + '</ul>']
    label = (f'<span class="l m">↑ 意図：{html.escape(intent)}</span>' if intent else '<span class="l e">↑ 記録に無い変更</span>')
    for c in targets:
        out.append(f'<h2>{html.escape(c["target"])}</h2>')
        if c['target'].endswith('.json') and c['before'] and c['after']:
            rows = json_fields(json.loads(c['before']), json.loads(c['after']))
            out.append('<table><tr><th>欄</th><th>前</th><th>後</th></tr>' + ''.join(
                f'<tr><td>{html.escape(p)}</td><td class="d">{html.escape(json.dumps(b, ensure_ascii=False))}</td><td class="a">{html.escape(json.dumps(a, ensure_ascii=False))}</td></tr>' for p, b, a in rows) + '</table>' + label)
        rows, _ = text_rows(c['before'], c['after'])
        out.append('<pre>' + ''.join(label if m == '@' else f'<span class="l {dict(zip(" -+", ["", "d", "a"]))[m]}">{html.escape(m + " " + l)}</span>' for m, l in rows) + '</pre>')
    print('\n'.join(out))


if __name__ == '__main__':
    kind, *args = sys.argv[1:]
    if kind == 'git':
        render(GitAdapter(args[0]), args[1] if len(args) > 1 else 'HEAD')
    else:
        render(SnapshotAdapter(*args), 'snapshot')
