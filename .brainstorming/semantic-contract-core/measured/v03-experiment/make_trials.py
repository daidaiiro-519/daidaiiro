"""いまの基盤で、決まりを宣言に移せるかを試すための写しを作り、検査を走らせて trials.out.json に書く。

使い方（リポジトリの根から）:
    python3 -I .brainstorming/semantic-contract-core/measured/v03-experiment/make_trials.py
読むもの（書き換えない）: ボードのスキーマ、.brainstorming/ai-driven-process.json、Skill の実行ファイル
書くもの: この実験のフォルダの xref-trial/ ・ jsonschema-trial/ ・ keyword-trial/ ・ trials.out.json

試すのは3つ。
- xref-trial：いまの注釈 x-ref の when と accept で、inspect.rs の2つの決まりを書けるか
- jsonschema-trial：標準の JSON Schema 2020-12 のキーワードだけで、x-prompt と inspect.rs の決まりを書けるか
- keyword-trial：新しい独自のキーワードを足したとき、基盤の契約と検査がどう振る舞うか
どれも Skill の名前（brainstorming）のフォルダに写したスキーマを置き、--skill でそのフォルダを渡す。
"""
import copy
import json
import os
import shutil
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SCHEMA = ".claude/skills/brainstorming/assets/board.schema.json"
REAL = ".brainstorming/ai-driven-process.json"
SD = ".claude/skills/brainstorming/scripts/bin/schema-driven"
BS = ".claude/skills/brainstorming/scripts/bin/brainstorming"
OPEN = {"properties": {"status": {"const": "open"}}, "required": ["status"]}


def load(path):
    return json.load(open(path, encoding="utf-8"))


def dump(body, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(body, f, ensure_ascii=False, indent=2)
        f.write("\n")


def run(*args):
    out = subprocess.run(args, capture_output=True, text=True)
    return json.loads(out.stdout)


def board(name, body, folder, file=None):
    body = copy.deepcopy(body)
    body["id"] = name
    dump(body, os.path.join(folder, f"{file or name}.json"))


def errors_and_drift(skill, boards):
    got = run(SD, "--skill", skill, "check", "--dir", boards)
    return {
        "errors": {os.path.basename(i["path"]): i["errors"] for i in got.get("instances", []) if i["errors"]},
        "drift": [{k: f[k] for k in ("check", "from", "to", "message")} for f in got.get("findings", []) if f["status"] != "合格"],
        "reason": got.get("reason"),
    }


def fresh(folder):
    shutil.rmtree(folder, ignore_errors=True)
    os.makedirs(folder)


def main():
    real = load(REAL)
    result = {}

    # ---- xref-trial ------------------------------------------------------------
    X = os.path.join(HERE, "xref-trial")
    fresh(X)
    s = load(SCHEMA)
    dump(s, os.path.join(X, "brainstorming/assets/board.schema.json"))
    b = os.path.join(X, "boards")
    board("base", real, b)
    q = copy.deepcopy(real); q["queue"] = [{"topic": "Q1", "why": "試し"}] + q["queue"]; board("queue-not-open", q, b)
    o = copy.deepcopy(real); o["topics"][1]["status"] = "open"; o["queue"].append({"topic": "Q1", "why": "試し"}); board("open-dep-unsettled", o, b)
    d = copy.deepcopy(real); d["topics"][2]["id"] = "Q1"; board("dup-topic-id", d, b)
    board("id-differs", real, b, file="id-differs-file")
    st = copy.deepcopy(real); st["status"] = "answer"; board("board-status", st, b)
    skill = os.path.join(X, "brainstorming")
    result["xref-trial"] = {"before": errors_and_drift(skill, b)}
    s["properties"]["queue"]["items"]["properties"]["topic"]["x-ref"]["accept"] = {"at": "status", "in": ["open"]}
    dep = s["$defs"]["topic"]["properties"]["depends"]["x-ref"]
    dep["when"] = {"status": ["open"]}
    dep["accept"] = {"at": "status", "in": ["settled"]}
    dump(s, os.path.join(skill, "assets/board.schema.json"))
    result["xref-trial"]["check-schemas"] = run(SD, "check-schemas", "--dir", os.path.join(skill, "assets"))["findings"]
    result["xref-trial"]["after"] = errors_and_drift(skill, b)
    result["xref-trial"]["inspect"] = {f: run(BS, "inspect", "--board", os.path.join(b, f))["findings"]
                                      for f in sorted(os.listdir(b))}

    # ---- jsonschema-trial ------------------------------------------------------
    J = os.path.join(HERE, "jsonschema-trial")
    fresh(J)
    s = load(SCHEMA)
    P, topic = s["properties"], s["$defs"]["topic"]
    P["title"]["maxLength"] = 40                                    # x-prompt short（board.schema.json:102）
    P["topics"]["minItems"] = 3                                     # x-prompt three-to-five（:519）
    P["topics"]["maxItems"] = 5
    P["topics"]["contains"] = OPEN                                  # inspect.rs:40
    P["topics"]["minContains"] = 0
    P["topics"]["maxContains"] = 5
    topic["properties"]["rejected"]["minItems"] = 1                 # x-prompt keep ・ refuted-first（:981 ・ :20）
    topic["allOf"] = [{"if": OPEN, "then": {"required": ["images"], "properties": {"images": {"minItems": 1}}}}]  # inspect.rs:123
    s["allOf"] = [{"if": {"properties": {"topics": {"contains": OPEN}}, "required": ["topics"]},
                   "then": {"required": ["queue"], "properties": {"queue": {"minItems": 1}}}}]                    # inspect.rs:50
    s["$defs"]["code"]["properties"]["language"]["not"] = {"enum": ["text", "plain", "txt"]}                     # inspect.rs:113
    skill = os.path.join(J, "brainstorming")
    dump(s, os.path.join(skill, "assets/board.schema.json"))
    b = os.path.join(J, "boards")
    board("real-as-is", real, b)
    five = copy.deepcopy(real); five["topics"] = five["topics"][:5]
    for t in five["topics"]:
        t["depends"] = [x for x in t.get("depends", []) if x != "Q5"]
    board("five-topics", five, b)
    lt = copy.deepcopy(five); lt["title"] = "あ" * 41; board("long-title", lt, b)
    ni = copy.deepcopy(five)
    for t in ni["topics"]:
        t["status"] = "open"
    ni["topics"][4]["images"] = []; ni["queue"] = []; board("open-no-image-no-queue", ni, b)
    ct = copy.deepcopy(five); ct["topics"][0]["images"] = [{"kind": "code", "caption": "c", "language": "text", "text": "x"}]
    board("code-text", ct, b)
    result["jsonschema-trial"] = {"check-schemas": run(SD, "check-schemas", "--dir", os.path.join(skill, "assets"))["findings"],
                                  "check": errors_and_drift(skill, b)}
    # maxContains だけを見る（maxItems を外して、6件のうち open が6件と1件）
    with tempfile.TemporaryDirectory() as tmp:
        s2 = copy.deepcopy(s); del s2["properties"]["topics"]["maxItems"]
        dump(s2, os.path.join(tmp, "brainstorming/assets/board.schema.json"))
        six = copy.deepcopy(real)
        for t in six["topics"]:
            t["status"] = "open"
        board("six-open", six, os.path.join(tmp, "boards"))
        board("six-one-open", real, os.path.join(tmp, "boards"))
        result["jsonschema-trial"]["maxContains-only"] = errors_and_drift(os.path.join(tmp, "brainstorming"), os.path.join(tmp, "boards"))

    # ---- keyword-trial ---------------------------------------------------------
    K = os.path.join(HERE, "keyword-trial")
    fresh(K)
    s = load(SCHEMA)
    s["x-constraints"] = [{"term": "brainstorming:OpenTopicsAtMost", "params": {"max": 5}}]
    s["properties"]["title"]["x-constraints"] = [{"term": "text:CharsAtMost", "params": {"max": 40}}]
    skill = os.path.join(K, "brainstorming")
    dump(s, os.path.join(skill, "assets/board.schema.json"))
    result["keyword-trial"] = {"undeclared": run(SD, "check-schemas", "--dir", os.path.join(skill, "assets"))["findings"]}
    s["x-annotations"] = {"x-constraints": "constraints.schema.json"}   # 仕様のファイルは置かない
    dump(s, os.path.join(skill, "assets/board.schema.json"))
    result["keyword-trial"]["declared"] = run(SD, "check-schemas", "--dir", os.path.join(skill, "assets"))["findings"]
    b = os.path.join(K, "boards")
    lt = copy.deepcopy(real); lt["title"] = "あ" * 50; board("long-title", lt, b)
    result["keyword-trial"]["check-long-title"] = errors_and_drift(skill, b)

    dump(result, os.path.join(HERE, "trials.out.json"))
    print(json.dumps(result, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
