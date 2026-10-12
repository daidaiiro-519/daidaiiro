//! ページを描画する。契約を通ったデータだけを、同じ入力から同じページへ描画する。
//! 部品の見た目と操作は、基盤の部品のカタログ（ポート Design）だけが持つ。
//! 基盤の文書（kind が document）は、決まったページテンプレートで描画する。何も指定が無いときに使う。
//! 具体のスキーマの種類は、具体のページテンプレートがカタログの部品を並べて描画する。どちらでもない種類があれば、何も書かずに失敗する。

use crate::application::checks::Checks;
use crate::application::context::contexts;
use crate::application::instances::{write_error, Instances, UseCaseError};
use crate::application::page::{document_page_template, page_errors, PageRenderer};
use crate::application::view::{Instance, ViewEngine};
use crate::domain::check::Doc;
use crate::domain::design_system;
use crate::domain::schema::Schema;
use crate::domain::values::{JsonValue, ValidationError};
use crate::ports::{Design, Files, Query, Schemas, WriteIf};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// 書いたページ1枚。
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedPage {
    pub path: String,
    /// 内容が変わったか（変わっていなければ書いていない）
    pub changed: bool,
    /// 使ったページテンプレート。具体のページテンプレートなら catalog、基盤の文書の決まったページテンプレートなら document
    pub design: String,
}

/// 描画の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    pub pages: Vec<RenderedPage>,
}

/// 描画のユースケース。部品のカタログを持つ。
#[derive(Clone)]
pub struct Renders<F, S, Q> {
    files: F,
    schemas: S,
    query: Q,
    catalog: Arc<dyn Design>,
    /// スキーマの名前を引く Skill のフォルダ
    skill: Option<String>,
}

impl<F: Files, S: Schemas, Q: Query> Renders<F, S, Q> {
    pub fn new(files: F, schemas: S, query: Q, catalog: Arc<dyn Design>) -> Self {
        Self {
            files,
            schemas,
            query,
            catalog,
            skill: None,
        }
    }

    /// スキーマの名前を引く Skill のフォルダを渡す。
    pub fn with_skill(mut self, dir: &str) -> Self {
        self.skill = Some(dir.to_owned());
        self
    }

    /// ページテンプレートのディレクトリを読み、kind → テンプレートにする。契約とカタログに違反があれば失敗する。
    fn templates(&self, dir: &str) -> Result<BTreeMap<String, Value>, UseCaseError> {
        let design = &self.catalog;
        if dir.is_empty() {
            return Ok(BTreeMap::new());
        }
        let mut reader =
            Instances::new(self.files.clone(), self.schemas.clone(), self.query.clone());
        if let Some(dir) = &self.skill {
            reader = reader.with_skill(dir);
        }
        let shape = Schema::new(
            "page.schema.json",
            serde_json::from_str(crate::embedded::schema_text("schema-driven/page@v1"))
                .unwrap_or(Value::Null),
            vec![],
        );
        let list = self
            .files
            .list(dir)
            .map_err(|error| UseCaseError::new("読めない", error.0))?;
        let (mut out, mut problems) = (BTreeMap::new(), Vec::new());
        for path in list.into_iter().filter(|path| path.ends_with(".json")) {
            let (_, value) = reader.read_json(&path)?;
            match shape.validate(&value) {
                Ok(validation) => {
                    problems.extend(
                        validation
                            .unfilled
                            .iter()
                            .map(|unfilled| format!("{path}: {} が無い", unfilled.property())),
                    );
                    problems.extend(
                        validation.errors.iter().map(|error| {
                            format!("{path}: {} {}", error.property(), error.reason())
                        }),
                    );
                }
                Err(error) => problems.push(format!("{path}: {}", error.0)),
            }
            problems.extend(
                page_errors(&**design, &value)
                    .into_iter()
                    .map(|error| format!("{path}: {error}")),
            );
            let kind = value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            if out.insert(kind.clone(), value).is_some() {
                problems.push(format!("{path}: kind {kind} のページテンプレートが2つある"));
            }
        }
        if problems.is_empty() {
            Ok(out)
        } else {
            Err(UseCaseError::new(
                "ページテンプレートが契約を通過しない",
                problems.join("\n"),
            ))
        }
    }
}

impl<F: Files, S: Schemas, Q: Query> Renders<F, S, Q> {
    /// ページを描画する。dir のインスタンスを、pages のページテンプレートで、out へ描画する
    pub fn render(&self, dir: &str, pages: &str, out: &str) -> Result<Rendered, UseCaseError> {
        // カタログはデザインシステムに従う。満たさなければ1ページも書かない
        let names: Vec<String> = self
            .catalog
            .components()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        let findings: Vec<String> =
            design_system::check(self.catalog.design_system(), self.catalog.parts(), &names)
                .into_iter()
                .map(|finding| format!("部品のカタログ：{finding}"))
                .collect();
        if !findings.is_empty() {
            return Err(UseCaseError::new(
                "デザインシステムが契約を満たさない",
                findings.join("\n"),
            ));
        }
        let mut checks = Checks::new(self.files.clone(), self.schemas.clone(), self.query.clone());
        if let Some(dir) = &self.skill {
            checks = checks.with_skill(dir);
        }
        let (loaded, schemas) = checks.load_dir(dir)?;
        // 契約を通ったデータだけを描画する（未記入は止めない）
        let mut errors = Vec::new();
        for instance in &loaded {
            let validation = schemas[&instance.schema]
                .validate(&instance.value)
                .map_err(|error| UseCaseError::new("スキーマが壊れている", error.0))?;
            for error in &validation.errors {
                if let Ok(validation_error) = ValidationError::new(
                    &format!("{}#{}", instance.path, error.property()),
                    error.reason(),
                ) {
                    errors.push(validation_error);
                }
            }
        }
        if !errors.is_empty() {
            let mut error = UseCaseError::new(
                "検証を通過しないインスタンスがある",
                format!("{} 件", errors.len()),
            );
            error.errors = errors;
            return Err(error);
        }
        let templates = self.templates(pages)?;
        let ctx = {
            let docs: Vec<Doc> = loaded
                .iter()
                .map(|instance| Doc {
                    path: instance.path.clone(),
                    value: instance.value.clone(),
                    hash: JsonValue::new(&instance.text).hash(),
                    schema: &schemas[&instance.schema],
                })
                .collect();
            contexts(&docs)
        };
        let list = loaded
            .iter()
            .map(|instance| Instance {
                value: instance.value.clone(),
                schema: instance.schema.clone(),
            })
            .collect();
        let engine = ViewEngine::new(self.query.clone(), schemas.into_iter().collect(), list);
        let renderer = PageRenderer::new(engine, self.catalog.clone())
            .map_err(|error| UseCaseError::new("部品のカタログを読めない", error.0))?;
        let document_template = document_page_template();
        // すべて生成してから書く（途中で失敗したときに、一部だけ書いた状態を残さない）
        let (mut generated, mut missing) = (Vec::new(), Vec::new());
        for (doc_index, instance) in loaded.iter().enumerate() {
            let kind = instance
                .value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("");
            let (html, design) = if kind == "document" {
                (
                    renderer.render(&document_template, &ctx[doc_index], &instance.schema),
                    "document",
                )
            } else if let Some(page_template) = templates.get(kind) {
                (
                    renderer.render(page_template, &ctx[doc_index], &instance.schema),
                    "catalog",
                )
            } else {
                missing.push(format!("{}（kind {kind}）", instance.path));
                continue;
            };
            let html = html.map_err(|error| {
                UseCaseError::new("描画できない", format!("{}: {}", instance.path, error.0))
            })?;
            let name = instance.path.rsplit('/').next().unwrap_or(&instance.path);
            let stem = name.strip_suffix(".json").unwrap_or(name);
            generated.push((
                format!("{}/{stem}.html", out.trim_end_matches('/')),
                html,
                design,
            ));
        }
        if !missing.is_empty() {
            return Err(UseCaseError::new(
                "ページテンプレートが無い種類がある",
                format!(
                    "次のインスタンスは、基盤の文書でもなく、その種類のページテンプレートも無い：{}",
                    missing.join("、")
                ),
            ));
        }
        let mut written = Vec::new();
        for (path, html, design) in generated {
            let changed = match self.files.read(&path) {
                Ok(current) if current == html.as_str() => false,
                Ok(current) => {
                    self.files
                        .write(
                            &path,
                            html.as_str(),
                            WriteIf::Unchanged(JsonValue::new(&current).hash()),
                        )
                        .map_err(write_error)?;
                    true
                }
                Err(_) => {
                    self.files
                        .write(&path, html.as_str(), WriteIf::Absent)
                        .map_err(write_error)?;
                    true
                }
            };
            written.push(RenderedPage {
                path,
                changed,
                design: design.to_owned(),
            });
        }
        Ok(Rendered { pages: written })
    }
}

#[cfg(test)]
mod tests {
    //! 描画のユースケースのテスト。部品のカタログは、デザインシステムに従う。
    //! 契約を満たさないカタログでは、1ページも書かない。

    use super::Renders;
    use crate::tests::{at, Answer, MemoryFiles, TableQuery, TestDesign};
    use serde_json::{json, Value};
    use std::sync::Arc;

    fn parts(pill_style: &str) -> String {
        format!(
            r#"<template id="page"><style>{{{{tokens}}}}</style><main><h1>{{{{title}}}}</h1>{{{{badges}}}}{{{{lead}}}}{{{{sections}}}}</main></template>
<template id="section"><section><h2>{{{{heading}}}}</h2>{{{{body}}}}</section></template>
<template id="pill"><b class="{{{{tone}}}}" style="{pill_style}">{{{{text}}}}</b></template>"#
        )
    }

    fn system() -> Value {
        json!({
            "scope": ":root",
            "tokens": {
                "base": {"color": {"teal-700": "#0d5c55", "teal-300": "#6cc9b8", "paper-100": "#eef1ef", "night-900": "#0e1614"}},
                "semantic": {"light": {"bg": "paper-100", "key": "teal-700"}, "dark": {"bg": "night-900", "key": "teal-300"}},
                "component": {"pill-bg": "key"}
            },
            "components": [{"name": "pill", "states": {"default": {"background": "pill-bg", "text": "bg"}}},
                           {"name": "table", "states": {"default": {"text": "bg"}}}]
        })
    }

    fn answers() -> Vec<(&'static str, Answer)> {
        vec![
            ("this.tags", |value, _| Ok(at(value, "/this/tags"))),
            ("@", |value, _| Ok(value.clone())),
        ]
    }

    /// Skill example のスキーマ ・ インスタンス T-1 ・ ページテンプレートを置いたメモリ上のファイル。
    fn files() -> MemoryFiles {
        MemoryFiles::with(&[
            (
                "skills/example/assets/thing.schema.json",
                json!({"$id": "example/thing@v1", "type": "object", "required": ["kind", "id"],
                       "properties": {"$schema": {}, "kind": {"type": "string"}, "id": {"type": "string"}, "tags": {"type": "array", "items": {"type": "string"}}}}),
            ),
            (
                "declarations/T-1.json",
                json!({"$schema": "example/thing@v1", "kind": "thing", "id": "T-1", "tags": ["a"]}),
            ),
            (
                "pages/thing.json",
                json!({"kind": "thing", "sections": [{"heading": "タグ", "body": [{"component": "pill", "each": "this.tags", "value": "@"}]}]}),
            ),
        ])
    }

    fn renders(
        files: &MemoryFiles,
        pill_style: &str,
    ) -> Renders<MemoryFiles, MemoryFiles, TableQuery> {
        let design = TestDesign {
            parts: parts(pill_style),
            system: system().to_string(),
        };
        Renders::new(
            files.clone(),
            files.clone(),
            TableQuery::new(answers()),
            Arc::new(design),
        )
        .with_skill("skills/example")
    }

    #[test]
    fn a_catalog_that_follows_its_design_system_writes_the_page() {
        let files = files();
        let done = renders(&files, "background:var(--pill-bg)")
            .render("declarations", "pages", "out")
            .unwrap();
        assert_eq!(done.pages.len(), 1);
        assert_eq!(done.pages[0].path, "out/T-1.html");
        assert_eq!(
            files.get("out/T-1.html").unwrap(),
            "<style>:root{--night-900:#0e1614;--paper-100:#eef1ef;--teal-300:#6cc9b8;--teal-700:#0d5c55;--bg:#eef1ef;--key:#0d5c55;--pill-bg:var(--key)}\n@media (prefers-color-scheme: dark){:root:not([data-theme=\"light\"]){--bg:#0e1614;--key:#6cc9b8;color-scheme:dark}}\n:root[data-theme=\"dark\"]{--bg:#0e1614;--key:#6cc9b8;color-scheme:dark}</style><main><h1>T-1</h1><section><h2>タグ</h2><b class=\"plain\" style=\"background:var(--pill-bg)\">a</b></section></main>"
        );
    }

    #[test]
    fn a_catalog_that_breaks_its_design_system_writes_no_page() {
        let files = files();
        let before = files.paths();
        let error = renders(&files, "background:#ff0000")
            .render("declarations", "pages", "out")
            .unwrap_err();
        assert_eq!(error.reason, "デザインシステムが契約を満たさない");
        assert!(error.detail.contains("色の直値"), "{}", error.detail);
        assert_eq!(files.paths(), before, "1ページも書かない");
    }
}
