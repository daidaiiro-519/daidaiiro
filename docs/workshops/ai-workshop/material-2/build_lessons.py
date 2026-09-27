"""教材の動画6本ぶんのスライドを、企画デッキと同じ型で組む。

中身（見出し・リード・読み上げ）は lesson-0N-*.json が持ち、
図は lesson_visuals.py が持つ。ここは並べるだけである。
"""
import html
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'proposal' / 'src'))
import build_deck as BD          # 型・CSS・表の部品を借りる
import ddd_visuals as V

FIG = {
 'D2-S5': V.d2_handoff,
 'D0-S2B': V.d0_roles,
 'D0-S1': V.d0_recap, 'D0-S2': V.d0_translation, 'D0-S3': V.d0_volume,
 'D0-S4': V.d0_words, 'D0-S5': V.d0_order,
}

# リード文（枚の頭に置く1〜2文）。読み上げの要点を、読んで分かる形にする
LEDE = json.loads((HERE / 'ledes.json').read_text()) if (HERE / 'ledes.json').exists() else {}


def esc(s):
    return html.escape(s)


def _norm(s):
    return re.sub(r'[。、　 「」]', '', s)


def _no_echo(sid, heading, lede, svg):
    """見出し・リード・図が、同じ文を2度出していないかを見る。"""
    h, l = _norm(heading), _norm(lede)
    if len(h) >= 8 and (h in l or l[:len(h)] == h):
        raise SystemExit(f'{sid}　リードが見出しの言い換えになっている: {lede}')
    # 図の中の見出し語が、枚の見出しと重なるのは自然である。見るのはリードとの重なりだけ
    for s in re.findall(r'>([^<>]+)</text>', svg):
        n = _norm(s)
        if len(n) >= 12 and n in l:
            raise SystemExit(f'{sid}　図の文が、リードと同じ: {s}')


def build(lesson_path):
    d = json.loads(Path(lesson_path).read_text())
    no, title = d['no'], d['title']
    name, sub = (title.split(' ── ') + [''])[:2]
    slides = [dict(label='表紙', title=('前置き' if no == 0 else f'{no}本目') + f'　{name}', cls='ws-titlepage',
                   intro=sub, body='', notes=d['slides'][0]['narration'])]
    for s in d['slides']:
        fig = FIG.get(s['id'])
        if fig is None:
            raise SystemExit(f'図が無い: {s["id"]}')
        body = fig()
        _no_echo(s['id'], s['heading'], LEDE.get(s['id'], ''), body)
        slides.append(dict(label=s['heading'][:22], title=s['heading'], cls='ws-diagram',
                           intro=LEDE.get(s['id'], ''), body=body, notes=s['narration']))

    template = BD.TEMPLATE.read_text()
    prefix = template.split('<!-- 01 -->')[0]
    prefix = re.sub(r'<link[^>]+>\s*', '', prefix)
    prefix = prefix.replace('<title>デッキの題名</title>',
        '<!doctype html>\n<html lang="ja"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        f'<title>{esc(name)}</title>')
    prefix = prefix.replace('<div class="navzone prev"',
                            '<style>' + BD.CSS + '</style></head><body>\n<div class="navzone prev"', 1)
    out = []
    for i, s in enumerate(slides, 1):
        heading = 'h1' if i == 1 else 'h2'
        out.append(
            f'<section class="slide {s["cls"]}" aria-label="{i}: {esc(s["title"])}">\n'
            f'<div class="kicker"><span class="ws-mark" aria-hidden="true"></span>'
            f'<span class="ws-kicker-label">{"前置き" if no == 0 else str(no) + "本目"}　{esc(name)}　／　{i - 1}／{len(slides) - 1}</span></div>\n'
            f'<{heading}>{esc(s["title"])}</{heading}>\n'
            f'<p class="ws-intro">{esc(s["intro"])}</p>\n'
            f'<div class="body">{s["body"]}</div>\n</section>')
    suffix = template[template.index('<div class="chrome">'):]
    labels = json.dumps([s['label'] for s in slides], ensure_ascii=False)
    suffix = re.sub(r'const LABELS = .*?;', f'const LABELS = {labels};', suffix)
    suffix = suffix.replace("const prev = ()=> show(i-1);",
        "const prev = ()=> show(i-1);\n  addEventListener('hashchange',()=>{const n=parseInt(location.hash.slice(1),10)-1;"
        "if(Number.isFinite(n)&&n!==i)show(n);});")
    suffix += '\n</body></html>\n'
    stem = f'ddd-{no:02d}-{d["key"]}'
    (HERE / 'decks' / f'{stem}.html').write_text(prefix + '\n'.join(out) + suffix)

    # 読み上げ原稿（narration Skill へ渡す入力のもと）
    notes = [f'# {"前置き" if no == 0 else str(no) + "本目"}　{title}\n', f'枚数 {len(slides)}（表紙1・本編{len(slides)-1}）　'
             f'読み上げ {sum(len(x["notes"]) for x in slides[1:])}字\n']
    for i, s in enumerate(slides[1:], 1):
        notes.append(f'## {i:02d}　{s["title"]}\n\n{s["notes"]}\n')
    (HERE / 'decks' / f'{stem}-script.md').write_text('\n'.join(notes))
    print(f'{stem}.html　{len(slides)}枚')


if __name__ == '__main__':
    targets = sys.argv[1:] or sorted(str(p) for p in HERE.glob('ddd-0*.json'))
    for t in targets:
        build(t)
