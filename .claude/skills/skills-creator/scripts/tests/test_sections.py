# SPDX-License-Identifier: MIT
"""節の構成が、対応する雛形を満たすかを事例で検証する。

    python3 scripts/tests/test_sections.py
"""
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import sections  # noqa: E402

count = 0


def expect(name: str, ok: bool) -> None:
    global count
    count += 1
    if not ok:
        raise AssertionError(name)


def write(dirpath: pathlib.Path, name: str, names: list[str]) -> pathlib.Path:
    p = dirpath / name
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text("\n\n".join(f"## {x}\n\n本文" for x in names) + "\n", encoding="utf-8")
    return p


def headings_are_read_in_order() -> None:
    """節の名前を、並びのまま取り出す。"""
    with tempfile.TemporaryDirectory() as t:
        p = write(pathlib.Path(t), "SKILL.md", ["目的", "役割", "参照"])
        expect("見出しの並びを返す", sections.headings(p) == ["目的", "役割", "参照"])


def missing_sections_are_reported() -> None:
    """雛形に在って文書に無い節を、名前で返す。"""
    with tempfile.TemporaryDirectory() as t:
        d = pathlib.Path(t)
        template = write(d, "template.md", ["目的", "役割", "出力形式"])
        document = write(d, "SKILL.md", ["目的", "役割"])
        missing = sections.missing(document, template)
        expect("欠けた節を返す", missing == ["出力形式"])
        expect("満たしていれば0件", sections.missing(template, template) == [])


def placeholders_in_the_template_are_ignored() -> None:
    """雛形の差し込む場所は、節として数えない。"""
    with tempfile.TemporaryDirectory() as t:
        d = pathlib.Path(t)
        template = d / "template.md"
        template.write_text("## 目的\n\n本文\n\n## {{節の名前}}\n\n本文\n", encoding="utf-8")
        document = write(d, "SKILL.md", ["目的"])
        expect("差し込む場所を要求しない", sections.missing(document, template) == [])


def the_order_is_not_required() -> None:
    """節の並び順は問わない ── 有無だけを見る。"""
    with tempfile.TemporaryDirectory() as t:
        d = pathlib.Path(t)
        template = write(d, "template.md", ["目的", "役割"])
        document = write(d, "SKILL.md", ["役割", "目的"])
        expect("並びが違っても0件", sections.missing(document, template) == [])


def an_unreadable_file_raises() -> None:
    """読めないものは、誤用として投げる。"""
    with tempfile.TemporaryDirectory() as t:
        d = pathlib.Path(t)
        template = write(d, "template.md", ["目的"])
        try:
            sections.missing(d / "無い.md", template)
            expect("読めなければ投げる", False)
        except OSError:
            expect("読めなければ投げる", True)


if __name__ == "__main__":
    for f in (headings_are_read_in_order, missing_sections_are_reported,
              placeholders_in_the_template_are_ignored, the_order_is_not_required,
              an_unreadable_file_raises):
        f()
    print(f"節の検査　{count} 件　通った")
