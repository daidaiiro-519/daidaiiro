"""変更の記録の契約の試作：1つのコミットの差分を、変更の記録の場所で意図に振り分けて、ファイルの全体に印を付けて描く。

使い方：python3 changeview.py <git の作業ツリー> [コミット（既定 HEAD）] > out.html
記録は .semantic-agent-schema/changes/*.json。場所は find（後の本文に現れる文字列）か pointer（JSON の欄）。
"""
import difflib, html, json, subprocess, sys

REC_DIR = '.semantic-agent-schema/changes/'


def git(repo, *a):
    return subprocess.run(['git', '-C', repo, *a], capture_output=True, text=True, check=True).stdout


def show(repo, rev, path):
    r = subprocess.run(['git', '-C', repo, 'show', f'{rev}:{path}'], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ''


def hunks(before, after):
    """差分のまとまり：(後の本文での行の範囲, 行の並び[(印, 本文)])。印は ' ' '-' '+'"""
    a, b = before.splitlines(), after.splitlines()
    rows, out = [], []
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b).get_opcodes():
        if tag == 'equal':
            rows += [(' ', l, j1 + k) for k, l in enumerate(b[j1:j2])]
        else:
            rows += [('-', l, j1) for l in a[i1:i2]] + [('+', l, j1 + k) for k, l in enumerate(b[j1:j2])]
            out.append((j1, max(j2, j1 + 1)))
    return rows, out


def locate(after, at):
    """記録の場所を、後の本文の行番号にする"""
    lines = after.splitlines()
    key = at.get('find') or ('"' + at['pointer'].strip('/').split('/')[-1] + '"' if 'pointer' in at else None)
    return [i for i, l in enumerate(lines) if key and key in l]


def main(repo, rev='HEAD'):
    parent = rev + '^'
    names = [n for n in git(repo, 'diff', '--name-only', parent, rev).split('\n') if n]
    recs = {n: json.loads(show(repo, rev, n)) for n in names if n.startswith(REC_DIR)}
    files = [n for n in names if not n.startswith(REC_DIR)]
    out = ['<meta charset="utf-8"><title>変更を見せる（試作）</title><style>body{font:14px/1.6 sans-serif;margin:16px;max-width:980px}'
           'pre{font:12px/1.5 monospace;border:1px solid #ccc;border-radius:8px;padding:6px 0;overflow-x:auto}'
           '.l{display:block;padding:0 10px;white-space:pre}.d{background:#fbe3e3}.a{background:#ddf3e4}.h{border-left:4px solid #0d5c55}.x{border-left:4px solid #9b2c2c}'
           '.m{background:#fff1a8;font-family:sans-serif}.e{background:#f8e1e1;font-family:sans-serif;border-radius:6px;padding:4px 8px}</style>']
    out.append(f'<h1>変更を見せる（試作）</h1><p>コミット {html.escape(git(repo, "rev-parse", "--short", rev).strip())} ・ 変更の記録 {len(recs)} 件 ・ ファイル {len(files)} 件</p>')
    exceptions = []
    body = []
    for f in files:
        before, after = show(repo, parent, f), show(repo, rev, f)
        rows, hs = hunks(before, after)
        owner = {}
        for rid, r in recs.items():
            for it in r['items']:
                if it['target'] != f:
                    continue
                for ln in locate(after, it['at']):
                    for k, (s, e) in enumerate(hs):
                        if s - 1 <= ln <= e:
                            owner.setdefault(k, []).append(r['intent'])
        def label(k):
            if k in owner:
                return f'<span class="l m">↑ 意図：{html.escape(" ・ ".join(sorted(set(owner[k]))))}</span>'
            return '<span class="l e">↑ 記録に無い変更</span>'
        body.append(f'<h2>{html.escape(f)}</h2><pre>')
        for k, (s, e) in enumerate(hs):
            if k not in owner:
                exceptions.append(f'{f} の {s + 1} 行目付近：どの記録の場所にも重ならない（記録に無い）')
        pre, last = [], None
        for mark, line, j in rows:
            k = None if mark == ' ' else next((k for k, (s, e) in enumerate(hs) if s <= j < e or j == s), None)
            if last is not None and k != last:
                pre.append(label(last))
            last = k
            cls = {'-': 'd', '+': 'a', ' ': ''}[mark] + ('' if mark == ' ' else (' h' if k in owner else ' x'))
            pre.append(f'<span class="l {cls}">{html.escape(mark + " " + line)}</span>')
        if last is not None:
            pre.append(label(last))
        body.append(''.join(pre) + '</pre>')
    out.append('<h2>例外</h2>' + (''.join(f'<div class="e">{html.escape(x)}</div>' for x in exceptions) or '<p>0件</p>'))
    out.append('<h2>目的 → 意図</h2><ul>' + ''.join(f'<li>{html.escape(r["purpose_ref"])} → {html.escape(r["intent"])}</li>' for r in recs.values()) + '</ul>')
    out += body
    print('\n'.join(out))


if __name__ == '__main__':
    main(*sys.argv[1:])
