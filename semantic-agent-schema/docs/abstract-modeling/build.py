"""抽象モデリング概念のページ（index.html）を、元（index.src.html）と図（figs/*.svg）から作る。"""
from pathlib import Path

P = Path(__file__).resolve().parent
s = (P / 'index.src.html').read_text()
for n in ['essence', 'concept', 'scope', 'domains', 'roundtrip', 'model', 'diff']:
    s = s.replace('{{%s}}' % n, (P / 'figs' / f'{n}.svg').read_text())
assert '{{' not in s
(P / 'index.html').write_text(s)
