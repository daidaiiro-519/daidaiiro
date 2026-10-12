//! 部品のカタログ（schemas/catalog.schema.json）。
//! 具体の種類は、ページテンプレートでカタログの部品を並べるだけで描ける。具体はコードを持たない。
//! 公開 API の振り分け（render）を、--skill に Skill example を渡して呼ぶ。

mod support;

use serde_json::{json, Value};
use support::{call, contains_all, contains_none, eq, has, run, Scratch};

/// スキーマの名前を引く Skill。
const SKILL: Option<&str> = Some("./example");

/// render の引数。
const ARGS: &[(&str, &str)] = &[("dir", "./records"), ("pages", "./pages"), ("out", "./out")];

fn field(description: &str) -> Value {
    json!({"type": "string", "description": description,
           "x-prompt": {"read": {"means": "見本の欄である。"},
                        "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}})
}

/// 決定の記録を1件持つ作業用のフォルダを作る。スキーマは Skill example の assets に置く。page はページテンプレート。
fn skill(name: &str, page: Value, decision: &str) -> Scratch {
    let scratch = Scratch::new(&format!("catalog-{name}"));
    let change = json!({"type": "object", "required": ["file", "from", "to", "why"],
        "additionalProperties": false,
        "properties": {"file": field("ファイル"), "from": field("変更前"), "to": field("変更後"), "why": field("なぜ")}});
    scratch.put(
        "example/assets/record.schema.json",
        &json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1", "$id": "example/record@v1", "title": "決定の記録", "description": "見本",
            "x-generates": "records/<番号>.json",
            "type": "object", "required": ["kind", "decision", "changes"],
            "properties": {
                "$schema": field("従うスキーマ"),
                "kind": {"const": "record", "description": "種類",
                         "x-prompt": {"read": {"means": "種類である。"},
                                      "write": [{"id": "const", "rule": "record と書く。", "why": "種類を判定する。", "test": "値が record かを見る。"}]}},
                "decision": field("決めたこと"),
                "figure": field("図"),
                "target": field("対象の文書"),
                "changes": {"type": "array", "items": change, "description": "変更",
                            "x-prompt": {"read": {"means": "変更の並びである。"},
                                         "write": [{"id": "with-why", "rule": "変更ごとに理由を書く。", "why": "承認の材料になる。", "test": "why があるかを見る。"}]}}
            }
        }),
    );
    scratch.put(
        "records/0001.json",
        &record(
            decision,
            "<svg viewBox=\"0 0 10 10\"><rect width=\"10\" height=\"10\"/></svg>",
        ),
    );
    scratch.put("pages/record.json", &page);
    scratch
}

fn record(decision: &str, figure: &str) -> Value {
    json!({
        "$schema": "example/record@v1", "kind": "record", "decision": decision,
        "figure": figure,
        "target": "<p>対象の文書</p><script>alert(1)</script>",
        "changes": [
            {"file": "SKILL.md", "from": "段の順に作業する", "to": "段階の順に作業する", "why": "段は一般の語ではない"},
            {"file": "implementation.json", "from": "copy", "to": "（外した）", "why": "ソースのコピーをやめた"}
        ]
    })
}

fn page() -> Value {
    json!({"$schema": "schema-driven/page@v1", "kind": "record", "sections": [
        {"heading": "決めたこと", "body": ["this.decision", {"component": "svg", "value": "this.figure"}]},
        {"heading": "変更", "body": [
            {"component": "jump", "label": {"text": "2件目へ"}, "to": "'implementation.json'"},
            {"component": "tabs", "items": "this.changes", "label": "file", "id": "file",
             "body": [{"component": "mark", "after": "to", "before": "from", "why": "why"}]}]},
        {"heading": "対象", "body": [{"component": "fold", "label": {"text": "開く"},
            "body": [{"component": "embed", "html": "this.target", "title": {"text": "対象の文書"}}]}]},
        {"heading": "回答", "body": [
            {"component": "choose", "name": "'verdict'", "options": [{"value": "approved", "label": "承認"}, {"value": "returned", "label": "差し戻し"}]},
            {"component": "write", "name": "'reason'", "label": {"text": "理由"}},
            {"component": "send", "label": {"text": "送る"}, "to": "answer"}]}
    ]})
}

/// 描画して、書いたページを返す。描画できなければ失敗にする。
fn drawn(scratch: &Scratch) -> String {
    run(scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    scratch.read("out/0001.html")
}

/// カタログは部品の名前を並べる。12の部品を、ページテンプレートで使えば検査を通って描ける。
/// page() が9つ（svg ・ jump ・ tabs ・ mark ・ fold ・ embed ・ choose ・ write ・ send）、ここで足す3つ（document ・ badges ・ inline）。
#[test]
fn the_catalog_names_every_part() {
    let scratch = skill("names", page(), "x");
    run(&scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    scratch.put(
        "pages/record.json",
        &json!({"kind": "record", "sections": [{"heading": "h", "body": [
            {"component": "document", "value": "this"},
            {"component": "badges", "value": "this.changes[].file"},
            {"component": "inline", "value": "this.decision"}]}]}),
    );
    run(&scratch, SKILL, &[call("render", ARGS, 0, &[])]);
}

#[test]
fn a_concrete_kind_is_drawn_with_the_catalog_only() {
    let scratch = skill("draw", page(), "段を段階に直す");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            0,
            &[("/pages/0/design", eq("catalog"))],
        )],
    );
    contains_all(
        "0001.html",
        &scratch.read("out/0001.html"),
        &[
            "段を段階に直す",
            r#"role="tab""#,
            "SKILL.md",
            "implementation.json",
            "段階の順に作業する",
            "段は一般の語ではない",
            "<svg viewBox",
            r#"value="approved""#,
            "<textarea",
        ],
    );
}

#[test]
fn the_parts_carry_their_behaviour_in_the_page() {
    let page = drawn(&skill("behaviour", page(), "段を段階に直す"));
    assert_eq!(
        page.matches("<script>").count(),
        1,
        "部品の操作は1つの script にまとまる"
    );
    contains_all(
        "0001.html",
        &page,
        &[
            r#"data-sd-part="tabs""#,
            r#"data-sd-part="mark""#,
            r#"data-sd-to="sd-implementation.json""#,
            r#"id="sd-implementation.json""#,
        ],
    );
}

#[test]
fn hidden_panes_stay_hidden_whatever_their_display() {
    let page = drawn(&skill("hidden", page(), "隠す"));
    contains_all(
        "0001.html",
        &page,
        &[
            // 部品の display が hidden を打ち消さない
            ".sd-doc [hidden]{display:none!important}",
            r#"<div class="sd-pane" role="tabpanel" id="sd-implementation.json" hidden>"#,
        ],
    );
}

#[test]
fn values_are_text_even_when_they_hold_html() {
    let page = drawn(&skill("escape", page(), "<b style=\"color:red\">赤</b>"));
    contains_all(
        "0001.html",
        &page,
        &["&lt;b style=&quot;color:red&quot;&gt;赤&lt;/b&gt;"],
    );
    contains_none("0001.html", &page, &["<b style"]);
}

#[test]
fn an_embedded_document_cannot_run_scripts() {
    let page = drawn(&skill("embed", page(), "埋める"));
    contains_all(
        "0001.html",
        &page,
        &[
            // 埋めた文書は script を動かせない枠に入る
            r#"sandbox="""#,
            "srcdoc=\"&lt;p&gt;対象の文書&lt;/p&gt;&lt;script&gt;",
        ],
    );
}

#[test]
fn a_part_missing_from_the_catalog_is_refused() {
    let mut bad = page();
    bad["sections"][1]["body"][1]["component"] = json!("changeMarker");
    let scratch = skill("unknown", bad, "x");
    run(
        &scratch,
        SKILL,
        &[call("render", ARGS, 1, &[("/detail", has("changeMarker"))])],
    );
    assert!(!scratch.exists("out"), "1ページも書かない");
}

#[test]
fn a_missing_input_is_refused() {
    let mut bad = page();
    bad["sections"][1]["body"][1]["body"][0]
        .as_object_mut()
        .unwrap()
        .remove("why");
    let scratch = skill("missing", bad, "x");
    run(
        &scratch,
        SKILL,
        &[call("render", ARGS, 1, &[("/detail", has("why"))])],
    );
}

#[test]
fn an_svg_with_a_script_is_refused() {
    let scratch = skill("svg", page(), "x");
    scratch.put(
        "records/0001.json",
        &record("x", "<svg><script>alert(1)</script></svg>"),
    );
    run(&scratch, SKILL, &[call("render", ARGS, 1, &[])]);
}

#[test]
fn the_same_input_draws_the_same_page() {
    let scratch = skill("same", page(), "同じ");
    let first = drawn(&scratch);
    run(
        &scratch,
        SKILL,
        &[call("render", ARGS, 0, &[("/pages/0/changed", eq(false))])],
    );
    assert_eq!(first, scratch.read("out/0001.html"));
}
