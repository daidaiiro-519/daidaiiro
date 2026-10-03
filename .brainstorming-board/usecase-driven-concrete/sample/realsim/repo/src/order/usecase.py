"""UC-1 注文を確定する（アプリケーションの層）。外部はポートで受ける"""
from dataclasses import dataclass
from datetime import datetime
from .domain import Rejected,CookAmount,estimate_pickup
@dataclass
class Result: ok:bool; outcome:str; detail:object=None
class ConfirmOrder:
  def __init__(self,orders,slots,catalog,payment,customer_ui,clock):
    self.orders,self.slots,self.catalog,self.payment,self.ui,self.clock=orders,slots,catalog,payment,customer_ui,clock
  def run(self,order_no,slot_id):
    o=self.orders[order_no]; slot=self.slots[slot_id]
    # 手順2 妥当性確認（BC-1.BR-1）。拡張 2a：変わった価格を示し、同意を得て手順2へ戻る
    while True:
      changed=[l for l in o.lines if l.presented_price.value!=self.catalog.price(l.product)]
      if not changed: break
      if not self.ui.accept_prices(changed): return Result(False,'価格に同意しなかった')
      for l in changed: l.presented_price=type(l.presented_price)(self.catalog.price(l.product))
    total=CookAmount(sum(l.quantity.value for l in o.lines))
    # 手順3 見積もる ・ 手順4 確保する。拡張 4a：空きのある時刻を示し、同意を得て手順4へ戻る
    while True:
      pickup=estimate_pickup(total,slot.free,slot.start)
      try:
        slot.reserve(total); break
      except Rejected as e:
        if e.reason=='一度に確保できる量を超えている':
          o.record_progress(self.clock()); self.ui.notify('一度に受けられる量'); return Result(False,'量を超えた')
        if e.reason!='調理の空きが足りない': raise
        slot=self.ui.choose_other_slot(self.slots,total)
        if slot is None: return Result(False,'時刻に同意しなかった')
    # 手順5 支払いの承認を得る。拡張 5a：調理枠を戻し、経過を記録し、承認の拒否を知らせる
    approval=self.payment.approve(o)
    if approval is None:
      slot.release(total); o.record_progress(self.clock())
      self.ui.notify('承認の拒否'); return Result(False,'承認されなかった')
    events=o.confirm(pickup,approval,slot.slot_id)          # 手順6
    self.ui.notify(('注文番号',o.order_no,o.pickup))        # 手順7
    return Result(True,'確定',events)
