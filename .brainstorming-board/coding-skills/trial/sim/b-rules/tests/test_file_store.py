"""外側の試験 ── 蓄積先だけを試す。内側は1行も要らない。"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from adapters.file_store import ファイルの蓄積先   # noqa: E402
from domain import 回答, 送信                      # noqa: E402


def 試す():
    tmp = pathlib.Path("/tmp/sim-b"); tmp.mkdir(exist_ok=True)
    p = tmp / "answers.jsonl"
    p.unlink(missing_ok=True)
    口 = ファイルの蓄積先(p)
    assert 口.直前() is None
    s = 送信((回答(1, "approve", "よい"),))
    口.足す(s)
    assert 口.直前() == s, "書いたものが読めない"
    print("規則あり版　外側　2件すべて通った")


if __name__ == "__main__":
    試す()
