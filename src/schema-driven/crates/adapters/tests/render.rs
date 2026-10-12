//! ページを描画する。Skill のフォルダのカタログで、契約を通ったデータだけを、同じ入力から同じページへ描画する。
//! 公開 API の振り分け（render）を、--skill に Skill example を渡して呼ぶ。

mod support;

use serde_json::{json, Value};
use support::{call, contains_all, dispatch, eq, has, put_catalog, run, Scratch};

/// スキーマの名前を引き、カタログを読む Skill。
const SKILL: Option<&str> = Some("./example");

/// render の引数。
const ARGS: &[(&str, &str)] = &[
    ("dir", "./declarations"),
    ("pages", "./pages"),
    ("out", "./out"),
];

/// 作業用のフォルダに、スキーマとカタログ（Skill example の assets）・ インスタンス ・ ページテンプレートを置く。
fn setup(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("render-{name}"));
    scratch.mkdir("declarations");
    scratch.mkdir("pages");
    put_catalog(&scratch, "example");
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
        "pages/thing.json",
        &json!({"kind": "thing", "head": {"lead": "this.id"}, "sections": [
        {"heading": "タグ", "body": [{"component": "pill", "each": "this.tags", "text": "@"}]}]}),
    );
    scratch
}

fn render(scratch: &Scratch) -> (i32, Value) {
    dispatch(scratch, SKILL, "render", ARGS)
}

#[test]
fn a_kind_is_drawn_with_its_page_template_and_the_skill_catalog() {
    let scratch = setup("kinds");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            0,
            &[
                ("/pages/0/path", eq("./out/T-1.html")),
                ("/pages/0/changed", eq(true)),
            ],
        )],
    );
    contains_all(
        "T-1.html",
        &scratch.read("out/T-1.html"),
        &[
            "<h1>&lt;甲&gt;</h1>",
            r#"<p class="lead">T-1</p>"#,
            "<h2>タグ</h2>",
            r#"<span class="pill ">a</span><span class="pill ">b</span>"#,
            ":root{--ink:#111d1a;",
            ".pill{color:var(--ink);background:var(--paper)}",
            "dataset.ready",
        ],
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
                ("/reason", eq("ページテンプレートの指定が無い種類がある")),
                ("/detail", has("kind other")),
            ],
        )],
    );
    assert!(!scratch.exists("out"));
}

#[test]
fn without_page_templates_no_kind_is_drawn() {
    let scratch = setup("no-pages");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            &[("dir", "./declarations"), ("out", "./out")],
            1,
            &[
                ("/reason", eq("ページテンプレートの指定が無い種類がある")),
                ("/detail", has("kind thing")),
            ],
        )],
    );
    assert!(!scratch.exists("out"));
}

#[test]
fn without_the_skill_catalog_nothing_is_written() {
    let scratch = setup("no-catalog");
    scratch.remove_dir("example/assets/catalog");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            ARGS,
            1,
            &[
                ("/reason", eq("部品のカタログを読めない")),
                ("/detail", has("assets/catalog が無い")),
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
    let (code, out) = render(&scratch);
    assert_eq!(code, 0, "{out}");
    assert!(scratch.exists("out/T-1.html"));
}

#[test]
fn page_template_violations_stop_rendering() {
    let scratch = setup("template");
    scratch.put(
        "pages/thing.json",
        &json!({"kind": "thing", "head": {"badges": "this.tags"}, "sections": [
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
                ("/detail", has("知らない部品 nope")),
                ("/detail", has("{{badges}} が無い")),
            ],
        )],
    );
    assert!(!scratch.exists("out"));
}
