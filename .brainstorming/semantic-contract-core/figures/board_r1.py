# 第1回の本体を JSON Patch にして出す。ボードは update で直す（手で書き換えない）。
import json, pathlib

HERE = pathlib.Path(__file__).parent
svg = lambda name: (HERE / f"{name}.svg").read_text()
code = "\n".join((HERE.parent / "measured/v03-experiment/v03/interpreter-response.trial.json").read_text().splitlines()[:45])

ADV = lambda n: f"{n} の回答（2026-10-17）。advisors/{n.split('-')[0] if n != 'meta-thinking-advisor' else 'meta'}.json"
EXP = "measured/v03-experiment/README.md（2026-10-17）"
SRC = "sources/README.md（2026-10-17 取得、fact-check で照合）"

topics = [
 {
  "id": "Q0", "name": "見逃している誤り", "status": "open",
  "question": "いまの基盤で、check と inspect を通ったのに、承認の前に人が読んで見つけている誤りは何か。それを承認の前に、誰が見つけるようにするか",
  "answer": {
   "text": "形の外にある決まりの多くは誰も検査しておらず、破れたまま承認の前まで届いている。決まりごとに、誰が確かめて結果がどうだったかを、承認の前に返すようにする。",
   "details": [
    "ボードの決まりは、出どころごとに数えて397件ある。形（JSON Schema）が259件、形の外が138件である",
    "形の外の138件のうち62件は、どこでも検査されていない。機械で見られるもの37件、意味の判断が要るもの12件、インスタンスの外の事実13件である",
    "実物の2枚のボードは論点が6件で「3〜5件」の決まりを外れ、2文の caption も3件あった。check と inspect の指摘はどちらも0件だった",
    "いまの基盤は、独自の注釈に書いた決まりを検査せず、検査しなかったことも報告しない。見つけるのは、承認の前に読む人だけである",
    "見つける担い手は4つに分ける。エンジン（形）、決まりごとの検査（Rust など）、AI（意味の判断）、人（承認）である",
    "確かめなかった決まりは、合格ではなく「確かめていない」として返す",
    "変わらないこと：承認するのは人で、check ・ approve ・ render の合否は AI を呼ばずに決まる"
   ]
  },
  "images": [{"kind": "figure", "caption": "形の外の決まり138件と、だれが確かめているか。", "svg": svg("q0-unchecked")}],
  "assumptions": ["数えたのは brainstorming のボードのスキーマ1つと、実物のボード2枚である。ACDR と skills-creator のスキーマは数えていない"],
  "checks": [
   {"target": "見つけたい誤り", "ask": "承認の前に見つけたいのは、この「形の外の決まりの破れ」で合っているか。ほかに見逃している誤りはあるか"},
   {"target": "担い手の4つ", "ask": "エンジン ・ 決まりごとの検査 ・ AI ・ 人の4つに分けることでよいか"}
  ],
  "grounds": [
   {"supports": "形の外の138件と、検査されていない62件", "basis": "形以外の138件のうち62件はどこでも検査されていない（機械で見られるもの37、意味の判断が必要なもの12、インスタンスの外の事実に関わるもの13）", "tag": "measured", "source": EXP},
   {"supports": "実物のボードで破れていたこと", "basis": "2枚のボードとも論点が6件で、x-prompt の「3〜5件」を外れている。agents-in-base には2文の caption が3件ある。inspect と check の指摘はどちらも0件", "tag": "measured", "source": EXP},
   {"supports": "検査しなかったことを報告しないこと", "basis": "独自の注釈を x-annotations に申告すれば check-schemas を通るが、その注釈は検査せず、何も報告しない（40字の決まりを書いたのに、50字の題で指摘0件）", "tag": "measured", "source": EXP + "。keyword-trial/"},
   {"supports": "問いを v0.3 からではなく、見逃している誤りから始めること", "basis": "Q0 を「いまの基盤が見逃している誤りは何か」という問題発見の問いに戻す", "tag": "primary", "source": "meta-thinking-advisor の回答（2026-10-17）。advisors/meta.json"}
  ],
  "rejected": [
   {"option": "v0.3 で何ができるようになるかを問う（最初に立てた Q0）", "reason": "手段から問いを始めるので、答えが UI ・ Workflow ・ コードの生成まで広がり、決められない"},
   {"option": "検査されていない決まりを、全部 inspect に足す", "reason": "意味の判断12件とインスタンスの外13件は Rust の検査では見られず、検査しなかったことも報告されないまま残る"}
  ],
  "verification": {"out_of_scope": [{"item": "ACDR と skills-creator のスキーマの決まりの数", "treatment": "later"}]}
 },
 {
  "id": "Q1", "name": "位置づけと取り込む範囲", "status": "waiting", "depends": ["Q0"],
  "question": "構想 Semantic Contract Core v0.3 を、いまの基盤の作り直しにするか、足すか。足すなら、どの考えを取り込み、どれを扱わないか",
  "answer": {
   "text": "基盤は作り直さず、abstract@v1 を変えずに足す。v0.3 から取り込むのは決まりごとの結果 ・ 担い手の宣言 ・ 確かめられなければ成功にしないことの3つで、正本は JSON Schema のままにする。",
   "details": [
    "足す場所は基盤の新しい版（@v2）である。いまの操作と、abstract@v1 に従うインスタンスは、そのまま同じ結果を返す",
    "取り込む1：決まり1件ごとに id を持たせ、結果を決まった5つの語で返す",
    "取り込む2：決まりごとに、確かめる担い手と、その担い手に要る能力を宣言する",
    "取り込む3：必須の決まりを確かめられなければ、全体を成功にしない",
    "正本は JSON Schema と、その上に足す独自の語彙のままにする。試作で、それでは持てない意味が実例で出たときだけ見直す",
    "扱わないもの：UI ・ Workflow などの Runtime、JSON-LD ・ RDF ・ SHACL への写し、自前のモデルを正本にすること",
    "名前をそろえる：抽象の側は「abstract スキーマ」1つにし、Foundation などの別名を使わない"
   ]
  },
  "images": [{"kind": "figure", "caption": "基盤に足すものと、扱わないもの。", "svg": svg("q1-layers")}],
  "assumptions": ["@v2 を並べても、公開した abstract@v1 を書き換えずに、基盤の版は足すだけで出せる"],
  "checks": [
   {"target": "扱わないもの", "ask": "UI ・ Workflow の Runtime と、JSON-LD ・ SHACL への写しを、このボードで扱わないことでよいか"}
  ],
  "grounds": [
   {"supports": "表せる範囲では作り直す理由が無いこと", "basis": "表せる範囲に差が出た決まりは 0件", "tag": "measured", "source": EXP + "。タスク4（独自のキーワードと v0.3 の自前のモデルの比較）"},
   {"supports": "v0.3 で増えるものが3つに絞れること", "basis": "v0.3 で増えるものは、表せる範囲ではなく「検査しなかった決まりを unverified と報告すること」", "tag": "measured", "source": EXP},
   {"supports": "確かめられなければ成功にしない仕組みが、JSON Schema にもともとあること", "basis": "If the value is true, then implementations that do not recognize the vocabulary MUST refuse to process any schemas that declare this meta-schema with \"$schema\".", "tag": "primary", "source": "draft-bhutton-json-schema-01.txt:1421。" + SRC},
   {"supports": "決まりごとの報告は、JSON Schema の外（基盤の応答）に持つこと", "basis": "Complex behavior is usually better delegated to applications on the basis of annotation data than implemented directly as schema keywords.", "tag": "primary", "source": "draft-bhutton-json-schema-01.txt:747。" + SRC},
   {"supports": "作り直すより足すこと", "basis": "いまの操作と abstract@v1 を変えずに層を足すほうが、移行と配布のリスクは小さい", "tag": "primary", "source": "platform-advisor の回答（2026-10-17）。advisors/platform.json"},
   {"supports": "@v2 に並べて足すこと", "basis": "案B「上に足す」：abstract@v1 はそのままにし、@v2 に Vocabulary と Capability を並べて足す", "tag": "primary", "source": "ddd-advisor の回答（2026-10-17）。advisors/ddd.json"}
  ],
  "rejected": [
   {"option": "作り直す（v0.3 の自前のモデルを正本にし、基盤を置き換える）", "reason": "表せる範囲の差は0件で、3つの Skill の公開した版とインスタンスがすべて移行の対象になる"},
   {"option": "横に足す（意味の検証を別の基盤にする）", "reason": "結果の語が check と別に並び、1つの決まりに2つの結果が出る"},
   {"option": "契約だけを基盤に置き、確かめる処理は Skill に残す", "reason": "確かめられないときに成功にしないことを、基盤の側で守らせられない"}
  ],
  "verification": {"out_of_scope": [{"item": "自前のモデルを正本にするかは、試作で実例が出たときに見直す", "treatment": "later"}]}
 },
 {
  "id": "Q2", "name": "決まりを確かめる担い手（AI を含む）", "status": "waiting", "depends": ["Q0", "Q1"],
  "question": "意味の決まりを、エンジン ・ AI ・ 人のどれが確かめ、AI を担い手にするときの Agents 契約は何を約束するか",
  "answer": {
   "text": "担い手は決まりごとに宣言し、AI は担い手の1つとして Agents 契約で呼ぶ。AI の答えは揺れるので、機械で裏付けた違反だけを違反とし、それ以外は確かめていないとして返す。",
   "details": [
    "結果の語は5つにそろえる。合格（verified）・ 違反（violated）・ 確かめていない（unverified）・ 担い手が無い（unsupported）・ 処理の失敗（error）",
    "担い手が無い決まりは unsupported、担い手はいるが合否を決められなかった決まりは unverified である",
    "必須の決まりに合格以外が1件でもあれば、全体を成功にしない",
    "AI は「決定的でない」と宣言する担い手である。決まりの id ・ 場所 ・ 引用を基盤が機械で確かめた指摘だけを違反にする",
    "AI の答えの揺れは、同じ入力を N 回呼んだ判定ごとの回数として記録に残し、CI と承認の合否には使わない",
    "Agents 契約が約束するもの：protocol の版 ・ 能力の宣言 ・ 応答の形 ・ 形を満たさない応答は error ・ 結果の出どころ ・ 書く操作は再試行しない",
    "結果の出どころには、CLI の名前と版 ・ モデル ・ 親と子のセッションの ID を残す",
    "Agents 契約の使い道は3つである。意味の確かめ（読むだけ）、会話（インスタンスを直す）、Skill のスキーマが担い手を宣言することである",
    "やらせたいことの型 ・ 親と子のセッション ・ 1つのインスタンスで子は1つ ・ ロックは、AI の呼び出しのモジュールにまとめる",
    "人の判断（目的に合うかの妥当性確認）は5つの結果の外に置き、いまの approve の記録として持つ"
   ]
  },
  "images": [{"kind": "figure", "caption": "決まりを確かめる担い手と、返す結果。", "svg": svg("q2-verifiers")}],
  "checks": [
   {"target": "Agents 契約の約束", "ask": "Agents 契約に足りないと考えていたものは、この6つの約束と3つの使い道で足りているか"}
  ],
  "grounds": [
   {"supports": "Agents 契約を、基盤が持つ契約として足すこと", "basis": "たぶん前提が変わるなら、少し変わるんだけども、Shema駆動がAgents契約を持つっていう契約概念は今回のZipの内容においても必要でこれは足りてない部分だと思う。", "tag": "primary", "source": "利用者の言葉（2026-10-17）"},
   {"supports": "v0.3 に AI を判断する役が無いこと", "basis": "v0.3 の Interpreter は結果の決まった実装だけを想定しており、同じ入力で結果が揺れる AI を Interpreter にするときの約束を持たない", "tag": "primary", "source": "ddd-advisor の回答（2026-10-17）。advisors/ddd.json"},
   {"supports": "結果の語を5つにし、AI の判定を合格にしないこと", "basis": "「守れている」と「判断できない」→ unverified。AI の判定は verified にしない", "tag": "primary", "source": "qa-advisor の回答（2026-10-17）。advisors/qa.json"},
   {"supports": "Agents 契約に足りない6つの約束", "basis": "protocol の版 ・ Capability の宣言 ・ 構造化した診断を返す応答の形 ・ 形を満たさない応答を error とする決まり ・ 結果の出所 ・ 書く操作は再試行しない決まり", "tag": "primary", "source": "platform-advisor の回答（2026-10-17）。advisors/platform.json"},
   {"supports": "Agents 契約を独立の論点にせず、この論点に置くこと", "basis": "「誰が意味を確かめるか」への答えの1つなので、独立させると同じ問いを2か所で答えることになる", "tag": "primary", "source": "meta-thinking-advisor の回答（2026-10-17）。advisors/meta.json"},
   {"supports": "やらせたいことの型 ・ 親と子のセッション ・ 子は1つ ・ ロック", "basis": "1つのインスタンスで動く子セッションは1つだけにする。ロックはファイルで持ち、子へ依頼の ID を環境変数で渡し、子が呼ぶ入口は同じロックの下で書く", "tag": "rule", "source": "ボード agents-in-base の Q1（保留中）と、brainstorming 0.1.4 の CODING.md"}
  ],
  "rejected": [
   {"option": "Agents 契約を独立の論点にする", "reason": "「誰が意味を確かめるか」と同じ問いに2か所で答えることになる"},
   {"option": "AI の「守れている」を合格にする", "reason": "同じ入力でも答えが揺れ、合否が呼ぶたびに変わる"},
   {"option": "結果の語を v0.3 の docs/04 の4つにする", "reason": "処理の失敗と確かめていないが同じ語になり、時間切れが「確かめていない」として残る"}
  ]
 },
 {
  "id": "Q3", "name": "最初の試作と完成の条件", "status": "waiting", "depends": ["Q1", "Q2"],
  "question": "何を試作し、何で確かめたら、このボードの答えが正しかったと言えるか",
  "answer": {
   "text": "brainstorming のボードの決まり116件を対象に、決まりごとの結果を返す試作を abstract@v1 を変えずに作り、4つの測り方がどれも0件なら完成とする。",
   "details": [
    "測り方1：公開した版を書き換えないと書けない決まりが0件",
    "測り方2：いまの check と意味の検証の結果を5つの語へ写して、写せないものが0件",
    "測り方3：確かめられない決まりを含むのに、合格と返した件数が0件",
    "測り方4：足す前と後で、いまの全部のインスタンスの check と approve の結果が変わった件数が0件",
    "段A（CI）：見本の期待結果と100%一致する。期待結果は試作を書く前に JSON で書く",
    "段B（CI）：無い id や、インスタンスに無い引用を混ぜた AI の答えは、全件 error になる",
    "段C（手元）：決まり10件の bad と good を Claude Code ・ Codex ・ Kiro で5回ずつ回し、指せた割合 ・ 偽陽性 ・ 揃った割合を記録する",
    "段D（妥当性確認）：実物のボード1枚に AI の確かめを当て、利用者が違反の報告を全件読んで分けた記録が残る",
    "測り方1で書けない決まりが1件でも出たら、Q1 の「正本は JSON Schema のまま」を見直す"
   ]
  },
  "images": [{"kind": "code", "caption": "実験で作った、決まりごとの結果の応答（試案。先頭45行）。", "path": ".brainstorming/semantic-contract-core/measured/v03-experiment/v03/interpreter-response.trial.json", "language": "json", "text": code}],
  "assumptions": ["段C の回数と割合（5回 ・ 8割 ・ 1割）は例で、利用者が決める"],
  "grounds": [
   {"supports": "4つの測り方と、判定の基準", "basis": "基準は「いまの基盤の、公開した版を書き換えずに受け止められるか」", "tag": "primary", "source": "qa-advisor の回答（2026-10-17）。advisors/qa.json"},
   {"supports": "対象を116件にすること", "basis": "中身の無い「説明のとおりに書く」22件を除いた116件", "tag": "measured", "source": EXP},
   {"supports": "試作を、正本の形を決めるより前に置くこと", "basis": "最初の試作を Q3 より前へ移す", "tag": "primary", "source": "meta-thinking-advisor の回答（2026-10-17）。advisors/meta.json"}
  ],
  "rejected": [
   {"option": "正本の形を試作より先に決める（最初に立てた Q3）", "reason": "JSON Schema で持てない意味の実例が無いまま、解き方どうしを比べることになる"}
  ]
 },
 {
  "id": "Q4", "name": "移し方", "status": "waiting", "depends": ["Q1", "Q2", "Q3"],
  "question": "いまの3つの Skill と、保留にしたボード agents-in-base を、どう移すか",
  "answer": {
   "text": "基盤を足すだけの版で出し、brainstorming から順に決まりごとの結果を使う。agents-in-base は再開せず、その答えをこのボードの Q2 の材料として閉じる。",
   "details": [
    "基盤は足すだけの版で出す。abstract@v1 に従う Skill は、手を入れなくても今までと同じ結果を返す",
    "最初に移すのは brainstorming である。試作の対象と同じなので、試作の結果をそのまま使える",
    "次に ACDR を、作り直すときに決まりごとの結果を使う形で作る",
    "skills-creator は、作る Skill のスキーマに担い手の宣言を書けるようにする",
    "agents-in-base の Q1 ・ Q4 の答え（AI の呼び出しの契約）は、Agents 契約の材料として Q2 へ移した。2つの契約を並べない",
    "agents-in-base は、このボードの Q4 が決着したら、置き換えたことを記録して決着にする"
   ]
  },
  "checks": [
   {"target": "agents-in-base の扱い", "ask": "保留のまま再開せず、答えをこのボードへ移して閉じることでよいか"}
  ],
  "grounds": [
   {"supports": "agents-in-base を並べて進めないこと", "basis": "agents-in-base の Q1・Q4 は、v0.3 の Interpreter Protocol と同じものを別の言葉で決めようとしている疑いがある", "tag": "primary", "source": "platform-advisor の回答（2026-10-17）。advisors/platform.json"},
   {"supports": "agents-in-base の答えを材料として先に使うこと", "basis": "あのボードの Q0 の答えは、このボードの Q0 を直したあとの答えの候補と、同じ目的を指しているように見える", "tag": "primary", "source": "meta-thinking-advisor の回答（2026-10-17）。advisors/meta.json"},
   {"supports": "@v2 を並べて段階的に移すこと", "basis": "変更が abstract@v1 の意味を変えるなら @v2 を並べて段階的に移す", "tag": "primary", "source": "ddd-advisor の回答（2026-10-17）。advisors/ddd.json"}
  ],
  "rejected": [
   {"option": "agents-in-base を再開して並べて進める", "reason": "AI の呼び出しの契約が2つのボードで別の言葉で決まり、どちらが正本か分からなくなる"}
  ]
 }
]

# 根拠の引用は、アドバイザーの回答と実験の記録から、原文のまま切り出したもので差し替える
quotes = json.loads((HERE / "quotes_r1.json").read_text())
for t in topics:
    for i, g in enumerate(t["grounds"]):
        q = quotes.get(f"{t['id']}|{i}")
        if q:
            g["basis"] = q

patch = [
 {"op": "add", "path": "/kind", "value": "board"},
 {"op": "add", "path": "/id", "value": "semantic-contract-core"},
 {"op": "add", "path": "/title", "value": "基盤が見逃している誤りと、構想 v0.3 の取り込み"},
 {"op": "add", "path": "/intro", "value": "構想 Semantic Contract Core v0.3 を受けて、いまの基盤が見逃している誤りから、基盤に何を足し、何を扱わないかを決める。"},
 {"op": "add", "path": "/round", "value": 1},
 {"op": "add", "path": "/queue", "value": [{"topic": "Q0", "why": "足すものも扱わないものも、どの誤りを承認の前に見つけたいかで決まるため"}]},
 {"op": "add", "path": "/topics", "value": topics},
 {"op": "add", "path": "/log", "value": [{"date": "2026-10-17", "text": "構想 v0.3 を受けて、原典の照合 ・ 4つのアドバイザー ・ 実験の結果から論点を5つ立てた"}]},
]
print(json.dumps(patch, ensure_ascii=False))
