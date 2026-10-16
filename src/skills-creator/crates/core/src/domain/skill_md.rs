//! SKILL.md を読み取り、型 skills-creator/skill@v1 のインスタンスにする。
//! 判定はしない。判定は基盤のエンジンが型のスキーマで行う（application の check）。

use regex::Regex;
use serde_json::{json, Map, Value};
use yaml_rust2::{Yaml, YamlLoader};

pub const SECTIONS: [&str; 6] = [
    "目的",
    "役割",
    "入力の想定",
    "実行手順",
    "出力形式",
    "ガードレール",
];

/// Skill の名前が Agent Skills の仕様の決まり（64文字までの a-z ・ 0-9 ・ -、- で始めず終えず続けない）に合うか。
pub fn valid_name(name: &str) -> bool {
    name.len() <= 64
        && Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$")
            .unwrap()
            .is_match(name)
}

/// 読み取った結果
pub struct Read {
    /// 型のインスタンス
    pub instance: Value,
    /// 先頭の欄が読めなかった理由。読めたら None
    pub frontmatter_error: Option<String>,
}

/// SKILL.md の本文と、フォルダの名前と、ファイルが在るかを答える関数から読み取る。
pub fn read(text: &str, folder: &str, exists: &dyn Fn(&str) -> bool) -> Read {
    let lines: Vec<&str> = text.lines().collect();
    let (frontmatter, body_start, frontmatter_error) = match frontmatter(&lines) {
        Ok((value, start)) => (value, start, None),
        Err((error, start)) => (Map::new(), start, Some(error)),
    };
    let name_is_folder = frontmatter.get("name").and_then(Value::as_str) == Some(folder);

    let placeholder = Regex::new(r"\{\{[^}]*\}\}").unwrap();
    let tool_path = Regex::new(
        r#"\.(claude|agents|github|cursor|gemini|codex|opencode|goose|windsurf)/skills/|\$\{[A-Za-z_][A-Za-z0-9_]*|(^|[\s`(\["'=])(/(home|Users|usr|opt|tmp|etc|var|root)/|[A-Za-z]:\\)"#,
    )
    .unwrap();
    let link = Regex::new(r"\]\(([^)\s]+)\)").unwrap();

    let mut sections = Vec::new();
    let mut placeholders = Vec::new();
    let mut tool_paths = Vec::new();
    let mut broken_links = Vec::new();
    let mut in_fence = false;
    for (index, line) in lines.iter().enumerate() {
        let number = index + 1;
        if let Some(found) = placeholder.find(line) {
            placeholders.push(hit(number, found.as_str()));
        }
        if index < body_start {
            continue;
        }
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if let Some(found) = tool_path.find(line) {
            tool_paths.push(hit(number, found.as_str().trim_start()));
        }
        if in_fence {
            continue;
        }
        if let Some(heading) = line.strip_prefix("## ") {
            let heading = heading.trim();
            if SECTIONS.contains(&heading) {
                sections.push(Value::from(heading));
            }
        }
        for captured in link.captures_iter(line) {
            let target = captured[1].split('#').next().unwrap_or("");
            if target.is_empty() || target.contains("://") || target.starts_with("mailto:") {
                continue;
            }
            if !exists(target) {
                broken_links.push(hit(number, target));
            }
        }
    }

    Read {
        instance: json!({
            "$schema": "skills-creator/skill@v1",
            "frontmatter": frontmatter,
            "folder": folder,
            "name_is_folder": name_is_folder,
            "lines": lines.len(),
            "sections": sections,
            "placeholders": placeholders,
            "tool_paths": tool_paths,
            "broken_links": broken_links,
        }),
        frontmatter_error,
    }
}

fn hit(line: usize, text: &str) -> Value {
    json!({"line": line, "text": text})
}

/// 先頭の --- で囲んだ YAML を、JSON の対応表にする。本文の始まりの行の位置も返す。
fn frontmatter(lines: &[&str]) -> Result<(Map<String, Value>, usize), (String, usize)> {
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return Err(("1行目が --- でない。先頭の欄が無い".into(), 0));
    }
    let Some(end) = lines
        .iter()
        .skip(1)
        .position(|l| l.trim_end() == "---")
        .map(|p| p + 1)
    else {
        return Err(("先頭の欄を閉じる --- が無い".into(), 0));
    };
    let source = lines[1..end].join("\n");
    let documents = YamlLoader::load_from_str(&source)
        .map_err(|error| (format!("YAML として読めない：{error}"), end + 1))?;
    match documents.into_iter().next() {
        Some(Yaml::Hash(hash)) => {
            let mut map = Map::new();
            for (key, value) in hash {
                let Yaml::String(key) = key else {
                    return Err(("先頭の欄のキーが文字列でない".into(), end + 1));
                };
                map.insert(key, to_json(value));
            }
            Ok((map, end + 1))
        }
        _ => Err(("先頭の欄が、キーと値の対応表でない".into(), end + 1)),
    }
}

fn to_json(value: Yaml) -> Value {
    match value {
        Yaml::String(s) => Value::from(s),
        Yaml::Integer(i) => Value::from(i),
        Yaml::Real(r) => r.parse::<f64>().map(Value::from).unwrap_or(Value::from(r)),
        Yaml::Boolean(b) => Value::from(b),
        Yaml::Array(items) => Value::Array(items.into_iter().map(to_json).collect()),
        Yaml::Hash(hash) => Value::Object(
            hash.into_iter()
                .map(|(k, v)| {
                    (
                        k.as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("{k:?}")),
                        to_json(v),
                    )
                })
                .collect(),
        ),
        Yaml::Null | Yaml::Alias(_) | Yaml::BadValue => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_text(text: &str) -> Read {
        read(text, "glossary-keeper", &|_| true)
    }

    #[test]
    fn a_quoted_name_is_read_as_the_name() {
        let r = read_text("---\nname: \"glossary-keeper\"\ndescription: x\n---\n");
        assert_eq!(r.instance["frontmatter"]["name"], "glossary-keeper");
        assert_eq!(r.instance["name_is_folder"], true);
    }

    #[test]
    fn sections_inside_a_code_fence_are_not_sections() {
        let r = read_text("---\nname: a\ndescription: x\n---\n```\n## 目的\n```\n## 役割\n");
        assert_eq!(r.instance["sections"], json!(["役割"]));
    }

    #[test]
    fn placeholders_are_found_with_their_line() {
        let r = read_text("---\nname: a\ndescription: x\n---\n## 目的\n{{目的を書く}}\n");
        assert_eq!(
            r.instance["placeholders"],
            json!([{"line": 6, "text": "{{目的を書く}}"}])
        );
    }

    #[test]
    fn a_missing_frontmatter_is_reported() {
        let r = read_text("# 題\n");
        assert_eq!(
            r.frontmatter_error.as_deref(),
            Some("1行目が --- でない。先頭の欄が無い")
        );
    }

    #[test]
    fn a_link_to_a_missing_file_is_broken() {
        let r = read(
            "---\nname: a\ndescription: x\n---\n[手引き](references/a.md)\n",
            "a",
            &|_| false,
        );
        assert_eq!(
            r.instance["broken_links"],
            json!([{"line": 5, "text": "references/a.md"}])
        );
    }

    #[test]
    fn names_follow_the_agent_skills_rules() {
        let names = [
            "glossary-keeper",
            "a",
            "-a",
            "a-",
            "a--b",
            "A",
            &"g".repeat(64),
            &"g".repeat(65),
        ];
        let got: Vec<bool> = names.iter().map(|name| valid_name(name)).collect();
        assert_eq!(
            got,
            vec![true, true, false, false, false, false, true, false]
        );
    }

    #[test]
    fn lines_are_counted_without_the_last_newline() {
        let r = read_text("---\nname: a\ndescription: x\n---\n");
        assert_eq!(r.instance["lines"], 4);
    }
}
