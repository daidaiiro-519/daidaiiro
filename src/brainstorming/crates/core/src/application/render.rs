//! 描画のユースケース。.brainstorming/ のボードとダッシュボードを、この Skill のページテンプレートとカタログで site/ へ描く。
//! ダッシュボードの印（dashboard.json）が無ければ置く。

use crate::application::engine::{call, incomplete_in};
use crate::application::{Env, Skill};
use crate::domain::board::{pretty, Layout};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::path::Path;

/// ダッシュボードの印。中身は描画のときに全部のボードから数える。
pub fn dashboard() -> Value {
    json!({"$schema": "brainstorming/dashboard@v1", "kind": "dashboard", "id": "dashboard", "title": "ブレインストーミング"})
}

/// root（.brainstorming/）を描く。作り途中のボード（検証エラーか未記入があるもの）は描かずに skipped で知らせ、ほかは描く。
/// 基盤の render は、形の崩れたインスタンスが1件でもあると1ページも書かないので、そのときは形の揃ったものだけを写した所から描く。
pub fn run<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
) -> Result<Value, String> {
    let layout = Layout::new(root);
    if !env.files.exists(&layout.dashboard()) {
        env.files
            .write(&layout.dashboard(), &pretty(&dashboard()))?;
    }
    let skipped = incomplete_in(env, skill, root)?;
    let source = if skipped.is_empty() {
        root.to_path_buf()
    } else {
        let stage = layout.root.join(".serve").join("render");
        env.files.remove_dir(&stage)?;
        for path in env.files.list(root) {
            let text = path.to_string_lossy();
            if text.ends_with(".json") && !skipped.iter().any(|s| Path::new(s) == path) {
                if let Some(name) = path.file_name() {
                    env.files
                        .write(&stage.join(name), &env.files.read(&path)?)?;
                }
            }
        }
        stage
    };
    let args = [
        "render".to_owned(),
        "--dir".to_owned(),
        source.to_string_lossy().into_owned(),
        "--pages".to_owned(),
        skill.pages().to_string_lossy().into_owned(),
        "--out".to_owned(),
        layout.site().to_string_lossy().into_owned(),
    ];
    let run = call(env, skill, &args)?;
    let mut value: Value = serde_json::from_str(&run.stdout)
        .map_err(|_| format!("render の出力を読めない：{}{}", run.stdout, run.stderr))?;
    match value.get("ok").and_then(Value::as_bool) {
        Some(true) => {
            value["skipped"] = json!(skipped);
            Ok(value)
        }
        _ => Err(format!("render が描かなかった：{}", run.stdout.trim())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, passing_engine, skill, MemoryFiles};

    #[test]
    fn rendering_places_the_dashboard_and_calls_the_engine_with_the_pages_and_site() {
        let env = env(MemoryFiles::with(&[]), passing_engine());
        run(&env, &skill(), Path::new("/r/.brainstorming")).unwrap();
        assert!(env
            .files
            .exists(Path::new("/r/.brainstorming/dashboard.json")));
        let calls = env.programs.calls();
        assert_eq!(
            calls.last().unwrap()[2..],
            [
                "render",
                "--dir",
                "/r/.brainstorming",
                "--pages",
                "/s/assets/pages",
                "--out",
                "/r/.brainstorming/site"
            ]
        );
    }

    #[test]
    fn an_incomplete_board_is_skipped_and_the_rest_are_drawn_from_a_stage() {
        let files = MemoryFiles::with(&[
            ("/r/.brainstorming/a.json", "{}"),
            ("/r/.brainstorming/b.json", "{}"),
        ]);
        let env = crate::tests::env_incomplete(files, "/r/.brainstorming/b.json");
        let out = run(&env, &skill(), Path::new("/r/.brainstorming")).unwrap();
        assert_eq!(out["skipped"], json!(["/r/.brainstorming/b.json"]));
        assert!(env
            .files
            .exists(Path::new("/r/.brainstorming/.serve/render/a.json")));
        assert!(!env
            .files
            .exists(Path::new("/r/.brainstorming/.serve/render/b.json")));
        let calls = env.programs.calls();
        assert_eq!(calls.last().unwrap()[4], "/r/.brainstorming/.serve/render");
    }
}
