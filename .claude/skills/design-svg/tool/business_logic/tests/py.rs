// SPDX-License-Identifier: MIT
//! 移す前と同じ数を出すための道具 ── **数の書き方と、足し算の順序。**
//!
//! 出力は1バイトでも違えば別の図である。Python の `repr(float)` と `sum()` は、素朴な書き方とは
//! 違う結果を返すので、そこを事例で縛る。

use ds_business_logic::intset::IntSet;
use ds_business_logic::py::{fixed, float, sum};

#[test]
fn float_is_written_like_python() {
    assert_eq!(float(12.0), "12.0");
    assert_eq!(float(0.1), "0.1");
    assert_eq!(float(1e-7), "1e-07");
    assert_eq!(float(1e16), "1e+16");
    assert_eq!(float(-0.5), "-0.5");
    assert_eq!(float(123_456.789), "123456.789");
}

#[test]
fn fixed_rounds_half_to_even_on_the_binary_value() {
    // `format(x, ".1f")` と同じ ── 2進で表した値に対して丸める
    assert_eq!(fixed(0.25, 1), "0.2");
    assert_eq!(fixed(0.35, 1), "0.3");
    assert_eq!(fixed(2.5, 0), "2");
}

#[test]
fn sum_is_compensated() {
    // Python 3.12 以降の `sum()` は補正付きで足す ── 素朴に足すと最後の桁が違う
    let xs = [0.1; 10];
    assert_eq!(sum(xs), 1.0);
    assert_ne!(xs.iter().fold(0.0, |a, b| a + b), 1.0);
    assert_eq!(sum([1e100, 1.0, -1e100, 1.0]), 2.0);
}

#[test]
fn small_int_set_iterates_like_cpython() {
    // CPython の集合は小さな整数を、表の位置の順に辿る。表が組み直されると順が変わる
    let mut s = IntSet::new();
    for i in [9, 1, 17, 3] {
        s.add(i);
    }
    let order = s.iter();
    assert_eq!(
        order,
        vec![9, 3, 1, 17],
        "python3 -c \"s=set(); [s.add(i) for i in (9,1,17,3)]; print(list(s))\""
    );
}
