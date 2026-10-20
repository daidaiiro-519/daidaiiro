# 第49回の図：Q3 を、ポートとアダプターの4つの層とドメインサービス3つで組み直す。
# 材料は advisors/ddd-engine-layers.json（ddd-advisor）。
import runpy
T = runpy.run_path('make13_table.py')
table, e = T['table'], T['e']

def chip(o, x, y, w, t, cls="box"):
    o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="30"/><text class="label" x="{x+10}" y="{y+20}">{e(t)}</text>')

# ── 層の図 ──
o = ['<svg viewBox="0 0 1014 470" style="min-width:900px" role="img" aria-label="Semantic Agent Schema のエンジンの4つの層">']
o.append('<rect class="area" x="16" y="16" width="170" height="438"/><text class="title" x="30" y="40">inbound adapter</text>')
chip(o, 30, 200, 142, "CLI"); chip(o, 30, 240, 142, "MCP")
o.append('<rect class="area" x="230" y="16" width="220" height="438"/><text class="title" x="244" y="40">アプリケーション層</text>')
o.append('<text class="note" x="244" y="60">command ごとの関数</text>')
for k, t in enumerate(["create ・ get", "update ・ delete", "prompt", "check ・ check-schemas", "approve", "render", "publish（無い）", "走査 ・ 索引（無い）"]):
    chip(o, 244, 76 + 40 * k, 192, t)
o.append('<text class="note" x="244" y="420">読む → 集約のコマンド →</text><text class="note" x="244" y="438">保存する → 結果を返す</text>')
o.append('<rect class="area" x="494" y="16" width="290" height="438"/><text class="title" x="508" y="40">ドメイン層</text>')
o.append('<text class="note" x="508" y="62">集約</text>')
chip(o, 508, 72, 262, "Semantic Concrete Schema", "box focus"); chip(o, 508, 108, 262, "instance", "box focus"); chip(o, 508, 144, 262, "承認の記録", "box focus")
o.append('<text class="note" x="508" y="196">ドメインサービス</text>')
chip(o, 508, 206, 262, "Semantic Schema Validator"); chip(o, 508, 242, 262, "Semantic Schema Resolver"); chip(o, 508, 278, 262, "Semantic Schema Renderer")
o.append('<text class="note" x="508" y="330">ポート（trait）</text>')
chip(o, 508, 340, 262, "リポジトリ4つ", "boundary"); chip(o, 508, 376, 262, "RuleValidator（rule の判定を頼む）", "boundary")
o.append('<text class="note" x="508" y="428">値オブジェクトもここに置く</text><text class="note" x="508" y="446">ライブラリ：JSON Schema の実装 ・ JMESPath</text>')
o.append('<rect class="area" x="828" y="16" width="170" height="438"/><text class="title" x="842" y="40">outbound adapter</text>')
for k, t in enumerate(["ファイル（JSON）", "Claude Code", "Codex", "Kiro", "GitHub Copilot"]):
    chip(o, 842, 200 + 40 * k, 142, t)
o.append('<path class="flow" d="M186,235 H230"/><path class="flow" d="M450,235 H494"/><path class="flow" d="M828,355 H784"/>')
o.append('</svg>')
open('q3-layers.src.svg', 'w').write('\n'.join(o))

# ── command ごとの表 ──
CMD = [
 ("create", "変える", "instance", "Validator ・ Resolver", ["ConcreteSchemaRepository（読む）・", "InstanceRepository（保存する）"]),
 ("get", "変えない", "──", "Resolver", "InstanceRepository（読む）"),
 ("update", "変える", "instance", "Validator ・ Resolver", ["ConcreteSchemaRepository（読む）・", "InstanceRepository（読む ・ 保存する）"]),
 ("delete", "変える", "instance", "Resolver（指される数）", "InstanceRepository（読む ・ 一覧 ・ 消す）"),
 ("prompt", "変えない", "──", "──", "ConcreteSchemaRepository（読む）"),
 ("check", "変えない", "──", "Validator ・ Resolver", ["ConcreteSchemaRepository ・ InstanceRepository ・", "RuleValidator"]),
 ("check-schemas", "変えない", "──", "──（Concrete の集約の検査）", "ConcreteSchemaRepository（一覧 ・ 読む）"),
 ("approve", "変える", "承認の記録", "Validator ・ Resolver", ["check のポートと", "ApprovalRepository（読む ・ 保存する）"]),
 ("render", "変えない", "──", "Renderer ・ Resolver", ["ViewRepository ・ ConcreteSchemaRepository ・", "InstanceRepository（どれも読む）"]),
 ("publish（無い）", "変える", "Semantic Concrete Schema", "──", ["ConcreteSchemaRepository", "（読む ・ 公開の記録を足す）"]),
 ("走査 ・ 索引（無い）", "変えない", "──", "Resolver", ["InstanceRepository（一覧 ・ 読む）・", "ConcreteSchemaRepository（読む）"]),
]
table('q3-commands', 'command ごとの、状態を変えるかと、保存する集約 ・ 使うドメインサービス ・ 呼ぶポート',
      ["command", "状態を変えるか", "保存する集約", "ドメインサービス", "呼ぶポート"], [160, 120, 150, 210, 400],
      [list(r) for r in CMD], emph=(0,), row_h=46)

# ── ポートの表 ──
PORT = [
 ("ConcreteSchemaRepository", "読む（$ref の先を含む）・ 一覧 ・ 保存する ・ 公開の記録を足す", "ファイル（JSON）"),
 ("InstanceRepository", "読む ・ 一覧 ・ 保存する（読んだ時点のハッシュで排他）・ 消す", "ファイル（JSON）"),
 ("ApprovalRepository", "読む ・ 保存する", "ファイル（JSON）"),
 ("ViewRepository", "ページテンプレート ・ カタログ ・ デザインシステムを読む", "ファイル（JSON）"),
 ("RuleValidator", "rule の判定を外の validator に頼む（約束は Agents 契約）", ["Claude Code ・ Codex ・", "Kiro ・ GitHub Copilot"]),
]
table('q3-ports', 'ユースケースが呼ぶポート5つと、それを実装するアダプター',
      ["ポート", "操作（ユースケースの言葉）", "アダプター（技術）"], [230, 470, 220], [list(r) for r in PORT], emph=(0,), row_h=46)

# ── 構想 v0.3 との対応 ──
V = [
 ("Schema Validator", "Validator の中の JSON Schema の validator"),
 ("Vocabulary Resolver", "Resolver の $ref → term"),
 ("Contract Registry", "Semantic Concrete Schema の集約（公開した版）と ConcreteSchemaRepository"),
 ("Relation/Graph Engine ・ Constraint Engine", "Resolver の x-ref と Validator の rule"),
 ("Semantic Interpreter", "Validator が RuleValidator のポートで呼ぶ AI の validator"),
 ("Verification Engine", "Validator の全体（検証結果の5つの値）"),
 ("（無い）", "Semantic Schema Renderer"),
]
table('q3-v03', '構想 v0.3 の部品と、Semantic Agent Schema で受け持つもの',
      ["構想 v0.3 の部品（参考）", "Semantic Agent Schema で受け持つもの"], [360, 520], [list(r) for r in V], emph=(1,), row_h=34)
