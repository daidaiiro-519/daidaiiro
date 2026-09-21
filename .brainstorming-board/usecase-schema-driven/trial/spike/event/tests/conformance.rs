//! 注入の試験 ── 語彙が1つも重ならない方法論を、**基盤を書き換えずに**入れられるか。

#[test]
fn 宣言と関門が一致する() {
    let decls = event::decls();
    let reg = event::wire();
    assert!(base::operation_holes(&decls, &reg).is_empty());
}

#[test]
fn 宣言された検査の名前には実体が在る() {
    let decls = event::decls();
    let reg = event::wire();
    assert!(base::missing_checks(&decls, &reg).is_empty());
}

#[test]
fn 射影を持たない型は無い() {
    let decls = event::decls();
    let reg = event::wire();
    assert!(base::without_render(&decls, &reg).is_empty());
}

#[test]
fn 人が読む形はHTMLである() {
    let decls = event::decls();
    let reg = event::wire();
    for d in &decls {
        assert!(base::render(d, &reg).expect("射影が無い").starts_with("<h1>"));
    }
}

#[test]
fn 対応表で三つの引き算が全部ゼロになる() {
    let decls = event::decls();
    let d = base::drift(&base::node_ids(&decls), &event::bindings(), &event::test_list());
    assert!(d.is_empty(), "{:?}", d);
}
