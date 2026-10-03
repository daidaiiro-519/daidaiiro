"""受注の文脈の component テスト。テスト条件の ID とハッシュ値は concrete conditions から写した"""
import pytest
from datetime import datetime,timedelta
from tests.trace import trace
from src.order.domain import *
T=datetime(2026,10,3,12,30)
def draft(n=2):
  o=Order('N-1','C-1','S-1')
  for i in range(n): o.add_line(Line(f'P-{i}',Quantity(1),Money(500)))
  return o
def test_inv1_confirmed_needs_pickup():
  trace("AGG-1.INV-1","00caf1db","component")
  o=draft()
  with pytest.raises(Rejected): o.confirm(None,'A-1','SL-1')
  assert o.status==DRAFT
def test_inv2_confirmed_needs_lines():
  trace("AGG-1.INV-2","9efd2f10","component")
  o=draft(0)
  with pytest.raises(Rejected): o.confirm(T,'A-1','SL-1')
  assert o.status==DRAFT
def test_lines_max_20():
  trace("AGG-1.ST-3.MAX","2f0bdb3c","component")
  o=draft(20)
  with pytest.raises(Rejected): o.add_line(Line('P-x',Quantity(1),Money(1)))
def test_confirm_rejects_confirmed():
  trace("AGG-1.CMD-1.PRE-1","d2b64f49","component")
  o=draft(1); o.status=CONFIRMED
  with pytest.raises(Rejected) as e: o.confirm(T,'A-1','SL-1')
  assert e.value.reason=='確定済の注文は確定できない'
def test_confirm_rejects_no_lines():
  trace("AGG-1.CMD-1.PRE-2","518eb331","component")
  with pytest.raises(Rejected) as e: draft(0).confirm(T,'A-1','SL-1')
  assert e.value.reason=='明細の無い注文は確定できない'
def test_confirm_accepts():
  trace("AGG-1.CMD-1.OK-1","2ac0efc1","component")
  o=draft(2); ev=o.confirm(T,'A-1029','調理枠 12:00')
  assert (o.status,o.pickup,o.approval_no,o.reserved_slot)==(CONFIRMED,T,'A-1029','調理枠 12:00')
  assert ev==[OrderConfirmed('N-1',T)]
def test_record_progress():
  trace("AGG-1.CMD-2.OK-1","1e5e7c59","component")
  o=draft(); o.record_progress(T); assert len(o.progress)==1
def test_hand_over_rejects_draft():
  trace("AGG-1.CMD-3.PRE-1","e8b99fcf","component")
  with pytest.raises(Rejected) as e: draft().hand_over()
  assert e.value.reason=='確定していない注文は渡せない'
def test_hand_over():
  trace("AGG-1.CMD-3.OK-1","a49b6c17","component")
  o=draft(); o.status=CONFIRMED; ev=o.hand_over()
  assert o.status==HANDED and ev==[OrderHanded('N-1')]
def slot(free): return CookingSlot('SL-1',datetime(2026,10,3,12,0),CookAmount(free),'S-1')
def test_reserve_rejects_short():
  trace("AGG-2.CMD-1.PRE-1","77d3e427","component")
  with pytest.raises(Rejected) as e: slot(1).reserve(CookAmount(1))
  assert e.value.reason=='調理の空きが足りない'
def test_reserve():
  trace("AGG-2.CMD-1.OK-1","6b300d99","component")
  s=slot(5); s.reserve(CookAmount(2)); assert s.free==CookAmount(3)
def test_release():
  trace("AGG-2.CMD-2.OK-1","51ef009f","component")
  s=slot(3); s.release(CookAmount(2)); assert s.free==CookAmount(5)
@pytest.mark.parametrize('cid,cls,bad',[("VO-1.INV-1",Quantity,0),("VO-1.INV-2",Quantity,100),("VO-5.INV-1",CookAmount,-1),("VO-5.INV-2",CookAmount,10000),("VO-9.INV-1",Money,-1)])
def test_vo_rejects(cid,cls,bad):
  trace(cid,H[cid],"component")
  with pytest.raises(Invalid): cls(bad)
H={"VO-1.INV-1":"aa1bca82","VO-1.INV-2":"ef446bfb","VO-5.INV-1":"07eb2c4b","VO-5.INV-2":"809cd20f","VO-9.INV-1":"07eb2c4b"}
def test_quantity_add():
  trace("VO-1.OP-1.OK-1","6ab6668d","component")
  assert Quantity(2).add(Quantity(3))==Quantity(5)
def test_amount_add():
  trace("VO-5.OP-1.OK-1","87326582","component")
  assert CookAmount(3).add(CookAmount(2))==CookAmount(5)
def test_amount_sub():
  trace("VO-5.OP-2.OK-1","f70805ab","component")
  assert CookAmount(5).sub(CookAmount(2))==CookAmount(3)
def test_estimate():
  trace("DS-1.OP-1.POST-1","0d7190ea","component")
  trace("DS-1.OP-1.POST-2","b692bac2","component")
  start=datetime(2026,10,3,12,0); r=estimate_pickup(CookAmount(4),CookAmount(10),start)
  assert r is not None and r>=start
def test_reserve_rejects_over_limit():
  trace("AGG-2.CMD-1.PRE-2","da3169c1","component")
  with pytest.raises(Rejected) as e: slot(20).reserve(CookAmount(11))
  assert e.value.reason=='一度に確保できる量を超えている'
