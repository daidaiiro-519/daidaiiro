// SPDX-License-Identifier: MIT
//! 道具の契約 ── **能力は1つ、呼び出し方は複数。**
//!
//! 戻り値は3つの欄を持つ ── `ok`（正常に終わったか）・ `findings`（検出したもの。
//! **誤りではない。** 0件でも正常である）・ `data`（機械が読む本体）。

use std::collections::BTreeMap;

use serde_json::{json, Value};

/// 引数1つ。**位置引数と旗を区別しない** ── 呼ぶ側の形はプレゼンテーション層が決める。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Arg {
    /// 呼ぶ側が使う名前。
    pub name: &'static str,
    /// 何を渡すか。
    pub summary: &'static str,
    /// 省略できないか。
    pub required: bool,
    /// 繰り返して渡せるか。
    pub many: bool,
    /// 省略したときの値。
    pub default: Option<&'static str>,
}

impl Arg {
    /// 省略できない引数を宣言する。
    #[must_use]
    pub const fn need(name: &'static str, summary: &'static str) -> Self {
        Self {
            name,
            summary,
            required: true,
            many: false,
            default: None,
        }
    }

    /// 省略できる引数を宣言する。
    #[must_use]
    pub const fn opt(
        name: &'static str,
        summary: &'static str,
        default: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            summary,
            required: false,
            many: false,
            default,
        }
    }

    /// **まとめて受ける引数を宣言する。** 繰り返すと足りていく ── 上書きにすると、
    /// 2つ目以降を黙って捨てることになる。
    #[must_use]
    pub const fn many(name: &'static str, summary: &'static str) -> Self {
        Self {
            name,
            summary,
            required: true,
            many: true,
            default: None,
        }
    }

    /// **省略できて、まとめて受ける引数を宣言する。** 0件で呼ぶことに意味がある
    /// ものは、これで宣言する ── `many` を使うと、0件の呼び方が誤用になる。
    #[must_use]
    pub const fn some(name: &'static str, summary: &'static str) -> Self {
        Self {
            name,
            summary,
            required: false,
            many: true,
            default: None,
        }
    }
}

/// 渡された引数。**プレゼンテーション層が組み、道具が読む。**
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Given {
    values: BTreeMap<String, Vec<String>>,
}

impl Given {
    /// 1つの値を足す。
    pub fn push(&mut self, name: &str, value: String) {
        self.values.entry(name.to_owned()).or_default().push(value);
    }

    /// 名前に対する最初の値を返す。**無ければ既定を返す。**
    #[must_use]
    pub fn one<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.values
            .get(name)
            .and_then(|v| v.first())
            .map_or(default, String::as_str)
    }

    /// 名前に対する値が在るかを返す。
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// 名前に対する値を全部返す。**まとめて受ける引数のためである。**
    #[must_use]
    pub fn all(&self, name: &str) -> &[String] {
        self.values.get(name).map_or(&[], Vec::as_slice)
    }

    /// この Skill の置き場所。**呼ぶ側が `skill_root` を渡したときは、それを優先する。**
    /// 渡されなければ、実行ファイル自身の位置から求める ── Rust の組は実行ファイルを `bin/` に置くので、
    /// 実行ファイルの1つ上（`bin/` の親）である。配布しても変わらない。どちらにも
    /// `SKILL.md` が無ければ、試した経路を示して止める ── `SKILL.md` はどの Skill も持つが、
    /// `references/` を持たない Skill も在る（実測）。
    ///
    /// # Errors
    ///
    /// 実行ファイルの位置を取れないとき、または `SKILL.md` が見つからないときに返す。
    pub fn skill_root(&self) -> Result<std::path::PathBuf, String> {
        if self.has("skill_root") {
            return Ok(std::path::PathBuf::from(self.one("skill_root", ".")));
        }
        let exe =
            std::env::current_exe().map_err(|e| format!("実行ファイルの位置を取れない ── {e}"))?;
        let root = exe
            .parent()
            .and_then(std::path::Path::parent)
            .ok_or_else(|| "実行ファイルの1つ上が無い ── --skill_root で渡す".to_owned())?;
        if root.join("SKILL.md").is_file() {
            Ok(root.to_path_buf())
        } else {
            Err(format!(
                "Skill の置き場所が見つからない ── {} に SKILL.md が無い。--skill_root で渡す",
                root.display()
            ))
        }
    }
}

impl Given {
    /// 外部の道具の、起動するコマンド。**業務ロジック層は名前を直書きせず、これで受け取る。**
    /// Skill のフォルダの `tool.json` の `external` から、名前で引く ── 利用者は
    /// `tool.json` を書き換えるだけで、呼ぶコマンドを差し替えられる。
    ///
    /// # Errors
    ///
    /// Skill のフォルダが見つからないとき、`tool.json` を読めないとき、
    /// tool.json にその名前の登録か、その `command` が無いときに返す。
    pub fn external(&self, name: &str) -> Result<String, String> {
        let path = self.skill_root()?.join("tool.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("{} を読めない ── {e}", path.display()))?;
        let doc: Value = serde_json::from_str(&text)
            .map_err(|e| format!("{} が JSON でない ── {e}", path.display()))?;
        doc.get("external")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|x| x.get("name").and_then(Value::as_str) == Some(name))
            .and_then(|x| x.get("command").and_then(Value::as_str))
            .filter(|c| !c.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| format!("{} の external に {name} の command が無い", path.display()))
    }
}

/// 道具の一覧を、機械が読む形で返す。**動詞なしで `--json` を付けたときに CLI が返す。**
/// 検査はこれを MCP の `tools/list` と突き合わせ、`skill_root` で Skill のフォルダの
/// 求め方を確かめる ── 言語に依存せずに、実行ファイルを起動するだけで検査できる。
#[must_use]
pub fn catalog(all: &[Tool], given: &Given) -> Outcome {
    let tools: Vec<Value> = all
        .iter()
        .map(|t| {
            let args: Vec<Value> = t
                .args
                .iter()
                .map(|a| json!({ "name": a.name, "required": a.required, "many": a.many }))
                .collect();
            json!({ "name": t.name, "summary": t.summary, "args": args })
        })
        .collect();
    let root = given
        .skill_root()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    Outcome::found(Vec::new(), json!({ "tools": tools, "skill_root": root }))
}

/// 道具の戻り値。**印字はしない** ── 印字と終了コードはプレゼンテーション層が持つ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Outcome {
    /// 正常に終わったか。
    pub ok: bool,
    /// 検出したもの。**誤りではない。**
    pub findings: Vec<String>,
    /// 機械が読む本体。
    pub data: Value,
}

impl Outcome {
    /// 検出を伴う正常な結果を組む。
    #[must_use]
    pub fn found(findings: Vec<String>, data: Value) -> Self {
        Self {
            ok: true,
            findings,
            data,
        }
    }

    /// 誤用を組む。
    #[must_use]
    pub fn misuse(reason: String) -> Self {
        Self {
            ok: false,
            findings: vec![reason],
            data: json!({}),
        }
    }

    /// 機械が読む形へ組む。
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "ok": self.ok, "findings": self.findings, "data": self.data })
    }

    /// 終了コードを返す ── 0 正常 ／ 1 検出あり ／ 2 誤用。
    /// **検出を誤用と同じ番号にしない。**
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        if !self.ok {
            return 2;
        }
        if self.findings.is_empty() {
            0
        } else {
            1
        }
    }
}

/// 道具1つ。`run` は結果を返し、`human` は人向けの文を組む。
#[derive(Clone)]
#[non_exhaustive]
pub struct Tool {
    /// 呼ぶときの名前。
    pub name: &'static str,
    /// 何をするか。
    pub summary: &'static str,
    /// 受け取る引数。
    pub args: Vec<Arg>,
    /// 実体。
    pub run: fn(&Given) -> Outcome,
    /// 人向けの文を組む。
    pub human: fn(&Outcome) -> String,
}
