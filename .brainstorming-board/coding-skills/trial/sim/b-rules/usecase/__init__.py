"""内側 ── 手順。外側の道具は、ポートを通してしか触らない。"""
from __future__ import annotations

from domain import submit, StorePort, is_duplicate


def receive_answer(port: StorePort, s: submit) -> int:
    """保存した件数を返す。**直前と同じなら0件である。**"""
    if is_duplicate(port.latest(), s):
        return 0
    port.add(s)
    return len(s.answers)
