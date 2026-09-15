#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""承認の画面を配り、押された回答を1件ずつファイルに落とす小さなサーバー。

    python3 serve.py <配るディレクトリ> [--port 8731] [--answers <積む先>]

画面からの POST /answer を受け、本文（JSON）を `<answers>/<日時>.json` に書く。
積む先に `{board}` を書くと、本文の board の値へ置き換わる ── 回答は、
そのブレストの持ち物だからである（例: `.brainstorm/{board}/answers`）。
回答は消さない ── 何を差し戻したかが、あとから順に読めるようにするためである。

形は answer-sheet.schema.json が決めるが、ここでは**素の形だけ**を見る
（必須の欄が在るか、諾否が決められた値か）。中身が妥当かは機械には分からない。
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

VERDICTS = {"approved", "returned", "unanswered"}
# 差し戻しの理由は自由文である。これは画面がボタンで差し込む「よく使う理由」で、値を限定するものではない。
COMMON_REASONS = ("もっと単純に", "前提が違う", "別の道も見たい")


def _fault(body: dict) -> list[str]:
    """回答1件の素の形を見る。返るのは、直すべきことの一覧（空なら通る）。"""
    bad: list[str] = []
    if not isinstance(body, dict):
        return ["いちばん外側が object ではない"]
    for key in ("board", "answeredAt", "answers"):
        if key not in body:
            bad.append(f"{key} が無い")
    answers = body.get("answers")
    if not isinstance(answers, list) or not answers:
        bad.append("answers が、1件以上の配列ではない")
        return bad
    for i, a in enumerate(answers):
        where = f"answers[{i}]"
        if not isinstance(a, dict):
            bad.append(f"{where} が object ではない")
            continue
        if not a.get("questionId"):
            bad.append(f"{where}.questionId が無い")
        verdict = a.get("verdict")
        if verdict not in VERDICTS:
            bad.append(f"{where}.verdict が {sorted(VERDICTS)} のどれでもない（{verdict!r}）")
        reason = a.get("returnReason")
        # 理由は自由文である。決められた値に限ると、言えない理由が出たときに書けない。
        if verdict == "returned" and not (isinstance(reason, str) and reason.strip()):
            bad.append(f"{where}.returnReason が空（差し戻すなら理由を書く）")
        if verdict != "returned" and reason not in (None, ""):
            bad.append(f"{where}.returnReason は、差し戻し以外では空でなければならない")
    # 論点に対して決定は1つなので、同じ問いが2件入っていたら受け取らない。
    ids = [a.get("questionId") for a in answers if isinstance(a, dict)]
    dup = sorted({i for i in ids if i and ids.count(i) > 1})
    if dup:
        bad.append(f"同じ問いが2件以上入っている（{'・'.join(dup)}）── 論点に対して決定は1つである")
    return bad


class Handler(SimpleHTTPRequestHandler):
    answers_dir: Path = Path("answers")

    def end_headers(self) -> None:  # 文字コードを言わないと、日本語が化ける
        if self.path.endswith(".html") or self.path.endswith("/"):
            self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def guess_type(self, path):  # type: ignore[override]
        t = super().guess_type(path)
        return t + "; charset=utf-8" if t in ("text/html", "text/plain", "text/css") else t

    def _json(self, code: int, payload: dict) -> None:
        raw = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def _dir_for(self, body: dict) -> Path | None:
        """積む先を決める。`{board}` は本文の board の値へ置き換える。"""
        raw = str(self.answers_dir)
        if "{board}" not in raw:
            return Path(raw)
        board = str(body.get("board") or "")
        # 置き場所の名前になるので、区切りや親への遡りは受け取らない
        if not board or "/" in board or "\\" in board or board.startswith("."):
            return None
        return Path(raw.replace("{board}", board))

    def do_POST(self) -> None:  # noqa: N802  （標準ライブラリの綴りに合わせる）
        if self.path.rstrip("/") != "/answer":
            self._json(404, {"error": "そのあて先は受け付けていない", "path": self.path})
            return
        length = int(self.headers.get("Content-Length") or 0)
        if length <= 0:
            self._json(400, {"error": "本文が空"})
            return
        try:
            body = json.loads(self.rfile.read(length).decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as e:
            self._json(400, {"error": "JSON として読めない", "detail": str(e)})
            return

        bad = _fault(body)
        if bad:
            self._json(422, {"error": "形が合っていない", "faults": bad})
            return

        answers_dir = self._dir_for(body)
        if answers_dir is None:
            self._json(422, {"error": "board の値が、置き場所の名前として使えない",
                             "board": body.get("board")})
            return
        answers_dir.mkdir(parents=True, exist_ok=True)

        # 同じ内容が直前に届いていたら、積まない。
        # 押し直しや二重発火で同じ回答が並ぶと、どれが最後の返答かを読む側が判断できなくなる。
        prev = sorted(answers_dir.glob("*.json"))
        if prev:
            try:
                if json.loads(prev[-1].read_text(encoding="utf-8")) == body:
                    self._json(200, {"saved": str(prev[-1]), "count": len(body["answers"]),
                                     "next": "直前と同じ内容だったので、積まずに済ませた"})
                    return
            except (UnicodeDecodeError, json.JSONDecodeError, OSError):
                pass  # 読めない古い記録は、判定に使わない

        name = datetime.now().strftime("%Y%m%dT%H%M%S") + ".json"
        out = answers_dir / name
        out.write_text(json.dumps(body, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        n = len(body["answers"])
        self._json(200, {"saved": str(out), "count": n,
                         "next": "Claude に一言送ると、この回答が読まれます"})

    def log_message(self, fmt: str, *args) -> None:  # 配信のたびに1行出すのは、うるさい
        if self.command == "POST":
            sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("root", nargs="?", default=".", help="画面を置いてあるディレクトリ")
    p.add_argument("--port", type=int, default=8731)
    p.add_argument("--answers", default="answers", help="回答を積む先。置き場所は呼び出し側が決める")
    a = p.parse_args()

    Handler.answers_dir = Path(a.answers).resolve()
    root = Path(a.root).resolve()
    if not root.is_dir():
        print(f"配るディレクトリが無い: {root}", file=sys.stderr)
        return 1

    handler = lambda *args, **kw: Handler(*args, directory=str(root), **kw)  # noqa: E731
    with ThreadingHTTPServer(("127.0.0.1", a.port), handler) as httpd:
        print(f"配る  : {root}")
        print(f"積む  : {Handler.answers_dir}")
        print(f"開く  : http://127.0.0.1:{a.port}/")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n止めました")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
