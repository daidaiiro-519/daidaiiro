//! 論点3の案：基盤が実体1件を描画し、concrete が集合を目的の連鎖として組む。
//! 上位の目的は、x-ref に付けた x-purpose（upward ＝参照先が目的 ／ downward ＝参照元が目的）で申告する。
use crate::base;
use crate::usecase_q2::{load_all, Decl};
use serde_json::Value;
use std::path::Path;

pub struct Ref { pub to_decl: String, pub to_item: Option<String>, pub kind: String, pub item_field: Option<String>, pub purpose: Option<String> }

fn id(d: &Decl) -> String { base::get(&d.doc, "id").as_str().unwrap_or("").to_string() }
fn name(d: &Decl) -> String { base::get(&d.doc, "name").as_str().unwrap_or("").to_string() }

pub fn refs(d: &Decl) -> Vec<Ref> {
    let mut out = vec![];
    for (k, p) in d.schema["properties"].as_object().unwrap() {
        let Some(x) = p.get("x-ref") else { continue };
        let (kind, item_field) = match x {
            Value::String(s) => (s.clone(), None),
            o => (o["kind"].as_str().unwrap().to_string(), o["item"].as_str().map(String::from)),
        };
        for v in base::get(&d.doc, &format!("[{k}][] | []")).as_array().unwrap() {
            let s = v.as_str().unwrap();
            let (to_decl, to_item) = match s.split_once('.') { Some((a, b)) => (a.to_string(), Some(b.to_string())), None => (s.to_string(), None) };
            out.push(Ref { to_decl, to_item, kind: kind.clone(), item_field: item_field.clone(), purpose: p["x-purpose"].as_str().map(String::from) });
        }
    }
    out
}

fn find<'a>(all: &'a [Decl], i: &str) -> Option<&'a Decl> { all.iter().find(|d| id(d) == i) }

fn item_name(d: &Decl, field: &str, item: &str) -> Option<String> {
    let v = base::get(&d.doc, &format!("{field}[?id == '{item}'].name | [0]"));
    v.as_str().map(String::from)
}

/// 集合の検証（論点2）に、参照先の項目の解決と、上位の目的の有無を足す。根は種類 root_kind の宣言である。
pub fn validate_set(all: &[Decl], root_kind: &str) -> Vec<String> {
    let mut errs = vec![];
    for d in all {
        for r in refs(d) {
            match find(all, &r.to_decl) {
                None => errs.push(format!("{} が指す {} が無い", id(d), r.to_decl)),
                Some(t) if t.kind != r.kind => errs.push(format!("{} が指す {} は {} でない", id(d), r.to_decl, r.kind)),
                Some(t) => if let (Some(it), Some(f)) = (&r.to_item, &r.item_field) {
                    if item_name(t, f, it).is_none() { errs.push(format!("{} が指す {}.{it} が無い", id(d), r.to_decl)); }
                },
            }
        }
        if d.kind != root_kind && uppers(all, d).is_empty() {
            errs.push(format!("{} は上位の目的を保持しない", id(d)));
        }
    }
    errs
}

/// この宣言の上位の目的（1段上）。
fn uppers<'a>(all: &'a [Decl], d: &Decl) -> Vec<(String, Option<&'a Decl>)> {
    let mut v: Vec<(String, Option<&Decl>)> = refs(d).into_iter().filter(|r| r.purpose.as_deref() == Some("upward")).filter_map(|r| {
        let t = find(all, &r.to_decl)?;
        let label = match (&r.to_item, &r.item_field) {
            (Some(it), Some(f)) => format!("{} › {it} {}", name(t), item_name(t, f, it).unwrap_or_default()),
            _ => name(t),
        };
        Some((label, None))
    }).collect();
    for s in all {
        if refs(s).iter().any(|r| r.purpose.as_deref() == Some("downward") && r.to_decl == id(d)) {
            v.push((name(s), Some(s)));
        }
    }
    v
}

/// 目的の連鎖：根から、この宣言までの名前の列。
pub fn chain(all: &[Decl], d: &Decl) -> String {
    match uppers(all, d).into_iter().next() {
        None => name(d),
        Some((label, Some(up))) => format!("{} › {}", chain(all, up), name(d)).replace(&format!("{label} › {label}"), &label),
        Some((label, None)) => format!("{label} › {}", name(d)),
    }
}

/// 頁の集まりを組む。根の頁を先頭に置き、どの頁も先頭に目的の連鎖を、末尾に逆参照を置く。
pub fn render_site(dir: &Path, root_kind: &str) -> String {
    let all = load_all(dir);
    let mut order: Vec<&Decl> = all.iter().filter(|d| d.kind == root_kind).collect();
    let mut rest: Vec<&Decl> = all.iter().filter(|d| d.kind != root_kind).collect();
    rest.sort_by_key(|d| chain(&all, d).matches(" › ").count());
    order.extend(rest);
    let mut out = String::new();
    for d in order {
        out += "---\n\n";
        if d.kind != root_kind {
            out += &format!("> 目的の連鎖：{}\n\n", chain(&all, d));
        }
        let fmt = |k: &str, v: &str| -> Option<String> {
            d.schema["properties"][k].get("x-ref")?;
            let (decl, item) = v.split_once('.').map_or((v, None), |(a, b)| (a, Some(b)));
            let t = find(&all, decl)?;
            Some(match (item, d.schema["properties"][k]["x-ref"]["item"].as_str()) {
                (Some(it), Some(f)) => format!("{v} {} › {}", name(t), item_name(t, f, it)?),
                _ => format!("{v} {}", name(t)),
            })
        };
        out += &base::render_with(&d.schema, &d.doc, &fmt);
        let back: Vec<String> = all.iter().filter(|s| refs(s).iter().any(|r| r.to_decl == id(d))).map(|s| format!("{} {}", id(s), name(s))).collect();
        if !back.is_empty() {
            out += &format!("## この宣言を指す要素\n\n{}\n\n", back.iter().map(|b| format!("- {b}")).collect::<Vec<_>>().join("\n"));
        }
    }
    out
}
