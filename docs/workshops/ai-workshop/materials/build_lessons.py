"""教材の動画9本ぶんのスライドを、企画デッキと同じ型で組む。

中身（見出し・リード・読み上げ）は lesson-0N-*.json が持ち、
図は本ごとの *_visuals.py が持つ。ここは並べるだけである。

組んだあとに、本の題を差し替え、語の検査を通す。
1件でも検出したら、組み立てを失敗にする。
"""
import html
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import build_deck as BD          # 型・CSS・表の部品を借りる
import lesson_visuals as V
import derived_visuals as DV     # 現行の図から派生させたもの
import intro_visuals as IV       # はじめに、および1本目の頭2枚
import word_visuals as WV        # 2本目
import scope_visuals as SV       # 3本目
import rule_visuals as RV        # 4本目
import fix_visuals as FV         # 5本目
import height_visuals as HV      # 1本目と6本目で共有する、高さの並び
import end_visuals as EV         # 終わりに

V.cover = DV.t_cover

FIG = {
 # はじめに
 'I0-S0A': IV.i0_why, 'I0-S0B': IV.i0_loop, 'I0-S1': IV.i0_case,
 'I0-S2': IV.i0_usecases, 'I0-S3': IV.i0_usecase, 'I0-S4': IV.i0_journey, 'I0-S5': IV.i0_base,
 # 教材1のはじめに
 'M1-S1': IV.i1_goal, 'M1-S2': IV.m1_task, 'M1-S3': IV.m1_map,
 'L1-S0': IV.i1_bridge, # 1本目　原因を知る
 'L1-S1': DV.t1_symptoms, 'L1-S2': DV.t1_height,
 'L1-S2B': HV.h1_axis, 'L1-S3': DV.t1_fit, 'L1-S4': V.l1_three, 'L1-S5': DV.t1_order,
 # 2本目　意味を決める
 'T2-S1': WV.t2_three_returns, 'T2-S2': WV.t2_sorting, 'T2-S3': WV.t2_ambiguous,
 'T2-S4': WV.t2_pick, 'T2-S5': WV.t2_after,
 # 3本目　範囲を決める
 'L3-S1': SV.s3_mixed, 'L3-S2': SV.s3_ranges, 'L3-S3': SV.s3_two_roles,
 'L3-S4': SV.s3_draw_line, 'L3-S5': SV.s3_after,
 # 4本目　条件を決める
 'L4-S1': DV.t4_missing, 'L4-S2': DV.t4_unwritten, 'L4-S3': DV.t4_layers,
 'L4-S4': RV.s4_effect, 'L4-S5': RV.s4_after,
 # 5本目　揺らぎを直す
 'L5-S1': DV.t6_symptoms, 'L5-S2': FV.f5_order, 'L5-S3': DV.t6_one_at_a_time,
 'L5-S4': DV.t6_two_goals, 'L5-S5': V.l6_whole,
 # 6本目　高さを合わせる
 'L6-S1': DV.t5_grown, 'L6-S2': DV.t5_swap, 'L6-S3': HV.h5_axis,
 'L6-S4': DV.t5_three_and_height2, 'L6-S5': V.l5_fit, 'L6-S6': V.l6_reproducible,
 # 終わりに
 'E-S1': EV.e_recap, 'E-S2': EV.e_gained, 'E-S3': EV.e_next,
}

# 本の題。表紙の大見出しは受講前でも読んで分かる1文にし、通し番号は出さない
# 本編6本の題。鍵は再生の順の番号で、本番号は これ - 1 である
TITLES = {
 2: ('原因を知る', '同じ依頼でも、返ってくるものが変わる原因を知る'),
 3: ('意味を決める', 'AIに、何を並べてほしいかを決める'),
 4: ('範囲を決める', 'AIに、どこまでを見てほしいかを決める'),
 5: ('条件を決める', 'どの項目にも必ず入れてほしいものを決める'),
 6: ('揺らぎを直す', 'それでも揃わないときに、どこを見るか'),
 7: ('高さを合わせる', '3つを書くことが、抽象の高さを合わせることでした'),
}
# 本編ではない3本。0は3つの教材に共通する前置きで、1と8が教材1の前後である
ENDS = {0: 'オリエンテーション', 1: 'はじめに', 8: '終わりに'}

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
    slides = [dict(label='表紙', title=f'{no}本目　{name}', cls='ws-titlepage',
                   intro=sub, body=V.cover(no), notes=d['slides'][0]['narration'])]
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
            f'<span class="ws-kicker-label">{no}本目　{esc(name)}　／　{i - 1}／{len(slides) - 1}</span></div>\n'
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
    stem = f'lesson-{no:02d}-{d["key"]}'
    (HERE / 'decks' / f'{stem}.html').write_text(prefix + '\n'.join(out) + suffix)

    # 読み上げ原稿（narration Skill へ渡す入力のもと）
    notes = [f'# {no}本目　{title}\n', f'枚数 {len(slides)}（表紙1・本編{len(slides)-1}）　'
             f'読み上げ {sum(len(x["notes"]) for x in slides[1:])}字\n']
    for i, s in enumerate(slides[1:], 1):
        notes.append(f'## {i:02d}　{s["title"]}\n\n{s["notes"]}\n')
    (HERE / 'decks' / f'{stem}-script.md').write_text('\n'.join(notes))
    print(f'{stem}.html　{len(slides)}枚')
    return no, stem


def retitle(no, stem):
    """表紙の大見出しと見出し帯から、制作側の通し番号を外す。"""
    f = HERE / 'decks' / f'{stem}.html'
    s = f.read_text()
    if no in ENDS:
        # 表紙の大見出しは、題だけを出す。見出し帯には はじめに ／ 終わりに を残す
        s = s.replace(f'<h1>{no}本目　', '<h1>')
        s = s.replace(f'{no}本目　', ENDS[no] + '　')
        s = s.replace('aria-label="6本の中での位置"', 'aria-label="6本の一覧"')
    else:
        short, full = TITLES[no]
        s = s.replace(f'<h1>{no}本目　{short}</h1>', f'<h1>{full}</h1>')
        s = s.replace(f'{no}本目　{short}　／　', f'{short}　／　')
        s = s.replace(f'aria-label="6本のうち、{no}本目"', 'aria-label="6本の中での位置"')
        s = s.replace(f'aria-label="1: {no}本目　{short}"', f'aria-label="1: {full}"')
    f.write_text(s)
    m = HERE / 'decks' / f'{stem}-script.md'
    if no in ENDS:
        m.write_text(m.read_text().replace(f'{no}本目　', ENDS[no] + '　'))


def check_ids():
    """枚の識別子と、図の対応表と、リード文が1対1かを確かめる。

    鍵が重複しても Python の辞書は後の値で上書きするだけで、組み立ては成功する。
    実際に、5本目の図が6本目の図へ差し替わったまま組み上がったことがある。
    """
    import ast
    src = (HERE / 'build_lessons.py').read_text()
    for node in ast.parse(src).body:
        if isinstance(node, ast.Assign) and getattr(node.targets[0], 'id', '') == 'FIG':
            keys = [k.value for k in node.value.keys]
    dup = sorted(k for k in set(keys) if keys.count(k) > 1)
    if dup:
        raise SystemExit(f'図の対応表に鍵の重複がある: {dup}')
    ids = []
    for p in sorted(HERE.glob('lesson-0*.json')):
        d = json.loads(p.read_text())
        for s in d['slides']:
            ids.append((s['id'], p.name))
    seen = {}
    for i, f in ids:
        seen.setdefault(i, []).append(f)
    dup = {i: f for i, f in seen.items() if len(f) > 1}
    if dup:
        raise SystemExit(f'枚の識別子が重複している: {dup}')
    missing = [i for i, _ in ids if i not in keys]
    if missing:
        raise SystemExit(f'図の対応表に無い枚がある: {missing}')
    nolede = [i for i, _ in ids if i not in LEDE]
    if nolede:
        raise SystemExit(f'リード文が無い枚がある: {nolede}')


if __name__ == '__main__':
    check_ids()
    targets = sys.argv[1:] or sorted(str(p) for p in HERE.glob('lesson-0*.json'))
    built = []
    for t in targets:
        built.append(build(t))
    for no, stem in built:
        retitle(no, stem)

    import check_words
    files = sorted((HERE / 'decks').glob('lesson-*.html')) + sorted((HERE / 'decks').glob('lesson-*-script.md'))
    if check_words.main(['check_words'] + [str(x) for x in files]):
        raise SystemExit('語の検査が通っていない')
