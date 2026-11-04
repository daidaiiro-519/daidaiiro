//! 本物の git で、一時のリポジトリからコミットにある中身を読む。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use outbound_git::GitCommits;
use std::path::{Path, PathBuf};
use std::process::Command;
use writing_application::ports::CommitReader;

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("sas-git-{}-{name}", std::process::id()));
    if d.exists() {
        std::fs::remove_dir_all(&d).unwrap();
    }
    std::fs::create_dir_all(d.join("u")).unwrap();
    d
}

fn git(d: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(d)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

#[test]
fn not_a_repository_has_nothing_to_read() {
    let d = dir("none");
    let v = GitCommits::new(d.clone());
    assert!(v.read("HEAD", "u/a.json").unwrap().is_none());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn contents_are_read_from_the_given_commit() {
    let d = dir("two");
    git(&d, &["init", "-q"]);
    std::fs::write(d.join("u/a.json"), "{\"v\": 1}\n").unwrap();
    git(&d, &["add", "u/a.json"]);
    git(&d, &["commit", "-q", "-m", "1"]);
    std::fs::write(d.join("u/a.json"), "{\"v\": 2}\n").unwrap();
    git(&d, &["commit", "-q", "-am", "2"]);
    let v = GitCommits::new(d.clone());
    assert_eq!(
        v.read("HEAD", "u/a.json").unwrap().unwrap(),
        b"{\"v\": 2}\n"
    );
    assert_eq!(
        v.read("HEAD~1", "u/a.json").unwrap().unwrap(),
        b"{\"v\": 1}\n"
    );
    assert!(
        v.read("HEAD", "u/b.json").unwrap().is_none(),
        "そのコミットに無い"
    );
    assert!(v.read("no-such-commit", "u/a.json").unwrap().is_none());
    std::fs::remove_dir_all(d).unwrap();
}
