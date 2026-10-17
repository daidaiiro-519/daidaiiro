//! ボードの置き場所と、ボードの値の読み方。
//!
//! ```text
//! .brainstorming/
//!   dashboard.json              ダッシュボード
//!   <ボード>.json               ボードの本体
//!   <ボード>/rounds/<回>.json   回の記録（書き換えない）
//!   <ボード>/answers/<id>.json  回答（serve が書く）
//!   <ボード>/figures/           図
//!   .chat/<ボード>/<AI>.json    チャットの記録（serve だけが書く）
//!   .serve/runs/<run>/          AI が作業する前の写し
//!   .serve/locks/<ボード>.json  ボードのロック（AI が動いているあいだ）
//!   site/                       描いたページ
//! ```

use serde_json::Value;
use std::path::{Path, PathBuf};

/// ボードの id として使える名前か（小文字の英数字とハイフン）。
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

/// .brainstorming/ の中の置き場所。
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub root: PathBuf,
}

impl Layout {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
    pub fn board_file(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }
    pub fn board_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }
    pub fn rounds(&self, id: &str) -> PathBuf {
        self.board_dir(id).join("rounds")
    }
    pub fn answers(&self, id: &str) -> PathBuf {
        self.board_dir(id).join("answers")
    }
    pub fn chat_file(&self, id: &str, agent: &str) -> PathBuf {
        self.root
            .join(".chat")
            .join(id)
            .join(format!("{agent}.json"))
    }
    pub fn chat_dir(&self) -> PathBuf {
        self.root.join(".chat")
    }
    /// serve と CLI が使う作業の置き場所（写し ・ ロック ・ 描画の写し）。
    pub fn serve_dir(&self) -> PathBuf {
        self.root.join(".serve")
    }
    /// ボードのロック。
    pub fn lock(&self, id: &str) -> PathBuf {
        self.serve_dir().join("locks").join(format!("{id}.json"))
    }
    pub fn runs(&self) -> PathBuf {
        self.root.join(".serve").join("runs")
    }
    pub fn site(&self) -> PathBuf {
        self.root.join("site")
    }
    pub fn dashboard(&self) -> PathBuf {
        self.root.join("dashboard.json")
    }
    /// リポジトリの根（.brainstorming/ の親）。AI の CLI はここで動く。
    pub fn repo(&self) -> PathBuf {
        self.root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.root.clone())
    }
}

/// オブジェクトの文字列の値。無ければ空。
pub fn text<'v>(value: &'v Value, key: &str) -> &'v str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

/// オブジェクトの並びの値。無ければ空。
pub fn list<'v>(value: &'v Value, key: &str) -> &'v [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// ボードの論点。
pub fn topics(board: &Value) -> &[Value] {
    list(board, "topics")
}

/// id の論点。
pub fn topic<'b>(board: &'b Value, id: &str) -> Option<&'b Value> {
    topics(board).iter().find(|topic| text(topic, "id") == id)
}

/// 文字列の並びとして読む（文字列でない要素は落とす）。
pub fn strings(value: &Value, key: &str) -> Vec<String> {
    list(value, key)
        .iter()
        .filter_map(|item| item.as_str().map(str::to_owned))
        .collect()
}

/// 回の番号。無ければ 0。
pub fn round(board: &Value) -> u64 {
    board.get("round").and_then(Value::as_u64).unwrap_or(0)
}

/// 整形した JSON（改行で終える）。
pub fn pretty(value: &Value) -> String {
    let mut out = serde_json::to_string_pretty(value).unwrap_or_default();
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ids_are_lowercase_words_joined_by_hyphens() {
        let cases = [
            ("agents-creator", true),
            ("q1", true),
            ("", false),
            ("Agents", false),
            ("-a", false),
            ("a-", false),
            ("a--b", false),
            ("a/b", false),
            ("..", false),
        ];
        let got: Vec<bool> = cases.iter().map(|(id, _)| valid_id(id)).collect();
        let want: Vec<bool> = cases.iter().map(|(_, ok)| *ok).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn layout_places_everything_under_the_root() {
        let layout = Layout::new(Path::new("/r/.brainstorming"));
        assert_eq!(
            layout.board_file("a"),
            Path::new("/r/.brainstorming/a.json")
        );
        assert_eq!(layout.rounds("a"), Path::new("/r/.brainstorming/a/rounds"));
        assert_eq!(
            layout.answers("a"),
            Path::new("/r/.brainstorming/a/answers")
        );
        assert_eq!(
            layout.chat_file("a", "codex"),
            Path::new("/r/.brainstorming/.chat/a/codex.json")
        );
        assert_eq!(layout.repo(), Path::new("/r"));
    }

    #[test]
    fn readers_return_empty_when_missing() {
        let board = json!({"topics": [{"id": "Q1", "name": "n"}], "round": 2});
        assert_eq!(text(&board, "title"), "");
        assert_eq!(topics(&board).len(), 1);
        assert_eq!(topic(&board, "Q1").map(|t| text(t, "name")), Some("n"));
        assert_eq!(round(&board), 2);
        assert_eq!(strings(&json!({"a": ["x", 1, "y"]}), "a"), vec!["x", "y"]);
    }
}
