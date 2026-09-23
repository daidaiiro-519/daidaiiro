"""層の向きを検査する ── 言語に依存しない試作。標準ライブラリだけで動く。

入力は2つである。
  記録　　層の名前と、その置き場所。並び（内から外）
  形　　　参照を書く行の形。言語ごとに正規表現1〜2本

処理は3段である。
  ①各ファイルの層を、置き場所から決める
  ②参照の行を拾い、参照先の文字列から相手の層を決める
  ③並びに違反した辺を出す

**解析器は書かない。** 参照の行を拾うだけで、名前の解決は実施しない ──
解決は言語ごとに規則が違うので、そこへ入ると言語に依存する。
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

形 = {
    ".py":  [r"^\s*from\s+([A-Za-z_][\w\.]*)", r"^\s*import\s+([A-Za-z_][\w\.]*)"],
    ".ts":  [r"""from\s+['"]([^'"]+)['"]""", r"""require\(\s*['"]([^'"]+)['"]"""],
    ".tsx": [r"""from\s+['"]([^'"]+)['"]""", r"""require\(\s*['"]([^'"]+)['"]"""],
    ".js":  [r"""from\s+['"]([^'"]+)['"]""", r"""require\(\s*['"]([^'"]+)['"]"""],
    ".go":  [r'^\s*"([^"]+)"', r'^\s*[\w\.]+\s+"([^"]+)"'],
    ".rs":  [r"^\s*(?:pub\s+)?use\s+([\w:]+)"],
    ".java": [r"^\s*import\s+(?:static\s+)?([\w\.]+)"],
}


def 層を決める(道: str, 層: dict[str, str]) -> str | None:
    """置き場所の前方一致で決める。**長い方を先に見る** ── 入れ子の層が在るため。"""
    道 = 道.replace("\\", "/")
    for 名, 場所 in sorted(層.items(), key=lambda kv: -len(kv[1])):
        場所 = 場所.strip("/")
        if 道 == 場所 or 道.startswith(場所 + "/"):
            return 名
    return None


def 参照先の層(参照: str, 層: dict[str, str]) -> str | None:
    """参照の文字列に、層の置き場所の末尾の名前が現れるかで決める。"""
    語 = re.split(r"[\./:\\]+", 参照)
    語 = [x for x in 語 if x]
    for 名, 場所 in 層.items():
        末尾 = 場所.strip("/").split("/")[-1]
        if 名 in 語 or 末尾 in 語:
            return 名
    return None


def 検査する(根: pathlib.Path, 記録: dict) -> list[tuple[str, int, str, str, str]]:
    層 = 記録["層"]
    並び = 記録["並び"]                      # 内から外
    位置 = {名: i for i, 名 in enumerate(並び)}
    違反 = []
    for p in sorted(根.rglob("*")):
        if not p.is_file() or p.suffix not in 形:
            continue
        道 = str(p.relative_to(根))
        自分 = 層を決める(道, 層)
        if 自分 is None:
            continue
        for n, 行 in enumerate(p.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            if 行.lstrip().startswith(("#", "//")):
                continue
            for 式 in 形[p.suffix]:
                m = re.search(式, 行)
                if not m:
                    continue
                相手 = 参照先の層(m.group(1), 層)
                if 相手 is None or 相手 == 自分:
                    continue
                # **内側が外側を参照したら違反である** ── 並びは内から外である
                if 位置[相手] > 位置[自分]:
                    違反.append((道, n, 自分, 相手, 行.strip()))
    return 違反


def main() -> int:
    根 = pathlib.Path(sys.argv[1])
    記録 = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
    違反 = 検査する(根, 記録)
    if not 違反:
        print("内側は外側を参照しない　合格")
        return 0
    print(f"内側は外側を参照しない　不合格　{len(違反)}件")
    for 道, n, 自分, 相手, 行 in 違反:
        print(f"  {道}:{n}　{自分} → {相手}　{行}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
