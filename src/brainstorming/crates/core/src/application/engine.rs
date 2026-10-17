//! 基盤のエンジン（scripts/bin/schema-driven）を、この Skill のフォルダを渡して呼ぶ。

use crate::application::{Env, Skill};
use crate::ports::{Clock, Files, Programs, Run};
use serde_json::Value;
use std::path::Path;

/// 基盤の操作の名前。brainstorming はこれらをそのままエンジンへ渡す（render は brainstorming の render が置き場所を決めて呼ぶ）。
pub const BASE: [&str; 8] = [
    "create",
    "get",
    "update",
    "delete",
    "prompt",
    "check",
    "check-schemas",
    "approve",
];

/// エンジンを `--skill <この Skill>` 付きで呼ぶ。
pub fn call<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    args: &[String],
) -> Result<Run, String> {
    if !env.files.exists(&skill.engine()) {
        return Err(format!(
            "基盤のエンジンが無い：{}",
            skill.engine().display()
        ));
    }
    let mut all = vec![
        "--skill".to_owned(),
        skill.dir.to_string_lossy().into_owned(),
    ];
    all.extend(args.iter().cloned());
    env.programs.run(&skill.engine(), &all, None)
}

/// インスタンス1件の、検証エラーと未記入の数。
fn instance_problems(item: &Value) -> usize {
    item.get("errors")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
        + item
            .get("unfilled")
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
}

/// check の結果のうち、合格でないもの（検証エラー ・ 未記入 ・ 合格でない検査）の数。
/// board が Some（ファイルと id）なら、そのボードのインスタンスと、そのボードから出た指摘だけを数える。読めなければ None。
pub fn problems(run: &Run, board: Option<(&Path, &str)>) -> Option<usize> {
    let value: Value = serde_json::from_str(&run.stdout).ok()?;
    let mine = |path: &str| board.is_none_or(|(file, _)| Path::new(path) == file);
    let from_mine = |from: &str| board.is_none_or(|(_, id)| from.split('/').next() == Some(id));
    let errors: usize = value
        .get("instances")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| mine(item.get("path").and_then(Value::as_str).unwrap_or("")))
        .map(instance_problems)
        .sum();
    let findings = value
        .get("findings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|finding| from_mine(finding.get("from").and_then(Value::as_str).unwrap_or("")))
        .filter(|finding| finding.get("status").and_then(Value::as_str) != Some("合格"))
        .count();
    (value.get("ok").and_then(Value::as_bool) == Some(true)).then_some(errors + findings)
}

/// check の結果のうち、検証エラーか未記入のあるインスタンスの場所。読めなければ None。
pub fn incomplete(run: &Run) -> Option<Vec<String>> {
    let value: Value = serde_json::from_str(&run.stdout).ok()?;
    (value.get("ok").and_then(Value::as_bool) == Some(true)).then_some(())?;
    Some(
        value
            .get("instances")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|item| instance_problems(item) > 0)
            .filter_map(|item| item.get("path").and_then(Value::as_str).map(str::to_owned))
            .collect(),
    )
}

fn check_run<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    dir: &Path,
) -> Result<Run, String> {
    call(
        env,
        skill,
        &[
            "check".to_owned(),
            "--dir".to_owned(),
            dir.to_string_lossy().into_owned(),
        ],
    )
}

/// ディレクトリを check し、合格でないものの数を返す。
pub fn check_dir<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    dir: &Path,
) -> Result<usize, String> {
    let run = check_run(env, skill, dir)?;
    problems(&run, None).ok_or_else(|| format!("check が失敗した：{}{}", run.stdout, run.stderr))
}

/// ボードの置き場所を check し、そのボードの分だけを数える。ほかのボードが作り途中でも止まらない。
pub fn check_board<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
    file: &Path,
    id: &str,
) -> Result<usize, String> {
    let run = check_run(env, skill, root)?;
    problems(&run, Some((file, id)))
        .ok_or_else(|| format!("check が失敗した：{}{}", run.stdout, run.stderr))
}

/// フォルダを check し、検証エラーか未記入のあるインスタンスの場所を返す。
pub fn incomplete_in<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    dir: &Path,
) -> Result<Vec<String>, String> {
    let run = check_run(env, skill, dir)?;
    incomplete(&run).ok_or_else(|| format!("check が失敗した：{}{}", run.stdout, run.stderr))
}

/// この Skill のスキーマの名前（brainstorming/board@v1 など）を、assets/ のファイルの場所にする。無ければ None。
pub fn schema_path<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    name: &str,
) -> Option<std::path::PathBuf> {
    env.files
        .list(&skill.dir.join("assets"))
        .into_iter()
        .filter(|path| path.to_string_lossy().ends_with(".schema.json"))
        .find(|path| {
            env.read_json(path)
                .ok()
                .and_then(|v| v.get("$id").and_then(Value::as_str).map(|id| id == name))
                == Some(true)
        })
}

/// 基盤の操作の引数のうち、--schema にこの Skill のスキーマの名前があれば、ファイルの場所に直す。
/// 基盤の create と prompt の --schema はファイルの場所を受けるので、SKILL.md は名前で書けるようにする。
pub fn resolve_args<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    args: &[String],
) -> Vec<String> {
    let mut out = args.to_vec();
    for i in 1..out.len() {
        if out[i - 1] == "--schema" && !env.files.exists(Path::new(&out[i])) {
            if let Some(path) = schema_path(env, skill, &out[i]) {
                out[i] = path.to_string_lossy().into_owned();
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(stdout: &str) -> Run {
        Run {
            code: 0,
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    #[test]
    fn problems_count_errors_unfilled_and_non_passing_findings() {
        let out = r#"{"ok":true,"instances":[{"errors":["e"],"unfilled":["u1","u2"]}],"findings":[{"status":"合格"},{"status":"ずれ"}]}"#;
        assert_eq!(problems(&run(out), None), Some(4));
        assert_eq!(
            problems(&run(r#"{"ok":true,"instances":[],"findings":[]}"#), None),
            Some(0)
        );
        assert_eq!(problems(&run(r#"{"ok":false}"#), None), None);
        assert_eq!(problems(&run("x"), None), None);
    }

    #[test]
    fn a_board_counts_only_its_own_instance_and_findings() {
        let out = r#"{"ok":true,"instances":[{"path":"/r/a.json","errors":[],"unfilled":[]},{"path":"/r/b.json","errors":[],"unfilled":["/id","/kind"]}],"findings":[{"status":"ずれ","from":"b/queue/0/topic"},{"status":"合格","from":"a/queue/0/topic"}]}"#;
        assert_eq!(
            problems(&run(out), Some((Path::new("/r/a.json"), "a"))),
            Some(0)
        );
        assert_eq!(
            problems(&run(out), Some((Path::new("/r/b.json"), "b"))),
            Some(3)
        );
        assert_eq!(problems(&run(out), None), Some(3));
        assert_eq!(incomplete(&run(out)), Some(vec!["/r/b.json".to_owned()]));
    }

    #[test]
    fn schema_names_of_this_skill_become_paths_and_others_stay() {
        let env = crate::tests::env(
            crate::tests::MemoryFiles::with(&[
                (
                    "/s/assets/board.schema.json",
                    r#"{"$id":"brainstorming/board@v1"}"#,
                ),
                ("/x/file.schema.json", "{}"),
            ]),
            crate::tests::passing_engine(),
        );
        let args = |list: &[&str]| {
            list.iter()
                .map(|a| (*a).to_owned())
                .collect::<Vec<String>>()
        };
        assert_eq!(
            resolve_args(
                &env,
                &crate::tests::skill(),
                &args(&[
                    "create",
                    "--schema",
                    "brainstorming/board@v1",
                    "--path",
                    "p"
                ])
            )[2],
            "/s/assets/board.schema.json"
        );
        assert_eq!(
            resolve_args(
                &env,
                &crate::tests::skill(),
                &args(&["create", "--schema", "/x/file.schema.json"])
            )[2],
            "/x/file.schema.json"
        );
        assert_eq!(
            resolve_args(
                &env,
                &crate::tests::skill(),
                &args(&["create", "--schema", "other/x@v1"])
            )[2],
            "other/x@v1"
        );
    }
}
