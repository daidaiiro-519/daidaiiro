"""層の向きを検査する ── 言語に依存しない試作。標準ライブラリだけで動く。

入力は2つである。
  記録　　層の名前と、その置き場所。並び（内から外）
  形　　　参照を書く行の形。言語ごとに正規表現1〜2本

処理は3工程である。
  ①各ファイルの層を、置き場所から決める
  ②参照の行を拾い、参照先の文字列から相手のdecide_layer
  ③並びに違反した辺を出す

**解析器は書かない。** 参照の行を拾うだけで、名前の解決は実施しない ──
解決は言語ごとに規則が違うので、そこへ入ると言語に依存する。
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

shape = {
    ".py":  [r"^\s*from\s+([A-Za-z_][\w\.]*)", r"^\s*import\s+([A-Za-z_][\w\.]*)"],
    ".ts":  [r"""from\s+['"]([^'"]+)['"]""", r"""require\(\s*['"]([^'"]+)['"]"""],
    ".tsx": [r"""from\s+['"]([^'"]+)['"]""", r"""require\(\s*['"]([^'"]+)['"]"""],
    ".js":  [r"""from\s+['"]([^'"]+)['"]""", r"""require\(\s*['"]([^'"]+)['"]"""],
    ".go":  [r'^\s*"([^"]+)"', r'^\s*[\w\.]+\s+"([^"]+)"'],
    ".rs":  [r"^\s*(?:pub\s+)?use\s+([\w:]+)"],
    ".java": [r"^\s*import\s+(?:static\s+)?([\w\.]+)"],
}


def decide_layer(path: str, layer: dict[str, str]) -> str | None:
    """置き場所の前方一致で決める。**長い方を先に見る** ── 入れ子の層が在るため。"""
    path = path.replace("\\", "/")
    for name, place in sorted(layers.items(), key=lambda kv: -len(kv[1])):
        place = place.strip("/")
        if path == place or path.startswith(place + "/"):
            return name
    return None


def layer_of(ref: str, layers: dict[str, str]) -> str | None:
    """参照の文字列に、層の置き場所の末尾の名前が現れるかで決める。"""
    word = re.split(r"[\./:\\]+", ref)
    word = [x for x in word if x]
    for name, place in layers.items():
        tail = place.strip("/").split("/")[-1]
        if name in word or tail in word:
            return name
    return None


def check(root: pathlib.Path, record: dict) -> list[tuple[str, int, str, str, str]]:
    layers = record["層"]
    order = record["並び"]                      # 内から外
    pos = {name: i for i, name in enumerate(order)}
    violations = []
    for p in sorted(root.rglob("*")):
        if not p.is_file() or p.suffix not in shapes:
            continue
        path = str(p.relative_to(root))
        mine = decide_layer(path, layers)
        if mine is None:
            continue
        for n, lines in enumerate(p.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            if lines.lstrip().startswith(("#", "//")):
                continue
            for pattern in shapes[p.suffix]:
                m = re.search(pattern, lines)
                if not m:
                    continue
                peer = layer_of(m.group(1), layers)
                if peer is None or peer == mine:
                    continue
                # **内側が外側を参照したら違反である** ── 並びは内から外である
                if pos[peer] > pos[mine]:
                    violations.append((path, n, mine, peer, lines.strip()))
    return violations


def main() -> int:
    root = pathlib.Path(sys.argv[1])
    record = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
    violations = check(root, record)
    if not violations:
        print("内側は外側を参照しない　合格")
        return 0
    print(f"内側は外側を参照しない　不合格　{len(違反)}件")
    for path, n, mine, peer, lines in violations:
        print(f"  {道}:{n}　{mine} → {相手}　{行}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
