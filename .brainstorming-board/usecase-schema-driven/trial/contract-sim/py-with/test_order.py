"""集約「出荷指示」(AGG-01J7Q4K) のシナリオを検査する。"""
import pytest

from order import Order
from scenario_trace import record_scenario


def test_明細が0件のとき_確定できない():
    """SC-01J7Q4M: given 作成する / when 確定する / then 失敗する (check.precondition)"""
    record_scenario("SC-01J7Q4M")
    order = Order()  # 作成する
    assert order.status == "準備中"
    with pytest.raises(ValueError):
        order.confirm()  # 確定する
    assert order.status == "準備中"


def test_明細が1件あれば_確定できる():
    """SC-01J7Q4N: given 作成する, 明細を足す / when 確定する / then 成功する (check.statusIs)"""
    record_scenario("SC-01J7Q4N")
    order = Order()  # 作成する
    order.lines.append({"qty": 1})  # 明細を足す
    assert len(order.lines) == 1
    order.confirm()  # 確定する
    assert order.status == "確定済"
