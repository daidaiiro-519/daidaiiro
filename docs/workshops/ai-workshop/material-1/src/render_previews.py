"""組んだスライド（out/slides/*.html）を、1枚ずつ画像（previews/NN-MM.png）にする。

画像は、動画の結合（build_videos.py）と照合の入力（review_input.py）が使う。

  python3 render_previews.py --browser <ブラウザの場所>          全9本
  python3 render_previews.py --browser <ブラウザの場所> 00 01    指定した本だけ

**ブラウザの場所は渡す** ── 端末ごとに置き場所が違うので、この側で推測しない。
描画の前に、枚が切り替わるときの動きを止める ── 動きの途中で撮ると、枚が半透明に写る。
"""
import argparse
import json
import pathlib
import subprocess
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
SLIDES = HERE.parent / 'out' / 'slides'
PREVIEWS = HERE / 'previews'


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--browser', required=True)
    ap.add_argument('numbers', nargs='*', help='本の番号（00〜08）。省くと全部')
    a = ap.parse_args()
    PREVIEWS.mkdir(exist_ok=True)
    done = []
    with tempfile.TemporaryDirectory() as tmp:
        for f in sorted(HERE.glob('lesson-0*.json')):
            no = f.name.split('-')[1]
            if a.numbers and no not in a.numbers:
                continue
            count = len(json.loads(f.read_text())['slides']) + 1   # 表紙を含む
            page = pathlib.Path(tmp) / f'{f.stem}.html'
            page.write_text((SLIDES / f'{f.stem}.html').read_text()
                            .replace('</style>', '.slide.on{animation:none!important}</style>', 1))
            for i in range(1, count + 1):
                shot = PREVIEWS / f'{no}-{i:02d}.png'
                subprocess.run([a.browser, '--headless=new', '--no-sandbox', '--window-size=1280,720',
                                f'--screenshot={shot}', f'file://{page}#{i}'], capture_output=True)
                if not shot.exists():
                    raise SystemExit(f'画像を作れなかった: {shot}')
            done.append(no)
    print('描画した本:', ' '.join(done))


if __name__ == '__main__':
    main()
