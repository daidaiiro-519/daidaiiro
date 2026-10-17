# 第13回：Agents 契約が約束するものに「AI ごとの許可の設定」を足す。
# brainstorming で一時に Concrete に置いた形（ACDR 0188、brainstorming/agents@v1）を、基盤（Abstract）が持つ形として Q3 に持ち込む。
import json, pathlib

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
REPO = BOARD.parent.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
schema = json.loads((REPO / ".claude/skills/brainstorming/assets/agents.schema.json").read_text())
U_TEMP = "そうだよね、これは一時的よね？これからこの契約を持つのはAbstruct側になるもんね？"
U_FILE = "これ許可リストは外出しのファイルを読む形にしないと回らないぞ。"
user = lambda supports, words: {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}

t = next(t for t in board["topics"] if t["id"] == "Q3")
d = t["answer"]["details"]
i = next(k for k, x in enumerate(d) if x.startswith("Agents 契約が約束するもの"))
d.insert(i + 1, "Agents 契約は AI ごとの許可の設定も持つ：許すツール ・ sandbox ・ 権限を外す値を通さない決まり ・ 設定が検査に通らなければ起動しない。基盤が形を持ち、Skill は値（自分のコマンドと使い方の文）だけを持つ")
d.insert(i + 2, "許可の設定は、いま brainstorming が一時に持っている（ACDR 0188、brainstorming/agents@v1）。Agents 契約が決まったら形を基盤へ移す")
t["images"].append({
    "kind": "code", "name": "許可の設定（いま）",
    "caption": "brainstorming が一時に持つ許可の設定の形（ACDR 0188）。基盤へ移すときは、この形を Agents 契約の一部にし、Skill 固有の値（${entry} ・ guide）だけを Skill に残す。",
    "path": ".claude/skills/brainstorming/assets/agents.schema.json（properties の型の部分）", "language": "json",
    "text": json.dumps({k: {kk: vv for kk, vv in v.items() if kk not in ("x-prompt",)} for k, v in schema["properties"].items() if k in ("claude", "codex", "kiro", "guide")}, ensure_ascii=False, indent=2),
})
t["checks"] = t.get("checks", []) + [{"target": "許可の設定", "ask": "AI ごとの許可の設定を Agents 契約の約束に入れ、Skill には値だけを残すことでよいか"}]
t["grounds"] += [user("許可をファイルに出すこと", U_FILE), user("許可の設定の契約は Abstract 側が持つこと", U_TEMP),
                 {"supports": "いま brainstorming が持っている形", "basis": schema["description"], "tag": "primary", "source": ".claude/skills/brainstorming/assets/agents.schema.json（ACDR 0188）"}]
patch = [
    {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "Q3 の Agents 契約が約束するものに、AI ごとの許可の設定を足した（brainstorming が一時に持つ形を基盤へ移す前提）"}},
    {"op": "replace", "path": "/topics", "value": board["topics"]},
]
print(json.dumps(patch, ensure_ascii=False))
