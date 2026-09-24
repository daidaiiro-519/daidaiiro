"""外側 ── 蓄積ポートを実装する。SQLite に1行ずつ書く。"""
from __future__ import annotations

import json
import sqlite3

from domain import answer, submit


class SqliteStore:
    def __init__(self, path: str):
        self.c = sqlite3.connect(path)
        self.c.execute("create table if not exists submissions "
                       "(id integer primary key, body text)")

    def latest(self) -> submit | None:
        row = self.c.execute("select body from submissions "
                             "order by id desc limit 1").fetchone()
        if row is None:
            return None
        d = json.loads(row[0])
        return submit(tuple(answer(**a) for a in d["回答群"]))

    def add(self, s: submit) -> None:
        d = {"回答群": [vars(a) for a in s.answers]}
        self.c.execute("insert into submissions (body) values (?)",
                       (json.dumps(d, ensure_ascii=False),))
        self.c.commit()
