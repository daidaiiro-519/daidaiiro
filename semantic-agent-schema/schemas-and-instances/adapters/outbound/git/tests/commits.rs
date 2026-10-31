//! 本物の git で、一時のリポジトリからコミットにある中身を読む。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use schemas_and_instances_adapters_outbound_git::GitCommits;
use schemas_and_instances_conformance_application::ports::CommitReader;
use std::path::{Path, PathBuf};
use std::process::Command;

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
fn not_a_repository_has_no_versions() {
    let d = dir("none");
    let v = GitCommits::new(d.clone());
    assert!(v.commits("u/a.json", 10).unwrap().is_empty());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn repository_without_commits_has_no_versions() {
    let d = dir("empty");
    git(&d, &["init", "-q"]);
    std::fs::write(d.join("u/a.json"), "{}\n").unwrap();
    let v = GitCommits::new(d.clone());
    assert!(v.commits("u/a.json", 10).unwrap().is_empty());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn commits_are_newest_first_and_contents_are_read() {
    let d = dir("two");
    git(&d, &["init", "-q"]);
    std::fs::write(d.join("u/a.json"), "{\"v\": 1}\n").unwrap();
    git(&d, &["add", "u/a.json"]);
    git(&d, &["commit", "-q", "-m", "1"]);
    std::fs::write(d.join("u/a.json"), "{\"v\": 2}\n").unwrap();
    git(&d, &["commit", "-q", "-am", "2"]);
    std::fs::write(d.join("u/b.json"), "{}\n").unwrap();
    git(&d, &["add", "u/b.json"]);
    git(&d, &["commit", "-q", "-m", "3"]);
    let v = GitCommits::new(d.clone());
    let commits = v.commits("u/a.json", 10).unwrap();
    assert_eq!(commits.len(), 2, "a.json を変えたコミットだけ");
    assert_eq!(
        v.read(&commits[0], "u/a.json").unwrap().unwrap(),
        b"{\"v\": 2}\n"
    );
    assert_eq!(
        v.read(&commits[1], "u/a.json").unwrap().unwrap(),
        b"{\"v\": 1}\n"
    );
    assert_eq!(v.commits("u/a.json", 1).unwrap().len(), 1);
    assert!(
        v.read(&commits[1], "u/b.json").unwrap().is_none(),
        "そのコミットに無い"
    );
    std::fs::remove_dir_all(d).unwrap();
}
