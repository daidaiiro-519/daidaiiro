//! 層の内側である。外側の層へ依存しない。

use std::error::Error;
use std::fmt;

/// この層の名前を返す。
#[must_use]
pub fn name() -> &'static str {
    "core"
}

/// 層の名前が受け取れない理由である。
///
/// あとから理由を足せるようにしてある。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayerError {
    /// 名前が空である。
    Empty,
}

impl fmt::Display for LayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Empty => write!(f, "名前が空である"),
        }
    }
}

impl Error for LayerError {}

/// 名前を1つ持つ層である。
///
/// 欄は私用にしてある ── 公開すると、不変条件を保持できない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    label: String,
}

impl Layer {
    /// 名前から層を組む。
    ///
    /// # Errors
    ///
    /// 名前が空のときは [`LayerError::Empty`] を返す。
    pub fn new(label: &str) -> Result<Self, LayerError> {
        if label.is_empty() {
            return Err(LayerError::Empty);
        }
        Ok(Self {
            label: label.to_owned(),
        })
    }

    /// 層の名前を返す。
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}
