"""外側 ── 受け取りアダプタ。HTTP の言葉を、内側の言葉へ直すだけである。"""
from __future__ import annotations

import json
from http.server import BaseHTTPRequestHandler

from domain import answer, submit
from usecase import receive_answer


def compose(port):
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            n = int(self.headers.get("content-length", 0))
            body = json.loads(self.rfile.read(n) or b"{}")
            try:
                s = submit(tuple(answer(a["no"], a["verdict"], a.get("reason", ""))
                              for a in body.get("answers") or []))
            except ValueError as e:
                self.send_error(400, str(e))
                return
            count = receive_answer(port, s)
            out = json.dumps({"saved": count}).encode()
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(out)))
            self.end_headers()
            self.wfile.write(out)
    return Handler
