"""受注の文脈のドメインモデル（集約 ・ 値オブジェクト ・ ドメインサービス）"""
from dataclasses import dataclass,field
from datetime import datetime,timedelta
class Rejected(Exception):
  """コマンドの拒否。reason は用語集の拒否の理由の語"""
  def __init__(self,reason): super().__init__(reason); self.reason=reason
class Invalid(ValueError): pass
# ── 値オブジェクト
@dataclass(frozen=True)
class Quantity:            # VO-1 数量
  value:int
  def __post_init__(self):
    if not (1<=self.value<=99): raise Invalid(f'数量は1以上99以下：{self.value}')
  def add(self,o): return Quantity(self.value+o.value)
@dataclass(frozen=True)
class CookAmount:          # VO-5 調理の量
  value:int
  def __post_init__(self):
    if not (0<=self.value<=9999): raise Invalid(f'調理の量は0以上9999以下：{self.value}')
  def add(self,o): return CookAmount(self.value+o.value)
  def sub(self,o): return CookAmount(self.value-o.value)
@dataclass(frozen=True)
class Money:               # VO-9 金額
  value:int
  def __post_init__(self):
    if self.value<0: raise Invalid(f'金額は0以上：{self.value}')
DRAFT,CONFIRMED,HANDED='下書き','確定済','受け渡し済'
@dataclass
class Line:                # ENT-1 明細
  product:str; quantity:Quantity; presented_price:Money
@dataclass(frozen=True)
class OrderConfirmed: order_no:str; pickup:datetime
@dataclass(frozen=True)
class OrderHanded: order_no:str
# ── 集約：注文（AGG-1）
@dataclass
class Order:
  order_no:str; customer:str; store:str
  status:str=DRAFT; lines:list=field(default_factory=list); pickup:datetime|None=None
  approval_no:str|None=None; reserved_slot:str|None=None; progress:list=field(default_factory=list)
  MAX_LINES=20
  def add_line(self,line):
    if len(self.lines)>=self.MAX_LINES: raise Rejected('明細は20件まで')
    self.lines.append(line)
  def confirm(self,pickup,approval_no,slot_id):            # CMD-1 確定する
    if self.status!=DRAFT: raise Rejected('確定済の注文は確定できない')
    if len(self.lines)<1: raise Rejected('明細の無い注文は確定できない')
    if pickup is None: raise Rejected('受け取り予定時刻が無い')
    self.status=CONFIRMED; self.pickup=pickup; self.approval_no=approval_no; self.reserved_slot=slot_id
    return [OrderConfirmed(self.order_no,self.pickup)]
  def record_progress(self,at):                            # CMD-2 経過を記録する
    self.progress.append(at); return []
  def hand_over(self):                                     # CMD-3 渡す
    if self.status!=CONFIRMED: raise Rejected('確定していない注文は渡せない')
    self.status=HANDED; return [OrderHanded(self.order_no)]
# ── 集約：調理枠（AGG-2）
@dataclass
class CookingSlot:
  slot_id:str; start:datetime; free:CookAmount; store:str
  def reserve(self,amount):                                # CMD-1 確保する
    if not (self.free.value>=amount.value+1): raise Rejected('調理の空きが足りない')
    if not (amount.value<=10): raise Rejected('一度に確保できる量を超えている')
    self.free=self.free.sub(amount); return []
  def release(self,amount):                                # CMD-2 戻す
    self.free=self.free.add(amount); return []
# ── ドメインサービス：受け取り予定時刻を見積もる（DS-1）
def estimate_pickup(total:CookAmount,free:CookAmount,start:datetime)->datetime:
  minutes=10+total.value*2
  return start+timedelta(minutes=minutes)
