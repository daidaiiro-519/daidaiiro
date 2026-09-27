// SPDX-License-Identifier: MIT
//! 道具の宣言 ── 宣言の JSON から図を組み、誤用を誤用として返すこと。

use ds_declare::{tools, Given, Outcome};

fn call(name: &str, args: &[(&str, &str)]) -> Outcome {
    let all = tools();
    let tool = all.iter().find(|t| t.name == name).expect("道具が在る");
    let mut given = Given::default();
    for (k, v) in args {
        given.push(k, (*v).to_owned());
    }
    for a in &tool.args {
        if !given.has(a.name) {
            if let Some(d) = a.default {
                given.push(a.name, d.to_owned());
            }
        }
    }
    (tool.run)(&given)
}

fn file(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join(format!("ds_declare_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("作れる");
    let p = dir.join(name);
    std::fs::write(&p, body).expect("書ける");
    p.display().to_string()
}

#[test]
fn every_tool_is_declared_once() {
    let names: Vec<&str> = tools().iter().map(|t| t.name).collect();
    assert_eq!(
        names,
        ["catalog", "figure", "chart", "canvas", "verify", "lint"]
    );
}

#[test]
fn the_declaration_carries_layout_and_direction() {
    // **宣言に書いたものを、入口が読む** ── 引数でしか渡せないと、宣言だけでは同じ図が組み直せない
    let p = file(
        "lr.json",
        r#"{"direction": "LR", "nodes": [{"id": "a", "label": "甲"}, {"id": "b", "label": "乙"}], "edges": [{"from": "a", "to": "b"}]}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
    let svg = out.data["svg"].as_str().expect("そのまま返る");
    let tb = call(
        "figure",
        &[(
            "declaration",
            &file(
                "tb.json",
                r#"{"nodes": [{"id": "a", "label": "甲"}, {"id": "b", "label": "乙"}], "edges": [{"from": "a", "to": "b"}]}"#,
            ),
        )],
    );
    assert_ne!(
        svg,
        tb.data["svg"].as_str().expect("そのまま返る"),
        "向きが効いている"
    );
}

#[test]
fn the_grid_is_reachable_from_the_declaration() {
    let p = file(
        "grid.json",
        r#"{"layout": "grid", "nodes": [{"id": "a", "label": "甲"}, {"id": "b", "label": "乙"}], "edges": [{"from": "a", "to": "b"}], "grid": {"at": {"a": [0, 0], "b": [1, 1]}, "elbow": [["a", "b", "vertical"]]}}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
    // 座標を持たない格子は誤用である
    let bad = call(
        "figure",
        &[(
            "declaration",
            &file(
                "grid_bad.json",
                r#"{"layout": "grid", "nodes": [{"id": "a"}]}"#,
            ),
        )],
    );
    assert!(!bad.ok);
    // 前の図の格子を読まない
    let after = call(
        "figure",
        &[(
            "declaration",
            &file("plain.json", r#"{"nodes": [{"id": "z", "label": "丙"}]}"#),
        )],
    );
    assert!(after.ok);
}

#[test]
fn an_unknown_token_is_a_misuse() {
    let out = call(
        "figure",
        &[(
            "declaration",
            &file(
                "theme.json",
                r##"{"nodes": [{"id": "a"}], "theme": {"color.nope": "#000"}}"##,
            ),
        )],
    );
    assert!(!out.ok);
    assert!(out.findings[0].contains("color.nope"), "{:?}", out.findings);
}

#[test]
fn a_new_role_can_be_added_from_the_declaration() {
    // **新しい役割はテーマへ行を足すだけで増える** ── 宣言の検査がそれを断ってはならない
    let ok = call(
        "figure",
        &[(
            "declaration",
            &file(
                "role.json",
                r#"{"nodes": [{"id": "a", "label": "停止", "role": "危険"}], "theme": {"role.危険.color.box-stroke": "color.warn", "role.危険.color.text": "color.warn"}}"#,
            ),
        )],
    );
    assert!(ok.ok, "{:?}", ok.findings);
    // 役割の行でも、綴りの誤りは断る
    let bad = call(
        "figure",
        &[(
            "declaration",
            &file(
                "role_bad.json",
                r##"{"nodes": [{"id": "a"}], "theme": {"role.危険.color.nope": "#000"}}"##,
            ),
        )],
    );
    assert!(!bad.ok);
}

#[test]
fn an_unknown_layout_is_a_misuse() {
    let out = call(
        "figure",
        &[(
            "declaration",
            &file(
                "spiral.json",
                r#"{"nodes": [{"id": "a"}], "layout": "spiral"}"#,
            ),
        )],
    );
    assert!(!out.ok);
}

#[test]
fn a_canvas_needs_its_size() {
    let p = file(
        "layers.json",
        r#"[{"kind": "box", "x": 0, "y": 0, "props": {"label": "甲"}}]"#,
    );
    assert!(!call("canvas", &[("layers", &p)]).ok);
    let out = call(
        "canvas",
        &[("layers", &p), ("width", "200"), ("height", "120")],
    );
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
}

#[test]
fn a_chart_is_checked_after_drawing() {
    let p = file(
        "bars.json",
        r#"{"bars": [{"name": "文書", "value": 13}, {"name": "図", "value": 5}]}"#,
    );
    let out = call("chart", &[("kind", "bars"), ("data", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
    assert_eq!(out.data["kind"], "bars");
}

#[test]
fn verify_reports_a_broken_drawing() {
    let p = file(
        "broken.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
    );
    let out = call("verify", &[("svg", &p)]);
    assert!(out.ok);
    assert_eq!(out.findings, vec!["viewBoxが無い".to_owned()]);
}

#[test]
fn lint_refuses_a_missing_place() {
    // **無い場所を検査して「0 箇所」と返さない**
    assert!(!call("lint", &[("path", "/nonexistent/place")]).ok);
    let here = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let out = call("lint", &[("skill_root", &format!("{here}/.."))]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
}

#[test]
fn the_catalog_lists_every_part() {
    let out = call("catalog", &[]);
    assert!(out.ok);
    assert!(out.data["parts"].as_array().expect("並び").len() >= 25);
    assert!(out.data["body"].as_str().expect("本文").starts_with('{'));
}
