"""内側 ── 外部の技術を1つも参照しない。

単位と変更理由:
  回答　　　　回答の項目が変わるとき
  送信　　　　1回に入るものが変わるとき
  重複の判定　判定の規則が変わるとき
  蓄積の口　　保存に何を渡すかが変わるとき（ポート。テストで差し替えるので作る）
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass(frozen=True)
class 回答:
    論点: int
    諾否: str          # approve / return / skip
    理由: str = ""


@dataclass(frozen=True)
class 送信:
    回答群: tuple[回答, ...]

    def __post_init__(self):
        番号 = [a.論点 for a in self.回答群]
        if len(番号) != len(set(番号)):
            raise ValueError("論点1件につき回答は1つである")


def 重複か(前: 送信 | None, いま: 送信) -> bool:
    """直前と同じ内容か。**保存の仕方を1つも知らない。**"""
    return 前 is not None and 前.回答群 == いま.回答群


class 蓄積の口(Protocol):
    """保存に何を渡すかだけを決める。どこへ保存するかは外側が決める。"""

    def 直前(self) -> 送信 | None: ...
    def 足す(self, s: 送信) -> None: ...
