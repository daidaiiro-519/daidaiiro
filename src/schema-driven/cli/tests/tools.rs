//! 共通ツールと宣言したツールが約束する振る舞い。公開したあとに振る舞いを変えるときは、abstract スキーマの版を上げる。
//! テストごとに、一時ディレクトリに Skill のフォルダ contract を作り、その中で CLI を実行する。

mod common;

use common::{sha256_of, stub};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");

/// Skill のフォルダの名前。スキーマの名前は contract/<種類>@v1 にする。
const SKILL: &str = "contract";

/// 作業用のフォルダと、その中の Skill のフォルダ contract。
struct Skill {
    _scratch: common::Scratch,
    dir: PathBuf,
}

fn skill(name: &str) -> Skill {
    let scratch = common::Scratch::new(&format!("abstract-v1-{name}"));
    let dir = scratch.join(SKILL);
    fs::create_dir_all(&dir).unwrap();
    Skill {
        _scratch: scratch,
        dir,
    }
}

/// Skill のフォルダの path に、JSON を書く。
fn put(dir: &Path, path: &str, content: Value) {
    let target = dir.join(path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, serde_json::to_string_pretty(&content).unwrap()).unwrap();
}

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
        "required": ["kind", "text"]
    })
}

/// assets にスキーマ word を置いた Skill。with_instance なら、インスタンス w.json も置く。
fn word_skill(name: &str, with_instance: bool) -> Skill {
    let skill = skill(name);
    put(&skill.dir, "assets/word.schema.json", word_schema());
    if with_instance {
        put(
            &skill.dir,
            "w.json",
            json!({"$schema": "contract/word@v1", "kind": "word", "text": "語"}),
        );
    }
    skill
}

/// 見本の実行ファイルを scripts/probe に置き、それを mode で呼ぶツール probe を tools.json に宣言する。
/// sha256 を渡さなければ、置いた実行ファイルの SHA-256 を宣言する。
fn probe_skill(
    name: &str,
    mode: &str,
    sha256: Option<&str>,
    timeout_seconds: Option<u64>,
) -> Skill {
    let skill = skill(name);
    // Windows では実行ファイルに .exe を付ける
    let at = format!("scripts/probe{}", std::env::consts::EXE_SUFFIX);
    let target = skill.dir.join(&at);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(stub(), &target).unwrap();
    let sha256 = sha256.map_or_else(|| sha256_of(&target), str::to_owned);
    let mut tool = json!({
        "name": "probe",
        "description": "見本",
        "args": [{"name": "word", "description": "語", "required": true}],
        "command": [at, mode],
        "sha256": sha256
    });
    if let Some(seconds) = timeout_seconds {
        tool["timeout_seconds"] = json!(seconds);
    }
    put(
        &skill.dir,
        "tools.json",
        json!({"$schema": "schema-driven/tools@v1", "kind": "tools", "tools": [tool]}),
    );
    skill
}

/// Skill のフォルダで CLI を実行し、終了コードと標準出力の JSON を返す。with_skill なら --skill にそのフォルダを渡す。
fn run(dir: &Path, with_skill: bool, tool: &str, args: &[(&str, &str)]) -> (i32, Value) {
    let mut command = Command::new(BIN);
    command.current_dir(dir);
    if with_skill {
        command.arg("--skill").arg(dir);
    }
    command.arg(tool);
    for (name, value) in args {
        command.arg(format!("--{name}")).arg(value);
    }
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let out =
        serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("出力が JSON でない：{stdout}"));
    (output.status.code().unwrap_or(-1), out)
}

/// 終了コードと、JSON Pointer で指した欄の値を比べる。
fn expect(result: (i32, Value), code: i32, fields: &[(&str, Value)]) {
    let (actual, out) = result;
    assert_eq!(actual, code, "終了コード：{out}");
    for (pointer, expected) in fields {
        assert_eq!(out.pointer(pointer), Some(expected), "{pointer}：{out}");
    }
}

#[test]
fn approve_records_a_checked_instance() {
    let skill = word_skill("approve_records_a_checked_instance", true);
    expect(
        run(&skill.dir, true, "approve", &[("dir", ".")]),
        0,
        &[("/ok", json!(true)), ("/changed", json!(true))],
    );
}

#[test]
fn check_passes_a_valid_instance() {
    let skill = word_skill("check_passes_a_valid_instance", true);
    expect(
        run(&skill.dir, true, "check", &[("dir", ".")]),
        0,
        &[
            ("/ok", json!(true)),
            ("/instances/0/errors", json!([])),
            ("/findings", json!([])),
        ],
    );
}

#[test]
fn check_schemas_passes_a_v1_schema() {
    let skill = word_skill("check_schemas_passes_a_v1_schema", false);
    expect(
        run(&skill.dir, true, "check-schemas", &[("dir", ".")]),
        0,
        &[("/ok", json!(true)), ("/findings", json!([]))],
    );
}

#[test]
fn create_refuses_existing_path() {
    let skill = word_skill("create_refuses_existing_path", true);
    expect(
        run(
            &skill.dir,
            true,
            "create",
            &[("schema", "assets/word.schema.json"), ("path", "w.json")],
        ),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("パスにインスタンスが既にある")),
        ],
    );
}

#[test]
fn create_returns_unfilled() {
    let skill = word_skill("create_returns_unfilled", false);
    expect(
        run(
            &skill.dir,
            true,
            "create",
            &[("schema", "assets/word.schema.json"), ("path", "w.json")],
        ),
        0,
        &[
            ("/ok", json!(true)),
            ("/unfilled/0/property", json!("/kind")),
            ("/unfilled/1/prompt/read/means", json!("語の文字である。")),
        ],
    );
}

#[test]
fn declared_tool_gets_args_on_stdin() {
    let skill = probe_skill("declared_tool_gets_args_on_stdin", "echo", None, None);
    expect(
        run(&skill.dir, true, "probe", &[("word", "語")]),
        0,
        &[
            ("/ok", json!(true)),
            ("/request/tool", json!("probe")),
            ("/request/args/word", json!("語")),
        ],
    );
}

#[test]
fn declared_tool_is_stopped_at_time_limit() {
    let skill = probe_skill(
        "declared_tool_is_stopped_at_time_limit",
        "slow",
        None,
        Some(1),
    );
    expect(
        run(&skill.dir, true, "probe", &[("word", "語")]),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("制限時間を超えた")),
        ],
    );
}

#[test]
fn declared_tool_output_is_limited() {
    let skill = probe_skill("declared_tool_output_is_limited", "big", None, None);
    expect(
        run(&skill.dir, true, "probe", &[("word", "語")]),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("出力が大きすぎる")),
        ],
    );
}

#[test]
fn declared_tool_output_must_be_json() {
    let skill = probe_skill("declared_tool_output_must_be_json", "not-json", None, None);
    expect(
        run(&skill.dir, true, "probe", &[("word", "語")]),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("出力が契約に合わない")),
        ],
    );
}

#[test]
fn declared_tool_with_wrong_sha256_is_not_run() {
    let skill = probe_skill(
        "declared_tool_with_wrong_sha256_is_not_run",
        "echo",
        Some("0000000000000000000000000000000000000000000000000000000000000000"),
        None,
    );
    expect(
        run(&skill.dir, true, "probe", &[("word", "語")]),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("実行ファイルの SHA-256 が宣言と合わない")),
        ],
    );
}

#[test]
fn delete_bare_path() {
    let skill = word_skill("delete_bare_path", true);
    expect(
        run(&skill.dir, true, "delete", &[("path", "w.json")]),
        0,
        &[("/ok", json!(true)), ("/remaining", json!([]))],
    );
}

#[test]
fn get_reads_a_value() {
    let skill = word_skill("get_reads_a_value", true);
    expect(
        run(
            &skill.dir,
            true,
            "get",
            &[("path", "w.json"), ("query", "text")],
        ),
        0,
        &[
            ("/ok", json!(true)),
            ("/value", json!("語")),
            ("/found", json!(true)),
        ],
    );
}

#[test]
fn prompt_returns_the_v1_form() {
    let skill = word_skill("prompt_returns_the_v1_form", false);
    expect(
        run(
            &skill.dir,
            true,
            "prompt",
            &[("schema", "assets/word.schema.json"), ("property", "/text")],
        ),
        0,
        &[
            ("/ok", json!(true)),
            ("/prompt/read/means", json!("語の文字である。")),
            ("/prompt/write/0/id", json!("one")),
        ],
    );
}

#[test]
fn render_draws_a_concrete_kind_with_the_catalog() {
    let skill = word_skill("render_draws_a_concrete_kind_with_the_catalog", true);
    put(
        &skill.dir,
        "pages/word.json",
        json!({"kind": "word", "sections": [{"heading": "語", "body": [
            {"component": "tabs", "items": "[this]", "label": "text", "body": ["text"]}
        ]}]}),
    );
    expect(
        run(
            &skill.dir,
            true,
            "render",
            &[("dir", "."), ("pages", "pages"), ("out", "out")],
        ),
        0,
        &[("/ok", json!(true)), ("/pages/0/design", json!("catalog"))],
    );
}

#[test]
fn render_refuses_a_part_missing_from_the_catalog() {
    let skill = word_skill("render_refuses_a_part_missing_from_the_catalog", true);
    put(
        &skill.dir,
        "pages/word.json",
        json!({"kind": "word", "sections": [{"heading": "語", "body": [
            {"component": "changeMarker", "value": "this.text"}
        ]}]}),
    );
    expect(
        run(
            &skill.dir,
            true,
            "render",
            &[("dir", "."), ("pages", "pages"), ("out", "out")],
        ),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("ページテンプレートが契約を通過しない")),
        ],
    );
}

#[test]
fn render_refuses_an_unknown_kind() {
    let skill = word_skill("render_refuses_an_unknown_kind", true);
    expect(
        run(&skill.dir, true, "render", &[("dir", "."), ("out", "out")]),
        1,
        &[("/ok", json!(false))],
    );
}

#[test]
fn unknown_tool_is_misuse() {
    let skill = skill("unknown_tool_is_misuse");
    expect(
        run(&skill.dir, false, "nothing", &[]),
        2,
        &[("/ok", json!(false)), ("/reason", json!("使い方が違う"))],
    );
}

#[test]
fn update_refuses_an_invalid_value() {
    let skill = word_skill("update_refuses_an_invalid_value", true);
    expect(
        run(
            &skill.dir,
            true,
            "update",
            &[
                ("path", "w.json"),
                ("patch", r#"[{"op":"replace","path":"/text","value":3}]"#),
            ],
        ),
        1,
        &[
            ("/ok", json!(false)),
            ("/reason", json!("検証を通過しない")),
            ("/errors/0/property", json!("/text")),
        ],
    );
}

#[test]
fn update_writes_a_valid_patch() {
    let skill = word_skill("update_writes_a_valid_patch", true);
    expect(
        run(
            &skill.dir,
            true,
            "update",
            &[
                ("path", "w.json"),
                (
                    "patch",
                    r#"[{"op":"replace","path":"/text","value":"言葉"}]"#,
                ),
            ],
        ),
        0,
        &[
            ("/ok", json!(true)),
            ("/changed", json!(true)),
            ("/errors", json!([])),
        ],
    );
}
