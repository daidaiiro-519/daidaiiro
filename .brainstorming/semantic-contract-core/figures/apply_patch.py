# ボード全体を置き換えるパッチは引数の上限を超えるので、変わった欄だけの操作に分けて、上限より小さい束で update へ渡す。
import json, subprocess, sys, pathlib
board_path = ".brainstorming/semantic-contract-core.json"
B = ".claude/skills/brainstorming/scripts/bin/brainstorming"
patch = json.loads(sys.stdin.read())
old = json.loads(pathlib.Path(board_path).read_text())
ops = []
for op in patch:
    if op["path"] == "/topics":
        for i, (a, b) in enumerate(zip(old["topics"], op["value"])):
            for k in sorted(set(a) | set(b)):
                if k == "images" and k in a and k in b and len(a[k]) == len(b[k]):
                    for j, (x, y) in enumerate(zip(a[k], b[k])):
                        for kk in sorted(set(x) | set(y)):
                            if x.get(kk) != y.get(kk):
                                ops.append({"op": "add" if kk in y else "remove", "path": f"/topics/{i}/images/{j}/{kk}", **({"value": y[kk]} if kk in y else {})})
                elif a.get(k) != b.get(k):
                    ops.append({"op": "add" if k in b else "remove", "path": f"/topics/{i}/{k}", **({"value": b[k]} if k in b else {})})
    else:
        ops.append(op)
batch, size = [], 0
def send(batch):
    out = subprocess.run([B, "update", "--path", board_path, "--patch", json.dumps(batch, ensure_ascii=False)], capture_output=True, text=True).stdout
    ok = json.loads(out).get("ok"); print(len(batch), "件", "ok" if ok else out[:400])
    if not ok: sys.exit(1)
for op in ops:
    n = len(json.dumps(op, ensure_ascii=False).encode())
    if batch and size + n > 100_000:
        send(batch); batch, size = [], 0
    batch.append(op); size += n
if batch: send(batch)
