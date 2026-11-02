//! 設計のファイル（architecture.json ・ system-tests/trace.json ・ untested.json）が、$schema で指す自分の schema に従う。

#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &PathBuf) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("{} が JSON でない: {e}", path.display()))
}

/// ファイルと、その $schema の形を置いた所。
const FILES: [(&str, &str); 3] = [
    ("architecture.json", "docs/design/architecture.schema.json"),
    (
        "skills/system-tests/trace.json",
        "skills/system-tests/trace.schema.json",
    ),
    (
        "skills/system-tests/untested.json",
        "skills/system-tests/untested.schema.json",
    ),
];

#[test]
fn design_files_follow_their_schema() {
    for (file, schema) in FILES {
        let value = read(&root().join(file));
        let schema = read(&root().join(schema));
        assert_eq!(
            value.get("$schema"),
            schema.get("$id"),
            "{file} の $schema が、形の $id と違う"
        );
        let v = jsonschema::validator_for(&schema).expect("形を組めること");
        let errors: Vec<String> = v.iter_errors(&value).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{file} が形に従わない: {errors:?}");
    }
}
