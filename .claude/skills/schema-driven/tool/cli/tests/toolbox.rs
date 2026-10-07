//! ツールの一覧に、具体が自分のツールを追加する（3c。ボード schema-driven-build の論点5 C）。
//! 引数の検査は基盤が行い、基盤のツールと同じ名前は追加できない。

use schema_driven_adapters::inbound::tools::{ExtraTools, ToolDef, Toolbox, TOOLS};
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use serde_json::{json, Map, Value};
use std::sync::Arc;

/// テスト用の具体のツール（acdr の seal のようなもの）。
struct Seal;

impl ExtraTools for Seal {
    fn tools(&self) -> Vec<ToolDef> {
        vec![ToolDef {
            name: "seal",
            description: "承認時点のハッシュで封印する",
            args: &[("dir", "記録のディレクトリ", true)],
        }]
    }
    fn call(&self, name: &str, args: &Map<String, Value>) -> (i32, Value) {
        (0, json!({"ok": true, "tool": name, "dir": args["dir"]}))
    }
}

struct Clash;

impl ExtraTools for Clash {
    fn tools(&self) -> Vec<ToolDef> {
        vec![ToolDef {
            name: "check",
            description: "基盤と同じ名前",
            args: &[],
        }]
    }
    fn call(&self, _: &str, _: &Map<String, Value>) -> (i32, Value) {
        (0, Value::Null)
    }
}

fn args(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap()
}

#[test]
fn concrete_tools_follow_base_tools_and_are_dispatched() {
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let (instances, checks) = (
        Instances::new(files.clone(), files.clone(), query.clone()),
        Checks::new(files.clone(), files, query),
    );
    let toolbox = Toolbox::with(Arc::new(Seal)).unwrap();
    let names: Vec<&str> = toolbox.list().iter().map(|tool| tool.name).collect();
    let mut want: Vec<&str> = TOOLS.iter().map(|tool| tool.name).collect();
    want.push("seal");
    assert_eq!(names, want);
    let (code, out) = toolbox.dispatch("seal", &args(json!({"dir": "d"})), &instances, &checks);
    assert_eq!(
        (code, out),
        (0, json!({"ok": true, "tool": "seal", "dir": "d"}))
    );
    // 基盤のツールは、基盤がそのまま処理する
    let (code, out) = toolbox.dispatch("get", &args(json!({})), &instances, &checks);
    assert_eq!(code, 2);
    assert!(out["detail"]
        .as_str()
        .unwrap()
        .contains("get の引数 path が無い"));
}

#[test]
fn base_checks_arguments_of_concrete_tools() {
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let (instances, checks) = (
        Instances::new(files.clone(), files.clone(), query.clone()),
        Checks::new(files.clone(), files, query),
    );
    let toolbox = Toolbox::with(Arc::new(Seal)).unwrap();
    let (code, out) = toolbox.dispatch(
        "seal",
        &args(json!({"dir": "d", "x": "1"})),
        &instances,
        &checks,
    );
    assert_eq!(code, 2);
    assert!(out["detail"]
        .as_str()
        .unwrap()
        .contains("seal は引数 x を受け付けない"));
    let (code, _) = toolbox.dispatch("seal", &args(json!({})), &instances, &checks);
    assert_eq!(code, 2);
    let (code, out) = toolbox.dispatch("nope", &args(json!({})), &instances, &checks);
    assert_eq!(code, 2);
    assert!(out["tools"].as_array().unwrap().contains(&json!("seal")));
}

#[test]
fn concrete_cannot_reuse_a_base_tool_name() {
    assert!(Toolbox::with(Arc::new(Clash)).is_err());
}
