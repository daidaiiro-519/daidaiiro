//! contracts ・ instances ・ judgments（rule の判定）・ change_summaries

use semantic_agent_schema_application::ports::contracts::Contracts;
use semantic_agent_schema_application::ports::instances::Instances;
use serde_json::Value;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

/// 作業の根の下のファイルで、契約を読み書きする
pub struct Files {
    root: PathBuf,
}

impl Files {
    /// 作業の根を渡して作る
    pub fn new(root: PathBuf) -> Self {
        Files { root }
    }
}

impl Files {
    /// 作業の根の下のパスにする。絶対パスと .. を含むパスは断る（根の外を読み書きしない）
    fn inside(&self, path: &str) -> Result<PathBuf, String> {
        let p = Path::new(path);
        if p.components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
        {
            return Err(format!(
                "{path} は作業の根の下のパスでない（絶対パスと .. は使わない）"
            ));
        }
        Ok(self.root.join(p))
    }
}

impl Contracts for Files {
    fn read(&self, path: &str) -> Result<Option<Value>, String> {
        match fs::read_to_string(self.inside(path)?) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| format!("{path} が JSON でない：{e}")),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{path} を読めない：{e}")),
        }
    }

    fn find(&self, id: &str) -> Result<Option<Value>, String> {
        let mut found: Vec<(PathBuf, Value)> = Vec::new();
        for path in json_files(&self.root) {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let doc = match serde_json::from_str::<Value>(&text) {
                Ok(doc) => doc,
                Err(e) if text.contains(id) => {
                    return Err(format!(
                        "{} は $id「{id}」を含むが JSON として読めない：{e}",
                        path.display()
                    ));
                }
                Err(_) => continue,
            };
            if doc.get("$id").and_then(Value::as_str) == Some(id) {
                found.push((path, doc));
            }
        }
        match found.len() {
            0 => Ok(None),
            1 => Ok(found.pop().map(|(_, d)| d)),
            _ => Err(format!(
                "$id「{id}」の文書が2つ以上ある：{}",
                found
                    .iter()
                    .map(|(p, _)| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" ・ ")
            )),
        }
    }

    fn write(&self, path: &str, document: &Value) -> Result<(), String> {
        let dest = self.inside(path)?;
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{} を作れない：{e}", dir.display()))?;
        }
        let mut text = serde_json::to_string_pretty(document).map_err(|e| e.to_string())?;
        text.push('\n');
        let tmp = dest.with_extension("json.tmp");
        fs::write(&tmp, text).map_err(|e| format!("{path} を書けない：{e}"))?;
        fs::rename(&tmp, &dest).map_err(|e| format!("{path} を置き換えられない：{e}"))
    }
}

impl Instances for Files {
    fn read(&self, path: &str) -> Result<Option<Value>, String> {
        Contracts::read(self, path)
    }

    fn write(&self, path: &str, document: &Value) -> Result<(), String> {
        Contracts::write(self, path, document)
    }
}

/// 根の下の .json を、名前の順に返す。.git ・ target ・ node_modules は見ない
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return out;
    };
    let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if p.is_dir() {
            if ![".git", "target", "node_modules"].contains(&name.as_str()) {
                out.extend(json_files(&p));
            }
        } else if name.ends_with(".json") {
            out.push(p);
        }
    }
    out
}
