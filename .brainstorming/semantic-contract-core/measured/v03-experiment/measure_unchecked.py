"""x-prompt に文で書いてあるが、inspect と check が見ていない決まりのうち、機械で数えられるものを実物のボードに適用する。
使い方: python3 -I measure_unchecked.py <ボードの JSON> ...   （読むだけ。何も書かない）
文の切り方は src/brainstorming/crates/core/src/domain/text.rs の sentences を写したもの。"""
import json, os, re, sys

def sentences(text):
    out, cur, depth = [], "", 0
    for ch in text:
        if ch in "「『": depth += 1
        elif ch in "」』": depth = max(0, depth - 1)
        if ch == "\n":
            if cur.strip(): out.append(cur.strip())
            cur = ""; continue
        cur += ch
        if depth == 0 and ch in "。！？":
            if cur.strip(): out.append(cur.strip())
            cur = ""
    if cur.strip(): out.append(cur.strip())
    return out

def measure(path):
    b = json.load(open(path))
    name = os.path.splitext(os.path.basename(path))[0]
    rounds_dir = os.path.join(os.path.dirname(path), name, "rounds")
    rounds = sorted(int(f[:-5]) for f in os.listdir(rounds_dir)) if os.path.isdir(rounds_dir) else []
    topics = b.get("topics", [])
    ids = [t["id"] for t in topics]
    r = {}
    r["topics-3-to-5（件数）"] = (len(topics), 3 <= len(topics) <= 5)
    r["title-40字まで"] = (len(b.get("title", "")), len(b.get("title", "")) <= 40)
    r["intro-1文"] = (len(sentences(b.get("intro", ""))), len(sentences(b.get("intro", ""))) <= 1)
    r["id-ファイル名と同じ"] = (b.get("id"), b.get("id") == name)
    r["round-rounds の最後と同じ"] = ((b.get("round"), rounds[-1] if rounds else None), (not rounds) or b.get("round") == rounds[-1])
    r["topic.id-重ならない"] = (len(ids) - len(set(ids)), len(ids) == len(set(ids)))
    no_rej = [t["id"] for t in topics if not t.get("rejected")]
    r["rejected-1件以上"] = (no_rej, not no_rej)
    long_names = [(t["id"], len(t["name"])) for t in topics if len(t["name"]) > 15]
    r["topic.name-10字くらい（15字超を数える）"] = (long_names, not long_names)
    caps = [(t["id"], i + 1) for t in topics for i, im in enumerate(t.get("images", [])) if len(sentences(im.get("caption", ""))) > 1]
    r["caption-1文"] = (caps, not caps)
    dates = [l["date"] for l in b.get("log", [])]
    r["log-新しい順"] = (len(dates), dates == sorted(dates, reverse=True))
    texts = [(t["id"], len(t["answer"]["text"])) for t in topics if len(t["answer"]["text"]) > 120]
    r["answer.text-120字まで（文ではなく欄全体）"] = (texts, not texts)
    bad_tree = []
    for t in topics:
        for im in t.get("images", []):
            if im.get("kind") == "tree":
                prev = -1
                for row in im["rows"]:
                    if row["depth"] > prev + 1: bad_tree.append(t["id"])
                    prev = row["depth"]
    r["tree-depth は親より1つ深い"] = (bad_tree, not bad_tree)
    return name, r

if __name__ == "__main__":
    result = {}
    for p in sys.argv[1:]:
        name, r = measure(p)
        result[name] = {k: {"value": v[0], "ok": v[1]} for k, v in r.items()}
    print(json.dumps(result, ensure_ascii=False, indent=2))
