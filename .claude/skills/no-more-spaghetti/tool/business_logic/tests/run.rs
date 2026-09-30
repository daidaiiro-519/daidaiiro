// SPDX-License-Identifier: MIT
//! 規則の実行を事例で検証する。
//!
//!     cargo test -p nms_business_logic

use std::path::{Path, PathBuf};

use nms_business_logic::label::Verdict;
use nms_business_logic::rules;
use nms_business_logic::run;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nms-test-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

fn write_rules(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("rules.json");
    std::fs::write(&path, body).expect("書ける");
    path
}

fn tool(name: &str, args: &[&str]) -> String {
    let listed: Vec<String> = args.iter().map(|a| format!("\"{a}\"")).collect();
    format!(
        r#"{{"rule":"{name}","source":{{"record":"x"}},"scope":"x","check":{{"tool":[{}]}}}}"#,
        listed.join(",")
    )
}

#[test]
fn the_exit_code_decides_the_verdict() {
    let dir = scratch("verdict");
    let body = format!(
        r#"{{"rules":[{},{}]}}"#,
        tool("通る", &["true"]),
        tool("落ちる", &["false"])
    );
    let file = write_rules(&dir, &body);
    let report = run::check(&dir, &file, 10).expect("実行できる");
    assert_eq!(report.count(Verdict::Pass), 1);
    assert_eq!(report.count(Verdict::Fail), 1);
    assert_eq!(report.findings.len(), 1, "合格は検出に並べない");
}

#[test]
fn an_absent_tool_is_not_a_pass() {
    let dir = scratch("absent");
    let file = write_rules(
        &dir,
        &format!(r#"{{"rules":[{}]}}"#, tool("無い", &["この道具は無い"])),
    );
    let report = run::check(&dir, &file, 10).expect("実行できる");
    assert_eq!(
        report.count(Verdict::Skip),
        1,
        "道具が無いのを合格に寄せない"
    );
    assert_eq!(report.count(Verdict::Pass), 0);
    assert!(report.rules[0].reason.contains("道具が見つからない"));
}

#[test]
fn a_tool_that_is_not_a_list_is_refused() {
    let dir = scratch("notlist");
    let body = r#"{"rules":[{"rule":"文字列","source":{"record":"x"},"scope":"x","check":{"tool":"true"}}]}"#;
    let file = write_rules(&dir, body);
    let report = run::check(&dir, &file, 10).expect("実行できる");
    assert_eq!(report.count(Verdict::Skip), 1);
    assert!(report.rules[0].reason.contains("配列ではない"));
}

#[test]
fn zero_rules_is_reported() {
    let dir = scratch("zero");
    let file = write_rules(&dir, r#"{"rules":[]}"#);
    let report = run::check(&dir, &file, 10).expect("実行できる");
    // **1件も検査していない状態を、合格と同じ姿で返さない。**
    assert!(report.findings.iter().any(|x| x.contains("規則が0件")));
}

#[test]
fn a_target_outside_the_root_is_refused() {
    let dir = scratch("outside");
    let body = r#"{"rules":[{"rule":"外","source":{"record":"x"},"scope":"x","check":{"tool":["true"],"target":"../"}}]}"#;
    let file = write_rules(&dir, body);
    let report = run::check(&dir, &file, 10).expect("実行できる");
    assert_eq!(report.rules[0].verdict, Verdict::Skip);
    assert!(report.rules[0].reason.contains("成果物の場所の外"));
}

#[test]
fn a_target_that_does_not_exist_is_refused() {
    let dir = scratch("missing");
    let body = r#"{"rules":[{"rule":"無い場所","source":{"record":"x"},"scope":"x","check":{"tool":["true"],"target":"無い包み"}}]}"#;
    let file = write_rules(&dir, body);
    let report = run::check(&dir, &file, 10).expect("実行できる");
    assert_eq!(report.rules[0].verdict, Verdict::Skip);
    assert!(report.rules[0].reason.contains("実在しない"));
}

#[test]
fn a_target_inside_the_root_runs_there() {
    let dir = scratch("inside");
    std::fs::create_dir_all(dir.join("inside")).expect("作れる");
    let body = r#"{"rules":[{"rule":"中","source":{"record":"x"},"scope":"x","check":{"tool":["true"],"target":"inside"}}]}"#;
    let file = write_rules(&dir, body);
    let report = run::check(&dir, &file, 10).expect("実行できる");
    assert_eq!(report.rules[0].verdict, Verdict::Pass);
}

#[test]
fn a_tool_that_does_not_finish_is_stopped() {
    let dir = scratch("slow");
    let file = write_rules(
        &dir,
        &format!(r#"{{"rules":[{}]}}"#, tool("遅い", &["sleep", "5"])),
    );
    let report = run::check(&dir, &file, 1).expect("実行できる");
    assert_eq!(report.rules[0].verdict, Verdict::Skip);
    assert!(report.rules[0].reason.contains("1秒で終わらない"));
}

#[test]
fn only_the_middle_is_dropped_and_the_rest_is_kept() {
    let dir = scratch("long");
    let n = run::OUTPUT_HEAD + run::OUTPUT_TAIL;
    let script = format!(
        r#"import sys;sys.stdout.write("あ"+"x"*{}+"ん");sys.exit(1)"#,
        n * 2
    );
    let body = format!(
        r#"{{"rules":[{{"rule":"長い","source":{{"record":"x"}},"scope":"x","check":{{"tool":["python3","-c","{}"]}}}}]}}"#,
        script.replace('"', "\\\"")
    );
    let file = write_rules(&dir, &body);
    let report = run::check(&dir, &file, 30).expect("実行できる");
    let got = &report.rules[0];
    assert!(got.output_size > n as u64, "全体の大きさを返す");
    assert!(got.output.starts_with('あ'), "先頭を読む");
    assert!(got.output.trim_end().ends_with('ん'), "末尾も読む");
    assert!(got.output.contains("中略"), "落としたことを書く");
    assert!(got.saved, "全文を保持する");

    // 続きを読む
    let head = run::read_output(&dir, &report.run, "長い", 0, 100).expect("読める");
    assert!(head.output.starts_with('あ'));
    assert_eq!(head.next_offset, Some(100));
    let tail =
        run::read_output(&dir, &report.run, "長い", got.output_size - 10, 100).expect("読める");
    assert!(tail.next_offset.is_none(), "末尾では次の位置を返さない");
}

#[test]
fn a_stale_run_is_refused() {
    let dir = scratch("stale");
    let n = run::OUTPUT_HEAD + run::OUTPUT_TAIL;
    let script = format!(r#"import sys;sys.stdout.write("x"*{})"#, n * 2);
    let body = format!(
        r#"{{"rules":[{{"rule":"長い","source":{{"record":"x"}},"scope":"x","check":{{"tool":["python3","-c","{}"]}}}}]}}"#,
        script.replace('"', "\\\"")
    );
    let file = write_rules(&dir, &body);
    let first = run::check(&dir, &file, 30).expect("実行できる").run;
    std::thread::sleep(std::time::Duration::from_millis(5));
    let second = run::check(&dir, &file, 30).expect("実行できる").run;
    assert_ne!(first, second, "実行ごとに識別子が変わる");
    let refused = run::read_output(&dir, &first, "長い", 0, 10);
    assert!(refused.is_err(), "前の実行は断る");
    assert!(
        run::read_output(&dir, &second, "長い", 0, 10).is_ok(),
        "直近は読める"
    );
    assert!(
        run::read_output(&dir, &second, "無い規則", 0, 10).is_err(),
        "無い名前は断る"
    );
}

#[test]
fn the_rules_file_is_read_in_both_shapes() {
    let dir = scratch("shape");
    let one = r#"{"rule":"1件だけ","source":{"record":"x"},"scope":"x","check":{"tool":["true"]}}"#;
    let file = write_rules(&dir, one);
    assert_eq!(
        rules::load(&file).expect("読める").len(),
        1,
        "1件だけの形も受け取る"
    );
    let many = format!(
        r#"{{"rules":[{},{}]}}"#,
        tool("a", &["true"]),
        tool("b", &["true"])
    );
    let file = write_rules(&dir, &many);
    assert_eq!(rules::load(&file).expect("読める").len(), 2);
}

#[test]
fn the_machine_readable_keys_are_ascii() {
    for v in [Verdict::Pass, Verdict::Fail, Verdict::Skip] {
        assert!(v.key().is_ascii(), "機械が分岐する値は ASCII である");
        assert!(!v.label().is_ascii(), "画面へ出す語は対応表が持つ");
    }
}
