"""組み立て ── ここだけが、内側と外側の両方を知る。"""
import pathlib
from http.server import HTTPServer

from adapters.sqlite_store import SqliteStore
from adapters.http_in import compose

if __name__ == "__main__":
    port = SqliteStore("answers.db")
    HTTPServer(("127.0.0.1", 8802), compose(port)).serve_forever()
