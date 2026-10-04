"""原稿から、narration Skill の入力（narration/narration.json）を組む。

  python3 build_narration.py              全9本を入力にする
  python3 build_narration.py 00-start     指定した本だけを入力にする

枚の識別子は build_videos.py と同じ規則で付ける ── 表紙は「<key の大文字>-COVER」、
それ以外は原稿の id である。表紙は、題と副題を読み上げる。
全9本ぶんは narration.full.json にも書く。音声は文 ・ 声 ・ エンジンから作った鍵で
取り出すので、原稿が変わらない枚は合成し直さない。
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
NARR = HERE / 'narration'
HEAD = {'voice': 'Takumi', 'engine': 'neural', 'format': 'mp3', 'cache': 'cache', 'lexicons': ['aiworkshop']}  # 辞書は narration/lexicon.pls


def items_of(path):
    d = json.loads(path.read_text())
    stem = f'{d["no"]:02d}-{d["key"]}'
    title = '。'.join(p for p in d.get('title', '').split(' ── ') if p) + '。'
    out = [{'id': f'{d["key"].upper()}-COVER', 'text': title}]
    out += [{'id': s['id'], 'text': s['narration']} for s in d['slides']]
    return stem, out


def main(want):
    every = [items_of(p) for p in sorted(HERE.glob('lesson-0*.json'))]
    full = dict(HEAD, items=[i for _, items in every for i in items])
    (NARR / 'narration.full.json').write_text(json.dumps(full, ensure_ascii=False, indent=1) + '\n')
    chosen = [i for stem, items in every if not want or stem in want for i in items]
    if not chosen:
        raise SystemExit(f'該当する本が無い: {want}')
    (NARR / 'narration.json').write_text(json.dumps(dict(HEAD, items=chosen), ensure_ascii=False, indent=1) + '\n')
    print(f'narration.json　{len(chosen)}枚')


if __name__ == '__main__':
    main(sys.argv[1:])
