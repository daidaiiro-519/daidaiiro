# 第49回の図：Q3 を、ポートとアダプターの4つの層とドメインサービス3つで組み直す。
# 材料は advisors/ddd-engine-layers.json（ddd-advisor）。
import runpy
T = runpy.run_path('make13_table.py')
table, e = T['table'], T['e']

def chip(o, x, y, w, t, cls="box"):
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="30"/><text class="label" x="{x+10}" y="{y+20}">{e(t)}</text>')

# ── 層の図 ──
o = ['<svg viewBox="0 0 1014 470" style="min-width:900px" role="img" aria-label="Semantic Schema Core のエンジンの4つの層">']
o.append('<rect class="area" x="16" y="16" width="170" height="438"/><text class="title" x="30" y="40">inbound adapter</text>')
chip(o, 30, 200, 142, "CLI"); chip(o, 30, 240, 142, "MCP")
o.append('<rect class="area" x="230" y="16" width="220" height="438"/><text class="title" x="244" y="40">アプリケーション層</text>')
o.append('<text class="note" x="244" y="60">command ごとの関数</text>')
for k, t in enumerate(["create ・ get", "update ・ delete", "prompt", "check ・ check-schemas", "approve", "render", "publish（無い）", "走査 ・ 索引（無い）"]):
    chip(o, 244, 76 + 40 * k, 192, t)
o.append('<text class="note" x="244" y="420">読む → 集約のコマンド →</text><text class="note" x="244" y="438">保存する → 結果を返す</text>')
o.append('<rect class="area" x="494" y="16" width="290" height="438"/><text class="title" x="508" y="40">ドメイン層</text>')
o.append('<text class="note" x="508" y="62">集約</text>')
chip(o, 508, 72, 262, "Concrete スキーマ", "box focus"); chip(o, 508, 108, 262, "instance", "box focus"); chip(o, 508, 144, 262, "view", "box focus")
o.append('<text class="note" x="508" y="196">ドメインサービス</text>')
chip(o, 508, 206, 262, "Semantic Schema Validator"); chip(o, 508, 242, 262, "Semantic Schema Resolver"); chip(o, 508, 278, 262, "Semantic Schema Renderer")
o.append('<text class="note" x="508" y="330">ポート（trait）</text>')
chip(o, 508, 340, 262, "リポジトリ3つ", "boundary"); chip(o, 508, 376, 262, "技術のポート5つ", "boundary")
o.append('<text class="note" x="508" y="436">値オブジェクトもここに置く</text>')
o.append('<rect class="area" x="828" y="16" width="170" height="438"/><text class="title" x="842" y="40">outbound adapter</text>')
for k, t in enumerate(["ファイル", "JSON Schema の実装", "JMESPath", "カタログ", "AI の CLI"]):
    chip(o, 842, 200 + 40 * k, 142, t)
o.append('<path class="flow" d="M186,235 H230"/><path class="flow" d="M450,235 H494"/><path class="flow" d="M828,375 H784"/>')
o.append('</svg>')
open('q3-layers.src.svg', 'w').write('\n'.join(o))

# ── command ごとの表 ──
CMD = [
 ("create", "コマンド", "instance", "Validator ・ Resolver", ["InstanceRepository ・", "ConcreteSchemaRepository ・ JsonSchema"]),
 ("get", "クエリ", "instance", "Resolver", ["InstanceRepository ・", "ConcreteSchemaRepository ・ Query"]),
 ("update", "コマンド", "instance（JSON Patch）", "Validator ・ Resolver", ["InstanceRepository ・", "ConcreteSchemaRepository ・ JsonSchema"]),
 ("delete", "コマンド", "instance", "Resolver（指される数）", "InstanceRepository"),
 ("prompt", "クエリ", "Concrete スキーマ", "──", "ConcreteSchemaRepository"),
 ("check", "クエリ", "──", "Validator ・ Resolver", ["InstanceRepository ・ ConcreteSchemaRepository ・", "JsonSchema ・ Query ・ Agents"]),
 ("check-schemas", "クエリ", ["Concrete スキーマ", "（自身の検査）"], "──", ["ConcreteSchemaRepository ・", "AbstractSpec ・ JsonSchema"]),
 ("approve", "コマンド", ["instance", "（承認の記録）"], "Validator", ["InstanceRepository ・ ConcreteSchemaRepository ・", "JsonSchema ・ Agents"]),
 ("render", "クエリ", "──", "Renderer ・ Resolver", ["InstanceRepository ・ ConcreteSchemaRepository ・", "ViewRepository ・ Design"]),
 ("publish（無い）", "コマンド", ["Concrete スキーマ", "（公開した版）"], "──", ["ConcreteSchemaRepository ・", "AbstractSpec"]),
 ("走査 ・ 索引（無い）", "クエリ", "──", "Resolver", ["InstanceRepository ・", "ConcreteSchemaRepository ・ Query"]),
]
table('q3-commands', 'command ごとの種類と、使う集約 ・ ドメインサービス ・ ポート',
      ["command", "種類", "集約", "ドメインサービス", "ポート"], [160, 90, 190, 190, 420],
      [list(r) for r in CMD], emph=(0,), row_h=46)

# ── ポートの表 ──
PORT = [
 ("ConcreteSchemaRepository", "リポジトリ", "新しく作る"),
 ("InstanceRepository", "リポジトリ", "新しく作る。保存は読んだ時点のハッシュで排他する"),
 ("ViewRepository", "リポジトリ", "新しく作る"),
 ("AbstractSpec", "技術", "いまの Schemas の名前を変える"),
 ("JsonSchema", "技術", "新しく作る（いまは domain が jsonschema を直接に呼ぶ）"),
 ("Query", "技術", "いまのまま（JMESPath）"),
 ("Design", "技術", "いまのまま（カタログ）"),
 ("Agents", "技術", "新しく作る（AI の validator を呼ぶ）"),
]
table('q3-ports', 'ポート8つ：集約ごとのリポジトリ3つと技術のポート5つ',
      ["ポート", "種類", "いまとの差"], [250, 110, 480], [list(r) for r in PORT], emph=(0,), row_h=34)

# ── 構想 v0.3 との対応 ──
V = [
 ("Schema Validator", "Validator の中の JSON Schema の validator"),
 ("Vocabulary Resolver", "Resolver の $ref → term"),
 ("Contract Registry", "Concrete スキーマの集約（公開した版）と ConcreteSchemaRepository"),
 ("Relation/Graph Engine ・ Constraint Engine", "Resolver の x-ref と Validator の rule"),
 ("Semantic Interpreter", "Validator の中の AI の validator（Agents）"),
 ("Verification Engine", "Validator の全体（検証結果の5つの値）"),
 ("（無い）", "Semantic Schema Renderer"),
]
table('q3-v03', '構想 v0.3 の部品と、Semantic Schema Core で受け持つもの',
      ["構想 v0.3 の部品（参考）", "Semantic Schema Core で受け持つもの"], [360, 520], [list(r) for r in V], emph=(1,), row_h=34)
