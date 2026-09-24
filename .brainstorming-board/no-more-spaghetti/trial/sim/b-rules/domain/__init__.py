"""内側 ── 外部の技術を1つも参照しない。

単位と変更理由:
  回答　　　　回答の項目が変わるとき
  送信　　　　1回に入るものが変わるとき
  重複の判定　判定の規則が変わるとき
  蓄積ポート　　保存に何を渡すかが変わるとき（ポート。テストで差し替えるので作る）
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass(frozen=True)
class answer:
    topic: int
    verdict: str          # approve / return / skip
    reason: str = ""


@dataclass(frozen=True)
class submit:
    answers: tuple[answer, ...]

    def __post_init__(self):
        number = [a.topic for a in self.answers]
        if len(number) != len(set(number)):
            raise ValueError("論点1件につき回答は1つである")


def is_duplicate(prev: submit | None, now: submit) -> bool:
    """直前と同じ内容か。**保存の仕方を1つも知らない。**"""
    return prev is not None and prev.answers == now.answers


class StorePort(Protocol):
    """保存に何を渡すかだけを決める。どこへ保存するかは外側が決める。"""

    def latest(self) -> submit | None: ...
    def add(self, s: submit) -> None: ...
