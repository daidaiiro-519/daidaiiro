// SPDX-License-Identifier: MIT
//! 受け入れの事例 ── **advisor の完成イメージのギャラリーの SVG 21個。**
//!
//! - `gallery/raw/` ── ギャラリーの frag-*.html から抽出した、手書きのままの21個
//! - `gallery/class/` ── 同じ21個を、class の記述へ変換したもの（座標は変えていない）
//! - `gallery/added/` ── 21個が使わない class（swatch ・ series）を使う、足した事例
//!
//! 変換した21個は検査1〜4が0件で、足した事例と合わせて class の一覧の全 class を使う。
//! 手書きのままの21個には検査1と2を適用し、ブレストボード design-svg-rework の論点1と3の計測と
//! 照合する ── 人が先に数えた結果があるので、検査の見落としと誤検出の両方を測れる。

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use ds_business_logic::checks::{check_classes, check_values};
use ds_business_logic::classes;
use ds_business_logic::publish::authored;
use regex::Regex;

fn dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/gallery")
        .join(name)
}

/// `(名前, 中身)` を名前の順に並べる。
fn svgs(name: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(dir(name))
        .expect("事例が在る")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "svg"))
        .map(|p| {
            (
                p.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                std::fs::read_to_string(&p).expect("読める"),
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn there_are_21_figures_in_both_forms() {
    let raw: Vec<String> = svgs("raw").into_iter().map(|(n, _)| n).collect();
    let class: Vec<String> = svgs("class").into_iter().map(|(n, _)| n).collect();
    assert_eq!(raw.len(), 21, "{raw:?}");
    assert_eq!(raw, class, "手書きと変換したものが同じ名前で対になる");
}

#[test]
fn the_converted_figures_pass_checks_1_to_4() {
    let mut bad = Vec::new();
    for (name, body) in svgs("class").into_iter().chain(svgs("added")) {
        let done = authored(&body, None, true).expect("解決できる");
        if !done.findings.is_empty() {
            bad.push(format!("{name}: {:?}", done.findings));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

fn used(list: &[(String, String)]) -> BTreeSet<String> {
    let re = Regex::new(r#"class="([^"]*)""#).expect("式");
    list.iter()
        .flat_map(|(_, b)| {
            re.captures_iter(b)
                .flat_map(|c| {
                    c[1].split_whitespace()
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .filter(|n| classes::known(n))
        .collect()
}

#[test]
fn every_class_is_used_by_some_figure() {
    let mut all = svgs("class");
    all.extend(svgs("added"));
    let seen = used(&all);
    let unused: Vec<&str> = classes::notation()
        .classes
        .iter()
        .map(|c| c.name.as_str())
        .filter(|n| !seen.contains(*n))
        .collect();
    assert!(unused.is_empty(), "どの事例も使わない class: {unused:?}");
}

#[test]
fn the_21_figures_alone_leave_only_content_colours_unused() {
    // 21個が使わないのは、コンテンツとしての色（swatch）と量のグラフの系列（series）だけである
    let seen = used(&svgs("class"));
    let unused: Vec<&str> = classes::notation()
        .classes
        .iter()
        .map(|c| c.name.as_str())
        .filter(|n| !seen.contains(*n))
        .collect();
    assert_eq!(unused, ["swatch", "series-1", "series-2", "series-3"]);
}

#[test]
fn the_converted_figures_keep_their_coordinates() {
    // **検査を通すために配置を変えていない** ── 座標 ・ 図形の数 ・ テキストは手書きと同じ。
    // 例外は、矢じりの marker を外したこと（design-svg が生成する）と、変換の一覧に書いた3つ
    // （meta-thinking-1 の両向きの軸 ・ ddd-8 の min-width ・ platform-5-1 の文字の大きさ）である
    let coords =
        Regex::new(r#" (x|y|x1|y1|x2|y2|cx|cy|r|width|height|d|points)="([^"]*)""#).expect("式");
    let texts = Regex::new(r">([^<]+)</text>").expect("式");
    let pick = |b: &str| -> (Vec<String>, Vec<String>) {
        let body = Regex::new(r"(?s)<defs>.*?</defs>")
            .expect("式")
            .replace_all(b, "");
        let open = Regex::new(r"<svg[^>]*>").expect("式").replace(&body, "");
        (
            coords
                .captures_iter(&open)
                .map(|c| format!("{}={}", &c[1], &c[2]))
                .collect(),
            texts
                .captures_iter(&open)
                .map(|c| c[1].to_owned())
                .collect(),
        )
    };
    let raw: BTreeMap<String, String> = svgs("raw").into_iter().collect();
    let mut bad = Vec::new();
    for (name, body) in svgs("class") {
        let (c1, t1) = pick(&raw[&name]);
        let (c2, t2) = pick(&body);
        if t1 != t2 {
            bad.push(format!("{name}: テキストが違う"));
        }
        if c1 != c2 && name != "meta-thinking-1" {
            bad.push(format!("{name}: 座標が違う"));
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

/// 手書きの21個に適用した検査2の検出から、属性ごとの値の集合を作る。
fn written_values() -> BTreeMap<String, BTreeSet<String>> {
    let pair = Regex::new(r"([\w-]+)='([^']*)'").expect("式");
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_, body) in svgs("raw") {
        for f in check_values(&body).expect("読める") {
            for c in pair.captures_iter(&f) {
                out.entry(c[1].to_owned())
                    .or_default()
                    .insert(c[2].to_owned());
            }
        }
    }
    out
}

#[test]
fn check1_on_the_raw_figures_matches_the_board() {
    // 論点3の計測：marker は 17個の defs に 31個。21個とも class を持たない
    let mut markers = 0;
    let mut without = 0;
    for (name, body) in svgs("raw") {
        let found = check_classes(&body).expect("読める");
        markers += found.iter().filter(|f| f.contains("marker")).count();
        if !found.iter().any(|f| f.contains("class が無い")) {
            without += 1;
            eprintln!("{name}: class が無いと検出されなかった");
        }
    }
    assert_eq!(markers, 31);
    assert_eq!(without, 0);
}

#[test]
fn check2_on_the_raw_figures_matches_the_board() {
    let v = written_values();
    let count = |k: &str| v.get(k).map_or(0, BTreeSet::len);
    // 論点1の計測：フォントサイズは4種（10 ・ 11 ・ 12 ・ 13）
    assert_eq!(
        v.get("font-size").cloned().unwrap_or_default(),
        ["10", "11", "12", "13"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    );
    // ストローク幅は9種（1〜2.5）・ 破線のパターンは6種
    assert_eq!(count("stroke-width"), 9);
    assert_eq!(count("stroke-dasharray"), 6);
    // 角丸の半径は、論点1の計測の11種から楕円の rx（110）を除いた10種 ── 楕円の rx は形そのもので、
    // 論点3の追加で検査2の対象から外した
    assert_eq!(count("rx"), 10);
}

#[test]
fn check2_finds_the_four_panels_that_left_the_specification() {
    // 論点1の計測：仕様書の10個の CSS 変数以外の値を使ったのは ddd-1 ・ meta-thinking-1 ・ 3 ・ ux-2
    let spec: BTreeSet<String> = [
        "accent",
        "accent-bg",
        "ink",
        "dim",
        "panel",
        "line",
        "warn",
        "warn-bg",
        "sel",
        "sel-bg",
    ]
    .iter()
    .map(|v| format!("var(--{v})"))
    .collect();
    let pair = Regex::new(r"(fill|stroke)='([^']*)'").expect("式");
    let mut panels = Vec::new();
    for (name, body) in svgs("raw") {
        let outside = check_values(&body)
            .expect("読める")
            .iter()
            .flat_map(|f| {
                pair.captures_iter(f)
                    .map(|c| c[2].to_owned())
                    .collect::<Vec<_>>()
            })
            .any(|v| !spec.contains(&v));
        if outside {
            panels.push(name);
        }
    }
    assert_eq!(
        panels,
        ["ddd-1", "meta-thinking-1", "meta-thinking-3", "ux-2"]
    );
}
