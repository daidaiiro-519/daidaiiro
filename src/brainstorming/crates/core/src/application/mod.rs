//! ユースケース。port を通して入出力し、決まりは domain に任せる。

pub mod advance;
pub mod answer;
pub mod chat;
pub mod engine;
pub mod hold;
pub mod inspect;
pub mod lock;
pub mod render;
pub mod status;
pub mod tokens;

use crate::ports::{Clock, Files, Programs};
use std::path::{Path, PathBuf};

/// port の組。
pub struct Env<F, P, C> {
    pub files: F,
    pub programs: P,
    pub clock: C,
}

/// brainstorming 自身の Skill のフォルダ。
#[derive(Clone, Debug, PartialEq)]
pub struct Skill {
    pub dir: PathBuf,
}

impl Skill {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }
    /// 照合して入れた基盤のエンジン。
    pub fn engine(&self) -> PathBuf {
        self.dir
            .join("scripts")
            .join("bin")
            .join(format!("schema-driven{}", std::env::consts::EXE_SUFFIX))
    }
    /// この Skill の実行ファイル。
    pub fn entry(&self) -> PathBuf {
        self.dir
            .join("scripts")
            .join("bin")
            .join(format!("brainstorming{}", std::env::consts::EXE_SUFFIX))
    }
    pub fn skill_md(&self) -> PathBuf {
        self.dir.join("SKILL.md")
    }
    pub fn pages(&self) -> PathBuf {
        self.dir.join("assets").join("pages")
    }
    pub fn catalog(&self) -> PathBuf {
        self.dir.join("assets").join("catalog")
    }
}

/// path を base からの場所にする（base の外なら path のまま）。区切りは / にする。
pub fn relative(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

impl<F: Files, P: Programs, C: Clock> Env<F, P, C> {
    /// JSON のファイルを読む。
    pub fn read_json(&self, path: &Path) -> Result<serde_json::Value, String> {
        let text = self.files.read(path)?;
        serde_json::from_str(&text)
            .map_err(|error| format!("{}：JSON として読めない：{error}", path.display()))
    }
}
