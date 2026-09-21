# SPDX-License-Identifier: MIT
"""デザイントークンを読み、CSS のカスタムプロパティへ組む。

    python3 tokens.py            検査して、通れば CSS を吐く
    python3 tokens.py --check    検査だけを実行する

**トークンの正本は1つである。** この道具が読むのは
`references/tokens.json` だけで、色も寸法もここから出る。
2か所に書くと、どちらが正しいかを毎回確認することになる。

**段を跨いだ参照を、形の段で弾く。** 意味の段は基礎の鍵だけを参照し、
部品の段は意味か基礎の鍵だけを参照する。散文の規定では破れる。
"""
from __future__ import annotations

import json
import pathlib
import sys

_REF = pathlib.Path(__file__).resolve().parent.parent / "references"
TOKENS = _REF / "tokens.json"
SCHEMA = _REF / "tokens.schema.json"

# 明暗の3つの選択子。**1つの表から生成する** ──
# 手で3か所へ書くと必ずずれる（実際に --shadow が [data-theme] の側から脱落していた）
_明の選択子 = ":root"
_暗の選択子 = ('@media (prefers-color-scheme:dark){:root:not([data-theme="light"])',
              ':root[data-theme="dark"]')


def load(path: pathlib.Path | None = None) -> dict:
    return json.loads((path or TOKENS).read_text(encoding="utf-8"))


def _原始値(t: dict) -> dict[str, str]:
    """基礎の段を、1つの辞書へ畳む。鍵の重複はそこで判明する。"""
    out: dict[str, str] = {}
    for 種, 表 in t["基礎"].items():
        for k, v in 表.items():
            if k in out:
                raise ValueError(f"基礎の鍵が重複している: {k}（{種}）")
            out[k] = v
    return out


def validate(t: dict, *, スキーマも: bool = True) -> list[str]:
    """形と、段を跨ぐ参照を検査する。**0件になるものだけを検査する。**"""
    err: list[str] = []
    if スキーマも:
        try:
            import jsonschema
        except ModuleNotFoundError:
            err.append("jsonschema が無いので、形の検査を実行していない")
        else:
            v = jsonschema.Draft202012Validator(
                json.loads(SCHEMA.read_text(encoding="utf-8")))
            for e in sorted(v.iter_errors(t), key=lambda x: list(x.path)):
                err.append("形: " + "/".join(map(str, e.path)) + " ── " + e.message)
    try:
        基礎 = _原始値(t)
    except ValueError as e:
        return err + [str(e)]

    明, 暗 = t["意味"]["明"], t["意味"]["暗"]
    for k in sorted(set(明) ^ set(暗)):
        側 = "明" if k in 明 else "暗"
        err.append(f"意味の鍵 --{k} が {側} にしか無い ── 明暗で同じ鍵集合にする")
    for 側, 表 in (("明", 明), ("暗", 暗)):
        for k, ref in 表.items():
            if ref not in 基礎:
                err.append(f"意味 {側} の --{k} が、基礎に無い鍵「{ref}」を参照している")
    for k, ref in t["部品"].items():
        if ref.startswith("calc(") or ref[0].isdigit() or ref[0] == ".":
            continue
        if ref not in 基礎 and ref not in 明:
            err.append(f"部品 --{k} が、意味にも基礎に無い鍵「{ref}」を参照している")
    return err


def _行(表: dict[str, str], 基礎: dict[str, str]) -> str:
    return "".join(f"--{k}:{基礎[v]};" for k, v in 表.items())


# 寸法の系は、名前のまま CSS 変数へ出す ── 色は意味の段を経由するが、
# 寸法・字寸・字送り・角丸・枠は役割ではなく段そのものが意味である
_寸法の系 = ("寸法", "字寸", "字送り", "角丸", "枠")


def css(t: dict) -> str:
    """3つの選択子と、寸法の系と、部品の段を組む。"""
    基礎 = _原始値(t)
    寸 = "".join(f"--{k}:{v};" for 種 in _寸法の系 for k, v in t["基礎"][種].items())
    部品 = []
    for k, ref in t["部品"].items():
        値 = ref if (ref.startswith("calc(") or ref[0].isdigit() or ref[0] == ".") \
            else f"var(--{ref})"
        部品.append(f"--{k}:{値};")
    明 = _行(t["意味"]["明"], 基礎) + 寸 + "".join(部品)
    暗 = _行(t["意味"]["暗"], 基礎)
    return (f"{_明の選択子}{{{明}}}"
            f"{_暗の選択子[0]}{{{暗}}}}}"
            f"{_暗の選択子[1]}{{{暗}}}")


def 参照と定義(組み上がり: str) -> tuple[set[str], set[str]]:
    """組み上がりから、参照した鍵と定義した鍵を抜く。差を0件にするために使う。"""
    import re
    # 代替値を持つ参照（var(--x,#fff)）は、定義が無くても崩壊しない ── 別に数える
    参照 = {m.group(1) for m in re.finditer(r"var\(\s*(--[a-z0-9-]+)\s*([,)])",
                                           組み上がり) if m.group(2) == ")"}
    定義 = set(re.findall(r"(--[a-z0-9-]+)\s*:", 組み上がり))
    return 参照, 定義


def main() -> int:
    t = load()
    err = validate(t)
    if err:
        for e in err:
            print("  ×", e, file=sys.stderr)
        print(f"トークンの検査　通っていない（{len(err)} 件）", file=sys.stderr)
        return 1
    if "--check" in sys.argv:
        print(f"トークンの検査　通った　／　基礎 {len(_原始値(t))} ・ "
              f"意味 {len(t['意味']['明'])} ・ 部品 {len(t['部品'])}")
        return 0
    print(css(t))
    return 0


if __name__ == "__main__":
    sys.exit(main())
