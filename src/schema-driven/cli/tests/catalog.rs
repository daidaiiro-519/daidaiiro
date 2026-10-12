//! 部品のカタログ（schemas/catalog.schema.json）。
//! 具体の種類は、ページテンプレートでカタログの部品を並べるだけで描ける。具体はコードを持たない。

mod common;

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const CATALOG: &str = include_str!("../../schemas/catalog.schema.json");

fn field(description: &str) -> Value {
    json!({"type": "string", "description": description,
           "x-prompt": {"read": {"means": "見本の欄である。"},
                        "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}})
}

/// 決定の記録を1件持つ作業用のフォルダを作る。スキーマは Skill example の assets に置く。page はページテンプレート。
fn skill(name: &str, page: Value, decision: &str) -> common::Scratch {
    let dir = common::Scratch::new(&format!("sd-catalog-{name}"));
    let _ = fs::remove_dir_all(&dir);
    for sub in ["example/assets", "records", "pages"] {
        fs::create_dir_all(dir.join(sub)).unwrap();
    }
    let change = json!({"type": "object", "required": ["file", "from", "to", "why"],
        "additionalProperties": false,
        "properties": {"file": field("ファイル"), "from": field("変更前"), "to": field("変更後"), "why": field("なぜ")}});
    let schema = json!({
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
    });
    fs::write(
        dir.join("example/assets/record.schema.json"),
        schema.to_string(),
    )
    .unwrap();
    let record = json!({
        "$schema": "example/record@v1", "kind": "record", "decision": decision,
        "figure": "<svg viewBox=\"0 0 10 10\"><rect width=\"10\" height=\"10\"/></svg>",
        "target": "<p>対象の文書</p><script>alert(1)</script>",
        "changes": [
            {"file": "SKILL.md", "from": "段の順に作業する", "to": "段階の順に作業する", "why": "段は一般の語ではない"},
            {"file": "implementation.json", "from": "copy", "to": "（外した）", "why": "ソースのコピーをやめた"}
        ]
    });
    fs::write(dir.join("records/0001.json"), record.to_string()).unwrap();
    fs::write(dir.join("pages/record.json"), page.to_string()).unwrap();
    dir
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

fn render(dir: &Path) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_schema-driven"))
        .current_dir(dir)
        .arg("--skill")
        .arg(dir.join("example"))
        .args([
            "render", "--dir", "records", "--pages", "pages", "--out", "out",
        ])
        .output()
        .unwrap();
    let out = serde_json::from_slice(&output.stdout).unwrap_or_else(
        |_| json!({"unparsed": String::from_utf8_lossy(&output.stdout).into_owned()}),
    );
    (output.status.code().unwrap_or(-1), out)
}

fn html(dir: &Path) -> String {
    fs::read_to_string(dir.join("out/0001.html")).unwrap()
}

#[test]
fn the_catalog_names_every_part() {
    let catalog: Value = serde_json::from_str(CATALOG).unwrap();
    assert_eq!(catalog["$id"], "schema-driven/catalog@v1");
    let names: Vec<&str> = catalog["$defs"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    for part in [
        "document", "badges", "inline", "tabs", "fold", "mark", "jump", "choose", "write", "send",
        "embed", "svg",
    ] {
        assert!(names.contains(&part), "カタログに {part} が無い：{names:?}");
    }
}

#[test]
fn a_concrete_kind_is_drawn_with_the_catalog_only() {
    let dir = skill("draw", page(), "段を段階に直す");
    let (code, out) = render(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["pages"][0]["design"], "catalog");
    let page = html(&dir);
    assert!(page.contains("段を段階に直す"));
    assert!(
        page.contains(r#"role="tab""#)
            && page.contains("SKILL.md")
            && page.contains("implementation.json")
    );
    assert!(page.contains("段階の順に作業する") && page.contains("段は一般の語ではない"));
    assert!(page.contains("<svg viewBox"));
    assert!(page.contains(r#"value="approved""#) && page.contains("<textarea"));
}

#[test]
fn the_parts_carry_their_behaviour_in_the_page() {
    let dir = skill("behaviour", page(), "段を段階に直す");
    assert_eq!(render(&dir).0, 0);
    let page = html(&dir);
    assert_eq!(
        page.matches("<script>").count(),
        1,
        "部品の操作は1つの script にまとまる"
    );
    assert!(page.contains(r#"data-sd-part="tabs""#) && page.contains(r#"data-sd-part="mark""#));
    assert!(page.contains(r#"data-sd-to="sd-implementation.json""#));
    assert!(page.contains(r#"id="sd-implementation.json""#));
}

#[test]
fn hidden_panes_stay_hidden_whatever_their_display() {
    let dir = skill("hidden", page(), "隠す");
    assert_eq!(render(&dir).0, 0);
    let page = html(&dir);
    assert!(
        page.contains(".sd-doc [hidden]{display:none!important}"),
        "部品の display が hidden を打ち消さない"
    );
    assert!(page
        .contains(r#"<div class="sd-pane" role="tabpanel" id="sd-implementation.json" hidden>"#));
}

#[test]
fn values_are_text_even_when_they_hold_html() {
    let dir = skill("escape", page(), "<b style=\"color:red\">赤</b>");
    assert_eq!(render(&dir).0, 0);
    let page = html(&dir);
    assert!(page.contains("&lt;b style=&quot;color:red&quot;&gt;赤&lt;/b&gt;"));
    assert!(!page.contains("<b style"));
}

#[test]
fn an_embedded_document_cannot_run_scripts() {
    let dir = skill("embed", page(), "埋める");
    assert_eq!(render(&dir).0, 0);
    let page = html(&dir);
    assert!(
        page.contains(r#"sandbox="""#),
        "埋めた文書は script を動かせない枠に入る"
    );
    assert!(page.contains("srcdoc=\"&lt;p&gt;対象の文書&lt;/p&gt;&lt;script&gt;"));
}

#[test]
fn a_part_missing_from_the_catalog_is_refused() {
    let mut bad = page();
    bad["sections"][1]["body"][1]["component"] = json!("changeMarker");
    let dir = skill("unknown", bad, "x");
    let (code, out) = render(&dir);
    assert_eq!(code, 1, "{out}");
    assert!(
        out["detail"].as_str().unwrap().contains("changeMarker"),
        "{out}"
    );
    assert!(!dir.join("out").exists(), "1ページも書かない");
}

#[test]
fn a_missing_input_is_refused() {
    let mut bad = page();
    bad["sections"][1]["body"][1]["body"][0]
        .as_object_mut()
        .unwrap()
        .remove("why");
    let dir = skill("missing", bad, "x");
    let (code, out) = render(&dir);
    assert_eq!(code, 1, "{out}");
    assert!(out["detail"].as_str().unwrap().contains("why"), "{out}");
}

#[test]
fn an_svg_with_a_script_is_refused() {
    let dir = skill("svg", page(), "x");
    let path = dir.join("records/0001.json");
    let mut record: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    record["figure"] = json!("<svg><script>alert(1)</script></svg>");
    fs::write(&path, record.to_string()).unwrap();
    assert_eq!(render(&dir).0, 1);
}

#[test]
fn the_same_input_draws_the_same_page() {
    let dir = skill("same", page(), "同じ");
    assert_eq!(render(&dir).0, 0);
    let first = html(&dir);
    let (code, out) = render(&dir);
    assert_eq!(code, 0);
    assert_eq!(out["pages"][0]["changed"], false);
    assert_eq!(first, html(&dir));
}
