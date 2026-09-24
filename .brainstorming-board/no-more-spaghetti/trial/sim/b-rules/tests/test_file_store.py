"""外側の試験 ── 蓄積先だけを試す。内側は1行も要らない。"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from adapters.file_store import FileStore   # noqa: E402
from domain import answer, submit                      # noqa: E402


def try_it():
    tmp = pathlib.Path("/tmp/sim-b"); tmp.mkdir(exist_ok=True)
    p = tmp / "answers.jsonl"
    p.unlink(missing_ok=True)
    port = FileStore(p)
    assert port.latest() is None
    s = submit((answer(1, "approve", "よい"),))
    port.add(s)
    assert port.latest() == s, "書いたものが読めない"
    print("規則あり版　外側　2件すべて通った")


if __name__ == "__main__":
    try_it()
