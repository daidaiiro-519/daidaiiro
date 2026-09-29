// SPDX-License-Identifier: MIT
//! references の部品（refs）を事例で検証する。**この部品は雛形から各 Skill へ複製する**ので、
//! ここで固定した振る舞いが、すべての Skill の get ・ validate ・ view ・ import になる。
//!
//!     cargo test -p sc_parts --test refs

#![recursion_limit = "256"]

use std::path::{Path, PathBuf};

use sc_parts::refs;
use serde_json::{json, Value};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-refs-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

fn write(dir: &Path, name: &str, v: &Value) {
    std::fs::write(
        dir.join(name),
        serde_json::to_string_pretty(v).expect("組める"),
    )
    .expect("書ける");
}

/// 判断基準の種類を1つ置く。
fn criteria(dir: &Path) {
    write(
        dir,
        "criteria.schema.json",
        &json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "判断基準",
            "type": "object", "additionalProperties": false, "required": ["items"],
            "properties": {"items": {"type": "array", "items": {
                "type": "object", "additionalProperties": false, "required": ["id", "title", "rule"],
                "properties": {
                    "id": {"type": "string"},
                    "title": {"title": "名前", "x-view": "heading", "type": "string"},
                    "rule": {"title": "判断の基準", "description": "判定に使う記述", "type": "string"}
                }}}}
        }),
    );
    write(
        dir,
        "criteria.json",
        &json!({"$schema": "criteria.schema.json", "items": [
            {"id": "aggregate", "title": "集約", "rule": "集約は1つのトランザクションの単位である"},
            {"id": "context", "title": "区切られた文脈", "rule": "語の意味は文脈の内側で1つに決まる"}
        ]}),
    );
}

#[test]
fn a_kind_that_matches_its_schema_passes() {
    let dir = scratch("ok");
    criteria(&dir);
    let found = refs::validate(&dir).expect("読める");
    assert!(found.is_empty(), "{found:?}");
    let names: Vec<String> = refs::kinds(&dir)
        .expect("読める")
        .into_iter()
        .map(|k| k.name)
        .collect();
    assert_eq!(names, vec!["criteria"]);
}

#[test]
fn markdown_in_references_is_reported() {
    // **Markdown は SKILL.md だけ** ── references に置いた Markdown は検出する
    let dir = scratch("md");
    criteria(&dir);
    std::fs::write(dir.join("knowledge.md"), "# 旧い形\n").expect("書ける");
    let found = refs::validate(&dir).expect("読める");
    assert!(
        found
            .iter()
            .any(|x| x.contains("knowledge.md") && x.contains("Markdown")),
        "{found:?}"
    );
}

#[test]
fn json_without_its_schema_is_reported() {
    let dir = scratch("noschema");
    write(&dir, "orphan.json", &json!({"a": 1}));
    let found = refs::validate(&dir).expect("読める");
    assert!(
        found.iter().any(|x| x.contains("$schema が無い")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|x| x.contains("スキーマ orphan.schema.json が無い")),
        "{found:?}"
    );
}

#[test]
fn json_that_breaks_its_schema_is_reported() {
    let dir = scratch("broken");
    criteria(&dir);
    write(
        &dir,
        "criteria.json",
        &json!({"$schema": "criteria.schema.json", "items": [{"id": "x", "title": "名前だけ"}]}),
    );
    let found = refs::validate(&dir).expect("読める");
    assert!(found.iter().any(|x| x.contains("rule")), "{found:?}");
}

#[test]
fn one_item_is_taken_by_its_id() {
    // **モデルは全体を読まない** ── id で1件だけを受け取る
    let dir = scratch("get");
    criteria(&dir);
    let one = refs::get(&dir, "criteria", Some("context")).expect("在る");
    assert_eq!(one["title"], "区切られた文脈");
    assert!(refs::get(&dir, "criteria", Some("none")).is_err());
    let all = refs::get(&dir, "criteria", None).expect("在る");
    assert!(all.get("$schema").is_none(), "スキーマを指す印は返さない");
}

#[test]
fn the_view_follows_the_schema_titles() {
    // **描画は種類の中身を参照しない** ── 見出しはスキーマの title、説明は description
    let dir = scratch("view");
    criteria(&dir);
    let html = refs::view(&dir, "criteria", Some("aggregate"), None).expect("描ける");
    assert!(
        html.contains("<h1>集約</h1>"),
        "x-view が heading の欄が h1 になる"
    );
    assert!(html.contains("判断の基準") && html.contains("判定に使う記述"));
    assert!(html.contains("集約は1つのトランザクションの単位である"));
    assert!(html.contains("<meta charset=\"utf-8\">"));
}

#[test]
fn the_tokens_can_be_replaced() {
    // **見た目はトークンが持つ** ── Skill は値を差し替えられる
    let dir = scratch("tokens");
    criteria(&dir);
    write(&dir, "view.tokens.json", &json!({"accent": "#123456"}));
    let html = refs::view(&dir, "criteria", None, None).expect("描ける");
    assert!(html.contains("--accent:#123456;"), "差し替えた値が使われる");
    assert!(!html.contains("--accent:#0f6e5c;"), "既定の値は残らない");
}

#[test]
fn the_template_can_be_replaced() {
    let dir = scratch("template");
    criteria(&dir);
    std::fs::write(
        dir.join("view.template.html"),
        "<html><head><style>{{style}}</style></head><body class=\"mine\">{{body}}</body></html>",
    )
    .expect("書ける");
    let html = refs::view(&dir, "criteria", None, None).expect("描ける");
    assert!(html.contains("class=\"mine\""));
}

fn answer_schema(dir: &Path) {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/answer.schema.json");
    std::fs::copy(here, dir.join("answer.schema.json")).expect("写せる");
}

#[test]
fn an_answer_is_checked_against_the_answer_schema() {
    // **回答は references の外の出力である** ── 種類のスキーマで検査し、欄の欠けを検出する
    let dir = scratch("answer");
    answer_schema(&dir);
    let fx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/answer.example.json");
    let found = refs::validate_file(&dir, "answer", &fx).expect("読める");
    assert!(found.is_empty(), "{found:?}");
    let mut broken: Value =
        serde_json::from_str(&std::fs::read_to_string(&fx).expect("読める")).expect("JSON");
    broken.as_object_mut().expect("表").remove("next");
    let bad = dir.join("bad.json");
    write(&dir, "bad.json", &broken);
    let found = refs::validate_file(&dir, "answer", &bad).expect("読める");
    assert!(found.iter().any(|x| x.contains("next")), "{found:?}");
}

#[test]
fn an_answer_is_drawn_by_x_view() {
    let dir = scratch("answer-view");
    answer_schema(&dir);
    let fx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/answer.example.json");
    let html = refs::view(&dir, "answer", None, Some(&fx)).expect("描ける");
    assert!(html.contains("class=\"card\""), "結論はカード");
    assert!(html.contains("判断相談"), "選択肢は oneOf の title で出す");
    assert!(html.contains("ol class=\"steps\""), "道筋は番号付きの段");
    assert!(html.contains("<table>"), "比較と根拠は表");
    assert!(
        html.contains("<h1>受注の確定が"),
        "x-view が heading の欄が h1"
    );
    assert!(!html.contains("1文で言い切る"), "$comment は描画しない");
}

#[test]
fn markdown_is_imported_as_nested_sections() {
    let text = "前文\n\n# 題\n\n本文1\n\n## 節A\n\nA の本文\n\n### 節A1\n\nA1\n\n## 節B\n\n```\n# コードの中は見出しではない\n```\n";
    let doc = refs::import_markdown("sample", "https://example.com/a.md", "2026-09-29", text);
    assert_eq!(doc["title"], "題");
    assert_eq!(doc["blocks"][0]["text"], "前文");
    let top = &doc["sections"][0];
    assert_eq!(top["sections"][0]["title"], "節A");
    assert_eq!(top["sections"][0]["sections"][0]["title"], "節A1");
    assert_eq!(top["sections"][1]["title"], "節B");
    assert_eq!(
        top["sections"][1]["blocks"][0]["kind"], "code",
        "コードの中の # は見出しにしない"
    );
    assert_eq!(doc["source"]["sha256"].as_str().map(str::len), Some(64));
}

#[test]
fn an_imported_document_passes_the_document_schema() {
    let dir = scratch("import");
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../references/profiles/rust/document.schema.json.tmpl");
    std::fs::copy(here, dir.join("document.schema.json")).expect("写せる");
    let doc = refs::import_markdown("sample", "a.md", "2026-09-29", "# 題\n\n## 節\n\n本文\n");
    refs::put_document(&dir, doc.clone()).expect("置ける");
    refs::put_document(&dir, doc).expect("置き換えられる");
    let found = refs::validate(&dir).expect("読める");
    assert!(found.is_empty(), "{found:?}");
    let all = refs::get(&dir, "document", None).expect("在る");
    assert_eq!(
        all["items"].as_array().map(Vec::len),
        Some(1),
        "同じ id は置き換える"
    );
}

#[test]
fn sha256_matches_the_known_value() {
    assert_eq!(
        refs::sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        refs::sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn the_copy_in_this_skill_matches_the_template() {
    // **雛形と、この Skill の複製がずれない** ── ずれると、試験しているものと配るものが違う
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmpl = std::fs::read_to_string(here.join("../../references/profiles/rust/refs.rs.tmpl"))
        .expect("読める");
    let copy = std::fs::read_to_string(here.join("src/refs.rs")).expect("読める");
    assert_eq!(tmpl, copy);
}

#[test]
fn frontmatter_is_not_imported() {
    // **frontmatter は原典の管理の欄であって、本文ではない**
    let doc = refs::import_markdown(
        "x",
        "a.md",
        "2026-09-29",
        "---\nid: x\ntitle: y\n---\n\n# 題\n\n本文\n",
    );
    assert!(doc.get("blocks").is_none(), "{doc}");
    assert_eq!(doc["title"], "題");
}

#[test]
fn sections_are_drawn_as_an_outline() {
    // **節の入れ子は、見出しと本文の字下げで描く** ── 欄の名前を並べない
    let dir = scratch("outline");
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../references/profiles/rust/document.schema.json.tmpl");
    std::fs::copy(here, dir.join("document.schema.json")).expect("写せる");
    let doc = refs::import_markdown(
        "sample",
        "a.md",
        "2026-09-29",
        "# 題\n\n## 節A\n\nA の本文\n",
    );
    refs::put_document(&dir, doc).expect("置ける");
    let html = refs::view(&dir, "document", Some("sample"), None).expect("描ける");
    assert!(
        html.contains("<h4>節A</h4>") || html.contains("<h5>節A</h5>"),
        "{html}"
    );
    assert!(!html.contains(">見出し<"), "欄の名前を並べない");
    assert!(!html.contains(">識別子<"), "hidden の欄は描かない");
}

#[test]
fn the_body_is_split_into_blocks() {
    // **本文の Markdown を記号のまま残さない** ── 段落 ・ 箇条書き ・ 表 ・ コードに分ける
    let lines: Vec<String> = "一文目の\n続き。\n\n- 甲\n- 乙\n\n| 列A | 列B |\n|---|---|\n| **1** | 2 |\n\n---\n\n```\nfn x() {}\n```"
        .lines().map(str::to_owned).collect();
    let b = refs::blocks_of(&lines);
    let kinds: Vec<&str> = b.iter().filter_map(|x| x["kind"].as_str()).collect();
    assert_eq!(kinds, vec!["para", "list", "table", "code"]);
    assert_eq!(b[0]["text"], "一文目の続き。");
    assert_eq!(b[2]["rows"][0][0], "1", "強調の記号は外す");
}

#[test]
fn a_figure_block_embeds_its_svg() {
    // **図は design-svg が組んだ SVG を埋め込む** ── 描画は図の記法を解釈しない
    let dir = scratch("figure");
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../references/profiles/rust/document.schema.json.tmpl");
    std::fs::copy(here, dir.join("document.schema.json")).expect("写せる");
    std::fs::create_dir_all(dir.join("figures")).expect("作れる");
    std::fs::write(dir.join("figures/a.svg"), "<svg id=\"a\"></svg>").expect("書ける");
    write(
        &dir,
        "document.json",
        &json!({"$schema": "document.schema.json", "items": [{
            "id": "x", "title": "題", "source": {"location": "a", "sha256": "0".repeat(64), "fetched": "d"},
            "sections": [{"title": "節", "blocks": [{"kind": "figure", "svg": "figures/a.svg", "caption": "図の説明"}]}]
        }]}),
    );
    assert!(refs::validate(&dir).expect("読める").is_empty());
    let html = refs::view(&dir, "document", Some("x"), None).expect("描ける");
    assert!(
        html.contains("<svg id=\"a\"></svg>") && html.contains("図の説明"),
        "{html}"
    );
}

/// 論点と単位の種類を1つ置く。**単位は kind で形が決まる** ── 形ごとに欄と見せ方が違う。
fn topics(dir: &Path) {
    std::fs::create_dir_all(dir.join("figures")).expect("作れる");
    std::fs::write(dir.join("figures/t.svg"), "<svg id=\"t\"></svg>").expect("書ける");
    write(
        dir,
        "note.schema.json",
        &json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "覚え書き",
            "type": "object", "required": ["items"],
            "properties": {"items": {"type": "array", "items": {
                "type": "object", "required": ["id", "title", "topics"],
                "properties": {
                    "id": {"type": "string", "x-view": "hidden"},
                    "title": {"type": "string", "x-view": "heading"},
                    "topics": {"title": "本文", "x-view": "sections", "type": "array", "items": {
                        "type": "object", "required": ["title", "claim", "units"],
                        "properties": {
                            "title": {"title": "題", "type": "string", "x-view": "heading"},
                            "claim": {"title": "主張", "type": "string", "x-view": "lead"},
                            "units": {"title": "単位", "x-view": "units", "type": "array", "items": {"oneOf": [
                                {"title": "規則", "type": "object", "required": ["kind", "rules"],
                                 "properties": {"kind": {"const": "rules"}, "heading": {"type": "string", "x-view": "subhead"}, "rules": {"type": "array", "items": {
                                    "type": "object", "properties": {
                                        "when": {"title": "条件", "type": "string"},
                                        "then": {"title": "対応", "type": "string"},
                                        "why": {"title": "理由", "type": "string"}}}}}},
                                {"title": "図", "type": "object", "required": ["kind", "purpose", "svg"],
                                 "properties": {"kind": {"const": "figure"},
                                    "purpose": {"title": "目的", "type": "string", "x-view": "lead"},
                                    "explanation": {"type": "array", "x-view": "paras", "items": {"type": "string"}},
                                    "svg": {"type": "string", "x-view": "svg"},
                                    "declaration": {"type": "object", "x-view": "hidden"}}},
                                {"title": "対比", "type": "object", "required": ["kind", "table"],
                                 "properties": {"kind": {"const": "comparison"},
                                    "table": {"type": "object", "x-view": "table"}}},
                                {"title": "手順", "type": "object", "required": ["kind", "steps"],
                                 "properties": {"kind": {"const": "steps"},
                                    "steps": {"type": "array", "x-view": "steps", "items": {"type": "object",
                                        "properties": {"do": {"type": "string"}, "why": {"type": "string"}}}}}}
                            ]}}
                        }}},
                    "pitfalls": {"title": "よくある誤り", "x-view": "group", "type": "object", "properties": {
                        "intro": {"type": "string", "x-view": "lead"},
                        "items": {"type": "array", "items": {"type": "object", "properties": {
                            "what": {"title": "誤り", "type": "string"}}}}}}
                }}}}
        }),
    );
    write(
        dir,
        "note.json",
        &json!({"$schema": "note.schema.json", "items": [{"id": "n", "title": "設計の覚え書き", "topics": [
            {"title": "論点A", "claim": "Aの主張", "units": [
                {"kind": "rules", "heading": "最初は広く区切る", "rules": [{"when": "知識が少ない", "then": "広く区切る"}]},
                {"kind": "rules", "rules": [{"then": "境界を監視する"}, {"then": "集約を小さくする"}]},
                {"kind": "rules", "rules": [{"when": "知識が増えた", "then": "分ける", "why": "境界が見える"}]},
                {"kind": "figure", "purpose": "図の目的", "explanation": ["段落1。", "段落2。"], "svg": "figures/t.svg", "declaration": {"nodes": []}},
                {"kind": "comparison", "table": {"head": ["方法", "欠点"], "rows": [["近似", "戻らない"]]}},
                {"kind": "steps", "steps": [{"do": "閉じる", "why": "直す箇所が分かる"}]}
            ]}
        ], "pitfalls": {"intro": "よく起きる誤り。", "items": [{"what": "細かく切る"}]}}]}),
    );
}

#[test]
fn topics_are_drawn_with_their_title_and_claim() {
    // **論点は、題 ・ 主張 ・ 単位の順に1枚で描く** ── 欄の名前（題 ・ 主張）を並べない
    let dir = scratch("topics");
    topics(&dir);
    assert!(refs::validate(&dir).expect("読める").is_empty());
    let html = refs::view(&dir, "note", Some("n"), None).expect("描ける");
    assert!(html.contains("<h2>論点A</h2>"), "{html}");
    assert!(html.contains("<p class=\"claim\">Aの主張</p>"), "{html}");
    assert!(!html.contains(">題<") && !html.contains(">主張<"), "{html}");
}

#[test]
fn a_unit_is_drawn_by_the_form_its_kind_selects() {
    // **単位の形は kind で決まる** ── 規則は表、図は SVG。形の名前を札にする
    let dir = scratch("units");
    topics(&dir);
    let html = refs::view(&dir, "note", Some("n"), None).expect("描ける");
    assert!(
        html.contains("<th>条件</th>") && html.contains("<td>広く区切る</td>"),
        "{html}"
    );
    assert!(html.contains("<svg id=\"t\"></svg>"), "{html}");
    assert!(
        html.contains(">規則<") && html.contains(">図<"),
        "形の名前を札にする: {html}"
    );
    assert!(!html.contains("nodes"), "hidden の欄は描かない: {html}");
    assert!(!html.contains(">rules<"), "kind の値を描かない: {html}");
    assert!(
        html.contains("<th>欠点</th>") && html.contains("<td>戻らない</td>"),
        "対比は列を自由に持つ表: {html}"
    );
    assert!(
        html.contains("<ol class=\"steps\"><li><p class=\"lead\">閉じる</p>"),
        "手順は番号付き: {html}"
    );
}
