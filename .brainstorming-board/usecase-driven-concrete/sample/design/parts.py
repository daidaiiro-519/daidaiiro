# 部品の雛形（design/parts.html）を読み、値を差し込む。差し込む場所の過不足は、その場で誤りにする
import os, re
H = os.path.dirname(os.path.abspath(__file__))
_T = dict(re.findall(r'<template id="([^"]+)">(.*?)</template>', open(os.path.join(H, "parts.html"), encoding="utf-8").read(), re.S))
def P(_part, **kw):
    t = _T[_part]
    need = set(re.findall(r"\{\{(\w+)\}\}", t))
    if need != set(kw): raise KeyError(f"部品 {_part} の差し込む場所が合わない：必要 {sorted(need)} ／ 渡した {sorted(kw)}")
    return re.sub(r"\{\{(\w+)\}\}", lambda m: str(kw[m.group(1)]), t)
