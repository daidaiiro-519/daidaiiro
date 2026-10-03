#!/usr/bin/env python3
"""正解の事例を、Skill の tool.json の起動のコマンドで照らす試作（ボード skills-creator-contract の試し）。

使い方: run.py <Skill のフォルダ> [--write]   --write は期待を書き出す（見本から事例を作るときだけ）
"""
import json, os, subprocess, sys, glob

here = os.path.dirname(os.path.abspath(__file__))
cases_dir = os.path.join(here, 'cases')

def command_of(skill):
    tj = json.load(open(os.path.join(skill, 'tool.json')))
    cli = tj['cli']
    cmd = cli['command'].replace('${CLAUDE_PROJECT_DIR:-.}', os.environ.get('CLAUDE_PROJECT_DIR', '/home/daidaiiro/workspace/daidaiiro'))
    return [cmd] + cli.get('args', [])

def call(cmd, args):
    p = subprocess.run(cmd + args + ['--json'], cwd=cases_dir, capture_output=True, text=True)
    try:
        out = json.loads(p.stdout)
    except Exception:
        out = None
    return p.returncode, out

def first_diff(a, b, at=''):
    if type(a) != type(b):
        return at or '/'
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                return f'{at}/{k}'
            d = first_diff(a[k], b[k], f'{at}/{k}')
            if d: return d
        return None
    if isinstance(a, list):
        if len(a) != len(b): return f'{at}（件数 {len(a)} と {len(b)}）'
        for i, (x, y) in enumerate(zip(a, b)):
            d = first_diff(x, y, f'{at}/{i}')
            if d: return d
        return None
    if a != b:
        if isinstance(a, str):
            n = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))
            return f'{at}（{n}文字目から）'
        return at
    return None

def main():
    skill = sys.argv[1]; write = '--write' in sys.argv
    cmd = command_of(skill)
    ok = ng = 0
    for f in sorted(glob.glob(os.path.join(cases_dir, '*.json'))):
        c = json.load(open(f))
        code, out = call(cmd, c['call'])
        if write:
            c['expect'] = {'exit': code} | ({'json': out} if c.get('compare_json', True) else {})
            json.dump(c, open(f, 'w'), ensure_ascii=False, indent=1)
            print('書き出し', c['case']); continue
        e = c['expect']
        why = None
        if code != e['exit']:
            why = f'終了コード {code}（期待 {e["exit"]}）'
        elif 'json' in e:
            d = first_diff(e['json'], out)
            if d: why = f'出力が期待と違う: {d}'
        if why: ng += 1; print(f'  不合格 {os.path.basename(f)[:-5]} ── {why}')
        else: ok += 1; print(f'  合格  {os.path.basename(f)[:-5]}')
    if not write:
        print(f'正解の事例 {ok+ng}件 ／ 合格 {ok}件 ／ 不合格 {ng}件')
        sys.exit(1 if ng else 0)

main()
