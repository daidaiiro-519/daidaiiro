"""教材の動画7本ぶんのスライドを、企画デッキと同じ型で組む。

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
# 型と部品は、企画デッキの組み立て（proposal/src/）から借りる
sys.path.insert(0, str(HERE.parents[1] / 'proposal' / 'src'))
import build_deck as BD          # 型・CSS・表の部品を借りる
import lesson_visuals as V
import derived_visuals as DV     # 現行の図から派生させたもの
import intro_visuals as IV       # はじめに、および1本目の頭2枚
import word_visuals as WV        # 2本目
import scope_visuals as SV       # 3本目
import rule_visuals as RV        # 4本目
import fix_visuals as FV         # 5本目
import height_visuals as HV      # 1本目と6本目で共有する、抽象度の並び
import end_visuals as EV         # 終わりに
import bridge_visuals as BV      # 動画のつなぎ（導入とまとめ）
import v1_visuals as V1          # 1本目（組み直した図）

import common_intro_visuals as CI
import orientation_current_visuals as OV
import summary_visuals as SM
V.cover = SM.cover

FIG = {
 # オリエンテーション（確認済みの正式版）
 'I0-CONTEXT': OV.FIG['I0-CONTEXT'],
 'I0-ACTION-02': OV.FIG['I0-ACTION-02'],
 'I0-ACTION-03': OV.FIG['I0-ACTION-03'],
 'I0-ACTION-04': OV.FIG['I0-ACTION-04'],
 'I0-ACTION-05': OV.FIG['I0-ACTION-05'],
 'I0-ACTION-06': OV.FIG['I0-ACTION-06'],
 'I0-ACTION-07': OV.FIG['I0-ACTION-07'],
 'I0-ACTION-08': OV.FIG['I0-ACTION-08'],
 'I0-ACTION-09': OV.FIG['I0-ACTION-09'],
 'I0-ACTION-10': OV.FIG['I0-ACTION-10'],
 'I0-PORTAL': OV.FIG['I0-PORTAL'],
 'I0-SUPPORT': OV.FIG['I0-SUPPORT'],
 'I0-SUMMARY': OV.FIG['I0-SUMMARY'],
 # 三教材共通のはじめに
 'COMMON-INTRO-01': lambda: CI.figure(1),
 'COMMON-INTRO-02': lambda: CI.figure(2),
 'COMMON-INTRO-03': lambda: CI.figure(3),
 'COMMON-INTRO-04': lambda: CI.figure(4),
 'COMMON-INTRO-05': lambda: CI.figure(5),
 'COMMON-INTRO-06': lambda: CI.figure(6),
 'COMMON-INTRO-07': lambda: CI.figure(7),
 'COMMON-INTRO-08': lambda: CI.figure(8),
 'COMMON-INTRO-09': lambda: CI.figure(9),
 'L1-S0': V1.ask, # 1本目　原因を知る
 'L1-S1': V1.symptoms, 'L1-S2': V1.ends,
 'L1-S3': V1.fit, 'T2-S3B': WV.t2_goal, 'L1-S4': V1.three, 'L1-S5': V1.order,
 # 2本目　意味を決める
 'T2-S1': WV.t2_three_returns, 'T2-S2': WV.t2_sorting, 'T2-S3': WV.t2_ambiguous,
 'T2-S4': WV.t2_pick, 'T2-S5': WV.t2_after,
 # 3本目　範囲を決める
 'L3-S1': SV.s3_mixed, 'L3-S2': SV.s3_ranges, 'L3-S3': SV.s3_two_roles,
 'L3-S4': SV.s3_draw_line, 'L3-S5': SV.s3_after,
 # 4本目　条件を決める
 'L4-S1': RV.s4_missing, 'L4-S2': RV.s4_unwritten, 'L4-S3': RV.s4_layers,
 'L4-S4': RV.s4_effect, 'L4-S5': RV.s4_after,
 # 最終の本編　具体と抽象を振り返る
 'F-S0': SM.opening, 'F-S1': SM.changed, 'F-S2': SM.meaning, 'F-S3': SM.values,
 'F-S4': SM.thinking, 'F-S5': SM.purpose, 'F-S6': SM.next_course,
 # 動画のつなぎ（導入とまとめ）
 'L1-IN': BV.I['L1-IN'], 'T2-S0': WV.t2_intro, 'T2-SE': WV.t2_end, 'L3-S0': SV.s3_intro, 'L3-SE': SV.s3_end,
 'L4-S0': BV.I['L4-S0'], 'L4-SE': RV.s4_end,
}

# 本の題。表紙の大見出しは短い名前だけにし、扱う問いは副題（各 JSON の title の ── の後）に1行で書く。
# 説明文を大見出しにしない ── 表紙に載せる情報の粒度は、教材の表紙「課題の把握」と安定させる
# 本編6本の題。鍵は再生の順の番号で、本番号は これ - 1 である
TITLES = {
 2: ('原因を知る', '原因を知る'),
 3: ('意味を決める', '意味を決める'),
 4: ('範囲を決める', '範囲を決める'),
 5: ('条件を決める', '条件を決める'),
 6: ('具体と抽象を振り返る', '具体と抽象を振り返る'),
}
# 本編ではない3本。0は3つの教材に共通する前置きで、1と8が教材1の前後である
ENDS = {0: 'オリエンテーション', 1: 'はじめに', 8: '終わりに'}

# リード文（枚の頭に置く1〜2文）。読み上げの要点を、読んで分かる形にする
# 出来上がりは out/ へ置く ── スライドは slides/、原稿は scripts/
SLIDES = HERE.parent / 'out' / 'slides'
SCRIPTS = HERE.parent / 'out' / 'scripts'
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
    cover = d.get('cover', {})
    slides = [dict(label='表紙', title=cover.get('heading', f'{no}本目　{name}'), cls='ws-titlepage',
                   intro=cover.get('lede', sub), body=CI.cover() if no == 1 and cover else V.cover(no),
                   notes=cover.get('narration', d['slides'][0]['narration']))] if d.get('include_cover', True) else []
    for s in d['slides']:
        fig = FIG.get(s['id'])
        if fig is None:
            raise SystemExit(f'図が無い: {s["id"]}')
        body = fig()
        _no_echo(s['id'], s['heading'], LEDE.get(s['id'], ''), body)
        slides.append(dict(label=s['heading'], title=s['heading'], cls='ws-diagram',
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
        heading = 'h1' if i == 1 and d.get('include_cover', True) else 'h2'
        title_html = esc(s['title'])
        if no == 0:
            title_html = title_html.replace('。仕事全体', '。<br>仕事全体')
        out.append(
            f'<section class="slide {s["cls"]}" aria-label="{i}: {esc(s["title"])}">\n'
            f'<div class="kicker"><span class="ws-mark" aria-hidden="true"></span>'
            f'<span class="ws-kicker-label">{no}本目　{esc(name)}　／　{i if no == 0 or not d.get('include_cover', True) else i - 1}／{len(slides) if no == 0 or not d.get('include_cover', True) else len(slides) - 1}</span></div>\n'
            f'<{heading}>{title_html}</{heading}>\n'
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
    (SLIDES / f'{stem}.html').write_text(prefix + '\n'.join(out) + suffix)

    # 読み上げ原稿（narration Skill へ入力する入力のもと）
    content = slides[1:] if d.get('include_cover', True) else slides
    cover_count = int(d.get('include_cover', True))
    notes = [f'# {no}本目　{title}\n', f'枚数 {len(slides)}（表紙{cover_count}・本編{len(content)}）　'
             f'読み上げ {sum(len(x["notes"]) for x in (slides if cover else content))}字\n']
    if cover and d.get('include_cover', True):
        notes.append(f'## 表紙　{cover["heading"]}\n\n{cover["narration"]}\n')
    for i, s in enumerate(content, 1):
        notes.append(f'## {i:02d}　{s["title"]}\n\n{s["notes"]}\n')
    (SCRIPTS / f'{stem}-script.md').write_text('\n'.join(notes))
    print(f'{stem}.html　{len(slides)}枚')
    return no, stem


def retitle(no, stem):
    """表紙の大見出しと見出し帯から、制作側の通し番号を外す。"""
    f = SLIDES / f'{stem}.html'
    s = f.read_text()
    if no in ENDS:
        # 表紙の大見出しは、題だけを出す。見出し帯には はじめに ／ 終わりに を残す
        s = s.replace(f'<h1>{no}本目　', '<h1>')
        s = s.replace(f'{no}本目　', ENDS[no] + '　')
        s = s.replace('aria-label="6本の中での位置"', 'aria-label="6本の一覧"')
        if no == 1:
            s = s.replace('はじめに　はじめに　', '3教材共通　はじめに　')
    else:
        short, full = TITLES[no]
        s = s.replace(f'<h1>{no}本目　{short}</h1>', f'<h1>{full}</h1>')
        s = s.replace(f'{no}本目　{short}　／　', f'{short}　／　')
        s = s.replace(f'aria-label="6本のうち、{no}本目"', 'aria-label="6本の中での位置"')
        s = s.replace(f'aria-label="1: {no}本目　{short}"', f'aria-label="1: {full}"')
    f.write_text(s)
    m = SCRIPTS / f'{stem}-script.md'
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
    orphan = sorted(set(keys) - {i for i, _ in ids})
    if orphan:
        raise SystemExit(f'枚が無いのに、図の対応表に残っている: {orphan}')
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
    files = sorted(SLIDES.glob('lesson-*.html')) + sorted(SCRIPTS.glob('lesson-*-script.md'))
    if check_words.main(['check_words'] + [str(x) for x in files]):
        raise SystemExit('語の検査が通っていない')
