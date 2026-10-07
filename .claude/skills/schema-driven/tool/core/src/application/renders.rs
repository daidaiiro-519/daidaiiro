//! UC-6 ページを描画する（3d）。具体のデザインを受け取り、契約を通ったデータだけを、同じ入力から同じページへ描画する。
//! 基盤の文書（kind が document）は、基盤の決まったデザインで描画する。何も指定が無いときに使う。
//! 具体のスキーマの種類は、具体のページテンプレートとデザインで描画する。どちらでもない種類があれば、何も書かずに失敗する。

use crate::application::checks::Checks;
use crate::application::context::contexts;
use crate::application::instances::{Instances, UseCaseError};
use crate::application::page::{document_page_template, page_errors, PageRenderer};
use crate::application::view::{Instance, ViewEngine};
use crate::domain::check::Doc;
use crate::domain::schema::Schema;
use crate::domain::values::{JsonValue, ValidationError};
use crate::ports::inbound::RenderUseCases;
use crate::ports::outbound::{Design, Files, Functions, Query, Schemas, WriteIf};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

const PAGE_SCHEMA: &str = include_str!("../../../../references/page.schema.json");

/// 書いたページ1枚。
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedPage {
    pub path: String,
    /// 内容が変わったか（変わっていなければ書いていない）
    pub changed: bool,
    /// 使ったデザイン。具体のページテンプレートとデザインなら concrete、基盤の文書の決まったデザインなら document
    pub design: String,
}

/// 描画の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    pub pages: Vec<RenderedPage>,
}

/// 描画のユースケース。基盤の文書の決まったデザインと、具体のデザイン（あれば）と、具体が追加した関数を持つ。
pub struct Renders {
    files: Arc<dyn Files>,
    schemas: Arc<dyn Schemas>,
    query: Arc<dyn Query>,
    document_design: Arc<dyn Design>,
    design: Option<Arc<dyn Design>>,
    functions: Option<Arc<dyn Functions>>,
}

impl Renders {
    pub fn new(
        files: Arc<dyn Files>,
        schemas: Arc<dyn Schemas>,
        query: Arc<dyn Query>,
        document_design: Arc<dyn Design>,
        design: Option<Arc<dyn Design>>,
        functions: Option<Arc<dyn Functions>>,
    ) -> Self {
        Self {
            files,
            schemas,
            query,
            document_design,
            design,
            functions,
        }
    }

    /// ページテンプレートのディレクトリを読み、kind → テンプレートにする。契約に違反があれば失敗する。
    /// ページテンプレートのコンポーネントは具体のデザインが持つので、具体のデザインが無ければ読まない。
    fn templates(&self, dir: &str) -> Result<BTreeMap<String, Value>, UseCaseError> {
        let Some(design) = &self.design else {
            return Ok(BTreeMap::new());
        };
        if dir.is_empty() {
            return Ok(BTreeMap::new());
        }
        let reader = Instances::new(self.files.clone(), self.schemas.clone(), self.query.clone());
        let shape = Schema::new(
            "page.schema.json",
            serde_json::from_str(PAGE_SCHEMA).unwrap_or(Value::Null),
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

impl RenderUseCases for Renders {
    fn render(&self, dir: &str, pages: &str, out: &str) -> Result<Rendered, UseCaseError> {
        let checks = Checks::new(self.files.clone(), self.schemas.clone(), self.query.clone());
        let (loaded, schemas) = checks.load_dir(dir)?;
        // 契約を通ったデータだけを描画する（未記入は止めない。ACDR 0118）
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
        let mut engine = ViewEngine::new(self.query.clone(), schemas.into_iter().collect(), list);
        if let Some(functions) = &self.functions {
            engine = engine
                .with_functions(functions.clone())
                .map_err(|error| UseCaseError::new("関数を追加できない", error.0))?;
        }
        let document_renderer = PageRenderer::new(engine.clone(), self.document_design.clone())
            .map_err(|error| UseCaseError::new("文書のデザインを読めない", error.0))?;
        let document_template = document_page_template();
        let concrete_renderer = match &self.design {
            Some(design) => Some(
                PageRenderer::new(engine.clone(), design.clone())
                    .map_err(|error| UseCaseError::new("デザインを読めない", error.0))?,
            ),
            None => None,
        };
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
                    document_renderer.render(&document_template, &ctx[doc_index], &instance.schema),
                    "document",
                )
            } else if let (Some(renderer), Some(page_template)) =
                (&concrete_renderer, templates.get(kind))
            {
                (
                    renderer.render(page_template, &ctx[doc_index], &instance.schema),
                    "concrete",
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
                "デザインテンプレートが無い種類がある",
                format!(
                    "次のインスタンスは、基盤の文書でもなく、具体のページテンプレートとデザインも無い：{}",
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
                        .map_err(Instances::write_error)?;
                    true
                }
                Err(_) => {
                    self.files
                        .write(&path, html.as_str(), WriteIf::Absent)
                        .map_err(Instances::write_error)?;
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
