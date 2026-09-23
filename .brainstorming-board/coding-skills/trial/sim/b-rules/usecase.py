"""内側 ── 手順。外側の道具は、口を通してしか触らない。"""
from __future__ import annotations

from domain import 送信, 蓄積の口, 重複か


def 回答を受け取る(口: 蓄積の口, s: 送信) -> int:
    """保存した件数を返す。**直前と同じなら0件である。**"""
    if 重複か(口.直前(), s):
        return 0
    口.足す(s)
    return len(s.回答群)
