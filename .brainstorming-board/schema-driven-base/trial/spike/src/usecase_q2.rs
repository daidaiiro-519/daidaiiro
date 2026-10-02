//! 論点2の案：concrete は、スキーマの注釈（x-ref ・ x-test-spec）で、参照とテスト仕様の在りかを申告する。
//! 検証は宣言の集合に対して行う ── Schema ＋ ID の一意 ＋ 参照の解決。
use crate::base;
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

pub struct Decl { pub path: PathBuf, pub kind: String, pub schema: Value, pub doc: Value }

pub fn load_all(dir: &Path) -> Vec<Decl> {
    let mut v: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    v.sort();
    v.into_iter().map(|path| {
        let doc = base::load(&path);
        let sp = path.parent().unwrap().join(doc["$schema"].as_str().expect("$schema が無い"));
        let kind = sp.file_name().unwrap().to_str().unwrap().split('.').next().unwrap().to_string();
        Decl { schema: base::load(&sp), path, kind, doc }
    }).collect()
}

fn annotated<'a>(schema: &'a Value, key: &str) -> Vec<(&'a String, &'a Value)> {
    schema["properties"].as_object().unwrap().iter().filter(|(_, p)| p.get(key).is_some()).map(|(k, p)| (k, &p[key])).collect()
}

fn id(doc: &Value) -> String { base::get(doc, "id").as_str().unwrap_or("").to_string() }

/// テスト仕様：x-test-spec の付いた欄の項目の ID を「宣言の ID.項目の ID」で返す。
pub fn spec_items(decls: &[Decl]) -> Vec<String> {
    decls.iter().flat_map(|d| annotated(&d.schema, "x-test-spec").into_iter().flat_map(move |(k, _)| {
        let ids = base::get(&d.doc, &format!("{k}[].id"));
        ids.as_array().unwrap().iter().map(|i| format!("{}.{}", id(&d.doc), i.as_str().unwrap())).collect::<Vec<_>>()
    })).collect()
}

/// 参照：x-ref の付いた欄の値と、指す先の種類。
pub fn refs(d: &Decl) -> Vec<(String, String)> {
    annotated(&d.schema, "x-ref").into_iter().flat_map(|(k, to)| {
        let v = base::get(&d.doc, &format!("[{k}][] | []"));
        v.as_array().unwrap().iter().map(|x| (x.as_str().unwrap().to_string(), to.as_str().unwrap().to_string())).collect::<Vec<_>>()
    }).collect()
}

/// 集合の検証：宣言の ID の一意 ・ 項目の ID の一意 ・ 参照の解決。
pub fn validate_set(decls: &[Decl]) -> Vec<String> {
    let mut errs = vec![];
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for d in decls {
        if seen.insert(id(&d.doc), &d.kind).is_some() {
            errs.push(format!("宣言の ID が重複している: {}", id(&d.doc)));
        }
    }
    let spec = spec_items(decls);
    for (i, s) in spec.iter().enumerate() {
        if spec[..i].contains(s) {
            errs.push(format!("項目の ID が重複している: {s}"));
        }
    }
    for d in decls {
        for (to_id, to_kind) in refs(d) {
            match seen.get(&to_id) {
                None => errs.push(format!("{} が指す {to_id} が無い", id(&d.doc))),
                Some(k) if *k != to_kind => errs.push(format!("{} が指す {to_id} は {to_kind} でない", id(&d.doc))),
                _ => {}
            }
        }
    }
    errs
}

/// 更新：基盤の更新に、集合の検証を注入する。
pub fn update(dir: &Path, target: &Path, patch: Value) -> Result<(), Vec<String>> {
    let all = load_all(dir);
    let me = all.iter().find(|d| d.path == target).unwrap();
    let check = |after: &Value| {
        let set: Vec<Decl> = all.iter().map(|d| Decl { path: d.path.clone(), kind: d.kind.clone(), schema: d.schema.clone(),
            doc: if d.path == target { after.clone() } else { d.doc.clone() } }).collect();
        validate_set(&set)
    };
    base::update_with(target, &me.schema, patch, &check)
}

/// 削除（実体）：除いた集合が検証を通過したときだけ、基盤の削除を呼ぶ。
pub fn delete(dir: &Path, target: &Path) -> Result<(), Vec<String>> {
    let rest: Vec<Decl> = load_all(dir).into_iter().filter(|d| d.path != target).collect();
    let errs = validate_set(&rest);
    if !errs.is_empty() {
        return Err(errs);
    }
    base::delete(target);
    Ok(())
}

pub struct Drift { pub missing: Vec<String>, pub extra: Vec<String>, pub not_passing: Vec<(String, String)> }

/// ドリフト検知：報告を Schema で検証してから、テスト仕様と突き合わせる。
pub fn drift(decls: &[Decl], report: &Value, report_schema: &Value) -> Result<Drift, Vec<String>> {
    let errs = base::validate(report_schema, report);
    if !errs.is_empty() {
        return Err(errs);
    }
    let spec = spec_items(decls);
    let rows = base::get(report, "results[].[id, result]");
    let tests: BTreeMap<String, String> = rows.as_array().unwrap().iter()
        .map(|r| (r[0].as_str().unwrap().to_string(), r[1].as_str().unwrap().to_string())).collect();
    Ok(Drift {
        missing: spec.iter().filter(|s| !tests.contains_key(*s)).cloned().collect(),
        extra: tests.keys().filter(|t| !spec.contains(t)).cloned().collect(),
        not_passing: tests.iter().filter(|(t, r)| *r != "pass" && spec.contains(t)).map(|(t, r)| (t.clone(), r.clone())).collect(),
    })
}
