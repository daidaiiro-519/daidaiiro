# 型 v1 の試験データを作る。合格例から1か所だけ変えた不合格例を、不合格になる場合ごとに1つ置く。
# 作り直すとき：python3 tests/gen_data.py（src/skills-creator/ で実行する）
import json, os, shutil
ROOT = "tests/data/skill-v1"
SK = """---
name: {name}
description: {desc}
---

# 用語集を JSON で持つ Skill：{name}

## 目的
用語の定義を、検証できる形で1か所に持つ。

## 役割
語を足す ・ 直す ・ 検査する。

## 入力の想定
足す語と、その定義。

## 実行手順
### Step 1: 語を足す
scripts/schema-driven --skill . create --schema assets/term.schema.json --path terms/<語>.json

## 出力形式
作ったインスタンスのパス。

## ガードレール
定義の無い語を足さない。
"""
DESC = "用語集を JSON のインスタンスで持ち、語を足す ・ 直す ・ 検査するときに使う。"
TOOL = {"$schema": "skills-creator/tool@v1", "abstract": "schema-driven/abstract@v1", "command": "scripts/schema-driven"}
TERM = {"$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1",
        "$id": "glossary-keeper/term@v1", "title": "用語", "description": "用語集の1語",
        "type": "object", "required": ["kind", "word"],
        "properties": {
            "kind": {"const": "term", "description": "種類",
                     "x-prompt": {"read": {"means": "用語の1件であること"}, "write": [{"id": "kind", "rule": "term と書く", "why": "種類で読み分ける", "test": "値が term である"}]}},
            "word": {"type": "string", "description": "語",
                     "x-prompt": {"read": {"means": "用語そのもの"}, "write": [{"id": "word", "rule": "1語を書く", "why": "索引に使う", "test": "空でない"}]}}}}
BASE = SK.format(name="glossary-keeper", desc=DESC)

def pad(text, lines):
    n = len(text.splitlines())
    return text + "\n" * (lines - n)

expected = []
def case(path, fails, skill=None, name="glossary-keeper", desc=DESC, folder=None, tool=None, schema=True, files=None):
    d = os.path.join(ROOT, path, folder or name)
    os.makedirs(d)
    open(os.path.join(d, "SKILL.md"), "w").write(skill if skill is not None else SK.format(name=name, desc=desc))
    json.dump(tool or TOOL, open(os.path.join(d, "tool.json"), "w"), ensure_ascii=False)
    if schema:
        os.makedirs(os.path.join(d, "assets"))
        json.dump(TERM, open(os.path.join(d, "assets/term.schema.json"), "w"), ensure_ascii=False, indent=1)
    for rel, body in (files or {}).items():
        os.makedirs(os.path.dirname(os.path.join(d, rel)), exist_ok=True)
        open(os.path.join(d, rel), "w").write(body)
    expected.append({"skill": os.path.relpath(d, ROOT), "fails": fails})

shutil.rmtree(ROOT, ignore_errors=True)
# 合格例と、境界の合格側
case("valid/minimal", [])
case("valid/name-64-chars", [], name="g" * 64)
case("valid/description-1024-chars", [], desc="あ" * 1024)
case("valid/compatibility-500-chars", [], skill=BASE.replace("---\n\n", "compatibility: " + "x" * 500 + "\n---\n\n", 1))
case("valid/lines-499", [], skill=pad(BASE, 499))
case("valid/quoted-name", [], skill=BASE.replace("name: glossary-keeper", 'name: "glossary-keeper"', 1))
case("valid/metadata-map", [], skill=BASE.replace("---\n\n", "metadata:\n  author: example\n  version: \"1.0\"\n---\n\n", 1))
case("valid/link-exists", [], skill=BASE.replace("定義の無い語を足さない。", "定義の無い語を足さない。詳しくは [書き方](references/WRITING.md) を読む。"),
     files={"references/WRITING.md": "# 書き方\n"})
case("valid/extra-section", [], skill=BASE.replace("## 出力形式", "## 例\n足す語の例。\n\n## 出力形式"))
# 先頭の欄
case("invalid/frontmatter-missing", ["skill.frontmatter"], skill=BASE.split("---\n", 2)[2])
case("invalid/frontmatter-not-yaml", ["skill.frontmatter"], skill=BASE.replace("name: glossary-keeper", "name: [glossary-keeper", 1))
case("invalid/field-version", ["skill.fields"], skill=BASE.replace("---\n\n", "version: 1.0.0\n---\n\n", 1))
# name
case("invalid/name-65-chars", ["skill.name"], name="g" * 65)
case("invalid/name-uppercase", ["skill.name"], name="Glossary-keeper")
case("invalid/name-leading-hyphen", ["skill.name"], name="-glossary")
case("invalid/name-trailing-hyphen", ["skill.name"], name="glossary-")
case("invalid/name-double-hyphen", ["skill.name"], name="glossary--keeper")
case("invalid/name-not-folder", ["skill.name"], folder="glossary")
case("invalid/name-missing", ["skill.name"], skill=BASE.replace("name: glossary-keeper\n", "", 1))
# description
case("invalid/description-1025-chars", ["skill.description"], desc="あ" * 1025)
case("invalid/description-missing", ["skill.description"], skill=BASE.replace("description: " + DESC + "\n", "", 1))
# compatibility
case("invalid/compatibility-501-chars", ["skill.compatibility"], skill=BASE.replace("---\n\n", "compatibility: " + "x" * 501 + "\n---\n\n", 1))
# 行数
case("invalid/lines-500", ["skill.lines"], skill=pad(BASE, 500))
# 6つの節
case("invalid/section-missing", ["skill.sections"], skill=BASE.replace("## 役割\n語を足す ・ 直す ・ 検査する。\n\n", ""))
case("invalid/section-duplicate", ["skill.sections"], skill=BASE.replace("## 出力形式", "## 役割\nもう一度。\n\n## 出力形式"))
case("invalid/section-order", ["skill.sections"], skill=BASE.replace("## 目的", "## 役割X").replace("## 役割\n", "## 目的\n").replace("## 役割X", "## 役割"))
# 記入欄
case("invalid/placeholder-left", ["skill.placeholders"], skill=BASE.replace("定義の無い語を足さない。", "{{…}}"))
# 本文のパス
case("invalid/body-tool-folder", ["skill.body.paths"], skill=BASE.replace("scripts/schema-driven --skill .", ".claude/skills/glossary-keeper/scripts/schema-driven --skill ."))
case("invalid/body-variable", ["skill.body.paths"], skill=BASE.replace("scripts/schema-driven --skill .", "${CLAUDE_PROJECT_DIR}/scripts/schema-driven --skill ."))
case("invalid/body-absolute-path", ["skill.body.paths"], skill=BASE.replace("scripts/schema-driven --skill .", "/home/user/glossary-keeper/scripts/schema-driven --skill ."))
# リンク
case("invalid/link-missing", ["skill.links"], skill=BASE.replace("定義の無い語を足さない。", "定義の無い語を足さない。詳しくは [書き方](references/WRITING.md) を読む。"))
# tool.json
case("invalid/tool-old-form", ["tool.schema"], tool={"contract": 3, "cli": {"command": "bin/glossary-keeper"}})
case("invalid/tool-extra-key", ["tool.schema"], tool=dict(TOOL, build=[["cargo", "build"]]))
case("invalid/tool-no-abstract", ["tool.abstract"], tool={k: v for k, v in TOOL.items() if k != "abstract"})
case("invalid/tool-unknown-abstract", ["tool.abstract"], tool=dict(TOOL, abstract="schema-driven/abstract@v9"))
case("invalid/tool-command-variable", ["tool.command"], tool=dict(TOOL, command="${CLAUDE_PROJECT_DIR}/scripts/schema-driven"))
case("invalid/tool-command-absolute", ["tool.command"], tool=dict(TOOL, command="/usr/local/bin/schema-driven"))
case("invalid/tool-command-outside", ["tool.command"], tool=dict(TOOL, command="../schema-driven/scripts/schema-driven"))
# assets のスキーマ（基盤の check-schemas の結果）
bad = dict(TERM); bad["properties"] = {"kind": {"const": "term", "description": "種類"}}
case("invalid/schema-no-prompt", ["assets.schemas"], files={})
os.remove(os.path.join(ROOT, "invalid/schema-no-prompt/glossary-keeper/assets/term.schema.json"))
json.dump(dict(bad, required=["kind"]), open(os.path.join(ROOT, "invalid/schema-no-prompt/glossary-keeper/assets/term.schema.json"), "w"), ensure_ascii=False)
# 警告だけ（合否に入れない）
case("warning/body-ai-name", [], skill=BASE.replace("定義の無い語を足さない。", "Claude は定義の無い語を足さない。"))
json.dump({"$schema": "../expected.schema.json", "kind": "expected", "cases": expected},
          open(os.path.join(ROOT, "expected.json"), "w"), ensure_ascii=False, indent=1)
print(len(expected), "件")
