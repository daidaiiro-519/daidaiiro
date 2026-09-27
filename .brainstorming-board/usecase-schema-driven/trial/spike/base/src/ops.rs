//! 仕様を取る操作と、受ける操作 ── **能力の契約の一部である。**
//!
//! **方法論の語を1つも持たない** ── 参照は「既知の ID と一致する文字列」として見つける。
//! 欄の名前（`context` ・ `upstream` など）を知る必要が無い。
use crate::{Decl, Registry};
use serde_json::Value;
use std::collections::BTreeSet;

/// 索引 ── ID ・ 名前 ・ 種類だけを並べる。`kind` を渡すと絞り込む。
pub fn list(decls: &[Decl], kind: Option<&str>) -> String {
    decls
        .iter()
        .filter(|d| kind.is_none_or(|k| d.kind == k))
        .map(|d| format!("{} {} <{}>", d.id, d.name, d.kind))
        .collect::<Vec<_>>()
        .join("\n")
}

fn 文字列を集める(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| 文字列を集める(x, out)),
        Value::Object(o) => o.values().for_each(|x| 文字列を集める(x, out)),
        _ => {}
    }
}

/// 参照 ── その宣言が指している ID と、その宣言を指している ID。
pub fn refs(decls: &[Decl], id: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let 既知: BTreeSet<&str> = decls.iter().map(|d| d.id.as_str()).collect();
    if !既知.contains(id) {
        return Err(format!("その ID の宣言が無い: {id}"));
    }
    let 指す = |d: &Decl| -> BTreeSet<String> {
        let mut xs = vec![];
        文字列を集める(&serde_json::to_value(d).unwrap(), &mut xs);
        xs.into_iter()
            .filter(|s| s != &d.id && 既知.contains(s.as_str()))
            .collect()
    };
    let me = decls.iter().find(|d| d.id == id).unwrap();
    let 先: Vec<String> = 指す(me).into_iter().collect();
    let 元: Vec<String> = decls
        .iter()
        .filter(|d| d.id != id && 指す(d).contains(id))
        .map(|d| d.id.clone())
        .collect();
    Ok((先, 元))
}

/// 宣言1件を取る。`as_` に射影の名前を渡すと、その形で返る。
pub fn get(decls: &[Decl], id: &str, as_: Option<&str>, reg: &Registry) -> Result<String, String> {
    let d = decls
        .iter()
        .find(|d| d.id == id)
        .ok_or(format!("その ID の宣言が無い: {id}"))?;
    match as_ {
        None => {
            let mut v = serde_json::json!({ "decl": d });
            if !d.nodes.is_empty() {
                let ids: Vec<String> = d.nodes.iter().map(|n| n.id.clone()).collect();
                v["test_contract"] = serde_json::Value::String(test_contract(&ids, &d.id));
            }
            serde_json::to_string(&v).map_err(|e| e.to_string())
        }
        Some(name) => crate::render_as(d, reg, name).ok_or(format!("その射影が無い: {name}")),
    }
}

/// 書いた仕様の形を受ける ── 種類ごとの schema で検査する。
pub fn validate(decls: &[Decl], kinds: &Value) -> Vec<String> {
    let mut out = vec![];
    for d in decls {
        let v = serde_json::to_value(d).unwrap();
        if let Some(s) = kinds.get(&d.kind) {
            crate::schema::validate(s, &v, &d.id, &mut out);
        }
    }
    out
}

/// 網羅されたシナリオを突き合わせた結果。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    /// 仕様が必要であるのに、テストの側が網羅していないシナリオ
    pub uncovered: Vec<String>,
    /// テストの側が記録したのに、仕様に無いシナリオ（書き間違い）
    pub unknown: Vec<String>,
}

/// **道具が知るのは2つの集合だけである** ── 仕様が必要とするシナリオと、テストの側が網羅したシナリオ。
///
/// 網羅したシナリオの集合をどう作ったか（走ったときに書き出したか、探したか）を、道具は知らない。
pub fn coverage(required: &BTreeSet<String>, covered: &BTreeSet<String>) -> Coverage {
    Coverage {
        uncovered: required.difference(covered).cloned().collect(),
        unknown: covered.difference(required).cloned().collect(),
    }
}

/// テストの側の契約 ── **道具の本体が持つ**。方法論が替わっても、この文面は変わらない。
///
/// 言語のコードは渡さない ── 渡すと、道具が言語ごとの雛形を持つことになる。
/// 渡すのは契約の文面だけで、補助の関数は AI がテストの言語で書く。
pub fn test_contract(ids: &[String], decl_id: &str) -> String {
    format!(
        "このシナリオを検査するテストは、次の契約を遵守すること。\n\
         1. テストの先頭で、環境変数 SCENARIO_TRACE を確認する。\n\
         2. 設定されていれば、そのファイルへ、そのテストが検査するシナリオID を1行追記する（改行で終える）。\n\
         3. 追記する処理は、テストの言語の標準ライブラリだけで補助の関数として1つ作り、テストの間で共有する。既に在れば、それを使用する。\n\
         4. そのテストが検査しないシナリオの ID を書き出さない。\n\
         5. テストを skip するときは、書き出す前に skip する。\n\
         対象のシナリオID: {}\n\
         照合: schema covered <記録のファイル> {}",
        ids.join(", "),
        decl_id
    )
}
