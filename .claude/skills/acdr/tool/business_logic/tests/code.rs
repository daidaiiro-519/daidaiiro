// SPDX-License-Identifier: MIT
//! コードと差分の組み立てを、事例で検証する。
//!
//! **差分の取り方は、移す前と1行まで同じでなければならない** ── 違うと、同じ入力から
//! 別の差分が出る。
//!
//!     cargo test -p acd_business_logic

use std::path::PathBuf;

use acd_business_logic::code::{self, Op};
use acd_business_logic::template::Parts;
use serde_json::{json, Value};

fn parts() -> Parts {
    let references = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    Parts::load(&references.join("acdr.template.html")).expect("読める")
}

fn change(find: &str, before: &str, why: &str) -> Value {
    json!({"find": find, "before": before, "why": why})
}

#[test]
fn only_a_declared_extension_is_code() {
    // **ここに無い拡張子は、コードとして扱わない**
    assert!(code::is_code(".rs") && code::is_code(".PY") && code::is_code(".yml"));
    assert!(!code::is_code(".md") && !code::is_code(".html") && !code::is_code(""));
    assert_eq!(code::lang_of(".tsx"), Some("js"));
    assert_eq!(code::lang_of(".svg"), Some("xml"));
    assert_eq!(code::lang_of(".md"), None);
}

#[test]
fn a_template_is_read_by_the_extension_before_tmpl() {
    // **雛形（.tmpl）は、その前の拡張子で言語を決める** ── 雛形の差分をコードの差分として示す
    use std::path::Path;
    assert_eq!(code::ext_of(Path::new("a/install.sh.tmpl")), ".sh");
    assert_eq!(code::ext_of(Path::new("release.yml.tmpl")), ".yml");
    assert!(code::is_code(&code::ext_of(Path::new("install.ps1.tmpl"))));
    assert_eq!(code::ext_of(Path::new("contract.RS.tmpl")), ".rs");
    assert_eq!(code::ext_of(Path::new("x/lib.rs")), ".rs");
    // 前の拡張子が無い雛形は、.tmpl のままで、コードとして扱わない
    assert_eq!(code::ext_of(Path::new("gitignore.tmpl")), ".tmpl");
    assert!(!code::is_code(&code::ext_of(Path::new("gitignore.tmpl"))));
}

#[test]
fn the_lines_are_numbered_from_one() {
    let got = code::render_code(&parts(), "a\nb\n", ".py", &[]).expect("組める");
    assert!(
        got.contains(">1<") && got.contains(">2<") && got.contains(">3<"),
        "{got}"
    );
}

#[test]
fn an_empty_line_does_not_collapse() {
    let got = code::render_code(&parts(), "a\n\nb", ".py", &[]).expect("組める");
    assert!(got.contains("&nbsp;"), "空の行が潰れている ── {got}");
}

#[test]
fn a_mark_lands_on_one_line_only() {
    // **同じ行に2件は付けない** ── 入れ子になるためである
    let got = code::render_code(
        &parts(),
        "let a = 1;\nlet a = 2;\n",
        ".rs",
        &[change("let a", "旧", "理由")],
    )
    .expect("組める");
    assert_eq!(got.matches("<mark class=\"chg\"").count(), 1, "{got}");
}

#[test]
fn a_keyword_is_painted() {
    let got = code::render_code(&parts(), "fn main() {}\n", ".rs", &[]).expect("組める");
    assert!(got.contains(">fn</span>"), "{got}");
}

#[test]
fn a_number_is_painted() {
    let got = code::render_code(&parts(), "x = 42\n", ".py", &[]).expect("組める");
    assert!(got.contains(">42</span>"), "{got}");
}

#[test]
fn a_comment_is_painted() {
    let got = code::render_code(&parts(), "# なぜ\n", ".py", &[]).expect("組める");
    assert!(got.contains(">#"), "{got}");
}

#[test]
fn the_source_is_escaped_not_interpreted() {
    // **コードは本文ではない** ── 通すと、対象の文書が頁の構造を書き換えられる
    let got = code::render_code(&parts(), "<script>x</script>\n", ".js", &[]).expect("組める");
    assert!(got.contains("&lt;script&gt;"), "{got}");
    assert!(!got.contains("<script>x"), "{got}");
}

#[test]
fn an_identical_pair_has_no_hunks() {
    let same: Vec<&str> = vec!["a", "b", "c"];
    assert!(code::hunks(&same, &same, 3).is_empty());
}

#[test]
fn the_opcodes_follow_the_source() {
    // 1行だけ置き換わった場合
    let old = vec!["a", "b", "c"];
    let new = vec!["a", "x", "c"];
    let got = code::opcodes(&old, &new);
    assert_eq!(
        got,
        vec![
            (Op::Equal, 0, 1, 0, 1),
            (Op::Replace, 1, 2, 1, 2),
            (Op::Equal, 2, 3, 2, 3),
        ],
        "{got:?}"
    );
}

#[test]
fn an_insert_and_a_delete_are_told_apart() {
    let got = code::opcodes(&["a", "c"], &["a", "b", "c"]);
    assert!(got.iter().any(|(k, ..)| *k == Op::Insert), "{got:?}");
    assert!(!got.iter().any(|(k, ..)| *k == Op::Delete), "{got:?}");
    let got = code::opcodes(&["a", "b", "c"], &["a", "c"]);
    assert!(got.iter().any(|(k, ..)| *k == Op::Delete), "{got:?}");
    assert!(!got.iter().any(|(k, ..)| *k == Op::Insert), "{got:?}");
}

#[test]
fn a_far_apart_change_makes_two_hunks() {
    // **途中で切ると文脈が落ちる** ── 近い変化は1つにまとめ、離れたものは分ける
    // **行は互いに違うものにする** ── 同じ行が並ぶと、どこを合わせても一致するので、
    // まとまりの切れ目が行の中身で決まらなくなる（実測 ── 移す前も同じ判定である）
    let numbered: Vec<String> = (0..40).map(|i| i.to_string()).collect();
    let old: Vec<&str> = numbered.iter().map(String::as_str).collect();
    let mut new = old.clone();
    new[1] = "a";
    new[35] = "b";
    assert_eq!(code::hunks(&old, &new, 3).len(), 2);
    let mut near = old.clone();
    near[1] = "a";
    near[4] = "b";
    assert_eq!(code::hunks(&old, &near, 3).len(), 1);
}

#[test]
fn the_context_is_kept_around_a_hunk() {
    let old: Vec<&str> = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"];
    let mut new = old.clone();
    new[4] = "x";
    let hunks = code::hunks(&old, &new, 3);
    assert_eq!(hunks.len(), 1);
    // 変化2行（- と +）と、前後3行ずつ
    assert_eq!(hunks[0].len(), 8, "{:?}", hunks[0]);
    assert_eq!(hunks[0].iter().filter(|(_, _, m, _)| *m != ' ').count(), 2);
}

#[test]
fn a_hunk_without_a_reason_is_counted() {
    // **Git は差分を出すが、なぜ変えたかを出さない** ── そこを埋めるのがこの道具である
    let diff = code::render_diff(&parts(), "a\nb\nc\n", "a\nx\nc\n", ".py", &[]).expect("組める");
    assert_eq!((diff.hunks, diff.explained), (1, 0));
    assert!(diff.body.contains("理由が付いていない"), "{}", diff.body);
}

#[test]
fn a_reason_attaches_to_the_added_line() {
    let diff = code::render_diff(
        &parts(),
        "a\nb\nc\n",
        "a\nx\nc\n",
        ".py",
        &[change("x", "b", "理由")],
    )
    .expect("組める");
    assert_eq!((diff.hunks, diff.explained), (1, 1));
    assert_eq!(
        diff.body.matches("<mark class=\"chg\"").count(),
        1,
        "{}",
        diff.body
    );
}

#[test]
fn one_reason_marks_one_hunk_only() {
    // **1つのまとまりに2つの印を付けない**
    let old = "a\nb\nb\nc\n";
    let new = "a\nx\nx\nc\n";
    let diff =
        code::render_diff(&parts(), old, new, ".py", &[change("x", "b", "理由")]).expect("組める");
    assert_eq!(
        diff.body.matches("<mark class=\"chg\"").count(),
        1,
        "{}",
        diff.body
    );
}

#[test]
fn the_default_before_names_the_line_above() {
    // 変更前を渡さなければ、差分そのものが変更前を示している
    let diff = code::render_diff(
        &parts(),
        "a\nb\nc\n",
        "a\nx\nc\n",
        ".py",
        &[json!({"find": "x", "why": "理由"})],
    )
    .expect("組める");
    assert!(
        diff.body.contains("上の - の行が変更前である"),
        "{}",
        diff.body
    );
}

#[test]
fn an_unknown_extension_is_refused() {
    assert!(code::render_code(&parts(), "x", ".md", &[]).is_err());
    assert!(code::render_diff(&parts(), "x", "y", ".md", &[]).is_err());
}

#[test]
fn a_code_line_is_marked_up_as_code() {
    // **コードの行はコードの要素に入れる** ── 文書の検査（doc-writing-skills）は、コードの要素の中を
    // 書き手の文として判定しない。入れないと、差分に並んだ注記を本文として判定する
    let diff = code::render_diff(&parts(), "a\nb\n", "a\nx\n", ".py", &[]).expect("組める");
    assert!(
        diff.body.contains("<td class=\"cd\"><code>"),
        "{}",
        diff.body
    );
}
