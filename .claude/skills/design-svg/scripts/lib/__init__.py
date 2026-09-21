# SPDX-License-Identifier: MIT
"""design-svg の部品。**入口を保持しない** ── 外から起動するのは `cli.py` だけである。

描画エンジンの実体は `svg_engine/` である。`scripts/` を道に載せれば
`import svg_engine` で読める ── 包みの名前は変えない（配布物の名前でもある）。
"""
import pathlib as _pathlib
import sys as _sys

# **置き場所を、階層の数で数えない** ── 部品を動かすたびに数が狂う
REFERENCES = next((d / "references" for d in _pathlib.Path(__file__).resolve().parents
                   if (d / "references").is_dir()), None)

# 包みを、この場所から読めるようにする
_sys.path.insert(0, str(_pathlib.Path(__file__).resolve().parent))
