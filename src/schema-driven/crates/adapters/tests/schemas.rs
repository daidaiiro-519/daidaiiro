//! check-schemas は、ディレクトリの具体のスキーマ（*.schema.json）だけを検査し、インスタンスは読まない。

mod support;

use serde_json::{json, Value};
use support::{call, eq, len, run, Scratch};

/// 欄1つ。extra を書き足す。
fn field(extra: Value) -> Value {
    let mut base = json!({"type": "string", "description": "説明", "x-prompt": {"read": {"means": "見本の欄である。"},
              "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}});
    base.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    base
}

/// 決まりに従っている具体のスキーマに、申告していない注釈 x-foo を1つ足したもの。
fn with_undeclared_annotation() -> Value {
    json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1", "$id": "lexicon/glossary@v1", "title": "用語集", "description": "語の一覧", "x-generates": "declarations/GLO-<番号>.json",
           "x-view": {"label": "header.name"},
           "properties": {
               "$schema": field(json!({"const": "lexicon/glossary@v1"})),
               "kind": field(json!({"const": "glossary"})),
               "id": field(json!({"pattern": "^GLO-[0-9]+$", "x-view": {"hidden": true}})),
               "terms": field(json!({"type": "array", "x-foo": 1, "items": {"type": "object", "properties": {
                   "word": {"type": "string", "x-view": {"text": "{word | quote(@)}"}}}}}))}})
}

#[test]
fn check_schemas_tool_reads_a_directory_of_schemas() {
    let dir = Scratch::new("check-schemas-dir");
    dir.put("glossary.schema.json", &with_undeclared_annotation());
    dir.put_text("GLO-1.json", "{}");
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/schemas", eq(json!(["glossary.schema.json"]))),
                ("/findings", len(1)),
                ("/findings/0/rule", eq("申告していない注釈")),
            ],
        )],
    );
}
