"""内側の試験 ── HTTP もファイルも立てない。ポートは偽物で差し替える。"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from domain import 回答, 送信, 重複か          # noqa: E402
from usecase import 回答を受け取る              # noqa: E402


class 覚えておく蓄積先:
    """テストで差し替える実装 ── ポートを作った理由がこれである。"""

    def __init__(self): self.保存 = []
    def 直前(self): return self.保存[-1] if self.保存 else None
    def 足す(self, s): self.保存.append(s)


def 試す():
    一 = 送信((回答(1, "approve"),))
    二 = 送信((回答(1, "return", "理由"),))

    assert 重複か(None, 一) is False
    assert 重複か(一, 一) is True
    assert 重複か(一, 二) is False

    ポート = 覚えておく蓄積先()
    assert 回答を受け取る(ポート, 一) == 1, "1件目が保存されない"
    assert 回答を受け取る(ポート, 一) == 0, "直前と同じなのに保存した"
    assert 回答を受け取る(ポート, 二) == 1, "内容が違うのに保存しない"

    try:
        送信((回答(1, "approve"), 回答(1, "return")))
    except ValueError:
        pass
    else:
        raise AssertionError("論点1件につき2つ入っても通った")

    print("規則あり版　内側　7件すべて通った")


if __name__ == "__main__":
    試す()
