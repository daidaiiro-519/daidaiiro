//! 契約の検査 ── **テストの一覧の形は、基盤が持つ**（論点3 の契約3本のうちの1本）。

#[test]
fn テストの一覧は_1行1識別子である() {
    let text = "\n# これは注記である\n  order::tests::甲  \n\norder::tests::乙\n";
    let got = base::read_test_list(text);
    assert_eq!(got.len(), 2, "{:?}", got);
    assert!(got.contains("order::tests::甲"), "前後の空白を落としていない");
    assert!(!got.iter().any(|x| x.starts_with('#')), "注記を識別子にしている");
}

#[test]
fn 実行系の生の出力は_そのままでは使えない() {
    // Rust の `cargo test -- --list` は、**関数名だけ**を出す（実測 2026-09-26）。
    let raw = "明細が0件のとき確定できない: test\n2 tests, 0 benchmarks\n";
    let got = base::read_test_list(raw);
    // 「: test」も「2 tests, 0 benchmarks」も、そのまま識別子になってしまう
    assert!(got.contains("明細が0件のとき確定できない: test"),
            "生の行をそのまま識別子にしている ── 変換は渡す側の仕事である");
    assert!(got.contains("2 tests, 0 benchmarks"), "件数の行まで識別子になる");
}
