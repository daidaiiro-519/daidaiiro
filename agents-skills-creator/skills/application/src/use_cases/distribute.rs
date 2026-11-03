//! UC5 配布する：skills distribute。配る前に確かめ（validate の検査と sas validate に、承認がいまの中身に当たるか
//! approval-current と、置き場所が空いているか destination-free を足す）、1つでも検出があれば置かない。検出が無ければ、
//! ツールと範囲ごとの置き場所に、Skill のフォルダへのシンボリックリンクを置き、置いた所から SKILL.md を読めるかを
//! 確かめる（placed-readable）。
//!
//! 置き場所：Claude Code は .claude/skills/<name>、Codex は .agents/skills/<name>。範囲が repository ならリポジトリの根
//! （打った場所）、user なら HOME の下。repository ではリンクの先を相対の経路にし、リポジトリを動かしても読めるようにする。
//! 最低保証：置き場所にほかのものがあれば置かない（同じ Skill へのリンクなら何もしない）。承認していない Skill は置かない（BR4）。

use crate::ports::{Destinations, JsonSchemaChecker, SkillFiles, SkillHistory, SkillValidator};
use crate::use_cases::validate::{Finding, Outcome, SkillChecked, finding, validate};
use crate::use_cases::{Refused, read_skill, trimmed};
use serde_json::Value;
use skills_domain::services::skill_md;

/// 配布先のツール。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Claude Code
    ClaudeCode,
    /// Codex
    Codex,
}

impl Tool {
    /// 名前から。
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    /// 置き場所のフォルダ（範囲の根から）。
    #[must_use]
    pub const fn dir(self) -> &'static str {
        match self {
            Self::ClaudeCode => ".claude/skills",
            Self::Codex => ".agents/skills",
        }
    }
}

/// 配布する範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// そのリポジトリ（打った場所）
    Repository,
    /// 利用者の全体（HOME）
    User,
}

impl Scope {
    /// 名前から。
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "repository" => Some(Self::Repository),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

/// 置いた結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    /// 置いた所
    pub path: String,
    /// リンクの先
    pub target: String,
    /// placed（置いた）・ already（同じリンクがもうあった）
    pub result: &'static str,
}

/// 配布した結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Distributed {
    /// 配る前の確かめ（validate の結果に approval-current と destination-free を足したもの）
    pub before: SkillChecked,
    /// 置いた結果（配る前に検出があれば None）
    pub placed: Option<Placed>,
    /// 置いたあとの確かめ（placed-readable）
    pub after: Vec<Finding>,
}

impl Distributed {
    /// 配布できなかったか（配る前の検出か、置いたあとに読めない）。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.placed.is_none() || self.after.iter().any(|f| f.outcome != Outcome::Verified)
    }
}

/// 承認の書き方（人が自分の端末で打つ）。
fn how_to_approve(folder: &str) -> String {
    format!(
        "作る人が skills validate の結果を承認したら、人が自分の端末で sas update {folder}/skill.json --patch '[{{\"op\":\"add\",\"path\":\"/approved\",\"value\":\"<承認した validate の操作 ID>\"}}]' --if-match <いまの中身のハッシュ> を打つ（AI のセッションから打つと、SAS が human_field で断る）"
    )
}

/// 承認（approved の validate の操作）のあとに、承認を書いた人の操作のほかに Skill を書き換えていないか。
fn approval_current(files: &dyn SkillFiles, history: &dyn SkillHistory, folder: &str) -> Finding {
    const CHECK: &str = "approval-current";
    let folder = trimmed(folder);
    let skill = match read_skill(files, folder) {
        Ok(v) => v,
        Err(r) => return finding(CHECK, Outcome::Error, None, r.message),
    };
    let Some(approved) = skill.get("approved").and_then(Value::as_str) else {
        return finding(
            CHECK,
            Outcome::Violated,
            None,
            format!(
                "承認していない（approved が無い）。{}",
                how_to_approve(folder)
            ),
        );
    };
    let rows: Vec<Value> = match history.log(&format!("{folder}/skill.json")) {
        Ok(r) if r.exit == 0 => match r.output {
            Value::Array(rows) => rows,
            other @ Value::Object(_) => vec![other],
            _ => Vec::new(),
        },
        Ok(r) => {
            return finding(
                CHECK,
                Outcome::Error,
                None,
                format!("sas log が断った：{}", r.output),
            );
        }
        Err(e) => return finding(CHECK, Outcome::Error, None, e),
    };
    let s = |v: &Value, k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    // sas log は新しい順。承認した確かめより新しい行を見る
    let Some(at) = rows
        .iter()
        .position(|r| s(r, "operation") == approved && s(r, "op") == "validate")
    else {
        return finding(
            CHECK,
            Outcome::Violated,
            Some(approved.to_string()),
            format!(
                "approved の {approved} が、この Skill の確かめ（validate の操作）の記録に無い。{}",
                how_to_approve(folder)
            ),
        );
    };
    for row in &rows[..at] {
        let op = s(row, "op");
        if !matches!(op.as_str(), "create" | "update" | "delete") {
            continue;
        }
        let id = s(row, "operation");
        let only_approval = op == "update"
            && row.get("session").is_none_or(Value::is_null)
            && history.operation(&id).ok().is_some_and(|d| {
                d.output
                    .pointer("/diff/changes")
                    .and_then(Value::as_array)
                    .is_some_and(|c| {
                        !c.is_empty()
                            && c.iter().all(|x| {
                                x.get("pointer").and_then(Value::as_str) == Some("/approved")
                            })
                    })
            });
        if !only_approval {
            let reason = row
                .get("reason")
                .and_then(Value::as_str)
                .map(|r| format!("、理由 {r}"))
                .unwrap_or_default();
            return finding(
                CHECK,
                Outcome::Violated,
                Some(id.clone()),
                format!(
                    "承認（{approved}）のあとに {id}（{op}{reason}）で書き換えた。承認は古い。もう一度確かめて承認する。{}",
                    how_to_approve(folder)
                ),
            );
        }
    }
    finding(
        CHECK,
        Outcome::Verified,
        None,
        format!("承認（{approved}）のあとに、承認を書いたほかに書き換えていない"),
    )
}

/// Skill を配布する。
///
/// # Errors
/// フォルダ ・ ツール ・ 範囲が無い（usage）・ 範囲が user で HOME が分からない（usage）・ sas を呼べない（sas_unavailable）・
/// 置けない（not_found）とき。
#[allow(clippy::too_many_arguments)]
pub fn distribute(
    folder: Option<&str>,
    tool: Option<Tool>,
    scope: Option<Scope>,
    sas: &dyn SkillValidator,
    history: &dyn SkillHistory,
    files: &dyn SkillFiles,
    schemas: &dyn JsonSchemaChecker,
    places: &dyn Destinations,
) -> Result<Distributed, Refused> {
    let usage = |m: &str| Refused {
        code: "usage",
        message: m.to_string(),
    };
    let folder = trimmed(folder.ok_or_else(|| usage("配布する Skill のフォルダを渡してください"))?);
    let tool = tool.ok_or_else(|| usage("--tool に claude-code か codex を渡してください"))?;
    let scope = scope.ok_or_else(|| usage("--scope に repository か user を渡してください"))?;
    let name = folder.rsplit('/').next().unwrap_or(folder);
    let (path, target) = match scope {
        Scope::Repository => {
            let up = "../".repeat(tool.dir().split('/').count());
            (format!("{}/{name}", tool.dir()), format!("{up}{folder}"))
        }
        Scope::User => {
            let home = places.home().ok_or_else(|| {
                usage("HOME が分からないので、利用者の全体の置き場所を決められない")
            })?;
            let target = places
                .canonical(folder)
                .unwrap_or_else(|| folder.to_string());
            (
                format!("{}/{}/{name}", home.trim_end_matches('/'), tool.dir()),
                target,
            )
        }
    };
    let mut before = validate(&[folder.to_string()], sas, history, files, schemas)?
        .skills
        .remove(0);
    before
        .creator
        .push(approval_current(files, history, folder));
    // 置き場所：何も無いか、同じ Skill へのリンクなら空いている
    let same = places.occupied(&path)
        && places.canonical(&path).is_some()
        && places.canonical(&path) == places.canonical(folder);
    let free = if !places.occupied(&path) {
        finding(
            "destination-free",
            Outcome::Verified,
            None,
            format!("{path} は空いている"),
        )
    } else if same {
        finding(
            "destination-free",
            Outcome::Verified,
            None,
            format!("{path} は同じ Skill へのリンク"),
        )
    } else {
        finding(
            "destination-free",
            Outcome::Violated,
            Some(path.clone()),
            format!(
                "{path} に、ほかのもの（同じ名前の別の Skill か、別の先へのリンク）がある。置かない"
            ),
        )
    };
    before.creator.push(free);
    if before.detected() {
        return Ok(Distributed {
            before,
            placed: None,
            after: Vec::new(),
        });
    }
    let result = if same {
        "already"
    } else {
        places.link(&path, &target).map_err(|message| Refused {
            code: "not_found",
            message,
        })?;
        "placed"
    };
    // 置いた所から、Skill のフォルダと同じ SKILL.md を読めるか
    let here = files.read(&format!("{path}/{}", skill_md::FILE));
    let source = files.read(&format!("{folder}/{}", skill_md::FILE));
    let readable = match (here, source) {
        (Ok(a), Ok(b)) if a == b => finding(
            "placed-readable",
            Outcome::Verified,
            None,
            format!("{path} から SKILL.md を読める"),
        ),
        (Ok(_), _) => finding(
            "placed-readable",
            Outcome::Violated,
            Some(path.clone()),
            format!("{path} の SKILL.md が、Skill のフォルダのものと違う"),
        ),
        (Err(e), _) => finding("placed-readable", Outcome::Violated, Some(path.clone()), e),
    };
    Ok(Distributed {
        before,
        placed: Some(Placed {
            path,
            target,
            result,
        }),
        after: vec![readable],
    })
}
