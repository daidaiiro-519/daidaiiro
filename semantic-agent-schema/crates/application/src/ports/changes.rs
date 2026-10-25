//! 変更の仕組みとやりとりする

use std::path::PathBuf;

/// 1つのファイルの前後：前の姿（前回の確定）と後の姿（今の作業ツリー）の中身。無ければ None
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    /// 作業の根からのパス
    pub path: String,
    /// 前の姿の中身
    pub before: Option<String>,
    /// 後の姿の中身
    pub after: Option<String>,
}

/// 変更の仕組みとやりとりする：前の姿の参照と、そこから今までに変わったファイルの前後を返す。参照は基盤が解釈しない文字列
pub trait Changes {
    /// 前の姿の参照（前回の確定）。まだ1つも確定していなければ None
    ///
    /// # Errors
    /// 変更の仕組みを呼べないとき
    fn head(&self) -> Result<Option<String>, String>;

    /// 前の姿から今の作業ツリーまでに変わったファイル（まだ追われていないファイルを含む）
    ///
    /// # Errors
    /// 変更の仕組みを呼べないとき
    fn changed(&self) -> Result<Vec<FileChange>, String>;

    /// すべての作業ツリーが共有する置き場所（手元の DB の既定の置き場所に使う）。変更の仕組みの外なら None
    fn shared_dir(&self) -> Option<PathBuf>;
}
