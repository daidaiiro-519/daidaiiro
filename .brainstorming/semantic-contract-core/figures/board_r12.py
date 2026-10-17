# 第12回：GitHub Copilot CLI を WSL に入れ（npm の @github/copilot、1.0.98）、help の出力で呼ぶ条件を確かめて Q0 に入れる。
# 根拠は measured/copilot-cli/help-1.0.98.txt から行ごとに切り出す。原文に無ければ止める。
import json, pathlib, sys

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
HELP = (BOARD / "measured/copilot-cli/help-1.0.98.txt").read_text().split("\n")
SRC = "measured/copilot-cli/help-1.0.98.txt（copilot --help、GitHub Copilot CLI 1.0.98、2026-10-17 に WSL で取得）"


def lines(a, b):
    text = "\n".join(HELP[a - 1 : b])
    if not text.strip():
        sys.exit(f"help の {a}〜{b} 行が空")
    return text


def m(supports, a, b):
    return {"supports": supports, "basis": lines(a, b), "tag": "measured", "source": f"{SRC}:{a}〜{b}"}


assert "--prompt" in HELP[37] and "--session-id" in HELP[64] and "--output-format" in HELP[97], "help の行が変わった"
assert not any("schema" in l.lower() for l in HELP), "help に schema の旗が現れた"

t = next(t for t in board["topics"] if t["id"] == "Q0")
d = t["answer"]["details"]
i = next(k for k, x in enumerate(d) if x.startswith("答えの形は JSON Schema で渡す"))
d[i] = "答えの形は JSON Schema で渡す。Claude Code は --json-schema、Codex は --output-schema で縛る。Kiro と GitHub Copilot は旗が無いので、答えから JSON を取り出す"
d[:] = [x for x in d if not x.startswith("GitHub Copilot は、非対話の起動")]
d.insert(i + 1, "GitHub Copilot：-p で非対話に起動し、--session-id ・ --resume でセッションを続け、--output-format json で1行ずつ JSON を返す。答えの形を縛る旗は無い")
for im in t["images"]:
    if im.get("name") == "AI の呼び方":
        head = im["text"].split("# GitHub Copilot")[0]
        im["text"] = head + "\n".join([
            "# GitHub Copilot（① ②。③ の旗が無いので、答えから JSON を取り出して基盤が確かめる）",
            "copilot -p \"<依頼>\" --output-format json --session-id <新しい UUID>",
            "copilot -p \"<続き>\" --output-format json --resume <ID>",
        ])
t["checks"] = [
    {"target": c["target"].replace("Kiro", "Kiro ・ GitHub Copilot"), "ask": "Kiro と GitHub Copilot を、答えの形を基盤だけで確かめる形で入れることでよいか"}
    if c["ask"].startswith("Kiro を") else c
    for c in t["checks"]
]
t["grounds"] += [
    m("GitHub Copilot を非対話で起動できること", 38, 40),
    m("GitHub Copilot がセッション ID で続けられること", 57, 67),
    m("GitHub Copilot の出力の形が text と json（1行ずつ）だけであること", 98, 100),
]
v = t.get("verification", {})
v["out_of_scope"] = [o for o in v.get("out_of_scope", []) if "GitHub Copilot" not in o["item"]]
if not v["out_of_scope"]:
    v.pop("out_of_scope")
if not v:
    t.pop("verification", None)

patch = [
    {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "GitHub Copilot CLI（1.0.98）を WSL に入れ、help の出力で呼ぶ条件を確かめて Agents 契約で呼ぶ AI に入れた"}},
    {"op": "replace", "path": "/topics", "value": board["topics"]},
]
print(json.dumps(patch, ensure_ascii=False))
