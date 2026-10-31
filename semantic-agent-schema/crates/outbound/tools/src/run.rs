//! argv の任せる先：x-delegates の run.argv をシェルを通さずに起動し、呼び出し（call.schema.json の形）を標準入力に渡して、出力の JSON を受け取る

use crate::process::{Launch, run};
use semantic_agent_schema_application::ports::delegates::{
    Answer, DelegateError, Delegates, Provenance,
};
use semantic_agent_schema_application::semantic_agent_schema_declaration::aggregates::entities::delegate::Delegate;
use semantic_agent_schema_application::semantic_agent_schema_validation::values::call::Call;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

/// argv の任せる先を、作業の根で起動する。終了コードは 0（検出なし）と 1（検出あり）を正常とみなす
pub struct ArgvDelegates {
    root: PathBuf,
    timeout: Duration,
}

impl ArgvDelegates {
    /// 作業の根と時間切れを渡して作る
    pub fn new(root: PathBuf, timeout: Duration) -> Self {
        ArgvDelegates { root, timeout }
    }
}

impl Delegates for ArgvDelegates {
    fn call(
        &self,
        delegate: &Delegate,
        call: &Call,
        _output_schema: &Value,
    ) -> Result<Answer, DelegateError> {
        let argv = delegate.argv().ok_or_else(|| {
            DelegateError::Unsupported(format!("任せる先 {} に argv が無い", delegate.name()))
        })?;
        let input = call.to_json().to_string();
        let out = run(&Launch {
            argv,
            input: input.as_bytes(),
            root: &self.root,
            timeout: self.timeout,
            env: &[],
            ok_codes: &[0, 1],
        })
        .map_err(DelegateError::Failed)?;
        let output: Value = serde_json::from_slice(&out)
            .map_err(|e| DelegateError::Failed(format!("{} の出力が JSON でない：{e}", argv[0])))?;
        Ok(Answer {
            output,
            provenance: Provenance::Process {
                argv: argv.to_vec(),
            },
        })
    }
}
