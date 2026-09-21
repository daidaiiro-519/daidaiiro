# SPDX-License-Identifier: MIT
"""build_acdr の振る舞いを事例で検証する。`python3 test_acdr.py` で走る。

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
CLI = HERE / "build_acdr.py"


def run(*args: str, cwd: str | None = None) -> subprocess.CompletedProcess:
    return subprocess.run([sys.executable, str(CLI), *args], cwd=cwd,
                          capture_output=True, text=True)


def minimal(folder: pathlib.Path, doc: pathlib.Path) -> None:
    """通る最小の記録を作る。図も docs も持つ。"""
    (folder / "flow.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"></svg>', encoding="utf-8")
    spec = {
        "番号": "ACDR 0001", "題": "ためし", "日付": "2026-01-01", "状態": "proposed",
        "決定": "決定を記述する", "なぜ": "理由を記述する", "適用先": "適用先を記述する",
        "形の変化": [{"何が": "甲", "いまの形": "乙", "これからの形": "丙"}],
        "図ファイル": "flow.svg", "図の説明": "説明",
        "比較した案": [{"案": "別案", "採らなかった理由": "成立しない"}],
        "承認後に実施すること": ["作業1"],
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
            self.assertEqual(spec["題"], "ためしの決定")
            self.assertEqual(spec["番号"], "ACDR 0007")
            self.assertEqual(spec["状態"], "proposed")

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
            del spec["なぜ"]
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            got = run(str(f))
            self.assertNotEqual(got.returncode, 0)
            self.assertIn("なぜ", got.stderr)

    def test_三つ組が欠けた変更で止まる(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["docs"][0]["marks"][0]["why"] = ""
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            got = run(str(f))
            self.assertNotEqual(got.returncode, 0)
            self.assertIn("3つ組", got.stderr)

    def test_状態は三つだけ(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["状態"] = "だいたい承認"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertNotEqual(run(str(f)).returncode, 0)


class Idempotent(unittest.TestCase):
    def test_二度組んで同じものが出る(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            self.assertEqual(run(str(f)).returncode, 0)
            first = (f / "index.html").read_text(encoding="utf-8")
            self.assertEqual(run(str(f)).returncode, 0)
            self.assertEqual(first, (f / "index.html").read_text(encoding="utf-8"))
            self.assertEqual(run(str(f), "--check").returncode, 0)

    def test_実行場所に依存しない(self):
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as other:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            run(str(f), cwd=d)
            first = (f / "index.html").read_text(encoding="utf-8")
            run(str(f), cwd=other)
            self.assertEqual(first, (f / "index.html").read_text(encoding="utf-8"))

    def test_中身が変われば差が出る(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            run(str(f))
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["決定"] = "別の決定を記述する"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertEqual(run(str(f), "--check").returncode, 1)


class Seal(unittest.TestCase):
    def test_承認済みは封印される(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["状態"] = "accepted"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            self.assertEqual(run(str(f)).returncode, 0)
            sealed = json.loads((f / "acdr.json").read_text(encoding="utf-8")).get("封印")
            self.assertTrue(sealed)

    def test_対象が変化したら組み直しを拒否する(self):
        with tempfile.TemporaryDirectory() as d:
            f = pathlib.Path(d)
            doc = f / "対象.md"
            doc.write_text("# 題\n\n本文である。\n", encoding="utf-8")
            minimal(f, doc)
            spec = json.loads((f / "acdr.json").read_text(encoding="utf-8"))
            spec["状態"] = "accepted"
            (f / "acdr.json").write_text(json.dumps(spec, ensure_ascii=False), encoding="utf-8")
            run(str(f))
            before = (f / "index.html").read_text(encoding="utf-8")
            doc.write_text("# 題\n\n本文を改訂した。\n", encoding="utf-8")
            got = run(str(f))
            self.assertEqual(got.returncode, 2)
            self.assertEqual(before, (f / "index.html").read_text(encoding="utf-8"))
            self.assertEqual(run(str(f), "--check").returncode, 0)
            self.assertEqual(run(str(f), "--force").returncode, 0)
            self.assertNotEqual(before, (f / "index.html").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
