//! 対象（id と kind か、パス）

use serde_json::{Value, json};

/// 対象：変更や判定が指すもの。インスタンスは id と kind（従う Concrete の URN）、それ以外のファイルはパス
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    /// インスタンス
    Instance {
        /// id
        id: String,
        /// 種類（従う Concrete の URN）
        kind: String,
    },
    /// それ以外のファイル（作業の根からのパス）
    Path(String),
}

impl Target {
    /// 文書から対象を決める。kind ・ id と、Concrete の URN の $schema があればインスタンス、無ければパス
    pub fn of_document(path: &str, document: Option<&Value>) -> Self {
        let text = |k: &str| document.and_then(|d| d.get(k)).and_then(Value::as_str);
        match (text("id"), text("kind"), text("$schema")) {
            (Some(id), Some(_), Some(schema))
                if schema.starts_with("urn:semantic-agent-schema:")
                    && !schema.contains(":base:") =>
            {
                Target::Instance {
                    id: id.to_string(),
                    kind: schema.to_string(),
                }
            }
            _ => Target::Path(path.to_string()),
        }
    }

    /// change.schema.json の target の形
    pub fn to_json(&self) -> Value {
        match self {
            Target::Instance { id, kind } => json!({"id": id, "kind": kind}),
            Target::Path(p) => json!({"path": p}),
        }
    }

    /// 1行の文字列（索引のキー）
    pub fn key(&self) -> String {
        match self {
            Target::Instance { id, kind } => format!("{kind}#{id}"),
            Target::Path(p) => p.clone(),
        }
    }

    /// 1行の文字列から戻す
    pub fn from_key(key: &str) -> Self {
        match key.split_once('#') {
            Some((kind, id)) if kind.starts_with("urn:") => Target::Instance {
                id: id.to_string(),
                kind: kind.to_string(),
            },
            _ => Target::Path(key.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Target;
    use serde_json::json;

    /// traces: U5:事後
    #[test]
    fn instance_is_pointed_by_id_and_kind() {
        let d = json!({"$schema": "urn:semantic-agent-schema:acme:record:v1", "kind": "record", "id": "r-1"});
        assert_eq!(
            Target::of_document("a.json", Some(&d)).key(),
            "urn:semantic-agent-schema:acme:record:v1#r-1"
        );
    }

    /// traces: U5:事後
    #[test]
    fn other_file_is_pointed_by_path() {
        assert_eq!(
            Target::of_document("src/a.rs", None),
            Target::Path("src/a.rs".into())
        );
    }

    /// traces: U13:事後
    #[test]
    fn key_round_trips() {
        let t = Target::Instance {
            id: "r-1".into(),
            kind: "urn:semantic-agent-schema:a:b:v1".into(),
        };
        assert_eq!(Target::from_key(&t.key()), t);
    }
}
