//! 5つの操作を、実物で流す。**出力をそのまま数えてトークンを測る。**
use base::ops;

fn 見出し(s: &str) {
    println!("\n@@{s}@@");
}

fn main() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let 引数: Vec<String> = std::env::args().skip(1).collect();
    let 的 = 引数.first().map(|s| s.as_str()).unwrap_or("all");

    if 的 == "all" || 的 == "list" {
        見出し("list");
        println!("{}", ops::list(&decls, None));
    }
    if 的 == "all" || 的 == "refs" {
        見出し("refs AGG-01J7Q4K");
        let (先, 元) = ops::refs(&decls, "AGG-01J7Q4K").unwrap();
        println!("指している　 {}", 先.join(" "));
        println!("指されている {}", 元.join(" "));
    }
    if 的 == "all" || 的 == "get" {
        見出し("get AGG-01J7Q4K");
        println!("{}", ops::get(&decls, "AGG-01J7Q4K", None, &reg).unwrap());
    }
    if 的 == "all" || 的 == "get-as" {
        見出し("get AGG-01J7Q4K --as render.aggregate");
        print!(
            "{}",
            ops::get(&decls, "AGG-01J7Q4K", Some("render.aggregate"), &reg).unwrap()
        );
    }
    if 的 == "all" || 的 == "validate" {
        見出し("validate");
        let s: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("schema/usecase-driven.schema.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let bad = ops::validate(&decls, &s["kinds"]);
        println!("検出 {} 件", bad.len());
        for b in &bad {
            println!("  × {b}");
        }
    }
    if 的 == "all" || 的 == "bind" {
        見出し("bind");
        let mut b = usecase::bindings();
        b.bind("SC-01J7Q4M", "usecase::tests::明細が0件のとき確定できない");
        println!("対応表へ1行入った（いま {} 行）", b.pairs().len());
    }
    if 的 == "dump" {
        print!("{}", serde_json::to_string(&decls).unwrap());
    }
}
