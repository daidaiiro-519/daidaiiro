"""テーマ ── 配色の正本を読み、検査し、図を組ませる側へ渡す形へ写す。

    python3 scripts/cli.py check [デッキのHTML...]
    python3 scripts/cli.py theme <テーマの名前>

検査で見るのは3つ ── 鍵がすべてのテーマで一致しているか、適合条件を満たすか、
テーマの外に16進の直書きが残っていないか。**見た目は見ない。**

**この Skill は図を描かない。** 渡すのは配色だけである ── `as_roles()` が、
テーマの鍵を**図の中の役割の名前**へ複製する。誰に組ませるかは配線表が決める。
"""
import glob, os, re

from . import REFERENCES

HERE = str(REFERENCES / "themes")

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


def theme_files() -> list[str]:
    """配色の正本の一覧。**名前の並びは、置いてあるファイルが決める。**"""
    return sorted(glob.glob(os.path.join(HERE, '*.css')))


def theme_path(name: str) -> str:
    """名前から正本の場所を引く。"""
    path = os.path.join(HERE, f'{name}.css')
    if not os.path.exists(path):
        raise ValueError(f'知らないテーマ: {name}。使えるのは '
                         f'{"／".join(theme_names())} である')
    return path


def theme_names() -> list[str]:
    return [os.path.splitext(os.path.basename(f))[0] for f in theme_files()]


# 図の中の役割と、テーマのどの鍵から取るか。**この表はこちら側の語彙である** ──
# 組ませる相手の名前も、相手のトークン名も、ここは保持しない。
FIGURE_ROLES = {
    'ink': '--ink',                  # 図の中の見出し・強い文字
    'dim': '--dim',                  # 図の中の補足
    'faint': '--faint',              # いちばん薄い文字
    'line': '--diagram-line',        # 図の線。**本文の文字色とは別の鍵から取る**
    'paper': '--paper',              # 明るい面
    'surface': '--surface',          # 面
    'accent': '--accent',            # 強調
    'on-accent': '--on-accent',      # 強調面の上に置く文字
}


def as_roles(name: str) -> dict[str, str]:
    """テーマを、図の中の役割ごとの色へ複製する。

    **色の正本は1つである** ── 図の側にも色を書くと、テーマを替えたときに
    図だけが前の配色のまま残る（実際に4配色のうち1つで文字が消えた）。
    """
    t = tokens(theme_path(name))
    missing = sorted({k for k in FIGURE_ROLES.values() if k not in t})
    if missing:
        raise ValueError(f'{name}: 図へ渡す鍵が欠けている: {"、".join(missing)}')
    return {k: t[v] for k, v in FIGURE_ROLES.items()}


def findings(decks: list[str]) -> list[str]:
    """検査の検出を返す。**印字はしない** ── 印字は入口が持つ。"""
    bad: list[str] = []
    files = theme_files()
    sets = {os.path.basename(f): set(tokens(f)) for f in files}
    every = set().union(*sets.values()) if sets else set()

    for f in files:
        name, t = os.path.basename(f), tokens(f)
        for k in sorted(every - set(t)):
            bad.append(f'{name}: {k} が無い')
        for fg, bg, label, need in RULES:
            if fg in t and bg in t:
                r = ratio(t[fg], t[bg])
                if r < need:
                    bad.append(f'{name}: {label} {r:.2f}（要 {need}）  {t[fg]} / {t[bg]}')

    for d in decks:
        s = open(d, encoding='utf-8').read()
        try:
            i, j = s.index('▼ テーマ'), s.index('▲ テーマここまで')
            outside = s[:i] + s[j:]
        except ValueError:
            bad.append(f'{d}: テーマの区切りが無い')
            continue
        for m in sorted({m.group(0) for m in re.finditer(r'#[0-9a-fA-F]{3,8}\b', outside)}):
            bad.append(f'{d}: テーマの外に色の直書き {m}')
    return bad
