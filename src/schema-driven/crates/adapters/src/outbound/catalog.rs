//! 部品のカタログを、Skill のフォルダから読む（ポート Design の実装）。
//! カタログの中身は concrete（Skill）が持ち、基盤は中身を取り込まない。描画のたびに、`--skill` で渡された
//! Skill のフォルダの決まった場所（assets/catalog/）から読む。

use schema_driven_core::ports::{Catalog, Design, ReadError};
use std::fs;
use std::path::Path;

/// Skill のフォルダの中で、カタログを置く場所。
const CATALOG_DIR: &str = "assets/catalog";
/// 部品の一覧のスキーマ（schema-driven/catalog@v1 の枠に従う）。
const SCHEMA: &str = "catalog.schema.json";
/// 部品の HTML。
const PARTS: &str = "parts.html";
/// 部品の見た目。無くてもよい。
const STYLE: &str = "catalog.css";
/// 部品の操作。無くてもよい。
const SCRIPT: &str = "catalog.js";
/// デザインシステム（schema-driven/design-system@v1 に従う）。
const DESIGN_SYSTEM: &str = "design-system.json";

/// `--skill` で渡された Skill のフォルダの assets/catalog/ から読むカタログ。
pub struct SkillCatalog {
    skill: Option<String>,
}

impl SkillCatalog {
    pub fn new(skill: Option<&str>) -> Self {
        Self {
            skill: skill.map(|dir| dir.trim_end_matches(['/', '\\']).to_owned()),
        }
    }
}

fn read(dir: &Path, file: &str) -> Result<String, ReadError> {
    let path = dir.join(file);
    fs::read_to_string(&path).map_err(|error| ReadError(format!("{}: {error}", path.display())))
}

/// 無くてもよいファイル。無ければ空。
fn read_optional(dir: &Path, file: &str) -> Result<String, ReadError> {
    if dir.join(file).exists() {
        read(dir, file)
    } else {
        Ok(String::new())
    }
}

impl Design for SkillCatalog {
    fn catalog(&self) -> Result<Catalog, ReadError> {
        let Some(skill) = &self.skill else {
            return Err(ReadError(format!(
                "--skill が無い。カタログは、--skill に渡した Skill のフォルダの {CATALOG_DIR}/ から読む"
            )));
        };
        let dir = Path::new(skill).join(CATALOG_DIR);
        if !dir.is_dir() {
            return Err(ReadError(format!(
                "{} が無い。Skill のフォルダの {CATALOG_DIR}/ に {SCHEMA} ・ {PARTS} ・ {DESIGN_SYSTEM}（と {STYLE} ・ {SCRIPT}）を置く",
                dir.display()
            )));
        }
        Ok(Catalog {
            schema: read(&dir, SCHEMA)?,
            parts: read(&dir, PARTS)?,
            style: read_optional(&dir, STYLE)?,
            script: read_optional(&dir, SCRIPT)?,
            design_system: read(&dir, DESIGN_SYSTEM)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SkillCatalog;
    use crate::tests::Dir;
    use schema_driven_core::ports::Design;

    #[test]
    fn the_catalog_is_read_from_the_skill_folder() {
        let dir = Dir::new("catalog-read");
        dir.put(&[
            ("example/assets/catalog/catalog.schema.json", "{}"),
            (
                "example/assets/catalog/parts.html",
                "<template id=\"page\"></template>",
            ),
            ("example/assets/catalog/catalog.css", "b{}"),
            (
                "example/assets/catalog/design-system.json",
                "{\"scope\": \":root\"}",
            ),
        ]);
        let catalog = SkillCatalog::new(Some(&dir.at("example")))
            .catalog()
            .unwrap();
        assert_eq!(catalog.schema, "{}");
        assert_eq!(catalog.parts, "<template id=\"page\"></template>");
        assert_eq!(catalog.style, "b{}");
        assert_eq!(catalog.script, "", "無いファイルは空");
        assert_eq!(catalog.design_system, "{\"scope\": \":root\"}");
    }

    #[test]
    fn without_a_skill_or_its_catalog_the_reason_is_returned() {
        let dir = Dir::new("catalog-missing");
        dir.put(&[("example/assets/thing.schema.json", "{}")]);
        let none = SkillCatalog::new(None).catalog().unwrap_err();
        assert!(none.0.starts_with("--skill が無い"), "{none}");
        let missing = SkillCatalog::new(Some(&dir.at("example")))
            .catalog()
            .unwrap_err();
        assert!(missing.0.contains("assets/catalog が無い"), "{missing}");
        dir.put(&[("example/assets/catalog/parts.html", "")]);
        let partial = SkillCatalog::new(Some(&dir.at("example")))
            .catalog()
            .unwrap_err();
        assert!(partial.0.contains("catalog.schema.json"), "{partial}");
    }
}
