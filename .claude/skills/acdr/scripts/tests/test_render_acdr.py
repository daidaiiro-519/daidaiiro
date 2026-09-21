# SPDX-License-Identifier: MIT
"""組み立ての振る舞いを事例で検証する。`python3 tests/test_render_acdr.py` で走る。

検証するのは4つ ── 雛形から起こせるか ／ 欠けた欄で止まるか ／
**同じ入力から同じ出力が出るか（冪等）** ／ 実行場所に依存しないか。
"""
from __future__ import annotations

import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
SCRIPTS = HERE.parent
sys.path.insert(0, str(SCRIPTS))
CLI = SCRIPTS / "cli.py"


def run(*args: str, cwd: str | None = None) -> subprocess.CompletedProcess:
    """唯一の入口を、動詞つきで呼ぶ。"""
    return subprocess.run([sys.executable, str(CLI), *args], cwd=cwd,
                          capture_output=True, text=True)


def build(folder, *extra: str, cwd: str | None = None) -> subprocess.CompletedProcess:
    """記録を1枚へ組む。"""
    return run("render", str(folder), *extra, cwd=cwd)


def minimal(folder: pathlib.Path, doc: pathlib.Path) -> None:
    """通る最小の記録を作る。図も docs も持つ。"""
    (folder / "flow.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"></svg>', encoding="utf-8")
    spec = {
        "no": "ACDR 0001", "title": "ためし", "date": "2026-01-01", "status": "proposed",
        "decision": "決定を記述する", "why": "理由を記述する", "applies_to": "適用先を記述する",
        "shift": [{"what": "甲", "from": "乙", "to": "丙"}],
        "figure": "flow.svg", "figure_caption": "説明",
        "alternatives": [{"option": "別案", "why_not": "成立しない"}],
        "after_approval": ["作業1"],
        "docs": [{"key": "d", "tab": "面", "file": str(doc),
                  "marks": [{"find": "本文", "before": "前", "why": "理由"}]}],
    }
    (folder / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")


class Template(unittest.TestCase):
    def test_雛形から起こせる(self):
        with tempfile.TemporaryDirectory() as d:
            got = run("new", os.path.join(d, "0007-ためし"), "ためしの決定")
            self.assertEqual(got.returncode, 0, got.stderr)
            spec = json.loads(
                pathlib.Path(d, "0007-ためし", "acdr.json").read_text(encoding="utf-8"))
            self.assertEqual(spec["title"], "ためしの決定")
            self.assertEqual(spec["no"], "ACDR 0007")
            self.assertEqual(spec["status"], "proposed")

    def test_同じ名前では起こさない(self):
        with tempfile.TemporaryDirectory() as d:
            run("new", os.path.join(d, "0007-ためし"), "甲")
            got = run("new", os.path.join(d, "0007-ためし"), "乙")
            self.assertNotEqual(got.returncode, 0)


class Validate(unittest.TestCase):
    def test_欠けた欄で止まる(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            del spec["why"]
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            got = build(f)
            self.assertNotEqual(got.returncode, 0)
            self.assertIn("why", got.stderr)

    def test_三つ組が欠けた変更で止まる(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["docs"][0]["marks"][0]["why"] = ""
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            got = build(f)
            self.assertNotEqual(got.returncode, 0)
            self.assertIn("3つ組", got.stderr)

    def test_状態は三つだけ(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["status"] = "だいたい承認"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertNotEqual(build(f).returncode, 0)


class Idempotent(unittest.TestCase):
    def test_二度組んで同じものが出る(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            self.assertEqual(build(f).returncode, 0)
            first = (f / "index.html").read_text(encoding="utf-8")
            self.assertEqual(build(f).returncode, 0)
            self.assertEqual(first, (f / "index.html").read_text(encoding="utf-8"))
            self.assertEqual(build(f, "--check", "1").returncode, 0)

    def test_実行場所に依存しない(self):
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as other:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            build(f, cwd=d)
            first = (f / "index.html").read_text(encoding="utf-8")
            build(f, cwd=other)
            self.assertEqual(first, (f / "index.html").read_text(encoding="utf-8"))

    def test_中身が変われば差が出る(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            build(f)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["decision"] = "別の決定を記述する"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertEqual(build(f, "--check", "1").returncode, 1)


class Code(unittest.TestCase):
    def test_コードは行の単位で印が付く(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.py"
            doc.write_text("# 注記である\nimport os\n\n\ndef 甲():\n    return 1\n",
                           encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["docs"][0]["marks"] = [{"find": "def 甲", "before": "def 乙", "why": "改称した"}]
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertEqual(build(f).returncode, 0)
            out = (f / "index.html").read_text(encoding="utf-8")
            self.assertIn('class="code"', out)
            self.assertIn('data-lang="python"', out)
            self.assertIn('class="t-k"', out)       # 予約語の色付け
            self.assertIn('class="t-c"', out)       # 注記の色付け
            self.assertIn('<td class="ln">5</td>', out)
            self.assertEqual(out.count('<mark class="chg"'), 1)

    def test_言語ごとに印が変わる(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.json"
            doc.write_text('{\n "甲": true\n}\n', encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["docs"][0]["marks"] = [{"find": "甲", "before": "乙", "why": "改称した"}]
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            build(f)
            out = (f / "index.html").read_text(encoding="utf-8")
            self.assertIn('data-lang="json"', out)

    def test_コードの面も押せるように配線される(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.py"
            doc.write_text("import os\n\n\ndef 甲():\n    return 1\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["docs"][0]["marks"] = [{"find": "def 甲", "before": "def 乙", "why": "改称した"}]
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            build(f)
            out = (f / "index.html").read_text(encoding="utf-8")
            # 押す処理の配線と、枠を行として挿入する経路が在ること
            self.assertIn(".pane.md, .pane.code-pane", out)
            self.assertIn('tr.className = "poprow"', out)
            self.assertIn("td.colSpan = row.children.length", out)

    def test_コード以外の拡張子はコードとして扱わない(self):
        from lib.code_diff import is_code
        self.assertTrue(is_code(".py"))
        self.assertTrue(is_code(".GO"))
        self.assertFalse(is_code(".md"))
        self.assertFalse(is_code(".html"))


class Diff(unittest.TestCase):
    def test_まとまりを組む(self):
        from lib.code_diff import hunks
        old = list("abcdefghi")
        new = list("abcDefghi")
        hs = hunks(old, new)
        self.assertEqual(len(hs), 1)
        self.assertEqual([m for _, _, m, _ in hs[0]], [" ", " ", " ", "-", "+", " ", " ", " "])

    def test_離れた変更は別のまとまりになる(self):
        from lib.code_diff import hunks
        old = [str(i) for i in range(40)]
        new = list(old)
        new[2] = "甲"
        new[30] = "乙"
        self.assertEqual(len(hunks(old, new)), 2)

    def test_理由の欠けを数える(self):
        from lib.code_diff import render_diff
        old = "a\nb\nc\n"
        new = "a\n甲\nc\n"
        html_, nh, nw = render_diff(old, new, ".py", [])
        self.assertEqual((nh, nw), (1, 0))
        self.assertIn("理由が付いていない", html_)
        html_, nh, nw = render_diff(old, new, ".py",
                                    [{"find": "甲", "why": "改称した"}])
        self.assertEqual((nh, nw), (1, 1))
        self.assertNotIn("理由が付いていない", html_)
        self.assertIn('class="chg"', html_)


class Seal(unittest.TestCase):
    def test_承認済みは封印される(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["status"] = "accepted"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertEqual(build(f).returncode, 0)
            sealed = json.loads((f / "acdr.json").read_text(encoding="utf-8")).get("seal")
            self.assertTrue(sealed)

    def test_対象が変化したら組み直しを拒否する(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["status"] = "accepted"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            build(f)
            before = (f / "index.html").read_text(encoding="utf-8")
            doc.write_text("# 題\n\n本文を改訂した。\n", encoding="utf-8")
            got = build(f)
            # **拒否は検出である。誤用ではない** ── 契約の終了コードは
            # 0 正常 ／ 1 検出あり ／ 2 誤用 であり、呼び方は誤っていない
            self.assertEqual(got.returncode, 1)
            self.assertEqual(before, (f / "index.html").read_text(encoding="utf-8"))
            self.assertEqual(build(f, "--check", "1").returncode, 0)
            self.assertEqual(build(f, "--force", "1").returncode, 0)
            self.assertNotEqual(before, (f / "index.html").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
