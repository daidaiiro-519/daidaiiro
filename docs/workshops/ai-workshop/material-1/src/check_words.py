"""教材の語を検査する。同じものを指す語が2つ以上あると、読み手の認知負荷が上がる。

正本は words.json である。検査するのは、組み上がった動画のスライドのうち
**読み手の目に入るもの** ── 見出し ・ リード ・ 図の文字 ・ 代替テキスト ── と、
読み上げ原稿である。
"""
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def visible(html: str):
    """スライドの中で読み手の目に入る文字を取り出す。"""
    out = []
    for m in re.finditer(r'>([^<>]+)<', html):
        s = m.group(1).strip()
        if s:
            out.append(s)
    out += re.findall(r'aria-label="([^"]*)"', html)
    return out


def check(paths, words=None):
    w = words or json.loads((HERE / 'words.json').read_text())
    hits = []
    for p in paths:
        p = Path(p)
        text = p.read_text()
        lines = visible(text) if p.suffix == '.html' else text.split('\n')
        for i, line in enumerate(lines, 1):
            # 書名は固有名詞なので、検査の前に除く
            for t in w.get('書名', {}).get('題', []):
                line = line.replace(t, '')
            for g in w['同じものを指す語']:
                for ng in g['使わない']:
                    rest = line
                    for ex in g.get('例外', []):
                        rest = rest.replace(ex, '')
                    if ng in rest:
                        hits.append((p.name, line, ng, f"「{g['使う']}」へ揃える（{g['指すもの']}）"))
            for g in w['使用しない語']:
                if g.get('対象') and g['対象'] not in p.name:
                    continue
                if g['語'] in line:
                    hits.append((p.name, line, g['語'], f"{g['理由']} ── {g['代わりに']}"))
            for g in w['使用しない書き方']:
                if g.get('対象') and not (g['対象'] in p.name or p.suffix == g['対象']):
                    continue
                m = re.search(g['正規表現'], line)
                if m:
                    hits.append((p.name, line, m.group(0), f"{g['理由']} ── {g['代わりに']}"))
    return hits


def main(argv):
    targets = [Path(a) for a in argv[1:]]
    if not targets:
        targets = sorted((HERE.parent / 'out' / 'slides').glob('lesson-*-trial.html'))
    hits = check(targets)
    seen = set()
    for name, line, ng, why in hits:
        key = (name, ng, line[:40])
        if key in seen:
            continue
        seen.add(key)
        print(f'× {name}　[{ng}]　{why}')
        print(f'    {line[:76]}')
    n = len(seen)
    print(f'語の検査　{"通った" if not n else f"通っていない（{n} 件）"}　／　{len(targets)} ファイル')
    return 1 if n else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
