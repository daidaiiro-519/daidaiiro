// SPDX-License-Identifier: MIT
//! テストケース（契約の正解の出力）を実行する段を、事例で検証する（ボード skills-creator-contract の論点2 ・ 3）。
//!
//!     cargo test -p sc_business_logic --test cases

use std::path::{Path, PathBuf};

use sc_business_logic::cases;
use serde_json::json;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-cases-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

fn write(path: &Path, body: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).expect("作れる");
    }
    std::fs::write(path, body).expect("書ける");
}

#[test]
fn ignored_pointers_are_left_out_of_the_comparison() {
    // **比較から除外する箇所は JSON Pointer で書く** ── スキーマ検査のエラー文はライブラリごとに違う
    let want = json!({"ok": false, "findings": ["a ── jsonschema の文"], "data": {"checked": "x"}});
    let have =
        json!({"ok": false, "findings": ["a ── 別のライブラリの文"], "data": {"checked": "x"}});
    assert!(cases::compare(&want, &have, &[]).is_some());
    assert_eq!(
        cases::compare(&want, &have, &["/findings".to_owned()]),
        None
    );
}

#[test]
fn the_first_differing_path_is_reported() {
    let want = json!({"data": {"html": "<p>a</p>", "n": [1, 2]}});
    let have = json!({"data": {"html": "<p>a</p>", "n": [1, 3]}});
    assert_eq!(
        cases::compare(&want, &have, &[]),
        Some("/data/n/1".to_owned())
    );
}

#[test]
fn every_test_of_the_reference_needs_a_case() {
    // **リファレンス実装のテスト1件ごとに、同名のテストケースを置く** ── 例外は理由を付けて置く
    let source = "#[test]\nfn one_is_drawn() {}\n#[test]\nfn two_is_taken() {}\nfn helper() {}\n#[test]\nfn three_is_internal() {}\n";
    let dir = scratch("map");
    write(
        &dir.join("one_is_drawn.json"),
        r#"{"case":"1","test":"one_is_drawn","call":["get"],"expect":{"exit":0}}"#,
    );
    write(
        &dir.join("orphan.json"),
        r#"{"case":"x","test":"no_such_test","call":["get"],"expect":{"exit":0}}"#,
    );
    write(
        &dir.join("exempt.json"),
        r#"[{"test":"three_is_internal","reason":"CLI から観察できない"}]"#,
    );
    let found = cases::unmatched(source, &dir).expect("読める");
    assert!(
        found.iter().any(|f| f.contains("two_is_taken")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|f| f.contains("no_such_test")),
        "{found:?}"
    );
    assert!(
        !found
            .iter()
            .any(|f| f.contains("helper") || f.contains("three_is_internal")),
        "{found:?}"
    );
}

/// 引数をそのまま JSON にして返す偽の CLI を置き、tool.json から呼べるようにする。
fn fake_skill(dir: &Path) -> PathBuf {
    let skill = dir.join("skill");
    let script = skill.join("bin/fake");
    write(
        &script,
        "#!/bin/sh\nif [ \"$1\" = boom ]; then echo '{\"ok\":false}'; exit 2; fi\nprintf '{\"ok\":true,\"findings\":[],\"data\":{\"verb\":\"%s\",\"here\":%s}}' \"$1\" \"$(ls fixtures | wc -l)\"\n",
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("権限");
    }
    write(
        &skill.join("tool.json"),
        &format!(
            r#"{{"contract":2,"cli":{{"command":"{}","args":[]}},"external":[]}}"#,
            script.display()
        ),
    );
    skill
}

#[test]
fn cases_are_run_through_the_command_in_tool_json() {
    // **tool.json の実行コマンドで呼ぶ** ── どの言語で書いた Skill にも、同じテストケースを実行できる
    let dir = scratch("run");
    let skill = fake_skill(&dir);
    let cs = dir.join("cases");
    write(&cs.join("fixtures/a.txt"), "a");
    write(
        &cs.join("pass.json"),
        r#"{"case":"合格","test":"t1","call":["get"],"expect":{"exit":0,"json":{"ok":true,"findings":[],"data":{"verb":"get","here":1}}}}"#,
    );
    write(
        &cs.join("wrong.json"),
        r#"{"case":"出力が違う","test":"t2","call":["view"],"expect":{"exit":0,"json":{"ok":true,"findings":[],"data":{"verb":"get","here":1}}}}"#,
    );
    write(
        &cs.join("exit.json"),
        r#"{"case":"終了コードが違う","test":"t3","call":["boom"],"expect":{"exit":0}}"#,
    );
    let results = cases::run(&skill, &cs).expect("実行できる");
    let by: std::collections::BTreeMap<_, _> = results.into_iter().collect();
    assert_eq!(by["pass"], None);
    assert!(
        by["wrong"]
            .as_deref()
            .is_some_and(|w| w.contains("/data/verb")),
        "{by:?}"
    );
    assert!(
        by["exit"]
            .as_deref()
            .is_some_and(|w| w.contains("終了コード 2")),
        "{by:?}"
    );
}

#[test]
fn each_case_runs_in_its_own_copy() {
    // **テストケースごとに、フォルダの複製で実行する** ── import のように書き換える呼び出しが在る
    let dir = scratch("copy");
    let skill = fake_skill(&dir);
    let cs = dir.join("cases");
    write(&cs.join("fixtures/a.txt"), "a");
    write(
        &cs.join("a.json"),
        r#"{"case":"setup で増やす","test":"t1","setup":[["touch"]],"call":["get"],"expect":{"exit":0}}"#,
    );
    let results = cases::run(&skill, &cs).expect("実行できる");
    assert_eq!(results, vec![("a".to_owned(), None)]);
    assert!(
        !cs.join("fixtures/b.txt").exists(),
        "元のフォルダは書き換えない"
    );
}

fn contract() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references/contract")
}

#[test]
fn the_contract_files_match_their_schemas() {
    // **契約のファイルは、それぞれのスキーマに合う**（references の直下ではないので、ここで検査する）
    for kind in ["structure", "tools", "assumptions"] {
        let found = sc_business_logic::refs::validate_file(
            &contract(),
            kind,
            &contract().join(format!("{kind}.json")),
        )
        .expect("読める");
        assert!(found.is_empty(), "{kind}: {found:?}");
    }
}

#[test]
fn every_assumption_points_at_existing_cases() {
    // **実装上の前提は、確認するテストケースを指す** ── 指す先が無い前提は、検証されていない
    let body = std::fs::read_to_string(contract().join("assumptions.json")).expect("読める");
    let v: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    for item in v["items"].as_array().expect("並び") {
        let cases = item["cases"].as_array().expect("並び");
        assert!(
            !cases.is_empty(),
            "{}: 確認するテストケースが無い",
            item["id"]
        );
        for c in cases {
            let name = c.as_str().expect("文字列");
            assert!(
                contract().join(format!("cases/{name}.json")).is_file(),
                "{}: テストケース {name} が無い",
                item["id"]
            );
        }
    }
}
