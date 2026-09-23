"""規則なし版 ── 依頼を読んで、そのまま書いた。（保存先を SQLite へ変えた）"""
import json
import sqlite3
from http.server import BaseHTTPRequestHandler, HTTPServer

DB = "answers.db"


def 接続():
    c = sqlite3.connect(DB)
    c.execute("create table if not exists submissions (id integer primary key, body text)")
    return c


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        n = int(self.headers.get("content-length", 0))
        body = json.loads(self.rfile.read(n) or b"{}")
        answers = body.get("answers") or []

        # 論点1件につき1つ
        seen = set()
        for a in answers:
            if a["no"] in seen:
                self.send_error(400, "duplicate topic")
                return
            seen.add(a["no"])

        # 直前と同じなら保存しない
        c = 接続()
        row = c.execute("select body from submissions order by id desc limit 1").fetchone()
        last = json.loads(row[0]) if row else None
        if last is not None and last.get("answers") == answers:
            c.close()
            self.respond(0)
            return

        c.execute("insert into submissions (body) values (?)",
                  (json.dumps({"answers": answers}, ensure_ascii=False),))
        c.commit()
        c.close()
        self.respond(len(answers))

    def respond(self, saved):
        out = json.dumps({"saved": saved}).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(out)))
        self.end_headers()
        self.wfile.write(out)


if __name__ == "__main__":
    HTTPServer(("127.0.0.1", 8801), Handler).serve_forever()
