//! Skill の項目（skill.json）から、AI ツールが読む SKILL.md を決まった形で作る。同じ項目からは同じ文字列になる。
//!
//! 形：frontmatter（name ・ description、あれば compatibility）と、目的 ・ 役割 ・ 事前条件 ・ 入力と出力（形に schema を添える）・
//! 参照する資料（形に schema を添える）・ 道具（scripts/）・ 外の道具 ・ 資源（assets/。形に schema を添える）・
//! 手順（読む資料と完了条件と例外）・
//! 守ること ・ 最低保証 ・ 完了条件 ・ 例の節。項目に無い節は書かない。項目を足さない（skill.json にあることだけを書く）。

use serde_json::Value;

/// SKILL.md のファイル名（Agent Skills の公開の仕様）。
pub const FILE: &str = "SKILL.md";

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// 値の文字列（数は数字のまま）。
fn plain(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// frontmatter の値。YAML で別の意味になる文字を含むなら、JSON の文字列（YAML の二重引用符の文字列）にする。
fn yaml(value: &str) -> String {
    let special = value.contains(": ")
        || value.contains(" #")
        || value.ends_with(':')
        || value.starts_with(|c: char| "-?:,[]{}#&*!|>'\"%@`".contains(c))
        || value.contains('\n');
    if special {
        Value::String(value.to_string()).to_string()
    } else {
        value.to_string()
    }
}

/// 語のあとに助詞を付ける。語が英数字で終わるなら、あいだに空白を入れる（「人か AI が」）。
fn particle(word: &str, p: &str) -> String {
    if word.ends_with(|c: char| c.is_ascii_alphanumeric()) {
        format!("{word} {p}")
    } else {
        format!("{word}{p}")
    }
}

/// 形に、従う schema を添える（「JSON（schema：references/criteria.schema.json）」）。schema が無ければ形のまま。
fn form_with_schema(v: &Value) -> String {
    let form = text(v, "form");
    match v.get("schema").and_then(Value::as_str) {
        Some(schema) if form.is_empty() => format!("（schema：{schema}）"),
        Some(schema) => format!("{}（schema：{schema}）", form.trim_end_matches('。')),
        None => form.to_string(),
    }
}

fn section(out: &mut String, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    out.push_str(&format!("\n## {title}\n\n"));
    for l in lines {
        out.push_str(l);
        out.push('\n');
    }
}

/// skill.json の値から SKILL.md の中身を作る。
#[must_use]
pub fn render(skill: &Value) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("name: {}\n", yaml(text(skill, "name"))));
    out.push_str(&format!(
        "description: {}\n",
        yaml(text(skill, "description"))
    ));
    if let Some(c) = skill.get("compatibility").and_then(Value::as_str) {
        out.push_str(&format!("compatibility: {}\n", yaml(c)));
    }
    out.push_str("---\n");

    let purpose = skill.get("purpose").cloned().unwrap_or_default();
    if purpose.is_object() {
        section(
            &mut out,
            "目的",
            &[format!(
                "{}、{}、{}。",
                particle(text(&purpose, "who"), "が"),
                particle(text(&purpose, "when"), "に"),
                particle(text(&purpose, "gets"), "を得る")
            )],
        );
    }

    let role = skill.get("role").cloned().unwrap_or_default();
    let mut lines: Vec<String> = items(&role, "does")
        .iter()
        .map(|d| format!("- 受け持つ：{}", plain(d)))
        .collect();
    lines.extend(
        items(&role, "does_not")
            .iter()
            .map(|d| format!("- 受け持たない：{}", text(d, "what"))),
    );
    section(&mut out, "役割", &lines);

    let lines: Vec<String> = items(skill, "preconditions")
        .iter()
        .map(|p| format!("- {}", plain(p)))
        .collect();
    section(&mut out, "事前条件", &lines);

    // Skill に従う AI が、受け取るものと返すものを読めるように
    let with_form = |v: &Value| {
        let form = form_with_schema(v);
        let more: Vec<&str> = [form.as_str(), text(v, "where")]
            .into_iter()
            .filter(|t| !t.is_empty())
            .collect();
        if more.is_empty() {
            text(v, "name").to_string()
        } else {
            format!("{}（{}）", text(v, "name"), more.join(" ・ "))
        }
    };
    let mut lines: Vec<String> = items(skill, "inputs")
        .iter()
        .map(|i| format!("- 入力：{}", with_form(i)))
        .collect();
    lines.extend(
        items(skill, "outputs")
            .iter()
            .map(|o| format!("- 出力：{}", with_form(o))),
    );
    section(&mut out, "入力と出力", &lines);

    // 参照する資料：Skill に従う AI が、どのステップでどの資料を読むかを読めるように
    let references = items(skill, "references");
    let steps_of = |r: &Value| -> Vec<String> { items(r, "read_by").iter().map(plain).collect() };
    let lines: Vec<String> = references
        .iter()
        .map(|r| {
            format!(
                "- {}（{}）：{}。形：{}。手順 {} で読む",
                text(r, "name"),
                text(r, "path"),
                text(r, "holds").trim_end_matches('。'),
                form_with_schema(r).trim_end_matches('。'),
                steps_of(r).join(" ・ ")
            )
        })
        .collect();
    section(&mut out, "参照する資料", &lines);

    // 自分の道具（scripts/）・ 外の道具 ・ 資源（assets/）
    let lines: Vec<String> = items(skill, "tools")
        .iter()
        .map(|t| {
            let mut l = format!(
                "- {}（{}）：{}",
                text(t, "name"),
                text(t, "path"),
                text(t, "does")
            );
            let ins: Vec<String> = items(t, "inputs").iter().map(plain).collect();
            let outs: Vec<String> = items(t, "outputs").iter().map(plain).collect();
            if !ins.is_empty() {
                l.push_str(&format!("。受け取る：{}", ins.join(" ・ ")));
            }
            if !outs.is_empty() {
                l.push_str(&format!("。返す：{}", outs.join(" ・ ")));
            }
            l
        })
        .collect();
    section(&mut out, "道具", &lines);
    let lines: Vec<String> = items(skill, "uses")
        .iter()
        .map(|u| {
            format!(
                "- {}（{}）：{}",
                text(u, "name"),
                text(u, "versions"),
                text(u, "for")
            )
        })
        .collect();
    section(&mut out, "外の道具", &lines);
    let lines: Vec<String> = items(skill, "assets")
        .iter()
        .map(|a| {
            let form = form_with_schema(a);
            if form.is_empty() {
                format!(
                    "- {}（{}）：{}",
                    text(a, "name"),
                    text(a, "path"),
                    text(a, "holds")
                )
            } else {
                format!(
                    "- {}（{}）：{}。形：{form}",
                    text(a, "name"),
                    text(a, "path"),
                    text(a, "holds")
                )
            }
        })
        .collect();
    section(&mut out, "資源", &lines);

    // 手順と、ステップ番号に結んだ例外（id の数字の部分がステップの番号）
    let edges = items(skill, "edge_cases");
    let mut lines = Vec::new();
    for s in items(skill, "steps") {
        let no = plain(s.get("id").unwrap_or(&Value::Null));
        // このステップが読む資料（references の read_by にこの番号があるもの）
        let read: Vec<&str> = references
            .iter()
            .filter(|r| steps_of(r).contains(&no))
            .map(|r| text(r, "name"))
            .collect();
        let reads = if read.is_empty() {
            String::new()
        } else {
            format!("（読む資料：{}）", read.join(" ・ "))
        };
        lines.push(format!(
            "{no}. {}{reads}（完了条件：{}）",
            text(s, "do"),
            text(s, "completion")
        ));
        for e in edges {
            let id = text(e, "id");
            if id.trim_end_matches(|c: char| c.is_ascii_lowercase()) == no {
                lines.push(format!(
                    "   - {id} {} → {}（{}）",
                    text(e, "when"),
                    text(e, "then"),
                    text(e, "end")
                ));
            }
        }
    }
    section(&mut out, "手順", &lines);

    let lines: Vec<String> = items(skill, "rules")
        .iter()
        .map(|r| format!("- {}（{}）", text(r, "rule"), text(r, "why")))
        .collect();
    section(&mut out, "守ること", &lines);

    let lines: Vec<String> = items(skill, "minimal_guarantees")
        .iter()
        .map(|g| format!("- {}", plain(g)))
        .collect();
    section(&mut out, "最低保証", &lines);

    let lines: Vec<String> = items(skill, "completion_criteria")
        .iter()
        .map(|c| {
            format!(
                "- {} {}（確かめ方：{}）",
                text(c, "id"),
                text(c, "criterion"),
                text(c, "check")
            )
        })
        .collect();
    section(&mut out, "完了条件", &lines);

    let lines: Vec<String> = items(skill, "examples")
        .iter()
        .map(|e| {
            format!(
                "- {}（{}）：{}。「{}」\n  → {}",
                text(e, "id"),
                text(e, "caller"),
                text(e, "given"),
                text(e, "input"),
                text(e, "expected")
            )
        })
        .collect();
    section(&mut out, "例", &lines);
    out
}

#[cfg(test)]
mod tests {
    use super::render;
    use serde_json::json;

    #[test]
    fn the_same_skill_gives_the_same_text_and_sections_without_items_are_left_out() {
        let skill = json!({"name": "a", "description": "使う: とき", "purpose": {"who": "人", "when": "書くとき", "gets": "分かった状態"},
                           "role": {"does": ["照合する"], "does_not": [{"what": "書き換える", "from": "作る人の答え"}]},
                           "steps": [{"id": 1, "do": "取る", "completion": "ある"}],
                           "edge_cases": [{"id": "1a", "when": "無い", "then": "止める", "end": "止める"}]});
        let a = render(&skill);
        assert_eq!(a, render(&skill));
        assert!(a.starts_with("---\nname: a\ndescription: \"使う: とき\"\n---\n"));
        assert!(a.contains("人が、書くときに、分かった状態を得る。"));
        assert!(a.contains("1. 取る（完了条件：ある）\n   - 1a 無い → 止める（止める）\n"));
        assert!(!a.contains("## 最低保証"));
        assert!(!a.contains("## 参照する資料"));
    }

    #[test]
    fn references_are_listed_and_named_on_the_steps_that_read_them() {
        let skill = json!({"name": "a", "description": "d",
                           "steps": [{"id": 1, "do": "見分ける", "completion": "ある"}, {"id": 2, "do": "読む", "completion": "読んだ"}],
                           "references": [{"name": "判断基準", "path": "references/criteria.json", "holds": "判断基準の並び。", "form": "JSON",
                                           "read_by": [2], "from": "作る人の答え"}]});
        let a = render(&skill);
        assert!(a.contains("## 参照する資料\n\n- 判断基準（references/criteria.json）：判断基準の並び。形：JSON。手順 2 で読む\n"));
        assert!(a.contains(
            "1. 見分ける（完了条件：ある）\n2. 読む（読む資料：判断基準）（完了条件：読んだ）\n"
        ));
    }
}
