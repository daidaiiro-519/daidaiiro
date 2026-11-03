//! ACDR 0224：古い置き場所の記録を、新しい置き場所へ1回だけ移す。
//! - チェックポイントをかけてから移すので、WAL にした記録の行も新しい置き場所で読め、WAL のファイルを残さない
//! - 古い置き場所には案内のファイルを置き、古い版の開き方（SqliteLog::open と同じ手順）で開くと断られ、案内は変わらない
//! - 案内のファイルは記録でないので、2回目は何もしない
//! - 新しい置き場所に記録が既にあれば、どちらも触らずに断る

#![allow(clippy::unwrap_used)]

use outbound_sqlite::{SqliteLog, move_records};
use rusqlite::Connection;
use writing_domain::values::ChangeRow;

const NOTICE: &str = "Semantic Agent Schema の記録は新しい置き場所へ移した。\n";

fn create() -> ChangeRow {
    ChangeRow {
        kind: "create".into(),
        source: "sas".into(),
        target_path: "u1.json".into(),
        request_id: None,
        before: None,
        after: Some("sha256:a".into()),
        reason: None,
        schema: None,
        fields: Vec::new(),
    }
}

fn place(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sas-move-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn records_move_once_and_the_old_version_cannot_open_the_notice() {
    let dir = place("once");
    let old = dir.join("sas").join("sas.db");
    let new = dir.join("semantic-agent-schema").join("records.sqlite");
    {
        let mut log = SqliteLog::open(&old).unwrap();
        log.append_change(None, &create()).unwrap();
        // WAL にしてから1行足す
        log.connection()
            .pragma_update(None, "journal_mode", "WAL")
            .unwrap();
        log.connection()
            .pragma_update(None, "wal_autocheckpoint", 0)
            .unwrap();
        let mut row = create();
        row.target_path = "u2.json".into();
        log.append_change(None, &row).unwrap();
    }
    assert!(move_records(&old, &new, NOTICE).unwrap());
    let log = SqliteLog::open(&new).unwrap();
    assert_eq!(log.writes("u1.json").unwrap().len(), 1);
    assert_eq!(log.writes("u2.json").unwrap().len(), 1);
    drop(log);
    assert!(!std::path::Path::new(&format!("{}-wal", old.display())).exists());
    assert_eq!(std::fs::read_to_string(&old).unwrap(), NOTICE);
    // 古い版は、古い置き場所を開いて表を作ろうとする。案内のファイルはデータベースでないので断られ、案内は変わらない
    assert!(SqliteLog::open(&old).is_err());
    assert!(
        Connection::open(&old)
            .unwrap()
            .execute_batch("CREATE TABLE t (x)")
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(&old).unwrap(), NOTICE);
    // 2回目は移さない
    assert!(!move_records(&old, &new, NOTICE).unwrap());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn nothing_moves_when_both_places_have_records() {
    let dir = place("both");
    let old = dir.join("sas").join("sas.db");
    let new = dir.join("semantic-agent-schema").join("records.sqlite");
    SqliteLog::open(&old)
        .unwrap()
        .append_change(None, &create())
        .unwrap();
    SqliteLog::open(&new)
        .unwrap()
        .append_change(None, &create())
        .unwrap();
    let before = (std::fs::read(&old).unwrap(), std::fs::read(&new).unwrap());
    let message = move_records(&old, &new, NOTICE).unwrap_err();
    assert!(message.contains("2か所"), "{message}");
    assert_eq!(
        (std::fs::read(&old).unwrap(), std::fs::read(&new).unwrap()),
        before
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn nothing_moves_without_old_records() {
    let dir = place("none");
    let old = dir.join("sas").join("sas.db");
    let new = dir.join("semantic-agent-schema").join("records.sqlite");
    assert!(!move_records(&old, &new, NOTICE).unwrap());
    assert!(!new.exists());
    std::fs::remove_dir_all(&dir).unwrap();
}
