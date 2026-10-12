//! 値オブジェクト。宣言 VO-n の成分 ・ 不変条件 ・ 操作をそのまま持つ。作れない値は `new` が拒む。

use sha2::{Digest, Sha256};
use std::fmt;

/// 値オブジェクトを作れなかった理由。どの宣言のどの不変条件に反したかを持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidValue {
    pub invariant: &'static str,
    pub message: String,
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.invariant, self.message)
    }
}

impl std::error::Error for InvalidValue {}

fn non_empty(value: &str, invariant: &'static str, what: &str) -> Result<String, InvalidValue> {
    if value.chars().count() >= 1 {
        Ok(value.to_owned())
    } else {
        Err(InvalidValue {
            invariant,
            message: format!("{what}が空である"),
        })
    }
}

/// パス。インスタンスのファイルの場所。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstancePath(String);

impl InstancePath {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        non_empty(value, "VO-1.INV-1", "ファイルのパス").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// スキーマ。インスタンスの形と注釈を書いた JSON Schema のファイルのパス。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaPath(String);

impl SchemaPath {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        non_empty(value, "VO-2.INV-1", "スキーマのファイルのパス").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 検証エラー。スキーマを満たさないプロパティと、その理由。未記入は含めない。
/// 処理の失敗を表す Rust のエラー型ではなく、検証の結果として返すデータである（std::error::Error を実装しない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    property: String,
    reason: String,
}

impl ValidationError {
    /// プロパティは JSON Pointer で、空ならインスタンスの根を指す。
    pub fn new(property: &str, reason: &str) -> Result<Self, InvalidValue> {
        let reason = non_empty(reason, "VO-5.INV-1", "理由")?;
        Ok(Self {
            property: property.to_owned(),
            reason,
        })
    }
    pub fn property(&self) -> &str {
        &self.property
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// ハッシュ値。ファイルの内容の sha256 を16進の文字列で表す。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash(String);

impl Hash {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        if value.chars().count() == 64 {
            Ok(Self(value.to_owned()))
        } else {
            Err(InvalidValue {
                invariant: "VO-6.INV-1",
                message: format!(
                    "16進の文字列の文字数が64でない（{}）",
                    value.chars().count()
                ),
            })
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// JSON Patch（RFC 6902）。インスタンスへ適用する操作の並び。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPatch(String);

/// JSON Patch を適用できなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchError(pub String);

impl fmt::Display for PatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "JSON Patch を適用できない：{}", self.0)
    }
}

impl std::error::Error for PatchError {}

impl JsonPatch {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        non_empty(value, "VO-7.INV-1", "操作の並び").map(Self)
    }

    /// 適用する。JSON Patch を JSON の値に適用して、新しい値を求める。
    pub fn apply(&self, target: &JsonValue) -> Result<JsonValue, PatchError> {
        let patch: json_patch::Patch = serde_json::from_str(&self.0)
            .map_err(|error| PatchError(format!("JSON Patch として読めない: {error}")))?;
        let mut doc: serde_json::Value = serde_json::from_str(target.as_str())
            .map_err(|error| PatchError(format!("JSON の値として読めない: {error}")))?;
        json_patch::patch(&mut doc, &patch)
            .map_err(|error| PatchError(format!("適用できない: {error}")))?;
        Ok(JsonValue::new(&doc.to_string()))
    }
}

/// JSON の値。インスタンスのファイルに書いてある JSON。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonValue(String);

impl JsonValue {
    pub fn new(value: &str) -> Self {
        Self(value.to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// ハッシュ値を求める。JSON の値の sha256。
    pub fn hash(&self) -> Hash {
        let digest = Sha256::digest(self.0.as_bytes());
        let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        Hash(hex)
    }
}

/// 未記入。スキーマの必須のプロパティが、インスタンスに無いこと（JSON Schema の required の違反）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unfilled(String);

impl Unfilled {
    pub fn new(property: &str) -> Result<Self, InvalidValue> {
        non_empty(property, "VO-12.INV-1", "プロパティ").map(Self)
    }
    pub fn property(&self) -> &str {
        &self.0
    }
}

/// 検査結果の状態（検査結果の成分「状態」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// 合格
    Pass,
    /// ずれ（直すべき食い違い）
    Drift,
    /// 確かめ直し（承認のあとに指す先が変わった、またはまだ承認していない）
    Recheck,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Pass => "合格",
            Status::Drift => "ずれ",
            Status::Recheck => "確かめ直し",
        }
    }
}

/// 参照。x-ref を付けたプロパティに書いた値 ・ どの注釈の参照か ・ 指す先の種類の並び。
/// 宣言の参照（指す先 ・ x-ref の値 ・ 指す先の種類）は、宣言を移すときにこの形へ直す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    written: String,
    annotation: String,
    kinds: Vec<String>,
}

impl Reference {
    /// `annotation` は注釈の場所（スキーマ#JSON Pointer）、`kinds` は注釈の `to` の並び。
    pub fn new(written: &str, annotation: &str, kinds: Vec<String>) -> Result<Self, InvalidValue> {
        let written = non_empty(written, "VO-3.INV-1", "書いた値")?;
        Ok(Self {
            written,
            annotation: annotation.to_owned(),
            kinds,
        })
    }
    pub fn written(&self) -> &str {
        &self.written
    }
    pub fn annotation(&self) -> &str {
        &self.annotation
    }
    pub fn kinds(&self) -> &[String] {
        &self.kinds
    }
}

/// 導出値。導いた値と宣言した値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derived {
    derived: serde_json::Value,
    declared: serde_json::Value,
}

impl Derived {
    pub fn new(derived: serde_json::Value, declared: serde_json::Value) -> Self {
        Self { derived, declared }
    }
    /// 導出値を確かめる。同じなら合格、違えばずれ。
    pub fn compare(&self) -> Status {
        if self.derived == self.declared {
            Status::Pass
        } else {
            Status::Drift
        }
    }
}

/// ずれ。ずれと出た検査の名前と、その指す先。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    check: String,
    target: String,
}

impl Drift {
    pub fn new(check: &str, target: &str) -> Self {
        Self {
            check: check.to_owned(),
            target: target.to_owned(),
        }
    }
    pub fn check(&self) -> &str {
        &self.check
    }
    pub fn target(&self) -> &str {
        &self.target
    }
}

/// 承認したインスタンス。パスとハッシュ値の組。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedInstance {
    path: String,
    hash: Hash,
}

impl ApprovedInstance {
    pub fn new(path: &str, hash: Hash) -> Result<Self, InvalidValue> {
        let path = non_empty(path, "VO-11.INV-1", "パス")?;
        Ok(Self { path, hash })
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

/// 生成した HTML の断片。作れるのは、文字をエスケープするか、部品のプレースホルダーを置換するかだけである。
/// 具体は HTML の文字列を自分で作らない ── エスケープ漏れを型で防ぐ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Html(String);

impl Html {
    /// 文字をエスケープして HTML にする。
    pub fn escape(text: &str) -> Self {
        let mut escaped = String::with_capacity(text.len());
        for character in text.chars() {
            match character {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&#39;"),
                other => escaped.push(other),
            }
        }
        Self(escaped)
    }

    /// 部品のプレースホルダーを置換した結果など、基盤の中で生成したものだけを包む。
    pub(crate) fn trusted(html: String) -> Self {
        Self(html)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 後ろにつなぐ。
    pub fn push(&mut self, other: Html) {
        self.0.push_str(&other.0);
    }
}

#[cfg(test)]
mod tests {
    // 値オブジェクトのテスト。

    use crate::domain::values::{
        ApprovedInstance, Derived, Hash, InstancePath, JsonPatch, JsonValue, Reference, SchemaPath,
        Status, Unfilled, ValidationError,
    };
    use serde_json::json;

    // パス：ファイルのパスの文字数は1以上
    #[test]
    fn vo_1_inv_1_rejects_empty_path() {
        assert!(InstancePath::new("").is_err());
        assert!(InstancePath::new("declarations/UC-1.json").is_ok());
    }

    // スキーマ：スキーマのファイルのパスの文字数は1以上
    #[test]
    fn vo_2_inv_1_rejects_empty_schema_path() {
        assert!(SchemaPath::new("").is_err());
        assert!(SchemaPath::new("use_case.schema.json").is_ok());
    }

    // 検証エラー：理由の文字数は1以上（プロパティは空でもよい：インスタンスの根を指す）
    #[test]
    fn vo_5_inv_1_rejects_empty_reason() {
        assert!(ValidationError::new("/name", "").is_err());
        assert!(ValidationError::new("", "型が object でない（値：\"x\"）").is_ok());
    }

    // ハッシュ値：16進の文字列の文字数は64
    #[test]
    fn vo_6_inv_1_rejects_wrong_length() {
        assert!(Hash::new(&"a".repeat(63)).is_err());
        assert!(Hash::new(&"a".repeat(65)).is_err());
        assert!(Hash::new(&"a".repeat(64)).is_ok());
    }

    // JSON Patch：操作の並びの文字数は1以上
    #[test]
    fn vo_7_inv_1_rejects_empty_patch() {
        assert!(JsonPatch::new("").is_err());
    }

    // JSON Patch：適用する
    #[test]
    fn vo_7_op_1_ok_1_apply() {
        let patch = JsonPatch::new(r#"[{"op":"add","path":"/a","value":1}]"#).unwrap();
        let after = patch.apply(&JsonValue::new("{}")).unwrap();
        assert_eq!(after.as_str(), r#"{"a":1}"#);
    }

    // JSON の値：ハッシュ値を求める
    #[test]
    fn vo_10_op_1_ok_1_hash() {
        let hash = JsonValue::new("{}").hash();
        assert_eq!(
            hash.as_str(),
            "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
        );
    }

    // 未記入：プロパティの文字数は1以上
    #[test]
    fn vo_12_inv_1_rejects_empty_property() {
        assert!(Unfilled::new("").is_err());
        assert!(Unfilled::new("/header").is_ok());
    }

    // ── 値オブジェクト

    #[test]
    fn vo_3_inv_1_rejects_an_empty_written_value() {
        let kinds = vec!["use_case".to_owned()];
        assert!(Reference::new("", "a.schema.json#/properties/uses", kinds.clone()).is_err());
        let reference = Reference::new("UC-1", "a.schema.json#/properties/uses", kinds).unwrap();
        assert_eq!(reference.written(), "UC-1");
        assert_eq!(reference.annotation(), "a.schema.json#/properties/uses");
        assert_eq!(reference.kinds(), ["use_case".to_owned()]);
    }

    #[test]
    fn vo_3_keeps_every_kind_the_annotation_allows() {
        let kinds = vec!["use_case".to_owned(), "aggregate".to_owned()];
        let reference = Reference::new("UC-1", "a.schema.json#/properties/uses", kinds).unwrap();
        assert_eq!(reference.kinds().len(), 2);
    }

    #[test]
    fn vo_4_op_1_ok_1_same_values_pass() {
        assert_eq!(
            Derived::new(json!("中核"), json!("中核")).compare(),
            Status::Pass
        );
    }

    #[test]
    fn vo_4_op_1_ok_2_different_values_drift() {
        assert_eq!(
            Derived::new(json!("中核"), json!("補完")).compare(),
            Status::Drift
        );
    }

    #[test]
    fn vo_11_inv_1_rejects_empty_path() {
        assert!(ApprovedInstance::new("", JsonValue::new("{}").hash()).is_err());
    }

    // 基盤のエラー型は、Display と std::error::Error を実装し、句点で終えない（C-GOOD-ERR）。
    #[test]
    fn patch_error_describes_itself() {
        let message = Box::<dyn std::error::Error>::from(crate::domain::values::PatchError(
            "path が無い".into(),
        ))
        .to_string();
        assert_eq!(message, "JSON Patch を適用できない：path が無い");
    }
}
