//! 起動時の結合。port の実装を application に渡す。結合はここ1か所に置く。

use crate::outbound::fs::RealFiles;
use crate::outbound::process::RealPrograms;
use crate::outbound::release::GhReleases;
use skills_creator_core::application::Env;
use std::path::PathBuf;

/// 本物の入出力で動く port の組。
pub fn env() -> Env<RealFiles, RealPrograms, GhReleases> {
    Env {
        files: RealFiles,
        programs: RealPrograms,
        releases: GhReleases,
    }
}

/// skills-creator 自身の Skill のフォルダ。実行ファイル（scripts/bin/skills-creator）の場所から決める。
pub fn own_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.parent()?.parent()?.to_path_buf()))
        .unwrap_or_default()
}
