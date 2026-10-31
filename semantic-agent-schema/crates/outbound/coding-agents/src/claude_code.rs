//! claude_code：判定させる任せる先を1回だけ、道具なし ・ 新しいセッション ・ スキーマに沿った出力で起動する
//! 旗の出どころ：code.claude.com/docs/en/cli-reference ・ headless（advisors/r96-ai-cli-facts.json で原文と照合済み）

use semantic_agent_schema_application::ports::delegates::DelegateError;
use semantic_agent_schema_tools::process::{Launch, run};
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

/// Claude Code にだけ渡す環境変数（--bare は API キーで認証する）
const ENV: [&str; 1] = ["ANTHROPIC_API_KEY"];

/// 起動の引数：--bare（hooks ・ MCP ・ CLAUDE.md を読まない）・ -p（1回だけ）・ --tools ""（組み込みの道具を外す）・
/// --disallowedTools "*"（MCP の道具も外す）・ --no-session-persistence（再開しない）・ --output-format json と --json-schema
pub fn argv(schema: &Value, model: Option<&str>) -> Vec<String> {
    let mut a: Vec<String> = [
        "claude",
        "--bare",
        "-p",
        "--tools",
        "",
        "--disallowedTools",
        "*",
        "--no-session-persistence",
        "--output-format",
        "json",
        "--json-schema",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    a.push(schema.to_string());
    if let Some(m) = model {
        a.push("--model".into());
        a.push(m.into());
    }
    a
}

/// 起動して、structured_output の答えと、実際に答えたモデル（modelUsage のキー。無ければ None）を返す
///
/// # Errors
/// 起動できない ・ 時間切れ ・ 出力に structured_output が無いとき
pub fn judge(
    prompt: &str,
    schema: &Value,
    model: Option<&str>,
    root: &Path,
    timeout: Duration,
) -> Result<(Value, Option<String>), DelegateError> {
    let out = run(&Launch {
        argv: &argv(schema, model),
        input: prompt.as_bytes(),
        root,
        timeout,
        env: &ENV,
        ok_codes: &[0],
    })
    .map_err(DelegateError::Failed)?;
    let v: Value = serde_json::from_slice(&out)
        .map_err(|e| DelegateError::Failed(format!("Claude Code の出力が JSON でない：{e}")))?;
    let answer = v.get("structured_output").cloned().ok_or_else(|| {
        DelegateError::Failed(
            "Claude Code の出力に structured_output が無い（答えを断った）".into(),
        )
    })?;
    Ok((answer, used_model(&v)))
}

/// 実際に答えたモデル：modelUsage（モデル名 → 使用量）のキー。複数なら「, 」でつなぐ。
/// modelUsage は Agent SDK の結果の欄として原文にあり、CLI の JSON 出力にも入ることは推測なので、無ければ None
pub fn used_model(output: &Value) -> Option<String> {
    let keys: Vec<&String> = output.get("modelUsage")?.as_object()?.keys().collect();
    (!keys.is_empty()).then(|| {
        keys.iter()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    })
}

/// claude --version の出力（読めなければ None）
pub fn version(root: &Path) -> Option<String> {
    let out = run(&Launch {
        argv: &["claude".to_string(), "--version".to_string()],
        input: b"",
        root,
        timeout: Duration::from_secs(10),
        env: &[],
        ok_codes: &[0],
    })
    .ok()?;
    let s = String::from_utf8_lossy(&out).trim().to_string();
    (!s.is_empty()).then_some(s)
}
