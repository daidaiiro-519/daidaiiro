# 第9回：利用者の指摘で、Q0 の先頭に目的だけを置き、標準は JSON Schema だけ ・ x- の拡張は基盤の契約（基盤のコマンドが確かめる）に直す。
# 標準へ写す（Projection）は外す。Q1 ・ Q2 ・ Q4 の該当の行も同じ前提に揃える。
# 根拠の引用は、組み立てるときに原文から切り出す。原文に無ければ止める。
import json, pathlib, sys

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
svg = lambda name: (HERE / f"{name}.svg").read_text()
core = (BOARD / "sources/www.ietf.org_archive_id_draft-bhutton-json-schema-01.txt").read_text().split("\n")


def q(lines, start, end):
    text = " ".join(l.strip() for l in lines)
    i = text.find(start)
    j = text.find(end, i)
    if i < 0 or j < 0:
        sys.exit(f"原文に無い：{start}")
    return text[i : j + len(end)]


CORE = "JSON Schema 2020-12 のコア（draft-bhutton-json-schema-01、474〜475行 ・ 1421〜1424行、2026-10-17 取得）"
U_STD = "標準はJsonSchmeaだけでいいのですが、それ以外の拡張はこちらが持つ契約だと思ってるよ。"
U_CMD = "私が考えているのはこの拡張した範囲はSchema駆動が持つセマンティックドメインが検証などのcommand能力を持つことをイメージしています。"
U_PURPOSE = "あとこれ目的と設計原則といっている割になんか目的がどこに書いているかわからないんだけど。。。"
user = lambda supports, words: {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
unknown = {"supports": "JSON Schema の実装は x- の拡張を確かめないこと", "basis": q(core, "Unknown keywords SHOULD be treated as annotations", "the value of the annotation."), "tag": "primary", "source": CORE}
vocab = {"supports": "足した語彙を必須にすると、知らない実装はそのスキーマを扱わないこと", "basis": q(core, "If the value is true, then implementations that do not recognize the vocabulary", "\"$schema\"."), "tag": "primary", "source": CORE}

T = {t["id"]: t for t in board["topics"]}

# ── Q0 ──
t = T["Q0"]
t["answer"] = {
    "text": "基盤の目的は、AI が書いたインスタンスを、宣言した決まりで機械が確かめ、確かめられなかった決まりも隠さず返すことである。",
    "details": [
        "そのために：5つの設計原則をすべて採り、基盤から AI を呼ぶ契約（Agents 契約）を足す",
        "原則1 宣言と解釈の分離：採る。いまも宣言（スキーマと注釈）と確かめ（エンジン）は分かれているが、確かめる担い手を外から足す仕組みが無い",
        "原則2 既存技術は部品として利用：採る。標準は JSON Schema だけにし、形（構造と値）の確かめを JSON Schema の実装に任せる",
        "原則2 の続き：JSON-LD ・ SHACL ・ RDF/SPARQL は、自前の概念を決めるときに参考にするだけで、標準として扱わず、写しもしない",
        "原則3 言語非依存の意味モデル（実装は Rust）：採る。いまも Rust で作り、CLI ・ MCP ・ JSON で返す。出力の形を JSON Schema で公開することが足りない",
        "原則4 拡張可能な表現モデル：採る。x- で足した範囲は基盤の契約で、基盤のコマンドが確かめ、結果を5つの語で返す",
        "原則4 の続き：JSON Schema の実装は、知らないキーワードを注釈として集めるだけで確かめない。いまの基盤も、足した注釈を確かめず、確かめなかったことも返さない",
        "原則4 の拡張には、外から足す解釈の担い手を閉じた環境で動かすこと（WASM Component）も含める。構想は、担い手の登録と実行の権限を管理するとしている",
        "原則5 検証可能で実用に耐える（再現性 ・ 適合性 ・ 可搬性）：採る。いまも同じ入力から同じ結果を返すが、別の言語の実装が使える適合の試験が無い",
        "従属させないための方針も採る。正本は abstract（JSON Schema に x- の拡張を足した私たちの契約）で、JSON Schema には形の確かめだけを任せる",
        "足すもの：基盤から AI を呼ぶ契約（Agents 契約）。全体構成図の外部インターフェースは、4つとも外から基盤を呼ぶ向きである",
        "基盤に残すもの：インスタンスの作成と更新 ・ 承認の記録 ・ 描画。全体構成図は描画を基盤の外に置くが、いまの基盤は承認の前に読む画面を作るために持つ",
    ],
}
for im in t["images"]:
    if im.get("name") == "設計原則":
        im["caption"] = "基盤の目的と、5つの設計原則のいまの基盤で足りているもの ・ 足りないもの。"
        im["svg"] = svg("q0-principles")
t["checks"] = [c for c in t["checks"] if c["target"] != "正本"]
t["grounds"] += [user("目的を先頭に置くこと", U_PURPOSE), user("標準は JSON Schema だけで、拡張は基盤の契約であること", U_STD), user("拡張した範囲を基盤のコマンドが確かめること", U_CMD), unknown]
t["rejected"].append({"option": "JSON-LD ・ SHACL ・ RDF/SPARQL の概念を自前で定義し、標準へ写して相互運用する（第5〜8回）", "reason": "標準は JSON Schema だけという決まりと食い違い、写す仕組みと写せなかったことの報告を基盤が持ち続けることになる"})

# ── Q1 ──
t = T["Q1"]
t["answer"]["details"] = [
    "公開の仕様：標準として互換を持つのは JSON/JSON Schema だけで、ほかは独自の拡張（x-）として基盤の契約に置く" if d.startswith("公開の仕様") else d
    for d in t["answer"]["details"]
]

# ── Q2 ──
t = T["Q2"]
t["name"] = "標準技術の概念"
t["question"] = "JSON Schema ・ JSON-LD ・ SHACL ・ RDF/SPARQL のうち、標準として使うものはどれで、ほかの概念のうち構想に要るものを基盤の概念としてどう持つか"
t["answer"] = {
    "text": "標準として使うのは JSON Schema だけで、形の確かめを任せる。JSON-LD ・ SHACL ・ RDF/SPARQL の概念のうち構想に要るものは、x- の拡張として抽象の仕様に定義し、基盤のコマンドが確かめる。",
    "details": [
        "JSON Schema：標準としてそのまま使う。構造と値の制約で、いまも abstract@v1 の土台であり、エンジンに組み込んで確かめている",
        "x- の拡張は JSON Schema の実装からは知らないキーワードで、注釈として集められるだけで合否に入らない。だから基盤のコマンドが確かめる",
        "足した語彙を $vocabulary で必須にすると、その語彙を知らない JSON Schema の実装はスキーマを扱わない。確かめずに合格と返す実装を止められる",
        "JSON-LD から参考にする概念：共通の識別子（@id）・ 手元の名前と共通の語の対応付け（@context）・ 再定義の保護（@protected）。Vocabulary Spec と Semantic Model に定義する",
        "SHACL から参考にする概念：ノードと関係に置く制約（Shape）・ 参照をたどる経路（path）・ 閉じるか開くか（closed）・ 検証結果の報告の形。Constraint Spec と結果の形に定義する",
        "RDF ・ SPARQL から参考にする概念：インスタンスをまたいだ関係のグラフと、それへの問い合わせ。いまは参照のつながり（links ・ referrers）を持つが、問い合わせの概念が無い",
        "標準へは写さない。JSON-LD ・ SHACL ・ RDF の実装（json-ld ・ rudof ・ oxigraph）を基盤に載せない",
        "そのまま使わない理由は2つ：既存仕様に表現を合わせないという方針と、役目の違い（JSON-LD は名前の対応付けで確かめない ・ SHACL は RDF グラフを確かめ、JSON をそのまま確かめない）",
    ],
}
t["images"] = [im for im in t["images"] if im.get("name") != "写しの報告（Zip）"]
for im in t["images"]:
    if im.get("name") == "取り込む概念":
        im["name"] = "標準と拡張"
        im["caption"] = "標準は JSON Schema だけ。ほかの技術の概念は自前の拡張（x-）として定義し、基盤のコマンドが確かめる。"
        im["svg"] = svg("q2-concepts")
    if im.get("name") == "自前の語彙（案）":
        im["caption"] = "語彙のスキーマの案：JSON-LD の識別子 ・ 対応付け ・ 再定義の保護と、SHACL の Shape ・ 経路 ・ 閉じるを参考に、自前の欄として持つ。"
t["checks"] = [{"target": "自前の概念", "ask": "3つの技術から参考にする概念は、この一覧で足りているか"}]
t["grounds"] = [g for g in t["grounds"] if "Context を取りに行く" not in g["supports"]]
t["grounds"] += [user("標準は JSON Schema だけで、拡張は基盤の契約であること", U_STD), user("拡張した範囲を基盤のコマンドが確かめること", U_CMD), unknown, vocab]
t["rejected"] = [
    {"option": "JSON-LD や SHACL を正本にする", "reason": "既存仕様に表現を合わせることになり、JSON-LD は確かめず、SHACL は JSON をそのまま確かめない"},
    {"option": "自前の概念から JSON-LD ・ SHACL へ写して相互運用する（第5〜8回）", "reason": "標準は JSON Schema だけという決まりと食い違い、写す仕組みと写せなかったことの報告を基盤が持ち続けることになる"},
    {"option": "x- の拡張の確かめを JSON Schema の実装に任せる", "reason": "JSON Schema の実装は知らないキーワードを注釈として集めるだけで、確かめない"},
    {"option": "実例が出るまで、これらの概念を持たない", "reason": "構想に要る概念を後回しにすると、抽象の仕様を後から書き換えることになる"},
]

# ── Q4 ──
t = T["Q4"]
t["answer"]["details"] = [
    "標準の部品の能力は、crate の feature で選ぶ。外へ取りに行く能力（$ref を HTTP で取りに行くなど）は既定で外す" if d.startswith("標準の部品の能力") else d
    for d in t["answer"]["details"]
]

patch = [
    {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、Q0 の先頭に目的だけを置き、標準は JSON Schema だけ ・ x- の拡張は基盤の契約で基盤のコマンドが確かめる、に直した。標準へ写すことは外した"}},
    {"op": "replace", "path": "/topics", "value": list(T.values())},
]
print(json.dumps(patch, ensure_ascii=False))
