//! ページの生成（3c。ボード schema-driven-build の論点5 C、ACDR 0132 ・ 0133）のテスト。
//! テスト用の小さな具体（MiniDesign）が、ポート Design にコンポーネントと部品の実装を渡す。
//! 基盤は節を並べ、show ・ each を適用し、式を描画の文脈で評価し、文字をエスケープし、プレースホルダーの過不足をエラーにする。

use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::application::context::contexts;
use schema_driven_core::application::page::{page_errors, PageRenderer};
use schema_driven_core::application::view::{Instance, ViewEngine};
use schema_driven_core::domain::check::Doc;
use schema_driven_core::domain::schema::Schema;
use schema_driven_core::domain::values::{Html, JsonValue};
use schema_driven_core::ports::outbound::{Design, Frame, Functions, Renderer};
use serde_json::{json, Value};
use std::sync::Arc;

/// テスト用の小さな具体。部品の HTML はデータで、Rust はプレースホルダーに値を入れるだけ。
struct MiniDesign {
    parts: &'static str,
}

const PARTS: &str = r#"<template id="page"><main><h1>{{title}}</h1>{{badges}}{{lead}}{{sections}}</main></template>
<template id="section"><section><h2>{{heading}}</h2>{{body}}</section></template>
<template id="pill"><span class="pill {{tone}}">{{text}}</span></template>
<template id="table"><table><tr>{{head}}</tr>{{body}}</table></template>
<template id="th"><th>{{text}}</th></template>
<template id="tr"><tr>{{cells}}</tr></template>
<template id="td"><td>{{text}}</td></template>
<template id="lead"><p>{{text}}</p></template>"#;

fn mini() -> MiniDesign {
    MiniDesign { parts: PARTS }
}

impl Design for MiniDesign {
    fn components(&self) -> Vec<(String, Value)> {
        vec![
            (
                "pill".into(),
                json!({"type": "object", "required": ["value"],
                       "properties": {"component": {}, "show": {}, "each": {}, "value": {}, "tone": {"type": "string"}},
                       "additionalProperties": false}),
            ),
            (
                "table".into(),
                json!({"type": "object", "required": ["rows", "cols"],
                       "properties": {"component": {}, "show": {}, "each": {}, "rows": {"type": "string"},
                                      "cols": {"type": "array"}},
                       "additionalProperties": false}),
            ),
        ]
    }

    fn parts(&self) -> &str {
        self.parts
    }

    fn page(&self, f: &Frame, r: &dyn Renderer) -> Result<Html, String> {
        let lead = if f.lead.as_str().is_empty() {
            Html::default()
        } else {
            r.part("lead", &[("text", f.lead.clone())])?
        };
        r.part(
            "page",
            &[
                ("title", r.text(&f.title)),
                ("badges", f.badges.clone()),
                ("lead", lead),
                ("sections", f.sections.clone()),
            ],
        )
    }

    fn section(&self, heading: &str, body: Html, r: &dyn Renderer) -> Result<Html, String> {
        r.part("section", &[("heading", r.text(heading)), ("body", body)])
    }

    fn render(&self, name: &str, input: &Value, r: &mut dyn Renderer) -> Result<Html, String> {
        match name {
            "pill" => {
                let v = r.node(&input["value"])?;
                let tone = match input.get("tone").and_then(Value::as_str) {
                    Some(table) => {
                        let raw = r.value(&input["value"])?;
                        r.tone(table, &raw).unwrap_or_else(|| "plain".into())
                    }
                    None => "plain".into(),
                };
                r.part("pill", &[("tone", r.text(&tone)), ("text", v)])
            }
            "table" => {
                let rows = r.value(&input["rows"])?;
                let cols = input["cols"].as_array().cloned().unwrap_or_default();
                let mut head = Html::default();
                for c in &cols {
                    head.push(r.part("th", &[("text", r.text(c["head"].as_str().unwrap_or("")))])?);
                }
                let mut body = Html::default();
                for row in rows.as_array().cloned().unwrap_or_default() {
                    let mut cells = Html::default();
                    for c in &cols {
                        let v = r.node_at(&c["value"], &row)?;
                        cells.push(r.part("td", &[("text", v)])?);
                    }
                    body.push(r.part("tr", &[("cells", cells)])?);
                }
                r.part("table", &[("head", head), ("body", body)])
            }
            other => Err(format!("MiniDesign は {other} を描画できない")),
        }
    }
}

/// 具体が足す関数の例。
struct Shout;

impl Functions for Shout {
    fn names(&self) -> Vec<String> {
        vec!["shout".into()]
    }
    fn call(&self, _: &str, args: &[Value]) -> Result<Value, String> {
        Ok(json!(format!("{}!", args[0].as_str().unwrap_or(""))))
    }
}

fn uc_schema() -> Value {
    json!({"x-view": {"label": "header.name"},
           "properties": {"rules": {"x-ref": {"to": "rule"}}}})
}

fn rule_schema() -> Value {
    json!({"x-view": {"label": "name"}})
}

fn instances() -> Vec<Value> {
    vec![
        json!({"id": "UC-1", "kind": "uc", "header": {"name": "注文 <する>", "level": "user"},
               "tags": ["甲", "乙"], "rules": ["R-1", "R-2"], "steps": []}),
        json!({"id": "R-1", "kind": "rule", "name": "在庫がある", "owner": "倉庫"}),
        json!({"id": "R-2", "kind": "rule", "name": "支払える", "owner": "経理"}),
    ]
}

/// UC-1 のページを生成する。
fn render_uc(
    page: Value,
    design: MiniDesign,
    extra: Option<Arc<dyn Functions>>,
) -> Result<String, String> {
    let uc = Schema::new("uc.schema.json", uc_schema(), vec![]);
    let rule = Schema::new("rule.schema.json", rule_schema(), vec![]);
    let values = instances();
    let docs: Vec<Doc> = values
        .iter()
        .map(|v| {
            let s = if v["kind"] == "uc" { &uc } else { &rule };
            Doc {
                path: format!("{}.json", v["id"].as_str().unwrap()),
                value: v.clone(),
                hash: JsonValue::new(&v.to_string()).hash(),
                schema: s,
            }
        })
        .collect();
    let ctx = contexts(&docs).remove(0);
    let list = values
        .iter()
        .map(|v| Instance {
            value: v.clone(),
            schema: if v["kind"] == "uc" {
                "uc.schema.json".into()
            } else {
                "rule.schema.json".into()
            },
        })
        .collect();
    let mut engine = ViewEngine::new(
        Arc::new(Jmespath),
        vec![
            (
                "uc.schema.json".into(),
                Schema::new("uc.schema.json", uc_schema(), vec![]),
            ),
            (
                "rule.schema.json".into(),
                Schema::new("rule.schema.json", rule_schema(), vec![]),
            ),
        ],
        list,
    );
    if let Some(f) = extra {
        engine = engine.with_functions(f).map_err(|e| e.0)?;
    }
    let renderer = PageRenderer::new(engine, Arc::new(design)).map_err(|e| e.0)?;
    renderer
        .render(&page, &ctx, "uc.schema.json")
        .map(|h| h.as_str().to_owned())
        .map_err(|e| e.0)
}

#[test]
fn frame_title_comes_from_label_and_sections_follow_show() {
    let page = json!({"kind": "uc", "head": {"lead": "this.header.level"}, "sections": [
        {"heading": "出る節", "body": [{"text": "固定"}]},
        {"heading": "出ない節", "show": "this.steps", "body": [{"text": "x"}]}]});
    let html = render_uc(page, mini(), None).unwrap();
    assert_eq!(
        html,
        "<main><h1>注文 &lt;する&gt;</h1><p>user</p><section><h2>出る節</h2>固定</section></main>"
    );
}

#[test]
fn expression_nodes_are_escaped_and_arrays_joined() {
    let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": ["this.header.name", "this.tags"]}]});
    let html = render_uc(page, mini(), None).unwrap();
    assert!(
        html.contains("<section><h2>h</h2>注文 &lt;する&gt;甲 ・ 乙</section>"),
        "{html}"
    );
}

#[test]
fn component_each_and_show_are_applied_by_the_base() {
    let page = json!({"kind": "uc", "tones": {"t": {"甲": "accent"}}, "sections": [{"heading": "h", "body": [
        {"component": "pill", "each": "this.tags", "value": "@", "tone": "t"},
        {"component": "pill", "show": "this.steps", "value": "'出ない'"}]}]});
    let html = render_uc(page, mini(), None).unwrap();
    assert!(
        html.contains(
            r#"<span class="pill accent">甲</span><span class="pill plain">乙</span></section>"#
        ),
        "{html}"
    );
}

#[test]
fn table_renders_column_nodes_against_each_row() {
    let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
        {"component": "table", "rows": "links[?at=='/rules'].target",
         "cols": [{"head": "ルール", "value": "name(@)"},
                  {"head": "バッジ", "value": {"component": "pill", "value": "@"}}]}]}]});
    let html = render_uc(page, mini(), None).unwrap();
    assert!(
        html.contains(
            r#"<table><tr><th>ルール</th><th>バッジ</th></tr><tr><td>在庫がある</td><td><span class="pill plain">R-1</span></td></tr><tr><td>支払える</td><td><span class="pill plain">R-2</span></td></tr></table>"#
        ),
        "{html}"
    );
}

#[test]
fn concrete_functions_are_usable_in_page_expressions() {
    let page =
        json!({"kind": "uc", "sections": [{"heading": "h", "body": ["shout(this.header.level)"]}]});
    let html = render_uc(page, mini(), Some(Arc::new(Shout))).unwrap();
    assert!(html.contains("user!"), "{html}");
}

#[test]
fn part_slots_must_match_exactly() {
    let missing = MiniDesign {
        parts: r#"<template id="page"><main>{{title}}{{sections}}</main></template>
<template id="section"><section>{{heading}}{{body}}</section></template>
<template id="lead"><p>{{text}}</p></template>"#,
    };
    let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [{"text": "x"}]}]});
    let err = render_uc(page, missing, None).unwrap_err();
    assert!(err.contains("badges"), "{err}");
}

#[test]
fn unknown_component_is_an_error() {
    let page =
        json!({"kind": "uc", "sections": [{"heading": "h", "body": [{"component": "nope"}]}]});
    // 具体に渡す前に、基盤が登録されていない名前を止める
    assert_eq!(
        render_uc(page, mini(), None).unwrap_err(),
        "知らないコンポーネント: nope"
    );
}

#[test]
fn page_errors_report_unknown_components_and_input_shape() {
    let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
        {"component": "nope"},
        {"component": "pill"},
        {"component": "table", "rows": "x", "cols": [{"head": "h", "value": {"component": "pill", "value": "@", "tone": 1}}]}]}]});
    let errors = page_errors(&mini(), &page);
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(errors[0].contains("nope"));
    assert!(errors[1].contains("pill") && errors[1].contains("value"));
    assert!(errors[2].contains("tone"));
}
