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

/// VO-1 パス。インスタンスのファイルの場所。
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

/// VO-2 スキーマ。インスタンスの形と注釈を書いた JSON Schema のファイルのパス。
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

/// VO-5 検証エラー。スキーマを満たさないプロパティと、その理由。未記入は含めない。
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

/// VO-6 ハッシュ値。ファイルの内容の sha256 を16進の文字列で表す。
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

/// VO-7 JSON Patch（RFC 6902）。インスタンスへ適用する操作の並び。
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

    /// OP-1 適用する。JSON Patch を JSON の値に適用して、新しい値を求める。
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

/// VO-10 JSON の値。インスタンスのファイルに書いてある JSON。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonValue(String);

impl JsonValue {
    pub fn new(value: &str) -> Self {
        Self(value.to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// OP-1 ハッシュ値を求める。JSON の値の sha256。
    pub fn hash(&self) -> Hash {
        let digest = Sha256::digest(self.0.as_bytes());
        let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        Hash(hex)
    }
}

/// VO-12 未記入。スキーマの必須のプロパティが、インスタンスに無いこと（JSON Schema の required の違反）。
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

/// 検査結果の状態（VO-9 検査結果の成分「状態」）。
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

/// VO-3 参照。指す先 ・ x-ref の値 ・ 指す先の種類。
/// 宣言にはあるが、検査はまだ使っていない（参照は check.rs の GraphLink が持つ）。どちらへ合わせるかは別に決める
#[cfg_attr(not(feature = "internals"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    target: String,
    value: String,
    kind: String,
}

#[cfg_attr(not(feature = "internals"), allow(dead_code))]
impl Reference {
    pub fn new(target: &str, value: &str, kind: &str) -> Result<Self, InvalidValue> {
        let target = non_empty(target, "VO-3.INV-1", "指す先")?;
        Ok(Self {
            target,
            value: value.to_owned(),
            kind: kind.to_owned(),
        })
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn kind(&self) -> &str {
        &self.kind
    }
}

/// VO-4 導出値。導いた値と宣言した値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derived {
    derived: serde_json::Value,
    declared: serde_json::Value,
}

impl Derived {
    pub fn new(derived: serde_json::Value, declared: serde_json::Value) -> Self {
        Self { derived, declared }
    }
    /// OP-1 導出値を確かめる。同じなら合格、違えばずれ。
    pub fn compare(&self) -> Status {
        if self.derived == self.declared {
            Status::Pass
        } else {
            Status::Drift
        }
    }
}

/// VO-8 ずれ。ずれと出た検査の名前と、その指す先。
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

/// VO-11 承認したインスタンス。パスとハッシュ値の組。
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

/// 生成した HTML の断片（3c）。作れるのは、文字をエスケープするか、部品のプレースホルダーを置換するかだけである。
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
