// SPDX-License-Identifier: MIT
//! **照合の出力は、1つの入力から組む** ── 比較ページ ・ 確認記録 ・ 台本 ・ 観点ごとの結果。
//!
//!     cargo test -p sd_parts --test review

use std::path::{Path, PathBuf};

use sd_parts::review::{self, Line};
use serde_json::json;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

/// 試験ごとに別の作業場所を作る。**同時に走る試験どうしで、書き出し先を共有しない。**
fn workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sd-review-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("img")).expect("作れる");
    dir
}

fn write_input(dir: &Path, log: serde_json::Value) -> PathBuf {
    for (name, bytes) in [("a.png", b"A".as_slice()), ("b.png", b"B"), ("c.png", b"C")] {
        std::fs::write(dir.join("img").join(name), bytes).expect("書ける");
    }
    let input = json!({
        "title": "ためしの照合",
        "summary": ["1枚を直し、1枚を足した"],
        "before": {"rev": "abc", "sections": [{"key": "00", "name": "前置き", "slides": [
            {"id": "S1", "heading": "前の見出し", "lede": "前の導入文", "narration": "残る文です。消す文です。", "image": "img/a.png"},
            {"id": "S2", "heading": "変えない枚", "narration": "同じです。", "image": "img/b.png"}
        ]}]},
        "after": {"sections": [{"key": "00", "name": "前置き", "slides": [
            {"id": "S1", "heading": "後の見出し", "lede": "後の導入文", "narration": "残る文です。足す文です。", "image": "img/c.png"},
            {"id": "S2", "heading": "変えない枚", "narration": "同じです。", "image": "img/b.png"},
            {"id": "S3", "heading": "足した枚", "narration": "新しい文です。", "image": "img/a.png"}
        ]}]},
        "log": log
    });
    let path = dir.join("review.json");
    std::fs::write(&path, input.to_string()).expect("書ける");
    path
}

#[test]
fn a_narration_is_split_after_each_full_stop() {
    let s = review::sentences("一つめです。二つめですか？三つめ");
    assert_eq!(s, vec!["一つめです。", "二つめですか？", "三つめ"]);
}

#[test]
fn a_diff_keeps_common_sentences_and_marks_the_rest() {
    let before: Vec<String> = ["A。", "B。", "C。"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let after: Vec<String> = ["A。", "X。", "C。"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let lines = review::diff(&before, &after);
    assert!(lines.contains(&Line::Same("A。".to_owned())));
    assert!(lines.contains(&Line::Del("B。".to_owned())));
    assert!(lines.contains(&Line::Ins("X。".to_owned())));
    assert!(lines.contains(&Line::Same("C。".to_owned())));
}

#[test]
fn a_compare_page_shows_changed_added_and_unchanged_slides() {
    let dir = workdir("compare");
    let input = write_input(&dir, json!([]));
    let out = dir.join("out/compare.html");
    let n = review::build_compare(&references(), &input, &out).expect("組める");
    assert_eq!(n, 3);
    let html = std::fs::read_to_string(&out).expect("読める");
    assert!(html.contains("消す文です。"), "変更前にだけ在る文が出る");
    assert!(html.contains("class=\"del\""), "消した文に印が付く");
    assert!(html.contains("class=\"ins\""), "足した文に印が付く");
    assert!(html.contains("導入文　前の導入文"), "導入文の変更が出る");
    assert!(html.contains("追加"), "足した枚に印が付く");
    assert!(
        html.contains("変更なし ── 開いて見る"),
        "変えない枚は折りたたむ"
    );
    assert!(
        dir.join("out/img/before/00-S1.png").exists(),
        "画像を出力の隣へ写す"
    );
    assert!(!html.contains("{{"), "差し込む場所が残らない");
}

#[test]
fn the_same_input_gives_the_same_page() {
    let dir = workdir("same");
    let input = write_input(&dir, json!([]));
    let (a, b) = (dir.join("a/p.html"), dir.join("b/p.html"));
    review::build_compare(&references(), &input, &a).expect("組める");
    review::build_compare(&references(), &input, &b).expect("組める");
    assert_eq!(
        std::fs::read_to_string(a).expect("読める"),
        std::fs::read_to_string(b).expect("読める")
    );
}

#[test]
fn exports_give_three_pages_from_one_input() {
    let dir = workdir("export");
    let log = json!([
        {"section": "00", "slide": "S1", "group": "figure", "status": "fixed", "found": "形が主張を示さない", "fixed": "入れ子にした"},
        {"section": "00", "slide": "S1", "group": "story", "status": "pending", "found": "未確認の語", "fixed": "次の枚で確認する"}
    ]);
    let input = write_input(&dir, log);
    let done = review::build_exports(&references(), &input, &dir.join("out")).expect("組める");
    let names: Vec<String> = done
        .iter()
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, vec!["script.html", "record.html", "results.html"]);
    let results = std::fs::read_to_string(dir.join("out/results.html")).expect("読める");
    assert!(results.contains("入れ子にした"));
    assert!(results.contains("保留"));
    let record = std::fs::read_to_string(dir.join("out/record.html")).expect("読める");
    assert!(
        record.contains(" open"),
        "確認記録は、変えない枚も開いて出す"
    );
}

#[test]
fn an_input_that_breaks_the_contract_writes_nothing() {
    let dir = workdir("bad");
    let path = dir.join("review.json");
    std::fs::write(
        &path,
        json!({"title": "欠けた入力", "before": {"sections": []}}).to_string(),
    )
    .expect("書ける");
    let out = dir.join("out/compare.html");
    let err = review::build_compare(&references(), &path, &out).expect_err("組めない");
    assert!(err.contains("入力の検査が通っていない"));
    assert!(!out.exists(), "不合格なら、何も書き出さない");
}
