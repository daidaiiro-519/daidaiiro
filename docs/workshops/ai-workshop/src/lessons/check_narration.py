"""読み上げ原稿を、耳で追えるかの観点で数える。

判定はしない。数えて並べるだけである ── どこを直すかは人が決める。
文章を読む目では出ないもの（位置だけの説明・数の密度・列挙の項目数）を対象にする。
"""
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

# 画面を見ないと決まらない語。図を指す語と、位置だけの語
POS = ['図では', '上に置', '下に置', 'いちばん上', 'いちばん下', '真ん中',
       '上から見', '上が決ま', '下を直', '左', '右', 'あいだは空', 'ほうの端']
# 列挙の頭に付く語
ENUM = re.compile(r'(\d)(?:つめ|つ目|回目|本目|段目)')


def rows():
    for f in sorted(HERE.glob('lesson-0*.json')):
        d = json.loads(f.read_text())
        for s in d['slides']:
            yield d['no'], s['id'], s['heading'], s['narration']


def main(argv):
    hit = 0
    print('■ 位置だけで説明している箇所')
    for no, sid, _, t in rows():
        for w in POS:
            for m in re.finditer(re.escape(w), t):
                a = max(0, m.start() - 18)
                print(f'  {sid:8} …{t[a:m.end() + 18]}…')
                hit += 1
    if not hit:
        print('  なし')

    print('\n■ 1枚に出る数の個数（多い順・5個以上）')
    for no, sid, h, t in sorted(rows(), key=lambda r: -len(re.findall(r'\d+[件本回つ枚分]', r[3]))):
        v = re.findall(r'\d+[件本回つ枚分]', t)
        if len(v) < 5:
            break
        print(f'  {sid:8} {len(v):2}個  {" ".join(v)}')

    print('\n■ 1枚で並べている項目の数（4つ以上）')
    for no, sid, h, t in rows():
        v = [int(x) for x in ENUM.findall(t)]
        if v and max(v) >= 4:
            print(f'  {sid:8} 最大 {max(v)} 項目　{h[:34]}')

    print('\n■ 正本の3つの文が、どこにどう出ているか')
    # 読み上げだけを直して、画面の文字が旧いまま残る事故が実際に起きた。
    # 3つの文は 読み上げ ・ 見出し ・ リード ・ 図 の4か所に散るので、全部並べて目で見る
    import build_lessons as B
    KEY = {'意味': '進行が止まるもの', '範囲': '切り替えまでに決める', '条件': 'いつまでか'}
    led = json.loads((HERE / 'ledes.json').read_text())
    for name, k in KEY.items():
        print(f'  ── {name}（{k}）')
        seen = set()
        for no, sid, h, t2 in rows():
            for src, s in (('読み上げ', t2), ('見出し', h), ('リード', led.get(sid, ''))):
                for x in re.findall(rf'[^。]*{re.escape(k)}[^。]*', s):
                    x = x.strip()
                    if x and x not in seen:
                        seen.add(x); print(f'     {src}  {sid:8} {x[:56]}')
        for sid, fn in sorted(B.FIG.items()):
            try: svg = fn()
            except Exception: continue
            for x in re.findall(r'>([^<>]+)</text>', svg):
                if k in x and x not in seen:
                    seen.add(x); print(f'     図　　  {sid:8} {x[:56]}')

    print('\n■ 短い文が同じ語尾で続く箇所（4つ以上）')
    # 声に出すと、接続のないまま独立した文が積まれていると単調に聞こえる。
    # ただし 1つめ ・ 2つめ のような列挙は意図した並びなので、判定はせず並べるだけにする
    def _end(x):
        for e in ('ません。', 'ます。', 'です。', 'ください。'):
            if x.endswith(e):
                return e
        return None
    for no, sid, h, t2 in rows():
        v = [x.strip() + '。' for x in t2.split('。') if x.strip()]
        run = best = 0
        prev = None
        for x in v:
            e = _end(x)
            if e and e == prev and len(x) <= 30:
                run += 1
                best = max(best, run)
            else:
                run = 1 if e else 0
            prev = e
        if best >= 3:
            print(f'  {sid:8} {best + 1}連　{h[:34]}')

    print('\n■ 件数が、読み上げと図でどう出ているか')
    # 3周目に「数は図が持ち、音声は要点だけ」と決めた。だから図だけに在るのは誤りではない。
    # ただし、数の設計をやり直したとき図が旧いまま残った事故が実際に起きたので、並べて目で見る
    for no, sid, h, t2 in rows():
        fig = re.findall(r'>(\d+件)</text>', B.FIG[sid]()) if sid in B.FIG else []
        nar = re.findall(r'\d+件', t2)
        if fig or nar:
            print(f'  {sid:8} 図 {" ".join(fig) or "―":22} 読み上げ {" ".join(nar) or "―"}')

    print('\n■ 助数詞の使われ方')
    for k in ('本目', '回目', 'つめ', '件', '回分', '段目'):
        n = sum(len(re.findall(rf'\d+{k}', t)) for *_, t in rows())
        print(f'  {k:4} {n:3}回')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
