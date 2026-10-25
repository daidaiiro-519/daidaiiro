//! 追跡の検査：test-suite/trace.json と untested.json と、コードと場面に書いた traces を突き合わせ、
//! CODING.md の grep で見る規則と、syn で見る欄の見せ方の規則を確かめる。

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// 見つけた問題を文で返す。空なら合格
pub fn check(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let trace = match read_json(&root.join("test-suite/trace.json")) {
        Ok(v) => v,
        Err(e) => return vec![e],
    };
    let untested = match read_json(&root.join("test-suite/untested.json")) {
        Ok(v) => v,
        Err(e) => return vec![e],
    };
    let rs = files(&root.join("crates"), "rs");

    // 1. 不変条件と使う場面：一覧の中の重なり、試験の traces と untested.json の、重なり ・ 漏れ ・ 知らない id
    for (name, list) in [
        ("trace.json の invariants", &trace["invariants"]),
        ("trace.json の use_cases", &trace["use_cases"]),
        ("untested.json", &untested),
    ] {
        let mut seen = BTreeSet::new();
        for id in list
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x["id"].as_str())
        {
            if !seen.insert(id) {
                out.push(format!("{name} に {id} が2回ある"));
            }
        }
    }
    let mut known = BTreeSet::new();
    for i in trace["invariants"].as_array().into_iter().flatten() {
        if let Some(id) = i["id"].as_str() {
            known.insert(id.to_string());
        }
    }
    for u in trace["use_cases"].as_array().into_iter().flatten() {
        if let Some(id) = u["id"].as_str() {
            known.insert(format!("{id}:事後"));
            known.insert(format!("{id}:失敗時"));
        }
    }
    let untested_ids: BTreeSet<String> = untested
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x["id"].as_str().map(str::to_string))
        .collect();
    let mut traced = BTreeSet::new();
    for f in &rs {
        let text = fs::read_to_string(f).unwrap_or_default();
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        for (n, line) in lines.iter().enumerate() {
            if *line == "#[test]" && !(n > 0 && lines[n - 1].starts_with("/// traces:")) {
                out.push(format!(
                    "{}:{} の試験に /// traces: の行が無い",
                    rel(root, f),
                    n + 1
                ));
            }
        }
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("/// traces:") {
                traced.extend(
                    rest.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty()),
                );
            }
        }
    }
    for f in files(&root.join("test-suite/scenarios"), "json") {
        if let Ok(v) = read_json(&f) {
            traced.extend(
                v["traces"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|x| x.as_str().map(str::to_string)),
            );
        }
    }
    for id in traced.intersection(&untested_ids) {
        out.push(format!(
            "{id} が試験の traces と untested.json の両方にある"
        ));
    }
    for id in &known {
        if !traced.contains(id) && !untested_ids.contains(id) {
            out.push(format!("{id} が試験の traces にも untested.json にも無い"));
        }
    }
    for id in traced.union(&untested_ids) {
        if !known.contains(id) {
            out.push(format!("{id} は trace.json に無い id"));
        }
    }

    // 2. crates/ に題材の語が無い（題材の名前は test-suite/fixtures/ のフォルダ名）
    for subject in dirs(&root.join("test-suite/fixtures")) {
        for f in &rs {
            if fs::read_to_string(f)
                .unwrap_or_default()
                .to_lowercase()
                .contains(&subject)
            {
                out.push(format!("{} に題材の語「{subject}」がある", rel(root, f)));
            }
        }
    }

    // 3. mod.rs を使わない ・ 共有の crate を作らない
    for f in &rs {
        if f.file_name().is_some_and(|n| n == "mod.rs") {
            out.push(format!(
                "{} は mod.rs（foo.rs の形にそろえる）",
                rel(root, f)
            ));
        }
    }
    for layer in ["domain", "outbound", "inbound"] {
        for d in dirs(&root.join("crates").join(layer)) {
            if ["common", "shared", "utils"].contains(&d.as_str()) {
                out.push(format!("crates/{layer}/{d} は共有の crate"));
            }
        }
    }

    // 4. use_cases/ と ports/ のファイルが trace.json の一覧と一致する
    for (dir, key) in [
        ("crates/application/src/use_cases", "command_groups"),
        ("crates/application/src/ports", "ports"),
    ] {
        let want: BTreeSet<String> = trace[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x["file"].as_str().map(str::to_string))
            .collect();
        let got: BTreeSet<String> = files(&root.join(dir), "rs")
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        if want != got {
            out.push(format!(
                "{dir} のファイルが trace.json の {key} と違う（無い {:?} ・ 余分 {:?}）",
                want.difference(&got).collect::<Vec<_>>(),
                got.difference(&want).collect::<Vec<_>>()
            ));
        }
    }

    // 5. cli の中で outbound の crate を使うのは compose.rs だけ
    for f in files(&root.join("crates/inbound/cli/src"), "rs") {
        if f.file_name().is_some_and(|n| n == "compose.rs") {
            continue;
        }
        let text = fs::read_to_string(&f).unwrap_or_default();
        for ad in [
            "agents_contract",
            "ai_cli",
            "semantic_agent_schema_git",
            "semantic_agent_schema_fs",
            "semantic_agent_schema_tools",
            "semantic_agent_schema_sqlite",
        ] {
            if text.contains(ad) {
                out.push(format!(
                    "{} が outbound の crate（{ad}）を使っている（compose.rs だけに置く）",
                    rel(root, &f)
                ));
            }
        }
    }

    // 6. domain の欄の見せ方：集約とエンティティは集約のフォルダの中だけ、値オブジェクトは private
    for f in &rs {
        let path = rel(root, f).replace('\\', "/");
        let Some(inner) = path.strip_prefix("crates/domain/") else {
            continue;
        };
        let Some((_, in_src)) = inner.split_once("/src/") else {
            continue;
        };
        let allowed: &[&str] = if in_src.starts_with("values/") {
            &[]
        } else if in_src.starts_with("aggregates/entities/") {
            &["crate::aggregates"]
        } else if let Some(rest) = in_src.strip_prefix("aggregates/") {
            let name = rest
                .trim_end_matches(".rs")
                .split('/')
                .next()
                .unwrap_or_default();
            if name == "entities" {
                continue;
            }
            &[]
        } else {
            continue;
        };
        let aggregate = in_src.strip_prefix("aggregates/").map(|r| {
            r.trim_end_matches(".rs")
                .split('/')
                .next()
                .unwrap_or_default()
                .to_string()
        });
        for (ty, field, vis) in field_visibilities(f) {
            let ok = match vis.as_deref() {
                None => true,
                Some(v) => {
                    allowed.contains(&v)
                        || aggregate.as_deref().is_some_and(|a| {
                            a != "entities" && v == format!("crate::aggregates::{a}")
                        })
                }
            };
            if !ok {
                out.push(format!(
                    "{path} の {ty}.{field} の見せ方が {}（CODING.md 3節）",
                    vis.unwrap_or_default()
                ));
            }
        }
    }
    out
}

/// struct の欄ごとの（型 ・ 欄 ・ 見せ方）。見せ方は private なら None、pub は "pub"、pub(crate) は "crate"、pub(in p) は "p"
fn field_visibilities(f: &Path) -> Vec<(String, String, Option<String>)> {
    let mut out = Vec::new();
    let Ok(text) = fs::read_to_string(f) else {
        return out;
    };
    let Ok(file) = syn::parse_file(&text) else {
        return out;
    };
    fn walk(items: &[syn::Item], out: &mut Vec<(String, String, Option<String>)>) {
        for item in items {
            match item {
                syn::Item::Struct(s) => {
                    for (n, field) in s.fields.iter().enumerate() {
                        let name = field
                            .ident
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| n.to_string());
                        let vis = match &field.vis {
                            syn::Visibility::Inherited => None,
                            syn::Visibility::Public(_) => Some("pub".to_string()),
                            syn::Visibility::Restricted(r) => Some(
                                r.path
                                    .segments
                                    .iter()
                                    .map(|s| s.ident.to_string())
                                    .collect::<Vec<_>>()
                                    .join("::"),
                            ),
                        };
                        out.push((s.ident.to_string(), name, vis));
                    }
                }
                syn::Item::Mod(m) => {
                    if let Some((_, inner)) = &m.content
                        && !m.attrs.iter().any(|a| a.path().is_ident("cfg"))
                    {
                        walk(inner, out);
                    }
                }
                _ => {}
            }
        }
    }
    walk(&file.items, &mut out);
    out
}

fn read_json(p: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(p).map_err(|e| format!("{}：{e}", p.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}：{e}", p.display()))
}

fn files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            out.extend(files(&p, ext));
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn dirs(dir: &Path) -> Vec<String> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<String> = rd
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_lowercase())
        .collect();
    out.sort();
    out
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .map(|x| x.display().to_string())
        .unwrap_or_else(|_| p.display().to_string())
}
