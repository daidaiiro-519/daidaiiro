// SPDX-License-Identifier: MIT
//! 意思決定の記録を、節と差分を持つ1枚の HTML へ組む。
//!
//! **この道具は、変更後の中身を複製しない。** 節が持つのは決定であり、変更そのものは
//! 対象の文書の上に印として出る。対象が無い決定（新規）は、節だけの1枚になる ──
//! 例外にせず、同じ器で空にする。
//!
//! **HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::markdown::esc;
use crate::panes::{self, Shop};
use crate::style::Style;
use crate::template::Parts;
use crate::validate;

/// 承認の状態。**これ以外を書かせない** ── 状態が自由文になると、「承認されているか」を
/// 読む側が判定することになる。
const STATUS: [(&str, &str, &str); 3] = [
    (
        "proposed",
        "提案",
        "まだ承認を得ていない。適用してはならない",
    ),
    ("accepted", "承認", "承認を得た。適用してよい"),
    (
        "superseded",
        "差し替え済み",
        "後の記録が、この決定を置き換えた",
    ),
];

/// リポジトリの根の印。
const ROOT_MARK: &str = ".git";

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

/// リポジトリの根を探す。
///
/// **パスを実行場所に依存させない** ── 依存させると、どこから呼んだかで結果が変わり、
/// 冪等でなくなる。
#[must_use]
pub fn repo_root(start: &Path) -> PathBuf {
    start
        .ancestors()
        .find(|d| d.join(ROOT_MARK).exists())
        .unwrap_or(start)
        .to_path_buf()
}

fn list(parts: &Parts, items: &[Value]) -> Result<String, String> {
    if items.is_empty() {
        return parts.part("none", &[("text", "無し".to_owned())]);
    }
    let mut inner = String::new();
    for item in items {
        let body = item
            .as_str()
            .map_or_else(|| item.to_string(), std::borrow::ToOwned::to_owned);
        inner.push_str(&parts.part("list-item", &[("item", body)])?);
    }
    parts.part("list", &[("items", inner)])
}

/// 比較した案。**どれも反証を通過しなかったものである** ── 未解決の懸念ではない。
fn dropped(parts: &Parts, rows: &[Value]) -> Result<String, String> {
    if rows.is_empty() {
        return parts.part("none", &[("text", "比較した案は無い".to_owned())]);
    }
    let mut inner = String::new();
    for row in rows {
        inner.push_str(&parts.part(
            "dropped-row",
            &[
                ("option", text_of(row, "option")),
                ("why_not", text_of(row, "why_not")),
            ],
        )?);
    }
    parts.part("dropped", &[("rows", inner)])
}

/// 変更前と変更後を、抽象の側で並べる。
///
/// **具体の差分は面が持つ。** ここが持つのは、何がどう変わるかの形である ── 抽象の対比が
/// 無いと、下に並ぶ具体の差分が何のためかを読み手が復元することになる。
fn shift(parts: &Parts, rows: &[Value]) -> Result<String, String> {
    let mut inner = String::new();
    for row in rows {
        inner.push_str(&parts.part(
            "shift-row",
            &[
                ("what", text_of(row, "what")),
                ("before", text_of(row, "from")),
                ("after", text_of(row, "to")),
            ],
        )?);
    }
    parts.part("shift", &[("rows", inner)])
}

/// 節を組む。**欠けている節が在れば止まる。**
///
/// 題はここが持つ ── 面の見出しと二重に出さない。
///
/// # Errors
///
/// 節が欠けているときと、状態が決められた値でないときに返す。
pub fn header(parts: &Parts, spec: &Value, figure: &str) -> Result<String, String> {
    let missing: Vec<&str> = validate::SECTIONS
        .iter()
        .copied()
        .filter(|k| text_of(spec, k).trim().is_empty())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "節が欠けている: {} ── 欠けた記録は、あとから誰にも補えない",
            missing.join("・")
        ));
    }
    let want = text_of(spec, "status");
    let want = if want.is_empty() {
        "proposed".to_owned()
    } else {
        want
    };
    let Some((_, label, note)) = STATUS.iter().find(|(k, _, _)| *k == want) else {
        return Err(format!(
            "状態が「{want}」。使えるのは {} である",
            STATUS
                .iter()
                .map(|(k, _, _)| *k)
                .collect::<Vec<_>>()
                .join("／")
        ));
    };
    let mut secs: Vec<(String, String)> = vec![(
        "なぜ、いま決めるのか".to_owned(),
        parts.part("para", &[("body", text_of(spec, "why"))])?,
    )];
    if !array_of(spec, "shift").is_empty() {
        secs.push((
            "形の変化".to_owned(),
            shift(parts, array_of(spec, "shift"))?,
        ));
    }
    if !figure.is_empty() {
        secs.push((
            "図で確認する".to_owned(),
            parts.part(
                "figure",
                &[
                    ("svg", figure.to_owned()),
                    ("caption", text_of(spec, "figure_caption")),
                ],
            )?,
        ));
    }
    if !text_of(spec, "how").is_empty() {
        secs.push((
            "実現の形".to_owned(),
            parts.part("para", &[("body", text_of(spec, "how"))])?,
        ));
    }
    secs.push((
        "適用先".to_owned(),
        parts.part("para", &[("body", text_of(spec, "applies_to"))])?,
    ));
    secs.push((
        "比較した案".to_owned(),
        dropped(parts, array_of(spec, "alternatives"))?,
    ));
    secs.push((
        "承認後に実施すること".to_owned(),
        list(parts, array_of(spec, "after_approval"))?,
    ));
    if !text_of(spec, "supersedes").is_empty() {
        secs.push((
            "supersedes".to_owned(),
            parts.part("para", &[("body", text_of(spec, "supersedes"))])?,
        ));
    }
    let mut body = String::new();
    for (heading, inner) in &secs {
        body.push_str(&parts.part(
            "sec",
            &[("heading", heading.clone()), ("body", inner.clone())],
        )?);
    }
    let no = esc(&value_text(spec, "no"));
    parts.part(
        "acdr",
        &[
            (
                "chip",
                if no.is_empty() {
                    String::new()
                } else {
                    parts.part("chip", &[("no", no)])?
                },
            ),
            ("title", esc(&text_of(spec, "title"))),
            ("status", want),
            ("label", (*label).to_owned()),
            ("note", (*note).to_owned()),
            ("when", esc(&value_text(spec, "date"))),
            ("decision", text_of(spec, "decision")),
            ("body", body),
        ],
    )
}

/// 文字でない値も、そのまま文字として出す。
fn value_text(value: &Value, key: &str) -> String {
    match value.get(key) {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// 記録1本を組む。
///
/// # Errors
///
/// 節が欠けているときと、型と噛み合わないときに返す。
pub fn build(shop: &Shop, spec: &Value, figure: &str) -> Result<panes::Made, String> {
    let head = header(shop.parts, spec, figure)?;
    if array_of(spec, "docs").is_empty() {
        // 新規の決定。差分が無いので、節だけの1枚になる
        let style = &shop.style;
        return Ok(panes::Made {
            page: shop.parts.part(
                "page-bare",
                &[
                    ("title", esc(&text_of(spec, "title"))),
                    (
                        "style",
                        format!("{}{}{}", style.tokens, style.base, style.section),
                    ),
                    ("head", head),
                ],
            )?,
            ..panes::Made::default()
        });
    }
    // 題は節が持つ。面の見出しと二重に出さない ── 面の見出しの場所を、節で差し替える。
    // 節と面のあいだに橋を1行置く ── 下に並ぶのは、この決定を採ったときの具体である
    let heading = head + &shop.parts.part("bridge", &[])?;
    panes::build(shop, spec, &heading, &shop.style.section)
}

/// 組んだあと、開閉が成立する形かを検査する。
#[must_use]
pub fn check(out: &str, spec: &Value, made: &panes::Made) -> Vec<(String, bool)> {
    let mut ok = vec![
        ("節が在る".to_owned(), out.contains("<div class=\"acdr\">")),
        ("状態が付いている".to_owned(), out.contains("class=\"st ")),
    ];
    if !array_of(spec, "docs").is_empty() {
        ok.extend(panes::check(out, spec, made));
    }
    ok
}

/// 記録のフォルダから入力を読む。**図は隣のファイルから読み、JSON へ埋め込まない。**
///
/// # Errors
///
/// 読めないときと、入力の検査が通らないときに返す。
pub fn load(references: &Path, folder: &Path) -> Result<(Value, String), String> {
    let path = folder.join("acdr.json");
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    let mut spec: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))?;
    // **パスはリポジトリの根から解決する** ── 実行場所に依存させると、冪等でなくなる
    let root = repo_root(folder);
    let bad = validate::inspect(references, &spec, Some(folder), Some(&root));
    if !bad.is_empty() {
        return Err(format!(
            "入力の検査が通っていない ── HTML は書き出さない:\n  {}",
            bad.iter()
                .map(|e| format!("× {e}"))
                .collect::<Vec<_>>()
                .join("\n  ")
        ));
    }
    // **空の欄は、図が無いことである** ── 名前として扱うと、フォルダを読もうとする
    let figure = match spec
        .get("figure")
        .and_then(|x| x.as_str())
        .filter(|name| !name.is_empty())
    {
        None => String::new(),
        Some(name) => {
            let at = folder.join(name);
            std::fs::read_to_string(&at)
                .map_err(|e| format!("{}: 読めない ── {e}", at.display()))?
        }
    };
    if let Some(docs) = spec.get_mut("docs").and_then(|x| x.as_array_mut()) {
        for doc in docs {
            let file = text_of(doc, "file");
            if let Some(map) = doc.as_object_mut() {
                map.insert(
                    "file".to_owned(),
                    Value::String(root.join(file).display().to_string()),
                );
            }
        }
    }
    Ok((spec, figure))
}

fn sha256_of(data: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 対象の文書の sha256 を取る。**承認済みの記録は、承認時点の姿を保持する。**
///
/// # Errors
///
/// 対象の文書を読めないときに返す。
pub fn seal(spec: &Value) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for doc in array_of(spec, "docs") {
        let file = text_of(doc, "file");
        let raw = std::fs::read(&file).map_err(|e| format!("{file}: 読めない ── {e}"))?;
        out.push((text_of(doc, "key"), sha256_of(&raw)));
    }
    Ok(out)
}

/// 封印のあとに、対象の文書が変化した面を返す。
///
/// **封印の判定を、組み立てより前に置く** ── 承認済みの記録は過去の姿であり、対象が
/// 移動 ・ 消滅していることが在る。先に組もうとすると、そこで停止して「封印されている」
/// という結論にすら到達しない。
#[must_use]
pub fn drifted(folder: &Path, raw: &Value) -> Vec<String> {
    let Some(sealed) = raw.get("seal").and_then(|x| x.as_object()) else {
        return Vec::new();
    };
    if sealed.is_empty() {
        return Vec::new();
    }
    let root = repo_root(folder);
    let mut moved = Vec::new();
    for doc in array_of(raw, "docs") {
        let path = root.join(text_of(doc, "file"));
        let now = std::fs::read(&path).ok().map(|body| sha256_of(&body));
        let key = text_of(doc, "key");
        let was = sealed.get(&key).and_then(|x| x.as_str());
        if was != now.as_deref() {
            moved.push(key);
        }
    }
    moved
}

/// 組んだ結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Report {
    /// 終了の判定。`0` 正常 ／ `1` 検出あり。
    pub code: i32,
    /// 報告へ出す行。
    pub lines: Vec<String>,
}

/// 記録を1枚へ組む。
///
/// # Errors
///
/// 読めないときと、入力の検査が通らないときに返す。
pub fn build_record(
    references: &Path,
    folder: &Path,
    check_only: bool,
    force: bool,
) -> Result<Report, String> {
    let dest = folder.join("index.html");
    let path = folder.join("acdr.json");
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    let mut raw: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))?;
    // **空の封印は、封印していないことである** ── 対象を持たない記録の `seal` は
    // 空になるので、鍵の有無で判定すると、そこで封印したことになってしまう
    let sealed = raw
        .get("seal")
        .and_then(|x| x.as_object())
        .is_some_and(|m| !m.is_empty());
    let moved = drifted(folder, &raw);

    if !moved.is_empty() && !force {
        if check_only {
            return Ok(Report {
                code: 0,
                lines: vec![format!(
                    "  承認時点の姿である  対象の文書が後に変化した: {}",
                    moved.join(" ・ ")
                )],
            });
        }
        return Ok(Report {
            code: 1,
            lines: vec![
                format!(
                    "  組み直しを拒否する  承認済みの記録で、対象の文書が後に変化している: {}",
                    moved.join(" ・ ")
                ),
                "  組み直すと承認時点の姿が失われる。意図する場合は --force を渡す".to_owned(),
            ],
        });
    }

    let (spec, figure) = load(references, folder)?;
    let parts = Parts::load(&references.join("acdr.template.html"))?;
    let style = Style::load(references)?;
    let shop = Shop {
        parts: &parts,
        style: &style,
    };
    let made = build(&shop, &spec, &figure)?;
    let mut lines = made.notes.clone();
    for (name, good) in check(&made.page, &spec, &made) {
        lines.push(format!("  {}  {name}", if good { "OK" } else { "NG" }));
    }

    if check_only {
        let same = std::fs::read_to_string(&dest).is_ok_and(|now| now == made.page);
        lines.push(format!(
            "  {}  {}",
            if same { "同一" } else { "差が在る" },
            dest.display()
        ));
        return Ok(Report {
            code: i32::from(!same),
            lines,
        });
    }

    std::fs::write(&dest, &made.page)
        .map_err(|e| format!("{}: 書けない ── {e}", dest.display()))?;
    if text_of(&raw, "status") == "accepted" && !sealed {
        let taken = seal(&spec)?;
        if let Some(map) = raw.as_object_mut() {
            let mut into = serde_json::Map::new();
            for (key, digest) in taken {
                into.insert(key, Value::String(digest));
            }
            map.insert("seal".to_owned(), Value::Object(into));
        }
        std::fs::write(&path, flat_json(&raw) + "\n")
            .map_err(|e| format!("{}: 書けない ── {e}", path.display()))?;
        lines.push("  封印した  対象の文書の sha256 を記録へ保存した".to_owned());
    }
    lines.push(format!(
        "  印 {} 件 / {} 面 / {} 字",
        made.total,
        array_of(&spec, "docs").len(),
        made.page.chars().count()
    ));
    Ok(Report { code: 0, lines })
}

/// 平らな JSON を、1字下げで組む。
fn flat_json(value: &Value) -> String {
    let mut out = Vec::new();
    let mut writer = serde_json::Serializer::with_formatter(
        &mut out,
        serde_json::ser::PrettyFormatter::with_indent(b" "),
    );
    use serde::Serialize as _;
    if value.serialize(&mut writer).is_err() {
        return String::new();
    }
    String::from_utf8(out).unwrap_or_default()
}

/// 今日の日付。**外の道具から取る** ── この側に時計を持たない。
fn today() -> String {
    Command::new("date")
        .arg("+%Y-%m-%d")
        .stdin(Stdio::null())
        .output()
        .ok()
        .map_or_else(String::new, |o| {
            String::from_utf8_lossy(&o.stdout).trim().to_owned()
        })
}

/// 雛形から記録のフォルダを作る。**同じ名前が在れば作らない。**
///
/// # Errors
///
/// 既に在るときと、雛形を読めないときと、書けないときに返す。
pub fn new(references: &Path, folder: &Path, title: &str) -> Result<Vec<String>, String> {
    if folder.exists() {
        return Err(format!("既に在る: {}", folder.display()));
    }
    let template = references.join("spec-template.json");
    let body = std::fs::read_to_string(&template)
        .map_err(|e| format!("{}: 読めない ── {e}", template.display()))?;
    let mut spec: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", template.display()))?;
    let name = folder
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let no = name.split('-').next().unwrap_or_default();
    if let Some(map) = spec.as_object_mut() {
        map.insert("title".to_owned(), Value::String(title.to_owned()));
        map.insert("no".to_owned(), Value::String(format!("ACDR {no}")));
        map.insert("date".to_owned(), Value::String(today()));
    }
    std::fs::create_dir_all(folder)
        .map_err(|e| format!("{}: 作れない ── {e}", folder.display()))?;
    let path = folder.join("acdr.json");
    std::fs::write(&path, flat_json(&spec) + "\n")
        .map_err(|e| format!("{}: 書けない ── {e}", path.display()))?;
    Ok(vec![
        format!("作った: {}", path.display()),
        "  図を置くなら flow.svg を隣に置く（描くのは design-svg である）".to_owned(),
        format!("  組む: acdr render {}", folder.display()),
    ])
}
