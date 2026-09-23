"""外側 ── 蓄積の口を実装する。SQLite に1行ずつ書く。"""
from __future__ import annotations

import json
import sqlite3

from domain import 回答, 送信


class SQLiteの蓄積先:
    def __init__(self, path: str):
        self.c = sqlite3.connect(path)
        self.c.execute("create table if not exists submissions "
                       "(id integer primary key, body text)")

    def 直前(self) -> 送信 | None:
        row = self.c.execute("select body from submissions "
                             "order by id desc limit 1").fetchone()
        if row is None:
            return None
        d = json.loads(row[0])
        return 送信(tuple(回答(**a) for a in d["回答群"]))

    def 足す(self, s: 送信) -> None:
        d = {"回答群": [vars(a) for a in s.回答群]}
        self.c.execute("insert into submissions (body) values (?)",
                       (json.dumps(d, ensure_ascii=False),))
        self.c.commit()
