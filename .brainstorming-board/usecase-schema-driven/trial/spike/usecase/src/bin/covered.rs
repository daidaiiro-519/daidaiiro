//! `schema covered <記録> [宣言ID]` ── テストの側が覆ったシナリオを、仕様と突き合わせる。
//!
//! 記録は1行1ID のテキストである。**どう作ったかを、この道具は知らない。**
use std::collections::BTreeSet;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = a.first() else {
        eprintln!("使い方: covered <記録> [宣言ID]");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("記録を読めない: {path} ── {e}");
        std::process::exit(2);
    });
    let decls = usecase::decls();
    // 宣言ID を渡せば、その宣言のシナリオだけを要る集合にする（実装の PR の範囲）
    let 要る: BTreeSet<String> = decls
        .iter()
        .filter(|d| a.get(1).is_none_or(|id| &d.id == id))
        .flat_map(|d| d.nodes.iter().map(|n| n.id.clone()))
        .collect();
    let 覆った = base::read_test_list(&text);
    let c = base::coverage(&要る, &覆った);

    for id in &要る {
        let name = decls
            .iter()
            .flat_map(|d| &d.nodes)
            .find(|n| &n.id == id)
            .map(|n| n.name.as_str())
            .unwrap_or("");
        if c.uncovered.contains(id) {
            println!("× {id}　{name}　── テストが無い");
        } else {
            println!("○ {id}　{name}");
        }
    }
    for id in &c.unknown {
        println!("× {id}　── 仕様に無いシナリオを名乗っている");
    }
    let bad = c.uncovered.len() + c.unknown.len();
    println!(
        "覆った {}/{} ／ 見つかったこと {bad} 件",
        要る.len() - c.uncovered.len(),
        要る.len()
    );
    std::process::exit(if bad == 0 { 0 } else { 1 });
}
