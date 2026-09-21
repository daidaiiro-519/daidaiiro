//! 生成コマンド ── 宣言を読み、**射影の名前を渡して出す**。
//!
//! 人が読む形も、つなぎ目も、**同じ1つの動詞で出る** ── 射影である。
//! **テストは出さない**（論点1 の決着）。
fn main() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let which = std::env::args().nth(1);
    for d in &decls {
        let name = which.clone().unwrap_or_else(|| d.render.clone());
        match base::render_as(d, &reg, &name) {
            Some(s) => println!("{}\n---", s),
            None => eprintln!("射影が無い: {} ({})", d.id, name),
        }
    }
    let found = base::audit_bindings(
        &base::node_ids(&decls),
        &usecase::bindings(),
        &usecase::test_list(),
    );
    for f in &found {
        eprintln!("{}", f.line());
    }
    eprintln!("見つかったこと {} 件", found.len());
}
