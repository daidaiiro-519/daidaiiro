//! 部品のカタログは、Skill のフォルダの assets/catalog/ にあり、基盤は中身を持たない。
//! 基盤は、カタログが枠（schema-driven/catalog@v1）とデザインシステムの契約に従うかを見て、
//! 部品の入力を同じ名前のプレースホルダーへ入れるだけである。Skill が部品を足しても、基盤は変わらない。
//! 公開 API の振り分け（render）を、--skill に Skill example を渡して呼ぶ。

mod support;

use serde_json::{json, Value};
use support::{
    call, catalog_schema, contains_all, contains_none, design_system, eq, has, put_catalog, run,
    Scratch, PARTS,
};

/// スキーマの名前を引き、カタログを読む Skill。
const SKILL: Option<&str> = Some("./example");

/// render の引数。
const ARGS: &[(&str, &str)] = &[("dir", "./records"), ("pages", "./pages"), ("out", "./out")];

fn field(description: &str) -> Value {
    json!({"type": "string", "description": description,
           "x-prompt": {"read": {"means": "見本の欄である。"},
                        "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}})
}

/// 決定の記録を1件持つ作業用のフォルダを作る。スキーマとカタログは Skill example の assets に置く。page はページテンプレート。
fn skill(name: &str, page: Value, decision: &str) -> Scratch {
    let scratch = Scratch::new(&format!("catalog-{name}"));
    put_catalog(&scratch, "example");
    scratch.put(
        "example/assets/record.schema.json",
        &json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1", "$id": "example/record@v1", "title": "決定の記録", "description": "見本",
            "x-generates": "records/<番号>.json",
            "type": "object", "required": ["kind", "decision", "files"],
            "properties": {
                "$schema": field("従うスキーマ"),
                "kind": {"const": "record", "description": "種類",
                         "x-prompt": {"read": {"means": "種類である。"},
                                      "write": [{"id": "const", "rule": "record と書く。", "why": "種類を判定する。", "test": "値が record かを見る。"}]}},
                "decision": field("決めたこと"),
                "figure": field("図"),
                "files": {"type": "array", "items": {"type": "string"}, "description": "変えるファイル",
                          "x-prompt": {"read": {"means": "変えるファイルの並びである。"},
                                       "write": [{"id": "path", "rule": "パスを書く。", "why": "読み手が開く。", "test": "パスかを見る。"}]}}
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
        "figure": figure, "files": ["SKILL.md", "implementation.json"]
    })
}

fn page() -> Value {
    json!({"$schema": "schema-driven/page@v1", "kind": "record", "head": {"lead": "this.decision"}, "sections": [
        {"heading": "決めたこと", "body": ["this.decision", {"component": "figure", "svg": "this.figure"}]},
        {"heading": "変えるファイル", "body": [
            {"component": "list", "items": [
                {"component": "pill", "each": "this.files", "text": "@", "tone": {"text": "file"}}]}]}
    ]})
}

/// 描画して、書いたページを返す。描画できなければ失敗にする。
fn drawn(scratch: &Scratch) -> String {
    run(scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    scratch.read("out/0001.html")
}

#[test]
fn a_concrete_kind_is_drawn_with_the_skill_catalog_only() {
    let page = drawn(&skill("draw", page(), "段を段階に直す"));
    contains_all(
        "0001.html",
        &page,
        &[
            r#"<p class="lead">段を段階に直す</p>"#,
            r#"<figure><svg viewBox="0 0 10 10">"#,
            r#"<ul><span class="pill file">SKILL.md</span><span class="pill file">implementation.json</span></ul>"#,
            r#"@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){--ink:var(--mist);--paper:var(--night);color-scheme:dark}}"#,
        ],
    );
}

#[test]
fn a_part_the_skill_adds_is_drawn_without_changing_the_base() {
    let scratch = skill("added", page(), "x");
    let mut schema = catalog_schema();
    schema["$defs"]["note"] = json!({"type": "object", "description": "注意", "required": ["text"],
        "properties": {"text": {}}, "x-states": {"default": "押せない"}});
    scratch.put("example/assets/catalog/catalog.schema.json", &schema);
    scratch.put_text(
        "example/assets/catalog/parts.html",
        &format!(r#"{PARTS}<template id="note"><aside>{{{{text}}}}</aside></template>"#),
    );
    scratch.put(
        "pages/record.json",
        &json!({"kind": "record", "sections": [{"heading": "h", "body": [{"component": "note", "text": "this.decision"}]}]}),
    );
    contains_all("0001.html", &drawn(&scratch), &["<aside>x</aside>"]);
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
fn an_svg_with_a_script_is_refused() {
    let scratch = skill("svg", page(), "x");
    scratch.put(
        "records/0001.json",
        &record("x", "<svg><script>alert(1)</script></svg>"),
    );
    run(
        &scratch,
        SKILL,
        &[call("render", ARGS, 1, &[("/detail", has("script"))])],
    );
    assert!(!scratch.exists("out"), "1ページも書かない");
}

#[test]
fn a_part_missing_from_the_catalog_is_refused() {
    let mut bad = page();
    bad["sections"][1]["body"][0]["component"] = json!("changeMarker");
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
    bad["sections"][1]["body"][0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("text");
    let scratch = skill("missing", bad, "x");
    run(
        &scratch,
        SKILL,
        &[call("render", ARGS, 1, &[("/detail", has("text"))])],
    );
}

#[test]
fn a_catalog_outside_the_frame_is_refused() {
    let scratch = skill("frame", page(), "x");
    let mut schema = catalog_schema();
    schema["$defs"]["pill"]
        .as_object_mut()
        .unwrap()
        .remove("x-states");
    scratch.put("example/assets/catalog/catalog.schema.json", &schema);
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            1,
            &[
                ("/reason", eq("部品のカタログが契約を満たさない")),
                ("/detail", has("x-states")),
            ],
        )],
    );
    assert!(!scratch.exists("out"), "1ページも書かない");
}

#[test]
fn a_design_system_below_the_contrast_floor_is_refused() {
    let scratch = skill("contrast", page(), "x");
    let mut system = design_system();
    system["dark"]["ink"] = json!("{night}");
    scratch.put("example/assets/catalog/design-system.json", &system);
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            1,
            &[
                ("/reason", eq("部品のカタログが契約を満たさない")),
                ("/detail", has("暗：ink と paper のコントラストが 1.00:1")),
            ],
        )],
    );
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
