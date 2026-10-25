//! 結合テスト：一時のリポジトリで、前回のコミットからの前後を読む

use semantic_agent_schema_application::ports::changes::Changes;
use semantic_agent_schema_git::changes::Git;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "git {args:?}");
}

fn repo(name: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("sas-git-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    git(&dir, &["init", "-q"]);
    Ok(dir)
}

/// traces: U25:事後
#[test]
fn changed_file_has_before_and_after() {
    let dir = repo("changed").unwrap();
    std::fs::write(dir.join("a.json"), "{\"why\":\"a\"}").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "first"]);
    std::fs::write(dir.join("a.json"), "{\"why\":\"b\"}").unwrap();
    std::fs::write(dir.join("new.json"), "{}").unwrap();
    let mut got = Git::new(dir).changed().unwrap();
    got.sort_by(|x, y| x.path.cmp(&y.path));
    let shown: Vec<(&str, Option<&str>, Option<&str>)> = got
        .iter()
        .map(|f| (f.path.as_str(), f.before.as_deref(), f.after.as_deref()))
        .collect();
    assert_eq!(
        shown,
        vec![
            ("a.json", Some("{\"why\":\"a\"}"), Some("{\"why\":\"b\"}")),
            ("new.json", None, Some("{}"))
        ]
    );
}

/// traces: U25:事後
#[test]
fn before_the_first_commit_there_is_no_head() {
    let dir = repo("empty").unwrap();
    std::fs::write(dir.join("a.json"), "{}").unwrap();
    let g = Git::new(dir);
    assert_eq!((g.head().unwrap(), g.changed().unwrap().len()), (None, 1));
}

/// traces: U25:事後
#[test]
fn shared_dir_is_the_common_git_dir() {
    let dir = repo("shared").unwrap();
    assert!(
        Git::new(dir)
            .shared_dir()
            .is_some_and(|p| p.ends_with(".git"))
    );
}

/// traces: I35
#[test]
fn each_worktree_has_its_own_workspace() {
    let dir = repo("worktree").unwrap();
    std::fs::write(dir.join("a.json"), "{}").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "first"]);
    git(&dir, &["worktree", "add", "-q", "-b", "x", "wt-x"]);
    let main = Git::new(dir.clone()).workspace().unwrap();
    let other = Git::new(dir.join("wt-x")).workspace().unwrap();
    assert_eq!(
        (main.as_deref(), other.as_deref()),
        (Some("main"), Some("worktrees/wt-x"))
    );
}
