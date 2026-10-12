//! ページを描画する。部品のカタログで、契約を通ったデータだけを、同じ入力から同じページへ描画する。
//! 公開 API の振り分け（render）を、--skill に Skill example を渡して呼ぶ。

mod support;

use serde_json::{json, Value};
use support::{call, contains_all, dispatch, eq, expect, has, run, Scratch};

/// スキーマの名前を引く Skill。
const SKILL: Option<&str> = Some("./example");

/// render の引数。
const ARGS: &[(&str, &str)] = &[
    ("dir", "./declarations"),
    ("pages", "./pages"),
    ("out", "./out"),
];

/// 作業用のフォルダに、スキーマ（Skill example の assets）・ インスタンス ・ ページテンプレートを置く。
fn setup(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("render-{name}"));
    scratch.mkdir("declarations");
    scratch.mkdir("pages");
    scratch.put(
        "example/assets/thing.schema.json",
        &json!({"$id": "example/thing@v1", "x-view": {"label": "name"}, "type": "object", "required": ["kind", "id", "name"],
            "properties": {"$schema": {}, "kind": {"type": "string"}, "id": {"type": "string"},
                           "name": {"type": "string"}, "tags": {"type": "array", "title": "タグの一覧", "items": {"type": "string"}}}}),
    );
    scratch.put(
        "declarations/T-1.json",
        &json!({"$schema": "example/thing@v1", "kind": "thing", "id": "T-1", "name": "<甲>", "tags": ["a", "b"]}),
    );
    scratch.put(
        "declarations/D-1.json",
        &json!({"$schema": "schema-driven/document@v1", "kind": "document", "id": "D-1", "title": "乙の報告",
               "sections": [{"heading": "結論", "blocks": [{"type": "paragraph", "text": "乙で進める", "strong": true}]}]}),
    );
    scratch.put(
        "pages/thing.json",
        &json!({"kind": "thing", "sections": [
        {"heading": "タグ", "body": [{"component": "badges", "value": "this.tags"}]}]}),
    );
    scratch
}

fn render(scratch: &Scratch) -> (i32, Value) {
    dispatch(scratch, SKILL, "render", ARGS)
}

#[test]
fn concrete_kinds_use_their_page_template_and_documents_the_fixed_one() {
    let scratch = setup("kinds");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            0,
            &[
                ("/pages/0/path", eq("./out/D-1.html")),
                ("/pages/0/design", eq("document")),
                ("/pages/1/path", eq("./out/T-1.html")),
                ("/pages/1/design", eq("catalog")),
            ],
        )],
    );
    contains_all(
        "T-1.html",
        &scratch.read("out/T-1.html"),
        &[
            "<h1>&lt;甲&gt;</h1>",
            "<h2>タグ</h2>",
            r#"<span class="sd-badge">a</span><span class="sd-badge">b</span>"#,
        ],
    );
    contains_all(
        "D-1.html",
        &scratch.read("out/D-1.html"),
        &["sd-doc", "<h1>乙の報告</h1>"],
    );
}

#[test]
fn kinds_without_a_template_stop_rendering_and_nothing_is_written() {
    let scratch = setup("missing");
    scratch.put(
        "declarations/O-1.json",
        &json!({"$schema": "example/thing@v1", "kind": "other", "id": "O-1", "name": "乙"}),
    );
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            1,
            &[
                ("/reason", eq("ページテンプレートが無い種類がある")),
                ("/detail", has("kind other")),
            ],
        )],
    );
    assert!(!scratch.exists("out"));
}

#[test]
fn same_input_gives_same_pages_and_inputs_are_untouched() {
    let scratch = setup("same");
    run(&scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    let first = scratch.snapshot(&["declarations", "pages", "out"]);
    run(
        &scratch,
        SKILL,
        &[call("render", ARGS, 0, &[("/pages/0/changed", eq(false))])],
    );
    assert_eq!(scratch.snapshot(&["declarations", "pages", "out"]), first);
}

#[test]
fn validation_errors_stop_rendering_and_nothing_is_written() {
    let scratch = setup("invalid");
    scratch.put(
        "declarations/T-1.json",
        &json!({"$schema": "example/thing@v1", "kind": "thing", "id": "T-1", "name": 1}),
    );
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            1,
            &[
                ("/reason", eq("検証を通過しないインスタンスがある")),
                ("/errors/0/property", has("T-1.json")),
            ],
        )],
    );
    assert!(!scratch.exists("out"));
}

#[test]
fn unfilled_properties_do_not_stop_rendering() {
    let scratch = setup("unfilled");
    scratch.put(
        "declarations/T-1.json",
        &json!({"$schema": "example/thing@v1", "kind": "thing", "id": "T-1"}),
    );
    run(&scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    assert!(scratch.exists("out/T-1.html"));
}

#[test]
fn page_template_violations_stop_rendering() {
    let scratch = setup("template");
    scratch.put(
        "pages/thing.json",
        &json!({"kind": "thing", "sections": [
        {"heading": "h", "body": [{"component": "nope"}]}]}),
    );
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            1,
            &[
                ("/reason", eq("ページテンプレートが契約を通過しない")),
                ("/detail", has("nope")),
            ],
        )],
    );
    assert!(!scratch.exists("out"));
}

#[test]
fn without_page_templates_documents_still_render() {
    let scratch = setup("document-only");
    scratch.remove("declarations/T-1.json");
    scratch.remove("pages/thing.json");
    run(&scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    assert!(scratch.exists("out/D-1.html"));
}

#[test]
fn document_blocks_render_by_information_type_and_escape_text() {
    let scratch = setup("blocks");
    scratch.remove("declarations/T-1.json");
    scratch.put(
        "declarations/D-1.json",
        &json!({"$schema": "schema-driven/document@v1", "kind": "document", "title": "<比較>", "lead": "前置き",
        "badges": ["2026-10-07"],
        "sections": [{"heading": "案", "blocks": [
            {"type": "table", "columns": ["案", "費用"], "rows": [["A", "小"], ["B", "大"]], "emphasis": 0},
            {"type": "steps", "items": [{"actor": "利用者", "text": "依頼する",
                "branches": [{"condition": "読めない", "steps": [{"text": "知らせる"}], "ending": "失敗"}]}]},
            {"type": "status", "items": [{"label": "3d", "state": "blocked", "note": "待ち"}]},
            {"type": "change", "rows": [{"what": "名前", "before": "l", "after": "instance"}]},
            {"type": "quote", "text": "A schema", "source": "json-schema-core:2057"},
            {"type": "bars", "items": [{"label": "甲", "value": 2}, {"label": "乙", "value": 4}]}]}]}),
    );
    run(&scratch, SKILL, &[call("render", ARGS, 0, &[])]);
    contains_all(
        "D-1.html",
        &scratch.read("out/D-1.html"),
        &[
            "<h1>&lt;比較&gt;</h1>",
            r#"<tr class="sd-emphasis"><td>A</td><td>小</td></tr>"#,
            r#"<span class="sd-actor">利用者</span>"#,
            r#"<div class="sd-branch-when">読めない</div>"#,
            r#"sd-state-blocked">止まっている"#,
            r#"<cite>json-schema-core:2057</cite>"#,
            r#"style="width:50%""#,
            r#"style="width:100%""#,
            r#"<span class="sd-badge">2026-10-07</span>"#,
        ],
    );
}

/// 文書を1件だけ置いた作業用のフォルダ。
fn with_document(name: &str, document: Value) -> Scratch {
    let scratch = setup(name);
    scratch.remove("declarations/T-1.json");
    scratch.put("declarations/D-1.json", &document);
    scratch
}

/// 文書を1件だけ置いて描画し、ページを返す。描画できなければ失敗にする。
fn rendered_page(name: &str, document: Value) -> String {
    let scratch = with_document(name, document);
    let (code, out) = render(&scratch);
    assert_eq!(code, 0, "{out}");
    scratch.read("out/D-1.html")
}

/// 文書を1件だけ置いて描画し、断られた結果の JSON を返す。
fn refused(name: &str, document: Value) -> Value {
    let scratch = with_document(name, document);
    let (code, out) = render(&scratch);
    assert_eq!(code, 1, "{out}");
    out
}

fn document_with(blocks: Value) -> Value {
    json!({"$schema": "schema-driven/document@v1", "kind": "document", "title": "t",
           "sections": [{"heading": "h", "blocks": blocks}]})
}

#[test]
fn inline_markup_becomes_strong_code_and_links_and_bad_urls_are_refused() {
    let page = rendered_page(
        "inline",
        document_with(
            json!([{"type": "paragraph", "text": "最大 **8.2 秒**、`pg_stat_activity` を見る。<b>は文字。[根拠](#source-s1)"}]),
        ),
    );
    contains_all(
        "D-1.html",
        &page,
        &[
            "<strong>8.2 秒</strong>",
            r#"<code class="sd-code-inline">pg_stat_activity</code>"#,
            "&lt;b&gt;は文字",
            r##"<a href="#source-s1">根拠</a>"##,
        ],
    );
    let out = refused(
        "inline-bad",
        document_with(json!([{"type": "paragraph", "text": "[押す](javascript:alert(1))"}])),
    );
    expect("refused", &out, &[("/detail", has("javascript:"))]);
}

#[test]
fn table_cells_can_be_badges_and_columns_right_aligned() {
    let page = rendered_page(
        "cells",
        document_with(
            json!([{"type": "table", "columns": ["候補", {"label": "件数", "align": "right"}],
            "rows": [[{"text": "原因", "tone": "bad"}, "40"]]}]),
        ),
    );
    contains_all(
        "D-1.html",
        &page,
        &[
            r#"<span class="sd-cell-badge sd-tone-bad">原因</span>"#,
            r#"<th class="sd-right">件数</th>"#,
            r#"<td class="sd-right">40</td>"#,
        ],
    );
}

#[test]
fn inline_svg_is_embedded_and_unsafe_svg_is_refused() {
    let page = rendered_page(
        "svg",
        document_with(
            json!([{"type": "figure", "svg": "<svg viewBox=\"0 0 1 1\"><rect width=\"1\" height=\"1\"/></svg>", "caption": "図"}]),
        ),
    );
    contains_all(
        "D-1.html",
        &page,
        &[r#"<div class="sd-svg"><svg viewBox="0 0 1 1">"#],
    );
    let out = refused(
        "svg-bad",
        document_with(
            json!([{"type": "figure", "svg": "<svg onload=\"alert(1)\"></svg>", "caption": "図"}]),
        ),
    );
    expect("refused", &out, &[("/detail", has("イベントの属性"))]);
}

#[test]
fn timeline_target_line_table_of_contents_and_sources() {
    let paragraph = json!([{"type": "paragraph", "text": "x"}]);
    let document = json!({"$schema": "schema-driven/document@v1", "kind": "document", "title": "t",
        "sections": [
            {"heading": "h", "blocks": [
                {"type": "timeline", "items": [{"time": "09:05", "text": "遅延が始まる"}]},
                {"type": "bars", "items": [{"label": "最大", "value": 8}], "target": {"label": "目標", "value": 2}}]},
            {"heading": "b", "blocks": paragraph},
            {"heading": "c", "blocks": paragraph},
            {"heading": "d", "blocks": paragraph}],
        "sources": [{"id": "s1", "label": "監視の記録", "url": "https://example.com/dashboard"}]});
    let page = rendered_page("more", document);
    contains_all(
        "D-1.html",
        &page,
        &[
            r#"<time class="sd-time">09:05</time>"#,
            r#"sd-bar-target"#,
            r#"style="left:25%""#,
            r##"<li><a href="#section-4">d</a></li>"##,
            r#"<li id="source-s1">監視の記録"#,
        ],
    );
}
