// SPDX-License-Identifier: MIT
//! 目録 ── 台帳 ・ 見本 ・ 部品の本文と、ずれていないか。
//!
//! 目録は利用側（変換器を書く人）が唯一見る面なので、ここがずれると、外から見て正しいのに
//! 動かない、という一番たちの悪い壊れ方をする。

use std::collections::{BTreeMap, BTreeSet};

use ds_parts::catalog::{catalog, examples, props_of, PARTS};
use ds_parts::registry::known_kinds;
use serde_json::Value;

fn cat() -> Value {
    catalog().expect("目録を組める")
}

#[test]
fn every_registered_component_is_listed() {
    let c = cat();
    let mut listed: Vec<&str> = c["parts"]
        .as_object()
        .expect("対応表")
        .keys()
        .map(String::as_str)
        .collect();
    listed.sort_unstable();
    assert_eq!(listed, known_kinds());
}

#[test]
fn every_component_has_an_example() {
    let ex = examples();
    let mut have: Vec<&str> = ex.keys().map(String::as_str).collect();
    have.sort_unstable();
    assert_eq!(have, known_kinds());
}

#[test]
fn unknown_component_is_refused_by_name() {
    let err = props_of("知らない部品").expect_err("断る");
    assert!(err.contains("box"), "{err}");
}

#[test]
fn examples_stay_within_the_catalog() {
    let c = cat();
    let ex = examples();
    for kind in known_kinds() {
        let entry = &c["parts"][kind];
        let declared: BTreeSet<&str> = entry["props"]
            .as_object()
            .expect("対応表")
            .keys()
            .map(String::as_str)
            .collect();
        let given: BTreeSet<&str> = ex[kind]
            .as_object()
            .expect("対応表")
            .keys()
            .map(String::as_str)
            .collect();
        let forwards_by_value = entry
            .get("forwards_to")
            .and_then(Value::as_str)
            .is_some_and(|s| s.starts_with("props:"));
        if !forwards_by_value {
            let extra: Vec<&&str> = given
                .iter()
                .filter(|k| **k != "label" && !declared.contains(**k))
                .collect();
            assert!(extra.is_empty(), "{kind}: 目録に無いキー {extra:?}");
        }
        let need: Vec<&str> = entry["props"]
            .as_object()
            .expect("対応表")
            .iter()
            .filter(|(_, v)| v["required"] == true)
            .map(|(k, _)| k.as_str())
            .filter(|k| !given.contains(k))
            .collect();
        assert!(need.is_empty(), "{kind}: 見本に足りない {need:?}");
    }
}

#[test]
fn catalog_is_usable() {
    let c = cat();
    assert!(serde_json::to_string(&c).is_ok());
    let d = &c["declaration"];
    assert_eq!(d["nodes"]["id"]["required"], true);
    assert_eq!(d["edges"]["from"]["required"], true);
    assert_eq!(d["edges"]["to"]["required"], true);
    assert_eq!(d["groups"]["members"]["required"], true);
    assert_eq!(c["parts"]["pie"]["forwards_to"], "donut");
    assert_eq!(c["parts"]["titled"]["forwards_to"], "props:of");
    assert!(c["parts"]["pie"]["props"].get("centre").is_some());
    assert!(!c["parts"]["pie"]["reads_itself"]
        .as_array()
        .expect("並び")
        .iter()
        .any(|x| x == "centre"));
    let kinds: BTreeSet<&str> = c["parts"]
        .as_object()
        .expect("対応表")
        .values()
        .filter_map(|p| p["placement"].as_str())
        .collect();
    assert!(
        kinds.iter().all(|k| ["own-origin", "absolute"].contains(k)),
        "{kinds:?}"
    );
    assert_eq!(
        c["tokens"]["font.size"]["range"],
        serde_json::json!([8.0, 40.0])
    );
    assert_eq!(c["roles"]["focus"]["color.box-stroke"], "color.accent");
}

#[test]
fn no_claim_vocabulary_leaks_in() {
    let text = serde_json::to_string(&cat()).expect("JSON");
    for word in ["主張", "asserts", "Document", "Schema", "読み方"] {
        assert!(
            !text.contains(word),
            "目録に利用側の語彙が混ざっている: {word}"
        );
    }
}

// ── 表と本文の突き合わせ ──

fn sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut all = String::new();
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .expect("在る")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("shapes"))
        })
        .collect();
    names.sort();
    for p in names {
        all.push_str(&std::fs::read_to_string(p).expect("読める"));
        all.push('\n');
    }
    all
}

/// `fn 名前(` の本文を、波括弧の対応で切り出す。
fn body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let at = src.find(&format!("fn {name}("))?;
    let open = at + src[at..].find('{')?;
    let mut depth = 0;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[open..=open + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// 台帳の行 `("名前", 関数)` から、部品の名前 → 関数名。**`register` の本文だけを読む。**
fn registered(src: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut rest = src;
    while let Some(at) = rest.find("pub fn register()") {
        rest = &rest[at..];
        let b = body(rest, "register").expect("本文が在る");
        let mut r = b;
        while let Some(i) = r.find("(\"") {
            r = &r[i + 2..];
            let Some((kind, tail)) = r.split_once("\", ") else {
                break;
            };
            let func: String = tail
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            out.insert(kind.to_owned(), func);
            r = tail;
        }
        rest = &rest[b.len()..];
    }
    out
}

/// 本文が入力（`p`）から読むキー。**入力を丸ごと渡した先の関数も辿る。**
fn keys_read(src: &str, func: &str, seen: &mut BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if !seen.insert(func.to_owned()) {
        return out;
    }
    let Some(b) = body(src, func) else { return out };
    // **書式の折り返しを詰める** ── `p` と `.get(` のあいだの改行は、読み方の違いではない
    let squeezed = squeeze(b);
    let b: &str = &squeezed;
    for pat in ["(p, \"", "p.get(\"", "p[\"", "p.contains_key(\""] {
        let mut rest = b;
        while let Some(at) = rest.find(pat) {
            rest = &rest[at + pat.len()..];
            if let Some(end) = rest.find('"') {
                out.insert(rest[..end].to_owned());
            }
        }
    }
    // 入力を渡した先 ── `名前(p, ` の形
    let mut rest = b;
    while let Some(at) = rest.find("(p, ") {
        let head = &rest[..at];
        let callee: String = head
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        rest = &rest[at + 1..];
        if !callee.is_empty()
            && !head.ends_with("props::")
            && !head.ends_with(&format!("props::{callee}"))
        {
            out.extend(keys_read(src, &callee, seen));
        }
    }
    out
}

/// 空白の並びのあとに `.` が来るとき、その空白を消す。
fn squeeze(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending = String::new();
    for c in s.chars() {
        if c.is_whitespace() {
            pending.push(c);
            continue;
        }
        if c != '.' {
            out.push_str(&pending);
        }
        pending.clear();
        out.push(c);
    }
    out
}

#[test]
fn the_table_matches_what_each_component_reads() {
    // **手書きの目録は、実物とずれても誰も気づけない。** 表と本文が食い違えば、ここで失敗する
    let src = sources();
    let reg = registered(&src);
    assert_eq!(
        reg.keys().map(String::as_str).collect::<Vec<_>>(),
        known_kinds(),
        "台帳の行を読み取れている"
    );
    let mut bad = Vec::new();
    for (kind, props, _) in PARTS {
        let func = &reg[*kind];
        let read = keys_read(&src, func, &mut BTreeSet::new());
        let listed: BTreeSet<String> = props.iter().map(|(k, _, _)| (*k).to_owned()).collect();
        if read != listed {
            bad.push(format!(
                "{kind}: 本文だけ {:?} ／ 表だけ {:?}",
                read.difference(&listed).collect::<Vec<_>>(),
                listed.difference(&read).collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "目録の表と部品の本文が食い違う:\n{}",
        bad.join("\n")
    );
}
