//! 値の入口（sas-data）から読む所の取り出し。

#![allow(clippy::unwrap_used)]

use outbound_javascript::Boa;
use rendering_application::ports::{DataRead, PathSeg, ScriptReader};

fn key(s: &str) -> PathSeg {
    PathSeg::Key(s.to_string())
}

#[test]
fn paths_from_the_entry_are_read() {
    let src = r#"
        const data = JSON.parse(document.getElementById('sas-data').textContent);
        el.textContent = data.goal;
        const first = data.calls[0].id;
        const { owner } = data;
    "#;
    let f = Boa.data_reads(src, "sas-data").unwrap();
    assert!(f.reads_entry);
    assert_eq!(
        f.reads,
        vec![
            DataRead::Path(vec![key("goal")]),
            DataRead::Path(vec![key("calls"), PathSeg::Index, key("id")]),
            DataRead::Path(vec![key("owner")]),
        ]
    );
}

#[test]
fn computed_keys_and_whole_values_are_untraceable() {
    let src = r#"
        const data = JSON.parse(document.getElementById("sas-data").textContent);
        const v = data[k];
        render(data);
    "#;
    let f = Boa.data_reads(src, "sas-data").unwrap();
    assert_eq!(f.reads.len(), 2);
    assert!(
        f.reads
            .iter()
            .all(|r| matches!(r, DataRead::Untraceable(_)))
    );
}

#[test]
fn a_script_without_the_entry_reads_nothing() {
    let f = Boa
        .data_reads("document.title = 'x'; const y = other.goal;", "sas-data")
        .unwrap();
    assert!(!f.reads_entry);
    assert!(f.reads.is_empty());
}

#[test]
fn broken_js_is_an_error() {
    assert!(Boa.data_reads("function ( {", "sas-data").is_err());
}
