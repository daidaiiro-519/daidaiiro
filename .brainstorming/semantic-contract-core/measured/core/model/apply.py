"""domain-model-v2.json に changes.py の変更を適用して model.json を作る。
変更前の文が原文と合わなければ止まる（推測で適用しない）。"""
import copy, json, sys, os
sys.path.insert(0, os.path.dirname(__file__))
from changes import C, BOARD

HERE = os.path.dirname(os.path.abspath(__file__))
V2 = os.path.join(HERE, '..', '..', 'domain-model-v2.json')


def find(container, key):
    if isinstance(container, dict):
        return container, key
    for i, x in enumerate(container):
        if isinstance(x, dict) and x.get('name') == key: return container, i
        if isinstance(x, list) and x and x[0] == key: return container, i
    if isinstance(key, int): return container, key
    raise KeyError(key)


def locate(doc, path):
    cur = doc
    for k in path[:-1]:
        c, i = find(cur, k); cur = c[i]
    return find(cur, path[-1]) if path else (None, None)


def apply(doc, ch):
    op = ch['op']
    if op == 'add':
        parent = doc
        for k in ch['path']:
            c, i = find(parent, k); parent = c[i]
        if any((isinstance(x, list) and x and x[0] == ch['after'][0]) for x in parent if isinstance(ch['after'], list)):
            raise SystemExit(f"{ch['id']}: 既に在る")
        parent.append(ch['after']); return
    if op == 'set_row':
        parent = doc
        for k in ch['path'][:-1]:
            c, i = find(parent, k); parent = c[i]
        key = ch.get('key', ch['path'][-1])
        c, i = find(parent, key)
        if c[i] != ch['before']: raise SystemExit(f"{ch['id']}: 変更前が合わない\n  原文 {c[i]}\n  書いた {ch['before']}")
        c[i] = ch['after']; return
    if op == 'text_all_list':
        c, i = locate(doc, ch['path']); lst = c[i]
        n = sum(x.count(ch['before']) for x in lst)
        if n == 0: raise SystemExit(f"{ch['id']}: 変更前の語が無い")
        c[i] = [x.replace(ch['before'], ch['after']) for x in lst]; ch['count'] = n; return
    if op == 'words':
        s = json.dumps(doc, ensure_ascii=False); n = 0
        for b, af in ch['pairs']:
            k = s.count(b)
            if k == 0: raise SystemExit(f"{ch['id']}: 変更前の語が無い {b}")
            n += k; s = s.replace(b, af)
        doc.clear(); doc.update(json.loads(s)); ch['count'] = n; return
    if op == 'text_all_rows':
        c, i = locate(doc, ch['path']); n = 0
        for row in c[i]:
            for j, x in enumerate(row):
                if isinstance(x, str) and ch['before'] in x: n += x.count(ch['before']); row[j] = x.replace(ch['before'], ch['after'])
        if n == 0: raise SystemExit(f"{ch['id']}: 変更前の語が無い")
        ch['count'] = n; return
    c, i = locate(doc, ch['path']); val = c[i]
    if op == 'set':
        if val != ch['before']: raise SystemExit(f"{ch['id']}: 変更前が合わない\n  原文 {val}\n  書いた {ch['before']}")
        c[i] = ch['after']
    elif op == 'text':
        n = val.count(ch['before'])
        if n != 1: raise SystemExit(f"{ch['id']}: 変更前の文が {n} か所（1か所でなければ止める）\n  原文 {val}")
        c[i] = val.replace(ch['before'], ch['after'])
    elif op == 'text_all':
        n = val.count(ch['before'])
        if n == 0: raise SystemExit(f"{ch['id']}: 変更前の語が無い")
        c[i] = val.replace(ch['before'], ch['after']); ch['count'] = n
    elif op == 'append':
        c[i] = val + ch['after']
    else:
        raise SystemExit(f"{ch['id']}: 知らない op {op}")


def main():
    v2 = json.load(open(V2))
    doc = copy.deepcopy(v2)
    for ch in C: apply(doc, ch)
    doc['about'] = ('Semantic Agent Schema の基盤のドメインモデル（核）。domain-model-v2.json（Q1 の正本）に、'
                    '2026-10-24 までの会話の決定を changes.py の件数ぶん適用したもの。apply.py が作る。手で直さない。')
    doc['applied_changes'] = [ch['id'] for ch in C]
    doc['board_only'] = [{'where': w, 'before': b, 'after': a} for w, b, a in BOARD]
    json.dump(doc, open(os.path.join(HERE, 'model.json'), 'w'), ensure_ascii=False, indent=1)
    json.dump([{k: v for k, v in ch.items()} for ch in C], open(os.path.join(HERE, 'changes.json'), 'w'), ensure_ascii=False, indent=1)
    print(f'適用した {len(C)} 件 ・ ボードだけの直し {len(BOARD)} 件')
    left = [s for s in json.dumps(doc, ensure_ascii=False).split('"') if any(w in s for w in ('コミット', '作業ツリー', 'HEAD', 'worktree', ' PR', 'Git ', 'core')) and not s.startswith(('Git ・', '外（AI', 'core'))]
    print('残る変更の仕組みの語・core:'); [print('  ', s[:140]) for s in left]

main()
