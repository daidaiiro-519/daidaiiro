//! ポート SchemaChecker を jsonschema で実装する。
//!
//! Abstract の本体 ・ 方言 ・ 語彙を実行ファイルに入れて出し、名前（$id）で引く。外へ取りに行かない。

use contracts_and_instances_conformance_application::ports::SchemaChecker;
use serde_json::Value;

/// 方言が $vocabulary で求める語彙。x- の形は語彙のメタスキーマが決め、意味は SAS が判定する。
const VOCABULARY: &str = "urn:semantic-agent-schema:base:vocab:v1";

const SHIPPED: [&str; 3] = [
    include_str!("../../../../schemas/v1/semantic-abstract-schema.json"),
    include_str!("../../../../schemas/v1/dialect.schema.json"),
    include_str!("../../../../schemas/v1/vocabulary.schema.json"),
];

/// 入れて出すスキーマを持つ検査器。
#[derive(Debug, Clone)]
pub struct Checker {
    shipped: Vec<(String, Value)>,
}

impl Checker {
    /// 入れて出すスキーマを読む。
    ///
    /// # Errors
    /// 入れて出すスキーマが JSON でない ・ $id が無いとき（作り方の誤り）。
    pub fn new() -> Result<Self, String> {
        let mut shipped = Vec::new();
        for text in SHIPPED {
            let doc: Value = serde_json::from_str(text)
                .map_err(|e| format!("入れて出すスキーマが JSON でない：{e}"))?;
            let id = doc
                .get("$id")
                .and_then(Value::as_str)
                .ok_or("入れて出すスキーマに $id が無い")?
                .to_string();
            shipped.push((id, doc));
        }
        Ok(Self { shipped })
    }
}

impl SchemaChecker for Checker {
    fn shipped(&self, id: &str) -> Option<Value> {
        self.shipped
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, d)| d.clone())
    }

    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        let mut builder = jsonschema::Registry::new();
        for (id, doc) in self.shipped.iter().chain(more) {
            builder = builder
                .add(id.clone(), doc.clone())
                .map_err(|e| e.to_string())?;
        }
        let registry = builder.prepare().map_err(|e| e.to_string())?;
        let doc = self
            .shipped
            .iter()
            .chain(more)
            .find(|(i, _)| i == schema)
            .map(|(_, d)| d.clone())
            .ok_or_else(|| format!("{schema} を引けない"))?;
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .with_vocabulary(VOCABULARY)
            .with_registry(&registry)
            .build(&doc)
            .map_err(|e| format!("{schema} を検査器に組めない：{e}"))?;
        Ok(validator
            .iter_errors(value)
            .map(|e| (e.instance_path().as_str().to_string(), e.to_string()))
            .collect())
    }
}
