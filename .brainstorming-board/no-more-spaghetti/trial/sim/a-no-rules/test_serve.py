"""規則なし版の試験 ── HTTP を立てないと1件も試せない。"""
import json
import pathlib
import threading
import urllib.request
from http.server import HTTPServer

import serve


def send(port, answers):
    body = json.dumps({"answers": answers}).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{port}/", data=body,
                                 headers={"content-type": "application/json"})
    with urllib.request.urlopen(req) as r:
        return json.loads(r.read())


def build(tmp):
    serve.OUT = tmp / "answers.jsonl"
    s = HTTPServer(("127.0.0.1", 0), serve.Handler)
    threading.Thread(target=s.serve_forever, daemon=True).start()
    return s


def try_it():
    tmp = pathlib.Path("/tmp/sim-a"); tmp.mkdir(exist_ok=True)
    for f in tmp.glob("*"): f.unlink()
    s = build(tmp)
    port = s.server_address[1]
    first = [{"no": 1, "verdict": "approve", "reason": ""}]
    assert send(port, first)["saved"] == 1, "1件目が保存されない"
    assert send(port, first)["saved"] == 0, "直前と同じなのに保存した"
    second = [{"no": 1, "verdict": "return", "reason": "理由"}]
    assert send(port, second)["saved"] == 1, "内容が違うのに保存しない"
    s.shutdown()
    print("規則なし版　3件すべて通った")


if __name__ == "__main__":
    try_it()
