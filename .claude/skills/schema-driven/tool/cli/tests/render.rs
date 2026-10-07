//! UC-6 ページを描画する（3d）。具体のデザインを受け取り、契約を通ったデータだけを、同じ入力から同じページへ描画する。

use schema_driven_adapters::inbound::tools::Toolbox;
use schema_driven_adapters::outbound::document_design::DocumentDesign;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use schema_driven_core::domain::values::Html;
use schema_driven_core::ports::outbound::{Design, Frame, Renderer};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// テスト用の小さな具体のデザイン。
struct Mini;

impl Design for Mini {
    fn components(&self) -> Vec<(String, Value)> {
        vec![(
            "pill".into(),
            json!({"type": "object", "required": ["value"]}),
        )]
    }
    fn parts(&self) -> &str {
        r#"<template id="page"><main><h1>{{title}}</h1>{{sections}}</main></template>
<template id="section"><section><h2>{{heading}}</h2>{{body}}</section></template>
<template id="pill"><b>{{text}}</b></template>"#
    }
    fn page(&self, frame: &Frame, renderer: &dyn Renderer) -> Result<Html, String> {
        renderer.part(
            "page",
            &[
                ("title", renderer.text(&frame.title)),
                ("sections", frame.sections.clone()),
            ],
        )
    }
    fn section(&self, heading: &str, body: Html, renderer: &dyn Renderer) -> Result<Html, String> {
        renderer.part(
            "section",
            &[("heading", renderer.text(heading)), ("body", body)],
        )
    }
    fn render(&self, _: &str, input: &Value, renderer: &mut dyn Renderer) -> Result<Html, String> {
        let text = renderer.node(&input["value"])?;
        renderer.part("pill", &[("text", text)])
    }
}

/// 一時ディレクトリに、スキーマ ・ インスタンス ・ ページテンプレートを置く。
fn setup(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("sd-render-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("declarations")).unwrap();
    fs::create_dir_all(root.join("pages")).unwrap();
    let schema = json!({"x-view": {"label": "name"}, "type": "object", "required": ["kind", "id", "name"],
        "properties": {"$schema": {}, "kind": {"type": "string"}, "id": {"type": "string"},
                       "name": {"type": "string"}, "tags": {"type": "array", "title": "タグの一覧", "items": {"type": "string"}}}});
    fs::write(
        root.join("declarations/thing.schema.json"),
        schema.to_string(),
    )
    .unwrap();
    let write = |file: &str, instance: Value| {
        fs::write(
            root.join(file),
            serde_json::to_string_pretty(&instance).unwrap(),
        )
        .unwrap()
    };
    write(
        "declarations/T-1.json",
        json!({"$schema": "thing.schema.json", "kind": "thing", "id": "T-1", "name": "<甲>", "tags": ["a", "b"]}),
    );
    fs::write(
        root.join("declarations/document.schema.json"),
        include_str!("../../../references/document.schema.json"),
    )
    .unwrap();
    write(
        "declarations/D-1.json",
        json!({"$schema": "document.schema.json", "kind": "document", "id": "D-1", "title": "乙の報告",
               "sections": [{"heading": "結論", "blocks": [{"type": "paragraph", "text": "乙で進める", "strong": true}]}]}),
    );
    write(
        "pages/thing.json",
        json!({"kind": "thing", "sections": [
        {"heading": "タグ", "body": [{"component": "pill", "each": "this.tags", "value": "@"}]}]}),
    );
    root
}

fn args(root: &Path) -> Map<String, Value> {
    let path = |dir: &str| root.join(dir).to_string_lossy().into_owned();
    json!({"dir": path("declarations"), "pages": path("pages"), "out": path("out")})
        .as_object()
        .cloned()
        .unwrap()
}

fn render(root: &Path) -> (i32, Value) {
    render_with(root, Some(Arc::new(Mini)))
}

fn render_with(root: &Path, concrete: Option<Arc<dyn Design>>) -> (i32, Value) {
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let (instances, checks) = (
        Instances::new(files.clone(), files.clone(), query.clone()),
        Checks::new(files.clone(), files.clone(), query.clone()),
    );
    let renders = Renders::new(
        files.clone(),
        files,
        query,
        Arc::new(DocumentDesign),
        concrete,
        None,
    );
    Toolbox::base().with_render(Arc::new(renders)).dispatch(
        "render",
        &args(root),
        &instances,
        &checks,
    )
}

fn snapshot(root: &Path, dirs: &[&str]) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for dir in dirs {
        let mut list: Vec<_> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        list.sort();
        for path in list {
            out.push((
                path.to_string_lossy().into_owned(),
                fs::read(&path).unwrap(),
            ));
        }
    }
    out
}

#[test]
fn concrete_kinds_use_the_concrete_design_and_documents_the_fixed_design() {
    let root = setup("kinds");
    let (code, out) = render(&root);
    assert_eq!(code, 0, "{out}");
    let page = fs::read_to_string(root.join("out/T-1.html")).unwrap();
    assert_eq!(
        page,
        "<main><h1>&lt;甲&gt;</h1><section><h2>タグ</h2><b>a</b><b>b</b></section></main>"
    );
    let document = fs::read_to_string(root.join("out/D-1.html")).unwrap();
    assert!(
        document.contains("sd-doc") && document.contains("<h1>乙の報告</h1>"),
        "{document}"
    );
    let designs: Vec<(String, String)> = out["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| {
            (
                page["path"]
                    .as_str()
                    .unwrap()
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .to_owned(),
                page["design"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        designs,
        vec![
            ("D-1.html".to_owned(), "document".to_owned()),
            ("T-1.html".to_owned(), "concrete".to_owned())
        ]
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn kinds_without_a_template_stop_rendering_and_nothing_is_written() {
    let root = setup("missing");
    fs::write(
        root.join("declarations/O-1.json"),
        json!({"$schema": "thing.schema.json", "kind": "other", "id": "O-1", "name": "乙"})
            .to_string(),
    )
    .unwrap();
    let (code, out) = render(&root);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "デザインテンプレートが無い種類がある");
    assert!(
        out["detail"].as_str().unwrap().contains("kind other"),
        "{out}"
    );
    assert!(!root.join("out").exists());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn same_input_gives_same_pages_and_inputs_are_untouched() {
    let root = setup("same");
    render(&root);
    let first = snapshot(&root, &["declarations", "pages", "out"]);
    let (code, out) = render(&root);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["pages"][0]["changed"], false);
    assert_eq!(snapshot(&root, &["declarations", "pages", "out"]), first);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn validation_errors_stop_rendering_and_nothing_is_written() {
    let root = setup("invalid");
    fs::write(
        root.join("declarations/T-1.json"),
        json!({"$schema": "thing.schema.json", "kind": "thing", "id": "T-1", "name": 1})
            .to_string(),
    )
    .unwrap();
    let (code, out) = render(&root);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "検証を通過しないインスタンスがある");
    assert!(out["errors"][0]["property"]
        .as_str()
        .unwrap()
        .contains("T-1.json"));
    assert!(!root.join("out").exists());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn unfilled_properties_do_not_stop_rendering() {
    let root = setup("unfilled");
    fs::write(
        root.join("declarations/T-1.json"),
        json!({"$schema": "thing.schema.json", "kind": "thing", "id": "T-1"}).to_string(),
    )
    .unwrap();
    let (code, out) = render(&root);
    assert_eq!(code, 0, "{out}");
    assert!(root.join("out/T-1.html").exists());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn page_template_violations_stop_rendering() {
    let root = setup("template");
    fs::write(
        root.join("pages/thing.json"),
        json!({"kind": "thing", "sections": [
        {"heading": "h", "body": [{"component": "nope"}]}]})
        .to_string(),
    )
    .unwrap();
    let (code, out) = render(&root);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "ページテンプレートが契約を通過しない");
    assert!(out["detail"].as_str().unwrap().contains("nope"));
    assert!(!root.join("out").exists());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn without_any_concrete_design_documents_still_render() {
    let root = setup("document-only");
    fs::remove_file(root.join("declarations/T-1.json")).unwrap();
    let (code, out) = render_with(&root, None);
    assert_eq!(code, 0, "{out}");
    assert!(root.join("out/D-1.html").exists());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn document_blocks_render_by_information_type_and_escape_text() {
    let root = setup("blocks");
    fs::remove_file(root.join("declarations/T-1.json")).unwrap();
    let document = json!({"$schema": "document.schema.json", "kind": "document", "title": "<比較>", "lead": "前置き",
        "badges": ["2026-10-07"],
        "sections": [{"heading": "案", "blocks": [
            {"type": "table", "columns": ["案", "費用"], "rows": [["A", "小"], ["B", "大"]], "emphasis": 0},
            {"type": "steps", "items": [{"actor": "利用者", "text": "依頼する",
                "branches": [{"condition": "読めない", "steps": [{"text": "知らせる"}], "ending": "失敗"}]}]},
            {"type": "status", "items": [{"label": "3d", "state": "blocked", "note": "待ち"}]},
            {"type": "change", "rows": [{"what": "名前", "before": "l", "after": "instance"}]},
            {"type": "quote", "text": "A schema", "source": "json-schema-core:2057"},
            {"type": "bars", "items": [{"label": "甲", "value": 2}, {"label": "乙", "value": 4}]}]}]});
    fs::write(root.join("declarations/D-1.json"), document.to_string()).unwrap();
    let (code, out) = render_with(&root, None);
    assert_eq!(code, 0, "{out}");
    let page = fs::read_to_string(root.join("out/D-1.html")).unwrap();
    assert!(page.contains("<h1>&lt;比較&gt;</h1>"), "{page}");
    assert!(
        page.contains(r#"<tr class="sd-emphasis"><td>A</td><td>小</td></tr>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<span class="sd-actor">利用者</span>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<div class="sd-branch-when">読めない</div>"#),
        "{page}"
    );
    assert!(page.contains(r#"sd-state-blocked">止まっている"#), "{page}");
    assert!(
        page.contains(r#"<cite>json-schema-core:2057</cite>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"style="width:50%""#) && page.contains(r#"style="width:100%""#),
        "{page}"
    );
    assert!(
        page.contains(r#"<span class="sd-badge">2026-10-07</span>"#),
        "{page}"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn toolbox_without_a_render_use_case_says_why() {
    let root = setup("nodesign");
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let (instances, checks) = (
        Instances::new(files.clone(), files.clone(), query.clone()),
        Checks::new(files.clone(), files, query),
    );
    let (code, out) = Toolbox::base().dispatch("render", &args(&root), &instances, &checks);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "描画のユースケースが渡されていない");
    fs::remove_dir_all(&root).unwrap();
}

/// 文書を1件だけ置いて描画し、ページを返す（失敗なら Err に結果の JSON）。
fn render_document(name: &str, document: Value) -> Result<String, Value> {
    let root = setup(name);
    fs::remove_file(root.join("declarations/T-1.json")).unwrap();
    fs::write(root.join("declarations/D-1.json"), document.to_string()).unwrap();
    let (code, out) = render_with(&root, None);
    let result = if code == 0 {
        Ok(fs::read_to_string(root.join("out/D-1.html")).unwrap())
    } else {
        Err(out)
    };
    fs::remove_dir_all(&root).unwrap();
    result
}

fn document_with(blocks: Value) -> Value {
    json!({"$schema": "document.schema.json", "kind": "document", "title": "t",
           "sections": [{"heading": "h", "blocks": blocks}]})
}

#[test]
fn inline_markup_becomes_strong_code_and_links_and_bad_urls_are_refused() {
    let page = render_document(
        "inline",
        document_with(json!([{"type": "paragraph", "text": "最大 **8.2 秒**、`pg_stat_activity` を見る。<b>は文字。[根拠](#source-s1)"}])),
    )
    .unwrap();
    assert!(page.contains("<strong>8.2 秒</strong>"), "{page}");
    assert!(
        page.contains(r#"<code class="sd-code-inline">pg_stat_activity</code>"#),
        "{page}"
    );
    assert!(page.contains("&lt;b&gt;は文字"), "{page}");
    assert!(
        page.contains(r##"<a href="#source-s1">根拠</a>"##),
        "{page}"
    );
    let refused = render_document(
        "inline-bad",
        document_with(json!([{"type": "paragraph", "text": "[押す](javascript:alert(1))"}])),
    )
    .unwrap_err();
    assert!(
        refused["detail"].as_str().unwrap().contains("javascript:"),
        "{refused}"
    );
}

#[test]
fn table_cells_can_be_badges_and_columns_right_aligned() {
    let page = render_document(
        "cells",
        document_with(
            json!([{"type": "table", "columns": ["候補", {"label": "件数", "align": "right"}],
            "rows": [[{"text": "原因", "tone": "bad"}, "40"]]}]),
        ),
    )
    .unwrap();
    assert!(
        page.contains(r#"<span class="sd-cell-badge sd-tone-bad">原因</span>"#),
        "{page}"
    );
    assert!(page.contains(r#"<th class="sd-right">件数</th>"#), "{page}");
    assert!(page.contains(r#"<td class="sd-right">40</td>"#), "{page}");
}

#[test]
fn inline_svg_is_embedded_and_unsafe_svg_is_refused() {
    let page = render_document(
        "svg",
        document_with(json!([{"type": "figure", "svg": "<svg viewBox=\"0 0 1 1\"><rect width=\"1\" height=\"1\"/></svg>", "caption": "図"}])),
    )
    .unwrap();
    assert!(
        page.contains(r#"<div class="sd-svg"><svg viewBox="0 0 1 1">"#),
        "{page}"
    );
    let refused = render_document(
        "svg-bad",
        document_with(
            json!([{"type": "figure", "svg": "<svg onload=\"alert(1)\"></svg>", "caption": "図"}]),
        ),
    )
    .unwrap_err();
    assert!(
        refused["detail"]
            .as_str()
            .unwrap()
            .contains("イベントの属性"),
        "{refused}"
    );
}

#[test]
fn timeline_target_line_table_of_contents_and_sources() {
    let section =
        |heading: &str| json!({"heading": heading, "blocks": [{"type": "paragraph", "text": "x"}]});
    let mut document = document_with(json!([
        {"type": "timeline", "items": [{"time": "09:05", "text": "遅延が始まる"}]},
        {"type": "bars", "items": [{"label": "最大", "value": 8}], "target": {"label": "目標", "value": 2}}]));
    for heading in ["b", "c", "d"] {
        document["sections"]
            .as_array_mut()
            .unwrap()
            .push(section(heading));
    }
    document["sources"] =
        json!([{"id": "s1", "label": "監視の記録", "url": "https://example.com/dashboard"}]);
    let page = render_document("more", document).unwrap();
    assert!(
        page.contains(r#"<time class="sd-time">09:05</time>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"sd-bar-target"#) && page.contains(r#"style="left:25%""#),
        "{page}"
    );
    assert!(
        page.contains(r##"<li><a href="#section-4">d</a></li>"##),
        "{page}"
    );
    assert!(page.contains(r#"<li id="source-s1">監視の記録"#), "{page}");
}
