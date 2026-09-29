// SPDX-License-Identifier: MIT
//! 記録1本の組み立てを、事例で検証する。
//!
//! **入力が通っていなければ、HTML を1バイトも出さない** ── 不合格の判明が生成物のあとに
//! なると、それはゲートではなく注記である。
//!
//!     cargo test -p acd_parts

use std::path::{Path, PathBuf};

use acd_parts::panes::{self, Shop};
use acd_parts::record;
use acd_parts::style::Style;
use acd_parts::template::Parts;
use serde_json::{json, Value};

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

/// 場所を1つ作る。**事例ごとに分ける** ── 共有すると、順に依存する
fn place(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("acd-record").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

fn sound() -> Value {
    json!({
        "no": "ACDR 0001", "title": "題", "date": "2026-09-27", "status": "proposed",
        "decision": "こうする。", "why": "こういう理由である。",
        "applies_to": "ここへ適用する。"
    })
}

fn made(spec: &Value, figure: &str) -> panes::Made {
    let refs = references();
    let parts = Parts::load(&refs.join("acdr.template.html")).expect("読める");
    let style = Style::load(&refs).expect("読める");
    let shop = Shop {
        parts: &parts,
        style: &style,
    };
    record::build(&shop, spec, figure).expect("組める")
}

fn write(folder: &Path, spec: &Value) {
    std::fs::write(
        folder.join("acdr.json"),
        serde_json::to_string(spec).expect("組める"),
    )
    .expect("書ける");
}

#[test]
fn a_decision_without_targets_is_one_page_of_sections() {
    // **対象が無い決定（新規）は、例外にせず、同じ器で空にする**
    let got = made(&sound(), "");
    assert!(got.page.contains("<div class=\"acdr\">"), "節が無い");
    assert!(got.page.contains("class=\"st "), "状態が付いていない");
    assert_eq!(got.total, 0);
    assert!(
        !got.page.contains("<button class=\"tab\""),
        "面が無いのにタブが在る"
    );
}

#[test]
fn the_sections_the_record_holds_are_in_the_page() {
    let mut spec = sound();
    spec["how"] = json!("こう実現する。");
    spec["shift"] = json!([{"what": "置き場所", "from": "旧", "to": "新"}]);
    spec["alternatives"] = json!([{"option": "案甲", "why_not": "これが壊れる。"}]);
    spec["after_approval"] = json!(["正本へ当てる"]);
    spec["supersedes"] = json!("ACDR 0000 を置き換える。");
    let got = made(&spec, "");
    for heading in [
        "なぜ、いま決めるのか",
        "形の変化",
        "実現の形",
        "適用先",
        "比較した案",
        "承認後に実施すること",
        "supersedes",
    ] {
        assert!(got.page.contains(heading), "{heading} が出ていない");
    }
}

#[test]
fn the_section_tables_scroll_inside_their_own_frame() {
    // **節の表も、横に溢れたら表の枠の中で横に送る** ── 枠が無いと、長い語を含む表が
    // 狭い画面でページごと横にはみ出す（実測 ── 390ピクセルの幅で、ページが422ピクセルになった）
    let mut spec = sound();
    spec["shift"] = json!([{"what": "置き場所", "from": "旧", "to": "新"}]);
    spec["alternatives"] = json!([{"option": "案甲", "why_not": "これが壊れる。"}]);
    let got = made(&spec, "");
    assert!(
        got.page
            .contains("<div class=\"scroll\"><table class=\"shift\">"),
        "形の変化の表に枠が無い"
    );
    assert!(
        got.page
            .contains("<div class=\"scroll\"><table><thead><tr><th>案</th>"),
        "比較した案の表に枠が無い"
    );
}

#[test]
fn an_absent_section_is_named_not_omitted() {
    // **比較した案が無いことと、書き忘れは、読み手には同じに見える**
    let got = made(&sound(), "");
    assert!(got.page.contains("比較した案は無い"), "{}", got.page.len());
    assert!(got.page.contains("無し"), "承認後に実施することが空である");
}

#[test]
fn a_figure_is_placed_as_it_came() {
    // **この Skill は図を描かない**
    let svg = "<svg><circle r=\"1\"/></svg>";
    let got = made(&sound(), svg);
    assert!(got.page.contains(svg), "SVG が書き換わっている");
    assert!(got.page.contains("図で確認する"), "節が出ていない");
}

#[test]
fn a_missing_section_refuses_the_build() {
    let refs = references();
    let parts = Parts::load(&refs.join("acdr.template.html")).expect("読める");
    let style = Style::load(&refs).expect("読める");
    let shop = Shop {
        parts: &parts,
        style: &style,
    };
    let mut spec = sound();
    spec.as_object_mut().expect("表である").remove("why");
    let why = record::build(&shop, &spec, "").expect_err("断る");
    assert!(why.contains("節が欠けている: why"), "{why}");
}

#[test]
fn a_status_outside_the_three_refuses_the_build() {
    let refs = references();
    let parts = Parts::load(&refs.join("acdr.template.html")).expect("読める");
    let style = Style::load(&refs).expect("読める");
    let shop = Shop {
        parts: &parts,
        style: &style,
    };
    let mut spec = sound();
    spec["status"] = json!("たぶん承認");
    let why = record::build(&shop, &spec, "").expect_err("断る");
    assert!(why.contains("状態が「たぶん承認」"), "{why}");
}

#[test]
fn the_same_input_gives_the_same_page() {
    // **日付も乱数も読まない**
    assert_eq!(made(&sound(), "").page, made(&sound(), "").page);
}

#[test]
fn an_input_that_does_not_pass_writes_nothing() {
    let folder = place("bad");
    let mut spec = sound();
    spec.as_object_mut().expect("表である").remove("decision");
    write(&folder, &spec);
    let why = record::build_record(&references(), &folder, false, false).expect_err("断る");
    assert!(why.contains("HTML は書き出さない"), "{why}");
    assert!(!folder.join("index.html").exists(), "出力が残っている");
}

#[test]
fn checking_only_does_not_write() {
    let folder = place("check");
    write(&folder, &sound());
    let got = record::build_record(&references(), &folder, true, false).expect("検査できる");
    assert_eq!(got.code, 1, "{:?}", got.lines);
    assert!(
        got.lines.iter().any(|x| x.contains("差が在る")),
        "{:?}",
        got.lines
    );
    assert!(
        !folder.join("index.html").exists(),
        "検査だけのときに書き出している"
    );
    record::build_record(&references(), &folder, false, false).expect("書ける");
    let got = record::build_record(&references(), &folder, true, false).expect("検査できる");
    assert_eq!(got.code, 0, "{:?}", got.lines);
    assert!(
        got.lines.iter().any(|x| x.contains("同一")),
        "{:?}",
        got.lines
    );
}

#[test]
fn an_accepted_record_is_sealed_once() {
    let folder = place("seal");
    let target = folder.join("対象.md");
    std::fs::write(&target, "# 題\n\n本文である。\n").expect("書ける");
    let mut spec = sound();
    spec["status"] = json!("accepted");
    spec["docs"] = json!([{"key": "m", "tab": "面", "file": target.display().to_string(),
                           "marks": []}]);
    write(&folder, &spec);
    let got = record::build_record(&references(), &folder, false, false).expect("組める");
    assert!(
        got.lines.iter().any(|x| x.contains("封印した")),
        "{:?}",
        got.lines
    );
    let now: Value =
        serde_json::from_str(&std::fs::read_to_string(folder.join("acdr.json")).expect("読める"))
            .expect("読める");
    let sealed = now["seal"]["m"].as_str().expect("在る");
    assert_eq!(sealed.len(), 64, "sha256 でない ── {sealed}");
}

#[test]
fn a_sealed_record_whose_target_moved_refuses_to_rebuild() {
    // **組み直すと承認時点の姿が失われる**
    let folder = place("drift");
    let target = folder.join("対象.md");
    std::fs::write(&target, "# 題\n\n本文である。\n").expect("書ける");
    let mut spec = sound();
    spec["status"] = json!("accepted");
    spec["seal"] = json!({"m": "0".repeat(64)});
    spec["docs"] = json!([{"key": "m", "tab": "面", "file": target.display().to_string(),
                           "marks": []}]);
    write(&folder, &spec);
    assert_eq!(record::drifted(&folder, &spec), vec!["m".to_owned()]);
    let got = record::build_record(&references(), &folder, false, false).expect("答える");
    assert_eq!(got.code, 1);
    assert!(
        got.lines.iter().any(|x| x.contains("組み直しを拒否する")),
        "{:?}",
        got.lines
    );
    assert!(
        !folder.join("index.html").exists(),
        "拒否したのに書き出している"
    );
    // **検査のときは、拒否ではなく事実として答える**
    let got = record::build_record(&references(), &folder, true, false).expect("答える");
    assert_eq!(got.code, 0);
    assert!(
        got.lines.iter().any(|x| x.contains("承認時点の姿である")),
        "{:?}",
        got.lines
    );
    // **意図する場合は force を渡す**
    let got = record::build_record(&references(), &folder, false, true).expect("組める");
    assert_eq!(got.code, 0);
    assert!(folder.join("index.html").exists());
}

#[test]
fn an_empty_seal_is_not_a_seal() {
    // **対象を持たない記録の seal は空になる** ── 鍵の有無で判定すると、そこで
    // 封印したことになってしまう
    let folder = place("empty-seal");
    let mut spec = sound();
    spec["status"] = json!("accepted");
    spec["seal"] = json!({});
    write(&folder, &spec);
    assert!(record::drifted(&folder, &spec).is_empty());
    let got = record::build_record(&references(), &folder, false, false).expect("組める");
    assert!(
        got.lines.iter().any(|x| x.contains("封印した")),
        "{:?}",
        got.lines
    );
}

#[test]
fn a_target_is_marked_and_counted() {
    let folder = place("mark");
    let target = folder.join("対象.md");
    std::fs::write(&target, "# 題\n\nこれは本文である。\n").expect("書ける");
    let mut spec = sound();
    spec["docs"] = json!([{"key": "m", "tab": "面", "file": target.display().to_string(),
                           "marks": [{"find": "本文", "before": "旧", "why": "理由"}]}]);
    write(&folder, &spec);
    let got = record::build_record(&references(), &folder, false, false).expect("組める");
    assert!(
        got.lines.iter().any(|x| x.contains("印 1 件 / 1 面")),
        "{:?}",
        got.lines
    );
    // **組んだものを、組んだ結果で検査する** ── 数を手で書くと、検査が自分の前提を
    // 確かめるだけになる
    let (spec, figure) = record::load(&references(), &folder).expect("読める");
    let built = made(&spec, &figure);
    for (name, good) in record::check(&built.page, &spec, &built) {
        assert!(good, "{name}");
    }
    assert!(
        got.lines.iter().all(|x| !x.starts_with("  NG")),
        "{:?}",
        got.lines
    );
}

#[test]
fn the_repo_root_is_found_from_a_mark() {
    let folder = place("root");
    std::fs::create_dir_all(folder.join(".git")).expect("作れる");
    let deep = folder.join("a/b");
    std::fs::create_dir_all(&deep).expect("作れる");
    assert_eq!(record::repo_root(&deep), folder);
}

#[test]
fn a_new_record_is_not_made_twice() {
    // **同じ名前が在れば作らない** ── 上書きすると、書いた決定が消える
    let folder = place("new").join("0099-ためし");
    let lines = record::new(&references(), &folder, "ためし").expect("作れる");
    assert!(lines[0].contains("acdr.json"), "{lines:?}");
    let spec: Value =
        serde_json::from_str(&std::fs::read_to_string(folder.join("acdr.json")).expect("読める"))
            .expect("読める");
    assert_eq!(spec["title"], json!("ためし"));
    assert_eq!(
        spec["no"],
        json!("ACDR 0099"),
        "番号はフォルダの名前から取る"
    );
    let why = record::new(&references(), &folder, "ためし").expect_err("断る");
    assert!(why.contains("既に在る"), "{why}");
}

#[test]
fn the_before_given_in_the_record_is_used() {
    // **欄の名前は契約（スキーマ）どおり before である** ── 日本語の名前で読むと、渡した変更前が使われない
    let doc = json!({"file": "/nowhere/x.rs", "before": "旧い中身"});
    assert_eq!(panes::before_of(&doc).as_deref(), Some("旧い中身"));
}

#[test]
fn the_before_is_read_from_the_revision_the_record_names() {
    // **rev に書いた版から変更前を読む** ── 欄の名前は契約どおり rev である
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../SKILL.md");
    let doc = json!({"file": here.display().to_string(), "rev": "HEAD"});
    let got = panes::before_of(&doc).expect("HEAD の版が在る");
    assert!(got.contains("acdr"), "{}", &got[..got.len().min(80)]);
    // 在りもしない版は None を返し、呼ぶ側が全文へ落とす
    let missing = json!({"file": here.display().to_string(), "rev": "no-such-rev-xyz"});
    assert!(panes::before_of(&missing).is_none());
}
