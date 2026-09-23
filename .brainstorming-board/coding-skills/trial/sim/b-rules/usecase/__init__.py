"""内側 ── 手順。外側の道具は、ポートを通してしか触らない。"""
from __future__ import annotations

from domain import 送信, 蓄積ポート, 重複か


def 回答を受け取る(ポート: 蓄積ポート, s: 送信) -> int:
    """保存した件数を返す。**直前と同じなら0件である。**"""
    if 重複か(ポート.直前(), s):
        return 0
    ポート.足す(s)
    return len(s.回答群)
