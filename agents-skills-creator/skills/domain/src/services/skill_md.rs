//! Skill の項目（skill.json）から、AI ツールが読む SKILL.md を決まった形で作る。同じ項目からは同じ文字列になる。
//!
//! 形：frontmatter（name ・ description、あれば compatibility）と、目的 ・ 役割 ・ 事前条件 ・ 入力と出力 ・ 手順（完了条件と例外）・
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
        let more: Vec<&str> = ["form", "where"]
            .into_iter()
            .map(|k| text(v, k))
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

    // 手順と、ステップ番号に結んだ例外（id の数字の部分がステップの番号）
    let edges = items(skill, "edge_cases");
    let mut lines = Vec::new();
    for s in items(skill, "steps") {
        let no = plain(s.get("id").unwrap_or(&Value::Null));
        lines.push(format!(
            "{no}. {}（完了条件：{}）",
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
                           "role": {"does": ["照合する"], "does_not": [{"what": "書き換える", "from": "問3 の答え"}]},
                           "steps": [{"id": 1, "do": "取る", "completion": "ある"}],
                           "edge_cases": [{"id": "1a", "when": "無い", "then": "止める", "end": "止める"}]});
        let a = render(&skill);
        assert_eq!(a, render(&skill));
        assert!(a.starts_with("---\nname: a\ndescription: \"使う: とき\"\n---\n"));
        assert!(a.contains("人が、書くときに、分かった状態を得る。"));
        assert!(a.contains("1. 取る（完了条件：ある）\n   - 1a 無い → 止める（止める）\n"));
        assert!(!a.contains("## 最低保証"));
    }
}
