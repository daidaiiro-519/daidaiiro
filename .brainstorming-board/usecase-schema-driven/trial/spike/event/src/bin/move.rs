//! 射影を、雛形へ移し切れるかを測る。**移せない件が、Rust の注入点が要る範囲である。**
use base::proj;

fn 雛形(dir: &str, name: &str) -> Option<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(dir)
        .join(name);
    std::fs::read_to_string(p).ok()
}

fn 差の先頭(a: &str, b: &str) -> String {
    for (i, (x, y)) in a.lines().zip(b.lines()).enumerate() {
        if x != y {
            return format!("{}行目\n      いま  {x}\n      雛形  {y}", i + 1);
        }
    }
    format!(
        "行数が違う（いま {} ／ 雛形 {}）",
        a.lines().count(),
        b.lines().count()
    )
}

fn 測る(名: &str, dir: &str, 雛形名: &str, decls: &[base::Decl], reg: &base::Registry) {
    let Some(tpl) = 雛形(dir, 雛形名) else {
        println!("  {名:34} 雛形が無い");
        return;
    };
    let 対象: Vec<_> = decls.iter().filter(|d| d.render == 名).collect();
    if 対象.is_empty() {
        println!("  {名:34} 宣言が無い");
        return;
    }
    let mut 一致 = 0;
    let mut 例 = String::new();
    for d in &対象 {
        let 旧 = base::render_as(d, reg, 名).unwrap();
        match proj::render(&tpl, &serde_json::to_value(d).unwrap()) {
            Ok(新) if 新 == 旧 => 一致 += 1,
            Ok(新) => {
                if 例.is_empty() {
                    例 = 差の先頭(&旧, &新);
                }
            }
            Err(e) => {
                if 例.is_empty() {
                    例 = format!("出せない ── {e}");
                }
            }
        }
    }
    let 印 = if 一致 == 対象.len() { "○" } else { "×" };
    println!("  {印} {名:32} {一致}/{} 一致", 対象.len());
    if !例.is_empty() {
        println!("      {例}");
    }
}

fn main() {
    println!("== 業務イベント駆動（注入の試験） ==");
    let d = event::decls();
    let r = event::wire();
    for (名, 雛形名) in [
        ("render.event", "event.html"),
        ("render.policy", "policy.html"),
    ] {
        測る(名, "event/render", 雛形名, &d, &r);
    }
}
