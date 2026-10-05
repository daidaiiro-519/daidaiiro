//! x-view の文の型（references/view.schema.json、ACDR 0129）のテスト。
//! 文の型の {…} は普通の JMESPath で、jmespath のアダプタを通して評価するので、アダプタと組み合わせて確かめる。

use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::application::view::{Instance, ViewEngine};
use schema_driven_core::domain::schema::Schema;
use serde_json::{json, Value};
use std::sync::Arc;

fn engine(schema: Value, instances: Vec<Value>) -> ViewEngine {
    let s = Schema::new("t.schema.json", schema, vec![]);
    let list = instances
        .into_iter()
        .map(|v| Instance {
            value: v,
            schema: "t.schema.json".into(),
        })
        .collect();
    ViewEngine::new(Arc::new(Jmespath), vec![("t.schema.json".into(), s)], list)
}

fn render(e: &ViewEngine, pointer: &str, value: Value) -> String {
    e.render("t.schema.json", pointer, &value).unwrap()
}

#[test]
fn text_fills_jmespath_and_formats_null_and_arrays() {
    let e = engine(
        json!({"$defs": {"m": {"x-view": {"text": "{min}件以上 {tags} [{none}]"}}}}),
        vec![],
    );
    assert_eq!(
        render(&e, "/$defs/m", json!({"min": 0, "tags": ["a", "b"]})),
        "0件以上 a ・ b []"
    );
}

#[test]
fn cases_pick_first_matching_when() {
    let e = engine(
        json!({"$defs": {"m": {"x-view": {"cases": [
            {"when": "min == `1` && max == `1`", "text": "1つ"},
            {"when": "max == null", "text": "{min}件以上"}], "text": "{min}〜{max}件"}}}}),
        vec![],
    );
    assert_eq!(render(&e, "/$defs/m", json!({"min": 1, "max": 1})), "1つ");
    assert_eq!(
        render(&e, "/$defs/m", json!({"min": 0, "max": null})),
        "0件以上"
    );
    assert_eq!(
        render(&e, "/$defs/m", json!({"min": 2, "max": 5})),
        "2〜5件"
    );
}

#[test]
fn parts_and_maps() {
    let e = engine(
        json!({"$defs": {"c": {"x-view": {
            "maps": {"agg": {"count": "の件数"}},
            "parts": {"subject": "{target}{map('agg', agg)}",
                      "predicate": [{"when": "op == 'le'", "text": "は{value}以下"}, {"when": "op == 'not_empty'", "text": "は空でない"}]},
            "text": "{part('subject', @)}{part('predicate', @)}"}}}}),
        vec![],
    );
    assert_eq!(
        render(
            &e,
            "/$defs/c",
            json!({"target": "エラー", "agg": "count", "op": "le", "value": 0})
        ),
        "エラーの件数は0以下"
    );
    assert_eq!(
        render(&e, "/$defs/c", json!({"target": "値", "op": "not_empty"})),
        "値は空でない"
    );
}

#[test]
fn quote_clause_neg_fill() {
    let e = engine(
        json!({"$defs": {"q": {"x-view": {"text": "{names | quote(@) | join('', @)}が無い"}},
                         "c": {"x-view": {"text": "{clause(s)}なら、"}},
                         "n": {"x-view": {"text": "{neg(a)} / {neg(b)}"}},
                         "f": {"x-view": {"text": "{fill(form, @)}"}}}}),
        vec![],
    );
    assert_eq!(
        render(&e, "/$defs/q", json!({"names": ["甲", "乙"]})),
        "「甲」「乙」が無い"
    );
    assert_eq!(
        render(&e, "/$defs/c", json!({"s": "ハッシュ値は空でない"})),
        "ハッシュ値が空でないなら、"
    );
    assert_eq!(
        render(
            &e,
            "/$defs/n",
            json!({"a": "値は空でない", "b": "件数は0以下"})
        ),
        "値は空だった / 件数は0以下でなかった"
    );
    assert_eq!(
        render(
            &e,
            "/$defs/f",
            json!({"form": "{to}に{data}を依頼する", "to": "システム", "data": ["パス", "スキーマ"]})
        ),
        "システムにパス、スキーマを依頼する"
    );
}

#[test]
fn name_uses_label_of_referenced_instance_or_item() {
    let schema = json!({
        "x-view": {"label": "header.name"},
        "properties": {"terms": {"items": {"x-view": {"label": "word"}}}},
        "$defs": {"r": {"x-view": {"text": "{refs | name(@) | join('、', @)}"}}}
    });
    let instances = vec![
        json!({"id": "UC-1", "kind": "uc", "header": {"name": "注文する"}}),
        json!({"id": "GLO-1", "kind": "glossary", "terms": [{"id": "TERM-1", "word": "注文"}]}),
    ];
    let e = engine(schema, instances);
    assert_eq!(
        render(
            &e,
            "/$defs/r",
            json!({"refs": ["UC-1", "GLO-1.TERM-1", "TERM-1", "UC-9"]})
        ),
        "注文する、注文、注文、UC-9"
    );
}

#[test]
fn view_renders_value_with_another_x_view() {
    let e = engine(
        json!({"$defs": {
            "inner": {"x-view": {"text": "{a}は{b}"}},
            "outer": {"x-view": {"text": "{clause(view('/$defs/inner', cond))}なら、終わる"}}}}),
        vec![],
    );
    assert_eq!(
        render(&e, "/$defs/outer", json!({"cond": {"a": "値", "b": "空"}})),
        "値が空なら、終わる"
    );
}

#[test]
fn hidden_renders_nothing_and_invalid_expression_is_error() {
    let e = engine(
        json!({"$defs": {"h": {"x-view": {"hidden": true}}, "bad": {"x-view": {"text": "{a[}"}}}}),
        vec![],
    );
    assert_eq!(render(&e, "/$defs/h", json!({"a": 1})), "");
    assert!(e
        .render("t.schema.json", "/$defs/bad", &json!({"a": 1}))
        .is_err());
}
