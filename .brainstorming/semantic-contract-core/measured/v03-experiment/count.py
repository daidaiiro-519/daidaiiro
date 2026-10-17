"""ボードに課している決まりを、実物のファイルから数え上げて inventory.json と inventory.md を書く。

使い方（リポジトリの根から）:
    python3 -I .brainstorming/semantic-contract-core/measured/v03-experiment/count.py

読むもの（書き換えない）:
    .claude/skills/brainstorming/assets/board.schema.json
書くもの（この実験のフォルダの中だけ）:
    inventory.json  決まり1件ずつ（出どころ・いまの表し方・いまの検査・v0.3 での分類）
    inventory.md    README に貼る表

スキーマの形と x-prompt の rule は機械で拾う。inspect.rs ・ status.rs ・ lock.rs ・ SKILL.md の決まりは、
読んで1件ずつ書き出したもの（行番号はこの実験のときのファイル）。分類は手で付けたもので、根拠は note に書く。
"""
import json
import os
import re
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
SCHEMA = ".claude/skills/brainstorming/assets/board.schema.json"

# ---- いまの検査（checked）の記号 -------------------------------------------------
CHECKED = {
    "S": "JSON Schema で検査",
    "R": "x-ref を check が検査",
    "I": "inspect.rs が検査",
    "W": "書く側 ・ 動かす側のコードが守る（検査ではない）",
    "M": "検査されていない（機械で見られる）",
    "A": "検査されていない（意味の判断を必要とする）",
    "O": "検査されていない（インスタンスの外の事実）",
    "D": "決まりの中身が無い（説明のとおりに書く）",
}
# ---- v0.3 の形にしたときの分類 -----------------------------------------------------
V03 = {
    "a": "宣言にでき、汎用の Interpreter で検査できる",
    "b": "宣言にできるが、検査はこの決まり専用の Interpreter（Rust ・ AI ・ 人）を必要とする",
    "c": "v0.3 の試案の形に置き場所が無い（1件のインスタンスの外 ・ 時間 ・ 誰が書いたか ・ 導く決まり）",
    "-": "数えない（中身が無い）",
}

# ---- x-prompt の rule の分類（キーは board.schema.json の "rule" の行） ----------------
# 値: (checked, v03, AIに意味を見させるか, 標準のJSON Schemaで試した結果, note)
#   JSON Schema の列: "標準のキーワードで書けた" = jsonschema-trial で検査が効いた / "" = 試していない
BY_ID = {
    "as-described": ("D", "-", False, "", "description と同じことを言うだけで、独自の決まりを持たない"),
    "only-needed": ("M", "a", False, "", "test は「60行くらいまで」。行数は数えられるが、どこも数えていない"),
    "one-thing": ("M", "b", False, "", "test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする"),
    "one-sentence": ("M", "b", False, "", "test は「1文か」。文の切り方を持つ Interpreter を必要とする"),
    "why-highlight": ("M", "b", False, "", "test は「1文か」。文の切り方を持つ Interpreter を必要とする"),
    "as-is": ("S", "a", False, "", "同じ節点の const が検査する"),
    "as-in-repo": ("O", "c", False, "", "test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする"),
    "lowercase": ("M", "a", False, "", "test は「知っている名前か」。言語の名前の一覧を持てば汎用の検査で足りる"),
    "numbered": ("A", "b", True, "", "test は「1件に1つのことか」。意味の判断"),
    "verbatim": ("O", "c", False, "", "test は「原文と同じか」。リポジトリのある時点のファイルを必要とする"),
}
BY_LINE = {
    14: ("A", "b", True, "", "test は「調べれば分かることが入っていないか」。意味の判断"),
    20: ("M", "a", False, "標準のキーワードで書けた", "test は「rejected が1件以上か」。rejected に minItems が無く、どこも数えていない"),
    50: ("S", "a", False, "", "同じ節点の const が検査する"),
    67: ("S", "a", False, "", "同じ節点の const が検査する"),
    84: ("O", "c", False, "", "test は「ファイル名と同じか」。xref-trial の id-differs-file で、check も inspect も見ないことを確かめた"),
    102: ("M", "a", False, "標準のキーワードで書けた", "test は「40字を超えていないか」。title は minLength 1 だけ"),
    120: ("M", "b", False, "", "test は「1文か」。文の切り方を持つ Interpreter を必要とする"),
    138: ("O", "c", False, "", "rule は「AI は書かない」。誰が書いたかはインスタンスに残らない。test の date の有無は required が見る"),
    159: ("S", "a", False, "", "同じ節点の pattern が検査する"),
    177: ("S", "a", False, "", "同じ節点の minLength が検査する"),
    197: ("A", "b", True, "", "test は「true のボードが少数か」。少数の線が決まっておらず、ほかの作業を止めているかは意味の判断"),
    214: ("R", "c", False, "", "x-ref（to: board）を check がディレクトリの全部のボードで検査する。v0.3 の要求の例はインスタンスを1件しか渡さない"),
    239: ("W", "c", False, "", "advance が書く（advance.rs:142）。test の「rounds/ の最後の番号と同じか」は誰も見ない。rounds/ はインスタンスの外"),
    257: ("S", "a", False, "", "test は「日付の形か」で pattern が検査する。書くのは advance"),
    275: ("I", "a", False, "x-ref で書けた", "inspect.rs:55 と :74 が見る。xref-trial で x-ref の accept と when でも書けた"),
    343: ("W", "c", False, "", "advance が前の回と比べて書く（advance.rs:162-205）。前の回はインスタンスの外"),
    458: ("M", "a", False, "", "test は「日付が新しい順か」。どこも見ていない"),
    519: ("M", "a", False, "標準のキーワードで書けた", "test は「件数」。minItems 1 だけで、実物のボード2枚とも6件（measure_unchecked.out.json）"),
    544: ("A", "b", True, "", "test は「答えの1文が1つのことを決めているか」。意味の判断"),
    569: ("M", "a", False, "", "test は「重なりが無いか」。xref-trial の dup-topic-id で、重なった id は参照されたときにだけ「1つに決まらない」で出た"),
    587: ("M", "a", False, "", "test は「長すぎないか」。線が「10字くらい」で決まっていない"),
    608: ("I", "a", False, "", "test は「queue と合っているか」。inspect.rs:50 ・ :55 ・ :65 が見る"),
    626: ("A", "b", True, "", "test は「答えの1文が問いに答えているか」。意味の判断"),
    652: ("I", "b", False, "", "test は「text が120字までか」。inspect.rs:105 が見るのは欄ではなく1文の長さ（引用を除く）"),
    740: ("I", "a", False, "標準のキーワードで書けた", "inspect.rs:123 が open の論点だけ見る。if/then で書けた"),
    746: ("A", "b", True, "", "test は「code がファイルとして成り立つ本文か」。inspect.rs:113 は言語が text などかだけを見る"),
    812: ("S", "a", False, "", "同じ節点の maxItems 3 が検査する"),
    877: ("S", "a", False, "", "items の required が4つを検査する"),
    981: ("M", "a", False, "標準のキーワードで書けた", "test は「1件以上あるか」。rejected に minItems が無い"),
    1042: ("R", "a", False, "", "x-ref（to: self）を check が、inspect.rs:72 も検査する"),
    1077: ("W", "b", False, "", "x-view の hidden に render が従う。test の「画面に出ていないか」は描いてみないと分からない"),
    1350: ("W", "b", False, "", "contentMediaType image/svg+xml の入力を render が検査する（schema-driven SKILL.md:173）"),
    1428: ("M", "b", False, "", "test は「外の URL を読んでいないか」。この実験で読んだ範囲では、見ているコードを見つけていない"),
    1447: ("A", "b", True, "", "test は「描いて目で見る」。人の目の判断"),
    1550: ("I", "a", False, "標準のキーワードで書けた", "inspect.rs:113 が見る。not enum で書けた（ただし大文字小文字を区別する）"),
    1574: ("A", "b", True, "", "test は「path のファイルの本文として成り立つか」。言語ごとの読み手を必要とする"),
    1596: ("A", "b", True, "", "test は「全体の行の数より十分少ないか」。十分の線が無い"),
    1766: ("S", "a", False, "", "同じ節点の enum が検査する"),
    1856: ("M", "b", False, "", "test は「depth が親より1つ深いか」。並びの前後を比べる専用の検査を必要とする"),
    2009: ("O", "c", False, "", "test は「場所が在るか」"),
    2027: ("O", "c", False, "", "test は「実行できるか」"),
    2044: ("O", "c", False, "", "test は「実物の出力と比べる」"),
    2061: ("S", "a", False, "", "同じ節点の type integer が検査する"),
}

# ---- コードと SKILL.md の決まり（読んで書き出したもの） ------------------------------
# (id, 出どころ, 決まり, expressed, checked, v03, AI, JSON Schema, 同じ決まり, note)
CODE = [
    ("I1", "src/brainstorming/crates/core/src/domain/inspect.rs:40", "回答待ち（open）の論点は5件まで", "Rust", "I", "a", False, "標準のキーワードで書けた", "", "contains と maxContains で書けた"),
    ("I2", "src/brainstorming/crates/core/src/domain/inspect.rs:50", "open の論点があれば queue は空でない", "Rust", "I", "a", False, "標準のキーワードで書けた", "x-prompt:608", "if/then で書けた"),
    ("I3", "src/brainstorming/crates/core/src/domain/inspect.rs:58", "queue の topic が指す論点がある", "Rust", "I", "a", False, "", "x-ref:/properties/queue/items/properties/topic", "x-ref の check と同じことを見る"),
    ("I4", "src/brainstorming/crates/core/src/domain/inspect.rs:55", "queue の topic が指す論点は open", "Rust", "I", "a", False, "x-ref で書けた", "x-prompt:275", "accept（at: status, in: [open]）"),
    ("I5", "src/brainstorming/crates/core/src/domain/inspect.rs:62", "ボードに status を書かない", "Rust", "I", "a", False, "標準のキーワードで書けた", "G8", "根の additionalProperties:false がすでに断る（xref-trial/boards/board-status.json で確かめた）"),
    ("I6", "src/brainstorming/crates/core/src/domain/inspect.rs:65", "open が無いのに決着していない論点があれば誤り", "Rust", "I", "a", False, "", "x-prompt:608", "anyOf と contains で書けるはずだが試していない"),
    ("I7", "src/brainstorming/crates/core/src/domain/inspect.rs:72", "depends が指す論点がある", "Rust", "I", "a", False, "", "x-prompt:1042", "x-ref の check と同じことを見る"),
    ("I8", "src/brainstorming/crates/core/src/domain/inspect.rs:74", "open の論点の depends はすべて settled", "Rust", "I", "a", False, "x-ref で書けた", "x-prompt:275", "when（status: [open]）と accept（at: status, in: [settled]）"),
    ("I9", "src/brainstorming/crates/core/src/domain/inspect.rs:92", "答えの本文が指す論点（Q と数字）は depends か下流にある", "Rust", "I", "b", False, "", "", "本文から id を拾う決まり（inspect.rs:12 の mentioned）を持つ Interpreter を必要とする"),
    ("I10", "src/brainstorming/crates/core/src/domain/inspect.rs:105", "答え ・ 詳しい決まり ・ 外した理由の1文は120字まで（「」の中は数えない）", "Rust", "I", "b", False, "", "x-prompt:652", "文の切り方は text.rs:7 ・ 上限は text.rs:4"),
    ("I11", "src/brainstorming/crates/core/src/domain/inspect.rs:113", "code の完成イメージの言語は text ・ plain ・ txt ・ 空でない", "Rust", "I", "a", False, "標準のキーワードで書けた", "x-prompt:1550", "not enum で書けた"),
    ("I12", "src/brainstorming/crates/core/src/domain/inspect.rs:123", "open の論点には完成イメージがある", "Rust", "I", "a", False, "標準のキーワードで書けた", "x-prompt:740", "if/then で書けた"),
    ("I13", "src/brainstorming/crates/core/src/domain/inspect.rs:132", "open の論点は、下流の論点に使われるか、外の作業の行き先が out_of_scope にある", "Rust", "I", "b", False, "", "", "逆向きの参照と文字列の中身の「または」。x-ref の inverse には「または」が無い"),
    ("D1", "src/brainstorming/crates/core/src/domain/status.rs:69", "論点のステータス：settled→決着、open→回答待ち、ほか→前提待ち", "Rust", "W", "c", False, "", "", "いまの基盤の x-derive（rules ・ otherwise）なら書ける形。v0.3 の試案には導く決まりの置き場所が無い"),
    ("D2", "src/brainstorming/crates/core/src/domain/status.rs:79", "AI が動いていればボードは更新中", "Rust", "W", "c", False, "", "", "ロックのファイルとプロセスの状態"),
    ("D3", "src/brainstorming/crates/core/src/domain/status.rs:82", "hold があればボードは保留", "Rust", "W", "c", False, "", "", "導く決まり。x-derive の when は値の一致だけで、有無を見られない"),
    ("D4", "src/brainstorming/crates/core/src/domain/status.rs:86", "論点がすべて settled（1件以上）ならボードは決着", "Rust", "W", "c", False, "", "", "導く決まり。全部についての条件"),
    ("D5", "src/brainstorming/crates/core/src/domain/status.rs:89", "それ以外のボードは回答待ち", "Rust", "W", "c", False, "", "", "導く決まり"),
    ("L1", "src/brainstorming/crates/core/src/domain/lock.rs:206", "ロックが新しいあいだは持ち主だけが書く", "Rust", "W", "c", False, "", "", "プロセスと時間"),
    ("L2", "src/brainstorming/crates/core/src/domain/lock.rs:175", "serve のロックは60秒で古くなる", "Rust", "W", "c", False, "", "", "時間"),
    ("L3", "src/brainstorming/crates/core/src/domain/lock.rs:177", "ターミナルのロックは10分で古くなる", "Rust", "W", "c", False, "", "", "時間"),
    ("L4", "src/brainstorming/crates/core/src/domain/lock.rs:190", "ロックの中身は board ・ owner ・ what ・ since ・ beat", "Rust", "W", "a", False, "", "", "形の決まりだがスキーマが無く、コードの中にだけある"),
    ("G1", ".claude/skills/brainstorming/SKILL.md:119", "案を並べて選ばせない", "SKILL.md の文", "A", "b", True, "", "x-prompt:20", ""),
    ("G2", ".claude/skills/brainstorming/SKILL.md:120", "事実を利用者に聞かない", "SKILL.md の文", "A", "b", True, "", "x-prompt:14", ""),
    ("G3", ".claude/skills/brainstorming/SKILL.md:121", "外した案を黙って消さない", "SKILL.md の文", "W", "c", False, "", "", "advance が数の増減を changes に書く（advance.rs:177）。前の回はインスタンスの外"),
    ("G4", ".claude/skills/brainstorming/SKILL.md:122", "ボードの JSON を手で書き換えない", "SKILL.md の文", "O", "c", False, "", "", "誰がどう書いたか"),
    ("G5", ".claude/skills/brainstorming/SKILL.md:123", "rounds/ と answers/ を書き換えない", "SKILL.md の文", "O", "c", False, "", "", "この実験で読んだ範囲では、見ているコードを見つけていない"),
    ("G6", ".claude/skills/brainstorming/SKILL.md:124", "ボードのフォルダの外を書き換えない", "SKILL.md の文", "W", "c", False, "", "", "serve が作業の前後を比べて知らせる（scope.rs:14）"),
    ("G7", ".claude/skills/brainstorming/SKILL.md:125", "AI の「検査した」で合格としない", "SKILL.md の文", "O", "a", False, "", "", "決まりではなく手順。v0.3 では Interpreter の応答の status と providedCapabilities が持つ"),
    ("G8", ".claude/skills/brainstorming/SKILL.md:126", "ボードに status を書かない", "SKILL.md の文", "S", "a", False, "標準のキーワードで書けた", "I5", "根の additionalProperties:false"),
    ("G9", ".claude/skills/brainstorming/SKILL.md:127", "ロックを残したまま作業をやめない", "SKILL.md の文", "W", "c", False, "", "", "外し忘れは lock.rs の時間で外れる"),
    ("G10", ".claude/skills/brainstorming/SKILL.md:128", "画面の AI に聞くで、Kiro は読むだけ", "SKILL.md の文", "W", "c", False, "", "", "agent.rs:3 が起動のしかたで絞る"),
    ("G11", ".claude/skills/brainstorming/SKILL.md:129", "扱わない範囲に扱いを付ける", "SKILL.md の文", "S", "a", False, "", "", "out_of_scope の items の required と enum"),
]

# ---- スキーマの形の決まり ----------------------------------------------------------
KEYWORDS = ["type", "const", "enum", "pattern", "minLength", "maxLength", "minItems", "maxItems",
            "uniqueItems", "minimum", "maximum", "additionalProperties", "required", "oneOf",
            "contentMediaType", "x-ref"]


def line_of(lines, start, key):
    """start 行より後で、"key": が最初に出る行（1始まり）。"""
    pat = re.compile(r'^\s*"' + re.escape(key) + r'":')
    for i in range(start, len(lines)):
        if pat.match(lines[i]):
            return i + 1
    return None


def scan(schema_path):
    text = open(schema_path, encoding="utf-8").read()
    lines = text.split("\n")
    schema = json.loads(text)
    rule_lines = [i + 1 for i, l in enumerate(lines) if re.match(r'\s*"rule":', l)]
    shape, prompts = [], []
    cursor = [0]
    rule_i = [0]

    def walk(node, ptr):
        if isinstance(node, dict):
            # 形の決まり
            for key in KEYWORDS:
                if key not in node:
                    continue
                if key == "additionalProperties" and node[key] is not False:
                    continue
                if key == "type" and not isinstance(node[key], str):
                    continue
                ln = line_of(lines, cursor[0], key)
                if key == "required":
                    for name in node[key]:
                        shape.append({"ptr": ptr, "keyword": "required", "value": name, "line": ln})
                else:
                    val = node[key] if not isinstance(node[key], (dict, list)) else json.dumps(node[key], ensure_ascii=False)
                    shape.append({"ptr": ptr, "keyword": key, "value": val, "line": ln})
            xp = node.get("x-prompt")
            if isinstance(xp, dict):
                for w in xp.get("write", []):
                    prompts.append({"ptr": ptr, "id": w["id"], "rule": w["rule"], "test": w.get("test"),
                                    "line": rule_lines[rule_i[0]]})
                    rule_i[0] += 1
            for k, v in node.items():
                if k == "x-prompt":
                    continue
                if isinstance(v, (dict, list)):
                    ln = line_of(lines, cursor[0], k) if isinstance(node, dict) else None
                    if ln:
                        cursor[0] = ln - 1
                walk(v, ptr + "/" + k)
        elif isinstance(node, list):
            for i, v in enumerate(node):
                walk(v, ptr + "/" + str(i))

    walk(schema, "")
    return shape, prompts


def main():
    shape, prompts = scan(SCHEMA)
    rows = []
    for s in shape:
        is_xref = s["keyword"] == "x-ref"
        v03 = "a"
        note = ""
        if is_xref and s["ptr"] == "/properties/waits_for":
            v03, note = "c", "ほかのボードを指す。v0.3 の要求の例はインスタンスを1件しか渡さない"
        rows.append({
            "id": f"S{len(rows) + 1}", "source": f"{SCHEMA}:{s['line']}", "where": s["ptr"] or "/",
            "rule": f"{s['keyword']} = {s['value']}", "expressed": "x-ref（独自の注釈）" if is_xref else "JSON Schema",
            "checked": "R" if is_xref else "S", "v03": v03, "ai": False, "jsonschema": "", "same_as": "", "note": note,
            "group": "形",
        })
    for n, p in enumerate(prompts, 1):
        c = BY_LINE.get(p["line"]) or BY_ID.get(p["id"])
        if c is None:
            raise SystemExit(f"分類が無い: {p}")
        checked, v03, ai, js, note = c
        rows.append({
            "id": f"P{n}", "source": f"{SCHEMA}:{p['line']}", "where": p["ptr"] or "/",
            "rule": f"{p['id']}：{p['rule']}（test：{p['test']}）", "expressed": "x-prompt の文",
            "checked": checked, "v03": v03, "ai": ai, "jsonschema": js, "same_as": "", "note": note,
            "group": "x-prompt",
        })
    for (rid, src, rule, expressed, checked, v03, ai, js, same, note) in CODE:
        group = {"I": "inspect.rs", "D": "status.rs", "L": "lock.rs", "G": "SKILL.md"}[rid[0]]
        rows.append({"id": rid, "source": src, "where": "", "rule": rule, "expressed": expressed,
                     "checked": checked, "v03": v03, "ai": ai, "jsonschema": js, "same_as": same,
                     "note": note, "group": group})

    summary = {
        "by_group": Counter(r["group"] for r in rows),
        "shape_by_keyword": Counter(r["rule"].split(" = ")[0] for r in rows if r["group"] == "形"),
        "checked_by_group": {g: Counter(r["checked"] for r in rows if r["group"] == g)
                             for g in ["形", "x-prompt", "inspect.rs", "status.rs", "lock.rs", "SKILL.md"]},
        "v03_by_group": {g: Counter(r["v03"] for r in rows if r["group"] == g)
                         for g in ["形", "x-prompt", "inspect.rs", "status.rs", "lock.rs", "SKILL.md"]},
        "v03_by_checked_outside_shape": {c: Counter(r["v03"] for r in rows if r["group"] != "形" and r["checked"] == c)
                                         for c in CHECKED},
        "xprompt_ai_meaning": sum(1 for r in rows if r["group"] == "x-prompt" and r["ai"]),
        "skill_ai_meaning": sum(1 for r in rows if r["group"] == "SKILL.md" and r["ai"]),
        "tried_standard": sum(1 for r in rows if r["jsonschema"] == "標準のキーワードで書けた"),
        "tried_xref": sum(1 for r in rows if r["jsonschema"] == "x-ref で書けた"),
    }
    out = {"_note": "count.py が実物のファイルから書いたもの。手で直さない。",
           "legend": {"checked": CHECKED, "v03": V03}, "summary": summary, "rules": rows}
    with open(os.path.join(HERE, "inventory.json"), "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)

    # README に貼る表（形の決まりはキーワードごとにまとめ、ほかは1件ずつ）
    md = ["| id | 出どころ | 決まり | いま表している所 | いまの検査 | v0.3 | いまの基盤で試した結果 | 同じ決まり | メモ |",
          "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        if r["group"] == "形":
            continue
        src = r["source"].replace(".claude/skills/brainstorming/", "brainstorming/").replace(
            "src/brainstorming/crates/core/src/domain/", "domain/")
        rule = r["rule"].replace("|", "／")
        where = f"`{r['where']}` " if r["where"] else ""
        md.append(f"| {r['id']} | {src} | {where}{rule} | {r['expressed']} | {r['checked']} | {r['v03']} | "
                  f"{r['jsonschema']} | {r['same_as']} | {r['note']} |")
    with open(os.path.join(HERE, "inventory.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(md) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
