//! インスタンスは、スキーマを「<Skill の名前>/<種類>@<版>」の名前で指す。
//! エンジンは --skill の Skill の assets と、自分が持つ abstract だけから名前を引く。

mod support;

use serde_json::{json, Value};
use support::{call, eq, has, len, run, Scratch};

const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
/// Skill lexicon。
const LEXICON: Option<&str> = Some("./skills/lexicon");
/// インスタンスを置くディレクトリ。
const WORDS: &str = "./project/words";

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

/// 作業用のフォルダに、プロジェクトの words/ を作る。
fn project(test: &str) -> Scratch {
    let scratch = Scratch::new(&format!("names-{test}"));
    scratch.mkdir("project/words");
    scratch
}

/// skills/lexicon/assets/word.schema.json と、プロジェクトの words/ を作る。
fn setup(test: &str) -> Scratch {
    let scratch = project(test);
    scratch.put(
        "skills/lexicon/assets/word.schema.json",
        &schema("lexicon/word@v1"),
    );
    scratch
}

/// words/a.json に、$schema が name の語を置く。
fn word(scratch: &Scratch, name: &str) {
    scratch.put(
        "project/words/a.json",
        &json!({"$schema": name, "kind": "word", "text": "語"}),
    );
}

#[test]
fn create_writes_the_name_of_a_named_schema() {
    let root = setup("create");
    run(
        &root,
        LEXICON,
        &[call(
            "create",
            &[
                ("schema", "./skills/lexicon/assets/word.schema.json"),
                ("path", "./project/words/a.json"),
            ],
            0,
            &[],
        )],
    );
    assert_eq!(
        root.read_json("project/words/a.json")["$schema"],
        "lexicon/word@v1"
    );
}

#[test]
fn check_finds_the_schema_by_name_wherever_the_skill_lives() {
    let root = setup("check");
    // 置き場所の違う同じ Skill（フォルダの名前は lexicon）
    root.put(
        "elsewhere/lexicon/assets/word.schema.json",
        &schema("lexicon/word@v1"),
    );
    word(&root, "lexicon/word@v1");
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            0,
            &[("/instances/0/errors", eq(json!([])))],
        )],
    );
    // 置き場所を変えても、名前で引ける
    run(
        &root,
        Some("./elsewhere/lexicon"),
        &[call(
            "check",
            &[("dir", WORDS)],
            0,
            &[("/instances/0/errors", eq(json!([])))],
        )],
    );
}

#[test]
fn a_name_needs_a_skill() {
    let root = setup("noskill");
    word(&root, "lexicon/word@v1");
    run(
        &root,
        None,
        &[call(
            "check",
            &[("dir", WORDS)],
            1,
            &[
                ("/reason", eq("スキーマの名前を引けない")),
                ("/detail", has("--skill")),
            ],
        )],
    );
}

#[test]
fn another_skills_name_stops_with_that_skill() {
    let root = setup("foreign");
    word(&root, "glossary/term@v1");
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            1,
            &[
                ("/reason", eq("スキーマの名前を引けない")),
                ("/detail", has("Skill glossary を --skill に渡して")),
            ],
        )],
    );
}

#[test]
fn an_unknown_name_in_the_skill_is_refused() {
    let root = setup("unknown");
    word(&root, "lexicon/word@v2");
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            1,
            &[
                ("/reason", eq("スキーマの名前を引けない")),
                ("/detail", has("lexicon/word@v2 を $id に持つスキーマ")),
            ],
        )],
    );
}

#[test]
fn two_schemas_with_one_name_are_refused() {
    let root = setup("duplicate");
    root.put(
        "skills/lexicon/assets/copy.schema.json",
        &schema("lexicon/word@v1"),
    );
    word(&root, "lexicon/word@v1");
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            1,
            &[
                ("/detail", has("copy.schema.json")),
                ("/detail", has("word.schema.json")),
            ],
        )],
    );
}

#[test]
fn a_relative_path_in_schema_is_refused() {
    let root = setup("path");
    root.put("project/words/kept.schema.json", &schema("lexicon/word@v1"));
    word(&root, "kept.schema.json");
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            1,
            &[("/reason", eq("スキーマの名前でない"))],
        )],
    );
}

#[test]
fn update_validates_against_the_named_schema() {
    let root = setup("update");
    root.put(
        "project/words/a.json",
        &json!({"$schema": "lexicon/word@v1", "kind": "word"}),
    );
    run(
        &root,
        LEXICON,
        &[call(
            "update",
            &[
                ("path", "./project/words/a.json"),
                ("patch", r#"[{"op":"add","path":"/text","value":1}]"#),
            ],
            1,
            &[("/reason", eq("検証を通過しない"))],
        )],
    );
}

#[test]
fn a_schema_refers_to_another_by_its_name() {
    let root = project("ref");
    // lexicon/word@v1 の text を、同じ Skill の lexicon/term@v1 の定義で縛る。$ref は名前で書き、JSON Schema の規則で解く
    let mut word_schema = schema("lexicon/word@v1");
    word_schema["properties"]["text"] = json!({"$ref": "term@v1#/$defs/short", "description": "語",
        "x-prompt": {"read": {"means": "見本の欄である。"},
                     "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}});
    root.put("skills/lexicon/assets/word.schema.json", &word_schema);
    root.put(
        "skills/lexicon/assets/term.schema.json",
        &json!({"$schema": DIALECT, "x-abstract": "schema-driven/abstract@v1", "$id": "lexicon/term@v1",
               "title": "語の決まり", "description": "見本", "$defs": {"short": {"type": "string", "maxLength": 3}}}),
    );
    root.put(
        "project/words/a.json",
        &json!({"$schema": "lexicon/word@v1", "kind": "word", "text": "長すぎる語"}),
    );
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            0,
            &[
                ("/instances/0/errors", len(1)),
                ("/instances/0/errors/0/property", eq("/text")),
            ],
        )],
    );
}

#[test]
fn a_schema_left_in_references_is_not_found_by_name() {
    let root = project("refs");
    root.put(
        "skills/lexicon/references/word.schema.json",
        &schema("lexicon/word@v1"),
    );
    word(&root, "lexicon/word@v1");
    run(
        &root,
        LEXICON,
        &[call(
            "check",
            &[("dir", WORDS)],
            1,
            &[
                ("/reason", eq("スキーマの名前を引けない")),
                ("/detail", has("assets")),
            ],
        )],
    );
}
