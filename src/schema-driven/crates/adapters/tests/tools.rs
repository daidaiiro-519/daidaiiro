//! 共通ツールが約束する振る舞い。公開したあとに振る舞いを変えるときは、abstract スキーマの版を上げる。
//! テストごとに、作業用のフォルダに Skill のフォルダ contract を作り、公開 API の振り分けを --skill にそれを渡して呼ぶ。

mod support;

use serde_json::{json, Value};
use support::{call, eq, has, put_catalog, run, Scratch};

/// Skill のフォルダ。スキーマの名前は contract/<種類>@v1 にする。
const SKILL: Option<&str> = Some("./contract");

/// assets/word.schema.json に置く、語1つの concrete スキーマ。
fn word_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "x-abstract": "schema-driven/abstract@v1",
        "$id": "contract/word@v1",
        "title": "語",
        "description": "語1つ。試験のための concrete スキーマ",
        "x-generates": "<名前>.json",
        "properties": {
            "$schema": {
                "const": "contract/word@v1",
                "description": "従うスキーマの名前",
                "x-prompt": {
                    "read": {"means": "このインスタンスが従うスキーマの名前である。"},
                    "write": [{"id": "name", "rule": "contract/word@v1 と書く。", "why": "スキーマを名前で引く。", "test": "値が contract/word@v1 かを見る。"}]
                }
            },
            "kind": {
                "const": "word",
                "description": "種類",
                "x-prompt": {
                    "read": {"means": "語であることを表す。"},
                    "write": [{"id": "const", "rule": "word と書く。", "why": "種類を判定する。", "test": "値が word かを見る。"}]
                }
            },
            "text": {
                "type": "string",
                "description": "語",
                "x-prompt": {
                    "read": {"means": "語の文字である。"},
                    "write": [{"id": "one", "rule": "語を1つ書く。", "why": "1つの値に1つのことを書く。", "test": "空白が無いかを見る。"}]
                }
            }
        },
        "required": ["$schema", "kind", "text"]
    })
}

/// assets にスキーマ word を置いた Skill。
fn word_skill(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("abstract-v1-{name}"));
    scratch.put("contract/assets/word.schema.json", &word_schema());
    scratch
}

/// スキーマ word と、インスタンス w.json を置いた Skill。
fn word_skill_with_instance(name: &str) -> Scratch {
    let scratch = word_skill(name);
    scratch.put(
        "contract/w.json",
        &json!({"$schema": "contract/word@v1", "kind": "word", "text": "語"}),
    );
    scratch
}

#[test]
fn approve_records_a_checked_instance() {
    let scratch = word_skill_with_instance("approve");
    run(
        &scratch,
        SKILL,
        &[call(
            "approve",
            &[("dir", "./contract")],
            0,
            &[("/ok", eq(true)), ("/changed", eq(true))],
        )],
    );
}

#[test]
fn check_passes_a_valid_instance() {
    let scratch = word_skill_with_instance("check");
    run(
        &scratch,
        SKILL,
        &[call(
            "check",
            &[("dir", "./contract")],
            0,
            &[
                ("/ok", eq(true)),
                ("/instances/0/errors", eq(json!([]))),
                ("/findings", eq(json!([]))),
            ],
        )],
    );
}

/// 検査するのは Skill の assets（スキーマを置いたフォルダ）である。
#[test]
fn check_schemas_passes_a_v1_schema() {
    let scratch = word_skill("check-schemas");
    run(
        &scratch,
        SKILL,
        &[call(
            "check-schemas",
            &[("dir", "./contract/assets")],
            0,
            &[
                ("/ok", eq(true)),
                ("/schemas", eq(json!(["word.schema.json"]))),
                ("/findings", eq(json!([]))),
            ],
        )],
    );
}

#[test]
fn create_refuses_existing_path() {
    let scratch = word_skill_with_instance("create-exists");
    run(
        &scratch,
        SKILL,
        &[call(
            "create",
            &[
                ("schema", "./contract/assets/word.schema.json"),
                ("path", "./contract/w.json"),
            ],
            1,
            &[
                ("/ok", eq(false)),
                ("/reason", eq("パスにインスタンスが既にある")),
            ],
        )],
    );
}

#[test]
fn create_returns_unfilled() {
    let scratch = word_skill("create-unfilled");
    run(
        &scratch,
        SKILL,
        &[call(
            "create",
            &[
                ("schema", "./contract/assets/word.schema.json"),
                ("path", "./contract/w.json"),
            ],
            0,
            &[
                ("/ok", eq(true)),
                ("/unfilled/0/property", eq("/kind")),
                ("/unfilled/1/prompt/read/means", eq("語の文字である。")),
            ],
        )],
    );
}

#[test]
fn delete_returns_no_remaining_reference() {
    let scratch = word_skill_with_instance("delete");
    run(
        &scratch,
        SKILL,
        &[call(
            "delete",
            &[("path", "./contract/w.json")],
            0,
            &[("/ok", eq(true)), ("/remaining", eq(json!([])))],
        )],
    );
}

#[test]
fn get_reads_a_value() {
    let scratch = word_skill_with_instance("get");
    run(
        &scratch,
        SKILL,
        &[call(
            "get",
            &[("path", "./contract/w.json"), ("query", "text")],
            0,
            &[
                ("/ok", eq(true)),
                ("/value", eq("語")),
                ("/found", eq(true)),
            ],
        )],
    );
}

#[test]
fn prompt_returns_the_v1_form() {
    let scratch = word_skill("prompt");
    run(
        &scratch,
        SKILL,
        &[call(
            "prompt",
            &[
                ("schema", "./contract/assets/word.schema.json"),
                ("property", "/text"),
            ],
            0,
            &[
                ("/ok", eq(true)),
                ("/prompt/read/means", eq("語の文字である。")),
                ("/prompt/write/0/id", eq("one")),
            ],
        )],
    );
}

#[test]
fn render_draws_a_concrete_kind_with_the_skill_catalog() {
    let scratch = word_skill_with_instance("render");
    put_catalog(&scratch, "contract");
    scratch.put(
        "contract/pages/word.json",
        &json!({"kind": "word", "sections": [{"heading": "語", "body": [
            {"component": "pill", "text": "this.text"}
        ]}]}),
    );
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            &[
                ("dir", "./contract"),
                ("pages", "./contract/pages"),
                ("out", "./contract/out"),
            ],
            0,
            &[
                ("/ok", eq(true)),
                ("/pages/0/path", eq("./contract/out/w.html")),
            ],
        )],
    );
}

#[test]
fn render_refuses_a_part_missing_from_the_catalog() {
    let scratch = word_skill_with_instance("render-missing-part");
    put_catalog(&scratch, "contract");
    scratch.put(
        "contract/pages/word.json",
        &json!({"kind": "word", "sections": [{"heading": "語", "body": [
            {"component": "changeMarker", "value": "this.text"}
        ]}]}),
    );
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            &[
                ("dir", "./contract"),
                ("pages", "./contract/pages"),
                ("out", "./contract/out"),
            ],
            1,
            &[
                ("/ok", eq(false)),
                ("/reason", eq("ページテンプレートが契約を通過しない")),
            ],
        )],
    );
}

#[test]
fn render_refuses_an_unknown_kind() {
    let scratch = word_skill_with_instance("render-unknown-kind");
    put_catalog(&scratch, "contract");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            &[("dir", "./contract"), ("out", "./contract/out")],
            1,
            &[
                ("/ok", eq(false)),
                ("/reason", eq("ページテンプレートの指定が無い種類がある")),
            ],
        )],
    );
}

#[test]
fn render_refuses_without_the_skill_catalog() {
    let scratch = word_skill_with_instance("render-no-catalog");
    run(
        &scratch,
        SKILL,
        &[call(
            "render",
            &[("dir", "./contract"), ("out", "./contract/out")],
            1,
            &[
                ("/reason", eq("部品のカタログを読めない")),
                ("/detail", has("assets/catalog")),
            ],
        )],
    );
}

#[test]
fn unknown_tool_is_misuse() {
    let scratch = Scratch::new("unknown-tool");
    run(
        &scratch,
        None,
        &[call(
            "nothing",
            &[],
            2,
            &[("/ok", eq(false)), ("/reason", eq("使い方が違う"))],
        )],
    );
}

#[test]
fn update_refuses_an_invalid_value() {
    let scratch = word_skill_with_instance("update-invalid");
    run(
        &scratch,
        SKILL,
        &[call(
            "update",
            &[
                ("path", "./contract/w.json"),
                ("patch", r#"[{"op":"replace","path":"/text","value":3}]"#),
            ],
            1,
            &[
                ("/ok", eq(false)),
                ("/reason", eq("検証を通過しない")),
                ("/errors/0/property", eq("/text")),
            ],
        )],
    );
}

#[test]
fn update_writes_a_valid_patch() {
    let scratch = word_skill_with_instance("update-valid");
    run(
        &scratch,
        SKILL,
        &[call(
            "update",
            &[
                ("path", "./contract/w.json"),
                (
                    "patch",
                    r#"[{"op":"replace","path":"/text","value":"言葉"}]"#,
                ),
            ],
            0,
            &[
                ("/ok", eq(true)),
                ("/changed", eq(true)),
                ("/errors", eq(json!([]))),
            ],
        )],
    );
}
