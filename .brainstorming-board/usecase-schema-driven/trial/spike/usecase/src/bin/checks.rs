//! 検査6件を、concrete の JSON Schema へ移し切れるかを測る。
use base::schema;
use serde_json::Value;

fn 種類のschema() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schema/usecase-driven.schema.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn 検査(decls: &[Value], s: &Value) -> Vec<String> {
    let mut out = vec![];
    for d in decls {
        let kind = d["kind"].as_str().unwrap_or("");
        if let Some(sub) = s.get("kinds").and_then(|k| k.get(kind)) {
            schema::validate(sub, d, d["id"].as_str().unwrap_or("?"), &mut out);
        }
    }
    out
}

fn main() {
    let s = 種類のschema();
    let decls: Vec<Value> = usecase::decls()
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect();

    println!("== いまの仕様を、schema で検査する ==");
    let bad = 検査(&decls, &s);
    println!(
        "  検出 {} 件{}",
        bad.len(),
        if bad.is_empty() {
            "（Rust の検査6件と同じ判定）"
        } else {
            ""
        }
    );
    for b in &bad {
        println!("    × {b}");
    }

    println!("\n== 壊してみる ── Rust の check.sumWithin と同じ判定が出るか ==");
    let mut 壊れ = decls.clone();
    for d in 壊れ.iter_mut() {
        if d["kind"] == "aggregate" {
            d["ops"][0]["writes"] = Value::Array(vec![]);
            break;
        }
    }
    for b in 検査(&壊れ, &s) {
        println!("    × {b}");
    }

    println!("\n== 語の欄を1つ落とす ── Rust には無い検査である ==");
    let mut 壊れ2 = decls.clone();
    for d in 壊れ2.iter_mut() {
        if d["kind"] == "bounded-context" {
            d["body"]["language"][0]
                .as_object_mut()
                .unwrap()
                .remove("means");
            d["body"]["language"][1]["note"] = Value::String("余分".into());
            break;
        }
    }
    for b in 検査(&壊れ2, &s) {
        println!("    × {b}");
    }
}
