// SPDX-License-Identifier: MIT
//! `review` の依頼文へ入れる手順を、document の1件から組む。**入出力を持たない** ── 読むのは
//! references の実装（`refs::get`）であり、ここは受け取った値を本文へ変換するだけである。
//!
//! 手順の正本は `references/document.json` の `review-instruction` である。依頼文はモデルが読むので、
//! 節の入れ子を見出しの記号へ、塊（段落 ・ 箇条書き ・ 表 ・ コード）をそれぞれの記法へ戻す。

use serde_json::Value;

/// document の1件を、見出しと本文の並びへ変換する。**先頭の見出しは題（`title`）である。**
#[must_use]
pub fn text(doc: &Value) -> String {
    let mut out = Vec::new();
    let sections = doc["sections"].as_array().cloned().unwrap_or_default();
    // import は、最上位の見出しを題として節の先頭に置く ── 題を二重に出さない
    match sections.as_slice() {
        [only] if only["title"] == doc["title"] => section(only, 1, &mut out),
        _ => {
            out.push(format!("# {}", doc["title"].as_str().unwrap_or_default()));
            out.push(String::new());
            blocks(&doc["blocks"], &mut out);
            for s in &sections {
                section(s, 2, &mut out);
            }
        }
    }
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

fn section(node: &Value, level: usize, out: &mut Vec<String>) {
    out.push(format!(
        "{} {}",
        "#".repeat(level.min(6)),
        node["title"].as_str().unwrap_or_default()
    ));
    out.push(String::new());
    blocks(&node["blocks"], out);
    for child in node["sections"].as_array().into_iter().flatten() {
        section(child, level + 1, out);
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|x| x.as_str().unwrap_or_default().to_owned())
        .collect()
}

fn blocks(list: &Value, out: &mut Vec<String>) {
    for b in list.as_array().into_iter().flatten() {
        match b["kind"].as_str().unwrap_or_default() {
            "para" => out.push(b["text"].as_str().unwrap_or_default().to_owned()),
            "code" => {
                out.push("```".to_owned());
                out.push(b["text"].as_str().unwrap_or_default().to_owned());
                out.push("```".to_owned());
            }
            "list" => {
                let ordered = b["ordered"].as_bool().unwrap_or(false);
                for (i, item) in strings(&b["items"]).iter().enumerate() {
                    if ordered {
                        out.push(format!("{}. {item}", i + 1));
                    } else {
                        out.push(format!("- {item}"));
                    }
                }
            }
            "table" => {
                let head = strings(&b["head"]);
                out.push(format!("| {} |", head.join(" | ")));
                out.push(format!("|{}", "---|".repeat(head.len())));
                for row in b["rows"].as_array().into_iter().flatten() {
                    out.push(format!("| {} |", strings(row).join(" | ")));
                }
            }
            _ => continue,
        }
        out.push(String::new());
    }
}
