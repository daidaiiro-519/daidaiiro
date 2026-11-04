"""具体モデリング概念のページ（index.html）を、元（index.src.html）と図（figs/*.svg）から組む。"""
from pathlib import Path

P = Path(__file__).resolve().parent
s = (P / 'index.src.html').read_text()
for n in ['arch', 'records', 'flow-validate', 'flow-semantic']:
    s = s.replace('{{%s}}' % n, (P / 'figs' / f'{n}.svg').read_text())
diff = s[s.index('いまの実装との差</h2>'):]
for k, c in [('n_fix', 'fix'), ('n_cut', 'cut'), ('n_add', 'add')]:
    s = s.replace('{{%s}}' % k, str(diff.count(f'class="k {c}"')))
(P / 'index.html').write_text(s)
