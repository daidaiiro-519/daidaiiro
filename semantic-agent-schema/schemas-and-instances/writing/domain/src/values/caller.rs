//! 誰が打ったか（人か、名乗ったセッションか）。
//!
//! 決めるのは入口（--human ・ --session ・ ツールの環境変数 ・ semantic-agent-schema init の登録）で、ユースケースはこの値を受け取る。
//! 書く条件（x-decided-by の欄は人だけが変える）の入口 Entry と、記録の打ったセッションの ID はここから決まる。

use crate::aggregates::Entry;

/// 打ったセッションの ID をどこから知ったか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// ツールの環境変数（callers.json の表）
    Env,
    /// --session
    Flag,
    /// このフォルダで semantic-agent-schema init が登録したセッション
    Init,
}

impl Via {
    /// 記録と出力に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Flag => "flag",
            Self::Init => "init",
        }
    }
}

/// 操作を呼んだのは誰か。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    /// semantic-agent-schema を打ったセッション（AI のエージェント ・ CI ・ オーケストレーター）。--session ・ ツールの環境変数 ・ semantic-agent-schema init で分かった
    Session {
        /// セッション ID（ツールのものか、semantic-agent-schema init で Semantic Agent Schema が決めたもの）
        id: String,
        /// ツールの名前（claude-code など。分からなければ unknown）
        tool: String,
        /// 打ったフォルダ（Git の中ならリポジトリの根、外なら作業フォルダ）
        folder: String,
        /// ID をどこから知ったか
        via: Via,
    },
    /// --human で人と明示した呼び出し（ツールの環境変数があるところでは受け取らない）
    Human,
}

impl Caller {
    /// 記録に書く、打ったセッションの ID。人なら None。
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Session { id, .. } => Some(id.as_str()),
            Self::Human => None,
        }
    }

    /// 返すコマンドに引き継ぐ、誰が打ったかの引数。--session で名乗ったセッションは --session、人は --human。
    /// 環境変数と semantic-agent-schema init で分かったセッションは、次に打つときも同じに決まるので引き継がない。
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        match self {
            Self::Session {
                id, via: Via::Flag, ..
            } => vec!["--session".to_string(), id.clone()],
            Self::Human => vec!["--human".to_string()],
            Self::Session { .. } => Vec::new(),
        }
    }

    /// 集約の書く条件の入口：人か、名乗ったツールか。
    #[must_use]
    pub const fn entry(&self) -> Entry {
        match self {
            Self::Human => Entry::Human,
            Self::Session { .. } => Entry::Tool,
        }
    }
}
