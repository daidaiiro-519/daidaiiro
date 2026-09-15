"""完成イメージ ── 図の宣言が、実際に絵になるまでの3段を通しで作る。

ここで作る変換（ホストの語彙 → 一般名詞）は、ADRが「アダプタの内側だけで起きる」
と決めたもの。まだWaffleへ組み込んでいないので、この場で試作して実物を見る。
"""
from __future__ import annotations

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_figure  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

# ── 1段目: 図の宣言（ホストの語彙）。ドメイン層が持つ値の組 ──────────────
DECLARATION = {
    "asserts": "つながり",
    "reading": "受け口はユースケースを呼び、ユースケースが業務サービスを介してモデルへ届く。",
    "items": [
        {"key": "cli", "name": "CLI"},
        {"key": "mcp", "name": "MCP"},
        {"key": "usecase", "name": "ユースケース", "role": "focus"},
        {"key": "service", "name": "業務サービス"},
        {"key": "model", "name": "モデル"},
    ],
    "links": [
        {"from": "cli", "to": "usecase"},
        {"from": "mcp", "to": "usecase"},
        {"from": "usecase", "to": "service", "name": "呼ぶ"},
        {"from": "service", "to": "model"},
    ],
}


# ── 2段目: 変換（アダプタの内側）。ホストの語彙 → 一般名詞 ────────────────
def to_engine_input(declaration: dict) -> dict:
    """図の宣言を、描画エンジンが受け取る一般名詞の形へ写す。

    ここがADRで言う「変換はアダプタの内側だけ」の実体。ホスト側の語彙
    (asserts/reading/items/links) が、ここから先へ一切出ない。
    """
    nodes = [{"id": it["key"], "label": it["name"], **({"role": it["role"]} if it.get("role") else {})}
             for it in declaration["items"]]
    edges = [{"from": lk["from"], "to": lk["to"], **({"label": lk["name"]} if lk.get("name") else {})}
             for lk in declaration["links"]]
    return {"nodes": nodes, "edges": edges}


ENGINE_INPUT = to_engine_input(DECLARATION)

# ── 3段目: 描く ────────────────────────────────────────────────
SVG = render_figure(ENGINE_INPUT["nodes"], ENGINE_INPUT["edges"], direction="TB")

(OUT / "pipeline_declaration.json").write_text(
    json.dumps(DECLARATION, ensure_ascii=False, indent=1), encoding="utf-8")
(OUT / "pipeline_engine_input.json").write_text(
    json.dumps(ENGINE_INPUT, ensure_ascii=False, indent=1), encoding="utf-8")
(OUT / "pipeline_result.svg").write_text(SVG, encoding="utf-8")
print("wrote 3 stages")
