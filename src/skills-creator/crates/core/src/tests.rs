//! 単体テストが共有する代役。port をメモリ上で実装する。

use crate::ports::{Files, Programs, Releases, Run};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// メモリ上のファイル。フォルダは、作ったものとファイルの親を在るものとする。写しても同じ中身を指す。
#[derive(Default, Clone)]
pub struct MemFiles {
    pub files: Rc<RefCell<BTreeMap<PathBuf, Vec<u8>>>>,
    pub dirs: Rc<RefCell<BTreeSet<PathBuf>>>,
    pub executable: Rc<RefCell<BTreeSet<PathBuf>>>,
    /// この経路への書き込み ・ 写しを失敗させる（失敗したときに元のまま残るかを確かめる）
    pub fail_on: Rc<RefCell<Option<PathBuf>>>,
}

impl MemFiles {
    pub fn put(&self, path: &str, text: &str) {
        self.files
            .borrow_mut()
            .insert(PathBuf::from(path), text.as_bytes().to_vec());
    }

    pub fn text(&self, path: &str) -> Option<String> {
        self.files
            .borrow()
            .get(Path::new(path))
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    }

    /// prefix の下にあるファイルの経路の一覧。
    pub fn under(&self, prefix: &str) -> Vec<String> {
        self.files
            .borrow()
            .keys()
            .filter(|path| path.starts_with(prefix))
            .map(|path| path.to_string_lossy().into_owned())
            .collect()
    }

    fn blocked(&self, path: &Path) -> Result<(), String> {
        match &*self.fail_on.borrow() {
            Some(blocked) if blocked == path => {
                Err(format!("{} に書けない（試験）", path.display()))
            }
            _ => Ok(()),
        }
    }
}

impl Files for MemFiles {
    fn read(&self, path: &Path) -> Result<String, String> {
        self.read_bytes(path)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.files
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{} が無い", path.display()))
    }
    fn write(&self, path: &Path, text: &str) -> Result<(), String> {
        self.blocked(path)?;
        self.files
            .borrow_mut()
            .insert(path.to_path_buf(), text.as_bytes().to_vec());
        Ok(())
    }
    fn exists(&self, path: &Path) -> bool {
        self.files.borrow().contains_key(path)
            || self.dirs.borrow().contains(path)
            || self
                .files
                .borrow()
                .keys()
                .any(|file| file.starts_with(path))
    }
    fn list(&self, dir: &Path) -> Vec<PathBuf> {
        let mut found: BTreeSet<PathBuf> = BTreeSet::new();
        self.files
            .borrow()
            .keys()
            .chain(self.dirs.borrow().iter())
            .filter_map(|path| path.strip_prefix(dir).ok())
            .filter_map(|rest| rest.components().next())
            .for_each(|first| {
                found.insert(dir.join(first));
            });
        found.into_iter().collect()
    }
    fn create_dir(&self, dir: &Path) -> Result<(), String> {
        self.dirs.borrow_mut().insert(dir.to_path_buf());
        Ok(())
    }
    fn copy(&self, from: &Path, to: &Path) -> Result<(), String> {
        self.blocked(to)?;
        let bytes = self.read_bytes(from)?;
        self.files.borrow_mut().insert(to.to_path_buf(), bytes);
        Ok(())
    }
    fn rename(&self, from: &Path, to: &Path) -> Result<(), String> {
        let moved: Vec<(PathBuf, Vec<u8>)> = self
            .files
            .borrow()
            .iter()
            .filter(|(path, _)| path.starts_with(from))
            .map(|(path, bytes)| {
                let rest = path.strip_prefix(from).unwrap();
                let moved = match rest.as_os_str().is_empty() {
                    true => to.to_path_buf(),
                    false => to.join(rest),
                };
                (moved, bytes.clone())
            })
            .collect();
        self.files
            .borrow_mut()
            .retain(|path, _| !path.starts_with(from));
        self.dirs
            .borrow_mut()
            .retain(|path| !path.starts_with(from));
        self.dirs.borrow_mut().insert(to.to_path_buf());
        self.files.borrow_mut().extend(moved);
        Ok(())
    }
    fn remove_dir(&self, dir: &Path) -> Result<(), String> {
        self.files
            .borrow_mut()
            .retain(|path, _| !path.starts_with(dir));
        self.dirs.borrow_mut().retain(|path| !path.starts_with(dir));
        Ok(())
    }
    fn remove_file(&self, path: &Path) -> Result<(), String> {
        self.files.borrow_mut().remove(path);
        Ok(())
    }
    fn set_executable(&self, path: &Path) -> Result<(), String> {
        self.executable.borrow_mut().insert(path.to_path_buf());
        Ok(())
    }
    fn temp_dir(&self) -> Result<PathBuf, String> {
        let dir = PathBuf::from("/tmp/work");
        self.dirs.borrow_mut().insert(dir.clone());
        Ok(dir)
    }
}

/// 決まった答えを返すプログラム。キーは「プログラムの経路 + 引数を空白でつないだもの」。
#[derive(Default)]
pub struct FakePrograms {
    pub answers: RefCell<BTreeMap<String, Run>>,
    pub calls: RefCell<Vec<String>>,
}

impl FakePrograms {
    pub fn answer(&self, program: &str, args: &str, code: i32, stdout: &str) {
        self.answers.borrow_mut().insert(
            format!("{program} {args}"),
            Run {
                code,
                stdout: stdout.to_owned(),
                stderr: String::new(),
            },
        );
    }
}

impl Programs for FakePrograms {
    fn run(&self, program: &Path, args: &[String]) -> Result<Run, String> {
        let key = format!("{} {}", program.display(), args.join(" "));
        self.calls.borrow_mut().push(key.clone());
        self.answers
            .borrow()
            .get(&key)
            .cloned()
            .ok_or(format!("{key} を起動できない（試験）"))
    }
}

/// 配布物を取る代わりに、決まったファイルを置く Release。
#[derive(Default)]
pub struct FakeReleases {
    /// download で into の下に置くファイル（名前 → 中身）
    pub assets: RefCell<BTreeMap<String, Vec<u8>>>,
    /// extract で into の下に置くファイル（into からの相対パス → 中身）
    pub unpacked: RefCell<BTreeMap<String, Vec<u8>>>,
    /// 証明の照合を失敗させる
    pub attest_fails: RefCell<bool>,
    pub files: MemFiles,
}

impl Releases for FakeReleases {
    fn download(&self, tag: &str, names: &[String], into: &Path) -> Result<(), String> {
        names.iter().try_for_each(|name| {
            let bytes = self
                .assets
                .borrow()
                .get(name)
                .cloned()
                .ok_or(format!("{tag} に {name} が無い"))?;
            self.files.files.borrow_mut().insert(into.join(name), bytes);
            Ok(())
        })
    }
    fn attest(&self, _file: &Path, _bundle: &Path, _tag: &str) -> Result<(), String> {
        match *self.attest_fails.borrow() {
            true => Err("証明が一致しない（試験）".to_owned()),
            false => Ok(()),
        }
    }
    fn extract(&self, _archive: &Path, into: &Path) -> Result<(), String> {
        self.unpacked.borrow().iter().for_each(|(path, bytes)| {
            self.files
                .files
                .borrow_mut()
                .insert(into.join(path), bytes.clone());
        });
        Ok(())
    }
}
