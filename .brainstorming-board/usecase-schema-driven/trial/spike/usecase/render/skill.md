---
name: {{id}}
description: {{name}}（宣言から導出。手で書かない）
---

# {{name}}

## 守ること
{{#each rules}}- {{name}}
{{/each}}
## できること
{{#each ops}}- `{{name}}` ── 事前 [{{#each pre ", "}}"{{.}}"{{/each}}]
{{/each}}