//! scaffold：新しい Skill のフォルダに、SKILL.md の雛形 ・ tool.json ・ スキーマの雛形を置く。
//! 雛形は、skills-creator の Skill の assets/templates/ が持つ。ここには雛形の文字列を持たない。

use regex::Regex;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub struct Request<'a> {
    pub dir: &'a Path,
    pub kind: &'a str,
    pub pages: bool,
    pub tools: bool,
    pub refs: bool,
}

/// 置いたファイルを返す。置けないときは理由を返す。
pub fn scaffold(own_skill: &Path, request: &Request) -> Result<Value, String> {
    let name = request
        .dir
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name_rule = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
    if name.len() > 64 || !name_rule.is_match(&name) {
        return Err(format!(
            "フォルダの名前 {name} は Skill の name にできない。64文字までの a-z ・ 0-9 ・ - で、- で始めず終えず、-- を続けない"
        ));
    }
    if !Regex::new(r"^[a-z][a-z0-9-]*$")
        .unwrap()
        .is_match(request.kind)
    {
        return Err(format!(
            "種類 {} は a-z で始まる a-z ・ 0-9 ・ - で書く",
            request.kind
        ));
    }
    if request.dir.exists() {
        return Err(format!(
            "{} は既にある。上書きしない",
            request.dir.display()
        ));
    }
    let templates = own_skill.join("assets/templates");
    let fill = |file: &str| -> Result<String, String> {
        fs::read_to_string(templates.join(file))
            .map(|t| {
                t.replace("{{name}}", &name)
                    .replace("{{kind}}", request.kind)
            })
            .map_err(|error| format!("雛形 {file} を読めない：{error}"))
    };
    let mut files = vec![
        ("SKILL.md".to_owned(), fill("SKILL.md.tmpl")?),
        ("tool.json".to_owned(), fill("tool.json.tmpl")?),
        (
            format!("assets/{}.schema.json", request.kind),
            fill("schema.json.tmpl")?,
        ),
    ];
    if request.pages {
        files.push((
            format!("assets/pages/{}.json", request.kind),
            fill("page.json.tmpl")?,
        ));
    }
    if request.tools {
        files.push(("tools.json".to_owned(), fill("tools.json.tmpl")?));
    }
    if request.refs {
        files.push((
            "references/REFERENCE.md".to_owned(),
            fill("REFERENCE.md.tmpl")?,
        ));
    }
    for (path, text) in &files {
        let target = request.dir.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("{} を作れない：{error}", parent.display()))?;
        }
        fs::write(&target, text)
            .map_err(|error| format!("{} を書けない：{error}", target.display()))?;
    }
    Ok(json!({
        "ok": true,
        "skill": request.dir.to_string_lossy(),
        "wrote": files.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(),
        "next": "{{…}} を全部書き換え、check で skill.placeholders が合格するまで直す",
    }))
}
