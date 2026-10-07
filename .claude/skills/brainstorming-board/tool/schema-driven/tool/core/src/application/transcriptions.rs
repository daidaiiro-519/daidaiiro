//! UC-9 基盤の複製を転写する ・ UC-10 複製と正本の差分を検査する。
//! 正本の tool/core ・ tool/adapters ・ references を、同じ並びのまま利用側の Skill の tool/schema-driven/ の下へ写す。
//! 並びが同じなら、crate が取り込むファイルの相対パス（../../../../references/…）が正本と複製で同じになり、複製を書き換えずに動く。
//! 転写した時点で、複製は具体の持ち物になる。基盤の新しい版へ更新するときも、具体が変えたファイルは上書きしない。
//! 転写した時点の基盤の版とハッシュ値は tool/schema-driven/transcription.json に記録し、基盤の変更と具体の変更を区別する。

use crate::application::instances::{Instances, UseCaseError};
use crate::domain::values::JsonValue;
use crate::ports::inbound::TranscriptionUseCases;
use crate::ports::outbound::{Files, WriteIf};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// 写す範囲（正本の中の場所。複製の中でも同じ場所に置く）。
const ROOTS: [&str; 3] = ["tool/core", "tool/adapters", "references"];
/// 写さないディレクトリ（正本のテストと、ビルドの出力）。
const EXCLUDED_DIRECTORIES: [&str; 2] = ["tests", "target"];
/// 複製の中の、転写の記録のファイル。
const RECORD: &str = "transcription.json";
/// 基盤の版を読むファイル（複製の中の場所）。
const VERSION_FILE: &str = "tool/core/Cargo.toml";
/// 基盤と具体の両方が変えたファイルについて、基盤の新しい版を隣に置くときの名前の後ろ。
pub const CONFLICT_SUFFIX: &str = ".schema-driven-new";

/// 転写の結果。パスは複製の中の場所（例：tool/core/src/lib.rs）。
#[derive(Debug, Clone, PartialEq)]
pub struct Transcribed {
    /// 前に転写した基盤の版。初めてなら None
    pub from_version: Option<String>,
    pub to_version: String,
    /// 基盤の版で書いたファイル
    pub written: Vec<String>,
    /// 基盤から消えたので消したファイル
    pub removed: Vec<String>,
    /// 具体が変えたので、そのまま残したファイル
    pub kept: Vec<String>,
    /// 基盤と具体の両方が変えたファイル。具体のものを残し、基盤の新しい版を隣に置いた
    pub conflicts: Vec<String>,
}

/// 複製と正本の差分1件。
#[derive(Debug, Clone, PartialEq)]
pub struct Difference {
    pub path: String,
    /// 正本が新しくなった ・ 具体が変えた ・ 衝突 ・ 正本に増えた ・ 正本から消えた ・ 具体が足した ・ 具体が消した ・ 衝突の新しい版が残っている
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyReport {
    /// 複製が持つ基盤の版（転写していなければ None）
    pub copy_version: Option<String>,
    /// いまの正本の版
    pub master_version: String,
    pub differences: Vec<Difference>,
}

/// 前に転写した時点の記録。
struct Record {
    version: Option<String>,
    hashes: BTreeMap<String, String>,
}

pub struct Transcriptions {
    files: Arc<dyn Files>,
    /// 正本（schema-driven の Skill）のディレクトリ
    master: String,
}

fn hash_of(text: &str) -> String {
    JsonValue::new(text).hash().as_str().to_owned()
}

/// Cargo.toml の [package] の version を読む。
fn version_of(cargo_toml: &str) -> Option<String> {
    cargo_toml.lines().find_map(|line| {
        let rest = line
            .trim()
            .strip_prefix("version")?
            .trim_start()
            .strip_prefix('=')?;
        Some(rest.trim().trim_matches('"').to_owned())
    })
}

/// ファイルごとの、基盤と具体の変更の有無。
struct Change {
    by_master: bool,
    by_concrete: bool,
}

impl Transcriptions {
    pub fn new(files: Arc<dyn Files>, master: String) -> Self {
        Self { files, master }
    }

    fn copy_dir(skill: &str) -> String {
        format!("{}/tool/schema-driven", skill.trim_end_matches('/'))
    }

    fn read(&self, path: &str) -> Result<String, UseCaseError> {
        self.files
            .read(path)
            .map_err(|error| UseCaseError::new("読めない", error.0))
    }

    /// 正本の写す範囲のファイル：複製の中の場所 → 内容。
    fn master_files(&self) -> Result<BTreeMap<String, String>, UseCaseError> {
        let mut out = BTreeMap::new();
        for root in ROOTS {
            let dir = format!("{}/{root}", self.master.trim_end_matches('/'));
            for path in self
                .files
                .list_tree(&dir)
                .map_err(|error| UseCaseError::new("読めない", error.0))?
            {
                let relative = path.trim_start_matches(&dir).trim_start_matches('/');
                if relative
                    .split('/')
                    .any(|segment| EXCLUDED_DIRECTORIES.contains(&segment))
                {
                    continue;
                }
                out.insert(format!("{root}/{relative}"), self.read(&path)?);
            }
        }
        if out.is_empty() {
            return Err(UseCaseError::new(
                "読めない",
                format!("正本に写すファイルが無い：{}", self.master),
            ));
        }
        Ok(out)
    }

    fn master_version(master: &BTreeMap<String, String>) -> String {
        master
            .get(VERSION_FILE)
            .and_then(|text| version_of(text))
            .unwrap_or_else(|| "不明".to_owned())
    }

    /// 複製のファイル（記録と、衝突の新しい版を除く）と、衝突の新しい版の一覧。
    #[allow(clippy::type_complexity)]
    fn copy_files(
        &self,
        skill: &str,
    ) -> Result<(BTreeMap<String, String>, Vec<String>), UseCaseError> {
        let dir = Self::copy_dir(skill);
        let (mut out, mut pending) = (BTreeMap::new(), Vec::new());
        for path in self
            .files
            .list_tree(&dir)
            .map_err(|error| UseCaseError::new("読めない", error.0))?
        {
            let relative = path.trim_start_matches(&dir).trim_start_matches('/');
            if relative == RECORD {
                continue;
            }
            if relative.ends_with(CONFLICT_SUFFIX) {
                pending.push(relative.to_owned());
                continue;
            }
            out.insert(relative.to_owned(), self.read(&path)?);
        }
        Ok((out, pending))
    }

    fn record(&self, skill: &str) -> Result<Record, UseCaseError> {
        let path = format!("{}/{RECORD}", Self::copy_dir(skill));
        if !self.files.exists(&path) {
            return Ok(Record {
                version: None,
                hashes: BTreeMap::new(),
            });
        }
        let record: Value = serde_json::from_str(&self.read(&path)?).map_err(|error| {
            UseCaseError::new("JSON として読めない", format!("{path}: {error}"))
        })?;
        Ok(Record {
            version: record
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_owned),
            hashes: record
                .get("files")
                .and_then(Value::as_object)
                .map(|files| {
                    files
                        .iter()
                        .map(|(path, hash)| (path.clone(), hash.as_str().unwrap_or("").to_owned()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    /// 前に転写した時点から、基盤と具体のそれぞれが変えたか。
    fn change(master: Option<&String>, copy: Option<&String>, recorded: Option<&String>) -> Change {
        let hash = |text: Option<&String>| text.map(|text| hash_of(text));
        Change {
            by_master: hash(master).as_ref() != recorded,
            by_concrete: hash(copy).as_ref() != recorded,
        }
    }

    fn write(&self, path: &str, text: &str) -> Result<(), UseCaseError> {
        let condition = if self.files.exists(path) {
            WriteIf::Unchanged(JsonValue::new(&self.read(path)?).hash())
        } else {
            WriteIf::Absent
        };
        self.files
            .write(path, text, condition)
            .map_err(Instances::write_error)
    }
}

impl TranscriptionUseCases for Transcriptions {
    fn transcribe(&self, skill: &str) -> Result<Transcribed, UseCaseError> {
        let master = self.master_files()?;
        let (copy, _) = self.copy_files(skill)?;
        let record = self.record(skill)?;
        let dir = Self::copy_dir(skill);
        let mut done = Transcribed {
            from_version: record.version.clone(),
            to_version: Self::master_version(&master),
            written: Vec::new(),
            removed: Vec::new(),
            kept: Vec::new(),
            conflicts: Vec::new(),
        };
        let paths: BTreeSet<&String> = master.keys().chain(copy.keys()).collect();
        for path in paths {
            let (master_text, copy_text) = (master.get(path), copy.get(path));
            if master_text.is_some() && master_text == copy_text {
                continue;
            }
            let change = Self::change(master_text, copy_text, record.hashes.get(path));
            match (master_text, change.by_master, change.by_concrete) {
                // 具体が変えていなければ、基盤の版にそろえる
                (Some(text), _, false) => {
                    self.write(&format!("{dir}/{path}"), text)?;
                    done.written.push(path.clone());
                }
                (None, _, false) => {
                    self.files
                        .remove(&format!("{dir}/{path}"))
                        .map_err(Instances::write_error)?;
                    done.removed.push(path.clone());
                }
                // 具体だけが変えたものは、そのまま残す（具体の持ち物）
                (_, false, true) => done.kept.push(path.clone()),
                // 両方が変えたもの：具体のものを残し、基盤の新しい版を隣に置く
                (Some(text), true, true) => {
                    self.write(&format!("{dir}/{path}{CONFLICT_SUFFIX}"), text)?;
                    done.conflicts.push(path.clone());
                }
                // 基盤は消したが、具体が変えていたもの：具体のものを残す
                (None, true, true) => done.kept.push(path.clone()),
            }
        }
        let hashes: serde_json::Map<String, Value> = master
            .iter()
            .map(|(path, text)| (path.clone(), Value::String(hash_of(text))))
            .collect();
        let new_record = serde_json::to_string_pretty(&json!({
            "source": "schema-driven",
            "version": done.to_version,
            "files": hashes
        }))
        .unwrap_or_default()
            + "\n";
        let record_path = format!("{dir}/{RECORD}");
        let current_record = if self.files.exists(&record_path) {
            Some(self.read(&record_path)?)
        } else {
            None
        };
        if current_record.as_ref() != Some(&new_record) {
            self.write(&record_path, &new_record)?;
        }
        Ok(done)
    }

    fn check_copy(&self, skill: &str) -> Result<CopyReport, UseCaseError> {
        let master = self.master_files()?;
        let (copy, pending) = self.copy_files(skill)?;
        let record = self.record(skill)?;
        let paths: BTreeSet<&String> = master.keys().chain(copy.keys()).collect();
        let mut differences = Vec::new();
        for path in paths {
            let (master_text, copy_text) = (master.get(path), copy.get(path));
            if master_text.is_some() && master_text == copy_text {
                continue;
            }
            let recorded = record.hashes.get(path);
            let change = Self::change(master_text, copy_text, recorded);
            let kind = match (master_text, copy_text) {
                (Some(_), Some(_)) if change.by_master && change.by_concrete => "衝突",
                (Some(_), Some(_)) if change.by_concrete => "具体が変えた",
                (Some(_), Some(_)) => "正本が新しくなった",
                (Some(_), None) if recorded.is_none() => "正本に増えた",
                (Some(_), None) if change.by_master => "衝突",
                (Some(_), None) => "具体が消した",
                (None, Some(_)) if recorded.is_none() => "具体が足した",
                (None, Some(_)) if change.by_concrete => "衝突",
                (None, Some(_)) => "正本から消えた",
                (None, None) => continue,
            };
            differences.push(Difference {
                path: path.clone(),
                kind: kind.to_owned(),
            });
        }
        differences.extend(pending.into_iter().map(|path| Difference {
            path,
            kind: "衝突の新しい版が残っている".to_owned(),
        }));
        differences.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(CopyReport {
            copy_version: record.version,
            master_version: Self::master_version(&master),
            differences,
        })
    }
}
