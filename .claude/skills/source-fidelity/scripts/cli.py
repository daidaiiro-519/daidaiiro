# SPDX-License-Identifier: MIT
"""source-fidelity の唯一の入口。

    python3 scripts/cli.py <動詞> [対象…] [--json]

**道具ごとに入口を作らない** ── 入口が増えると、呼ぶ側が形を推測することになる。
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from contract import main  # noqa: E402
from tools import TOOLS  # noqa: E402

if __name__ == "__main__":
    raise SystemExit(main(TOOLS))
