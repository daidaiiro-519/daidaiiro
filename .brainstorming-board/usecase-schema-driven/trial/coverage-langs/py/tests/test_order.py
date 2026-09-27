import os, pytest
from order import Order

def scenario(i):
    p = os.environ.get("SCENARIO_TRACE")
    if p:
        with open(p, "a", encoding="utf-8") as f: f.write(i + "\n")

@pytest.mark.parametrize("lines,sid,ok", [([], "SC-01J7Q4M", False), (["りんご"], "SC-01J7Q4N", True)])
def test_confirm(lines, sid, ok):
    scenario(sid)
    o = Order(lines)
    if ok: o.confirm(); assert o.status == "確定済"
    else:
        with pytest.raises(ValueError): o.confirm()
