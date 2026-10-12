//! ディレクトリのインスタンスの検査 ・ 承認と、参照が残るインスタンスの削除のテスト。
//! 公開 API の振り分けを、作業用のフォルダで呼んで確かめる。スキーマは Skill example の assets に置き、--skill で渡す。

mod support;

use serde_json::{json, Value};
use support::{call, eq, len, run, Scratch};

/// スキーマを置く Skill。
const SKILL: Option<&str> = Some("./skills/example");
/// インスタンスを置くディレクトリ。
const DIR: &str = "./declarations";
const APPROVAL: &str = "declarations/approval.json";

/// Skill example の assets のスキーマ（example/thing@v1）と、インスタンスを置くディレクトリ declarations を用意する。
fn workdir(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("check-{name}"));
    scratch.mkdir("declarations");
    scratch.put(
        "skills/example/assets/thing.schema.json",
        &json!({
            "$id": "example/thing@v1",
            "type": "object", "additionalProperties": false, "required": ["id", "kind", "name"],
            "properties": {
                "$schema": {"type": "string"}, "id": {"type": "string"}, "kind": {"const": "thing"},
                "name": {"type": "string", "minLength": 1},
                "refs": {"type": "array", "items": {"type": "string"}, "x-ref": {"to": "thing"}}
            }
        }),
    );
    scratch
}

/// thing のインスタンス id を置く。fields は書き足す欄。
fn put(scratch: &Scratch, id: &str, fields: Value) {
    let mut instance =
        json!({"$schema": "example/thing@v1", "id": id, "kind": "thing", "name": id});
    instance
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    scratch.put(&format!("declarations/{id}.json"), &instance);
}

/// name を持たない thing のインスタンス id を置く。
fn put_unnamed(scratch: &Scratch, id: &str) {
    scratch.put(
        &format!("declarations/{id}.json"),
        &json!({"$schema": "example/thing@v1", "id": id, "kind": "thing"}),
    );
}

// ── ディレクトリのインスタンスを検査する

#[test]
fn check_returns_validation_and_findings() {
    let dir = workdir("check-returns");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    let before = dir.read("declarations/A.json");
    run(
        &dir,
        SKILL,
        &[call(
            "check",
            &[("dir", DIR)],
            0,
            &[
                ("/instances", len(2)),
                (
                    "/findings",
                    eq(
                        json!([{"status": "合格", "check": "指す先がある", "from": "A/refs", "to": "B", "message": ""}]),
                    ),
                ),
            ],
        )],
    );
    assert_eq!(
        dir.read("declarations/A.json"),
        before,
        "インスタンスを書き換えない"
    );
}

#[test]
fn check_of_an_unreadable_instance_fails() {
    let dir = workdir("check-unreadable");
    put(&dir, "A", json!({}));
    dir.put_text("declarations/broken.json", "{");
    run(
        &dir,
        SKILL,
        &[call(
            "check",
            &[("dir", DIR)],
            1,
            &[("/reason", eq("JSON として読めない"))],
        )],
    );
}

#[test]
fn check_of_an_empty_directory_ends() {
    let dir = workdir("check-empty");
    run(
        &dir,
        SKILL,
        &[call(
            "check",
            &[("dir", DIR)],
            0,
            &[("/instances", eq(json!([]))), ("/findings", eq(json!([])))],
        )],
    );
}

// ── 承認を記録する

#[test]
fn approve_records_paths_and_hashes() {
    let dir = workdir("approve-records");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    run(
        &dir,
        SKILL,
        &[
            call(
                "approve",
                &[("dir", DIR)],
                0,
                &[
                    ("/changed", eq(true)),
                    ("/instances/0/path", eq("./declarations/A.json")),
                    ("/instances/1/path", eq("./declarations/B.json")),
                ],
            ),
            call(
                "check",
                &[("dir", DIR)],
                0,
                &[(
                    "/findings/0",
                    eq(
                        json!({"status": "合格", "check": "承認のあとの変化", "from": "A/refs", "to": "B", "message": "承認した時点から変わっていない"}),
                    ),
                )],
            ),
        ],
    );
    support::expect(
        "承認記録",
        &dir.read_json(APPROVAL),
        &[("/instances", len(2))],
    );
}

#[test]
fn approve_refuses_a_validation_error() {
    let dir = workdir("approve-invalid");
    put(&dir, "A", json!({"name": ""}));
    run(
        &dir,
        SKILL,
        &[call(
            "approve",
            &[("dir", DIR)],
            1,
            &[("/reason", eq("検証を通過しないインスタンスがある"))],
        )],
    );
    assert!(!dir.exists(APPROVAL), "拒んだら承認記録を書かない");
}

#[test]
fn approve_without_a_change_since_the_last_approval_ends() {
    let dir = workdir("approve-same");
    put(&dir, "A", json!({}));
    run(&dir, SKILL, &[call("approve", &[("dir", DIR)], 0, &[])]);
    let before = dir.read(APPROVAL);
    run(
        &dir,
        SKILL,
        &[call(
            "approve",
            &[("dir", DIR)],
            0,
            &[("/changed", eq(false))],
        )],
    );
    assert_eq!(dir.read(APPROVAL), before);
}

#[test]
fn approve_in_an_unwritable_directory_fails() {
    let dir = workdir("approve-unwritable");
    put(&dir, "A", json!({}));
    dir.set_readonly("declarations", true);
    let (code, out) = support::dispatch(&dir, SKILL, "approve", &[("dir", DIR)]);
    dir.set_readonly("declarations", false);
    support::expect(
        "approve",
        &json!({"code": code, "out": out}),
        &[("/code", eq(1)), ("/out/reason", eq("書けない"))],
    );
}

#[test]
fn approve_refuses_a_drift() {
    let dir = workdir("approve-drift");
    put(&dir, "A", json!({}));
    run(&dir, SKILL, &[call("approve", &[("dir", DIR)], 0, &[])]);
    let before = dir.read(APPROVAL);
    put(&dir, "A", json!({"refs": ["Z"]}));
    run(
        &dir,
        SKILL,
        &[call(
            "approve",
            &[("dir", DIR)],
            1,
            &[("/reason", eq("参照と導出値のずれがある"))],
        )],
    );
    assert_eq!(dir.read(APPROVAL), before, "拒んだら前の承認記録のまま");
}

#[test]
fn approve_refuses_an_unfilled_property() {
    let dir = workdir("approve-unfilled");
    put_unnamed(&dir, "A");
    run(
        &dir,
        SKILL,
        &[call(
            "approve",
            &[("dir", DIR)],
            1,
            &[("/reason", eq("未記入のプロパティがある"))],
        )],
    );
}

// ── インスタンスを削除する

#[test]
fn delete_reports_remaining_references() {
    let dir = workdir("delete-remaining");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    run(
        &dir,
        SKILL,
        &[call(
            "delete",
            &[("path", "./declarations/B.json")],
            0,
            &[
                ("/remaining", len(1)),
                ("/remaining/0/from", eq("A/refs")),
                ("/remaining/0/status", eq("ずれ")),
            ],
        )],
    );
    assert!(!dir.exists("declarations/B.json"));
}

#[test]
fn delete_of_a_missing_instance_fails() {
    let dir = workdir("delete-missing");
    run(
        &dir,
        SKILL,
        &[call(
            "delete",
            &[("path", "./declarations/none.json")],
            1,
            &[("/reason", eq("読めない"))],
        )],
    );
}

#[test]
fn delete_in_an_unwritable_directory_fails() {
    let dir = workdir("delete-unwritable");
    put(&dir, "B", json!({}));
    dir.set_readonly("declarations", true);
    let (code, out) =
        support::dispatch(&dir, SKILL, "delete", &[("path", "./declarations/B.json")]);
    dir.set_readonly("declarations", false);
    support::expect(
        "delete",
        &json!({"code": code, "out": out}),
        &[("/code", eq(1)), ("/out/reason", eq("書けない"))],
    );
    assert!(dir.exists("declarations/B.json"));
}

#[test]
fn delete_fails_when_the_directory_is_unreadable_afterwards() {
    let dir = workdir("delete-broken-dir");
    put(&dir, "B", json!({}));
    dir.put_text("declarations/broken.json", "{");
    run(
        &dir,
        SKILL,
        &[call(
            "delete",
            &[("path", "./declarations/B.json")],
            1,
            &[("/reason", eq("JSON として読めない"))],
        )],
    );
}
