//! adapters の単体テストが共有する補助。src の側のテストの補助は、ここ1か所に置く。

use std::fs;
use std::path::PathBuf;

/// 作業用のフォルダ。テストごとに別の名前で作り、終わると消す。
pub(crate) struct Dir(PathBuf);

impl Dir {
    pub(crate) fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sd-fs-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Dir(dir)
    }

    /// フォルダからの相対パスを、文字のパスにする。
    pub(crate) fn at(&self, rel: &str) -> String {
        self.0.join(rel).to_string_lossy().into_owned()
    }

    /// ファイルを書く（親のフォルダも作る）。
    pub(crate) fn put(&self, files: &[(&str, &str)]) {
        for (rel, text) in files {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    /// フォルダからの相対パスの並び。
    pub(crate) fn rel(&self, paths: Vec<String>) -> Vec<String> {
        let root = format!("{}/", self.0.to_string_lossy());
        paths
            .into_iter()
            .map(|path| path.trim_start_matches(&root).to_owned())
            .collect()
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
