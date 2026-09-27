//! 契約の検査 ── **テストの一覧の形は、基盤が持つ**（論点3 の契約3本のうちの1本）。

#[test]
fn テストの一覧は_1行1識別子である() {
    let text = "\n# これは注記である\n  order::tests::甲  \n\norder::tests::乙\n";
    let got = base::read_test_list(text);
    assert_eq!(got.len(), 2, "{:?}", got);
    assert!(
        got.contains("order::tests::甲"),
        "前後の空白を落としていない"
    );
    assert!(
        !got.iter().any(|x| x.starts_with('#')),
        "注記を識別子にしている"
    );
}

#[test]
fn 実行系の生の出力は_そのままでは使えない() {
    // Rust の `cargo test -- --list` は、**関数名だけ**を出す（実測 2026-09-26）。
    let raw = "明細が0件のとき確定できない: test\n2 tests, 0 benchmarks\n";
    let got = base::read_test_list(raw);
    // 「: test」も「2 tests, 0 benchmarks」も、そのまま識別子になってしまう
    assert!(
        got.contains("明細が0件のとき確定できない: test"),
        "生の行をそのまま識別子にしている ── 変換は渡す側の仕事である"
    );
    assert!(
        got.contains("2 tests, 0 benchmarks"),
        "件数の行まで識別子になる"
    );
}

// ── 仕様の木を読む（走査する場所は1つだけである） ──────────────

// **検査どうしで木を共有しない** ── 同じ場所を使うと、並行して走ったときに壊し合う
fn 仮の木(名前: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("spec-{}-{}", std::process::id(), 名前));
    let _ = std::fs::remove_dir_all(&root);
    for (rel, body) in files {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }
    root
}

const 事業領域: &str = r#"[{"kind":"business-domain","id":"BD-1","name":"甲",
  "render":"r","rules":[],"ops":[],"body":{}}]"#;

#[test]
fn 仕様の木は_根からの相対の道で読む() {
    let root = 仮の木(
        "木",
        &[
            ("business-domain.json", 事業領域),
            ("usecases/all.json", "[]"),
            ("読まない.txt", "これは JSON ではない"),
        ],
    );
    let files = base::read_spec_tree(&root).expect("読めない");
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["business-domain.json", "usecases/all.json"],
        "JSON 以外を読んでいるか、並びが道から決まっていない"
    );
    assert!(files[0].at_root(), "根に在ると判定できていない");
    assert!(!files[1].at_root(), "根の外を根と判定している");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn 根が無ければ_読めないと言う() {
    let e = base::read_spec_tree(std::path::Path::new("/在りもしない/根"))
        .err()
        .expect("在りもしない根を読めてしまった");
    assert!(e.contains("仕様の根が無い"), "{e}");
}

#[test]
fn 形の違う仕様は_どのファイルかを言う() {
    let root = 仮の木(
        "形",
        &[("business-domain.json", "{ これは JSON ではない }")],
    );
    let e = base::read_spec_tree(&root)
        .err()
        .expect("形の違う仕様を読めてしまった");
    assert!(
        e.contains("形が違う") && e.contains("business-domain.json"),
        "{e}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ── 覆われたシナリオ ── 道具が知るのは2つの集合だけである ──────────
fn 集合(xs: &[&str]) -> std::collections::BTreeSet<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn 覆われていないシナリオが出る() {
    let 要る = 集合(&["SC-A", "SC-B", "SC-C"]);
    let 覆った = 集合(&["SC-A", "SC-C"]);
    let c = base::coverage(&要る, &覆った);
    assert_eq!(c.uncovered, vec!["SC-B".to_string()]);
    assert!(c.unknown.is_empty());
}

#[test]
fn 仕様に無いシナリオを名乗ると出る() {
    let 要る = 集合(&["SC-A"]);
    let 覆った = 集合(&["SC-A", "SC-TYPO"]);
    let c = base::coverage(&要る, &覆った);
    assert!(c.uncovered.is_empty());
    assert_eq!(c.unknown, vec!["SC-TYPO".to_string()]);
}

#[test]
fn そろっていれば何も出ない() {
    let 要る = 集合(&["SC-A", "SC-B"]);
    let c = base::coverage(&要る, &要る);
    assert!(c.uncovered.is_empty() && c.unknown.is_empty());
}

// ── テストの側の契約は、本体が持つ ── シナリオを持つ宣言の get にだけ付く ──
#[test]
fn シナリオを持つ宣言の_get_に契約が付く() {
    let decls = base::read(r#"[
      {"kind":"k","id":"A-1","name":"甲","render":"r","ops":[{"name":"o","nodes":[{"id":"SC-X","name":"x"},{"id":"SC-Y","name":"y"}]}]},
      {"kind":"k","id":"B-1","name":"乙","render":"r"}
    ]"#).unwrap();
    let reg = base::Registry::new();
    let a: serde_json::Value = serde_json::from_str(&base::ops::get(&decls, "A-1", None, &reg).unwrap()).unwrap();
    let c = a["test_contract"].as_str().expect("契約が付いていない");
    assert!(c.contains("SCENARIO_TRACE") && c.contains("SC-X") && c.contains("SC-Y"));
    assert_eq!(a["decl"]["id"], "A-1");
    let b: serde_json::Value = serde_json::from_str(&base::ops::get(&decls, "B-1", None, &reg).unwrap()).unwrap();
    assert!(b.get("test_contract").is_none(), "シナリオの無い宣言に契約が付いた");
}
