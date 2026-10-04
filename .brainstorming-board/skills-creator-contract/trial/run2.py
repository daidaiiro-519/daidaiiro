#!/usr/bin/env python3
"""テストケースを、Skill の tool.json の実行コマンドで実行する試作（最終の形式）。

使い方: run2.py <テストケースのフォルダ> <Skill のフォルダ> [--write] [--only 名前]
--write は、リファレンス実装の出力から期待値を書き出す（ignore と setup は保つ）
"""
import json, os, subprocess, sys, glob, shutil, tempfile

def command_of(skill):
    cli = json.load(open(os.path.join(skill, 'tool.json')))['cli']
    proj = '/home/daidaiiro/workspace/daidaiiro'
    return [cli['command'].replace('${CLAUDE_PROJECT_DIR:-.}', proj)] + cli.get('args', [])

def call(cmd, args, cwd):
    p = subprocess.run(cmd + args + ['--json'], cwd=cwd, capture_output=True, text=True)
    try: out = json.loads(p.stdout)
    except Exception: out = None
    return p.returncode, out

def drop(v, pointer):
    parts = [p.replace('~1', '/').replace('~0', '~') for p in pointer.split('/')[1:]]
    cur = v
    for p in parts[:-1]:
        if isinstance(cur, dict): cur = cur.get(p)
        elif isinstance(cur, list) and p.isdigit() and int(p) < len(cur): cur = cur[int(p)]
        else: return
    last = parts[-1]
    if isinstance(cur, dict): cur.pop(last, None)
    elif isinstance(cur, list) and last.isdigit() and int(last) < len(cur): cur.pop(int(last))

def diff(a, b, at=''):
    if type(a) != type(b): return at or '/'
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b: return f'{at}/{k}'
            d = diff(a[k], b[k], f'{at}/{k}')
            if d: return d
        return None
    if isinstance(a, list):
        if len(a) != len(b): return f'{at}（件数 {len(a)} と {len(b)}）'
        for i, (x, y) in enumerate(zip(a, b)):
            d = diff(x, y, f'{at}/{i}')
            if d: return d
        return None
    return None if a == b else at or '/'

def main():
    cases, skill = sys.argv[1], sys.argv[2]
    write = '--write' in sys.argv
    only = sys.argv[sys.argv.index('--only') + 1] if '--only' in sys.argv else None
    cmd = command_of(skill)
    ok = ng = 0
    for f in sorted(glob.glob(os.path.join(cases, '*.json'))):
        name = os.path.basename(f)[:-5]
        if name == 'exempt' or (only and name != only): continue
        c = json.load(open(f))
        tmp = tempfile.mkdtemp()
        work = os.path.join(tmp, 'cases'); shutil.copytree(cases, work)
        for s in c.get('setup', []): call(cmd, s, work)
        code, out = call(cmd, c['call'], work)
        shutil.rmtree(tmp)
        if write:
            c['expect'] = {'exit': code}
            if out is not None and c.get('expect_json', True): c['expect']['json'] = out
            c.pop('expect_json', None)
            json.dump(c, open(f, 'w'), ensure_ascii=False, indent=1); print('書き出し', name); continue
        e = c['expect']; why = None
        if code != e['exit']: why = f'終了コード {code}（期待値 {e["exit"]}）'
        elif 'json' in e:
            x, y = json.loads(json.dumps(e['json'])), json.loads(json.dumps(out))
            for p in c.get('ignore', []): drop(x, p); drop(y, p)
            d = diff(x, y)
            if d: why = f'出力が期待値と違う: {d}'
        if why: ng += 1; print(f'  不合格 {name} ── {why}')
        else: ok += 1; print(f'  合格  {name}')
    if not write:
        print(f'テストケース {ok+ng}件 ／ 合格 {ok}件 ／ 不合格 {ng}件'); sys.exit(1 if ng else 0)
main()
