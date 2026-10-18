//! インスタンスの作成 ・ 取得 ・ 更新 ・ 削除と、x-prompt の受け取りのテスト。
//! 公開 API の振り分けを、作業用のフォルダで呼んで確かめる。スキーマは Skill example の assets に置き、--skill で渡す。

mod support;

use serde_json::{json, Value};
use support::{call, eq, run, Is, Scratch};

/// スキーマを置く Skill。
const SKILL: Option<&str> = Some("./skills/example");
/// thing のスキーマ。
const THING: &str = "./skills/example/assets/thing.schema.json";
/// 作るインスタンス。
const A: &str = "./data/a.json";

/// 作業用のフォルダ。Skill example の assets にスキーマ（$ref で同じ Skill のスキーマを名前で指す）を置いておく。
fn workdir(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("instances-{name}"));
    scratch.put(
        "skills/example/assets/thing.schema.json",
        &json!({
            "$id": "example/thing@v1",
            "type": "object",
            "additionalProperties": false,
            "required": ["name", "items"],
            "properties": {
                "$schema": {"type": "string"},
                "name": {"type": "string", "minLength": 1,
                         "x-prompt": {"read": "名前として読む", "write": "名前を1語で書く"}},
                "items": {"type": "array", "items": {"$ref": "common@v1#/$defs/item"},
                          "x-prompt": {"read": "項目の一覧", "write": "項目を1件ずつ書く"}},
                "note": {"type": "string"}
            }
        }),
    );
    scratch.put(
        "skills/example/assets/common.schema.json",
        &json!({"$id": "example/common@v1", "$defs": {"item": {"type": "string", "minLength": 1}}}),
    );
    scratch
}

/// thing から A を作る行。
fn create<'row>(code: i32, expect: &'row [(&'row str, Is)]) -> support::Call<'row> {
    call("create", &[("schema", THING), ("path", A)], code, expect)
}

/// 必須の欄を埋める JSON Patch。
const FILL: &str =
    r#"[{"op":"add","path":"/name","value":"a"},{"op":"add","path":"/items","value":["x"]}]"#;
/// name を書く JSON Patch。
const NAME: &str = r#"[{"op":"add","path":"/name","value":"a"}]"#;

fn saved(scratch: &Scratch) -> Value {
    scratch.read_json(A)
}

// ── インスタンスの作成 ・ 更新 ・ 削除と、ハッシュ

#[test]
fn create_writes_the_schema_name_and_returns_the_path() {
    let dir = workdir("create-name");
    run(&dir, SKILL, &[create(0, &[("/path", support::eq(A))])]);
    assert_eq!(
        saved(&dir)["$schema"],
        "example/thing@v1",
        "スキーマの名前を書く"
    );
}

#[test]
fn create_refuses_an_existing_path() {
    let dir = workdir("create-exists");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            create(1, &[("/reason", eq("パスにインスタンスが既にある"))]),
        ],
    );
}

#[test]
fn update_applies_the_patch() {
    let dir = workdir("update-applies");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call("update", &[("path", A), ("patch", NAME)], 0, &[]),
        ],
    );
    assert_eq!(saved(&dir)["name"], "a");
}

#[test]
fn update_reads_a_long_patch_from_a_file() {
    let dir = workdir("update-patch-file");
    dir.put_text("data/name.patch.json", NAME);
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call(
                "update",
                &[("path", A), ("patch_file", "./data/name.patch.json")],
                0,
                &[],
            ),
        ],
    );
    assert_eq!(saved(&dir)["name"], "a");
}

#[test]
fn update_takes_either_patch_or_patch_file_but_not_both() {
    let dir = workdir("update-patch-one");
    dir.put_text("data/name.patch.json", NAME);
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call("update", &[("path", A)], 2, &[]),
            call(
                "update",
                &[
                    ("path", A),
                    ("patch", NAME),
                    ("patch_file", "./data/name.patch.json"),
                ],
                2,
                &[],
            ),
            call(
                "update",
                &[("path", A), ("patch_file", "./data/missing.json")],
                1,
                &[("/reason", eq("読めない"))],
            ),
        ],
    );
}

#[test]
fn update_refuses_a_stale_hash() {
    let dir = workdir("update-stale");
    let stale = "0".repeat(64);
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call(
                "update",
                &[("path", A), ("patch", NAME), ("hash", &stale)],
                1,
                &[("/reason", eq("ほかの更新と競合した"))],
            ),
        ],
    );
}

#[test]
fn delete_removes_the_file() {
    let dir = workdir("delete");
    run(
        &dir,
        SKILL,
        &[create(0, &[]), call("delete", &[("path", A)], 0, &[])],
    );
    assert!(!dir.exists(A));
}

/// ハッシュ値は、書いたファイルの中身の SHA-256 である。期待値は、中身の文字から前もって計算した値。
#[test]
fn hash_is_the_sha256_of_the_content() {
    let dir = workdir("hash");
    run(
        &dir,
        SKILL,
        &[create(
            0,
            &[(
                "/hash",
                eq("873fc93d3884644a0795964f3109130e0d9a12f8821b318361eeee096b3b0aa8"),
            )],
        )],
    );
    assert_eq!(dir.read(A), r#"{"$schema":"example/thing@v1"}"#);
    run(
        &dir,
        SKILL,
        &[call(
            "update",
            &[("path", A), ("patch", NAME)],
            0,
            &[(
                "/hash",
                eq("8a78f479cb1c9f4c9f75469473e6519991dda52a03c242edfcbfa4f70ccdb695"),
            )],
        )],
    );
    assert_eq!(dir.read(A), r#"{"$schema":"example/thing@v1","name":"a"}"#);
}

// ── インスタンスを作成する

#[test]
fn create_returns_unfilled_properties_with_their_prompt() {
    let dir = workdir("create-unfilled");
    run(
        &dir,
        SKILL,
        &[create(
            0,
            &[
                ("/unfilled/0/property", eq("/items")),
                ("/unfilled/1/property", eq("/name")),
                ("/unfilled", support::len(2)),
                ("/unfilled/1/prompt/write", eq("名前を1語で書く")),
            ],
        )],
    );
    assert!(dir.exists(A));
}

#[test]
fn create_from_an_unreadable_schema_fails_without_a_file() {
    let dir = workdir("create-unreadable");
    dir.put_text("skills/example/assets/broken.schema.json", "{");
    run(
        &dir,
        SKILL,
        &[
            call(
                "create",
                &[
                    ("schema", "./skills/example/assets/none.schema.json"),
                    ("path", A),
                ],
                1,
                &[("/reason", eq("読めない"))],
            ),
            call(
                "create",
                &[
                    ("schema", "./skills/example/assets/broken.schema.json"),
                    ("path", A),
                ],
                1,
                &[("/reason", eq("JSON として読めない"))],
            ),
        ],
    );
    assert!(!dir.exists(A));
}

#[test]
fn create_keeps_an_existing_instance() {
    let dir = workdir("create-keeps");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call("update", &[("path", A), ("patch", FILL)], 0, &[]),
        ],
    );
    let before = dir.read(A);
    run(
        &dir,
        SKILL,
        &[create(
            1,
            &[("/reason", eq("パスにインスタンスが既にある"))],
        )],
    );
    assert_eq!(dir.read(A), before);
}

#[test]
fn create_in_an_unwritable_directory_fails() {
    let dir = workdir("create-unwritable");
    dir.mkdir("data");
    dir.set_readonly("data", true);
    let (code, out) = support::dispatch(&dir, SKILL, "create", &[("schema", THING), ("path", A)]);
    dir.set_readonly("data", false);
    support::expect(
        "create",
        &json!({"code": code, "out": out}),
        &[("/code", eq(1)), ("/out/reason", eq("書けない"))],
    );
}

// ── 値を取得する

#[test]
fn get_returns_the_value() {
    let dir = workdir("get");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call("update", &[("path", A), ("patch", FILL)], 0, &[]),
            call(
                "get",
                &[("path", A), ("query", "items[0]")],
                0,
                &[("/value", eq("x"))],
            ),
        ],
    );
}

#[test]
fn get_with_an_invalid_query_fails() {
    let dir = workdir("get-invalid");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call(
                "get",
                &[("path", A), ("query", "")],
                1,
                &[("/reason", eq("JMESPath 式が読めない"))],
            ),
            call(
                "get",
                &[("path", A), ("query", "items[0")],
                1,
                &[("/reason", eq("JMESPath 式が読めない"))],
            ),
        ],
    );
}

#[test]
fn get_from_an_unreadable_instance_fails() {
    let dir = workdir("get-unreadable");
    run(
        &dir,
        SKILL,
        &[call(
            "get",
            &[("path", "./data/none.json"), ("query", "name")],
            1,
            &[("/reason", eq("読めない"))],
        )],
    );
}

#[test]
fn get_matching_nothing_returns_null() {
    let dir = workdir("get-nothing");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call(
                "get",
                &[("path", A), ("query", "note")],
                0,
                &[("/value", eq(Value::Null)), ("/found", eq(false))],
            ),
        ],
    );
}

// ── インスタンスを更新する

#[test]
fn update_returns_the_validation_result() {
    let dir = workdir("update-result");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call(
                "update",
                &[("path", A), ("patch", NAME)],
                0,
                &[
                    ("/errors", eq(json!([]))),
                    ("/unfilled/0/property", eq("/items")),
                ],
            ),
        ],
    );
}

#[test]
fn update_with_an_empty_patch_fails() {
    let dir = workdir("update-empty");
    run(&dir, SKILL, &[create(0, &[])]);
    let before = dir.read(A);
    run(
        &dir,
        SKILL,
        &[call(
            "update",
            &[("path", A), ("patch", "")],
            1,
            &[("/reason", eq("JSON Patch が空である"))],
        )],
    );
    assert_eq!(dir.read(A), before);
}

#[test]
fn update_of_an_unreadable_instance_fails() {
    let dir = workdir("update-unreadable");
    dir.put_text("data/a.json", "{");
    run(
        &dir,
        SKILL,
        &[call(
            "update",
            &[("path", A), ("patch", NAME)],
            1,
            &[("/reason", eq("JSON として読めない"))],
        )],
    );
}

#[test]
fn update_refuses_a_patch_that_cannot_apply() {
    let dir = workdir("update-cannot-apply");
    run(&dir, SKILL, &[create(0, &[])]);
    let before = dir.read(A);
    run(
        &dir,
        SKILL,
        &[call(
            "update",
            &[
                ("path", A),
                (
                    "patch",
                    r#"[{"op":"replace","path":"/missing/x","value":1}]"#,
                ),
            ],
            1,
            &[("/reason", eq("JSON Patch を適用できない"))],
        )],
    );
    assert_eq!(dir.read(A), before);
}

#[test]
fn update_refuses_an_invalid_result() {
    let dir = workdir("update-invalid");
    run(&dir, SKILL, &[create(0, &[])]);
    let before = dir.read(A);
    run(
        &dir,
        SKILL,
        &[call(
            "update",
            &[
                ("path", A),
                ("patch", r#"[{"op":"add","path":"/name","value":""}]"#),
            ],
            1,
            &[
                ("/reason", eq("検証を通過しない")),
                ("/errors/0/property", eq("/name")),
            ],
        )],
    );
    assert_eq!(dir.read(A), before);
}

#[test]
fn update_without_a_change_ends() {
    let dir = workdir("update-no-change");
    run(
        &dir,
        SKILL,
        &[
            create(0, &[]),
            call("update", &[("path", A), ("patch", FILL)], 0, &[]),
        ],
    );
    let before = dir.read(A);
    run(
        &dir,
        SKILL,
        &[call(
            "update",
            &[
                ("path", A),
                ("patch", r#"[{"op":"replace","path":"/name","value":"a"}]"#),
            ],
            0,
            &[("/changed", eq(false))],
        )],
    );
    assert_eq!(dir.read(A), before);
}

#[test]
fn update_of_an_unwritable_file_fails() {
    let dir = workdir("update-unwritable");
    run(&dir, SKILL, &[create(0, &[])]);
    let before = dir.read(A);
    dir.set_readonly(A, true);
    let (code, out) = support::dispatch(&dir, SKILL, "update", &[("path", A), ("patch", NAME)]);
    dir.set_readonly(A, false);
    support::expect(
        "update",
        &json!({"code": code, "out": out}),
        &[("/code", eq(1)), ("/out/reason", eq("書けない"))],
    );
    assert_eq!(dir.read(A), before);
}

// ── x-prompt を受け取る

#[test]
fn prompt_returns_the_x_prompt() {
    let dir = workdir("prompt");
    run(
        &dir,
        SKILL,
        &[call(
            "prompt",
            &[("schema", THING), ("property", "/name")],
            0,
            &[("/prompt/write", eq("名前を1語で書く"))],
        )],
    );
}

#[test]
fn prompt_from_an_unreadable_schema_fails() {
    let dir = workdir("prompt-unreadable");
    run(
        &dir,
        SKILL,
        &[call(
            "prompt",
            &[
                ("schema", "./skills/example/assets/none.schema.json"),
                ("property", "/name"),
            ],
            1,
            &[("/reason", eq("読めない"))],
        )],
    );
}

#[test]
fn prompt_for_a_property_without_one_fails() {
    let dir = workdir("prompt-without");
    run(
        &dir,
        SKILL,
        &[
            call(
                "prompt",
                &[("schema", THING), ("property", "/note")],
                1,
                &[("/reason", eq("x-prompt を持つプロパティがスキーマに無い"))],
            ),
            call(
                "prompt",
                &[("schema", THING), ("property", "/missing")],
                1,
                &[("/reason", eq("x-prompt を持つプロパティがスキーマに無い"))],
            ),
        ],
    );
}

#[test]
fn create_refuses_a_schema_without_a_name() {
    let dir = workdir("create-unnamed");
    dir.put(
        "skills/example/assets/unnamed.schema.json",
        &json!({"type": "object", "properties": {"$schema": {"type": "string"}}}),
    );
    run(
        &dir,
        SKILL,
        &[call(
            "create",
            &[
                ("schema", "./skills/example/assets/unnamed.schema.json"),
                ("path", A),
            ],
            1,
            &[("/reason", eq("スキーマの名前が無い"))],
        )],
    );
    assert!(!dir.exists(A), "断ったらファイルを作らない");
}
