// SPDX-License-Identifier: MIT
//! 雛形の生成を事例で検証する。
//!
//!     cargo test -p nms_business_logic

use std::path::PathBuf;

use nms_business_logic::init::{self, Layer};

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
    let path = init::create(&root, "server", "go", &layers, &contract()).expect("置ける");
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
    let unit = &body["units"]["server"];
    assert_eq!(unit["root"], "server", "成果物の根は、渡した場所である");
    assert_eq!(unit["language"], "go");
    assert_eq!(unit["layers"][0]["name"], "core", "並びは内から外である");
    assert_eq!(
        unit["layers"][0]["where"][0], "internal/core",
        "識別子は成果物の根からの相対である"
    );
    assert_eq!(
        unit["layers"][1]["where"][0], "app.core",
        "識別子の形を問わない"
    );
    let rules = body["rules"].as_array().expect("配列");
    assert_eq!(rules.len(), 2, "内を指す規則を2件置く");
    assert_eq!(
        rules[0]["check"]["tool"].as_array().expect("配列").len(),
        0,
        "道具は空である"
    );
    assert_eq!(rules[0]["units"][0], "server", "規則は成果物の名前を持つ");
    assert!(
        rules[0]["check"].get("target").is_none(),
        "実行する場所は成果物の根なので、書かない"
    );
    assert_eq!(
        rules[1]["check"]["inward"], true,
        "依存の向きは、この Skill の inward が測る"
    );
}

#[test]
fn the_repository_itself_is_a_unit_rooted_at_the_top() {
    let root = scratch("repo");
    let layers = vec![Layer::new("skills".into(), ".claude/skills".into())];
    let path = init::create(&root, ".", "", &layers, &contract()).expect("置ける");
    let body: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("読める")).expect("JSON");
    assert_eq!(body["units"]["repository"]["root"], ".");
    assert_eq!(
        body["units"]["repository"]["layers"][0]["where"][0], ".claude/skills",
        "層はそのままである"
    );
    assert!(
        body["units"]["repository"].get("language").is_none(),
        "渡されなければ、言語を書かない"
    );
}

#[test]
fn an_existing_file_is_not_rebuilt() {
    let root = scratch("exists");
    let layers = vec![Layer::new("core".into(), "x".into())];
    init::create(&root, "server", "go", &layers, &contract()).expect("置ける");
    let again = init::create(&root, "server", "go", &layers, &contract());
    assert!(again.is_err(), "既に在れば断る ── 書いた規則が消える");
}

#[test]
fn no_layers_is_refused() {
    let root = scratch("nolayers");
    assert!(
        init::create(&root, "server", "go", &[], &contract()).is_err(),
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
    let body = init::skeleton(
        &contract(),
        &[Layer::new("core".into(), "x".into())],
        "",
        "",
    )
    .expect("組める");
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
