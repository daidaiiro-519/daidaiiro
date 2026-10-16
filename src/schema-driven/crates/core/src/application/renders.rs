//! ページを描画する。契約を通ったデータだけを、同じ入力から同じページへ描画する。
//! 部品の見た目と操作は、concrete が渡す部品のカタログ（ポート Design）だけが持ち、基盤はカタログの中身を持たない。
//! インスタンスの種類ごとに、その種類のページテンプレートがカタログの部品を並べて描画する。
//! ページテンプレートの指定が無い種類があれば、何も書かずに「指定が無い」と返す。

use crate::application::checks::Checks;
use crate::application::context::contexts;
use crate::application::instances::{write_error, Instances, UseCaseError};
use crate::application::page::{page_errors, PageRenderer, Parts};
use crate::application::view::{Instance, ViewEngine};
use crate::domain::catalog;
use crate::domain::check::Doc;
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
}

/// 描画の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    pub pages: Vec<RenderedPage>,
}

/// 描画のユースケース。部品のカタログの供給元を持つ。
#[derive(Clone)]
pub struct Renders<F, S, Q> {
    files: F,
    schemas: S,
    query: Q,
    design: Arc<dyn Design>,
    /// スキーマの名前を引く Skill のフォルダ
    skill: Option<String>,
}

impl<F: Files, S: Schemas, Q: Query> Renders<F, S, Q> {
    pub fn new(files: F, schemas: S, query: Q, design: Arc<dyn Design>) -> Self {
        Self {
            files,
            schemas,
            query,
            design,
            skill: None,
        }
    }

    /// スキーマの名前を引く Skill のフォルダを渡す。
    pub fn with_skill(mut self, dir: &str) -> Self {
        self.skill = Some(dir.to_owned());
        self
    }

    /// ページテンプレートのディレクトリを読み、kind → テンプレートにする。契約とカタログに違反があれば失敗する。
    fn templates(&self, dir: &str, parts: &Parts) -> Result<BTreeMap<String, Value>, UseCaseError> {
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
                page_errors(parts, &value)
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
        // カタログは concrete が渡す。無いか、枠とデザインシステムに従わなければ1ページも書かない
        let given = self
            .design
            .catalog()
            .map_err(|error| UseCaseError::new("部品のカタログを読めない", error.0))?;
        let findings = catalog::check(
            &given.schema,
            &given.parts,
            &given.style,
            &given.design_system,
        );
        if !findings.is_empty() {
            return Err(UseCaseError::new(
                "部品のカタログが契約を満たさない",
                findings.join("\n"),
            ));
        }
        let parts = Parts::new(&given)
            .map_err(|error| UseCaseError::new("部品のカタログを読めない", error))?;
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
        let templates = self.templates(pages, &parts)?;
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
        let renderer = PageRenderer::new(engine, parts);
        // すべて生成してから書く（途中で失敗したときに、一部だけ書いた状態を残さない）
        let (mut generated, mut missing) = (Vec::new(), Vec::new());
        for (doc_index, instance) in loaded.iter().enumerate() {
            let kind = instance
                .value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("");
            let Some(page_template) = templates.get(kind) else {
                missing.push(format!("{}（kind {kind}）", instance.path));
                continue;
            };
            let html = renderer.render(page_template, &ctx[doc_index], &instance.schema);
            let html = html.map_err(|error| {
                UseCaseError::new("描画できない", format!("{}: {}", instance.path, error.0))
            })?;
            let name = crate::domain::path::file_name(&instance.path);
            let stem = name.strip_suffix(".json").unwrap_or(name);
            generated.push((format!("{}/{stem}.html", out.trim_end_matches('/')), html));
        }
        if !missing.is_empty() {
            return Err(UseCaseError::new(
                "ページテンプレートの指定が無い種類がある",
                format!(
                    "次のインスタンスの種類には、ページテンプレートの指定が無い：{}",
                    missing.join("、")
                ),
            ));
        }
        let mut written = Vec::new();
        for (path, html) in generated {
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
            written.push(RenderedPage { path, changed });
        }
        Ok(Rendered { pages: written })
    }
}

#[cfg(test)]
mod tests {
    //! 描画のユースケースのテスト。部品のカタログは、試験用の小さなカタログ（crate::tests::small_catalog）を代役が渡す。
    //! カタログが無いか、契約を満たさないか、ページテンプレートの指定が無い種類があれば、1ページも書かない。

    use super::Renders;
    use crate::ports::Catalog;
    use crate::tests::{at, small_catalog, Answer, MemoryFiles, TableQuery, TestDesign};
    use serde_json::json;
    use std::sync::Arc;

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
                json!({"kind": "thing", "sections": [{"heading": "タグ", "body": [{"component": "pill", "each": "this.tags", "text": "@"}]}]}),
            ),
        ])
    }

    fn renders(
        files: &MemoryFiles,
        catalog: Option<Catalog>,
    ) -> Renders<MemoryFiles, MemoryFiles, TableQuery> {
        Renders::new(
            files.clone(),
            files.clone(),
            TableQuery::new(answers()),
            Arc::new(TestDesign(catalog)),
        )
        .with_skill("skills/example")
    }

    #[test]
    fn the_given_catalog_writes_the_page() {
        let files = files();
        let done = renders(&files, Some(small_catalog()))
            .render("declarations", "pages", "out")
            .unwrap();
        assert_eq!(done.pages.len(), 1);
        assert_eq!(done.pages[0].path, "out/T-1.html");
        assert_eq!(
            files.get("out/T-1.html").unwrap(),
            "<style>:root{--ink:#111d1a;--paper:#f7f8f7}.pill{color:var(--ink)}</style><script>/* 操作 */</script><main><h1>T-1</h1><p></p><section><h2>タグ</h2><b class=\"pill \">a</b></section></main>"
        );
    }

    #[test]
    fn without_a_catalog_no_page_is_written() {
        let files = files();
        let before = files.paths();
        let error = renders(&files, None)
            .render("declarations", "pages", "out")
            .unwrap_err();
        assert_eq!(error.reason, "部品のカタログを読めない");
        assert_eq!(error.detail, "カタログが無い");
        assert_eq!(files.paths(), before, "1ページも書かない");
    }

    #[test]
    fn a_catalog_that_breaks_its_design_system_writes_no_page() {
        let files = files();
        let before = files.paths();
        let mut broken = small_catalog();
        broken.style = ".pill{color:#ff0000}".to_owned();
        let error = renders(&files, Some(broken))
            .render("declarations", "pages", "out")
            .unwrap_err();
        assert_eq!(error.reason, "部品のカタログが契約を満たさない");
        assert!(error.detail.contains("色の直値"), "{}", error.detail);
        assert_eq!(files.paths(), before, "1ページも書かない");
    }

    #[test]
    fn a_kind_without_a_page_template_is_not_drawn() {
        let files = files();
        files.put(
            "declarations/O-1.json",
            &json!({"$schema": "example/thing@v1", "kind": "other", "id": "O-1"}).to_string(),
        );
        let before = files.paths();
        let error = renders(&files, Some(small_catalog()))
            .render("declarations", "pages", "out")
            .unwrap_err();
        assert_eq!(error.reason, "ページテンプレートの指定が無い種類がある");
        assert_eq!(
            error.detail,
            "次のインスタンスの種類には、ページテンプレートの指定が無い：declarations/O-1.json（kind other）"
        );
        assert_eq!(files.paths(), before, "1ページも書かない");
    }
}
