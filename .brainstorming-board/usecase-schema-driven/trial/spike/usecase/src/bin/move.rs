//! 射影を、雛形へ移し切れるかを測る。**移せない件が、Rust の注入点が要る範囲である。**
use base::proj;

fn 雛形(dir: &str, name: &str) -> Option<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(dir)
        .join(name);
    std::fs::read_to_string(p).ok()
}

fn 宣言の穴(dir: &str, 雛形名: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let base = 雛形名.rsplit_once('.').map(|x| x.0).unwrap_or(雛形名);
    let s = 雛形(dir, &format!("{base}.render.json"))?;
    let v: serde_json::Value = serde_json::from_str(&s).ok()?;
    v.get("slots")?.as_object().cloned()
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
    let 対象: Vec<_> = if 名.starts_with("render.") && !雛形名.ends_with(".html") {
        // 変換機は、宣言の render 欄では選ばれない ── 振る舞いを持つ種類へ当てる
        decls.iter().filter(|d| !d.ops.is_empty()).collect()
    } else {
        decls.iter().filter(|d| d.render == 名).collect()
    };
    if 対象.is_empty() {
        println!("  {名:34} 宣言が無い");
        return;
    }
    let mut 一致 = 0;
    let mut 例 = String::new();
    for d in &対象 {
        let 旧 = base::render_as(d, reg, 名).unwrap();
        // 宣言に無い値は、射影の宣言が持つ（既定値の表）
        let mut ctx = serde_json::to_value(d).unwrap();
        if let Some(s) = 宣言の穴(dir, 雛形名) {
            if let Some(o) = ctx.as_object_mut() {
                for (k, v) in s {
                    o.insert(k, v);
                }
            }
        }
        match proj::render(&tpl, &ctx) {
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
    println!("== ユースケース駆動 ==");
    let d = usecase::decls();
    let r = usecase::wire();
    for (名, 雛形名) in [
        ("render.aggregate", "aggregate.html"),
        ("render.service", "service.html"),
        ("render.usecase", "usecase.html"),
        ("render.business-domain", "business-domain.html"),
        ("render.subdomain", "subdomain.html"),
        ("render.context", "context.html"),
        ("render.skill.md", "skill.md"),
        ("render.kiro.agent.json", "kiro.agent.json"),
        ("render.claude.settings.json", "claude.settings.json"),
        ("render.codex.agent.toml", "codex.agent.toml"),
    ] {
        測る(名, "usecase/render", 雛形名, &d, &r);
    }
}
