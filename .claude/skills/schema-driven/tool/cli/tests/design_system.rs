//! デザインテンプレートを持つものは、デザインシステムを持つ（ボード board-on-schema-driven の論点5）。
//! 基盤は、デザインシステム（トークンの3層と部品ごとの状態）を検査し、トークンから CSS の変数を作る。
//! 契約を満たさないデザインでは、1ページも描画しない。

use schema_driven_adapters::inbound::tools::Toolbox;
use schema_driven_adapters::outbound::document_design::DocumentDesign;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use schema_driven_core::domain::Html;
use schema_driven_core::ports::outbound::{Design, Frame, Renderer};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// テスト用の具体のデザイン。テンプレートとデザインシステムを差し替えられる。
struct Probe {
    parts: String,
    system: String,
}

fn parts(pill_style: &str) -> String {
    format!(
        r#"<template id="page"><style>{{{{tokens}}}}</style><main><h1>{{{{title}}}}</h1>{{{{sections}}}}</main></template>
<template id="section"><section><h2>{{{{heading}}}}</h2>{{{{body}}}}</section></template>
<template id="pill"><b style="{pill_style}">{{{{text}}}}</b></template>"#
    )
}

fn system() -> Value {
    json!({
        "scope": ":root",
        "tokens": {
            "base": {"color": {"teal-700": "#0d5c55", "teal-300": "#6cc9b8", "paper-100": "#eef1ef", "night-900": "#0e1614"}},
            "semantic": {"light": {"bg": "paper-100", "key": "teal-700"}, "dark": {"bg": "night-900", "key": "teal-300"}},
            "component": {"pill-bg": "key"}
        },
        "components": [{"name": "pill", "states": {"default": {"background": "pill-bg", "text": "bg"}}}]
    })
}

impl Design for Probe {
    fn components(&self) -> Vec<(String, Value)> {
        vec![(
            "pill".into(),
            json!({"type": "object", "required": ["value"]}),
        )]
    }
    fn parts(&self) -> &str {
        &self.parts
    }
    fn design_system(&self) -> &str {
        &self.system
    }
    fn page(&self, frame: &Frame, renderer: &dyn Renderer) -> Result<Html, String> {
        renderer.part(
            "page",
            &[
                ("tokens", renderer.tokens()),
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

fn probe(pill_style: &str, system: Value) -> Arc<dyn Design> {
    Arc::new(Probe {
        parts: parts(pill_style),
        system: system.to_string(),
    })
}

fn setup(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("sd-design-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("declarations")).unwrap();
    fs::create_dir_all(root.join("pages")).unwrap();
    let write = |file: &str, value: Value| fs::write(root.join(file), value.to_string()).unwrap();
    write(
        "declarations/thing.schema.json",
        json!({"type": "object", "required": ["kind", "id"],
               "properties": {"$schema": {}, "kind": {"type": "string"}, "id": {"type": "string"}, "tags": {"type": "array", "items": {"type": "string"}}}}),
    );
    write(
        "declarations/T-1.json",
        json!({"$schema": "thing.schema.json", "kind": "thing", "id": "T-1", "tags": ["a"]}),
    );
    write(
        "pages/thing.json",
        json!({"kind": "thing", "sections": [{"heading": "タグ", "body": [{"component": "pill", "each": "this.tags", "value": "@"}]}]}),
    );
    root
}

fn render(root: &Path, design: Arc<dyn Design>) -> (i32, Value) {
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
        Some(design),
        None,
    );
    let path = |dir: &str| root.join(dir).to_string_lossy().into_owned();
    let args: Map<String, Value> =
        json!({"dir": path("declarations"), "pages": path("pages"), "out": path("out")})
            .as_object()
            .cloned()
            .unwrap();
    Toolbox::base()
        .with_render(Arc::new(renders))
        .dispatch("render", &args, &instances, &checks)
}

fn refused(root: &Path, design: Arc<dyn Design>, expect: &str) {
    let (code, out) = render(root, design);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "デザインシステムが契約を満たさない", "{out}");
    assert!(
        out["detail"].as_str().unwrap().contains(expect),
        "{expect} が無い：{out}"
    );
    assert!(!root.join("out").exists(), "1ページも書かない");
}

#[test]
fn tokens_become_css_variables_for_light_and_both_dark_settings() {
    let root = setup("css");
    let (code, out) = render(&root, probe("background:var(--pill-bg)", system()));
    assert_eq!(code, 0, "{out}");
    let page = fs::read_to_string(root.join("out/T-1.html")).unwrap();
    for expected in [
        ":root{--bg:#eef1ef;--key:#0d5c55;--pill-bg:var(--key)}",
        r#"@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){--bg:#0e1614;--key:#6cc9b8;color-scheme:dark}}"#,
        r#":root[data-theme="dark"]{--bg:#0e1614;--key:#6cc9b8;color-scheme:dark}"#,
    ] {
        assert!(page.contains(expected), "{expected}\n---\n{page}");
    }
}

#[test]
fn a_literal_color_in_the_template_stops_rendering() {
    refused(
        &setup("literal"),
        probe("background:#ff0000", system()),
        "色の直値",
    );
}

#[test]
fn a_variable_the_tokens_do_not_define_stops_rendering() {
    refused(
        &setup("undefined"),
        probe("background:var(--nope)", system()),
        "--nope",
    );
}

#[test]
fn a_semantic_token_must_point_at_a_base_token() {
    let mut broken = system();
    broken["tokens"]["semantic"]["light"]["key"] = json!("teal-999");
    refused(
        &setup("semantic"),
        probe("background:var(--pill-bg)", broken),
        "teal-999",
    );
}

#[test]
fn light_and_dark_must_have_the_same_keys() {
    let mut broken = system();
    broken["tokens"]["semantic"]["dark"]
        .as_object_mut()
        .unwrap()
        .remove("bg");
    refused(
        &setup("keys"),
        probe("background:var(--pill-bg)", broken),
        "bg",
    );
}

#[test]
fn every_component_has_its_states() {
    let mut broken = system();
    broken["components"] = json!([]);
    refused(
        &setup("states"),
        probe("background:var(--pill-bg)", broken),
        "pill",
    );
}

#[test]
fn a_state_must_use_a_defined_token() {
    let mut broken = system();
    broken["components"][0]["states"]["pressed"] = json!({"background": "warn"});
    refused(
        &setup("state-token"),
        probe("background:var(--pill-bg)", broken),
        "warn",
    );
}

#[test]
fn the_base_document_design_follows_its_own_design_system() {
    let root = setup("document");
    fs::remove_file(root.join("declarations/T-1.json")).unwrap();
    fs::write(
        root.join("declarations/document.schema.json"),
        include_str!("../../../references/document.schema.json"),
    )
    .unwrap();
    fs::write(
        root.join("declarations/D-1.json"),
        json!({"$schema": "document.schema.json", "kind": "document", "id": "D-1", "title": "乙の報告",
               "sections": [{"heading": "結論", "blocks": [{"type": "paragraph", "text": "乙で進める"}]}]})
        .to_string(),
    )
    .unwrap();
    let (code, out) = render(&root, probe("background:var(--pill-bg)", system()));
    assert_eq!(code, 0, "{out}");
    let page = fs::read_to_string(root.join("out/D-1.html")).unwrap();
    assert!(
        page.contains(".sd-doc{--sd-") && page.contains("--sd-bg:#f7f8f7"),
        "{page}"
    );
    assert!(
        page.contains(r#":root[data-theme="dark"] .sd-doc{"#),
        "{page}"
    );
}
