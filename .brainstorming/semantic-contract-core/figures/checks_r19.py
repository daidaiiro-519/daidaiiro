# 第19回：確かめてほしい箇所に、私の見解（view）・ 根拠（basis）・ 確かめてほしい理由（why）を書く（ACDR 0189）。
import json, pathlib, sys
VIEWS = {
 ("Q2", "自前の概念"): ("足りている。語彙 ・ 制約 ・ 関係の3つに当てられる", ["4つの技術の位置づけ", "公開仕様の互換"], "Concrete で何を表したいかで、要る概念が変わる"),
 ("Q3", "許可の設定"): ("入れる。どの Skill から AI を呼んでも同じ形が要る", ["ACDR 0188", "利用者の言葉"], "Skill ごとに許可を変えられる幅を、どれだけ残すか"),
 ("Q4", "HTTP"): ("基盤は待ち受けず、Concrete が CLI を包む", ["platform-advisor の回答", "非機能要求グレード E.5.2.1"], "別の機械やクラウドの AI から呼ぶ予定があるか"),
 ("Q4", "WASM"): ("両方として持つ", ["Component Model の文書", "wasmtime の README"], "ほかの言語から使う日と、担い手を足す日のどちらが先か"),
}
board = json.loads(pathlib.Path(sys.argv[1]).read_text())
ops = []
for i, t in enumerate(board["topics"]):
    for j, c in enumerate(t.get("checks", [])):
        v = VIEWS.get((t["id"], c["target"]))
        if v:
            for k, val in zip(("view", "basis", "why"), v):
                ops.append({"op": "add", "path": f"/topics/{i}/checks/{j}/{k}", "value": val})
assert len(ops) == 12, len(ops)
ops.insert(0, {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "確かめてほしい箇所に、私の見解 ・ 根拠 ・ 確かめてほしい理由を書いた"}})
print(json.dumps(ops, ensure_ascii=False))
