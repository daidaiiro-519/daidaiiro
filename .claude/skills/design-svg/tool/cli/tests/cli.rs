// SPDX-License-Identifier: MIT
//! design-svg の CLI の共通の決まりを、実行ファイルを起動して確かめる。**どの Skill の CLI も同じ決まりに従う**
//! ── 契約のテストケース（skills-creator の references/contract/cases/）が、同じ名前で同じことを確かめる。
//!
//!     cargo test -p ds_cli

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .display()
        .to_string()
}

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_design-svg"))
        .args(args)
        .output()
        .expect("起動できる");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn options_can_come_before_the_subcommand() {
    // **オプションはサブコマンドの前にも後ろにも置ける** ── 呼ぶ側が書き方を覚えなくてよい
    let root = root();
    let after = run(&[
        "get",
        "--kind",
        "view.tokens",
        "--skill_root",
        &root,
        "--json",
    ]);
    let before = run(&[
        "--kind",
        "view.tokens",
        "--skill_root",
        &root,
        "get",
        "--json",
    ]);
    assert_eq!(after.0, 0, "{}", after.1);
    assert_eq!(before, after);
}

#[test]
fn an_unknown_option_is_refused_with_exit_code_2() {
    // **道具の一覧に無いオプションは断る** ── 黙って無視すると、書き誤りに気づけない
    let root = root();
    let (code, _) = run(&[
        "get",
        "--kind",
        "view.tokens",
        "--no-such-option",
        "1",
        "--skill_root",
        &root,
        "--json",
    ]);
    assert_eq!(code, 2);
}

#[test]
fn without_a_subcommand_json_lists_the_tools() {
    // **サブコマンドを付けずに --json を付けると、ツールの一覧を返す** ── 検査はこれを読む
    let (code, out) = run(&["--json"]);
    assert_eq!(code, 0, "{out}");
    let doc: Value = serde_json::from_str(&out).expect("JSON である");
    assert_eq!(doc["ok"], Value::Bool(true));
    let names: Vec<&str> = doc["data"]["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for need in ["get", "validate", "view", "import"] {
        assert!(names.contains(&need), "{names:?}");
    }
}

fn svg_file(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join(format!("ds_cli_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("作れる");
    let p = dir.join(name);
    std::fs::write(&p, body).expect("書ける");
    p.display().to_string()
}

const AUTHORED: &str = r#"<svg viewBox="0 0 300 100"><rect class="box" x="10" y="20" width="130" height="40"/><text class="label" x="75" y="44" text-anchor="middle">注文</text><path class="flow" d="M140,40 H200"/></svg>"#;

#[test]
fn resolve_exits_0_when_nothing_is_found() {
    let (code, out) = run(&["resolve", &svg_file("ok.svg", AUTHORED), "--json"]);
    assert_eq!(code, 0, "{out}");
    let doc: Value = serde_json::from_str(&out).expect("JSON である");
    assert!(doc["data"]["svg"]
        .as_str()
        .is_some_and(|s| s.contains("<marker")));
}

#[test]
fn resolve_exits_1_when_something_is_found() {
    let bad = AUTHORED.replace(r#"<rect class="box""#, "<rect");
    let (code, out) = run(&["resolve", &svg_file("found.svg", &bad), "--json"]);
    assert_eq!(code, 1, "{out}");
}

#[test]
fn resolve_exits_2_on_misuse() {
    let (code, _) = run(&[
        "resolve",
        &svg_file("broken.svg", "<svg><rect></svg>"),
        "--json",
    ]);
    assert_eq!(code, 2);
}

#[test]
fn canvas_is_no_longer_a_subcommand() {
    // **canvas は削除した** ── 知らないサブコマンドとして誤用（終了コード 2）になる
    let (code, _) = run(&["canvas", "layers.json", "--json"]);
    assert_eq!(code, 2);
}
