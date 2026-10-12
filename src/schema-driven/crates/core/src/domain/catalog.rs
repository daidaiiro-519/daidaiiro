//! 部品のカタログの検査と読み取り。カタログの中身（部品の一覧 ・ HTML ・ CSS ・ JS ・ デザインシステム）は concrete が持つ。
//! 基盤は部品の名前を持たず、部品1つの書き方の形（schemas/catalog.schema.json の枠）と、
//! ページと節のレイアウトのテンプレートの id とプレースホルダーだけを決める。

use crate::domain::design_system;
use crate::domain::schema::Schema;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// ページのレイアウトのテンプレートの id。
pub const PAGE: &str = "page";
/// 節のレイアウトのテンプレートの id。
pub const SECTION: &str = "section";
/// ページのテンプレートへ基盤が入れる値の名前。ほかのプレースホルダーには、ページテンプレートの head の値が入る。
pub const PAGE_SLOTS: [&str; 5] = ["title", "tokens", "style", "script", "sections"];
/// 節のテンプレートへ基盤が入れる値の名前。
pub const SECTION_SLOTS: [&str; 3] = ["heading", "help", "body"];
/// 部品の呼び出しのうち、基盤が読む欄。部品の入力ではない。
pub const CONTROL: [&str; 3] = ["component", "show", "each"];

/// 部品の HTML を、id → 中身に分ける。
pub fn parse_parts(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut rest = source;
    while let Some(start) = rest.find("<template id=\"") {
        let after = &rest[start + "<template id=\"".len()..];
        let id_end = after.find('"').ok_or("部品の id が閉じていない")?;
        let id = after[..id_end].to_owned();
        let body_start = after[id_end..]
            .find('>')
            .ok_or("部品の開きタグが閉じていない")?
            + id_end
            + 1;
        let body_end = after[body_start..]
            .find("</template>")
            .ok_or_else(|| format!("部品 {id} が閉じていない"))?
            + body_start;
        if out
            .insert(id.clone(), after[body_start..body_end].to_owned())
            .is_some()
        {
            return Err(format!("部品 {id} が2つある"));
        }
        rest = &after[body_end + "</template>".len()..];
    }
    Ok(out)
}

/// テンプレートのプレースホルダー {{名前}} の一覧。
pub fn slots_of(template: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        let after = &rest[open + 2..];
        match after.find("}}") {
            Some(close) => {
                out.insert(after[..close].to_owned());
                rest = &after[close + 2..];
            }
            None => break,
        }
    }
    out
}

/// 部品の一覧のスキーマの $defs（部品の名前 → 入力の形）。
pub fn shapes(schema: &Value) -> BTreeMap<String, Value> {
    schema["$defs"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, shape)| (name.clone(), shape.clone()))
        .collect()
}

/// 部品の一覧のスキーマを、枠（schemas/catalog.schema.json）で検査する。
fn frame_findings(schema: &Value) -> Vec<String> {
    let frame = Schema::new(
        "catalog.schema.json",
        serde_json::from_str(crate::embedded::schema_text("schema-driven/catalog@v1"))
            .unwrap_or(Value::Null),
        vec![],
    );
    match frame.validate(schema) {
        Ok(validation) => validation
            .unfilled
            .iter()
            .map(|unfilled| format!("部品の一覧：{} が無い", unfilled.property()))
            .chain(
                validation
                    .errors
                    .iter()
                    .map(|error| format!("部品の一覧：{} {}", error.property(), error.reason())),
            )
            .collect(),
        Err(error) => vec![format!("部品の一覧：{}", error.0)],
    }
}

/// 部品の一覧と HTML が合っているかを検査する：レイアウトのテンプレートがあり、部品ごとに同じ id のテンプレートがあり、
/// 入力の名前がそのテンプレートのプレースホルダーにある。
fn template_findings(
    shapes: &BTreeMap<String, Value>,
    templates: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for layout in [PAGE, SECTION] {
        if !templates.contains_key(layout) {
            out.push(format!(
                "parts.html に、レイアウトのテンプレート {layout} が無い"
            ));
        }
    }
    if let Some(section) = templates.get(SECTION) {
        for slot in slots_of(section) {
            if !SECTION_SLOTS.contains(&slot.as_str()) {
                out.push(format!(
                    "テンプレート {SECTION} の {{{{{slot}}}}} に入れる値が無い（入れられるのは heading ・ help ・ body）"
                ));
            }
        }
    }
    for (name, shape) in shapes {
        let Some(template) = templates.get(name) else {
            out.push(format!("部品 {name} のテンプレートが parts.html に無い"));
            continue;
        };
        let slots = slots_of(template);
        for input in shape["properties"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(key, _)| key)
        {
            if !slots.contains(input) {
                out.push(format!(
                    "部品 {name} の入力 {input} を入れるプレースホルダーが、テンプレートに無い"
                ));
            }
        }
    }
    out
}

/// カタログを検査する。schema は部品の一覧のスキーマ、parts は部品の HTML、style は CSS、system はデザインシステム。
/// 見つかったことを、見つけた順に返す。空なら契約を満たしている。
pub fn check(schema_text: &str, parts: &str, style: &str, system: &str) -> Vec<String> {
    let schema: Value = match serde_json::from_str(schema_text) {
        Ok(value) => value,
        Err(error) => return vec![format!("部品の一覧が JSON として読めない：{error}")],
    };
    let mut out = frame_findings(&schema);
    if out.is_empty() {
        match parse_parts(parts) {
            Ok(templates) => out.extend(template_findings(&shapes(&schema), &templates)),
            Err(error) => out.push(format!("parts.html：{error}")),
        }
    }
    out.extend(
        design_system::check(system, parts, style)
            .into_iter()
            .map(|finding| format!("デザインシステム：{finding}")),
    );
    out
}

#[cfg(test)]
mod tests {
    // 基盤は部品の名前を持たず、カタログが枠に従い、部品の一覧と HTML が合うかだけを見る。
    use super::{check, parse_parts, slots_of};
    use serde_json::{json, Value};

    const PARTS: &str = r#"<template id="page"><h1>{{title}}</h1>{{sections}}</template>
<template id="section"><h2>{{heading}}</h2>{{body}}</template>
<template id="pill"><b>{{text}}</b></template>"#;

    const SYSTEM: &str = r##"{"scope": ":root", "tokens": {"ink": "#111d1a"}, "contrast": []}"##;

    fn schema() -> Value {
        json!({"$schema": "schema-driven/catalog@v1", "$id": "example/catalog@v1",
               "$defs": {"pill": {"type": "object", "description": "短い分類",
                                  "properties": {"text": {"type": "string"}}, "required": ["text"],
                                  "x-states": {"default": "押せない"}}}})
    }

    fn findings(schema: Value, parts: &str) -> String {
        check(&schema.to_string(), parts, "", SYSTEM).join("\n")
    }

    #[test]
    fn a_catalog_that_follows_the_frame_has_no_findings() {
        assert_eq!(findings(schema(), PARTS), "");
    }

    #[test]
    fn a_part_needs_its_states_and_description() {
        let mut broken = schema();
        broken["$defs"]["pill"]
            .as_object_mut()
            .unwrap()
            .remove("x-states");
        assert!(findings(broken, PARTS).contains("x-states"));
        let mut broken = schema();
        broken["$defs"]["pill"]
            .as_object_mut()
            .unwrap()
            .remove("description");
        assert!(findings(broken, PARTS).contains("description"));
    }

    #[test]
    fn layout_ids_and_control_names_are_not_part_names() {
        let mut broken = schema();
        broken["$defs"]["page"] = broken["$defs"]["pill"].clone();
        assert!(findings(broken, PARTS).starts_with("部品の一覧："));
        let mut broken = schema();
        broken["$defs"]["pill"]["properties"]["each"] = json!({});
        assert!(findings(broken, PARTS).starts_with("部品の一覧："));
    }

    #[test]
    fn every_part_has_a_template_and_every_input_a_placeholder() {
        let mut more = schema();
        more["$defs"]["note"] = more["$defs"]["pill"].clone();
        more["$defs"]["pill"]["properties"]["tone"] = json!({"type": "string"});
        assert_eq!(
            findings(more, PARTS),
            "部品 note のテンプレートが parts.html に無い\n部品 pill の入力 tone を入れるプレースホルダーが、テンプレートに無い"
        );
    }

    #[test]
    fn the_layout_templates_are_required() {
        let found = findings(
            schema(),
            r#"<template id="pill"><b>{{text}}</b></template>"#,
        );
        assert_eq!(
            found,
            "parts.html に、レイアウトのテンプレート page が無い\nparts.html に、レイアウトのテンプレート section が無い"
        );
        let found = findings(schema(), &PARTS.replace("{{body}}", "{{body}}{{anchor}}"));
        assert!(found.contains("{{anchor}}"), "{found}");
    }

    #[test]
    fn the_design_system_is_checked_with_the_catalog() {
        let found = check(&schema().to_string(), PARTS, "b{color:#ff0000}", SYSTEM).join("\n");
        assert!(found.starts_with("デザインシステム："), "{found}");
    }

    #[test]
    fn templates_are_split_by_id_and_slots_are_listed() {
        let templates = parse_parts(PARTS).unwrap();
        assert_eq!(
            templates.keys().collect::<Vec<_>>(),
            vec!["page", "pill", "section"]
        );
        assert_eq!(
            slots_of(&templates["page"]).into_iter().collect::<Vec<_>>(),
            vec!["sections", "title"]
        );
        assert_eq!(
            parse_parts(&format!("{PARTS}{PARTS}")).unwrap_err(),
            "部品 page が2つある"
        );
    }
}
