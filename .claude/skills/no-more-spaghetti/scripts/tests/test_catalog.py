# SPDX-License-Identifier: MIT
"""索引から規則ファイルを解決する契約を、事例で検証する。

    python3 scripts/tests/test_catalog.py
"""
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import catalog  # noqa: E402

count = 0


def expect(name: str, ok: bool) -> None:
    global count
    count += 1
    if not ok:
        raise AssertionError(name)


def make(root: pathlib.Path, entries: list[dict], files: list[str]) -> None:
    (root / catalog.RULES_DIR).mkdir(parents=True, exist_ok=True)
    for f in files:
        p = root / catalog.RULES_DIR / f
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text('{"rules": []}', encoding="utf-8")
    (root / catalog.INDEX_PATH).write_text(
        json.dumps({"entries": entries}, ensure_ascii=False), encoding="utf-8")


def a_name_resolves_to_a_file() -> None:
    """索引に在る名前から、規則ファイルの経路を得る。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        make(root, [{"name": "repo", "rules": "applied/repo.json"},
                    {"name": "acdr", "rules": "applied/skills/acdr.json"}],
             ["applied/repo.json", "applied/skills/acdr.json"])
        expect("名前から解決する",
               catalog.resolve(root, "acdr") == root / ".coding-rules/applied/skills/acdr.json")
        expect("名前の一覧を返す", catalog.names(root) == ["repo", "acdr"])


def a_path_is_taken_as_is() -> None:
    """経路をそのまま渡した場合は、索引を引かない。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        make(root, [{"name": "repo", "rules": "applied/repo.json"}], ["applied/repo.json"])
        p = root / ".coding-rules/applied/repo.json"
        expect("経路はそのまま", catalog.resolve(root, str(p)) == p)


def an_unknown_name_is_refused() -> None:
    """索引に無い名前は、名前を挙げて断る。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        make(root, [{"name": "repo", "rules": "applied/repo.json"}], ["applied/repo.json"])
        try:
            catalog.resolve(root, "無い名前")
            expect("知らない名前は断る", False)
        except FileNotFoundError as e:
            expect("名前の一覧を添える", "repo" in str(e))


def a_missing_index_is_refused() -> None:
    """索引が無ければ、作り方を添えて断る。"""
    with tempfile.TemporaryDirectory() as t:
        try:
            catalog.resolve(pathlib.Path(t), "repo")
            expect("索引が無ければ断る", False)
        except FileNotFoundError as e:
            expect("索引の経路を添える", ".coding-rules/index.json" in str(e))


def the_index_is_checked() -> None:
    """索引そのものを検査する ── 名前の重複と、実在しない経路。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        make(root, [{"name": "a", "rules": "applied/a.json"},
                    {"name": "a", "rules": "applied/b.json"},
                    {"name": "c", "rules": "applied/無い.json"}],
             ["applied/a.json", "applied/b.json"])
        findings = catalog.check_index(root)
        expect("名前の重複を検出する", any("重複" in x for x in findings))
        expect("実在しない経路を検出する", any("実在しない" in x for x in findings))


def a_broken_schema_path_is_reported() -> None:
    """$schema が解決しない規則ファイルを検出する。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        make(root, [{"name": "a", "rules": "applied/a.json"}], ["applied/a.json"])
        p = root / ".coding-rules/applied/a.json"
        p.write_text('{"$schema": "../無い.json", "rules": []}', encoding="utf-8")
        findings = catalog.check_index(root)
        expect("解決しない $schema を検出する", any("$schema" in x for x in findings))

        p.write_text('{"$schema": "../index.json", "rules": []}', encoding="utf-8")
        (root / ".coding-rules/index.json").touch()
        expect("解決する $schema は検出しない",
               not any("$schema" in x for x in catalog.check_index(root)))


if __name__ == "__main__":
    for f in (a_name_resolves_to_a_file, a_path_is_taken_as_is, an_unknown_name_is_refused,
              a_missing_index_is_refused, the_index_is_checked,
              a_broken_schema_path_is_reported):
        f()
    print(f"索引の検査　{count} 件　通った")
