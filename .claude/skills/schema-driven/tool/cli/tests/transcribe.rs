//! UC-9 基盤の複製を転写する ・ UC-10 複製と正本の差分を検査する。
//! 正本の tool/core ・ tool/adapters ・ references を、同じ並びのまま利用側の Skill の tool/schema-driven/ の下へ写す。
//! 写した複製は具体の持ち物で、基盤の新しい版へ更新するときも、具体が変えたファイルは上書きしない。

use schema_driven_adapters::outbound::fs::FileSystem;
use schema_driven_core::application::transcriptions::Transcriptions;
use schema_driven_core::ports::inbound::TranscriptionUseCases;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const CONFLICT_SUFFIX: &str = ".schema-driven-new";

fn write_file(root: &Path, path: &str, text: &str) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, text).unwrap();
}

/// 正本（schema-driven の Skill）と、空の利用側の Skill を置く。
fn setup(name: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("sd-transcribe-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let master = root.join("schema-driven");
    write_file(
        &master,
        "tool/core/Cargo.toml",
        "[package]\nname = \"schema-driven-core\"\nversion = \"0.1.0\"\n",
    );
    write_file(&master, "tool/core/src/lib.rs", "pub fn base() {}\n");
    write_file(&master, "tool/core/src/view.rs", "pub fn view() {}\n");
    write_file(
        &master,
        "tool/core/tests/only_in_master.rs",
        "// 写さない\n",
    );
    write_file(&master, "tool/adapters/src/lib.rs", "pub fn adapter() {}\n");
    write_file(&master, "tool/target/debug/build.log", "写さない\n");
    write_file(&master, "references/meta-schema.json", "{}\n");
    let skill = root.join("acdr");
    fs::create_dir_all(&skill).unwrap();
    (master, skill)
}

fn use_cases(master: &Path) -> Transcriptions {
    Transcriptions::new(Arc::new(FileSystem), master.to_string_lossy().into_owned())
}

fn copy_of(skill: &Path, path: &str) -> PathBuf {
    skill.join("tool/schema-driven").join(path)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn kinds(transcriptions: &Transcriptions, skill: &Path) -> Vec<(String, String)> {
    transcriptions
        .check_copy(skill.to_str().unwrap())
        .unwrap()
        .differences
        .iter()
        .map(|difference| (difference.path.clone(), difference.kind.clone()))
        .collect()
}

fn pair(path: &str, kind: &str) -> (String, String) {
    (path.to_owned(), kind.to_owned())
}

#[test]
fn transcribe_copies_crates_and_references_and_records_version_and_hashes() {
    let (master, skill) = setup("copy");
    let done = use_cases(&master)
        .transcribe(skill.to_str().unwrap())
        .unwrap();
    assert_eq!(
        read(&copy_of(&skill, "tool/core/src/lib.rs")),
        "pub fn base() {}\n"
    );
    assert!(copy_of(&skill, "tool/adapters/src/lib.rs").exists());
    assert!(copy_of(&skill, "references/meta-schema.json").exists());
    assert!(!copy_of(&skill, "tool/core/tests/only_in_master.rs").exists());
    assert!(!copy_of(&skill, "tool/target").exists());
    let record = read(&copy_of(&skill, "transcription.json"));
    assert!(record.contains("\"version\": \"0.1.0\""), "{record}");
    assert_eq!(
        (done.from_version, done.to_version.as_str()),
        (None, "0.1.0")
    );
    assert_eq!(done.written.len(), 5, "{:?}", done.written);
    fs::remove_dir_all(master.parent().unwrap()).unwrap();
}

#[test]
fn check_reports_nothing_right_after_transcribing_and_never_writes() {
    let (master, skill) = setup("clean");
    let transcriptions = use_cases(&master);
    transcriptions.transcribe(skill.to_str().unwrap()).unwrap();
    let before = read(&copy_of(&skill, "transcription.json"));
    let report = transcriptions.check_copy(skill.to_str().unwrap()).unwrap();
    assert!(report.differences.is_empty(), "{report:?}");
    assert_eq!(report.copy_version.as_deref(), Some("0.1.0"));
    assert_eq!(report.master_version, "0.1.0");
    assert_eq!(read(&copy_of(&skill, "transcription.json")), before);
    fs::remove_dir_all(master.parent().unwrap()).unwrap();
}

#[test]
fn check_tells_master_changes_concrete_changes_and_conflicts_apart() {
    let (master, skill) = setup("diff");
    let transcriptions = use_cases(&master);
    transcriptions.transcribe(skill.to_str().unwrap()).unwrap();
    write_file(
        &master,
        "tool/core/Cargo.toml",
        "[package]\nname = \"schema-driven-core\"\nversion = \"0.2.0\"\n",
    );
    write_file(
        &master,
        "tool/core/src/lib.rs",
        "pub fn base() { /* 0.2.0 */ }\n",
    );
    write_file(
        &skill,
        "tool/schema-driven/tool/adapters/src/lib.rs",
        "pub fn adapter() { /* 具体 */ }\n",
    );
    write_file(
        &master,
        "tool/core/src/view.rs",
        "pub fn view() { /* 0.2.0 */ }\n",
    );
    write_file(
        &skill,
        "tool/schema-driven/tool/core/src/view.rs",
        "pub fn view() { /* 具体 */ }\n",
    );
    write_file(&master, "references/document.schema.json", "{}\n");
    fs::remove_file(master.join("references/meta-schema.json")).unwrap();
    write_file(
        &skill,
        "tool/schema-driven/tool/core/src/extra.rs",
        "// 具体が足した\n",
    );
    let report = transcriptions.check_copy(skill.to_str().unwrap()).unwrap();
    assert_eq!(
        (
            report.copy_version.as_deref(),
            report.master_version.as_str()
        ),
        (Some("0.1.0"), "0.2.0")
    );
    assert_eq!(
        kinds(&transcriptions, &skill),
        vec![
            pair("references/document.schema.json", "正本に増えた"),
            pair("references/meta-schema.json", "正本から消えた"),
            pair("tool/adapters/src/lib.rs", "具体が変えた"),
            pair("tool/core/Cargo.toml", "正本が新しくなった"),
            pair("tool/core/src/extra.rs", "具体が足した"),
            pair("tool/core/src/lib.rs", "正本が新しくなった"),
            pair("tool/core/src/view.rs", "衝突"),
        ]
    );
    fs::remove_dir_all(master.parent().unwrap()).unwrap();
}

#[test]
fn updating_to_a_new_version_never_overwrites_what_the_concrete_changed() {
    let (master, skill) = setup("update");
    let transcriptions = use_cases(&master);
    transcriptions.transcribe(skill.to_str().unwrap()).unwrap();
    // 基盤の新しい版
    write_file(
        &master,
        "tool/core/Cargo.toml",
        "[package]\nname = \"schema-driven-core\"\nversion = \"0.2.0\"\n",
    );
    write_file(
        &master,
        "tool/core/src/lib.rs",
        "pub fn base() { /* 0.2.0 */ }\n",
    );
    write_file(
        &master,
        "tool/core/src/view.rs",
        "pub fn view() { /* 0.2.0 */ }\n",
    );
    fs::remove_file(master.join("references/meta-schema.json")).unwrap();
    // 具体の変更
    write_file(
        &skill,
        "tool/schema-driven/tool/adapters/src/lib.rs",
        "pub fn adapter() { /* 具体 */ }\n",
    );
    write_file(
        &skill,
        "tool/schema-driven/tool/core/src/view.rs",
        "pub fn view() { /* 具体 */ }\n",
    );
    let done = transcriptions.transcribe(skill.to_str().unwrap()).unwrap();
    assert_eq!(
        (done.from_version.as_deref(), done.to_version.as_str()),
        (Some("0.1.0"), "0.2.0")
    );
    // 基盤だけが変えたものは、新しい版になる
    assert_eq!(
        read(&copy_of(&skill, "tool/core/src/lib.rs")),
        "pub fn base() { /* 0.2.0 */ }\n"
    );
    assert!(!copy_of(&skill, "references/meta-schema.json").exists());
    // 具体だけが変えたものは、そのまま
    assert_eq!(
        read(&copy_of(&skill, "tool/adapters/src/lib.rs")),
        "pub fn adapter() { /* 具体 */ }\n"
    );
    // 両方が変えたものは、具体のものを残し、新しい版を隣に置く
    assert_eq!(
        read(&copy_of(&skill, "tool/core/src/view.rs")),
        "pub fn view() { /* 具体 */ }\n"
    );
    assert_eq!(
        read(&copy_of(
            &skill,
            &format!("tool/core/src/view.rs{CONFLICT_SUFFIX}")
        )),
        "pub fn view() { /* 0.2.0 */ }\n"
    );
    assert_eq!(done.conflicts, vec!["tool/core/src/view.rs".to_owned()]);
    assert_eq!(done.kept, vec!["tool/adapters/src/lib.rs".to_owned()]);
    assert_eq!(done.removed, vec!["references/meta-schema.json".to_owned()]);
    // 更新のあと：衝突の新しい版が残っていることを知らせる。具体が合わせて新しい版を消せば、具体の変更だけが残る
    assert_eq!(
        kinds(&transcriptions, &skill),
        vec![
            pair("tool/adapters/src/lib.rs", "具体が変えた"),
            pair("tool/core/src/view.rs", "具体が変えた"),
            pair(
                &format!("tool/core/src/view.rs{CONFLICT_SUFFIX}"),
                "衝突の新しい版が残っている"
            ),
        ]
    );
    fs::remove_file(copy_of(
        &skill,
        &format!("tool/core/src/view.rs{CONFLICT_SUFFIX}"),
    ))
    .unwrap();
    write_file(
        &skill,
        "tool/schema-driven/tool/core/src/view.rs",
        "pub fn view() { /* 0.2.0 */ }\n",
    );
    assert_eq!(
        kinds(&transcriptions, &skill),
        vec![pair("tool/adapters/src/lib.rs", "具体が変えた")]
    );
    fs::remove_dir_all(master.parent().unwrap()).unwrap();
}

#[test]
fn files_the_concrete_changed_survive_even_when_the_master_removes_them() {
    let (master, skill) = setup("removed");
    let transcriptions = use_cases(&master);
    transcriptions.transcribe(skill.to_str().unwrap()).unwrap();
    write_file(
        &skill,
        "tool/schema-driven/references/meta-schema.json",
        "{\"具体\": true}\n",
    );
    fs::remove_file(master.join("references/meta-schema.json")).unwrap();
    let done = transcriptions.transcribe(skill.to_str().unwrap()).unwrap();
    assert_eq!(
        read(&copy_of(&skill, "references/meta-schema.json")),
        "{\"具体\": true}\n"
    );
    assert_eq!(done.kept, vec!["references/meta-schema.json".to_owned()]);
    assert!(done.removed.is_empty());
    fs::remove_dir_all(master.parent().unwrap()).unwrap();
}
