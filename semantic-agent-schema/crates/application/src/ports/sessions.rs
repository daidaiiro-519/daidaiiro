//! セッションと手元のセッションの索引（索引の型もここ）

/// セッション ID：AI ツールの種類と、ツールが発行した文字列。基盤は中身を解釈しない（不変条件 I34）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRef {
    /// AI ツールの種類
    pub tool: String,
    /// ツールが発行した文字列
    pub id: String,
}

impl SessionRef {
    /// tool:id の形から読む
    ///
    /// # Errors
    /// tool:id の形でないとき
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.split_once(':') {
            Some((tool, id)) if !tool.is_empty() && !id.is_empty() => Ok(SessionRef {
                tool: tool.to_string(),
                id: id.to_string(),
            }),
            _ => Err(format!("セッション「{s}」は <ツール>:<id> の形で書く")),
        }
    }
}
