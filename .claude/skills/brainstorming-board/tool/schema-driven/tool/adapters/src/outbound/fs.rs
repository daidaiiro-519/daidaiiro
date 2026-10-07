//! ファイルシステムのアダプタ（ポート Files の実装）。

use schema_driven_core::domain::JsonValue;
use schema_driven_core::ports::outbound::{Files, ReadError, Schemas, WriteError, WriteIf};
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct FileSystem;

fn unwritable(error: std::io::Error) -> WriteError {
    WriteError::Unwritable(error.to_string())
}

impl Files for FileSystem {
    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn read(&self, path: &str) -> Result<String, ReadError> {
        fs::read_to_string(path).map_err(|error| ReadError(format!("{path}: {error}")))
    }

    fn list(&self, dir: &str) -> Result<Vec<String>, ReadError> {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir).map_err(|error| ReadError(format!("{dir}: {error}")))? {
            let entry = entry.map_err(|error| ReadError(error.to_string()))?;
            if entry.path().is_file() {
                out.push(format!(
                    "{}/{}",
                    dir.trim_end_matches('/'),
                    entry.file_name().to_string_lossy()
                ));
            }
        }
        out.sort();
        Ok(out)
    }

    fn list_tree(&self, dir: &str) -> Result<Vec<String>, ReadError> {
        fn walk(dir: &Path, out: &mut Vec<String>) -> Result<(), ReadError> {
            for entry in fs::read_dir(dir)
                .map_err(|error| ReadError(format!("{}: {error}", dir.display())))?
            {
                let path = entry.map_err(|error| ReadError(error.to_string()))?.path();
                if path.is_dir() {
                    walk(&path, out)?;
                } else if path.is_file() {
                    out.push(path.to_string_lossy().into_owned());
                }
            }
            Ok(())
        }
        let root = Path::new(dir);
        if !root.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        walk(root, &mut out)?;
        out.sort();
        Ok(out)
    }

    /// 作成は、まだ無いときだけ書く。更新は、読んだ時点のハッシュ値のままのときだけ、
    /// 一時ファイルへ書いてから置き換える（書く途中で失敗しても、前の内容が残る）。
    fn write(&self, path: &str, content: &str, cond: WriteIf) -> Result<(), WriteError> {
        let target = Path::new(path);
        match cond {
            WriteIf::Absent => {
                if let Some(parent) = target
                    .parent()
                    .filter(|parent_dir| !parent_dir.as_os_str().is_empty())
                {
                    fs::create_dir_all(parent).map_err(unwritable)?;
                }
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(target)
                    .map_err(|error| {
                        if error.kind() == std::io::ErrorKind::AlreadyExists {
                            WriteError::Conflict
                        } else {
                            unwritable(error)
                        }
                    })?;
                file.write_all(content.as_bytes()).map_err(unwritable)
            }
            WriteIf::Unchanged(expected) => {
                let current = fs::read_to_string(target).map_err(unwritable)?;
                if JsonValue::new(&current).hash() != expected {
                    return Err(WriteError::Conflict);
                }
                if fs::metadata(target)
                    .map_err(unwritable)?
                    .permissions()
                    .readonly()
                {
                    return Err(WriteError::Unwritable(format!(
                        "{path}: 読み取り専用である"
                    )));
                }
                let tmp = target.with_extension("json.tmp");
                fs::write(&tmp, content).map_err(unwritable)?;
                fs::rename(&tmp, target).map_err(|error| {
                    let _ = fs::remove_file(&tmp);
                    unwritable(error)
                })
            }
        }
    }

    fn remove(&self, path: &str) -> Result<(), WriteError> {
        fs::remove_file(path).map_err(unwritable)
    }
}

/// スキーマの供給元。同じディレクトリの `*.schema.json` を返す（ネットワークには出ない）。
impl Schemas for FileSystem {
    fn referenced(&self, path: &str) -> Result<Vec<(String, String)>, ReadError> {
        let dir = Path::new(path)
            .parent()
            .filter(|parent_dir| !parent_dir.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut out = Vec::new();
        for entry in fs::read_dir(dir).map_err(|error| ReadError(error.to_string()))? {
            let entry = entry.map_err(|error| ReadError(error.to_string()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".schema.json") {
                if let Ok(text) = fs::read_to_string(entry.path()) {
                    out.push((name, text));
                }
            }
        }
        out.sort();
        Ok(out)
    }
}

/// 正本（schema-driven の Skill）のディレクトリ。実行ファイルは <Skill>/bin/ に置かれるので、その1つ上。
pub fn master_root() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent()?.parent().map(Path::to_path_buf))
        .map(|root| root.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_owned())
}
