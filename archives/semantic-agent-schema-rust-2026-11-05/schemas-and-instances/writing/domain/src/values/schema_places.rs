//! 渡されたスキーマの置き場所（--schemas）。受け取ったままの並びと、読むファイルの並びを持つ。

/// --schemas で受け取った置き場所と、それを開いた、読むファイルの経路。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaPlaces {
    given: Vec<String>,
    files: Vec<String>,
}

impl SchemaPlaces {
    /// 受け取ったままの置き場所（given）と、読むファイルの経路（files）から作る。
    #[must_use]
    pub const fn new(given: Vec<String>, files: Vec<String>) -> Self {
        Self { given, files }
    }

    /// 読むファイルの経路。
    #[must_use]
    pub fn files(&self) -> &[String] {
        &self.files
    }

    /// 返すコマンドに引き継ぐ引数（--schemas と、受け取ったままの置き場所）。
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        std::iter::once("--schemas".to_string())
            .chain(self.given.iter().cloned())
            .collect()
    }
}
