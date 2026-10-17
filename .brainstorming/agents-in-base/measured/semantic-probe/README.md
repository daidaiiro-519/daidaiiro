# 3つの CLI に意味の検証をさせた実測（2026-10-17）

task.txt のインスタンス（問い「練習の Skill の入口をどこに置くか」に、答え「SKILL.md は日本語で書く。」をわざと付けた論点）と4つの決まりを渡し、verdict.schema.json の形で答えさせた。

| CLI | 起動 | 形の縛り | 結果 |
|---|---|---|---|
| Claude Code 2.1.301 | `claude -p --output-format json --json-schema <schema> --permission-mode dontAsk` | --json-schema | decidable ・ one-sentence-first ・ keep が ng、fits-repo が unknown |
| Codex 0.163.0 | `codex exec --json --sandbox read-only --output-schema verdict.schema.json -` | --output-schema | 同じ |
| Kiro 2.21.1 | `kiro-cli chat --agent-engine v2 --output-format stream-json --trust-tools= "<依頼>"` | 無い（依頼の文に形を書いた） | 同じ。答えはコードの囲みの中の JSON |

fits-repo（リポジトリの CODING.md と食い違わないか）は、3つとも材料が無いので unknown と答えた。
