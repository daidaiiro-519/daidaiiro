//! インスタンスは、スキーマを「<Skill の名前>/<種類>@<版>」の名前で指す。
//! エンジンは --skill の Skill の assets と、自分が持つ abstract だけから名前を引く。

mod common;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

fn field(description: &str) -> Value {
    json!({"type": "string", "description": description,
           "x-prompt": {"read": {"means": "見本の欄である。"},
                        "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}})
}

fn schema(id: &str) -> Value {
    json!({"$schema": DIALECT, "x-abstract": "schema-driven/abstract@v1", "$id": id,
           "title": "語", "description": "見本", "x-generates": "words/<名前>.json",
           "type": "object", "required": ["kind", "text"],
           "properties": {"$schema": field("従うスキーマ"),
                          "kind": {"const": "word", "description": "種類",
                                   "x-prompt": {"read": {"means": "種類である。"},
                                                "write": [{"id": "const", "rule": "word と書く。", "why": "種類を判定する。", "test": "値が word かを見る。"}]}},
                          "text": field("語")}})
}

/// skills/<name>/assets/word.schema.json と、プロジェクトの words/ を作る。
fn setup(test: &str, name: &str) -> (common::Scratch, PathBuf) {
    let root = common::Scratch::new(&format!("sd-names-{test}"));
    let _ = fs::remove_dir_all(&root);
    let skill = root.join("skills").join(name);
    fs::create_dir_all(skill.join("assets")).unwrap();
    fs::create_dir_all(root.join("project/words")).unwrap();
    fs::write(
        skill.join("assets/word.schema.json"),
        schema(&format!("{name}/word@v1")).to_string(),
    )
    .unwrap();
    (root, skill)
}

fn run(dir: &Path, skill: Option<&Path>, args: &[&str]) -> (i32, Value) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_schema-driven"));
    command.current_dir(dir);
    if let Some(skill) = skill {
        command.arg("--skill").arg(skill);
    }
    let output = command.args(args).output().unwrap();
    let out = serde_json::from_slice(&output.stdout).unwrap_or_else(
        |_| json!({"unparsed": String::from_utf8_lossy(&output.stdout).into_owned()}),
    );
    (output.status.code().unwrap_or(-1), out)
}

fn write(path: PathBuf, value: Value) {
    fs::write(path, value.to_string()).unwrap();
}

#[test]
fn create_writes_the_name_of_a_named_schema() {
    let (root, skill) = setup("create", "lexicon");
    let schema_path = skill.join("assets/word.schema.json");
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &[
            "create",
            "--schema",
            schema_path.to_str().unwrap(),
            "--path",
            "words/a.json",
        ],
    );
    assert_eq!(code, 0, "{out}");
    let written: Value =
        serde_json::from_str(&fs::read_to_string(root.join("project/words/a.json")).unwrap())
            .unwrap();
    assert_eq!(written["$schema"], "lexicon/word@v1");
}

#[test]
fn check_finds_the_schema_by_name_wherever_the_skill_lives() {
    let (root, skill) = setup("check", "lexicon");
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v1", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["instances"][0]["errors"], json!([]), "{out}");
    // 置き場所を変えても、名前で引ける
    let moved = root.join("elsewhere/lexicon");
    fs::create_dir_all(moved.parent().unwrap()).unwrap();
    fs::rename(&skill, &moved).unwrap();
    let (code, out) = run(
        &root.join("project"),
        Some(&moved),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 0, "{out}");
}

#[test]
fn a_name_needs_a_skill() {
    let (root, _) = setup("noskill", "lexicon");
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v1", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(&root.join("project"), None, &["check", "--dir", "words"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "スキーマの名前を引けない", "{out}");
    assert!(out["detail"].as_str().unwrap().contains("--skill"), "{out}");
}

#[test]
fn another_skills_name_stops_with_that_skill() {
    let (root, skill) = setup("foreign", "lexicon");
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "glossary/term@v1", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "スキーマの名前を引けない", "{out}");
    let detail = out["detail"].as_str().unwrap();
    assert!(
        detail.contains("Skill glossary を --skill に渡して"),
        "{out}"
    );
}

#[test]
fn an_unknown_name_in_the_skill_is_refused() {
    let (root, skill) = setup("unknown", "lexicon");
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v2", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "スキーマの名前を引けない", "{out}");
    assert!(
        out["detail"]
            .as_str()
            .unwrap()
            .contains("lexicon/word@v2 を $id に持つスキーマ"),
        "{out}"
    );
}

#[test]
fn two_schemas_with_one_name_are_refused() {
    let (root, skill) = setup("duplicate", "lexicon");
    write(
        skill.join("assets/copy.schema.json"),
        schema("lexicon/word@v1"),
    );
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v1", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 1, "{out}");
    let detail = out["detail"].as_str().unwrap();
    assert!(
        detail.contains("copy.schema.json") && detail.contains("word.schema.json"),
        "{out}"
    );
}

#[test]
fn a_relative_path_to_a_kept_copy_still_works() {
    let (root, _) = setup("copy", "lexicon");
    write(
        root.join("project/words/kept.schema.json"),
        schema("lexicon/word@v1"),
    );
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "kept.schema.json", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(&root.join("project"), None, &["check", "--dir", "words"]);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn update_validates_against_the_named_schema() {
    let (root, skill) = setup("update", "lexicon");
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v1", "kind": "word"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &[
            "update",
            "--path",
            "words/a.json",
            "--patch",
            r#"[{"op":"add","path":"/text","value":1}]"#,
        ],
    );
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "検証を通過しない", "{out}");
}

#[test]
fn a_schema_refers_to_another_by_its_name() {
    let (root, skill) = setup("ref", "lexicon");
    // lexicon/word@v1 の text を、同じ Skill の lexicon/term@v1 の定義で縛る。$ref は名前で書き、JSON Schema の規則で解く
    let mut word = schema("lexicon/word@v1");
    word["properties"]["text"] = json!({"$ref": "term@v1#/$defs/short", "description": "語",
        "x-prompt": {"read": {"means": "見本の欄である。"},
                     "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}});
    write(skill.join("assets/word.schema.json"), word);
    write(
        skill.join("assets/term.schema.json"),
        json!({"$schema": DIALECT, "x-abstract": "schema-driven/abstract@v1", "$id": "lexicon/term@v1",
               "title": "語の決まり", "description": "見本", "$defs": {"short": {"type": "string", "maxLength": 3}}}),
    );
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v1", "kind": "word", "text": "長すぎる語"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 0, "{out}");
    let errors = out["instances"][0]["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{out}");
    assert_eq!(errors[0]["property"], "/text", "{out}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_schema_left_in_references_is_not_found_by_name() {
    let (root, skill) = setup("refs", "lexicon");
    fs::create_dir_all(skill.join("references")).unwrap();
    fs::rename(
        skill.join("assets/word.schema.json"),
        skill.join("references/word.schema.json"),
    )
    .unwrap();
    write(
        root.join("project/words/a.json"),
        json!({"$schema": "lexicon/word@v1", "kind": "word", "text": "語"}),
    );
    let (code, out) = run(
        &root.join("project"),
        Some(&skill),
        &["check", "--dir", "words"],
    );
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "スキーマの名前を引けない", "{out}");
    assert!(out["detail"].as_str().unwrap().contains("assets"), "{out}");
}
