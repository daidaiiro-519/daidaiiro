// SPDX-License-Identifier: MIT
//! 雛形の生成を事例で検証する。
//!
//!     cargo test -p nms_parts

use std::path::PathBuf;

use nms_parts::init::{self, Layer};

fn contract() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references/rules.schema.json")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nms-init-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

#[test]
fn the_rules_file_goes_to_one_place_in_the_repository() {
    let root = scratch("place");
    std::fs::create_dir_all(root.join("server")).expect("作れる");
    let layers = vec![
        Layer::new("core".into(), "internal/core".into()),
        Layer::new("app".into(), "app.core".into()),
    ];
    let path = init::create(&root, "server", &layers, &contract()).expect("置ける");
    assert_eq!(
        path,
        root.join(".coding-rules").join("rules.json"),
        "リポジトリに1つ置く"
    );
    assert!(
        !root.join("server/.coding-rules").exists(),
        "成果物の中に作らない"
    );
    let body: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("読める")).expect("JSON");
    assert_eq!(body["order"][0], "core");
    assert_eq!(
        body["layers"]["core"], "server/internal/core",
        "層は成果物からの経路になる"
    );
    assert_eq!(
        body["layers"]["app"], "server/app.core",
        "識別子の形を問わない"
    );
    assert_eq!(
        body["rules"].as_array().expect("配列").len(),
        2,
        "内を指す規則を2件置く"
    );
    assert_eq!(
        body["rules"][0]["check"]["tool"]
            .as_array()
            .expect("配列")
            .len(),
        0,
        "道具は空である"
    );
    assert_eq!(
        body["rules"][0]["check"]["target"], "server",
        "実行する場所は成果物である"
    );
}

#[test]
fn the_repository_itself_needs_no_target() {
    let root = scratch("repo");
    let layers = vec![Layer::new("skills".into(), ".claude/skills".into())];
    let path = init::create(&root, ".", &layers, &contract()).expect("置ける");
    let body: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("読める")).expect("JSON");
    assert_eq!(
        body["layers"]["skills"], ".claude/skills",
        "層はそのままである"
    );
    assert!(
        body["rules"][0]["check"].get("target").is_none(),
        "実行する場所を書かない"
    );
}

#[test]
fn an_existing_file_is_not_rebuilt() {
    let root = scratch("exists");
    let layers = vec![Layer::new("core".into(), "x".into())];
    init::create(&root, "server", &layers, &contract()).expect("置ける");
    let again = init::create(&root, "server", &layers, &contract());
    assert!(again.is_err(), "既に在れば断る ── 書いた規則が消える");
}

#[test]
fn no_layers_is_refused() {
    let root = scratch("nolayers");
    assert!(
        init::create(&root, "server", &[], &contract()).is_err(),
        "層が無ければ断る"
    );
}

#[test]
fn a_layer_is_a_name_and_an_identifier() {
    let got = init::parse_layers(&["core=internal/core".to_owned()]).expect("読める");
    assert_eq!(got, vec![Layer::new("core".into(), "internal/core".into())]);
    for bad in ["core", "=x", "core="] {
        assert!(
            init::parse_layers(&[bad.to_owned()]).is_err(),
            "形が違えば断る ── {bad}"
        );
    }
}

#[test]
fn every_item_of_the_contract_appears_in_the_skeleton() {
    // **項目の一覧を、この側に書かない** ── 契約から組む
    let body =
        init::skeleton(&contract(), &[Layer::new("core".into(), "x".into())], "").expect("組める");
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(contract()).expect("読める")).expect("JSON");
    let shape = schema["$defs"]["rule"]["properties"]
        .as_object()
        .expect("形");
    let rule = body["rules"][0].as_object().expect("規則");
    for key in shape.keys() {
        assert!(rule.contains_key(key), "契約の {key} が雛形に無い");
    }
}
