//! 記録で Concrete に従う文書を引く：その経路の最後の変更が、その $id に従うと書いたものだけ。消した文書は除く。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use writing_domain::values::ChangeRow;

fn row(
    kind: &str,
    path: &str,
    schema: &str,
    before: Option<&str>,
    after: Option<&str>,
) -> ChangeRow {
    ChangeRow {
        kind: kind.into(),
        source: "sas".into(),
        target_path: path.into(),
        request_id: None,
        before: before.map(Into::into),
        after: after.map(Into::into),
        reason: Some("試す".into()),
        schema: Some(schema.into()),
        fields: Vec::new(),
    }
}

#[test]
fn only_the_last_change_of_each_path_counts() {
    let mut log = SqliteLog::in_memory().unwrap();
    let a = "urn:a";
    let b = "urn:b";
    log.append_change(None, &row("create", "u1.json", a, None, Some("sha256:1")))
        .unwrap();
    log.append_change(None, &row("create", "u2.json", a, None, Some("sha256:2")))
        .unwrap();
    log.append_change(
        None,
        &row("update", "u2.json", b, Some("sha256:2"), Some("sha256:3")),
    )
    .unwrap();
    log.append_change(None, &row("create", "u3.json", a, None, Some("sha256:4")))
        .unwrap();
    log.append_change(None, &row("delete", "u3.json", a, Some("sha256:4"), None))
        .unwrap();
    assert_eq!(log.instances_of(a).unwrap(), ["u1.json"]);
    assert_eq!(log.instances_of(b).unwrap(), ["u2.json"]);
}
