//! ポート SchemaValidator（書く ・ 検証 ・ 描画 ・ 検索のモジュールがそれぞれ宣言する）を jsonschema で実装する。
//!
//! Abstract の本体 ・ 方言 ・ 語彙と、デザインシステムの受け取りの型 ・ テンプレートの書き方の契約を実行ファイルに入れて出し、名前（$id）で引く。外へ取りに行かない。

use serde_json::Value;

/// 方言が $vocabulary で求める語彙。x- の形は語彙のメタスキーマが決め、意味は Semantic Agent Schema が判定する。
const VOCABULARY: &str = "urn:semantic-agent-schema:published-language:vocab:v1";

const SHIPPED: [&str; 5] = [
    include_str!("../../../../published-language/v1/abstract.schema.json"),
    include_str!("../../../../published-language/v1/design-system.schema.json"),
    include_str!("../../../../published-language/v1/template-language.json"),
    include_str!("../../../../published-language/v1/dialect.schema.json"),
    include_str!("../../../../published-language/v1/vocabulary.schema.json"),
];

/// 入れて出すスキーマを持つバリデーター。
#[derive(Debug, Clone)]
pub struct Validator {
    shipped: Vec<(String, Value)>,
}

impl Validator {
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

impl Validator {
    /// 入れて出すスキーマの文書。無い名前は None。
    #[must_use]
    pub fn shipped(&self, id: &str) -> Option<Value> {
        self.shipped
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, d)| d.clone())
    }

    /// value を、$id が schema のスキーマに照らし、外れた所（JSON Pointer）と理由を返す。
    ///
    /// # Errors
    /// スキーマを組めないとき。
    pub fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Ok(self
            .validator(schema, more)?
            .iter_errors(value)
            .map(|e| (e.instance_path().as_str().to_string(), e.to_string()))
            .collect())
    }

    /// value を、$id が schema のスキーマに照らし、形が通った所の注釈（インスタンスの所と、キーワード → 値）を返す。
    ///
    /// # Errors
    /// スキーマを組めないとき。
    pub fn annotations(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, String> {
        let evaluation = self.validator(schema, more)?.evaluate(value);
        Ok(evaluation
            .iter_annotations()
            .map(|a| {
                (
                    a.instance_location.as_str().to_string(),
                    a.annotations.value().clone(),
                )
            })
            .collect())
    }
}

impl writing_application::ports::SchemaValidator for Validator {
    fn shipped(&self, id: &str) -> Option<Value> {
        Self::shipped(self, id)
    }

    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }

    fn annotations(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, String> {
        Self::annotations(self, schema, value, more)
    }
}

impl validation_application::ports::SchemaValidator for Validator {
    fn shipped(&self, id: &str) -> Option<Value> {
        Self::shipped(self, id)
    }

    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }
}

impl rendering_application::ports::SchemaValidator for Validator {
    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }
}

impl search_application::ports::SchemaValidator for Validator {
    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }
}

impl Validator {
    /// $id が schema のスキーマを、入れて出すスキーマと more を引ける形で組む。
    fn validator(
        &self,
        schema: &str,
        more: &[(String, Value)],
    ) -> Result<jsonschema::Validator, String> {
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
        jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .with_vocabulary(VOCABULARY)
            .with_registry(&registry)
            .build(&doc)
            .map_err(|e| format!("{schema} をバリデーターに組めない：{e}"))
    }
}
