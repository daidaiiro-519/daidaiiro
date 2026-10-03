"""駆動する側のアダプター（CLI）。標準入力の JSON で注文を受け、UC-1 を実行して結果を JSON で返す。
決済代行 ・ メニュー（今の単価）・ 顧客の画面は、駆動される側の偽物を入力で指定する（system レベルのテスト用）"""
import sys,json
from datetime import datetime
from .domain import Order,Line,Quantity,Money,CookingSlot,CookAmount
from .usecase import ConfirmOrder
class Catalog:
  def __init__(self,prices): self.prices=prices
  def price(self,p): return self.prices[p]
class Payment:
  def __init__(self,mode): self.mode=mode
  def approve(self,order): return 'A-1029' if self.mode=='approve' else None
class UI:
  def __init__(self,inp): self.inp=inp; self.sent=[]
  def accept_prices(self,changed): self.sent.append('変わった価格'); return self.inp.get('accept_prices',True)
  def choose_other_slot(self,slots,total):
    self.sent.append('空きのある時刻')
    if not self.inp.get('accept_other_slot',True): return None
    c=[s for s in slots.values() if s.free.value>=total.value]
    return min(c,key=lambda s:s.start) if c else None
  def notify(self,x): self.sent.append(x if isinstance(x,str) else x[0])
def main():
  i=json.load(sys.stdin)
  o=Order(order_no='N-0001',customer='C-1',store='S-1')
  for l in i['lines']: o.add_line(Line(l['product'],Quantity(l['qty']),Money(l['price'])))
  slots={s['id']:CookingSlot(s['id'],datetime.fromisoformat(s['start']),CookAmount(s['free']),'S-1') for s in i['slots']}
  ui=UI(i); uc=ConfirmOrder({'N-0001':o},slots,Catalog(i['catalog']),Payment(i['payment']),ui,lambda:datetime(2026,10,3,11,0))
  r=uc.run('N-0001',i['slots'][0]['id'])
  print(json.dumps({"ok":r.ok,"outcome":r.outcome,"status":o.status,"pickup":o.pickup.isoformat() if o.pickup else None,
    "approval_no":o.approval_no,"reserved_slot":o.reserved_slot,"slot_free":{k:s.free.value for k,s in slots.items()},
    "progress":len(o.progress),"sent":ui.sent},ensure_ascii=False))
if __name__=='__main__': main()
