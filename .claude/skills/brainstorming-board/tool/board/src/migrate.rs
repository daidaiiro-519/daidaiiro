//! 古い形（論点の18の欄）の board.json を、新しい形（board.schema.json）へ写す。
//! 古いファイルは board.json.v1 として残す（拡張子を .json にしないのは、基盤の check がディレクトリの .json をすべてインスタンスとして読むため）。写さない欄（経過 ・ 分かったこと など）も、そこから読める。

use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

fn text<'value>(value: &'value Value, key: &str) -> &'value str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn list<'value>(value: &'value Value, key: &str) -> &'value [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// タグを除き、文字の参照（&quot; など）を戻す。
fn plain(html: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for character in html.chars() {
        match character {
            '<' => inside = true,
            '>' if inside => inside = false,
            _ if !inside => out.push(character),
            _ => {}
        }
    }
    out.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// 答えの下位の決定：decision の中の list の項目。
fn details(topic: &Value) -> Vec<Value> {
    let decision = topic.get("decision").cloned().unwrap_or(Value::Null);
    list(&decision, "text")
        .iter()
        .filter(|block| text(block, "kind") == "list")
        .flat_map(|block| {
            list(block, "items")
                .iter()
                .map(|item| Value::String(plain(text(item, "text"))))
        })
        .collect()
}

/// 段落 ・ 表 ・ 注記などのブロックを、HTML の断片にする（UI の完成イメージにまとめるため）。
fn block_html(block: &Value) -> String {
    let cells = |row: &Value| -> String {
        match row {
            Value::Array(items) => items
                .iter()
                .map(|item| match item {
                    Value::Array(inner) => inner
                        .iter()
                        .map(|cell| format!("<td>{}</td>", cell.as_str().unwrap_or_default()))
                        .collect::<String>(),
                    other => format!("<td>{}</td>", other.as_str().unwrap_or_default()),
                })
                .collect(),
            other => format!("<td>{}</td>", other.as_str().unwrap_or_default()),
        }
    };
    match text(block, "kind") {
        "para" => format!("<p>{}</p>", text(block, "text")),
        "note" => format!("<p class=\"note\">{}</p>", text(block, "text")),
        "heading" => format!("<h3>{}</h3>", text(block, "text")),
        "list" => format!(
            "<ul>{}</ul>",
            list(block, "items")
                .iter()
                .map(|item| format!("<li>{}</li>", text(item, "text")))
                .collect::<String>()
        ),
        "grid" | "table" => {
            let head: String = list(block, "cols")
                .iter()
                .map(|col| format!("<th>{}</th>", col.as_str().unwrap_or_default()))
                .collect();
            let head = if list(block, "cols").len()
                < list(block, "rows")
                    .first()
                    .map(|row| match row {
                        Value::Array(items) => items
                            .iter()
                            .map(|item| item.as_array().map_or(1, Vec::len))
                            .sum(),
                        _ => 1,
                    })
                    .unwrap_or(0)
            {
                format!("<th></th>{head}")
            } else {
                head
            };
            let rows: String = list(block, "rows")
                .iter()
                .map(|row| format!("<tr>{}</tr>", cells(row)))
                .collect();
            format!("<table><tr>{head}</tr>{rows}</table>")
        }
        "fold" => format!(
            "<details open><summary>{}</summary>{}</details>",
            text(block, "heading"),
            list(block, "body")
                .iter()
                .map(block_html)
                .collect::<String>()
        ),
        "html" => text(block, "text").to_owned(),
        _ => String::new(),
    }
}

/// UI の完成イメージ1枚。古い完成イメージの段落 ・ 表などを、読める見た目で包む。
fn ui_image(body: &str) -> Value {
    let page = format!(
        "<!-- ui --><!doctype html><meta charset=\"utf-8\"><style>body{{font-family:system-ui,sans-serif;font-size:14px;line-height:1.8;margin:12px;color:#111d1a;background:#f8faf9}}table{{border-collapse:collapse;margin:8px 0}}th,td{{border:1px solid #ccd8d4;padding:4px 8px;text-align:left;vertical-align:top}}th{{background:#eef1ef}}.note{{color:#5b6b66}}pre{{background:#eef1ef;padding:8px;overflow-x:auto}}</style>{body}"
    );
    json!({"kind": "ui", "html": page})
}

/// 完成イメージ：example と figures のブロックから写す。図は figures/ の SVG を中へ入れ、コードはコードにする。
/// それ以外（段落 ・ 表 ・ 注記 など）は、続いた分をまとめて1枚の UI にして、中身を残す。
fn images(topic: &Value, dir: &Path) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    let mut rich = String::new();
    let flush = |rich: &mut String, out: &mut Vec<Value>| {
        if !rich.trim().is_empty() {
            out.push(ui_image(rich));
        }
        rich.clear();
    };
    for block in list(topic, "example").iter().chain(list(topic, "figures")) {
        match text(block, "kind") {
            "figure" => {
                flush(&mut rich, &mut out);
                let path = dir
                    .join("figures")
                    .join(format!("{}.svg", text(block, "name")));
                let svg = fs::read_to_string(&path)
                    .map_err(|error| format!("{}: 読めない ── {error}", path.display()))?;
                let mut image = json!({"kind": "figure", "svg": svg});
                if !text(block, "caption").is_empty() {
                    image["caption"] = json!(plain(text(block, "caption")));
                }
                out.push(image);
            }
            "html" => {
                let raw = text(block, "text");
                let body = plain(raw);
                if let Some(ui) = body.strip_prefix("<!-- ui -->") {
                    flush(&mut rich, &mut out);
                    out.push(json!({"kind": "ui", "html": format!("<!-- ui -->{ui}")}));
                } else if raw.trim_start().starts_with("<pre") || !raw.contains('<') {
                    flush(&mut rich, &mut out);
                    if !body.trim().is_empty() {
                        out.push(json!({"kind": "code", "text": body}));
                    }
                } else {
                    rich.push_str(raw);
                }
            }
            _ => rich.push_str(&block_html(block)),
        }
    }
    flush(&mut rich, &mut out);
    Ok(out)
}

fn topic(old: &Value, dir: &Path) -> Result<Value, String> {
    let id = format!("Q{}", old.get("no").and_then(Value::as_u64).unwrap_or(0));
    let mut answer = json!({"text": plain(text(old, "answer"))});
    let details = details(old);
    if !details.is_empty() {
        answer["details"] = Value::Array(details);
    }
    let mut new = json!({
        "id": id, "name": text(old, "name"), "status": text(old, "status"),
        "question": plain(text(old, "question")), "answer": answer
    });
    let images = images(old, dir)?;
    if !images.is_empty() {
        new["images"] = Value::Array(images);
    }
    let grounds: Vec<Value> = list(old, "grounds")
        .iter()
        .map(|ground| {
            json!({"supports": plain(text(ground, "supports")), "basis": plain(text(ground, "basis")),
                   "tag": text(ground, "tag"), "source": plain(text(ground, "source"))})
        })
        .collect();
    if !grounds.is_empty() {
        new["grounds"] = Value::Array(grounds);
    }
    let rejected: Vec<Value> = list(old, "dropped")
        .iter()
        .map(|option| json!({"option": plain(text(option, "body")), "reason": plain(text(option, "reason"))}))
        .collect();
    if !rejected.is_empty() {
        new["rejected"] = Value::Array(rejected);
    }
    let mut verification = Map::new();
    let passed: Vec<Value> = list(old, "passed")
        .iter()
        .map(|option| json!({"option": plain(text(option, "name")), "cost": plain(text(option, "cost"))}))
        .filter(|option| !text(option, "option").is_empty() && !text(option, "cost").is_empty())
        .collect();
    if !passed.is_empty() {
        verification.insert("passed".into(), Value::Array(passed));
    }
    let out_of_scope: Vec<Value> = list(old, "out_of_scope")
        .iter()
        .map(|item| {
            let mut entry = json!({"item": plain(text(item, "item")), "treatment": item.get("treatment").and_then(Value::as_str).unwrap_or("out")});
            if !text(item, "note").is_empty() {
                entry["note"] = json!(plain(text(item, "note")));
            }
            entry
        })
        .collect();
    if !out_of_scope.is_empty() {
        verification.insert("out_of_scope".into(), Value::Array(out_of_scope));
    }
    if !verification.is_empty() {
        new["verification"] = Value::Object(verification);
    }
    Ok(new)
}

/// 古い形の board.json を新しい形へ写す。schema は board.json から board.schema.json への相対パス。
pub fn migrate(path: &Path, schema: &str) -> Result<Value, String> {
    let dir = path.parent().ok_or("board.json の置き場所が無い")?;
    let text_of = fs::read_to_string(path)
        .map_err(|error| format!("{}: 読めない ── {error}", path.display()))?;
    let old: Value = serde_json::from_str(&text_of)
        .map_err(|error| format!("JSON として読めない ── {error}"))?;
    if old.get("kind").and_then(Value::as_str) == Some("board") {
        return Err("既に新しい形である".to_owned());
    }
    let topics: Vec<Value> = list(&old, "topics")
        .iter()
        .map(|item| topic(item, dir))
        .collect::<Result<_, _>>()?;
    let queue: Vec<Value> = list(&old, "queue")
        .iter()
        .map(|item| json!({"topic": format!("Q{}", item.get("no").and_then(Value::as_u64).unwrap_or(0)), "why": plain(text(item, "why"))}))
        .collect();
    let intro: String = list(&old, "intro")
        .iter()
        .map(|block| plain(text(block, "text")))
        .collect::<Vec<_>>()
        .join("");
    let id = if text(&old, "board").is_empty() {
        dir.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        text(&old, "board").to_owned()
    };
    let mut new = json!({
        "$schema": schema, "kind": "board", "id": id, "title": plain(text(&old, "title")),
        "round": old.get("round").and_then(Value::as_u64).unwrap_or(1),
        "queue": queue, "topics": topics
    });
    if !intro.is_empty() {
        new["intro"] = json!(intro);
    }
    fs::write(dir.join("board.json.v1"), &text_of)
        .map_err(|error| format!("board.json.v1 を書けない ── {error}"))?;
    fs::write(
        path,
        serde_json::to_string_pretty(&new).unwrap_or_default() + "\n",
    )
    .map_err(|error| format!("board.json を書けない ── {error}"))?;
    Ok(
        json!({"ok": true, "path": path.display().to_string(), "kept": dir.join("board.json.v1").display().to_string(),
              "topics": list(&new, "topics").len()}),
    )
}
