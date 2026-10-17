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

/// check の結果のうち、合格でないもの（検証エラー ・ 未記入 ・ 合格でない検査）の数。読めなければ None。
pub fn check_problems(run: &Run) -> Option<usize> {
    let value: Value = serde_json::from_str(&run.stdout).ok()?;
    let errors: usize = value
        .get("instances")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|item| {
            item.get("errors")
                .and_then(Value::as_array)
                .map_or(0, Vec::len)
                + item
                    .get("unfilled")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len)
        })
        .sum();
    let findings = value
        .get("findings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|finding| finding.get("status").and_then(Value::as_str) != Some("合格"))
        .count();
    (value.get("ok").and_then(Value::as_bool) == Some(true)).then_some(errors + findings)
}

/// ディレクトリを check し、合格でないものの数を返す。
pub fn check_dir<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    dir: &Path,
) -> Result<usize, String> {
    let run = call(
        env,
        skill,
        &[
            "check".to_owned(),
            "--dir".to_owned(),
            dir.to_string_lossy().into_owned(),
        ],
    )?;
    check_problems(&run).ok_or_else(|| format!("check が失敗した：{}{}", run.stdout, run.stderr))
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
        assert_eq!(check_problems(&run(out)), Some(4));
        assert_eq!(
            check_problems(&run(r#"{"ok":true,"instances":[],"findings":[]}"#)),
            Some(0)
        );
        assert_eq!(check_problems(&run(r#"{"ok":false}"#)), None);
        assert_eq!(check_problems(&run("x")), None);
    }
}
