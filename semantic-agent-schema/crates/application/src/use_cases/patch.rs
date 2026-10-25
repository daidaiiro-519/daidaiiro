//! patch（検査①：適用する前に入力を見る）

use crate::use_cases::validate::{is_declaration, write_checked};
use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::values::etag::ETag;
use semantic_agent_schema_instance::aggregates::instance::Instance;
use serde_json::{Value, json};

/// Concrete かインスタンスに JSON Patch を適用する。検査①で外れたら断る。
/// 検査②で、この書き込みが新しく持ち込んだ機械の violated があれば書かずに返す。
/// if_match に読んだときの ETag を渡すと、今の ETag と違うとき（ほかの誰かが先に書いたとき）に断る（楽観ロック）
pub fn patch(ports: &Ports<'_>, path: &str, ops: &Value, if_match: Option<&str>) -> Outcome {
    let document = match ports.instances.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    // 楽観ロック：読んだときの ETag と今の ETag が違えば、ほかの誰かが先に書き換えている
    if let Some(given) = if_match
        && !ETag::of(&document).matches(given)
    {
        return Outcome::refused(
            "",
            "読んだあとに中身が変わっている（ETag が違う）。読み直してから書く",
        );
    }
    let next = if is_declaration(&document) {
        ConcreteSchema::from_document(document.clone())
            .map_err(|e| (String::new(), e))
            .and_then(|c| {
                c.patched(ops)
                    .map(|n| n.document().clone())
                    .map_err(|r| (r.pointer().to_string(), r.reason().to_string()))
            })
    } else {
        Instance::from_document(document.clone())
            .map_err(|e| (String::new(), e))
            .and_then(|i| {
                i.patched(ops)
                    .map(|n| n.document().clone())
                    .map_err(|r| (r.pointer().to_string(), r.reason().to_string()))
            })
    };
    let next = match next {
        Ok(n) => n,
        Err((at, why)) => return Outcome::refused(&at, &why),
    };
    let declaration = is_declaration(&document);
    write_checked(
        ports,
        path,
        Some(&document),
        &next,
        json!({"path": path}),
        &|doc| {
            if declaration {
                ports.contracts.write(path, doc)
            } else {
                ports.instances.write(path, doc)
            }
        },
    )
}
