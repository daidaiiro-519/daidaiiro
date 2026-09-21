# SPDX-License-Identifier: MIT
"""句の末尾を全部拾って並べる。**語彙表を使わない。**

    python3 scripts/cli.py tails <ファイル>...

ゲート1 の「述部が和語である」は語彙表で照合するので、**表に無い和語は通過する**。
この道具は逆で、句の末尾を機械的に全部拾い、漢語の述部だけを除外して残りを並べる ──
判定は人が実施するが、**拾い残しが発生しない**。

確定したものは `gate.py` の WAGO へ追加する ── そうすると次からは機械が検出する。
この道具は「未知を洗い出すための2周目」であって、ゲート1 の代替ではない。
"""
import re, sys, pathlib, collections, json

KANJI = r"一-鿿"
# 漢語の述部（…する／…した／…である／…できる…）はここで除外する
KANGO = re.compile(rf"[{KANJI}]{{1,6}}(する|した|しない|される|された|できる|できない|"
                   rf"である|でない|になる|による|とする|しうる|し、|する。)$")
TAIL = re.compile(rf"([{KANJI}ぁ-んァ-ヶー]{{2,8}})(?=[。、）」\n]|$)")

def tails(text):
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"`[^`]*`", "", text)
    for m in TAIL.finditer(text):
        s = m.group(1)
        if KANGO.search(s):
            continue
        # 述部らしい語尾だけを残す
        if re.search(r"(る|た|い|う|く|す|つ|ぶ|む|ぬ|ぐ|ず|ない|なる|れる|られる|ある|いる)$", s):
            yield s

def counted(paths):
    """述部の末尾を数える。**印字はしない。**"""
    c = collections.Counter()
    for p in paths:
        raw = pathlib.Path(p).read_text(encoding="utf-8")
        if p.endswith(".json"):
            def walk(o):
                if isinstance(o, str): return [o]
                if isinstance(o, list): return [x for v in o for x in walk(v)]
                if isinstance(o, dict): return [x for k, v in o.items()
                                                if not k.startswith("_") for x in walk(v)]
                return []
            raw = "。".join(walk(json.loads(raw)))
        for t in tails(raw):
            c[t] += 1
    return sorted(c.items(), key=lambda kv: (-kv[1], kv[0]))


def tails_lines(paths):
    """一覧の行。**印字する側と、機械へ返す側が、同じ文字列を使う。**"""
    return [f"{n:3}  {word}" for word, n in counted(paths)]


def main(paths):
    for line in tails_lines(paths):
        print(line)
    return 0
