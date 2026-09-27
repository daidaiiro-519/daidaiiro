//! `schema covered <記録> [宣言ID]` ── テストの側が網羅したシナリオを、仕様と突き合わせる。
//!
//! 記録は1行1ID のテキストである。**どう作ったかを、この道具は知らない。**
use std::collections::BTreeSet;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = a.first() else {
        eprintln!("使い方: covered <記録> [宣言ID]");
        std::process::exit(2);
    };
    // 記録のファイルが無いのは、どのテストも記録しなかったときの自然な結果である。
    // 誤用として止めず、記録0件として「テストが無い」の側に帰着させる。
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("記録のファイルが無い ── どのテストもシナリオID を記録していない: {path}");
            String::new()
        }
        Err(e) => {
            eprintln!("記録を読めない: {path} ── {e}");
            std::process::exit(2);
        }
    };
    let decls = usecase::decls();
    // 宣言ID を渡せば、その宣言のシナリオだけを必要とする集合にする（実装の PR の範囲）
    let 必要である: BTreeSet<String> = decls
        .iter()
        .filter(|d| a.get(1).is_none_or(|id| &d.id == id))
        .flat_map(|d| d.nodes.iter().map(|n| n.id.clone()))
        .collect();
    let 網羅 = base::read_test_list(&text);
    let c = base::coverage(&必要である, &網羅);

    for id in &必要である {
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
        println!("× {id}　── 仕様に無いシナリオを記録している");
    }
    let bad = c.uncovered.len() + c.unknown.len();
    println!(
        "網羅 {}/{} ／ 見つかったこと {bad} 件",
        必要である.len() - c.uncovered.len(),
        必要である.len()
    );
    std::process::exit(if bad == 0 { 0 } else { 1 });
}
