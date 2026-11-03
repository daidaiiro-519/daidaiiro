//! 誰が打ったか（人か、名乗ったセッションか）。
//!
//! 決めるのは入口（--session ・ ツールの環境変数 ・ semantic-agent-schema init の登録）で、ユースケースはこの値を受け取る。
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
        /// セッション ID（ツールのものか、semantic-agent-schema init で SAS が決めたもの）
        id: String,
        /// ツールの名前（claude-code など。分からなければ unknown）
        tool: String,
        /// 打ったフォルダ（Git の中ならリポジトリの根、外なら作業フォルダ）
        folder: String,
        /// ID をどこから知ったか
        via: Via,
    },
    /// --session も環境変数も semantic-agent-schema init の登録も無い呼び出し。人とみなす
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

    /// --session で名乗ったセッションの ID（次に打つコマンドでも名乗らせる）。それ以外は None。
    #[must_use]
    pub fn flagged(&self) -> Option<&str> {
        match self {
            Self::Session {
                id, via: Via::Flag, ..
            } => Some(id.as_str()),
            _ => None,
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
