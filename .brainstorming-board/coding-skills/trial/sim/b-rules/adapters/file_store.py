"""外側 ── 蓄積ポートを実装する。ファイルに1行ずつ書く。"""
from __future__ import annotations

import json
import pathlib

from domain import answer, submit


class FileStore:
    def __init__(self, path: pathlib.Path):
        self.path = path

    def latest(self) -> submit | None:
        if not self.path.exists():
            return None
        lines = self.path.read_text(encoding="utf-8").splitlines()
        if not lines:
            return None
        d = json.loads(lines[-1])
        return submit(tuple(answer(**a) for a in d["回答群"]))

    def add(self, s: submit) -> None:
        d = {"回答群": [vars(a) for a in s.answers]}
        with self.path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(d, ensure_ascii=False) + "\n")
