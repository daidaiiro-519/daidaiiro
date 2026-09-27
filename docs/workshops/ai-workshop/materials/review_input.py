"""教材のデッキを、slide-deck の照合の入力（review.schema.json の形）へ変換する。

slide-deck は、この教材の入力の形（lesson-0N-*.json ・ ledes.json ・ previews/）を知らない。
変換は教材の側に置く ── Skill が特定のリポジトリの形を知ると、ほかのデッキに使えなくなる。

  python3 review_input.py snapshot <置き場所>
      いまの原稿 ・ 導入文 ・ 描画した画像を、変更前として固定する（照合の工程の1段目）
  python3 review_input.py build <変更前の置き場所> <出力.json> [--log review-log.json] [--title 題]
      変更前と、いまの状態（変更後）と、確認の結果から、照合の入力を組む

画像は previews/ を使う。先に build_lessons.py で組み、描画しておく。
"""
import argparse
import json
import pathlib
import shutil
import subprocess

HERE = pathlib.Path(__file__).resolve().parent

# 区切りの名前。区切りの識別子は、ファイル名の番号である
NAMES = {'00': 'オリエンテーション', '01': '教材1のはじめに', '02': '1本目　原因を知る',
         '03': '2本目　意味を決める', '04': '3本目　範囲を決める', '05': '4本目　条件を決める',
         '06': '5本目　揺らぎを直す', '07': '6本目　抽象の高さを合わせる', '08': '教材1のまとめ'}


def lessons(root):
    return sorted(root.glob('lesson-0*.json'))


def slide_ids(lesson):
    """表紙を先頭に置いた、枚の識別子の並び。previews/ の番号と同じ順である。"""
    return ['COVER'] + [s['id'] for s in json.loads(lesson.read_text())['slides']]


def snapshot(dest, rev=''):
    dest.mkdir(parents=True, exist_ok=True)
    for f in lessons(HERE):
        shutil.copy(f, dest / f.name)
        no = f.name.split('-')[1]
        for i, sid in enumerate(slide_ids(f), 1):
            src = HERE / 'previews' / f'{no}-{i:02d}.png'
            (dest / 'img').mkdir(exist_ok=True)
            shutil.copy(src, dest / 'img' / f'{no}-{sid}.png')
    shutil.copy(HERE / 'ledes.json', dest / 'ledes.json')
    if not rev:
        rev = subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], cwd=HERE,
                             capture_output=True, text=True).stdout.strip()
    (dest / 'rev.txt').write_text(rev + '\n')


def side(root, img_of, out_dir):
    """1つの版を、照合の入力の snapshot の形へ組む。"""
    ledes = json.loads((root / 'ledes.json').read_text())
    sections = []
    for f in lessons(root):
        d = json.loads(f.read_text())
        no = f.name.split('-')[1]
        cover_title, _, cover_sub = d.get('title', '').partition(' ── ')
        slides = [{'id': 'COVER', 'heading': cover_title, 'lede': cover_sub,
                   'image': img_of(no, 'COVER', 1)}]
        for i, s in enumerate(d['slides'], 2):
            one = {'id': s['id'], 'heading': s['heading'], 'narration': s['narration'],
                   'image': img_of(no, s['id'], i)}
            if s['id'] in ledes:
                one['lede'] = ledes[s['id']]
            slides.append(one)
        for s in slides:
            s['image'] = str(pathlib.Path(s['image']).resolve().relative_to(out_dir.resolve(), walk_up=True))
        sections.append({'key': no, 'name': NAMES.get(no, no), 'slides': slides})
    return {'sections': sections}


def build(before_dir, out, log=None, title=''):
    out_dir = out.parent
    before = side(before_dir, lambda no, sid, i: before_dir / 'img' / f'{no}-{sid}.png', out_dir)
    rev = (before_dir / 'rev.txt')
    if rev.exists():
        before['rev'] = rev.read_text().strip()
    after = side(HERE, lambda no, sid, i: HERE / 'previews' / f'{no}-{i:02d}.png', out_dir)
    doc = {'title': title or '下期AI活用ワークショップ　オリエンテーションと教材1の照合',
           'before': before, 'after': after}
    if log:
        body = json.loads(log.read_text())
        doc['summary'] = body.get('summary', [])
        doc['log'] = body.get('log', [])
    out.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + '\n')


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest='cmd', required=True)
    s = sub.add_parser('snapshot')
    s.add_argument('dest', type=pathlib.Path)
    s.add_argument('--rev', default='')
    b = sub.add_parser('build')
    b.add_argument('before', type=pathlib.Path)
    b.add_argument('out', type=pathlib.Path)
    b.add_argument('--log', type=pathlib.Path)
    b.add_argument('--title', default='')
    a = ap.parse_args()
    if a.cmd == 'snapshot':
        snapshot(a.dest, a.rev)
    else:
        build(a.before, a.out, a.log, a.title)


if __name__ == '__main__':
    main()
