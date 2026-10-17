//! 単体テストが共有する代役（port をメモリ上で実装したもの）。

use crate::application::{Env, Skill};
use crate::domain::agent::Plan;
use crate::ports::{Agents, Clock, Files, Next, Programs, Run, Running};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

/// メモリ上のファイル。複製しても同じ中身を指す。
#[derive(Clone, Default)]
pub struct MemoryFiles {
    inner: Rc<RefCell<BTreeMap<PathBuf, Vec<u8>>>>,
}

impl MemoryFiles {
    pub fn with(items: &[(&str, &str)]) -> Self {
        let files = Self::default();
        items.iter().for_each(|(path, text)| {
            files
                .inner
                .borrow_mut()
                .insert(PathBuf::from(path), text.as_bytes().to_vec());
        });
        files
    }
    pub fn paths(&self) -> Vec<PathBuf> {
        self.inner.borrow().keys().cloned().collect()
    }
}

impl Files for MemoryFiles {
    fn read(&self, path: &Path) -> Result<String, String> {
        self.read_bytes(path)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.inner
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{}：無い", path.display()))
    }
    fn write(&self, path: &Path, text: &str) -> Result<(), String> {
        self.inner
            .borrow_mut()
            .insert(path.to_path_buf(), text.as_bytes().to_vec());
        Ok(())
    }
    fn create_new(&self, path: &Path, text: &str) -> Result<bool, String> {
        if self.inner.borrow().contains_key(path) {
            return Ok(false);
        }
        self.write(path, text).map(|_| true)
    }
    fn exists(&self, path: &Path) -> bool {
        self.inner
            .borrow()
            .keys()
            .any(|p| p == path || p.starts_with(path))
    }
    fn list(&self, dir: &Path) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = self
            .inner
            .borrow()
            .keys()
            .filter_map(|p| {
                p.strip_prefix(dir)
                    .ok()
                    .and_then(|rest| rest.components().next())
                    .map(|first| dir.join(first))
            })
            .collect();
        out.dedup();
        out
    }
    fn tree(&self, dir: &Path) -> Vec<PathBuf> {
        self.inner
            .borrow()
            .keys()
            .filter(|p| p.starts_with(dir))
            .cloned()
            .collect()
    }
    fn remove_file(&self, path: &Path) -> Result<(), String> {
        self.inner
            .borrow_mut()
            .remove(path)
            .map(|_| ())
            .ok_or(format!("{}：無い", path.display()))
    }
    fn remove_dir(&self, dir: &Path) -> Result<(), String> {
        self.inner.borrow_mut().retain(|p, _| !p.starts_with(dir));
        Ok(())
    }
}

/// 起動を記録し、決まった答えを返すプログラム。git は、メモリの中のファイルをすべて「追跡していない」と答える。
#[derive(Clone)]
pub struct FakePrograms {
    check: &'static str,
    files: MemoryFiles,
    calls: Rc<RefCell<Vec<Vec<String>>>>,
    git: bool,
}

impl FakePrograms {
    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }
}

impl Programs for FakePrograms {
    fn run(&self, program: &Path, args: &[String], cwd: Option<&Path>) -> Result<Run, String> {
        self.calls.borrow_mut().push(args.to_vec());
        if program == Path::new("git") {
            let root = cwd.unwrap_or(Path::new("/"));
            let listed: String = self
                .files
                .paths()
                .iter()
                .filter_map(|p| p.strip_prefix(root).ok())
                .map(|p| format!("?? {}\0", p.display()))
                .collect();
            return Ok(Run {
                code: if self.git { 0 } else { 128 },
                stdout: listed,
                stderr: String::new(),
            });
        }
        let stdout = match args.get(2).map(String::as_str) {
            Some("render") => r#"{"ok":true,"pages":[]}"#.to_owned(),
            _ => self.check.to_owned(),
        };
        Ok(Run {
            code: 0,
            stdout,
            stderr: String::new(),
        })
    }
}

const PASS: &str = r#"{"ok":true,"instances":[],"findings":[]}"#;
const FAIL: &str = r#"{"ok":true,"instances":[{"errors":["e"],"unfilled":[]}],"findings":[]}"#;

pub fn passing_engine() -> (&'static str, bool) {
    (PASS, false)
}
pub fn failing_engine() -> (&'static str, bool) {
    (FAIL, false)
}

/// 決まった時刻を返し、呼ぶたびに経過を1秒進め、UUID を U1 ・ U2 … と返す時計。
#[derive(Default)]
pub struct FakeClock {
    ticks: Cell<u64>,
    ids: Cell<u64>,
}

impl Clock for FakeClock {
    fn now(&self) -> String {
        "2026-10-17T01:02:03Z".to_owned()
    }
    fn today(&self) -> String {
        "2026-10-17".to_owned()
    }
    fn elapsed_ms(&self) -> u64 {
        self.ticks.set(self.ticks.get() + 1000);
        self.ticks.get()
    }
    fn uuid(&self) -> String {
        self.ids.set(self.ids.get() + 1);
        format!("U{}", self.ids.get())
    }
    fn epoch_secs(&self) -> u64 {
        EPOCH
    }
}

/// FakeClock の epoch_secs。
pub const EPOCH: u64 = 1_792_200_000;

pub type Env3 = Env<MemoryFiles, FakePrograms, FakeClock>;

fn make(files: MemoryFiles, (check, git): (&'static str, bool)) -> Env3 {
    files
        .write(Path::new("/s/scripts/bin/schema-driven"), "engine")
        .unwrap_or(());
    Env {
        programs: FakePrograms {
            check,
            files: files.clone(),
            calls: Rc::default(),
            git,
        },
        files,
        clock: FakeClock::default(),
    }
}

pub fn env(files: MemoryFiles, engine: (&'static str, bool)) -> Env3 {
    make(files, engine)
}

/// check が path のインスタンスに未記入があると答えるエンジン。
pub fn env_incomplete(files: MemoryFiles, path: &str) -> Env3 {
    let out: &'static str = Box::leak(format!(r#"{{"ok":true,"instances":[{{"path":"{path}","errors":[],"unfilled":["/kind"]}}],"findings":[]}}"#).into_boxed_str());
    make(files, (out, false))
}

pub fn env_with_git(files: MemoryFiles) -> Env3 {
    make(files, (PASS, true))
}

pub fn skill() -> Skill {
    Skill::new(Path::new("/s"))
}

/// 決まった行を返す AI の CLI。started で、最後に起動したときの Plan を返す。
pub struct FakeAgents {
    lines: Vec<String>,
    code: i32,
    quiet: bool,
    writes: Vec<(String, String)>,
    files: Option<MemoryFiles>,
    last: RefCell<Option<Plan>>,
}

impl FakeAgents {
    pub fn new(lines: &[&str], code: i32) -> Self {
        Self {
            lines: lines.iter().map(|l| (*l).to_owned()).collect(),
            code,
            quiet: false,
            writes: vec![],
            files: None,
            last: RefCell::new(None),
        }
    }
    pub fn quiet() -> Self {
        Self {
            quiet: true,
            ..Self::new(&[], 0)
        }
    }
    pub fn writing(files: &MemoryFiles, writes: &[(&str, &str)]) -> Self {
        Self {
            writes: writes
                .iter()
                .map(|(p, t)| ((*p).to_owned(), (*t).to_owned()))
                .collect(),
            files: Some(files.clone()),
            ..Self::new(&[], 0)
        }
    }
    pub fn started(&self) -> Plan {
        self.last.borrow().clone().unwrap_or(Plan {
            program: String::new(),
            args: vec![],
            stdin: None,
            env: vec![],
        })
    }
}

pub struct FakeRun {
    lines: Vec<String>,
    code: i32,
    quiet: bool,
}

impl Running for FakeRun {
    fn next(&mut self, _wait: Duration) -> Next {
        match (self.quiet, self.lines.is_empty()) {
            (true, _) => Next::Quiet,
            (false, true) => Next::Ended(self.code, "tail".to_owned()),
            (false, false) => Next::Line(self.lines.remove(0)),
        }
    }
    fn stop(&mut self) {
        self.quiet = false;
        self.lines.clear();
    }
}

impl Agents for FakeAgents {
    type Started = FakeRun;
    fn start(&self, plan: &Plan, _cwd: &Path) -> Result<FakeRun, String> {
        *self.last.borrow_mut() = Some(plan.clone());
        if let Some(files) = &self.files {
            self.writes
                .iter()
                .for_each(|(p, t)| files.write(Path::new(p), t).unwrap_or(()));
        }
        Ok(FakeRun {
            lines: self.lines.clone(),
            code: self.code,
            quiet: self.quiet,
        })
    }
    fn found(&self, _program: &str) -> bool {
        true
    }
}
