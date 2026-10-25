//! 結合テスト：Files が契約を読み書きする

use semantic_agent_schema_application::ports::contracts::Contracts;
use semantic_agent_schema_fs::files::Files;
use serde_json::json;

fn work(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sas-fs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// traces: U1:事後
#[test]
fn written_document_is_read_back() {
    let files = Files::new(work("roundtrip"));
    files.write("spec/a.schema.json", &json!({"a": 1})).unwrap();
    assert_eq!(
        files.read("spec/a.schema.json").unwrap(),
        Some(json!({"a": 1}))
    );
}

/// traces: U1:失敗時
#[test]
fn missing_file_is_none() {
    let files = Files::new(work("missing"));
    assert_eq!(files.read("spec/none.schema.json").unwrap(), None);
}

/// traces: I4
#[test]
fn document_is_found_by_its_id() {
    let files = Files::new(work("find"));
    files
        .write(
            "g/a.schema.json",
            &json!({"$id": "urn:semantic-agent-schema:a:glossary:v1"}),
        )
        .unwrap();
    assert!(
        files
            .find("urn:semantic-agent-schema:a:glossary:v1")
            .unwrap()
            .is_some()
    );
}

/// traces: I4
#[test]
fn two_documents_with_the_same_id_are_an_error() {
    let files = Files::new(work("twice"));
    files.write("a.json", &json!({"$id": "urn:x"})).unwrap();
    files.write("b.json", &json!({"$id": "urn:x"})).unwrap();
    assert!(files.find("urn:x").is_err());
}

/// traces: U1:失敗時
#[test]
fn path_outside_the_root_is_refused() {
    let files = Files::new(work("outside"));
    assert!(files.write("../escape.json", &json!({})).is_err());
}

/// traces: I4
#[test]
fn document_in_a_dot_folder_is_found() {
    let files = Files::new(work("dot"));
    files
        .write(".records/g.json", &json!({"$id": "urn:y"}))
        .unwrap();
    assert!(files.find("urn:y").unwrap().is_some());
}

/// traces: I4
#[test]
fn broken_document_with_the_id_is_an_error() {
    let dir = work("broken");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("g.json"), "{\"$id\": \"urn:z\",").unwrap();
    assert!(Files::new(dir).find("urn:z").is_err());
}
