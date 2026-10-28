"""組み上がったスライドを、全枚を縦に並べたHTMLとPDFにする。

  python3 export_pdf.py --browser <ブラウザの場所> [本の番号 …]   # 既定は 00（オリエンテーション）

出力は dist/ の本ごとのフォルダの slides-all.html と slides.pdf。並べ方は企画書と同じ FLAT_CSS を使う。
"""
import argparse, pathlib, subprocess, sys

HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE.resolve().parent / 'proposal'))
from places import place  # noqa: E402
import build_deck as BD  # noqa: E402



def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--browser', required=True)
    ap.add_argument('books', nargs='*', default=['00'])
    a = ap.parse_args()
    for no in a.books:
        stem = next(HERE.glob(f'lesson-{no}-*.json')).stem
        src = place(stem) / 'slides.html'
        flat = place(stem) / 'slides-all.html'
        flat.write_text(src.read_text().replace('</head>', BD.FLAT_CSS + '</head>', 1))
        pdf = place(stem) / 'slides.pdf'
        subprocess.run([a.browser, '--headless', '--no-pdf-header-footer', f'--print-to-pdf={pdf}', flat.resolve().as_uri()],
                       check=True, capture_output=True)
        print('書き出し:', flat.relative_to(HERE.resolve().parents[1]), '・', pdf.relative_to(HERE.resolve().parents[1]))


if __name__ == '__main__':
    main()
