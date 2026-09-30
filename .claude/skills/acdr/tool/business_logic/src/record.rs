// SPDX-License-Identifier: MIT
//! 意思決定の記録を、節と差分を持つ1枚の HTML へ組む。
//!
//! **この道具は、変更後の中身を複製しない。** 節が持つのは決定であり、変更そのものは
//! 対象の文書の上に印として出る。対象が無い決定（新規）は、節だけの1枚になる ──
//! 例外にせず、同じ器で空にする。
//!
//! **HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::data_access;
use crate::markdown::esc;
use crate::panes::{self, Shop};
use crate::refs;
use crate::style::Style;
use crate::template::Parts;
use crate::validate;

/// 承認の状態。**これ以外を書かせない** ── 状態が自由文になると、「承認されているか」を
/// 読む側が判定することになる。
const STATUS: [&str; 3] = ["proposed", "accepted", "superseded"];

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

/// 記録のフォルダを、絶対の経路へ解く。**サービス層は入出力を持たない**ので、ここを通す。
///
/// # Errors
///
/// フォルダが無いときに返す。
pub fn resolve(record: &str) -> Result<PathBuf, String> {
    data_access::files::canonicalize(record).map_err(|e| format!("{record}: 開けない ── {e}"))
}

/// リポジトリの根を探す。
///
/// **パスを実行場所に依存させない** ── 依存させると、どこから呼んだかで結果が変わり、
/// 冪等でなくなる。
#[must_use]
pub fn repo_root(start: &Path) -> PathBuf {
    start
        .ancestors()
        .find(|d| data_access::files::exists(d.join(ROOT_MARK)))
        .unwrap_or(start)
        .to_path_buf()
}

/// 上部の節を組む。**欠けている節が在れば止まる。**
///
/// 描画は契約の共通の部品（`refs`）が持つ ── 見出しはスキーマの title、見せ方は x-view である。
/// 上部に置くのは4つの節（決めたこと ・ なぜ ・ どう変わるか ・ 比較した案）だけで、具体の
/// 差分は面が持つ。題はここが持つ ── 面の見出しと二重に出さない。
///
/// # Errors
///
/// 節が欠けているときと、状態が決められた値でないときに返す。
pub fn header(
    schema: &Value,
    spec: &Value,
    figure: &str,
    references: &Path,
) -> Result<String, String> {
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
    if array_of(spec, "alternatives").is_empty() {
        return Err(
            "比較した案が無い ── 無いことと書き忘れは、読み手には同じに見える。何と比べて選んだかを書く"
                .to_owned(),
        );
    }
    let mut view = spec.clone();
    let want = text_of(spec, "status");
    let want = if want.is_empty() {
        "proposed".to_owned()
    } else {
        want
    };
    if !STATUS.contains(&want.as_str()) {
        return Err(format!(
            "状態が「{want}」。使えるのは {} である",
            STATUS.join("／")
        ));
    }
    if let Some(map) = view.as_object_mut() {
        map.insert("status".to_owned(), Value::String(want));
        // **図は読み込み済みの SVG を渡す** ── 共通の部品は、svg の欄の SVG をそのまま埋め込む
        if figure.is_empty() {
            map.remove("figure");
        } else {
            map.insert(
                "figure".to_owned(),
                serde_json::json!({ "svg": figure, "caption": text_of(spec, "figure_caption") }),
            );
        }
    }
    Ok(format!(
        "<style>{}</style>{}",
        refs::scoped_style(references)?,
        refs::render_body(schema, &view, Path::new("."))
    ))
}

/// 記録1本を組む。
///
/// # Errors
///
/// 節が欠けているときと、型と噛み合わないときに返す。
pub fn build(shop: &Shop, spec: &Value, figure: &str) -> Result<panes::Made, String> {
    let head = header(shop.schema, spec, figure, shop.references)?;
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
        ("節が在る".to_owned(), out.contains("<div class=\"rv\">")),
        ("状態が付いている".to_owned(), out.contains("class=\"tag")),
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
    let body = data_access::files::read_to_string(&path)
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
            data_access::files::read_to_string(&at)
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
        let raw =
            data_access::files::read(&file).map_err(|e| format!("{file}: 読めない ── {e}"))?;
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
        let now = data_access::files::read(&path)
            .ok()
            .map(|body| sha256_of(&body));
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
    git: &str,
) -> Result<Report, String> {
    let dest = folder.join("index.html");
    let path = folder.join("acdr.json");
    let body = data_access::files::read_to_string(&path)
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
    let schema: Value = serde_json::from_str(
        &data_access::files::read_to_string(references.join("acdr.schema.json"))
            .map_err(|e| format!("acdr.schema.json を読めない ── {e}"))?,
    )
    .map_err(|e| format!("acdr.schema.json が JSON でない ── {e}"))?;
    let shop = Shop {
        parts: &parts,
        style: &style,
        git,
        schema: &schema,
        references,
    };
    let made = build(&shop, &spec, &figure)?;
    let mut lines = made.notes.clone();
    for (name, good) in check(&made.page, &spec, &made) {
        lines.push(format!("  {}  {name}", if good { "OK" } else { "NG" }));
    }

    if check_only {
        let same = data_access::files::read_to_string(&dest).is_ok_and(|now| now == made.page);
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

    data_access::files::write(&dest, &made.page)
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
        data_access::files::write(&path, flat_json(&raw) + "\n")
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

/// 今日の日付（利用者の地域の時刻）。
fn today() -> String {
    // **外部コマンドの date を呼ばない** ── Windows に実行ファイルとして無い（ACDR 0029）。
    // 利用者の地域の時刻で書く ── date と同じく、協定世界時にしない
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// 雛形から記録のフォルダを作る。**同じ名前が在れば作らない。**
///
/// # Errors
///
/// 既に在るときと、雛形を読めないときと、書けないときに返す。
pub fn new(references: &Path, folder: &Path, title: &str) -> Result<Vec<String>, String> {
    if data_access::files::exists(folder) {
        return Err(format!("既に在る: {}", folder.display()));
    }
    let template = references.join("spec-template.json");
    let body = data_access::files::read_to_string(&template)
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
    data_access::files::create_dir_all(folder)
        .map_err(|e| format!("{}: 作れない ── {e}", folder.display()))?;
    let path = folder.join("acdr.json");
    data_access::files::write(&path, flat_json(&spec) + "\n")
        .map_err(|e| format!("{}: 書けない ── {e}", path.display()))?;
    Ok(vec![
        format!("作った: {}", path.display()),
        "  図を置くなら flow.svg を隣に置く（描くのは design-svg である）".to_owned(),
        format!("  組む: acdr render {}", folder.display()),
    ])
}
