//! ポート IdentifyCaller を、引数で実装する。--session を渡したツールはそのセッション、渡さなければ人。
//! エージェントの環境から見分けるもの（outbound の agents）は、エージェントをつなぐステップ6 で足す。

use contracts_and_instances_operation_recording_application::ports::{Caller, IdentifyCaller};

/// 引数の --session から決める呼び出し元。
pub struct FromArguments {
    session: Option<String>,
}

impl FromArguments {
    /// --session の値（無ければ None）。
    pub const fn new(session: Option<String>) -> Self {
        Self { session }
    }
}

impl IdentifyCaller for FromArguments {
    fn identify(&self) -> Caller {
        self.session
            .as_ref()
            .map_or(Caller::Human, |s| Caller::Tool {
                tool_ref: s.clone(),
            })
    }
}
