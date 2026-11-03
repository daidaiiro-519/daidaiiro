//! ポート DocumentStore（書く ・ 検証 ・ 描画 ・ 検索のモジュールがそれぞれ宣言する）を、作業ディレクトリのファイルで実装する。
//! 描いたページの書き出し先（PageWriter）も持つ。書くときは一時ファイルに書いてから置き換える。リポジトリの外と、降りないフォルダには書かない。

use rendering_application::ports::{PageRefusal, PageWriter};
use std::path::{Path, PathBuf};

/// 降りないフォルダ（前の版が Git の外で記録を置いていた .sas ・ ビルドの出力 ・ Git の中身。いまの記録は .git の中の共通フォルダにある）。
const SKIP: [&str; 3] = [".sas", "target", ".git"];

/// 作業ディレクトリを根にした文書の置き場所。
#[derive(Debug, Clone)]
pub struct WorkingTree {
    root: PathBuf,
}

impl WorkingTree {
    /// root を根にする。
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("{} を読めない：{e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let here = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            if !SKIP.contains(&name.as_str()) {
                walk(&entry.path(), &here, out)?;
            }
        } else if kind.is_file() && name.ends_with(".json") {
            out.push(here);
        }
    }
    Ok(())
}

impl WorkingTree {
    /// 作業ディレクトリの JSON の文書の経路（降りないフォルダを除き、経路の順）。
    ///
    /// # Errors
    /// フォルダを読めないとき。
    pub fn list(&self) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        walk(&self.root, "", &mut out)?;
        out.sort();
        Ok(out)
    }

    /// 文書の中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    pub fn read(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(path)).map_err(|e| format!("{path} を読めない：{e}"))
    }

    /// 文書を書く（一時ファイルに書いてから置き換える）。
    ///
    /// # Errors
    /// 経路がリポジトリの外か降りないフォルダを指す ・ 書けないとき。
    pub fn write(&self, path: &str, text: &str) -> Result<(), String> {
        inside(path, "書かない")?;
        let to = self.root.join(path);
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("{} を作れない：{e}", dir.display()))?;
        }
        let tmp = to.with_file_name(format!(
            ".{}.sas-tmp",
            to.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        ));
        std::fs::write(&tmp, text).map_err(|e| format!("{path} を書けない：{e}"))?;
        std::fs::rename(&tmp, &to).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("{path} を置き換えられない：{e}")
        })
    }

    /// 文書を消す。
    ///
    /// # Errors
    /// 経路がリポジトリの外か降りないフォルダを指す ・ 無い ・ 消せないとき。
    pub fn remove(&self, path: &str) -> Result<(), String> {
        inside(path, "消さない")?;
        std::fs::remove_file(self.root.join(path)).map_err(|e| format!("{path} を消せない：{e}"))
    }
}

impl writing_application::ports::DocumentStore for WorkingTree {
    fn list(&self) -> Result<Vec<String>, String> {
        Self::list(self)
    }

    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }

    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        Self::write(self, path, text)
    }

    fn remove(&self, path: &str) -> Result<(), String> {
        Self::remove(self, path)
    }
}

impl validation_application::ports::DocumentStore for WorkingTree {
    fn list(&self) -> Result<Vec<String>, String> {
        Self::list(self)
    }

    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }
}

impl rendering_application::ports::DocumentStore for WorkingTree {
    fn list(&self) -> Result<Vec<String>, String> {
        Self::list(self)
    }

    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }
}

impl search_application::ports::DocumentStore for WorkingTree {
    fn list(&self) -> Result<Vec<String>, String> {
        Self::list(self)
    }

    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }
}

/// 書く ・ 消す経路が、リポジトリの根からの相対で、降りないフォルダの中でないか。
fn inside(path: &str, verb: &str) -> Result<(), String> {
    let parts: Vec<&str> = path.split('/').collect();
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || parts
            .iter()
            .any(|p| p.is_empty() || *p == "." || *p == "..")
        || parts.iter().any(|p| SKIP.contains(p))
    {
        return Err(format!(
            "{path} には{verb}（リポジトリの根からの相対の経路で、{} の中でないものだけ）",
            SKIP.join(" ・ ")
        ));
    }
    Ok(())
}

/// 描いたページの書き出し先（作業ディレクトリからの相対か、絶対の経路）。リポジトリの外にも書ける。
#[derive(Debug, Clone)]
pub struct OutDir {
    root: PathBuf,
}

impl OutDir {
    /// 相対の経路は root（作業ディレクトリ）から引く。
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

/// まだ無いかもしれないファイルの実体の経路（在るフォルダまでを実体にし、残りを足す）。
fn real_path(p: &Path) -> PathBuf {
    if let Ok(real) = std::fs::canonicalize(p) {
        return real;
    }
    match (p.parent(), p.file_name()) {
        (Some(dir), Some(name)) if !dir.as_os_str().is_empty() => real_path(dir).join(name),
        _ => p.to_path_buf(),
    }
}

impl PageWriter for OutDir {
    /// 入力と重なるページが無いことを実体の経路で確かめてから、全部のページを一時ファイルに書く。
    /// 全部書けたら、前から在ったページを退避して置き換える。途中で失敗したら、置き換えたページを退避から戻し、
    /// 一時ファイルと、このとき作ったフォルダを消す。
    fn write_pages(
        &self,
        out: &str,
        pages: &[(String, String)],
        inputs: &[String],
    ) -> Result<(), PageRefusal> {
        let base = self.root.join(out);
        let real_inputs: Vec<(PathBuf, &String)> = inputs
            .iter()
            .map(|i| (real_path(&self.root.join(i)), i))
            .collect();
        for (rel, _) in pages {
            if rel
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
            {
                return Err(PageRefusal::Failed(format!("{rel} は書き出す先の外を指す")));
            }
            let page = real_path(&base.join(rel));
            if let Some((_, input)) = real_inputs.iter().find(|(r, _)| *r == page) {
                return Err(PageRefusal::Overlaps {
                    page: rel.clone(),
                    input: (*input).clone(),
                });
            }
        }
        let mut made: Vec<PathBuf> = Vec::new();
        let mut staged: Vec<(PathBuf, PathBuf)> = Vec::new();
        let undo = |staged: &[(PathBuf, PathBuf)], made: &[PathBuf]| {
            for (tmp, _) in staged {
                let _ = std::fs::remove_file(tmp);
            }
            for dir in made.iter().rev() {
                let _ = std::fs::remove_dir(dir);
            }
        };
        for (rel, html) in pages {
            let to = base.join(rel);
            if to.exists() && !to.is_file() {
                undo(&staged, &made);
                return Err(PageRefusal::Failed(format!(
                    "{} はファイルではないので、ページで置き換えない",
                    to.display()
                )));
            }
            if let Some(dir) = to.parent() {
                let mut missing: Vec<PathBuf> = dir
                    .ancestors()
                    .take_while(|a| !a.as_os_str().is_empty() && !a.exists())
                    .map(Path::to_path_buf)
                    .collect();
                if let Err(e) = std::fs::create_dir_all(dir) {
                    undo(&staged, &made);
                    return Err(PageRefusal::Failed(format!(
                        "{} を作れない：{e}",
                        dir.display()
                    )));
                }
                missing.reverse();
                made.extend(missing);
            }
            let tmp = sibling(&to, "sas-tmp");
            if let Err(e) = std::fs::write(&tmp, html) {
                undo(&staged, &made);
                return Err(PageRefusal::Failed(format!(
                    "{} を書けない：{e}",
                    to.display()
                )));
            }
            staged.push((tmp, to));
        }
        let mut replaced: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
        for (i, (tmp, to)) in staged.iter().enumerate() {
            let backup = sibling(to, "sas-bak");
            let had = to.exists();
            let moved = if had {
                std::fs::rename(to, &backup)
            } else {
                Ok(())
            };
            let moved_ok = had && moved.is_ok();
            let result = moved.and_then(|()| std::fs::rename(tmp, to));
            if let Err(e) = result {
                // 自分で退避したときだけ戻す（退避に失敗したなら、前のページはそのまま在る）
                if moved_ok {
                    let _ = std::fs::rename(&backup, to);
                }
                for (page, saved) in replaced.iter().rev() {
                    match saved {
                        Some(b) => {
                            let _ = std::fs::rename(b, page);
                        }
                        None => {
                            let _ = std::fs::remove_file(page);
                        }
                    }
                }
                undo(&staged[i..], &made);
                return Err(PageRefusal::Failed(format!(
                    "{} を置き換えられない：{e}",
                    to.display()
                )));
            }
            replaced.push((to.clone(), had.then_some(backup)));
        }
        // 書き終えたら、置き換えたページの隣の退避を消す。前の書き出しが置き換えの途中で止められて残った退避
        // （前のページが無く、退避だけがある）も、ここで消える
        for (page, _) in replaced {
            let _ = std::fs::remove_file(sibling(&page, "sas-bak"));
        }
        Ok(())
    }
}

/// 同じフォルダの、隠した作業用のファイル（.<名前>.<印>）。
fn sibling(to: &Path, mark: &str) -> PathBuf {
    to.with_file_name(format!(
        ".{}.{mark}",
        to.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    ))
}
