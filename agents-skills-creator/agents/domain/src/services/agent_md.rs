//! Agent の項目（agent.json）から、AI ツールが読む Agent の定義を決まった形で作る。同じ項目からは同じ文字列になる。
//!
//! - Claude Code の subagent（AGENT.md）：frontmatter に name ・ description ・ skills（配下の Skill。読み込ませる）・
//!   tools（Skill を含めない。配下のほかの Skill を呼ばせない）。本文は目的 ・ 役割 ・ 手順 ・ 分かれ道 ・ まとめ方の決まり ・
//!   最低保証 ・ 完了条件
//! - Codex の custom agent（codex.toml）：name ・ description ・ developer_instructions（AGENT.md の本文と同じ）。Codex は
//!   Skill を配下に絞れない（親から引き継ぐ）ので、本文で配下の Skill だけを呼ぶよう書く
//!
//! 項目を足さない（agent.json にあることだけを書く）。どちらにも、どの Agent のフォルダから作ったかの印を入れる
//! （AGENT.md は frontmatter の外の末尾の HTML のコメント、codex.toml は先頭のコメント行）。配布先の写しが、どの Agent の
//! ものかを、中身が古くても見分けるため。

use serde_json::Value;

/// Claude Code の subagent のファイル名（Agent のフォルダの中）。
pub const FILE: &str = "AGENT.md";

/// Codex の custom agent のファイル名（Agent のフォルダの中）。
pub const CODEX_FILE: &str = "codex.toml";

/// 印の頭。
const MARK: &str = "agents-skills-creator: ";

/// どの Agent のフォルダから作ったかの印（AGENT.md の末尾の行）。
#[must_use]
pub fn mark_md(folder: &str) -> String {
    format!("<!-- {MARK}{folder} -->")
}

/// どの Agent のフォルダから作ったかの印（codex.toml の先頭の行）。
#[must_use]
pub fn mark_toml(folder: &str) -> String {
    format!("# {MARK}{folder}")
}

/// ファイルの中身から、どの Agent のフォルダから作ったかの印を読む（AGENT.md は最後の行、codex.toml は最初の行）。
/// 印が無ければ None。
#[must_use]
pub fn read_mark(text: &str) -> Option<String> {
    let last = text.lines().rev().find(|l| !l.trim().is_empty())?;
    if let Some(rest) = last
        .trim()
        .strip_prefix("<!-- ")
        .and_then(|r| r.strip_suffix(" -->"))
    {
        return rest.strip_prefix(MARK).map(ToString::to_string);
    }
    let first = text.lines().next()?;
    first
        .trim()
        .strip_prefix("# ")
        .and_then(|r| r.strip_prefix(MARK))
        .map(ToString::to_string)
}

/// subagent に使わせるツール。Skill を含めないので、配下に読み込ませた Skill のほかは呼ばない。
pub const TOOLS: [&str; 4] = ["Read", "Grep", "Glob", "Bash"];

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn strings(v: &Value, key: &str) -> Vec<String> {
    items(v, key)
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

/// frontmatter の値。YAML で別の意味になる文字を含むなら、JSON の文字列にする。
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

/// 本文（AGENT.md の frontmatter のあと ・ Codex の developer_instructions）。
#[must_use]
pub fn body(agent: &Value) -> String {
    let mut out = String::new();
    let purpose = agent.get("purpose").cloned().unwrap_or_default();
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
    let role = agent.get("role").cloned().unwrap_or_default();
    let mut lines: Vec<String> = strings(&role, "does")
        .iter()
        .map(|d| format!("- 受け持つ：{d}"))
        .collect();
    lines.extend(
        items(&role, "does_not")
            .iter()
            .map(|d| format!("- 受け持たない：{}", text(d, "what"))),
    );
    section(&mut out, "役割", &lines);
    let skills = strings(agent, "skills");
    section(
        &mut out,
        "配下の Skill",
        &[format!(
            "呼んでよいのは、次の Skill だけである：{}。ほかの Skill は呼ばない。",
            skills.join(" ・ ")
        )],
    );
    let lines: Vec<String> = items(agent, "steps")
        .iter()
        .map(|s| {
            let id = s.get("id").map(Value::to_string).unwrap_or_default();
            let mut line = format!("{id}. {}", text(s, "do"));
            if text(s, "kind") == "call" {
                line.push_str(&format!(
                    "（同時に呼ぶ：{}",
                    strings(s, "calls").join(" ・ ")
                ));
                if let Some(c) = s.get("choose").and_then(Value::as_str) {
                    line.push_str(&format!("。選び方：{c}"));
                }
                let passes = strings(s, "passes");
                if !passes.is_empty() {
                    line.push_str(&format!("。渡すもの：{}", passes.join(" ・ ")));
                }
                line.push('）');
            } else {
                line.push_str("（人が承認する。承認を受けるまで次へ進まない）");
            }
            line.push_str(&format!("。完了：{}", text(s, "completion")));
            line
        })
        .collect();
    section(&mut out, "手順", &lines);
    let lines: Vec<String> = items(agent, "edge_cases")
        .iter()
        .map(|e| {
            format!(
                "- {} {}：{}（{}）",
                text(e, "id"),
                text(e, "when"),
                text(e, "then"),
                text(e, "end")
            )
        })
        .collect();
    section(&mut out, "分かれ道", &lines);
    let lines: Vec<String> = items(agent, "rules")
        .iter()
        .map(|r| format!("- {}（{}）", text(r, "rule"), text(r, "why")))
        .collect();
    section(&mut out, "まとめ方と守ること", &lines);
    let lines: Vec<String> = items(agent, "minimal_guarantees")
        .iter()
        .filter_map(Value::as_str)
        .map(|g| format!("- {g}"))
        .collect();
    section(&mut out, "最低保証", &lines);
    let lines: Vec<String> = items(agent, "completion_criteria")
        .iter()
        .map(|c| {
            format!(
                "- {}（確かめ方：{}）",
                text(c, "criterion"),
                text(c, "check")
            )
        })
        .collect();
    section(&mut out, "完了条件", &lines);
    out
}

/// agent.json の値から AGENT.md（Claude Code の subagent）の中身を作る。folder は Agent のフォルダ（印に入れる）。
#[must_use]
pub fn render(agent: &Value, folder: &str) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("name: {}\n", yaml(text(agent, "name"))));
    out.push_str(&format!(
        "description: {}\n",
        yaml(text(agent, "description"))
    ));
    out.push_str("skills:\n");
    for s in strings(agent, "skills") {
        out.push_str(&format!("  - {}\n", yaml(&s)));
    }
    out.push_str(&format!("tools: {}\n", TOOLS.join(", ")));
    out.push_str("---\n");
    out.push_str(&body(agent));
    out.push('\n');
    out.push_str(&mark_md(folder));
    out.push('\n');
    out
}

/// TOML の基本の文字列（二重引用符）。
fn toml_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// agent.json の値から Codex の custom agent（TOML）の中身を作る。folder は Agent のフォルダ（印に入れる）。
#[must_use]
pub fn render_codex(agent: &Value, folder: &str) -> String {
    format!(
        "{}\nname = {}\ndescription = {}\ndeveloper_instructions = {}\n",
        mark_toml(folder),
        toml_string(text(agent, "name")),
        toml_string(text(agent, "description")),
        toml_string(body(agent).trim_start())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_subagent_preloads_the_skills_and_has_no_skill_tool() {
        let a = json!({"name": "a", "description": "d: x", "skills": ["s1", "s2"],
            "steps": [{"id": 1, "kind": "call", "do": "聞く", "calls": ["s1", "s2"], "passes": ["見解"], "completion": "そろう"},
                      {"id": 2, "kind": "approve", "do": "承認する", "completion": "承認がある"}]});
        let md = render(&a, "agents/a");
        assert!(md.starts_with("---\nname: a\ndescription: \"d: x\"\nskills:\n  - s1\n  - s2\ntools: Read, Grep, Glob, Bash\n---\n"));
        assert!(!md.contains("tools: Skill") && !TOOLS.contains(&"Skill"));
        assert!(md.contains("1. 聞く（同時に呼ぶ：s1 ・ s2。渡すもの：見解）"));
        assert_eq!(render(&a, "agents/a"), md);
        assert_eq!(read_mark(&md).as_deref(), Some("agents/a"));
        let toml = render_codex(&a, "agents/a");
        assert_eq!(read_mark(&toml).as_deref(), Some("agents/a"));
        assert_eq!(read_mark("name = \"x\"\n"), None);
        assert!(
            toml.starts_with(
                "name = \"a\"\ndescription = \"d: x\"\ndeveloper_instructions = \"## 目的"
            ) || toml.contains("developer_instructions = \"## 配下の Skill")
        );
    }
}
