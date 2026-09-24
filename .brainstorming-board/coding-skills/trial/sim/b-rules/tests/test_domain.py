"""内側の試験 ── HTTP もファイルも立てない。ポートは偽物で差し替える。"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from domain import answer, submit, is_duplicate          # noqa: E402
from usecase import receive_answer              # noqa: E402


class MemoryStore:
    """テストで差し替える実装 ── ポートを作った理由がこれである。"""

    def __init__(self): self.保存 = []
    def latest(self): return self.保存[-1] if self.保存 else None
    def add(self, s): self.保存.append(s)


def try_it():
    first = submit((answer(1, "approve"),))
    second = submit((answer(1, "return", "理由"),))

    assert is_duplicate(None, first) is False
    assert is_duplicate(first, first) is True
    assert is_duplicate(first, second) is False

    port = MemoryStore()
    assert receive_answer(port, first) == 1, "1件目が保存されない"
    assert receive_answer(port, first) == 0, "直前と同じなのに保存した"
    assert receive_answer(port, second) == 1, "内容が違うのに保存しない"

    try:
        submit((answer(1, "approve"), answer(1, "return")))
    except ValueError:
        pass
    else:
        raise AssertionError("論点1件につき2つ入っても通った")

    print("規則あり版　内側　7件すべて通った")


if __name__ == "__main__":
    try_it()
