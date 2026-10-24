"""ボード dependency-rules の中身（論点4つ）を JSON Patch にして、brainstorming update で入れる。"""
import json, os, subprocess
H = os.path.dirname(os.path.abspath(__file__)); R = os.path.dirname(os.path.dirname(H))
BIN = os.path.join(R, '.claude/skills/brainstorming/scripts/bin/brainstorming')
svg = open(os.path.join(H, 'figures/boundary.svg')).read()
SAS = {
 "components": [
  {"name": "declaration", "nodes": ["semantic-agent-schema-declaration"]},
  {"name": "change", "nodes": ["semantic-agent-schema-change"]},
  {"name": "validation", "nodes": ["semantic-agent-schema-validation"]},
  {"name": "render", "nodes": ["semantic-agent-schema-render"]},
  {"name": "search", "nodes": ["semantic-agent-schema-search"]},
  {"name": "instance", "nodes": ["semantic-agent-schema-instance"]},
  {"name": "migration-publish", "nodes": ["semantic-agent-schema-migration", "semantic-agent-schema-publish"]},
  {"name": "session", "nodes": ["semantic-agent-schema-session"]},
  {"name": "application", "nodes": ["semantic-agent-schema-application"]},
  {"name": "agents-contract", "nodes": ["semantic-agent-schema-agents-contract"]},
  {"name": "ai-cli", "nodes": ["semantic-agent-schema-ai-cli"]},
  {"name": "sqlite", "nodes": ["semantic-agent-schema-sqlite"]},
  {"name": "outbound", "nodes": ["semantic-agent-schema-git", "semantic-agent-schema-fs", "semantic-agent-schema-tools"]},
  {"name": "cli", "nodes": ["semantic-agent-schema-cli"]},
  {"name": "jsonschema", "nodes": ["jsonschema"]},
  {"name": "rusqlite", "nodes": ["rusqlite"]},
  {"name": "tempfile", "nodes": ["tempfile"]}
 ],
 "allow": [
  {"from": "change", "to": ["declaration"]},
  {"from": "validation", "to": ["change", "declaration", "jsonschema"], "description": "Validator は判定の記録と I33 に change の型を使う"},
  {"from": "render", "to": ["validation", "change", "declaration"], "description": "Resolver を描画も使う（Q5 #1）"},
  {"from": "search", "to": ["change", "declaration"]},
  {"from": "instance", "to": ["declaration"]},
  {"from": "migration-publish", "to": ["declaration"]},
  {"from": "application", "to": ["declaration", "change", "validation", "render", "search", "instance", "migration-publish", "session"]},
  {"from": "agents-contract", "to": ["application", "ai-cli"]},
  {"from": "ai-cli", "to": ["application"]},
  {"from": "sqlite", "to": ["application", "rusqlite"]},
  {"from": "sqlite", "to": ["tempfile"], "kinds": ["dev"], "description": "テストの一時ディレクトリだけ"},
  {"from": "outbound", "to": ["application"]},
  {"from": "cli", "to": ["application", "agents-contract", "ai-cli", "sqlite", "outbound"]}
 ]
}
def _line(o): return json.dumps(o, ensure_ascii=False, separators=(', ', ': '))
sas_text = '{\n  "components": [\n' + ',\n'.join('    ' + _line(c) for c in SAS['components']) + '\n  ],\n  "allow": [\n' + ',\n'.join('    ' + _line(a) for a in SAS['allow']) + '\n  ]\n}'
json.loads(sas_text)
run_out = """違反 2件（規則 semantic-agent-schema/architecture.json ・ グラフ cargo metadata --no-deps）

not-in-allowed  semantic-agent-schema-change -> semantic-agent-schema-fs  (normal)
  component: change -> outbound   allow に change -> outbound が無い
uncovered       semantic-agent-schema-sqlite -> tempfile  (normal)
  component: sqlite -> tempfile   kinds が dev だけなので、normal の辺は許されない"""
tree_rows = [
 ("dependency-rules/", 0, "リポジトリ直下。1つの crate"), ("Cargo.toml", 1, None), ("README.md", 1, "規則の書き方 ・ 終了コード"),
 ("rules.schema.json", 1, "規則のファイルの形（JSON Schema）"),
 ("src/", 1, None), ("main.rs", 2, "cli：引数 ・ 出力 ・ 終了コード"), ("core.rs", 2, "判定の中心。入出力をしない ・ 言語の語を持たない"),
 ("cargo.rs", 2, "読み取り：cargo metadata --no-deps → node と edge"), ("tests/", 1, None), ("check.rs", 2, "小さな workspace を一時ディレクトリに作って起動する"),
 ("semantic-agent-schema/", 0, "最初の利用者"), ("architecture.json", 1, "規則のファイル（利用者が場所と名前を決める）"),
]
tree = [{"path": p, "depth": d, **({"note": n} if n else {})} for p, d, n in tree_rows]
T = []
T.append({
 "id": "Q1", "name": "規則の形", "status": "open",
 "question": "依存の規則を、どの形で書くか。",
 "answer": {"text": "node の名前を component にまとめ、component の間で許す辺だけを allow に書く許可リスト型にする。書いていない辺はすべて禁止する。",
  "details": [
   "component は name と nodes（node の名前。完全一致か末尾の *）を持つ。1つの node が2つの component に当たれば規則の誤りにする",
   "allow は from と to（component の並び）を持つ。同じ component の中の辺も、書かなければ禁止する",
   "allow は任意で kinds（辺の種類の並び）を持つ。省けばすべての種類を許す。dev の辺だけ許すような外の依存を書ける",
   "外の依存も node にする。最初の利用者の「crate ごとに許す外の crate」は、外の node の component への allow で書く",
   "workspace の member と、その直接の依存が、どの component にも入らなければ違反（uncovered）にする",
   "allow と component は任意で description（理由）を持つ。表と理由を規則のファイル1か所に置ける",
   "既知の違反を一時的に許す仕組みは持たない"]},
 "images": [{"kind": "code", "name": "SAS の規則", "caption": "アーキテクチャの頁の4節の表を、この形で書き写したもの（外の crate は分かっている3つだけ）", "path": "semantic-agent-schema/architecture.json", "language": "json", "text": sas_text}],
 "assumptions": ["最初の利用者 semantic-agent-schema は、違反0件から始められる", "Cargo は書いていない依存の使用と、通常の依存の循環を止める（arch.html 2節）"],
 "checks": [{"target": "allow の kinds", "ask": "辺の種類で許し分ける欄を最初の版に入れてよいか", "view": "入れる。build 用 ・ dev 用の外の依存を本体で使えないようにする", "basis": ["arch.html 5節", "反証役"], "why": "種類を区別しない案も dev の循環は止められる"}],
 "grounds": [
  {"supports": "許可リスト型", "basis": "By default, all dependencies between layers are forbidden!", "tag": "primary", "source": "deptrac concepts:39（research.md Q91）"},
  {"supports": "許可リスト型", "basis": "A list of rules that describe dependencies that are _allowed_. dependency-cruiser will emit a 'not-in-allowed' message for each dependency that does not satisfy at least one of them.", "tag": "primary", "source": "dependency-cruiser rules:76（Q31）"},
  {"supports": "component という語", "basis": "component is abstraction on go packages. One component = one or more go packages", "tag": "primary", "source": "go-arch-lint syntax:13（Q130）"},
  {"supports": "uncovered", "basis": "You can use the `--fail-on-uncovered` option to let Deptrac fail when any uncovered dependencies are encountered.", "tag": "primary", "source": "deptrac concepts:229（Q95）"},
  {"supports": "kinds を持つ", "basis": "architecture.json：許す辺 ・ 辺の種類 ・ crate ごとに許す外の crate（検査の正本）", "tag": "rule", "source": "semantic-contract-core arch.html 5節"}],
 "rejected": [
  {"option": "層の並びだけで書く（import-linter layers ・ ArchUnit layeredArchitecture）", "reason": "SAS の domain の中は並びにならない。search → validation を許してしまう"},
  {"option": "禁止リスト型（forbidden）", "reason": "書き忘れた辺が通る。SAS は「書いていない組はすべて禁止」と決めている"},
  {"option": "パスの正規表現を規則に直接書く（dependency-cruiser）", "reason": "言語のパスの形が規則に入る"},
  {"option": "allow の component の間の循環を、規則の誤りとして断る", "reason": "同じ component の中の辺を許す書き方と両立しない。通常の循環は Cargo が止め、dev の循環は書かなければ止まる"},
  {"option": "group と呼ぶ", "reason": "既存のツールの語に無い。component は go-arch-lint が同じ意味で使う"}],
 "verification": {"passed": [{"option": "許可リスト型 ＋ 任意の kinds", "cost": "外の crate を足すたびに規則を書き足す（利用者が望むこと）"}],
  "out_of_scope": [{"item": "既知の違反の記録（baseline）", "treatment": "later", "note": "違反0件から始められない利用者が現れたら"},
   {"item": "1つの crate の中のモジュールどうしの向き", "treatment": "out", "note": "SAS は crate を単位に向きを守る"}],
  "history": ["反証役：allow の循環の検査と同じ component の中の辺が両立しない → 循環の検査を外した", "反証役 ・ ddd：arch.html 5節は辺の種類を持つ → kinds を足した", "ddd：group → component、要素 → node"]}})
T.append({
 "id": "Q2", "name": "言語との境目", "status": "waiting", "depends": ["Q1"],
 "question": "判定の中心は何を受け取り、言語ごとの読み取りとどう分けるか。",
 "answer": {"text": "判定の中心（core）は node と edge のグラフと規則だけを受け取り、言語の語を持たない。最初の版の読み取りは Rust だけで、公式の cargo metadata --no-deps を読む。",
  "details": [
   "読み取りは cargo metadata --no-deps --format-version 1 を起動し、packages の dependencies を辺にする",
   "node の名前は package の name（依存では dependencies の name。rename の名前は使わない）",
   "辺の種類は dependencies の kind（dev ・ build、null は normal）。optional の依存も feature に依らず全部入る",
   "target（プラットフォームの条件）は見ない。宣言されたすべての辺を検査する",
   "内の node は workspace_members の package。外の node は内の node の直接の依存だけ",
   "グラフの JSON を外へ公開することはしない。2つ目の言語の読み取りが現れたら、そのときに形を公開する"]},
 "images": [{"kind": "figure", "name": "境目", "caption": "core は Cargo の語を知らない。読み取りがグラフに直して渡す", "svg": svg}],
 "assumptions": ["検査したいのは「Cargo.toml に書いた辺」で、ビルドで解決された辺ではない"],
 "checks": [{"target": "言語に依らないこと", "ask": "最初の版では、グラフの形を外へ公開しなくてよいか", "view": "公開しない。core が言語の語を持たないことだけを守る", "basis": ["利用者の言葉", "足す前に引く"], "why": "あなたの言葉は「アーキテクチャに寄らず」で言語には触れていない"}],
 "grounds": [
  {"supports": "--no-deps を使う", "basis": "Output information only about the workspace members and don't fetch dependencies.", "tag": "primary", "source": "cargo metadata（Cargo の原文）"},
  {"supports": "resolve を使わない", "basis": "Inactivated optional dependencies are not listed.", "tag": "primary", "source": "cargo metadata（Cargo の原文）"},
  {"supports": "kind の読み替え", "basis": "\"dev\", \"build\", or null for a normal dependency.", "tag": "primary", "source": "cargo metadata:274（Q150）"},
  {"supports": "Package ID を名前に使わない", "basis": "fields related to \"Source ID\" are treated as opaque identifiers", "tag": "primary", "source": "cargo metadata Compatibility"},
  {"supports": "core が Cargo の語を持たない", "basis": "It is recommended to include the <code>--format-version</code> flag to future-proof your code", "tag": "primary", "source": "cargo metadata:196（Q157）"}],
 "rejected": [
  {"option": "resolve.nodes の deps を辺にする", "reason": "既定で有効でない feature の後ろの依存が出ず、検査を通る。ネットワークにも依る"},
  {"option": "core が cargo metadata を直接読む", "reason": "core に packages ・ dependencies の語が入り、2つ目の言語で core を直すことになる"},
  {"option": "グラフの JSON Schema を公開し、標準入力でも受け取る", "reason": "2つ目の言語の利用者がいない。公開した形は戻せない"}],
 "verification": {"out_of_scope": [{"item": "Rust 以外の読み取り", "treatment": "later", "note": "利用者が現れたら、cargo と同じ位置に足す"}],
  "history": ["反証役：利用者の言葉は「アーキテクチャに寄らず」→ グラフの公開を外した", "反証役：resolve は feature の後ろの依存を落とす → --no-deps と dependencies に替えた", "ddd：node の id の決め方が無い → package の name にした"]}})
T.append({
 "id": "Q3", "name": "出力と終了コード", "status": "waiting", "depends": ["Q1", "Q2"],
 "question": "違反をどう報告し、終了コードを何通りにするか。",
 "answer": {"text": "違反を1件1行の文で出し（--json で JSON）、終了コードは 0 違反なし ・ 1 違反あり ・ 2 規則かグラフが読めないの3つにする。",
  "details": [
   "違反の種類は2つ。not-in-allowed（allow に無い辺）と uncovered（どの component にも入らない node）",
   "1件は from ・ to ・ kind と、両端の component を持つ",
   "規則の誤り（知らない component を指す ・ 1つの node が2つの component に当たる ・ 形の違反）と、cargo metadata の失敗は 2",
   "一度も使われなかった allow の報告は持たない"]},
 "images": [{"kind": "run", "name": "違反の例", "caption": "SAS の change に outbound/fs への依存を足し、sqlite で tempfile を本体から使ったとき", "cwd": "semantic-agent-schema", "command": "dependency-rules check --rules architecture.json --cargo .", "output": run_out, "exit": 1}],
 "assumptions": ["CI は終了コードで止め、人と AI ツールは文を読んで直す場所を決める"],
 "grounds": [
  {"supports": "違反の名前", "basis": "dependency-cruiser will emit a 'not-in-allowed' message for each dependency that does not satisfy at least one of them.", "tag": "primary", "source": "dependency-cruiser rules:76（Q31）"},
  {"supports": "0 と 1", "basis": "Project has correct architecture", "tag": "primary", "source": "go-arch-lint README:129（Q136）"},
  {"supports": "0 ・ 1 ・ 2", "basis": "このリポジトリの道具はどれも 0 正常 ／ 1 検出あり ／ 2 誤用", "tag": "rule", "source": "各 Skill の CLI（fact-check ・ acdr ・ design-svg など）"}],
 "rejected": [
  {"option": "違反の数を終了コードにする（dependency-cruiser）", "reason": "合否だけで足りる。終了状態は下位8ビットしか親に渡らない"},
  {"option": "「違反あり」と「読めない」を同じ 1 にする", "reason": "CI で規則の書き損じと本物の違反が区別できない"},
  {"option": "使われなかった allow を報告する（cargo-deny skip ・ import-linter unmatched）", "reason": "最初の利用者が求めていない。足すのは求められてから"}],
 "verification": {"history": ["反証役：使われなかった allow の報告は外せる → 外した", "ddd：not-allowed → not-in-allowed、ungrouped → uncovered"]}})
T.append({
 "id": "Q4", "name": "置き場所と名前", "status": "waiting", "depends": ["Q2", "Q3"],
 "question": "ツールをどこに置き、どう配り、何と呼ぶか。",
 "answer": {"text": "リポジトリ直下の dependency-rules/ に1つの crate として置き、実行ファイル dependency-rules を作る。利用者は規則のファイルを持ち、CI で check を呼ぶ。",
  "details": [
   "crate の中は core ・ cargo ・ main の3つのモジュール。crate を分けない",
   "規則のファイルの場所と名前は利用者が決め、--rules で渡す。既定の名前は持たない",
   "semantic-agent-schema の CI は、同じリポジトリの dependency-rules を cargo install --path で入れて呼ぶ",
   "名前は目的（依存の規則）を表し、Rust の語も最初の利用者の名前も入れない"]},
 "images": [{"kind": "tree", "name": "フォルダ", "caption": "ツールと最初の利用者の置き場所", "rows": tree}],
 "assumptions": ["ツールと最初の利用者は同じリポジトリにある"],
 "checks": [{"target": "名前 dependency-rules", "ask": "この名前でよいか", "view": "よい。目的を表し、言語の語を含まない", "basis": ["meta-thinking", "反証役"], "why": "crates.io などとの重なりはまだ確かめていない"}],
 "grounds": [
  {"supports": "リポジトリ直下", "basis": "Aで、先にこっちを進めてください", "tag": "rule", "source": "利用者（2026-10-24）"},
  {"supports": "1つの crate", "basis": "ツールが検査するのは crate の間の辺で、1つの crate の中のモジュールの向きは測らない", "tag": "rule", "source": "Q1 の範囲の外"}],
 "rejected": [
  {"option": "core ・ 読み取り ・ 入口を別々の crate にして自分を検査する", "reason": "最初の版には過剰。自分の中のモジュールの向きは、このツールでは測れない"},
  {"option": "Skill として .claude/skills/ に置く", "reason": "特定の AI ツールの置き場所に入る。CI から呼ぶツールである"},
  {"option": "cargo のサブコマンド（cargo-xxx）として配る", "reason": "名前と呼び方が Rust に寄る"}],
 "verification": {"out_of_scope": [{"item": "外の作業：dependency-rules を作り、semantic-agent-schema の CI で architecture.json を検査する", "treatment": "later"}], "history": ["反証役：3つの crate に分けて自分を検査するのは過剰 → 1つの crate にした"]}})
patch = [
 {"op": "add", "path": "/id", "value": "dependency-rules"},
 {"op": "add", "path": "/kind", "value": "board"},
 {"op": "add", "path": "/title", "value": "依存の向きの検査ツール"},
 {"op": "add", "path": "/intro", "value": "アーキテクチャに寄らず、許していない依存の辺を検出して止めるツールの、規則の形 ・ 言語との境目 ・ 出力 ・ 置き場所を決める。"},
 {"op": "add", "path": "/topics", "value": T},
 {"op": "add", "path": "/queue", "value": [{"topic": "Q1", "why": "規則の形が決まると、グラフに持たせる属性と出力が決まる"}]},
]
open(os.path.join(H, 'patch.json'), 'w').write(json.dumps(patch, ensure_ascii=False))
r = subprocess.run([BIN, 'update', '--path', os.path.join(R, '.brainstorming/dependency-rules.json'), '--patch', json.dumps(patch, ensure_ascii=False)], capture_output=True, text=True, cwd=R)
print(r.returncode, r.stdout[:1500], r.stderr[:1500])
