"""外側 ── 蓄積ポートを実装する。ファイルに1行ずつ書く。"""
from __future__ import annotations

import json
import pathlib

from domain import 回答, 送信


class ファイルの蓄積先:
    def __init__(self, path: pathlib.Path):
        self.path = path

    def 直前(self) -> 送信 | None:
        if not self.path.exists():
            return None
        行 = self.path.read_text(encoding="utf-8").splitlines()
        if not 行:
            return None
        d = json.loads(行[-1])
        return 送信(tuple(回答(**a) for a in d["回答群"]))

    def 足す(self, s: 送信) -> None:
        d = {"回答群": [vars(a) for a in s.回答群]}
        with self.path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(d, ensure_ascii=False) + "\n")
