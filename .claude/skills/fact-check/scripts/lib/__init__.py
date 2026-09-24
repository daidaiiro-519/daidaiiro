# SPDX-License-Identifier: MIT
"""fact-check の部品。**入口を保持しない** ── 外から起動するのは `cli.py` だけである。

ここに置くのは、読み込まれて使われるものである。能力の宣言は `tools.py` が、
入口は `cli.py` が保持する。
"""
import pathlib as _pathlib

# **置き場所を、階層の数で数えない** ── 部品を動かすたびに数が狂う
REFERENCES = next((d / "references" for d in _pathlib.Path(__file__).resolve().parents
                   if (d / "references").is_dir()), None)
