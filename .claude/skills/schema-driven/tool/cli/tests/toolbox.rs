//! ツールの一覧に、具体が自分のツールを追加する（3c。ボード schema-driven-build の論点5 C）。
//! 引数の検査は基盤が行い、基盤のツールと同じ名前は追加できない。

use schema_driven_adapters::inbound::tools::{ExtraTools, ToolDef, Toolbox, TOOLS};
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use serde_json::{json, Map, Value};

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

fn args(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap()
}

#[test]
fn concrete_tools_follow_base_tools_and_are_dispatched() {
    let (files, query) = (FileSystem, Jmespath);
    let (uc, cc) = (
        Instances::new(&files, &files, &query),
        Checks::new(&files, &files, &query),
    );
    let seal = Seal;
    let tb = Toolbox::with(&seal).unwrap();
    let names: Vec<&str> = tb.list().iter().map(|t| t.name).collect();
    let mut want: Vec<&str> = TOOLS.iter().map(|t| t.name).collect();
    want.push("seal");
    assert_eq!(names, want);
    let (code, out) = tb.dispatch("seal", &args(json!({"dir": "d"})), &uc, &cc);
    assert_eq!(
        (code, out),
        (0, json!({"ok": true, "tool": "seal", "dir": "d"}))
    );
    // 基盤のツールは、基盤がそのまま処理する
    let (code, out) = tb.dispatch("get", &args(json!({})), &uc, &cc);
    assert_eq!(code, 2);
    assert!(out["detail"]
        .as_str()
        .unwrap()
        .contains("get の引数 path が無い"));
}

#[test]
fn base_checks_arguments_of_concrete_tools() {
    let (files, query) = (FileSystem, Jmespath);
    let (uc, cc) = (
        Instances::new(&files, &files, &query),
        Checks::new(&files, &files, &query),
    );
    let seal = Seal;
    let tb = Toolbox::with(&seal).unwrap();
    let (code, out) = tb.dispatch("seal", &args(json!({"dir": "d", "x": "1"})), &uc, &cc);
    assert_eq!(code, 2);
    assert!(out["detail"]
        .as_str()
        .unwrap()
        .contains("seal は引数 x を受け付けない"));
    let (code, _) = tb.dispatch("seal", &args(json!({})), &uc, &cc);
    assert_eq!(code, 2);
    let (code, out) = tb.dispatch("nope", &args(json!({})), &uc, &cc);
    assert_eq!(code, 2);
    assert!(out["tools"].as_array().unwrap().contains(&json!("seal")));
}

#[test]
fn concrete_cannot_reuse_a_base_tool_name() {
    let clash = Clash;
    assert!(Toolbox::with(&clash).is_err());
}
