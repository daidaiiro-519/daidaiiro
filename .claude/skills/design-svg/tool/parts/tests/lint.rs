// SPDX-License-Identifier: MIT
//! 生の数値の検出器 ── **鳴るべき所で鳴り、名前へ束ねた数では鳴らない。**

use ds_parts::lint::scan;

fn hits(src: &str) -> usize {
    scan("x.rs", src).len()
}

#[test]
fn a_guessed_number_is_reported() {
    let found = scan("x.rs", "fn pad() -> f64 {\n    let w = 37.5;\n    w\n}\n");
    assert_eq!(found.len(), 1);
    assert_eq!((found[0].1, found[0].2.as_str()), (2, "pad"));
}

#[test]
fn named_constants_are_not_reported() {
    assert_eq!(
        hits("const GAP: f64 = 37.5;\nstatic TABLE: [u8; 3] = [4, 5, 6];\n"),
        0
    );
}

#[test]
fn structural_numbers_are_not_reported() {
    assert_eq!(
        hits("fn f(v: &[f64]) -> f64 { v[0] * 2.0 - 1.0 + 0.5 + v[3] + 360.0 }\n"),
        0
    );
}

#[test]
fn comments_and_strings_are_not_read() {
    assert_eq!(
        hits("// 37.5 は注釈\nfn f() -> &'static str { \"font-size: 37.5\" }\n"),
        0
    );
}

#[test]
fn tuple_fields_and_literal_tables_are_not_reported() {
    assert_eq!(hits("fn f(a: (f64, f64, f64, f64)) -> f64 { a.3 }\n"), 0);
    assert_eq!(
        hits("fn arity(c: char) -> usize { match c { 'C' => 6, 'Q' | 'S' => 4, _ => 0 } }\n"),
        0
    );
}

#[test]
fn a_missing_place_is_an_error_not_zero() {
    // **無い場所を検査して「0 箇所」と返さない。** 呼ぶ側は合格と受け取る
    assert!(ds_parts::lint::findings(std::path::Path::new("/nonexistent/place")).is_err());
}
