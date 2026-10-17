//! トークンを移すユースケース。Design Tokens 形式の正本（light.tokens.json ・ dark.tokens.json）から design-system.json を作る。
//! check のときは書かずに、いまの design-system.json と同じかだけを見る。

use crate::application::{Env, Skill};
use crate::domain::board::pretty;
use crate::domain::tokens::design_system;
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};

/// 移す。check が true なら書かずに比べる。
pub fn run<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    check: bool,
) -> Result<Value, String> {
    let dir = skill.catalog();
    let light = env.read_json(&dir.join("light.tokens.json"))?;
    let dark = env.read_json(&dir.join("dark.tokens.json"))?;
    let value = design_system(&light, &dark)?;
    let made = pretty(&value);
    let target = dir.join("design-system.json");
    let current = env.files.read(&target).unwrap_or_default();
    // 改行の形（LF と CRLF）に左右されないように、値で比べる
    let same = serde_json::from_str::<Value>(&current).ok().as_ref() == Some(&value);
    if check {
        return match same {
            true => Ok(json!({"same": true})),
            false => Err("design-system.json が正本（*.tokens.json）から作ったものと違う。brainstorming tokens を実行する".to_owned()),
        };
    }
    env.files.write(&target, &made)?;
    Ok(json!({"written": target.to_string_lossy(), "changed": !same}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, passing_engine, skill, MemoryFiles};
    use std::path::Path;

    const LIGHT: &str = r##"{"base":{"$type":"color","a":{"$value":{"hex":"#ABCDEF"}}},"$extensions":{"works.daidaiiro.contrast":[]}}"##;

    #[test]
    fn writing_then_checking_agrees_and_a_hand_edit_is_caught() {
        let env = env(
            MemoryFiles::with(&[
                ("/s/assets/catalog/light.tokens.json", LIGHT),
                ("/s/assets/catalog/dark.tokens.json", "{}"),
            ]),
            passing_engine(),
        );
        run(&env, &skill(), false).unwrap();
        assert!(run(&env, &skill(), true).is_ok());
        env.files
            .write(Path::new("/s/assets/catalog/design-system.json"), "{}")
            .unwrap();
        assert!(run(&env, &skill(), true).is_err());
    }
}
