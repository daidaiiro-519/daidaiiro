# 第6回：実装の言語 Rust の仕組み（workspace ・ crate ・ module ・ trait ・ feature ・ enum）を Q3 ・ Q4 に足す。
# 第5回：利用者の問いに答え直し、標準の概念を自前の概念として定義する形（Q2）と、運び方に依らない protocol と口（Q4）に直す。
# 第4回：利用者の指摘で、全体構成図（images/semantic-contract-core-overview-illustrated.png）の層に沿って論点を組み直す。
# 第3回：基盤の実物（abstract@v1 ・ 注釈 ・ エンジン）を読み直し、足りている／足りないと完成イメージを実物に合わせる。
# 第2回：基盤だけを対象に、論点を依存の順に組み直す。ボードは update で直す（手で書き換えない）。
# 根拠の引用は、組み立てるときに原文から切り出す。原文に無ければ止める。
import json, pathlib, sys

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
REPO = BOARD.parent.parent
svg = lambda name: (HERE / f"{name}.svg").read_text()


def strings(o, acc):
    if isinstance(o, str):
        acc.append(o)
    elif isinstance(o, dict):
        [strings(v, acc) for v in o.values()]
    elif isinstance(o, list):
        [strings(v, acc) for v in o]
    return acc


TEXTS = {
    "meta": strings(json.loads((BOARD / "advisors/meta.json").read_text()), []),
    "ddd": strings(json.loads((BOARD / "advisors/ddd.json").read_text()), []),
    "platform": strings(json.loads((BOARD / "advisors/platform.json").read_text()), []),
    "qa": strings(json.loads((BOARD / "advisors/qa.json").read_text()), []),
    "exp": (BOARD / "measured/v03-experiment/README.md").read_text().split("\n"),
    "src": (BOARD / "sources/README.md").read_text().split("\n"),
    "base": (REPO / ".claude/skills/schema-driven/SKILL.md").read_text().split("\n"),
    "v10": (BOARD / "sources/v0.3/docs/10_abstract-concrete-schema-boundaries.md").read_text().split("\n"),
    "v11": (BOARD / "sources/v0.3/docs/11_semantic-layered-model.md").read_text().split("\n"),
    "v15": (BOARD / "sources/v0.3/docs/15_json-tree-and-semantic-graph.md").read_text().split("\n"),
    "v17": (BOARD / "sources/v0.3/docs/17_decision-ledger-v0.3.md").read_text().split("\n"),
    "ann": strings(json.loads((REPO / ".claude/skills/schema-driven/assets/annotations.schema.json").read_text()), []),
    "ms": strings(json.loads((REPO / ".claude/skills/schema-driven/assets/meta-schema.json").read_text()), []),
    "chk": (REPO / "src/schema-driven/crates/core/src/domain/values.rs").read_text().split("\n"),
    "view": strings(json.loads((REPO / ".claude/skills/schema-driven/assets/view.schema.json").read_text()), []),
    "page": strings(json.loads((REPO / ".claude/skills/schema-driven/assets/page.schema.json").read_text()), []),
    "dddc": strings(json.loads((BOARD / "advisors/ddd-components.json").read_text()), []),
    "pfa": strings(json.loads((BOARD / "advisors/platform-adapters.json").read_text()), []),
    "ad": (BOARD / "sources/adapters/README.md").read_text().split("\n"),
    "wit": (BOARD / "sources/adapters/component-model.bytecodealliance.org_design_wit.html").read_text().split("\n"),
    "why": (BOARD / "sources/adapters/component-model.bytecodealliance.org_design_why-component-model.html").read_text().split("\n"),
    "jld": (BOARD / "sources/www.w3.org_TR_json-ld11").read_text().split("\n"),
    "book7": (BOARD / "sources/rust/doc.rust-lang.org_book_ch07-01-packages-and-crates.html").read_text().split("\n"),
    "book7b": (BOARD / "sources/rust/doc.rust-lang.org_book_ch07-02-defining-modules-to-control-scope-and-privacy.html").read_text().split("\n"),
    "book10": (BOARD / "sources/rust/doc.rust-lang.org_book_ch10-02-traits.html").read_text().split("\n"),
    "book6": (BOARD / "sources/rust/doc.rust-lang.org_book_ch06-02-match.html").read_text().split("\n"),
    "feat": (BOARD / "sources/rust/doc.rust-lang.org_cargo_reference_features.html").read_text().split("\n"),
    "ws": (BOARD / "sources/rust/doc.rust-lang.org_cargo_reference_workspaces.html").read_text().split("\n"),
    "wbg": (BOARD / "sources/rust/raw.githubusercontent.com_bytecodealliance_wit-bindgen_main_README.md").read_text().split("\n"),
    "wt": (BOARD / "sources/rust/raw.githubusercontent.com_bytecodealliance_wasmtime_main_README.md").read_text().split("\n"),
}


def q(key, start, end=None):
    for s in TEXTS[key]:
        i = s.find(start)
        if i < 0:
            continue
        if end is None:
            return s[i:].strip()
        j = s.find(end, i)
        if j >= 0:
            return s[i : j + len(end)]
    sys.exit(f"原文に無い：{key}：{start}")


SRC = {
    "meta": "meta-thinking-advisor の回答（2026-10-17）。advisors/meta.json",
    "ddd": "ddd-advisor の回答（2026-10-17）。advisors/ddd.json",
    "platform": "platform-advisor の回答（2026-10-17）。advisors/platform.json",
    "qa": "qa-advisor の回答（2026-10-17）。advisors/qa.json",
    "exp": "measured/v03-experiment/README.md（2026-10-17）",
    "base": "基盤の SKILL.md（.claude/skills/schema-driven/SKILL.md）",
    "v10": "構想 v0.3 の docs/10",
    "v11": "構想 v0.3 の docs/11",
    "v15": "構想 v0.3 の docs/15",
    "v17": "構想 v0.3 の docs/17",
    "ann": "基盤の assets/annotations.schema.json（schema-driven/annotations@v1）",
    "ms": "基盤の assets/meta-schema.json（schema-driven/abstract@v1）",
    "chk": "基盤のエンジン src/schema-driven/crates/core/src/domain/values.rs",
    "view": "基盤の assets/view.schema.json（schema-driven/view@v1）",
    "page": "基盤の assets/page.schema.json（schema-driven/page@v1）",
    "dddc": "ddd-advisor の回答（2026-10-17、構成の対応）。advisors/ddd-components.json",
    "pfa": "platform-advisor の回答（2026-10-17、技術選定）。advisors/platform-adapters.json",
    "ad": "sources/adapters/README.md（2026-10-17 取得、fact-check で照合）",
    "src": "sources/README.md（2026-10-17 取得、fact-check で照合）",
    "wit": "Component Model の WIT の文書（component-model.bytecodealliance.org/design/wit.html、2026-10-17 取得）",
    "why": "Component Model の文書（component-model.bytecodealliance.org/design/why-component-model.html、2026-10-17 取得）",
    "jld": "JSON-LD 1.1（www.w3.org/TR/json-ld11、2026-10-17 取得）",
    "book7": "The Rust Programming Language 7.1（ch07-01:186、2026-10-17 取得、fact-check で照合）",
    "book7b": "The Rust Programming Language 7.2（ch07-02:273、2026-10-17 取得、fact-check で照合）",
    "book10": "The Rust Programming Language 10.2（ch10-02:187、2026-10-17 取得、fact-check で照合）",
    "book6": "The Rust Programming Language 6.2（ch06-02:465、2026-10-17 取得、fact-check で照合）",
    "feat": "The Cargo Book の Features（features:189〜190、2026-10-17 取得、fact-check で照合）",
    "ws": "The Cargo Book の Workspaces（workspaces:189、2026-10-17 取得、fact-check で照合）",
    "wbg": "wit-bindgen の README（main、32〜34行、2026-10-17 取得、fact-check で照合）",
    "wt": "wasmtime の README（main、141行、2026-10-17 取得、fact-check で照合）",
}


def g(supports, key, start, end=None, tag="primary", source=None):
    return {"supports": supports, "basis": q(key, start, end), "tag": tag, "source": source or SRC[key]}


def user(supports, words):
    return {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}


U_FOCUS = "そもそもこの新しい概念においてSchema駆動がどのように変わるという部分を論点として扱い、何が足りないのか逆に今何が足りているのか、本当に必要なものか、このような内容を論点を進めて把握してい行きたいんです。"
U_BASE = "まずは基盤を確立しないことには決まらないです。"
U_SYN = "そしてZipの内容は私はSchema駆動にAIのAgents契約を持たせたい目的とかなりシナジーがある内容だと思ってます。"
U_AGENTS = "たぶん前提が変わるなら、少し変わるんだけども、Shema駆動がAgents契約を持つっていう契約概念は今回のZipの内容においても必要でこれは足りてない部分だと思う。"

ms_lines = (REPO / ".claude/skills/schema-driven/assets/meta-schema.json").read_text().split("\n")
rule_before = "\n".join(ms_lines[346:388])
assert ms_lines[346].strip() == '"rule": {' and ms_lines[387].strip() == "},", "abstract@v1 の rule の定義の行が変わった"
rule_after = rule_before.replace('''    "test": {
     "$ref": "#/$defs/sentence"
    },''', '''    "test": {
     "$ref": "#/$defs/sentence"
    },
    "by": {
     "description": "この決まりを確かめる担い手",
     "enum": ["engine", "check", "ai", "human"]
    },
    "requires": {
     "description": "担い手に要る能力",
     "type": "array",
     "items": { "type": "string", "minLength": 1 }
    },
    "required": {
     "description": "確かめられなければ全体を成功にしない決まりか",
     "type": "boolean"
    },''')
assert rule_after != rule_before
vocab_code = (BOARD / "sources/v0.3/examples/semantic/vocabulary.duration.example.json").read_text().rstrip()
resp_code = (BOARD / "sources/v0.3/examples/protocol/interpreter-response.example.json").read_text().rstrip()

IMG = "構想 v0.3 の全体構成図（images/semantic-contract-core-overview-illustrated.png。画像の文字を書き写した）"


def img(supports, words):
    return {"supports": supports, "basis": words, "tag": "primary", "source": IMG}


U_TECH = "そのまま使用できるかどうかの技術選定の観点の論点"
U_NEED = "今考えているのは必要になったら入れるではなくて、この構想に必要な概念かどうかを聞いていて、後回しにする判断を効いてるわけではない。"

principles_svg = svg("q0-principles")
adapters_svg = svg("q2-adapters")

topics = [
 {
  "id": "Q0", "name": "目的と設計原則", "status": "open",
  "question": "構想 v0.3 の5つの設計原則と従属させないための方針を、いまの基盤の約束と照らして、基盤は何を目的にし、どれを採るか",
  "answer": {
   "text": "基盤は、AI が書いたインスタンスを宣言した決まりで機械が確かめ、確かめられなかった決まりも隠さずに返すためにある。5つの設計原則はすべて採り、基盤から AI を呼ぶ口（Agents 契約）を足す。",
   "details": [
    "原則1 宣言と解釈の分離：採る。いまも宣言（スキーマと注釈）と確かめ（エンジン）は分かれているが、確かめる担い手を外から足す口が無い",
    "原則2 既存技術は部品として利用：採る。いまは JSON Schema をエンジンに組み込んでいる。JSON-LD ・ SHACL ・ RDF/SPARQL の概念は自前の概念として定義し、標準へは写して相互運用する",
    "原則3 言語非依存の意味モデル（実装は Rust）：採る。いまも Rust で作り、CLI ・ MCP ・ JSON で返す。出力の形を JSON Schema で公開することが足りない",
    "原則4 拡張可能な表現モデル：採る。いまも x-annotations で注釈を足せるが、足した注釈を確かめず、確かめなかったことも返さない",
    "原則4 の拡張には、外から足す解釈の担い手を閉じた環境で動かすこと（WASM Component）も含める。構想は、担い手の登録と実行の権限を管理するとしている",
    "原則5 検証可能で実用に耐える（再現性 ・ 適合性 ・ 可搬性）：採る。いまも同じ入力から同じ結果を返すが、別の言語の実装が使える適合の試験が無い",
    "従属させないための方針も採る。ただし「既存仕様をそのまま正本としない」は、いまの正本が JSON Schema であることと食い違うので、抽象の仕様の論点で扱う",
    "足すもの：基盤から AI を呼ぶ口（Agents 契約）。全体構成図の外部インターフェースは、4つとも外から基盤を呼ぶ向きである",
    "基盤に残すもの：インスタンスの作成と更新 ・ 承認の記録 ・ 描画。全体構成図は描画を基盤の外に置くが、いまの基盤は承認の前に読む画面を作るために持つ"
   ]
  },
  "images": [{"kind": "figure", "caption": "5つの設計原則と、いまの基盤で足りているもの ・ 足りないもの。", "svg": principles_svg}],
  "assumptions": ["全体構成図の5つの設計原則と従属させないための方針は、画像の文字を書き写したもので、docs には同じ一覧が無い"],
  "checks": [
   {"target": "Agents 契約", "ask": "基盤から AI を呼ぶ口を、全体構成図の外部インターフェースの逆向きとして足すことでよいか"},
   {"target": "描画", "ask": "描画を基盤に残すことでよいか（全体構成図とは置き場所が逆になる）"},
   {"target": "正本", "ask": "「既存仕様をそのまま正本としない」といまの JSON Schema の正本の食い違いを、抽象の仕様の論点で扱うことでよいか"}
  ],
  "grounds": [
   img("5つの設計原則", "設計原則：1 宣言と解釈の分離（Vocabulary と Interpreter）／2 既存技術は「部品」として利用（表現を制限しない）／3 言語非依存の意味モデル（実装は Rust を第一候補）／4 拡張可能な表現モデル（既存標準を超えた独自表現も可能）／5 検証可能で実用に耐える（再現性・適合性・可搬性）"),
   img("従属させないための方針", "従属させないための方針：既存仕様をそのまま正本としない／私たちの抽象モデルを既存仕様に合わせない／必要な部分のみマッピング・アダプタで利用／独自の拡張要素を自由に定義できる／将来の標準変更にも影響を受けない設計にする"),
   g("基盤から AI を呼ぶ口が、全体構成図に無いこと", "dddc", "無い。図の外部インターフェースは4つとも外から基盤を呼ぶ向きで", "具体の形である"),
   user("Agents 契約を基盤が持つこと", U_SYN),
   user("論点の立て方", U_FOCUS),
   g("描画の置き場所が、全体構成図といまの基盤で逆であること", "dddc", "v0.3 は Renderer を Runtime（基盤の外）に置くが、いまの基盤は描画を基盤の中に持つ。"),
   g("確かめなかったことを返さないこと", "exp", "いまの基盤は、知らない注釈を申告さえすれば", "何も報告しない（keyword-trial）。", tag="measured")
  ],
  "rejected": [
   {"option": "設計原則のうち、原則2 ・ 原則4 を採らない（JSON Schema に閉じる）", "reason": "構想の中核の原則で、利用者の要求「既存技術に表現を制限されない」と食い違う"},
   {"option": "描画を基盤の外（Runtime）へ移す", "reason": "承認の前に人が読む画面を、Skill ごとに作り直すことになる"},
   {"option": "Concrete の決まりから基盤を決める（第1回）", "reason": "ACDR などほかの Skill に当てはまるかが分からない"}
  ]
 },
 {
  "id": "Q1", "name": "抽象の仕様", "status": "waiting", "depends": ["Q0"],
  "question": "抽象の5つの仕様（Semantic Model ・ Vocabulary Spec ・ JSON Representation ・ Constraint Spec ・ Interpreter Protocol）を、いまの abstract@v1 でどこまで持ち、何を足すか",
  "answer": {
   "text": "JSON Representation は abstract@v1 で足りている。Semantic Model ・ Constraint Spec ・ Interpreter Protocol は一部あり、Vocabulary Spec は無い。足すものは abstract の新しい版に置き、abstract@v1 は変えない。",
   "details": [
    "JSON Representation：同じ。共通の JSON 形式とスキーマの記述は、abstract@v1（JSON Schema 2020-12 と注釈）が持つ",
    "Semantic Model（Entity ・ Property ・ Type）：一部ある。インスタンスの kind と id、プロパティと型がこれにあたる。語の意味が通る範囲（文脈）が無い",
    "Vocabulary Spec（識別子 ・ 意味 ・ 階層 ・ 関係）：無い。$ref で形は共有できるが、語の意味や単位を共有する仕様が無い",
    "Constraint Spec（構造 ・ 値 ・ 関係 ・ 意味制約）：一部ある。構造と値は JSON Schema、関係と数は x-ref、導出値は x-derive が持つ",
    "Constraint Spec の足りないもの：意味の決まりは x-prompt の test の文だけで、担い手 ・ 要る能力 ・ 必須かを宣言できない",
    "Interpreter Protocol（能力表明 ・ 検証インターフェース）：一部ある。結果の形（合格 ・ ずれ ・ 確かめ直し）はあるが、能力の表明と、確かめなかった決まりの報告が無い",
    "公開の仕様：JSON/JSON Schema 互換と独自拡張はある。JSON-LD/Context と SHACL 互換マッピングは、標準の概念を写す形で持つ",
    "全体構成図の表現モデル（Abstract Schema ・ Vocabulary ・ Relation ・ Constraint ・ Interpretation）はエンジンの中のモデルとし、正本はこの抽象の仕様の1か所に置く"
   ]
  },
  "images": [
   {"kind": "diff", "caption": "Constraint Spec を足す例：abstract の決まり1件に、担い手 ・ 要る能力 ・ 必須かを足す（新しい版 @v2。abstract@v1 は変えない）。", "path": ".claude/skills/schema-driven/assets/meta-schema.json（$defs/rule）", "language": "json", "before": rule_before, "after": rule_after},
   {"kind": "code", "caption": "Vocabulary Spec の例（Zip）：語に値の形 ・ 意味（量と単位）・ 要る能力を持たせる。いまの基盤には、スキーマをまたいで共有するこの形が無い。", "path": "semantic-contract-core-v0.3/examples/semantic/vocabulary.duration.example.json", "language": "json", "text": vocab_code}
  ],
  "grounds": [
   img("5つの仕様", "Abstract Schema Layer（言語非依存の仕様・契約）：Semantic Model（意味の最小単位）Entity / Property / Type 等／Vocabulary Spec（語彙仕様）識別子・意味・階層・関係／JSON Representation（共通 JSON 形式）拡張可能なスキーマ記述／Constraint Spec（制約仕様）構造・値・関係・意味制約／Interpreter Protocol（解釈プロトコル）能力表明・検証インターフェース"),
   g("書く決まりが、すでに1件ずつ id を持つこと", "ms", "書くときの決まり1件"),
   g("関係と指される数を、いまの基盤が確かめていること", "ann", "指される数に決まりがあるときは、x-ref に inverse を足す。"),
   g("表現モデルを内部のモデルとすること", "dddc", "docs/13 が「semantic-model は内部型（外部仕様ではない）」とし"),
   g("意味の判断が要る決まりが、いま検査されていないこと", "exp", "そのうち、test を確かめるのに意味の判断を必要とするものは", "書く AI が自分で読むだけである。", tag="measured")
  ],
  "rejected": [
   {"option": "abstract@v1 を書き換えて足す", "reason": "公開した版の中身が変わり、前に書いたインスタンスが別のスキーマで検査される"},
   {"option": "表現モデルと抽象の仕様を、別々に正本として持つ", "reason": "同じものに2つの名前が付き、どちらが正しいかを毎回確かめることになる"}
  ]
 },
 {
  "id": "Q2", "name": "標準技術の概念と写し", "status": "waiting", "depends": ["Q0", "Q1"],
  "question": "JSON Schema ・ JSON-LD ・ SHACL ・ RDF/SPARQL の概念のうち、構想に要るものはどれで、それを基盤の概念としてどう持つか",
  "answer": {
   "text": "4つの標準の概念はどれも構想に要るので、自前の概念として抽象の仕様に定義し、標準は正本にせず、写して相互運用する。写せなかったものは報告し、必須の意味が写せなければ合格にしない。",
   "details": [
    "そのまま正本にしない理由は2つ：既存仕様に表現を合わせないという方針と、役目の違い（JSON-LD は名前の対応付けで確かめない ・ SHACL は RDF グラフを確かめる）",
    "JSON Schema から：構造と値の制約。いまも abstract@v1 の土台で、エンジンに組み込んで確かめている",
    "JSON-LD から：共通の識別子（@id）・ 手元の名前と共通の語の対応付け（@context）・ 再定義の保護（@protected）。Vocabulary Spec と Semantic Model に定義する",
    "SHACL から：ノードと関係に置く制約（Shape）・ 参照をたどる経路（path）・ 閉じるか開くか（closed）・ 検証結果の報告の形。Constraint Spec と結果の形に定義する",
    "RDF ・ SPARQL から：インスタンスをまたいだ関係のグラフと、それへの問い合わせ。いまは参照のつながり（links ・ referrers）を持つが、問い合わせの概念が無い",
    "写し（Projection）も基盤の概念に入れる。標準へ写すたびに、写せた ・ 写せなかった ・ 扱えないを返す",
    "どのライブラリで写すか（jsonschema ・ json-ld ・ rudof ・ oxigraph）は、概念を決めたあとの実装の判断として分ける"
   ]
  },
  "images": [{"kind": "figure", "caption": "標準の概念を自前の概念として定義し、標準へは写して相互運用する。", "svg": svg("q2-concepts")}],
  "checks": [
   {"target": "自前の概念", "ask": "4つの標準から取り込む概念は、この一覧で足りているか"}
  ],
  "grounds": [
   user("概念として要るかで決めること", U_NEED),
   user("技術選定の論点を立てること", U_TECH),
   img("4つの技術の位置づけ", "標準技術アダプター（部品として利用）：JSON Schema 構造・値制約（Draft 2020-12 など）／JSON-LD 意味を持つ識別子・Context（RDF との相互運用）／SHACL 関係・制約・検証結果（RDF グラフ上の制約）／その他の技術・ライブラリ（RDF/グラフ、SPARQL など）（必要に応じて選択的に利用）"),
   img("公開仕様の互換", "公開仕様・契約（言語非依存）：JSON / JSON Schema 互換／JSON-LD / Context／SHACL 互換マッピング／独自拡張（自由な表現）"),
   g("SHACL の検証結果が必ず持つ項目", "src", "All the properties described in the remaining sub-sections of this section can be specified in a", "are the only properties that are mandatory for all validation results.", source="www.w3.org_TR_shacl:2513。sources/README.md（fact-check で照合）"),
   g("外の Context を取りに行く恐れ", "pfa", "原典は外の Context を取りに行くことを中間者攻撃と外への露出の恐れとして挙げ", "ローカルの版を持つよう勧めている。"),
   g("いまの基盤が参照のつながりを持つこと", "dddc", "一部ある。x-ref を解いて参照と指す先の組（GraphLink）と導出値（GraphDerived）を作る Graph を持ち")
  ],
  "rejected": [
   {"option": "JSON-LD や SHACL を正本にする", "reason": "既存仕様に表現を合わせることになり、JSON-LD は確かめず、SHACL は JSON をそのまま確かめない"},
   {"option": "実例が出るまで、これらの概念を持たない", "reason": "構想に要る概念を後回しにすると、抽象の仕様を後から書き換えることになる"},
   {"option": "標準ごとに別の結果の形で返す", "reason": "同じ決まりに標準ごとの結果が並び、全体の合否が決まらない"}
  ]
 },
 {
  "id": "Q3", "name": "エンジンの部品と Agents 契約", "status": "waiting", "depends": ["Q0", "Q1", "Q2"],
  "question": "エンジンの6つの部品といまのエンジンをどう対応させ、意味を確かめる担い手に AI を入れる Agents 契約は何を約束するか",
  "answer": {
   "text": "6つの部品は1つの文脈の中のモジュールとし、関係と制約は1つにまとめる。Semantic Interpreter を足し、AI はその担い手の1つとして、基盤から外へ出ていく口（Agents 契約）で呼ぶ。",
   "details": [
    "Schema Validator：同じ。いまの jsonschema と schema.rs が受け持つ",
    "Vocabulary Resolver と Contract Registry：一部ある。スキーマの名前 ・ 版 ・ 公開の記録だけを扱い、語彙の管理は無い",
    "Relation/Graph Engine と Constraint Engine：いまの check（参照 ・ 指される数 ・ 導出値）が1つで受け持つ。指される数は関係でもあり数の決まりでもあるので、分けない",
    "Semantic Interpreter：無い。決まりごとに担い手を選び、結果を5つの語（verified ・ violated ・ unverified ・ unsupported ・ error）で返す部品として足す",
    "いまの合格 ・ ずれは verified ・ violated に写し、確かめ直しは承認の状態として別に持つ",
    "AI は「決定的でない」と宣言する担い手で、決まりの id ・ 場所 ・ 引用を基盤が確かめた指摘だけを違反にし、それ以外は unverified にする",
    "Agents 契約が約束するもの：protocol の版 ・ 能力の宣言 ・ 応答の形 ・ 形を満たさない応答は error ・ 結果の出どころ ・ 書く操作は再試行しない",
    "AI の CLI を呼ぶ処理（起動 ・ 権限の型 ・ 時間切れ ・ 子孫まで止める）は6つの部品の外で、エンジンの外へ出ていく口に置く",
    "作成 ・ 更新 ・ 承認 ・ 描画は、6つの部品に無い基盤の持ち物として残す",
    "実装は Rust にする。1つの workspace にし、版と Cargo.lock を1か所で決める",
    "crate で分けるのは、破られたら困る境界である（core と adapters、取り替えたい標準の部品）。Cargo.toml に書いた crate しか使えないので、依存の向きをビルドが守る",
    "1つの部品の中は module で分ける。module は見える範囲を決めるが、同じ crate の中の module どうしの依存の向きはビルドが止めない",
    "ポート（ファイル ・ プログラム ・ AI の起動）は trait にし、adapters が実物で満たす。trait は Rust の中だけの約束なので、外の担い手との契約にはしない",
    "結果の5つの語は enum にする。match はすべての場合を尽くさないとビルドが通らないので、unverified や unsupported の扱い忘れをビルドの時点で止める"
   ]
  },
  "images": [
   {"kind": "figure", "caption": "Rust の構成：workspace ・ crate ・ module ・ trait ・ feature と、外への契約。", "svg": svg("q3-rust")},
   {"kind": "figure", "caption": "決まりを確かめる担い手と、返す結果。", "svg": svg("q2-verifiers")},
   {"kind": "code", "caption": "Interpreter の応答の例（Zip）：足りない能力を返し、合格にせず unverified にしている。いまの check の結果には、この欄が無い。", "path": "semantic-contract-core-v0.3/examples/protocol/interpreter-response.example.json", "language": "json", "text": resp_code}
  ],
  "grounds": [
   img("6つの部品", "Semantic Contract Core（Rust 実装）：Vocabulary Resolver 識別子・語彙解決／Schema Validator 構造・値の検証／Semantic Interpreter 意味の解釈・調停／Relation / Graph Engine 関係・グラフ処理／Constraint Engine 制約・検証／Contract Registry スキーマ・語彙管理"),
   g("関係と制約を分けないこと", "dddc", "区切られた文脈は同じ言葉の一貫性を保つ境界で", "いまの形のほうが正しく"),
   g("Agents 契約の置き場所", "dddc", "2つに分かれる。意味の検証（x-prompt の決まり1件ごとの判定と、その記録）は Semantic Interpreter に属する。"),
   user("Agents 契約は、v0.3 に足りない部分であること", U_AGENTS),
   g("AI の判定を合格にしないこと", "qa", "守れている → unverified（verified にしない）。", "理由を diagnostic に書く）。"),
   g("Agents 契約に足りない約束", "platform", "v0.3 の Interpreter Protocol と比べて足りないのは、", "読むだけの検証は再試行してよい）"),
   g("いまの check の結果の語", "chk", "Status::Pass => \"合格\","),
   g("crate はコンパイルの最小の単位であること", "book7", "<p>A <em>crate</em> is the smallest amount of code that the Rust compiler considers at a"),
   g("workspace で package を一緒に管理すること", "ws", "<p>A <em>workspace</em> is a collection of one or more packages, called <em>workspace"),
   g("module の中は既定で外から見えないこと", "book7b", "module is private by default. Private items are internal implementation details"),
   g("trait が型の機能を定めること", "book10", "<p>A <em>trait</em> defines the functionality a particular type has and can share with"),
   g("match はすべての場合を尽くすこと", "book6", "pattern we forgot! Matches in Rust are <em>exhaustive</em>: We must exhaust every last")
  ],
  "rejected": [
   {"option": "Relation/Graph Engine と Constraint Engine を別の部品にする", "reason": "指される数は関係でもあり数の決まりでもあり、同じ解決を2か所で持つことになる"},
   {"option": "AI の「守れている」を合格にする", "reason": "同じ入力でも答えが揺れ、合否が呼ぶたびに変わる"},
   {"option": "AI を呼ぶ処理を Semantic Interpreter の中に置く", "reason": "プロセスの起動と止め方は外へ出ていく口の仕事で、解釈と変わる理由が違う"},
   {"option": "6つの部品をそれぞれ crate に分ける", "reason": "関係と制約は同じ解決を使い、同じ言葉の中のモジュールの切り方である。crate は依存の向きを強制したい境界にだけ使う"}
  ]
 },
 {
  "id": "Q4", "name": "protocol と外部インターフェース", "status": "waiting", "depends": ["Q0", "Q1", "Q2", "Q3"],
  "question": "やりとりの中身（protocol）と、それを運ぶ口（JSON I/O ・ CLI/IPC ・ HTTP/REST ・ WASM Component）、いまの基盤からの移し方と完成の条件をどうするか",
  "answer": {
   "text": "protocol を運び方から切り離して JSON Schema で定義し、CLI ・ MCP ・ HTTP ・ WASM を同じ protocol を運ぶ口として並べる。基盤は足すだけの版で出し、4つの測り方がどれも0件なら完成とする。",
   "details": [
    "protocol：要求と応答の形 ・ 版 ・ 能力の表明 ・ 結果の5つの語を、JSON Schema で公開する。どの口から呼んでも同じ形で返す",
    "CLI（標準入出力）と MCP：いまある。出力の形が JSON Schema になっておらず、版が無く、結果の値が日本語の文字列である",
    "HTTP：外から呼ばれる口として持つ。「ネットワークに出ない」は、エンジンが自分から外へ取りに行かないという約束で、HTTP で受けることとは別である",
    "HTTP で受けるときは、受ける範囲と認証を口の側で決める。手元だけで受ける形は、brainstorming の serve と同じ守りを使える",
    "WASM Component：2つの使い道がある。エンジンをほかの言語の中で使うことと、外から足す担い手を閉じた環境で動かすことである",
    "WASM の部品は、WIT で宣言した受け取りと返しの関数でだけ外とやりとりするので、担い手に与える能力を宣言で絞れる",
    "Rust は WASM の両側に道具がある。部品を作る側は wit-bindgen（Rust を含む）、動かす側は wasmtime を crate として埋め込める",
    "標準の部品の能力は、crate の feature で選ぶ。外へ取りに行く能力（HTTP のローダーなど）は既定で外す",
    "試験は、入力と期待結果の JSON の組で持ち、どの口とどの言語の実装も同じ試験を通す",
    "基盤は足すだけの版で出す。abstract@v1 に従う Skill は、手を入れなくても今までと同じ結果を返す",
    "測り方：公開した版を書き換えないと書けない決まり ・ 結果の語へ写せないもの ・ 確かめられないのに合格と返したもの ・ 足す前と後で結果が変わったものが、どれも0件"
   ]
  },
  "images": [{"kind": "figure", "caption": "1つの protocol を、4つの口が同じ形で運ぶ。", "svg": svg("q4-protocol")}],
  "checks": [
   {"target": "HTTP", "ask": "HTTP を外から呼ばれる口として持つことでよいか（エンジンが自分から外へ取りに行かない約束は変えない）"},
   {"target": "WASM", "ask": "WASM を、ほかの言語から使う口と、外から足す担い手を閉じて動かす手段の両方として持つことでよいか"}
  ],
  "grounds": [
   user("概念として要るかで決めること", U_NEED),
   img("外部インターフェース", "外部インターフェース（言語非依存）：JSON I/O（ファイル・ストリーム）／CLI / IPC（ローカル連携）／HTTP / REST（リモート連携）／WebAssembly Component（将来対応）"),
   g("いまの外部インターフェース", "dddc", "JSON I/O と CLI は同じ（どのツールも標準出力へ JSON を1つ書く）。", "HTTP/REST と WASM Component は無い"),
   g("WIT は部品どうしの契約だけを定めること", "wit", "WIT isn’t a general-purpose programming language and doesn’t define behaviour;"),
   g("部品は宣言した関数でだけ外とやりとりすること", "why", "that interact with each other only through exports and imports of functions"),
   g("Component Model が閉じた環境の上にあること", "why", "<p>Building upon Wasm’s strong"),
   g("wit-bindgen は WASM へコンパイルされる側を作り、Rust を含むこと", "wbg", "The `wit-bindgen` repository is currently focused on **guest** programs which"),
   g("wasmtime を Rust の crate として埋め込めること", "wt", "* **[Rust]** - the [`wasmtime` crate]"),
   g("feature が任意の依存と条件付きのコンパイルを表すこと", "feat", "A package defines a set of"),
   g("言語に依存しない口にするための、いまとの差", "platform", "いまの基盤は、CLI と MCP の両方で同じ名前と引数を受け"),
   g("測り方の基準", "qa", "v0.3 の要求を、いまの基盤が公開した版を書き換えずに受け止められるかで決める。", "決める。")
  ],
  "rejected": [
   {"option": "HTTP を持たない（ネットワークに出ない約束を理由に）", "reason": "約束はエンジンが外へ取りに行かないことで、外から受ける口の話ではない。ほかの言語 ・ プロセスから使う目的に届かない"},
   {"option": "口ごとに別の形で返す", "reason": "同じ問いに口ごとの答えが並び、別の言語の実装が同じ試験を使えない"},
   {"option": "Rust の trait ・ 型を、外の担い手との契約にする", "reason": "別の言語の実装が同じ契約に従えなくなる"}
  ]
 }
]

patch = [
 {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "実装の言語 Rust の仕組み（workspace ・ crate ・ module ・ trait ・ feature ・ enum）を Q3 ・ Q4 に足した。CODING.md の土台にする"}},
 {"op": "replace", "path": "/intro", "value": "構想 v0.3 の全体構成図の層に沿って、基盤 schema-driven で何が変わり、何が足りていて、何が足りず、何が本当に必要かを、依存の順に決める。"},
 {"op": "replace", "path": "/queue", "value": [{"topic": "Q0", "why": "抽象の仕様 ・ 技術選定 ・ エンジン ・ 口は、どれも基盤の目的と設計原則で決まるため"}]},
 {"op": "replace", "path": "/topics", "value": topics},
]
print(json.dumps(patch, ensure_ascii=False))
