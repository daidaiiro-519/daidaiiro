// SPDX-License-Identifier: MIT
//! references の実装。**どの Skill も同じファイルを複製して使う** ── `contract.rs` と同じ扱いである。
//!
//! references は、Skill の目的を達成するために参照する情報である（ACDR 0043）。種類ごとに、
//! JSON Schema（`<種類>.schema.json`）と、それに従う JSON（`<種類>.json`）を置く。JSON は
//! `$schema` でスキーマを指す。回答の形のように、スキーマだけの種類も在る。
//!
//! 持つ能力は4つである ── 取り出す（`get`）・ 検査する（`validate`）・ 描画する（`view`）・
//! 既存の文書を取り込む（`import`）。**描画は回答の形も種類の中身も参照しない** ── スキーマの
//! `title` を見出しに、`description` を説明にし、見せ方は `x-view` が決める。

use std::path::{Path, PathBuf};

use crate::data_access;

use serde_json::{json, Map, Value};

/// スキーマのファイル名の末尾。
pub const SCHEMA_TAIL: &str = ".schema.json";

/// 見た目の正本の複製（ボード view-design-tokens）。**描画は、Skill の references/ に置いた複製だけを読む**
/// ── コードに見た目の値を保持しない。正本は Skill を生んだ道具が持ち、Skill を生むときに複製する。
const VIEW_TOKENS: &str = "view.tokens.json";
/// 描画の規則（`.rv` の囲みの中だけに適用される）。
const VIEW_CSS: &str = "view.css";
/// 頁の型。**差し込む場所は3つ** ── `{{title}}` ・ `{{style}}` ・ `{{body}}`。
const VIEW_TEMPLATE: &str = "view.template.html";
/// Skill 固有のトークン（必要とする Skill だけが置く）。
const SKILL_TOKENS: &str = "view.skill.tokens.json";
/// Skill 固有の規則（必要とする Skill だけが置く）。
const SKILL_CSS: &str = "view.skill.css";

/// 種類1つ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Kind {
    /// 種類の名前（ファイル名の頭）。
    pub name: String,
    /// スキーマの置き場所。
    pub schema: PathBuf,
    /// JSON の置き場所。**スキーマだけの種類では None。**
    pub data: Option<PathBuf>,
}

/// references の種類を並べる。**名前の順である** ── 実行ごとに順が変わると、差分が毎回出る。
///
/// # Errors
///
/// references を読めないときに返す。
pub fn kinds(refs: &Path) -> Result<Vec<Kind>, String> {
    let mut out = Vec::new();
    let entries = data_access::files::list(refs)
        .map_err(|e| format!("{} を読めない ── {e}", refs.display()))?;
    let mut names: Vec<String> = file_names(entries)
        .into_iter()
        .filter(|n| n.ends_with(SCHEMA_TAIL))
        // **見た目の正本の複製（view.*）は種類ではない** ── 中身ではなく見た目である。形は validate が検査する
        .filter(|n| !n.starts_with("view."))
        .collect();
    names.sort();
    for file in names {
        let name = file.trim_end_matches(SCHEMA_TAIL).to_owned();
        let data = refs.join(format!("{name}.json"));
        out.push(Kind {
            schema: refs.join(&file),
            data: data_access::files::is_file(&data).then_some(data),
            name,
        });
    }
    Ok(out)
}

/// 描画した頁を書き出す。**サービス層は入出力を持たない**ので、ここを通す。
///
/// # Errors
///
/// 書けないときに返す。
pub fn save(out: &Path, html: &str) -> Result<(), String> {
    data_access::files::write(out, html).map_err(|e| format!("{} に書けない ── {e}", out.display()))
}

/// 取り込む文書を読む。**サービス層は入出力を持たない**ので、ここを通す。
///
/// # Errors
///
/// 読めないときに返す。
pub fn read_text(file: &Path) -> Result<String, String> {
    data_access::files::read_to_string(file)
        .map_err(|e| format!("{} を読めない ── {e}", file.display()))
}

/// 経路の並びから、ファイルの名前だけを取り出す。
fn file_names(paths: Vec<PathBuf>) -> Vec<String> {
    paths
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect()
}

fn read_json(path: &Path) -> Result<Value, String> {
    let body = data_access::files::read_to_string(path)
        .map_err(|e| format!("{} を読めない ── {e}", path.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("{} が JSON でない ── {e}", path.display()))
}

/// スキーマで検査する。**`$schema` の欄は、スキーマを指す印なので外してから当てる。**
fn against(schema: &Value, instance: &Value, head: &str) -> Vec<String> {
    let mut plain = instance.clone();
    if let Some(m) = plain.as_object_mut() {
        m.shift_remove("$schema");
    }
    let built = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(schema);
    let Ok(validator) = built else {
        return vec![format!("{head}: スキーマとして無効である")];
    };
    let mut found: Vec<String> = validator
        .iter_errors(&plain)
        .map(|e| {
            let at = e.instance_path.to_string();
            format!("{head}: {} ── {e}", at.trim_start_matches('/'))
        })
        .collect();
    found.sort();
    found
}

/// references を検査する。**スキーマに合わない JSON、スキーマを指さない JSON、Markdown を並べる。**
///
/// # Errors
///
/// references を読めないときに返す。
pub fn validate(refs: &Path) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    let mut files: Vec<String> = file_names(
        data_access::files::list(refs)
            .map_err(|e| format!("{} を読めない ── {e}", refs.display()))?,
    );
    files.sort();
    for file in &files {
        if file.ends_with(".md") {
            found.push(format!(
                "{file}: Markdown が在る ── references は JSON Schema と JSON で持つ（Markdown は SKILL.md だけ）"
            ));
        }
        if !file.ends_with(".json") || file.ends_with(SCHEMA_TAIL) {
            continue;
        }
        let path = refs.join(file);
        let data = match read_json(&path) {
            Ok(v) => v,
            Err(why) => {
                found.push(why);
                continue;
            }
        };
        // **$schema が指すスキーマで検査する** ── 雛形のように、種類の名前と違うスキーマを指してよい
        let own = format!("{}{SCHEMA_TAIL}", file.trim_end_matches(".json"));
        let want = match data.get("$schema").and_then(Value::as_str) {
            Some(s) => s.to_owned(),
            None => {
                found.push(format!("{file}: $schema が無い ── {own} を指す"));
                own
            }
        };
        match read_json(&refs.join(&want)) {
            Ok(schema) => found.extend(against(&schema, &data, file)),
            Err(_) => found.push(format!("{file}: スキーマ {want} が無い")),
        }
    }
    Ok(found)
}

/// 1つの JSON を、種類のスキーマで検査する。**回答のように、references の外の JSON に使う。**
///
/// # Errors
///
/// スキーマか JSON を読めないときに返す。
pub fn validate_file(refs: &Path, kind: &str, file: &Path) -> Result<Vec<String>, String> {
    let schema = read_json(&refs.join(format!("{kind}{SCHEMA_TAIL}")))?;
    let data = read_json(file)?;
    Ok(against(&schema, &data, &file.display().to_string()))
}

/// 種類の JSON を取り出す。**id を渡すと、その1件だけを返す** ── モデルは全体を読まずに済む。
///
/// # Errors
///
/// 種類が無いとき、その id の項目が無いときに返す。
pub fn get(refs: &Path, kind: &str, id: Option<&str>) -> Result<Value, String> {
    let path = refs.join(format!("{kind}.json"));
    let mut data = read_json(&path)?;
    if let Some(m) = data.as_object_mut() {
        m.shift_remove("$schema");
    }
    let Some(id) = id else { return Ok(data) };
    let items = data
        .get("items")
        .or(Some(&data))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{kind} は項目の並びではない ── id で取り出せない"))?;
    items
        .iter()
        .find(|x| x.get("id").and_then(Value::as_str) == Some(id))
        .cloned()
        .ok_or_else(|| format!("{kind} に id {id} の項目が無い"))
}

/// 本文を塊に分ける ── 段落 ・ 箇条書き ・ 表 ・ コード。**記号のまま残さない** ── 読み手に
/// Markdown の構造を解釈させない。区切りの線（---）は捨てる。
#[must_use]
pub fn blocks_of(lines: &[String]) -> Vec<Value> {
    fn cells(line: &str) -> Vec<String> {
        line.trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().replace("**", "").replace('`', ""))
            .collect()
    }
    let plain = |t: &str| t.replace("**", "").replace('`', "");
    let mut out: Vec<Value> = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let flush = |para: &mut Vec<String>, out: &mut Vec<Value>| {
        if !para.is_empty() {
            out.push(json!({"kind": "para", "text": para.join("")}));
            para.clear();
        }
    };
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_end();
        let t = line.trim_start();
        if t.starts_with("```") {
            flush(&mut para, &mut out);
            let mut code = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                code.push(lines[i].clone());
                i += 1;
            }
            out.push(json!({"kind": "code", "text": code.join("\n")}));
        } else if t.starts_with('|') {
            flush(&mut para, &mut out);
            let head = cells(t);
            let mut rows = Vec::new();
            i += 1;
            while i < lines.len() && lines[i].trim_start().starts_with('|') {
                let r = cells(&lines[i]);
                if !r
                    .iter()
                    .all(|c| c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
                {
                    rows.push(r);
                }
                i += 1;
            }
            out.push(json!({"kind": "table", "head": head, "rows": rows}));
            continue;
        } else if t.starts_with("- ")
            || t.starts_with("* ")
            || t.split_once(". ")
                .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        {
            flush(&mut para, &mut out);
            let ordered = !t.starts_with('-') && !t.starts_with('*');
            let mut items = Vec::new();
            while i < lines.len() {
                let l = lines[i].trim_start();
                let item = if let Some(x) = l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")) {
                    x
                } else if let Some((n, x)) = l
                    .split_once(". ")
                    .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                {
                    let _ = n;
                    x
                } else {
                    break;
                };
                items.push(plain(item));
                i += 1;
            }
            out.push(json!({"kind": "list", "ordered": ordered, "items": items}));
            continue;
        } else if t.is_empty() || t.chars().all(|c| c == '-') {
            flush(&mut para, &mut out);
        } else {
            para.push(plain(t.trim_start_matches('>').trim()));
        }
        i += 1;
    }
    flush(&mut para, &mut out);
    out
}

/// Markdown の文書を、document の1件へ取り込む。**見出しを節の入れ子に分ける。**
#[must_use]
pub fn import_markdown(id: &str, location: &str, fetched: &str, text: &str) -> Value {
    #[derive(Default)]
    struct Node {
        title: String,
        level: usize,
        body: Vec<String>,
        children: Vec<Node>,
    }
    fn to_json(n: Node) -> Value {
        let mut m = Map::new();
        m.insert("title".to_owned(), json!(n.title));
        let blocks = blocks_of(&n.body);
        if !blocks.is_empty() {
            m.insert("blocks".to_owned(), Value::Array(blocks));
        }
        if !n.children.is_empty() {
            m.insert(
                "sections".to_owned(),
                Value::Array(n.children.into_iter().map(to_json).collect()),
            );
        }
        Value::Object(m)
    }
    let mut stack: Vec<Node> = vec![Node::default()];
    let mut fence = false;
    // **frontmatter は取り込まない** ── 原典の管理のための欄であって、本文ではない
    let body = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n").map(|(_, after)| after))
        .unwrap_or(text);
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fence = !fence;
        }
        let hashes = line.chars().take_while(|c| *c == '#').count();
        let heading = !fence && (1..=6).contains(&hashes) && line[hashes..].starts_with(' ');
        if !heading {
            if let Some(top) = stack.last_mut() {
                top.body.push(line.to_owned());
            }
            continue;
        }
        while stack.len() > 1 && stack.last().is_some_and(|n| n.level >= hashes) {
            if let Some(done) = stack.pop() {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(done);
                }
            }
        }
        stack.push(Node {
            title: line[hashes..].trim().to_owned(),
            level: hashes,
            ..Node::default()
        });
    }
    while stack.len() > 1 {
        if let Some(done) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(done);
            }
        }
    }
    let root = stack.pop().unwrap_or_default();
    let title = root
        .children
        .first()
        .map_or_else(|| id.to_owned(), |n| n.title.clone());
    let sha = sha256_hex(text.as_bytes());
    let mut doc = json!({
        "id": id,
        "title": title,
        "source": {"location": location, "sha256": sha, "fetched": fetched},
    });
    let sections: Vec<Value> = root.children.into_iter().map(to_json).collect();
    doc["sections"] = Value::Array(sections);
    let lead = blocks_of(&root.body);
    if !lead.is_empty() {
        doc["blocks"] = Value::Array(lead);
    }
    doc
}

/// document の並びへ1件を置く。**同じ id が在れば置き換える。**
///
/// # Errors
///
/// document を読めない ・ 書けないときに返す。
pub fn put_document(refs: &Path, doc: Value) -> Result<PathBuf, String> {
    let path = refs.join("document.json");
    let mut all = if data_access::files::is_file(&path) {
        read_json(&path)?
    } else {
        json!({"$schema": "document.schema.json", "items": []})
    };
    let id = doc.get("id").cloned().unwrap_or(Value::Null);
    let items = all
        .get_mut("items")
        .and_then(Value::as_array_mut)
        .ok_or("document.json に items が無い")?;
    items.retain(|x| x.get("id") != Some(&id));
    items.push(doc);
    let text = serde_json::to_string_pretty(&all).map_err(|e| e.to_string())?;
    data_access::files::write(&path, text + "\n")
        .map_err(|e| format!("{} に書けない ── {e}", path.display()))?;
    Ok(path)
}

/// SHA-256 を16進で返す。**外の crate に頼らない** ── 業務ロジック層の依存を増やさない。
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (a, b) in h.iter_mut().zip(v) {
            *a = a.wrapping_add(b);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

// ── 描画 ────────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn str_of<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

/// `$ref`（`#/$defs/…`）を解く。**スキーマの中だけを指す** ── 外を指す参照は解かない。
fn resolve<'a>(schema: &'a Value, root: &'a Value) -> &'a Value {
    let Some(r) = str_of(schema, "$ref") else {
        return schema;
    };
    r.strip_prefix('#')
        .and_then(|p| root.pointer(p))
        .unwrap_or(schema)
}

/// $ref の隣の語（title ・ x-view ・ description など）を、参照先に重ねる。**隣の語が参照先より優先する**
/// （JSON Schema 2020-12 では隣の語も効く）。隣の語の無い $ref は、そのまま残す。**深さで止める** ──
/// 自分を参照する定義でも、展開が終わる。
fn merge_refs(v: &Value, root: &Value, depth: usize) -> Value {
    match v {
        Value::Object(m) => {
            if let (Some(r), true) = (m.get("$ref").and_then(Value::as_str), m.len() > 1) {
                if let Some(Value::Object(target)) =
                    r.strip_prefix('#').and_then(|p| root.pointer(p))
                {
                    if depth < 8 {
                        let mut merged =
                            match merge_refs(&Value::Object(target.clone()), root, depth + 1) {
                                Value::Object(t) => t,
                                _ => serde_json::Map::new(),
                            };
                        for (k, x) in m {
                            if k != "$ref" {
                                merged.insert(k.clone(), merge_refs(x, root, depth));
                            }
                        }
                        return Value::Object(merged);
                    }
                }
            }
            Value::Object(
                m.iter()
                    .map(|(k, x)| (k.clone(), merge_refs(x, root, depth)))
                    .collect(),
            )
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| merge_refs(x, root, depth)).collect()),
        other => other.clone(),
    }
}

/// 節点と辺だけの図を、つながりの並びにする。**SVG の無い図でも、読み手が構造を確認できる。**
fn graph_html(value: &Value) -> String {
    let nodes: Vec<&Value> = value
        .get("nodes")
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    let edges: Vec<&Value> = value
        .get("edges")
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    if nodes.is_empty() && edges.is_empty() {
        return String::new();
    }
    let label = |id: &str| -> String {
        nodes
            .iter()
            .find(|n| str_of(n, "id") == Some(id))
            .and_then(|n| str_of(n, "label"))
            .unwrap_or(id)
            .to_owned()
    };
    let mut lis = String::new();
    for e in &edges {
        let mut notes: Vec<String> = Vec::new();
        if let Some(l) = str_of(e, "label").filter(|l| !l.is_empty()) {
            notes.push(l.to_owned());
        }
        if e.get("dashed").and_then(Value::as_bool) == Some(true) {
            notes.push("破線".to_owned());
        }
        let note = if notes.is_empty() {
            String::new()
        } else {
            format!("（{}）", notes.join(" ・ "))
        };
        lis.push_str(&format!(
            "<li>{} → {}{}</li>",
            esc(&label(str_of(e, "from").unwrap_or(""))),
            esc(&label(str_of(e, "to").unwrap_or(""))),
            esc(&note)
        ));
    }
    for n in &nodes {
        let id = str_of(n, "id").unwrap_or("");
        let linked = edges
            .iter()
            .any(|e| str_of(e, "from") == Some(id) || str_of(e, "to") == Some(id));
        if !linked {
            lis.push_str(&format!(
                "<li>{}</li>",
                esc(str_of(n, "label").unwrap_or(id))
            ));
        }
    }
    format!("<ol class=\"graph\">{lis}</ol>")
}

fn label_of(schema: &Value, value: &Value) -> String {
    schema
        .get("oneOf")
        .and_then(Value::as_array)
        .and_then(|all| all.iter().find(|o| o.get("const") == Some(value)))
        .and_then(|o| str_of(o, "title"))
        .map_or_else(|| text_of(value), str::to_owned)
}

fn tag(schema: &Value, value: &Value) -> String {
    let neg = matches!(value, Value::Bool(false)) || value.as_str() == Some("reject");
    format!(
        "<span class=\"{}\">{}</span>",
        if neg { "tag neg" } else { "tag" },
        esc(&label_of(schema, value))
    )
}

fn is_scalar(schema: &Value) -> bool {
    // **画面に出さない欄は、表にするかの判定を妨げない**（ACDR 0073）── 出典のように各行が持つ欄で、
    // 表が見出しつきの積み重ねに崩れていた
    str_of(schema, "x-view") == Some("hidden")
        || schema.get("oneOf").is_some()
        || matches!(
            str_of(schema, "type"),
            Some("string" | "number" | "integer" | "boolean")
        )
}

fn head_of(key: &str, schema: &Value, level: usize) -> String {
    let h = level.clamp(2, 3);
    let title = str_of(schema, "title").unwrap_or(key);
    let desc = str_of(schema, "description")
        .map(|d| format!("<p class=\"desc\">{}</p>", esc(d)))
        .unwrap_or_default();
    format!("<h{h}>{}</h{h}>{desc}", esc(title))
}

struct Ctx<'a> {
    root: &'a Value,
    base: &'a Path,
}

fn view_of(schema: &Value) -> &str {
    str_of(schema, "x-view").unwrap_or("")
}

fn cell(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let schema = resolve(schema, ctx.root);
    if view_of(schema) == "tag" || schema.get("oneOf").is_some() {
        return tag(schema, value);
    }
    esc(&text_of(value))
}

fn inner(value: &Value, schema: &Value, level: usize, ctx: &Ctx) -> String {
    let schema = resolve(schema, ctx.root);
    match value {
        Value::Object(map) => {
            let props = schema.get("properties").and_then(Value::as_object);
            let mut out = String::new();
            let keys: Vec<&String> =
                props.map_or_else(|| map.keys().collect(), |p| p.keys().collect());
            for key in keys {
                let Some(v) = map.get(key) else { continue };
                let sub = props.and_then(|p| p.get(key)).unwrap_or(&Value::Null);
                if view_of(resolve(sub, ctx.root)) == "hidden" {
                    continue;
                }
                out.push_str(&format!(
                    "<div class=\"nest\">{}{}</div>",
                    head_of(key, resolve(sub, ctx.root), level),
                    inner(v, sub, level + 1, ctx)
                ));
            }
            out
        }
        Value::Array(items) => {
            let item = resolve(schema.get("items").unwrap_or(&Value::Null), ctx.root);
            let props = item.get("properties").and_then(Value::as_object);
            match props {
                Some(p) if p.values().all(|s| is_scalar(resolve(s, ctx.root))) => {
                    // **どの行も値を保持しない列は描かない** ── 空の列は、欠けがあるように読める
                    let p: Vec<(&String, &Value)> = p
                        .iter()
                        .filter(|(k, _)| items.iter().any(|v| v.get(k.as_str()).is_some()))
                        .filter(|(_, s)| str_of(resolve(s, ctx.root), "x-view") != Some("hidden"))
                        .collect();
                    // **列が1つなら箇条書きにする** ── 1列の表は、見出しの升が中身を説明しない
                    if p.len() == 1 {
                        let (k, s) = p[0];
                        let lis: String = items
                            .iter()
                            .filter_map(|v| v.get(k.as_str()))
                            .map(|v| format!("<li>{}</li>", cell(s, v, ctx)))
                            .collect();
                        return format!("<ul>{lis}</ul>");
                    }
                    let th: String = p
                        .iter()
                        .map(|(k, s)| format!("<th>{}</th>", esc(str_of(s, "title").unwrap_or(k))))
                        .collect();
                    let rows: String = items
                        .iter()
                        .map(|v| {
                            let tds: String = p
                                .iter()
                                .map(|(k, s)| {
                                    format!(
                                        "<td>{}</td>",
                                        cell(s, v.get(k).unwrap_or(&Value::Null), ctx)
                                    )
                                })
                                .collect();
                            format!("<tr>{tds}</tr>")
                        })
                        .collect();
                    format!("<div class=\"scroll\"><table><thead><tr>{th}</tr></thead><tbody>{rows}</tbody></table></div>")
                }
                Some(_) => items
                    .iter()
                    .map(|v| {
                        format!(
                            "<div class=\"item\">{}</div>",
                            inner(v, item, level + 1, ctx)
                        )
                    })
                    .collect(),
                None => {
                    let lis: String = items
                        .iter()
                        .map(|v| format!("<li>{}</li>", esc(&text_of(v))))
                        .collect();
                    format!("<ul>{lis}</ul>")
                }
            }
        }
        other => {
            if schema.get("oneOf").is_some() || view_of(schema) == "tag" {
                return tag(schema, other);
            }
            let t = text_of(other);
            if t.contains('\n') {
                format!("<pre>{}</pre>", esc(&t))
            } else {
                format!("<p>{}</p>", esc(&t))
            }
        }
    }
}

/// SVG を埋め込む。**`svg` はファイル名（references からの経路）か、SVG そのもの。**
fn svg_of(p: &str, base: &Path) -> String {
    if p.trim_start().starts_with("<svg") {
        p.to_owned()
    } else {
        data_access::files::read_to_string(base.join(p)).unwrap_or_default()
    }
}

/// 論点の並びを描く ── **1件を1枚にし、`heading` の欄を題、`lead` の欄を主張にする。**
/// 残りの欄は `field` が描く。欄の名前は並べない。
fn sections_html(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let item = resolve(schema.get("items").unwrap_or(&Value::Null), ctx.root);
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|v| {
            format!(
                "<section class=\"block topic\">{}</section>",
                fields(item, v, ctx)
            )
        })
        .collect()
}

/// 欄を順に描く。**`heading` は題、`lead` は主張、`hidden` は描かない。**
fn fields(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let Some(props) = schema.get("properties").and_then(Value::as_object) else {
        return inner(value, schema, 3, ctx);
    };
    props
        .iter()
        .filter(|(k, _)| k.as_str() != "kind")
        .filter_map(|(k, p)| value.get(k).map(|v| field(resolve(p, ctx.root), v, ctx)))
        .collect()
}

/// 欄1つを描く。見出しを付けない ── 論点と単位の中では、形そのものが役割を示す。
fn field(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    match view_of(schema) {
        "hidden" => String::new(),
        "heading" => format!("<h2>{}</h2>", esc(&text_of(value))),
        "lead" => format!("<p class=\"claim\">{}</p>", esc(&text_of(value))),
        "subhead" => format!("<h3>{}</h3>", esc(&text_of(value))),
        // **段落の並びは、段落ごとに描く** ── 1つの文字列へ連結すると、原文の段落の切れ目が消える
        "paras" => value
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| format!("<p>{}</p>", esc(&text_of(p))))
            .collect(),
        "svg" => format!(
            "<figure>{}</figure>",
            svg_of(value.as_str().unwrap_or(""), ctx.base)
        ),
        "code" => format!("<pre class=\"code\">{}</pre>", esc(&text_of(value))),
        "units" => units_html(schema, value, ctx),
        "blocks" => blocks_html(value, ctx.base),
        "steps" => steps_html(schema, value, ctx),
        "table" => {
            let mut t = value.clone();
            t["kind"] = json!("table");
            blocks_html(&json!([t]), ctx.base)
        }
        _ => inner(value, schema, 4, ctx),
    }
}

/// 単位の並びを描く。**形は `kind` の値で、`items.oneOf` から選ぶ** ── 形の `title` を札にし、
/// 欄はその形の見せ方で描く。合う形が無ければ、欄をそのまま描く。
fn units_html(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let forms: Vec<&Value> = schema
        .get("items")
        .and_then(|i| resolve(i, ctx.root).get("oneOf"))
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|f| resolve(f, ctx.root)).collect())
        .unwrap_or_default();
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|u| {
            let form = forms
                .iter()
                .find(|f| f.pointer("/properties/kind/const") == u.get("kind"))
                .copied()
                .unwrap_or(&Value::Null);
            let label = str_of(form, "title")
                .map(|t| format!("<span class=\"ulabel\">{}</span>", esc(t)))
                .unwrap_or_default();
            format!("<div class=\"unit\">{label}{}</div>", fields(form, u, ctx))
        })
        .collect()
}

/// 手順を番号付きで描く。**各段の1つ目の欄を太字にし、残りを添える。**
fn steps_html(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let item = resolve(schema.get("items").unwrap_or(&Value::Null), ctx.root);
    let keys: Vec<&String> = item
        .get("properties")
        .and_then(Value::as_object)
        .map(|p| p.keys().collect())
        .unwrap_or_default();
    let lis: String = value
        .as_array()
        .into_iter()
        .flatten()
        .map(|v| {
            let mut parts = keys.iter().filter_map(|k| v.get(k.as_str()));
            let lead = parts
                .next()
                .map(|x| format!("<p class=\"lead\">{}</p>", esc(&text_of(x))))
                .unwrap_or_default();
            let rest: String = parts
                .map(|x| format!("<p class=\"sub\">{}</p>", esc(&text_of(x))))
                .collect();
            format!("<li>{lead}{rest}</li>")
        })
        .collect();
    format!("<ol class=\"steps\">{lis}</ol>")
}

/// 本文の塊を描く ── 段落 ・ 一覧 ・ 表 ・ コード ・ 図。**図は SVG をそのまま埋め込む** ──
/// 描くのは呼ぶ側で、ここは描かない。`svg` はファイル名（references からの経路）か SVG そのもの。
fn blocks_html(blocks: &Value, base: &Path) -> String {
    blocks
        .as_array()
        .into_iter()
        .flatten()
        .map(|b| match str_of(b, "kind") {
            Some("list") => {
                let tag = if b.get("ordered").and_then(Value::as_bool) == Some(true) { "ol" } else { "ul" };
                let lis: String = b.get("items").and_then(Value::as_array).into_iter().flatten()
                    .map(|x| format!("<li>{}</li>", esc(&text_of(x)))).collect();
                format!("<{tag}>{lis}</{tag}>")
            }
            Some("table") => {
                let row = |r: &Value, t: &str| -> String {
                    r.as_array().into_iter().flatten().map(|c| format!("<{t}>{}</{t}>", esc(&text_of(c)))).collect()
                };
                let th = b.get("head").map(|h| row(h, "th")).unwrap_or_default();
                let trs: String = b.get("rows").and_then(Value::as_array).into_iter().flatten()
                    .map(|r| format!("<tr>{}</tr>", row(r, "td"))).collect();
                format!("<div class=\"scroll\"><table><thead><tr>{th}</tr></thead><tbody>{trs}</tbody></table></div>")
            }
            Some("figure") => {
                let svg = str_of(b, "svg")
                    .map(|p| {
                        if p.trim_start().starts_with("<svg") {
                            p.to_owned()
                        } else {
                            data_access::files::read_to_string(base.join(p)).unwrap_or_default()
                        }
                    })
                    .unwrap_or_default();
                let cap = str_of(b, "caption")
                    .map(|c| format!("<figcaption>{}</figcaption>", esc(c)))
                    .unwrap_or_default();
                format!("<figure>{svg}{cap}</figure>")
            }
            Some("code") => format!("<pre class=\"code\">{}</pre>", esc(str_of(b, "text").unwrap_or(""))),
            _ => format!("<p>{}</p>", esc(str_of(b, "text").unwrap_or(""))),
        })
        .collect()
}

/// 節の入れ子を、見出しと本文の字下げで組む。**欄の名前（見出し ・ 本文）は表示しない。**
fn outline(items: &Value, level: usize, base: &Path) -> String {
    let h = (level + 1).clamp(3, 6);
    items
        .as_array()
        .into_iter()
        .flatten()
        .map(|n| {
            let title = esc(str_of(n, "title").unwrap_or(""));
            let body = n
                .get("blocks")
                .map(|b| blocks_html(b, base))
                .unwrap_or_default();
            let kids = n
                .get("sections")
                .map(|k| outline(k, level + 1, base))
                .unwrap_or_default();
            format!("<div class=\"nest\"><h{h}>{title}</h{h}>{body}{kids}</div>")
        })
        .collect()
}

fn block(key: &str, schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let schema = resolve(schema, ctx.root);
    let head = head_of(key, schema, 2);
    match view_of(schema) {
        "hidden" => String::new(),
        "outline" => format!(
            "<section class=\"block\">{head}{}</section>",
            outline(value, 2, ctx.base)
        ),
        "blocks" => format!(
            "<section class=\"block\">{head}{}</section>",
            blocks_html(value, ctx.base)
        ),
        "sections" => sections_html(schema, value, ctx),
        // **導入の文と表を1枚にまとめる** ── 分けると、導入の文が別の論点に属すように読める
        "group" => format!(
            "<section class=\"block\">{head}{}</section>",
            fields(schema, value, ctx)
        ),
        "card" if value.is_string() => {
            let title = esc(str_of(schema, "title").unwrap_or(key));
            format!(
                "<section class=\"card\"><div class=\"cardhead\"><h2>{title}</h2></div><p class=\"lead\">{}</p></section>",
                esc(&text_of(value))
            )
        }
        "card" => {
            let props = schema.get("properties").and_then(Value::as_object);
            let mut tags = String::new();
            let mut lines = Vec::new();
            for (k, p) in props.into_iter().flatten() {
                let Some(v) = value.get(k) else { continue };
                if view_of(p) == "tag" {
                    tags.push_str(&tag(p, v));
                } else {
                    lines.push(esc(&text_of(v)));
                }
            }
            let body: String = lines
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    format!(
                        "<p class=\"{}\">{l}</p>",
                        if i == 0 { "lead" } else { "sub" }
                    )
                })
                .collect();
            let title = esc(str_of(schema, "title").unwrap_or(key));
            format!("<section class=\"card\"><div class=\"cardhead\"><h2>{title}</h2>{tags}</div>{body}</section>")
        }
        "figure" => {
            // **svg は、ファイル名か、SVG そのもの** ── 呼ぶ側が読み込み済みなら、そのまま埋め込む
            let svg = str_of(value, "svg")
                .map(|p| {
                    if p.trim_start().starts_with("<svg") {
                        p.to_owned()
                    } else {
                        data_access::files::read_to_string(ctx.base.join(p)).unwrap_or_default()
                    }
                })
                .unwrap_or_default();
            // **SVG が無ければ、節点と辺の宣言を描く**
            let svg = if svg.is_empty() {
                graph_html(value)
            } else {
                svg
            };
            let cap = esc(str_of(value, "caption").unwrap_or(""));
            // **説明を図の前に置く** ── 何の図かを知ってから、図を読む
            format!("<section class=\"block\">{head}<figure><figcaption>{cap}</figcaption>{svg}</figure></section>")
        }
        "steps" => format!(
            "<section class=\"block\">{head}{}</section>",
            steps_html(schema, value, ctx)
        ),
        _ => format!(
            "<section class=\"block\">{head}{}</section>",
            inner(value, schema, 3, ctx)
        ),
    }
}

/// 1件を描画する。**x-view が heading の欄を h1、トップの tag の欄を見出しの上の札にする。**
fn page_body(schema: &Value, value: &Value, ctx: &Ctx) -> String {
    let schema = resolve(schema, ctx.root);
    let Some(props) = schema.get("properties").and_then(Value::as_object) else {
        return inner(value, schema, 2, ctx);
    };
    let mut eyebrow = vec![esc(str_of(ctx.root, "title").unwrap_or(""))];
    let mut h1 = String::new();
    let mut body = String::new();
    for (k, p) in props {
        let p = resolve(p, ctx.root);
        let Some(v) = value.get(k) else { continue };
        match view_of(p) {
            "tag" => eyebrow.push(tag(p, v)),
            "meta" => {
                let t = text_of(v);
                if !t.is_empty() {
                    eyebrow.push(esc(&t));
                }
            }
            "heading" => h1 = esc(&text_of(v)),
            _ => body.push_str(&block(k, p, v, ctx)),
        }
    }
    let lead = if h1.is_empty() {
        str_of(value, "title").map(esc).unwrap_or_default()
    } else {
        h1
    };
    format!(
        "<header><p class=\"eyebrow\">{}</p><h1>{lead}</h1></header>{body}",
        eyebrow.join(" ・ ")
    )
}

/// 見た目の正本の複製が無いときの理由。
fn missing(name: &str) -> String {
    format!("references/{name} が無い ── 見た目の正本の複製である。Skill を生んだ道具が置く（その道具の check が正本との差を報告する）")
}

/// 明と暗の色の変数を組む。**共通のパレットの後ろに Skill 固有のトークンを足し、鍵の順に並べる** ──
/// 並びを固定しないと、言語の組ごとに違う文字列になる。
fn palette_vars(refs: &Path) -> Result<(String, String), String> {
    let tokens = read_json(&refs.join(VIEW_TOKENS)).map_err(|_| missing(VIEW_TOKENS))?;
    let name = tokens
        .get("palette")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{VIEW_TOKENS}: palette が無い"))?;
    let chosen = tokens
        .get("palettes")
        .and_then(|p| p.get(name))
        .ok_or_else(|| format!("{VIEW_TOKENS}: palettes に {name} が無い"))?;
    let skill = read_json(&refs.join(SKILL_TOKENS)).ok();
    let mut out = Vec::new();
    for scheme in ["light", "dark"] {
        let mut vars: Vec<(String, String)> = Vec::new();
        for source in [Some(chosen), skill.as_ref()].into_iter().flatten() {
            for (k, v) in source
                .get(scheme)
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                if let Some(v) = v.as_str() {
                    vars.retain(|(n, _)| n != k);
                    vars.push((k.clone(), v.to_owned()));
                }
            }
        }
        vars.sort();
        out.push(
            vars.iter()
                .map(|(k, v)| format!("--{k}:{v};"))
                .collect::<String>(),
        );
    }
    Ok((out[0].clone(), out[1].clone()))
}

/// 色の変数を `selector` に定め、共通と Skill 固有の規則を続ける。
fn style_for(refs: &Path, selector: &str) -> Result<String, String> {
    let (light, dark) = palette_vars(refs)?;
    let css =
        data_access::files::read_to_string(refs.join(VIEW_CSS)).map_err(|_| missing(VIEW_CSS))?;
    let own = data_access::files::read_to_string(refs.join(SKILL_CSS)).unwrap_or_default();
    Ok(format!(
        "{selector}{{{light}}}@media (prefers-color-scheme:dark){{{selector}{{{dark}}}}}{css}{own}"
    ))
}

/// `.rv` の囲みの中だけに適用される規則と色。**他の型の頁に埋め込むときに使う** ── 色の変数も `.rv` に
/// 定めるので、埋め込む側の `:root` の変数を上書きしない。
///
/// # Errors
///
/// 見た目の正本の複製が無いか、読めないときに返す。
pub fn scoped_style(refs: &Path) -> Result<String, String> {
    style_for(refs, ".rv")
}

/// 1件の本文を組む（`.rv` の囲みを含む）。**他の型の頁に埋め込むときに使う。**
#[must_use]
pub fn render_body(schema: &Value, value: &Value, base: &Path) -> String {
    let schema = &merge_refs(schema, schema, 0);
    let ctx = Ctx { root: schema, base };
    format!("<div class=\"rv\">{}</div>", page_body(schema, value, &ctx))
}

/// 種類の JSON（か、その id の1件か、渡された JSON）を HTML にする。
///
/// # Errors
///
/// スキーマか JSON を読めないとき、その id の項目が無いときに返す。
pub fn view(
    refs: &Path,
    kind: &str,
    id: Option<&str>,
    file: Option<&Path>,
) -> Result<String, String> {
    let schema_path = refs.join(format!("{kind}{SCHEMA_TAIL}"));
    let root = read_json(&schema_path)?;
    // **$ref の隣の語を、参照先に重ねてから描く**
    let root = merge_refs(&root, &root, 0);
    let (value, base) = match file {
        Some(f) => (
            read_json(f)?,
            f.parent()
                .map_or_else(|| refs.to_path_buf(), Path::to_path_buf),
        ),
        None => (get(refs, kind, None)?, refs.to_path_buf()),
    };
    let ctx = Ctx {
        root: &root,
        base: &base,
    };
    let items_schema = || {
        let top = resolve(&root, &root);
        top.pointer("/properties/items/items")
            .or_else(|| top.get("items"))
            .cloned()
            .unwrap_or(Value::Null)
    };
    let body = match id {
        Some(id) => {
            let one = if file.is_some() {
                value
                    .get("items")
                    .or(Some(&value))
                    .and_then(Value::as_array)
                    .and_then(|a| a.iter().find(|x| str_of(x, "id") == Some(id)))
                    .cloned()
                    .ok_or_else(|| format!("id {id} の項目が無い"))?
            } else {
                get(refs, kind, Some(id))?
            };
            page_body(&items_schema(), &one, &ctx)
        }
        None => {
            let mut plain = value.clone();
            if let Some(m) = plain.as_object_mut() {
                m.shift_remove("$schema");
            }
            // **項目の並びだけを持つ種類は、1件ずつの頁を並べる** ── 全体を1つの値として描くと、
            // 項目の中の欄（units など）が見せ方を失い、JSON の文字列のまま並ぶ
            let only_items = plain.as_object().is_some_and(|m| m.len() == 1);
            let item_schema = resolve(&root, &root).pointer("/properties/items/items");
            match (
                only_items,
                plain.get("items").and_then(Value::as_array),
                item_schema,
            ) {
                (true, Some(list), Some(s)) => list
                    .iter()
                    .map(|it| {
                        format!(
                            "<section class=\"entry\">{}</section>",
                            page_body(s, it, &ctx)
                        )
                    })
                    .collect(),
                _ => page_body(&root, &plain, &ctx),
            }
        }
    };
    let template = data_access::files::read_to_string(refs.join(VIEW_TEMPLATE))
        .map_err(|_| missing(VIEW_TEMPLATE))?;
    let title = esc(str_of(&root, "title").unwrap_or(kind));
    Ok(template
        .replace("{{title}}", &title)
        .replace("{{style}}", &style_for(refs, ":root")?)
        .replace("{{body}}", &body))
}
