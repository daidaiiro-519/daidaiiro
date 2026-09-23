"""組み立て ── ここだけが、内側と外側の両方を知る。"""
import pathlib
from http.server import HTTPServer

from adapters.sqlite_store import SQLiteの蓄積先
from adapters.http_in import 組む

if __name__ == "__main__":
    口 = SQLiteの蓄積先("answers.db")
    HTTPServer(("127.0.0.1", 8802), 組む(口)).serve_forever()
