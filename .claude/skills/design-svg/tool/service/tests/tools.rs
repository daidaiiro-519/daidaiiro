// SPDX-License-Identifier: MIT
//! サービス層の道具 ── 宣言の JSON から図を組み、誤用を誤用として返すこと。

use ds_service::{tools, Given, Outcome};

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
        [
            "catalog", "figure", "chart", "canvas", "verify", "lint", "get", "validate", "view",
            "import"
        ]
    );
}

#[test]
fn the_declaration_carries_layout_and_direction() {
    // **宣言に書いたものを、道具が読む** ── 引数でしか渡せないと、宣言だけでは同じ図が組み直せない
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

#[test]
fn an_elbow_inside_a_group_is_applied() {
    // **囲みの内側の辺にも鍵線を適用できる** ── 往復する2本の辺を、別の角で分けるために使用する
    let p = file(
        "grid_group.json",
        r#"{"layout": "grid", "nodes": [{"id": "a", "label": "甲"}, {"id": "b", "label": "乙"}, {"id": "c", "label": "丙"}],
            "edges": [{"from": "a", "to": "b", "label": "行き"}, {"from": "b", "to": "a", "label": "帰り"}],
            "groups": [{"label": "囲み", "members": ["a", "b"]}],
            "grid": {"at": {"__g0": [0, 0], "c": [1, 0], "a": [0, 0], "b": [1, 1]},
                     "elbow": [["a", "b", "vertical"], ["b", "a", "vertical"]]}}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
}

#[test]
fn an_elbow_on_an_undeclared_edge_is_misuse() {
    // **宣言に無い辺の鍵線は、図全体で照合して断る** ── 段ごとの配置では検出できない
    let p = file(
        "grid_absent.json",
        r#"{"layout": "grid", "nodes": [{"id": "a"}, {"id": "b"}], "edges": [{"from": "a", "to": "b"}],
            "grid": {"at": {"a": [0, 0], "b": [1, 1]}, "elbow": [["a", "c", "vertical"]]}}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(!out.ok, "{:?}", out.findings);
    assert!(
        out.findings
            .iter()
            .any(|f| f.contains("a") && f.contains("c")),
        "{:?}",
        out.findings
    );
}

#[test]
fn two_self_loops_on_one_node_are_separated() {
    // **同じ節点の2本目の輪は、別の辺へ出す** ── 同じ辺に置くと、経路も注記も重なる
    let p = file(
        "self_two.json",
        r#"{"nodes": [{"id": "a", "label": "主アクター"}, {"id": "s", "label": "システム"}],
            "edges": [{"from": "a", "to": "s"}, {"from": "s", "to": "s", "label": "確認する"}, {"from": "s", "to": "s", "label": "変更する"}],
            "direction": "LR"}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
}

#[test]
fn a_self_loop_label_stays_off_its_node() {
    // **輪の注記は輪の外側に置く** ── 輪は節点に接しているので、経路上の点を中心に置くと節点にかかる
    let p = file(
        "self_label.json",
        r#"{"nodes": [{"id": "s", "label": "システム"}],
            "edges": [{"from": "s", "to": "s", "label": "要求とデータを確認する"}]}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
}

#[test]
fn a_long_label_wraps_inside_its_box() {
    // **長いラベルは箱の中で折り返す** ── 1行のまま箱を広げると、横に並んだ図が縮小されて読めなくなる
    let p = file(
        "long.json",
        r#"{"nodes": [{"id": "a", "label": "ドメイン・アプリケーション層が「何をすべきか」を決め、実行手段はポート経由でアダプターに委譲する"}, {"id": "b", "label": "短い"}],
            "edges": [{"from": "a", "to": "b"}]}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
    let svg = out.data["svg"].as_str().expect("そのまま返る");
    let lines = svg.matches("委譲する</text>").count() + svg.matches("ドメイン・").count();
    assert_eq!(lines, 2, "1行目と最終行が別の text に在る");
    let width: f64 = svg
        .split("viewBox=\"0 0 ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|s| s.parse().ok())
        .expect("幅が在る");
    assert!(width < 400.0, "箱が1行の幅まで広がっていない: {width}");
}

#[test]
fn no_line_starts_with_closing_punctuation() {
    // **行頭に閉じ括弧 ・ 句読点を置かない**（行頭の禁則）
    let p = file(
        "kinsoku.json",
        r#"{"nodes": [{"id": "a", "label": "あいうえおかきくけこさしすせそたちつてとなにぬねの、はひふへほまみむめもやゆよらりるれろわをん」。"}]}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    let svg = out.data["svg"].as_str().expect("そのまま返る");
    for close in ["、", "。", "」", "）"] {
        assert!(!svg.contains(&format!(">{close}")), "行頭に {close}");
    }
}

#[test]
fn a_latin_word_is_not_split_and_no_line_ends_with_an_opening_bracket() {
    // **英数字の語の途中で折り返さない。行末に開き括弧を置かない**（行末の禁則）
    let p = file(
        "latin.json",
        r#"{"nodes": [{"id": "a", "label": "原典を取得する（書籍はスキャン、公開の文書は URL ・ 取得した日 ・ sha256）と design-svg で図を組む（手元だけに置く）"}]}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
    let svg = out.data["svg"].as_str().expect("そのまま返る");
    assert!(svg.contains("sha256"), "sha256 が1行に在る");
    assert!(svg.contains("design-svg"), "design-svg が1行に在る");
    for open in ["（", "「"] {
        assert!(!svg.contains(&format!("{open}</text>")), "行末に {open}");
    }
}

#[test]
fn wrapped_lines_are_balanced() {
    // **最後の行に1 ・ 2文字だけを残さない** ── 行数を変えない範囲で幅を詰め、各行の長さをそろえる
    let p = file(
        "balance.json",
        r#"{"nodes": [{"id": "a", "label": "金額を扱う ・ 分析する ・ 監査の記録が要るかを判定する"}]}"#,
    );
    let out = call("figure", &[("declaration", &p)]);
    let svg = out.data["svg"].as_str().expect("そのまま返る");
    let lines: Vec<usize> = svg
        .split("</text>")
        .filter_map(|t| t.rsplit('>').next())
        .filter(|t| !t.is_empty())
        .map(|t| t.chars().count())
        .collect();
    assert!(lines.len() >= 2, "{lines:?}");
    assert!(lines.iter().all(|n| *n > 2), "短すぎる行がある: {lines:?}");
}

#[test]
fn the_references_pass_their_schemas() {
    // **references の JSON は、指しているスキーマに合う**（契約の版2）── theme.json も document.json も対象である
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = call("validate", &[("skill_root", &root.display().to_string())]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
    assert_eq!(out.data["kinds"], serde_json::json!(["document", "theme"]));
}
