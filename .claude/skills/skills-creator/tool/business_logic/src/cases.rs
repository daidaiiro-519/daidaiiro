// SPDX-License-Identifier: MIT
//! テストケースを実行する（ボード skills-creator-contract の論点2 ・ 3）。**共通ツールの振る舞いは、
//! テストケースで決まる。**
//!
//! テストケースは1件1ファイルの JSON で、呼び出しと期待値（終了コードと `--json` の出力）を持つ。
//! 検証する Skill の tool.json の実行コマンドで呼ぶので、どの言語で書いた Skill にも同じテストケースを
//! 実行できる。JSON は値として比較し、文字のエスケープの違いは問わない。
//!
//! | 欄 | 中身 |
//! |---|---|
//! | `case` | 何を確認するか |
//! | `test` | 対応するリファレンス実装のテストの名前 |
//! | `setup` | 先に実行する呼び出し（比較しない）。任意 |
//! | `call` | 比較する呼び出し。経路はテストケースのフォルダからの相対 |
//! | `expect` | `exit`（終了コード）と、任意の `json`（`--json` の出力） |
//! | `ignore` | 比較から除外する JSON Pointer。スキーマ検査のエラー文はライブラリごとに違う。任意 |

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::behavior;
use crate::data_access::{files, process};

/// 1回の呼び出しの制限時間。
const LIMIT: Duration = Duration::from_secs(120);

/// 一時ディレクトリの通し番号。**同じプロセスの中で並行して実行しても、名前が重ならない。**
static SERIAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// テストケースの対応の例外を置くファイルの名前。
const EXEMPT: &str = "exempt.json";

/// テストケース1件。
#[derive(Debug, Clone)]
struct Case {
    name: String,
    setup: Vec<Vec<String>>,
    call: Vec<String>,
    exit: i64,
    json: Option<Value>,
    ignore: Vec<String>,
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect()
}

/// テストケースのファイル（`exempt.json` を除く）を、名前の順に並べる。
fn files_of(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found: Vec<PathBuf> = files::list(dir)
        .map_err(|e| format!("{} を読めない ── {e}", dir.display()))?
        .into_iter()
        .filter(|p| {
            files::is_file(p)
                && p.extension().is_some_and(|x| x == "json")
                && p.file_name().is_some_and(|n| n != EXEMPT)
        })
        .collect();
    found.sort();
    Ok(found)
}

fn read(path: &Path) -> Result<Value, String> {
    let body =
        files::read_to_string(path).map_err(|e| format!("{} を読めない ── {e}", path.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("{} が JSON でない ── {e}", path.display()))
}

fn load(dir: &Path) -> Result<Vec<Case>, String> {
    files_of(dir)?
        .into_iter()
        .map(|p| {
            let v = read(&p)?;
            Ok(Case {
                name: p
                    .file_stem()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                setup: v
                    .get("setup")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(strings)
                    .collect(),
                call: strings(&v["call"]),
                exit: v["expect"]["exit"].as_i64().unwrap_or(0),
                json: v["expect"].get("json").cloned(),
                ignore: strings(&v["ignore"]),
            })
        })
        .collect()
}

/// JSON Pointer の位置を除く。**無い位置は無視する** ── 期待値と出力の両方から同じ位置を除く。
fn drop_at(v: &mut Value, pointer: &str) {
    let Some((parent, last)) = pointer.rsplit_once('/') else {
        return;
    };
    let last = last.replace("~1", "/").replace("~0", "~");
    let target = if parent.is_empty() {
        Some(v)
    } else {
        v.pointer_mut(parent)
    };
    match target {
        Some(Value::Object(m)) => {
            m.shift_remove(&last);
        }
        Some(Value::Array(a)) => {
            if let Ok(i) = last.parse::<usize>() {
                if i < a.len() {
                    a.remove(i);
                }
            }
        }
        _ => {}
    }
}

/// 最初に違う位置を返す。**同じなら None**。
fn first_diff(a: &Value, b: &Value, at: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let mut keys: Vec<&String> = x.keys().chain(y.keys()).collect();
            keys.sort();
            keys.dedup();
            keys.into_iter().find_map(|k| match (x.get(k), y.get(k)) {
                (Some(p), Some(q)) => first_diff(p, q, &format!("{at}/{k}")),
                _ => Some(format!("{at}/{k}")),
            })
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() == y.len() {
                x.iter()
                    .zip(y)
                    .enumerate()
                    .find_map(|(i, (p, q))| first_diff(p, q, &format!("{at}/{i}")))
            } else {
                Some(format!("{at}（件数 {} と {}）", x.len(), y.len()))
            }
        }
        _ if a == b => None,
        _ => Some(if at.is_empty() {
            "/".to_owned()
        } else {
            at.to_owned()
        }),
    }
}

/// 期待値と出力を比較する。**除外する位置を両方から除いてから比べる。** 同じなら None、違えば最初の位置。
#[must_use]
pub fn compare(expected: &Value, actual: &Value, ignore: &[String]) -> Option<String> {
    let (mut x, mut y) = (expected.clone(), actual.clone());
    for p in ignore {
        drop_at(&mut x, p);
        drop_at(&mut y, p);
    }
    first_diff(&x, &y, "")
}

/// リファレンス実装のテストと、テストケースの対応を照合する。**テストケースの無いテストと、
/// 無いテストを指すテストケースを返す。** 例外は `exempt.json` に理由を付けて置く。
///
/// # Errors
///
/// テストケースのフォルダを読めないときに返す。
pub fn unmatched(test_source: &str, dir: &Path) -> Result<Vec<String>, String> {
    let tests: Vec<String> = test_source
        .split("#[test]")
        .skip(1)
        .filter_map(|chunk| {
            let rest = chunk.trim_start().strip_prefix("fn ")?;
            let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
            Some(rest[..end].to_owned())
        })
        .collect();
    let named: Vec<String> = files_of(dir)?
        .iter()
        .map(|p| read(p).map(|v| v["test"].as_str().unwrap_or_default().to_owned()))
        .collect::<Result<_, _>>()?;
    let exempt: Vec<String> = if files::is_file(dir.join(EXEMPT)) {
        read(&dir.join(EXEMPT))?
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x["test"].as_str().map(str::to_owned))
            .collect()
    } else {
        Vec::new()
    };
    let mut found: Vec<String> = tests
        .iter()
        .filter(|t| !named.contains(t) && !exempt.contains(t))
        .map(|t| format!("テストケースの無いテスト: {t} ── 同じ名前のテストケースを置くか、理由を付けて {EXEMPT} に置く"))
        .collect();
    found.extend(
        named
            .iter()
            .filter(|n| !tests.contains(n))
            .map(|n| format!("無いテストを指すテストケース: {n}")),
    );
    Ok(found)
}

/// フォルダを丸ごと複製する。
fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    files::create_dir_all(to).map_err(|e| format!("{} を作れない ── {e}", to.display()))?;
    for p in files::list(from).map_err(|e| format!("{} を読めない ── {e}", from.display()))?
    {
        let name = p.file_name().map(PathBuf::from).unwrap_or_default();
        if files::is_dir(&p) {
            copy_dir(&p, &to.join(name))?;
        } else {
            let body = files::read(&p).map_err(|e| format!("{} を読めない ── {e}", p.display()))?;
            files::write(to.join(name), body)
                .map_err(|e| format!("{} を書けない ── {e}", to.display()))?;
        }
    }
    Ok(())
}

/// テストケースを、Skill の tool.json の実行コマンドで実行する。**テストケースごとに、フォルダの
/// 複製で実行する** ── import のように references を書き換える呼び出しが在る。
/// 結果は（テストケースの名前、不合格の理由）の並びで、合格なら理由は None。
///
/// # Errors
///
/// tool.json を読めないとき、テストケースを読めないときに返す。
pub fn run(root: &Path, dir: &Path) -> Result<Vec<(String, Option<String>)>, String> {
    let (command, base) = behavior::cli_of(root)?;
    let mut out = Vec::new();
    for case in load(dir)? {
        let n = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let work = files::temp_dir().join(format!("sc-cases-run-{}-{n}", std::process::id()));
        let _ = files::remove_dir_all(&work);
        copy_dir(dir, &work)?;
        let call = |args: &[String]| {
            let mut with = base.clone();
            with.extend(args.iter().cloned());
            with.push("--json".to_owned());
            process::run(&command, &with, &work, LIMIT)
        };
        for s in &case.setup {
            let _ = call(s);
        }
        let why = match call(&case.call) {
            Err(e) => Some(format!("実行できない ── {e:?}")),
            Ok(ran) if i64::from(ran.code) != case.exit => {
                Some(format!("終了コード {}（期待値 {}）", ran.code, case.exit))
            }
            Ok(ran) => case.json.as_ref().and_then(|want| {
                let have: Value = serde_json::from_str(&ran.stdout).unwrap_or(Value::Null);
                compare(want, &have, &case.ignore).map(|at| format!("出力が期待値と違う: {at}"))
            }),
        };
        let _ = files::remove_dir_all(&work);
        out.push((case.name, why));
    }
    Ok(out)
}
