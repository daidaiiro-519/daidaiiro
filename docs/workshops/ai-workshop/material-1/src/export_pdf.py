"""組み上がったスライドを、全枚を縦に並べたHTMLとPDFにする。

  python3 export_pdf.py --browser <ブラウザの場所> [本の番号 …]   # 既定は 00（オリエンテーション）

出力は out/pdf/<本>-all.html と out/pdf/<本>.pdf。並べ方は企画書と同じ FLAT_CSS を使う。
"""
import argparse, pathlib, subprocess, sys

HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE.parents[1] / 'proposal' / 'src'))
import build_deck as BD  # noqa: E402

SLIDES = HERE.parent / 'out' / 'slides'
PDF = HERE.parent / 'out' / 'pdf'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--browser', required=True)
    ap.add_argument('books', nargs='*', default=['00'])
    a = ap.parse_args()
    PDF.mkdir(exist_ok=True)
    for no in a.books:
        src = next(SLIDES.glob(f'lesson-{no}-*.html'))
        flat = PDF / f'{src.stem}-all.html'
        flat.write_text(src.read_text().replace('</head>', BD.FLAT_CSS + '</head>', 1))
        pdf = PDF / f'{src.stem}.pdf'
        subprocess.run([a.browser, '--headless', '--no-pdf-header-footer', f'--print-to-pdf={pdf}', flat.resolve().as_uri()],
                       check=True, capture_output=True)
        print('書き出し:', flat.relative_to(HERE.parent), '・', pdf.relative_to(HERE.parent))


if __name__ == '__main__':
    main()
