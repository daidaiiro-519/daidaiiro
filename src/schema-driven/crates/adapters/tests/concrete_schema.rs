//! concrete スキーマは abstract スキーマのインスタンスとして、エンティティのインスタンスと同じ共通ツールで作り、書き進め、検査する。

mod support;

use serde_json::json;
use support::{call, eq, len, run, Call, Scratch};

const ABSTRACT: &str = "schema-driven/abstract@v1";
const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
/// abstract から作る concrete スキーマ。
const TERM: &str = "./demo/assets/term.schema.json";

/// abstract から demo/assets/term.schema.json を作る行。
fn create_schema<'row>(code: i32, expect: &'row [(&'row str, support::Is)]) -> Call<'row> {
    call(
        "create",
        &[("schema", ABSTRACT), ("path", TERM)],
        code,
        expect,
    )
}

fn workdir(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("concrete-{name}"));
    scratch.mkdir("demo/assets");
    scratch
}

#[test]
fn create_from_the_abstract_writes_the_dialect_and_the_version() {
    let dir = workdir("create");
    run(
        &dir,
        None,
        &[create_schema(
            0,
            &[
                ("/unfilled", len(2)),
                ("/unfilled/0/property", eq("/description")),
                ("/unfilled/0/prompt/write/0/id", eq("what-and-not")),
                ("/unfilled/1/property", eq("/title")),
                ("/unfilled/1/prompt/write/0/id", eq("noun")),
            ],
        )],
    );
    assert_eq!(
        dir.read_json(TERM),
        json!({"$schema": DIALECT, "x-abstract": ABSTRACT})
    );
}

#[test]
fn create_from_the_abstract_refuses_an_existing_path() {
    let dir = workdir("exists");
    run(&dir, None, &[create_schema(0, &[]), create_schema(1, &[])]);
}

#[test]
fn prompt_reads_the_abstract_by_name() {
    let dir = workdir("prompt");
    run(
        &dir,
        None,
        &[call(
            "prompt",
            &[("schema", ABSTRACT), ("property", "/title")],
            0,
            &[("/prompt/write/0/id", eq("noun"))],
        )],
    );
}

#[test]
fn a_concrete_schema_is_written_with_update_and_then_makes_entities() {
    let dir = workdir("flow");
    let prompt = json!({"read": {"means": "種類である。"},
                        "write": [{"id": "const", "rule": "term と書く。", "why": "種類を判定する。", "test": "値が term かを見る。"}]});
    let patch = json!([
        {"op": "add", "path": "/$id", "value": "demo/term@v1"},
        {"op": "add", "path": "/title", "value": "用語"},
        {"op": "add", "path": "/description", "value": "用語1つ。説明の文書は持たない。"},
        {"op": "add", "path": "/x-generates", "value": "terms/<名前>.json"},
        {"op": "add", "path": "/type", "value": "object"},
        {"op": "add", "path": "/required", "value": ["$schema", "kind"]},
        {"op": "add", "path": "/properties", "value": {
            "$schema": {"const": "demo/term@v1", "description": "従うスキーマの名前", "x-prompt": prompt},
            "kind": {"const": "term", "description": "種類", "x-prompt": prompt}}}
    ])
    .to_string();
    run(
        &dir,
        None,
        &[
            create_schema(0, &[]),
            call(
                "update",
                &[("path", TERM), ("patch", &patch)],
                0,
                &[("/errors", eq(json!([])))],
            ),
            call(
                "check-schemas",
                &[("dir", "./demo/assets")],
                0,
                &[("/findings", eq(json!([])))],
            ),
        ],
    );
    run(
        &dir,
        Some("./demo"),
        &[call(
            "create",
            &[("schema", TERM), ("path", "./terms/a.json")],
            0,
            &[],
        )],
    );
    assert_eq!(
        dir.read_json("terms/a.json"),
        json!({"$schema": "demo/term@v1"})
    );
}

#[test]
fn check_schemas_returns_how_to_write_a_missing_field() {
    let dir = workdir("findings");
    run(
        &dir,
        None,
        &[
            create_schema(0, &[]),
            call(
                "check-schemas",
                &[("dir", "./demo/assets")],
                0,
                &[
                    ("/findings/1/at", eq("/title")),
                    ("/findings/1/prompt/write/0/id", eq("noun")),
                ],
            ),
        ],
    );
}
