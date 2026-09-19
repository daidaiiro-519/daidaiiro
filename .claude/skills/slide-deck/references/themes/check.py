"""テーマの形を検査する。0件にできるものだけを対象にする。

    python3 references/themes/check.py [デッキのHTML...]

見るのは3つ ── 鍵が4つのテーマで一致しているか、適合条件を満たすか、
テーマの外に16進の直書きが残っていないか。**見た目は見ない。**
"""
import glob, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))

# 適合条件。(文字, 地, 名前, 要る比)
RULES = [
    ('--ink', '--ground', '本文', 4.5),
    ('--dim', '--ground', '補助', 4.5),
    ('--faint', '--ground', '最薄', 4.5),
    ('--accent', '--ground', '強調', 4.5),
    ('--accent', '--surface', '強調（面の上）', 4.5),
    ('--on-accent', '--accent', '強調面の文字', 4.5),
    ('--on-accent-dim', '--accent-dim', '濃い強調面の文字', 4.5),
    ('--on-paper', '--paper', '明るい面の文字', 4.5),
    ('--on-paper-dim', '--paper', '明るい面の補足', 4.5),
    ('--on-paper', '--mark', '印の上の文字', 4.5),
    ('--ink', '--surface', '面の上の本文', 4.5),
    ('--diagram-line', '--canvas', '図の線（図の地）', 3.0),
    ('--diagram-line', '--ground', '図の線（地）', 3.0),
]
# --line は装飾専用。情報を単独で担わせないので、比を課さない。


def _lum(h):
    h = h.lstrip('#')
    if len(h) == 3:
        h = ''.join(c * 2 for c in h)
    def f(c):
        c = int(c, 16) / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    return 0.2126 * f(h[0:2]) + 0.7152 * f(h[2:4]) + 0.0722 * f(h[4:6])


def ratio(a, b):
    la, lb = _lum(a), _lum(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def tokens(path):
    return dict(re.findall(r'(--[a-z0-9-]+)\s*:\s*(#[0-9a-fA-F]{3,8})\s*;',
                           open(path, encoding='utf-8').read()))


def main(decks):
    bad = 0
    files = sorted(glob.glob(os.path.join(HERE, '*.css')))
    sets = {os.path.basename(f): set(tokens(f)) for f in files}
    every = set().union(*sets.values()) if sets else set()

    for f in files:
        name, t = os.path.basename(f), tokens(f)
        for k in sorted(every - set(t)):
            print(f'× {name}: {k} が無い'); bad += 1
        for fg, bg, label, need in RULES:
            if fg in t and bg in t:
                r = ratio(t[fg], t[bg])
                if r < need:
                    print(f'× {name}: {label} {r:.2f}（要 {need}）  {t[fg]} / {t[bg]}')
                    bad += 1

    for d in decks:
        s = open(d, encoding='utf-8').read()
        try:
            i, j = s.index('▼ テーマ'), s.index('▲ テーマここまで')
            outside = s[:i] + s[j:]
        except ValueError:
            print(f'× {d}: テーマの区切りが無い'); bad += 1; continue
        for m in sorted({m.group(0) for m in re.finditer(r'#[0-9a-fA-F]{3,8}\b', outside)}):
            print(f'× {d}: テーマの外に色の直書き {m}'); bad += 1

    print(f'\nテーマの検査　{"通った" if not bad else f"通っていない（{bad} 件）"}'
          f'　／　テーマ {len(files)} 本、デッキ {len(decks)} 本')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
