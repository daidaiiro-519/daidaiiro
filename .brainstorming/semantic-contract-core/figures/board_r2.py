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
}


def g(supports, key, start, end=None, tag="primary", source=None):
    return {"supports": supports, "basis": q(key, start, end), "tag": tag, "source": source or SRC[key]}


def user(supports, words):
    return {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}


U_FOCUS = "そもそもこの新しい概念においてSchema駆動がどのように変わるという部分を論点として扱い、何が足りないのか逆に今何が足りているのか、本当に必要なものか、このような内容を論点を進めて把握してい行きたいんです。"
U_BASE = "まずは基盤を確立しないことには決まらないです。"
U_SYN = "そしてZipの内容は私はSchema駆動にAIのAgents契約を持たせたい目的とかなりシナジーがある内容だと思ってます。"
U_AGENTS = "たぶん前提が変わるなら、少し変わるんだけども、Shema駆動がAgents契約を持つっていう契約概念は今回のZipの内容においても必要でこれは足りてない部分だと思う。"

schema_lines = (REPO / ".claude/skills/brainstorming/assets/board.schema.json").read_text().split("\n")
rule_code = "\n".join(schema_lines[509:530])

topics = [
 {
  "id": "Q0", "name": "基盤の目的と境界", "status": "open",
  "question": "構想 v0.3 を受けたとき、基盤 schema-driven は何のためにあり、何を持ち、何を持たないか",
  "answer": {
   "text": "基盤は、AI が書いたインスタンスを宣言した決まりで機械が確かめ、確かめられなかった決まりも隠さずに返すためにある。宣言の契約 ・ 確かめる仕組み ・ AI を呼ぶ契約を持ち、使う側のデータと動かす側は持たない。",
   "details": [
    "変わること：目的に「確かめられなかった決まりを隠さず返す」と「AI を確かめる担い手として呼ぶ」を足す",
    "足りているもの：JSON Schema で形を持つこと ・ 書いたその場の検証 ・ 参照と導出値の検査 ・ 承認の記録 ・ 公開した版を書き換えないこと ・ 言語に依存しない CLI と MCP",
    "足りないもの：決まりごとの結果 ・ 確かめなかった決まりの報告 ・ Agents 契約（AI を呼ぶ約束）の3つである",
    "Agents 契約を基盤が持つのは、確かめる担い手としての AI と、インスタンスを直す会話の相手としての AI を、どの Skill でも同じ約束で呼ぶためである",
    "持たないもの：使う側のデータ ・ UI や Workflow を動かす側 ・ コードの生成と技術の選び方 ・ JSON-LD ・ RDF ・ SHACL への写し",
    "描画（render）は持ち続ける。承認の前に人が読む画面を作るためで、UI を動かす側とは別である",
    "変わらないこと：基盤は何のデータを持つかを決めず、使う側の Skill が concrete スキーマで決める"
   ]
  },
  "images": [{"kind": "figure", "caption": "基盤が持つものと持たないもの。いま足りているものと、足りないもの。", "svg": svg("q0-boundary")}],
  "assumptions": ["Concrete の Skill（brainstorming ・ ACDR）は、この基盤が決着したあとに、別のボードで移し方を決める"],
  "checks": [
   {"target": "基盤が持つもの", "ask": "Agents 契約を、基盤が持つものとして足すことでよいか"},
   {"target": "持たないもの", "ask": "UI ・ Workflow を動かす側と、コードの生成を、基盤の外に置くことでよいか"},
   {"target": "描画", "ask": "描画（render）を、承認の画面を作るものとして基盤に残すことでよいか"}
  ],
  "grounds": [
   user("論点の立て方（変わること ・ 足りているもの ・ 足りないもの ・ 本当に必要か）", U_FOCUS),
   user("基盤を先に決め、Concrete をそのあとにすること", U_BASE),
   user("Agents 契約を基盤が持つものに入れること", U_SYN),
   g("v0.3 の基盤の定義", "v10", "**Schema駆動基盤（Abstract Schema / Semantic Contract Core）**"),
   g("動かす側を基盤の外に置くこと", "v10", "UI RendererやWorkflow Engineは、それぞれのSchemaを利用する**Runtime**であり、Schemaそのものとは区別する。"),
   g("いまの基盤が何のデータを持つかを決めないこと", "base", "この Skill は**基盤**である。何のデータを持つかは決めない。"),
   g("確かめなかった決まりを報告しないことが、いま足りないこと", "exp", "いまの基盤は、知らない注釈を申告さえすれば", "何も報告しない（keyword-trial）。", tag="measured"),
   g("目的を手段（v0.3）から始めないこと", "meta", "Q0 を「いまの基盤で見逃している誤りは何か」という問題発見の問いに戻し", "問いに戻し")
  ],
  "rejected": [
   {"option": "基盤の目的を、v0.3 で何ができるようになるかから決める（第1回の Q0 の前）", "reason": "手段から問いを始めるので、答えが UI ・ Workflow ・ コードの生成まで広がり、決められない"},
   {"option": "brainstorming のボードの決まりから入る（第1回の Q0）", "reason": "Concrete の1つから基盤を決めることになり、ACDR などほかの Skill に当てはまるかが分からない"},
   {"option": "描画を基盤から外し、使う側の Runtime に置く", "reason": "承認の前に人が読む画面を、Skill ごとに作り直すことになる"}
  ],
  "verification": {"out_of_scope": [{"item": "Concrete の Skill（brainstorming ・ ACDR ・ skills-creator）の移し方は、基盤が決着したあとの別のボードで決める", "treatment": "later"}]}
 },
 {
  "id": "Q1", "name": "意味と最小単位", "status": "waiting", "depends": ["Q0"],
  "question": "基盤が扱う「意味」は何か。宣言の最小単位は何にするか",
  "answer": {
   "text": "意味は、インスタンスに課す決まりと、それを誰がどう確かめるかとして扱う。宣言の単位は名前 ・ 識別 ・ 関係 ・ 決まり ・ 文脈の5つにし、解釈は宣言から外す。",
   "details": [
    "v0.3 の4つの側面のうち、基盤が宣言するのは構造と形式で、説明（x-prompt の文）は AI が確かめる決まりとして扱う",
    "足りているもの：構造（JSON Schema）・ 関係（x-ref）・ 導出値（x-derive）・ 名前（$id）・ 説明（description と x-prompt）",
    "足りないもの：決まり1件ごとの id ・ 語の意味が通る範囲（文脈）・ x-prompt の rule を確かめられる決まりとして持つ形",
    "解釈（Interpretation）は宣言の単位にせず、確かめる担い手の側に置く。宣言の側には、どの担い手が要るかだけを置く",
    "本当に必要か：振る舞い（状態の移り変わり ・ 時間）の宣言は、いまの Skill の決まりに実例が無いので、まだ持たない"
   ]
  },
  "images": [{"kind": "code", "caption": "文で書いただけで確かめていない決まりの実例（ボードの論点の件数）。", "path": ".claude/skills/brainstorming/assets/board.schema.json", "language": "json", "text": rule_code, "highlight": [10, 18], "note": "x-prompt は「3〜5件」と書くが、形は minItems 1 だけで、件数を確かめていない"}],
  "grounds": [
   g("v0.3 の4つの側面", "v11", "| Descriptive | 名前・説明・意図 |"),
   g("最小単位に文脈を足し、解釈を宣言から外すこと", "ddd", "最小単位の候補に「文脈」を足し、Interpretation を宣言の単位から外す案を反証にかける"),
   g("意味の判断が要る決まりが、いま検査されていないこと", "exp", "そのうち、test を確かめるのに意味の判断を必要とするものは", "書く AI が自分で読むだけである。", tag="measured"),
   g("v0.3 で未決の1つ目と2つ目", "v17", "1. **Semanticの形式的な定義**")
  ],
  "rejected": [
   {"option": "状態の移り変わりや時間の決まりも、最初から宣言できるようにする", "reason": "いまの Skill の決まりに実例が無く、宣言できても確かめる担い手が無いまま増える"},
   {"option": "解釈（Interpretation）を宣言の単位に入れる", "reason": "宣言と解釈を分けるという v0.3 自身の決定と食い違う"}
  ]
 },
 {
  "id": "Q2", "name": "抽象と具体の関係と正本の形", "status": "waiting", "depends": ["Q0", "Q1"],
  "question": "abstract と concrete の関係、語彙の版と互換、正本の形をどうするか",
  "answer": {
   "text": "関係はいまの「concrete は abstract のインスタンス」のままにし、独自の語彙は abstract の新しい版として足し、正本は JSON Schema とその語彙のままにする。",
   "details": [
    "足りているもの：concrete は abstract のインスタンスである。公開した版は書き換えず、@v2 で足す。公開した版の指紋を published.json に残す",
    "これらは、v0.3 が未決とした「Abstract と Concrete の関係」と「版と互換」に、いまの基盤がすでに答えている",
    "変わること：独自の語彙を $vocabulary で必須として宣言し、その語彙を知らない実装には処理させない",
    "足りないもの：語彙の名前が衝突したときの扱い。JSON Schema には名前空間の仕組みが無い",
    "本当に必要か：自前のモデルを正本にすることは、いまは要らない。JSON Schema で持てない意味が実例で出たときだけ見直す",
    "本当に必要か：JSON とグラフを並べて持つことは、いまは要らない。関係は x-ref で持てている"
   ]
  },
  "images": [{"kind": "figure", "caption": "抽象と具体の関係と、版の足し方。", "svg": svg("q2-versions")}],
  "grounds": [
   g("未決に、いまの基盤がすでに答えていること", "ddd", "いまの基盤は「concrete スキーマは abstract スキーマのインスタンスである」と決めており", "で答えている"),
   g("必須の語彙を知らない実装に処理させない仕組み", "src", "If the value is true, then implementations that do not recognize the vocabulary MUST refuse to process", "with \"$schema\".", source="draft-bhutton-json-schema-01.txt:1421。sources/README.md（fact-check で照合）"),
   g("語彙の衝突の扱いが決まっていないこと", "src", "JSON Schema does not provide any formal namespacing system", "namespacing approaches.", source="draft-bhutton-json-schema-01.txt:3827。sources/README.md（fact-check で照合）"),
   g("表せる範囲に差が無いこと", "exp", "JSON Schema 2020-12 の独自のキーワードで書く場合と", "差が出た決まりは0件**だった。", tag="measured"),
   g("グラフは派生にとどめること（v0.3 の仮説と同じ向き）", "v15", "**外部交換はJSON、元JSONを保持、意味解決後のGraphは必要に応じた派生表現。**")
  ],
  "rejected": [
   {"option": "自前のモデルを正本にし、JSON Schema を部品に下げる", "reason": "表せる範囲の差は0件で、3つの Skill の公開した版とインスタンスがすべて移行の対象になる"},
   {"option": "JSON とグラフを並べて持つ", "reason": "関係はいまの x-ref で持てていて、グラフを持つ実例が無い"}
  ]
 },
 {
  "id": "Q3", "name": "解釈と結果、Agents 契約", "status": "waiting", "depends": ["Q0", "Q1", "Q2"],
  "question": "誰が決まりを確かめ、何を結果として返し、AI を担い手にするとき Agents 契約は何を約束するか",
  "answer": {
   "text": "担い手と要る能力を決まりごとに宣言し、結果は5つの語で返し、必須の決まりを確かめられなければ成功にしない。AI は Agents 契約で呼ぶ担い手で、機械で裏付けた違反しか出さない。",
   "details": [
    "足りているもの：check と approve は、形の誤りがあれば止まる",
    "足りないもの：形の外の決まりを、どの担い手がどう確かめたかを返す仕組み。実験では、形の外の138件のうち62件がどこでも検査されていなかった",
    "結果の語は5つにそろえる。合格（verified）・ 違反（violated）・ 確かめていない（unverified）・ 担い手が無い（unsupported）・ 処理の失敗（error）",
    "担い手が無い決まりは unsupported、担い手はいるが合否を決められなかった決まりは unverified である",
    "AI は「決定的でない」と宣言する担い手である。決まりの id ・ 場所 ・ 引用を基盤が機械で確かめた指摘だけを違反にし、それ以外は unverified にする",
    "AI の答えの揺れは、同じ入力を N 回呼んだ判定ごとの回数として記録に残し、CI と承認の合否には使わない",
    "Agents 契約が約束するもの：protocol の版 ・ 能力の宣言 ・ 応答の形 ・ 形を満たさない応答は error ・ 結果の出どころ ・ 書く操作は再試行しない",
    "Agents 契約の使い道は、意味の確かめ（読むだけ）と会話（インスタンスを直す）の2つで、やらせたいことの型 ・ 親と子のセッション ・ 子は1つ ・ ロックを共通に持つ",
    "人の判断（目的に合うかの妥当性確認）は5つの結果の外に置き、いまの approve の記録として持つ"
   ]
  },
  "images": [{"kind": "figure", "caption": "決まりを確かめる担い手と、返す結果。", "svg": svg("q2-verifiers")}],
  "grounds": [
   user("Agents 契約は、v0.3 に足りない部分であること", U_AGENTS),
   g("v0.3 に、答えの揺れる AI を担い手にする約束が無いこと", "ddd", "利用者の「足りていない」という指摘は"),
   g("AI の判定を合格にしないこと", "qa", "守れている → unverified（verified にしない）。", "理由を diagnostic に書く）。"),
   g("Agents 契約に足りない約束", "platform", "v0.3 の Interpreter Protocol と比べて足りないのは、", "読むだけの検証は再試行してよい）"),
   g("形の外の決まりの多くが検査されていないこと", "exp", "形以外の138件のうち", "13件である。", tag="measured"),
   g("Agents 契約を、確かめる担い手の論点に置くこと", "meta", "Agents 契約は「意味の決まりを、誰が", "同じ問いを2か所で答えることになる。")
  ],
  "rejected": [
   {"option": "Agents 契約を独立の論点にする", "reason": "「誰が意味を確かめるか」と同じ問いに2か所で答えることになる"},
   {"option": "AI の「守れている」を合格にする", "reason": "同じ入力でも答えが揺れ、合否が呼ぶたびに変わる"},
   {"option": "結果の語を v0.3 の docs/04 の4つにする", "reason": "処理の失敗と確かめていないが同じ語になり、時間切れが「確かめていない」として残る"}
  ]
 },
 {
  "id": "Q4", "name": "公開の口 ・ 移し方 ・ 完成の条件", "status": "waiting", "depends": ["Q0", "Q1", "Q2", "Q3"],
  "question": "言語に依存しない口と実装を、いまの基盤からどう移し、何で確かめたら完成とするか",
  "answer": {
   "text": "口はいまの CLI ・ MCP ・ JSON のまま、出力の形と結果の語を JSON Schema で公開し、適合の試験を JSON の組で持ち、基盤を足すだけの版で出す。4つの測り方がどれも0件なら完成とする。",
   "details": [
    "足りているもの：CLI と MCP が同じ名前と引数を受け、標準出力の JSON と終了コードで返す。中心は入出力に触らない",
    "足りないもの：各ツールの出力の形が JSON Schema になっていない ・ 出力に protocol の版が無い ・ 結果の値が日本語の文字列である",
    "足りないもの：試験が Rust の中にだけあり、別の言語の実装が同じ試験を使えない。入力と期待結果の JSON の組で持つ",
    "実装は Rust のままにし、Rust の型を公開の契約にしない",
    "基盤は足すだけの版で出す。abstract@v1 に従う Skill は、手を入れなくても今までと同じ結果を返す",
    "測り方：公開した版を書き換えないと書けない決まり ・ 結果の語へ写せないもの ・ 確かめられないのに合格と返したもの ・ 足す前と後で結果が変わったものが、どれも0件",
    "保留にしたボード agents-in-base の答えは Q3 の材料として使い、2つの契約を並べない"
   ]
  },
  "grounds": [
   g("言語に依存しない口にするための、いまとの差", "platform", "いまの基盤は、CLI と MCP の両方で同じ名前と引数を受け"),
   g("測り方の基準", "qa", "v0.3 の要求を、いまの基盤が公開した版を書き換えずに受け止められるかで決める。", "決める。"),
   g("足すだけの版で移すこと", "ddd", "(2) 公開された言葉への影響", "段階的に移す。"),
   g("agents-in-base と契約を並べないこと", "platform", "Q1 の5項目の契約と Q4 の約束の分け方は", "別の言葉で決めようとしている疑いがある。")
  ],
  "rejected": [
   {"option": "Rust の trait ・ 型を、外の Interpreter との契約にする", "reason": "別の言語の実装が同じ契約に従えなくなる"},
   {"option": "agents-in-base を再開して並べて進める", "reason": "AI を呼ぶ契約が2つのボードで別の言葉で決まり、どちらが正本か分からなくなる"}
  ]
 }
]

patch = [
 {"op": "replace", "path": "/title", "value": "基盤 schema-driven はどう変わるか（構想 v0.3 と Agents 契約）"},
 {"op": "replace", "path": "/intro", "value": "構想 v0.3 を受けて、基盤 schema-driven で何が変わり、何が足りていて、何が足りず、何が本当に必要かを、依存の順に決める。"},
 {"op": "replace", "path": "/queue", "value": [{"topic": "Q0", "why": "意味 ・ 正本の形 ・ 結果 ・ 移し方は、どれも基盤が何のためにあり何を持つかで決まるため"}]},
 {"op": "replace", "path": "/topics", "value": topics},
]
print(json.dumps(patch, ensure_ascii=False))
