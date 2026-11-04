//! Concrete とインスタンスを書く手続き（業務領域「書いて検証する」の書くモジュールの手続き）。
//!
//! - UC1 作成する ・ UC2 更新する（機械）：値か JSON Patch から書く中身を作り、形と集約の書く条件を確かめてから書き、
//!   前後の中身から求めた差分と一緒に操作を記録する。書く手前で断ったものは記録せず、理由をその場で返す。
//! - SAS を通らない変更の取り込み（UC1 の変形 ・ UC2 の --adopt）：ファイルは書かず、いまの中身を形と書く条件に照らして、
//!   source = outside の操作として記録する。更新の取り込みの欄ごとの前の値は、記録のどれかの事後と同じ中身のコミットから取る。
//! - UC3 削除する（機械）：対象を消し、消す前の状態を記録する（delete.rs）。
//! - 対象の見分けは writing-domain の TargetKind、更新のあとに入力が変わった前の判定は validation-domain の判定の基準で決める。
//!   文書の読み書きと形の検査は、このモジュールのポート（ports.rs）を使う。

mod delete;
mod patch;
mod semantic;

pub use delete::{DeleteRequest, Deletion, delete};
pub use semantic::{SemanticRequest, semantic_procedure};

use crate::ports::{ChangeLog, CommitReader, DocumentStore, SchemaValidator};
use serde_json::Value;
use std::collections::BTreeSet;
use validation_domain::services::{
    EXAMPLE_RELATED, Judged, concrete_rules, example_related, instance_rules, reads_hash,
};
use validation_domain::values::{By, RuleResult, Verdict};
use writing_domain::aggregates::{ConcreteSchema, Entry, Instance};
use writing_domain::services::{Resolved, SchemaIndex, Unresolved, concrete_of, refs};
use writing_domain::values::{
    Caller, Change, ChangeRow, ContentHash, Diff, FieldChange, FieldRow, SchemaPlaces,
    StoredChange, TargetKind, WriteRecord, unrecorded_since,
};

/// 操作を始められなかった ・ 書かなかった理由。断った操作は記録しない（理由 ・ 案内 ・ 間に入った変更をその場で返す）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（usage ・ schema_not_listed ・ not_found ・ not_json ・ no_schema ・ unknown_schema ・ duplicate_schema ・ exists ・
    /// invalid_input ・ request_reused ・ recorded ・ not_recorded ・ nothing_to_adopt ・ store ・ validator ・ versions ・ log、
    /// 書く手前の判定で断ったもの：invalid_value ・ conflict ・ patch_failed ・ human_field ・ published_changed、
    /// 止めた削除：has_dependents、意味で書く手順を組めないもの：reads_cycle ・ ref_too_large）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// ハッシュが違って断ったとき（conflict）の、間に入った変更
    pub interleaved: Option<Box<Interleaved>>,
    /// 次にすることの案内（SAS を通らない変更を取り込むコマンドなど）
    pub hint: Option<Box<String>>,
    /// 削除を止めた理由になったもの（Concrete に従うインスタンス）
    pub blocked: Option<Box<Vec<Blocked>>>,
    /// 同じ $id を持つファイルの経路（duplicate_schema）
    pub paths: Option<Box<Vec<String>>>,
}

/// 削除を止めた理由になった文書1つと、それを片付けるコマンドの案内。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocked {
    /// 経路
    pub path: String,
    /// 決まった名前（follows）
    pub code: &'static str,
    /// 理由
    pub message: String,
    /// 次に打つコマンド
    pub hint: String,
}

/// 対象の経路を付けて断る。
#[must_use]
pub fn refused(code: &'static str, path: &str, message: String) -> Refused {
    Refused {
        code,
        path: Some(path.into()),
        message,
        ..Refused::default()
    }
}

/// 記録を読み書きできなかったことを、断った理由にする。
#[must_use]
pub fn log_refused(e: String) -> Refused {
    Refused {
        code: "log",
        message: e,
        ..Refused::default()
    }
}

/// 渡されたスキーマの置き場所のファイルを1件ずつ読み、$id と経路だけを索引に残す。
///
/// # Errors
/// ファイルを読めないとき。
fn index_of(store: &dyn DocumentStore, schemas: &[String]) -> Result<SchemaIndex, Refused> {
    let mut index = SchemaIndex::new();
    for p in schemas {
        let text = store.read(p).map_err(|e| refused("store", p, e))?;
        index.add(p, &text);
    }
    Ok(index)
}

/// 索引から $id の文書を1つ読む。from は $ref で指した文書の $id。
///
/// # Errors
/// 置き場所に無い ・ 2つ以上ある ・ 読めない ・ JSON でないとき。
fn read_one(
    index: &SchemaIndex,
    store: &dyn DocumentStore,
    id: &str,
    from: Option<&str>,
    target: &str,
) -> Result<(String, Value), Refused> {
    let path = index
        .path_of(id, from)
        .map_err(|u| unresolved(Some(target), u))?
        .to_string();
    let text = store.read(&path).map_err(|e| refused("store", &path, e))?;
    let value = serde_json::from_str(&text)
        .map_err(|e| refused("store", &path, format!("JSON として読めない：{e}")))?;
    Ok((path, value))
}

/// $id が id の文書（中身は value）と、その $ref の先（同じ置き場所から引く。入れて出すスキーマは引かない）を、
/// 形の検査に渡す並びにする。
///
/// # Errors
/// $ref の先が置き場所に無い ・ 2つ以上ある ・ 読めないとき。
fn with_refs(
    index: &SchemaIndex,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    target: &str,
    id: &str,
    value: Value,
) -> Result<Vec<(String, Value)>, Refused> {
    let mut seen: BTreeSet<String> = BTreeSet::from([id.to_string()]);
    let mut queue: Vec<(String, String)> = refs(&value)
        .into_iter()
        .map(|r| (r, id.to_string()))
        .collect();
    let mut more = vec![(id.to_string(), value)];
    while let Some((next, from)) = queue.pop() {
        if validator.shipped(&next).is_some() || !seen.insert(next.clone()) {
            continue;
        }
        let (_, doc) = read_one(index, store, &next, Some(&from), target)?;
        queue.extend(refs(&doc).into_iter().map(|r| (r, next.clone())));
        more.push((next, doc));
    }
    Ok(more)
}

/// 置き場所から引けなかった理由を、断った理由にする。
fn unresolved(path: Option<&str>, u: Unresolved) -> Refused {
    Refused {
        code: u.code(),
        path: path.map(Into::into),
        message: u.message().to_string(),
        paths: (!u.paths().is_empty()).then(|| Box::new(u.paths().to_vec())),
        ..Refused::default()
    }
}

/// インスタンスが従う Concrete（$id が declared）と $ref の先を、渡された置き場所から引く。
///
/// # Errors
/// 置き場所に無い ・ 2つ以上ある ・ Concrete でない ・ 読めないとき。
fn resolve(
    index: &SchemaIndex,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    path: &str,
    declared: &str,
) -> Result<Resolved, Refused> {
    let (at, value) = read_one(index, store, declared, None, path)?;
    let concrete = concrete_of(declared, &at, &value).map_err(|u| unresolved(Some(path), u))?;
    let more = with_refs(index, store, validator, path, declared, value)?;
    Ok(Resolved::new(at, concrete, more))
}

/// $schema の申告から対象の種類を見分ける。見分けられなければ、取りに行かずに断る理由。
fn classify(path: &str, declared: &str) -> Result<TargetKind, Refused> {
    TargetKind::classify(declared).map_err(|u| refused(u.code(), path, u.message().to_string()))
}

/// 対象の種類と、インスタンスなら従う Concrete。
struct Target {
    kind: TargetKind,
    concrete: Option<Resolved>,
}

impl Target {
    /// 形の検査で引ける文書（インスタンスなら Concrete と $ref の先。Concrete なら無い）。
    fn more(&self) -> &[(String, Value)] {
        self.concrete.as_ref().map_or(&[], Resolved::more)
    }
}

/// 対象の種類を見分け、インスタンスなら従う Concrete を渡された置き場所から引く。
///
/// # Errors
/// 見分けられない ・ 置き場所から引けないとき。
fn target_of_declared(
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    schemas: &[String],
    path: &str,
    declared: &str,
) -> Result<Target, Refused> {
    let kind = classify(path, declared)?;
    let concrete = match kind {
        TargetKind::Concrete => None,
        TargetKind::Instance => Some(resolve(
            &index_of(store, schemas)?,
            store,
            validator,
            path,
            declared,
        )?),
    };
    Ok(Target { kind, concrete })
}

/// 形（JSON Schema）の外れ（JSON Pointer と理由）。Concrete は申告した Abstract と方言の両方、インスタンスは従う Concrete に照らす。
///
/// # Errors
/// バリデーターが動かないとき。
fn form_violations(
    validator: &dyn SchemaValidator,
    kind: TargetKind,
    declared: &str,
    value: &Value,
    more: &[(String, Value)],
) -> Result<Vec<(String, String)>, String> {
    let mut broken: Vec<(String, String)> = Vec::new();
    for schema in kind.form_schemas(declared) {
        for v in validator.check(schema, value, more)? {
            if !broken.contains(&v) {
                broken.push(v);
            }
        }
    }
    Ok(broken)
}

/// 書く前に、呼ぶ側が前提にしたハッシュといまのハッシュが違ったときに返す「間に入った変更」。
/// 呼ぶ側（人か AI）は、間に入った操作の理由と変わった欄を読んで、直し方を考え直す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interleaved {
    /// 呼ぶ側が前提にしたハッシュ（--if-match か、最後に記録した書いた操作の事後のハッシュ）
    pub expected: String,
    /// いまの中身のハッシュ
    pub current: String,
    /// 前提のハッシュのあとに記録された、対象を書いた操作（古い順）
    pub operations: Vec<WriteRecord>,
    /// いまの中身が、この経路の記録の事後のどれでもない（SAS を通らない変更があった）
    pub unrecorded: bool,
    /// 最後に記録した、対象を書いた操作（無ければ None）
    pub last_recorded: Option<WriteRecord>,
}

/// 前提のハッシュからいまのハッシュまでに記録された操作を、事前と事後のハッシュをたどって組む。
/// writes は経路の対象を書いた操作（記録した順）。前提のハッシュが記録のどこにも無いときは、記録した操作をすべて返す。
#[must_use]
pub fn interleaved(writes: &[WriteRecord], expected: &str, current: &str) -> Interleaved {
    let start = writes
        .iter()
        .rposition(|w| w.row.after.as_deref() == Some(expected))
        .map(|i| i + 1)
        .or_else(|| {
            writes
                .iter()
                .position(|w| w.row.before.as_deref() == Some(expected))
        })
        .unwrap_or(0);
    let operations: Vec<WriteRecord> = writes[start.min(writes.len())..].to_vec();
    // 外の変更（Semantic Agent Schema を通らない変更）かは、検証と同じ決まり（このワークツリーの最後の事後、まだ書いていなければ記録のどれかの中身）で見る
    let unrecorded = unrecorded_since(writes, current).is_some();
    Interleaved {
        expected: expected.to_string(),
        current: current.to_string(),
        unrecorded,
        last_recorded: writes.last().cloned(),
        operations,
    }
}

/// 書いた作成 ・ 更新 ・ 削除を1件記録し、打ったセッションの ID と操作の ID を返す。断った操作は記録しない。
/// 打ったセッションの行は、入口が先に足している。
///
/// # Errors
/// 置き場所が行を拒んだとき。
fn record_change(
    caller: &Caller,
    log: &mut dyn ChangeLog,
    row: &ChangeRow,
) -> Result<(Option<String>, String), Refused> {
    let session = caller.session_id();
    let operation = log.append_change(session, row).map_err(log_refused)?;
    Ok((session.map(ToString::to_string), operation))
}

/// Abstract の語彙（入れ子に書ける x- の名前と、その値の形の対応を持つ）。
const VOCABULARY: &str = "urn:semantic-agent-schema:published-language:vocabulary:v1";

/// いまの中身で、対象に当たる rule ごとの読んだ所のハッシュ（rule の id ・ 当たる所 ・ ハッシュ）。形の検査は除く。
/// 更新のあと、前の判定のうち判定し直しが要るものを決めるのに使う（検証の --stale と同じ基準：validation-domain の reads_hash）。
///
/// # Errors
/// 型が決まらない ・ 申告した Abstract をバリデーターが持たないとき。
fn reads_now(
    path: &str,
    value: &Value,
    target: &Target,
    validator: &dyn SchemaValidator,
) -> Result<Vec<(String, String, String)>, Refused> {
    let declared = value
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let judged: Vec<Judged> = match classify(path, &declared)? {
        TargetKind::Concrete => {
            let abstract_doc = validator.shipped(&declared).ok_or_else(|| {
                refused(
                    "validator",
                    path,
                    format!("{declared} をバリデーターが持たない"),
                )
            })?;
            let concrete = ConcreteSchema::read(value)
                .ok_or_else(|| refused("validator", path, "Concrete として読めない".to_string()))?;
            let vocabulary = validator.shipped(VOCABULARY);
            concrete_rules(&abstract_doc, vocabulary.as_ref(), &concrete)
        }
        TargetKind::Instance => {
            let concrete_doc = target
                .concrete
                .as_ref()
                .map(|c| c.concrete().document().clone())
                .unwrap_or_default();
            instance_rules(&concrete_doc, value, target.more())
        }
    };
    Ok(judged
        .into_iter()
        .map(|(r, at, definition)| {
            (
                r.rule().to_string(),
                r.pointer().to_string(),
                reads_hash(value, &at, definition.as_ref())
                    .as_str()
                    .to_string(),
            )
        })
        .collect())
}

/// 作成か更新か。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// 作った
    Created,
    /// 直した
    Updated,
}

impl Written {
    /// 出力に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
        }
    }

    const fn kind(self) -> &'static str {
        match self {
            Self::Created => "create",
            Self::Updated => "update",
        }
    }
}

/// この変更で入力が変わった、前の検証の判定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    /// rule の id
    pub rule: String,
    /// 判定した所
    pub pointer: String,
    /// 判定を記録した検証の操作の ID
    pub operation: String,
}

/// Concrete の更新で、新しい Concrete の形に従わなくなったインスタンス。
#[derive(Debug, Clone, PartialEq)]
pub struct Violated {
    /// 経路
    pub path: String,
    /// 形の外れ
    pub results: Vec<RuleResult>,
}

/// Concrete を直す ・ 消すときに、従うインスタンスかを見た文書1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// 経路
    pub path: String,
    /// 見たときの中身のハッシュ
    pub hash: ContentHash,
    /// どこから見たか（range ＝ 渡された範囲 ・ record ＝ 記録）
    pub from: &'static str,
    /// その Concrete に従うインスタンスか
    pub follows: bool,
}

/// 渡された範囲（--range）と、記録でその Concrete に従うと書いた文書を読む。own は直す ・ 消す Concrete の経路（見ない）。
/// 範囲の文書は読めなければ断る。記録の文書は、いま無ければ見ない（見た文書だけを返す）。
///
/// # Errors
/// 範囲の文書を読めない ・ JSON でない ・ 記録を読めないとき。
fn range_docs(
    store: &dyn DocumentStore,
    log: &dyn ChangeLog,
    own: &str,
    id: &str,
    range: &[String],
) -> Result<Vec<(Checked, Value)>, Refused> {
    let mut out: Vec<(Checked, Value)> = Vec::new();
    let recorded = log.instances_of(id).map_err(log_refused)?;
    let from_range = range.iter().map(|p| (p, "range"));
    let from_record = recorded.iter().map(|p| (p, "record"));
    for (p, from) in from_range.chain(from_record) {
        if p == own || out.iter().any(|(c, _)| c.path == *p) {
            continue;
        }
        let text = match store.read(p) {
            Ok(t) => t,
            Err(e) if from == "range" => return Err(refused("not_found", p, e)),
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) if from == "range" => {
                return Err(refused("not_json", p, format!("JSON として読めない：{e}")));
            }
            Err(_) => continue,
        };
        let follows = Instance::read(&value).is_some_and(|i| i.concrete() == id);
        out.push((
            Checked {
                path: p.clone(),
                hash: ContentHash::of(text.as_bytes()),
                from,
                follows,
            },
            value,
        ));
    }
    Ok(out)
}

/// Concrete を直す ・ 消すときの範囲。Concrete なら必ず要り、インスタンスには渡さない。
///
/// # Errors
/// Concrete で範囲が無い ・ インスタンスに範囲を渡したとき（usage）。
fn range_for<'a>(
    kind: TargetKind,
    path: &str,
    range: Option<&'a [String]>,
    verb: &str,
) -> Result<&'a [String], Refused> {
    match (kind, range) {
        (TargetKind::Concrete, Some(r)) => Ok(r),
        (TargetKind::Concrete, None) => Err(refused(
            "usage",
            path,
            format!(
                "Concrete を{verb}ときは、従うインスタンスを確かめる範囲を --range <経路>... で渡す（渡された範囲と記録で確かめる）"
            ),
        )),
        (TargetKind::Instance, Some(_)) => Err(refused(
            "usage",
            path,
            "--range は Concrete を直す ・ 消すときだけ渡す".to_string(),
        )),
        (TargetKind::Instance, None) => Ok(&[]),
    }
}

/// 作成か更新の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeResult {
    /// 打ったセッションの ID（人なら None）
    pub session: Option<String>,
    /// 操作の ID
    pub operation: String,
    /// 作ったか直したか
    pub written: Written,
    /// 対象の種類
    pub kind: TargetKind,
    /// 経路
    pub path: String,
    /// 従う schema の $id（インスタンスなら Concrete、Concrete なら Abstract の版）
    pub schema: String,
    /// 事前のハッシュ（作成では None）
    pub before: Option<ContentHash>,
    /// 事後のハッシュ
    pub after: ContentHash,
    /// 前後の中身から求めた差分
    pub diff: Diff,
    /// 入力が変わった前の判定
    pub stale: Vec<Stale>,
    /// 外れたインスタンス（Concrete の更新のとき）
    pub violated: Vec<Violated>,
    /// Concrete の更新で、従うインスタンスかを見た文書（渡された範囲と記録から）
    pub checked: Vec<Checked>,
    /// 同じ request_id の操作がすでにあり、書かずにそれを返したか
    pub replayed: bool,
    /// SAS を通らない変更を取り込んだ操作か（source = outside）
    pub outside: bool,
    /// 更新の取り込みで、欄ごとの前の値を取ったコミット。そのコミットが無ければ None（差分の欄は前の値を持たない）
    pub before_commit: Option<String>,
}

/// 作成の依頼。
#[derive(Debug, Clone, PartialEq)]
pub struct CreateRequest {
    /// 作る経路
    pub path: String,
    /// 従う Concrete の $id（Concrete を作るときは Abstract の版）
    pub concrete: String,
    /// 書く値。$schema が無ければ concrete を入れる
    pub value: Value,
    /// スキーマの置き場所（従う Concrete と $ref の先を $id で引く。返すコマンドにも引き継ぐ）
    pub schemas: SchemaPlaces,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 理由（なぜ作るか）。記録に残し、間に入った変更として後の書き手に返す
    pub reason: Option<String>,
}

/// 更新の依頼。
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateRequest {
    /// 直す経路
    pub path: String,
    /// JSON Patch（RFC 6902）の操作の並び
    pub patch: Value,
    /// 直す前に読んだ中身のハッシュ。無ければ、Semantic Agent Schema が最後に書いた事後のハッシュと比べる
    pub if_match: Option<String>,
    /// スキーマの置き場所（従う Concrete と $ref の先を $id で引く。返すコマンドにも引き継ぐ）
    pub schemas: SchemaPlaces,
    /// Concrete を直すときに、従うインスタンスを確かめる範囲（--range の経路）
    pub range: Option<Vec<String>>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 理由（なぜ直すか）。記録に残し、間に入った変更として後の書き手に返す
    pub reason: Option<String>,
}

/// 書く文書の文字列（1字下げの JSON と、末尾の改行。リポジトリの文書の書き方に合わせる）。
fn to_text(value: &Value) -> String {
    let mut out = String::new();
    pretty(value, 0, &mut out);
    out.push('\n');
    out
}

fn pretty(value: &Value, depth: usize, out: &mut String) {
    let pad = |n: usize| " ".repeat(n);
    match value {
        Value::Object(m) if !m.is_empty() => {
            out.push_str("{\n");
            for (i, (k, v)) in m.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                out.push_str(&Value::String(k.clone()).to_string());
                out.push_str(": ");
                pretty(v, depth + 1, out);
                out.push_str(if i + 1 < m.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(depth));
            out.push('}');
        }
        Value::Array(a) if !a.is_empty() => {
            out.push_str("[\n");
            for (i, v) in a.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                pretty(v, depth + 1, out);
                out.push_str(if i + 1 < a.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(depth));
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// 書く手前の判定に要るもの：形を照らすバリデーターと、リポジトリの Concrete。
struct Judge<'a> {
    validator: &'a dyn SchemaValidator,
    more: &'a [(String, Value)],
}

/// 人が決める欄の所と default。前と後の中身のどちらかで、x-decided-by: human が当たった所（入れ子 ・ $ref の先も）。
fn human_fields(
    judge: &Judge<'_>,
    concrete: &str,
    values: &[&Value],
) -> Result<Vec<(String, Option<Value>)>, String> {
    let mut out: Vec<(String, Option<Value>)> = Vec::new();
    for v in values {
        for (at, a) in judge.validator.annotations(concrete, v, judge.more)? {
            if a.get("x-decided-by").and_then(Value::as_str) == Some("human")
                && !out.iter().any(|(p, _)| *p == at)
            {
                out.push((at, a.get("default").cloned()));
            }
        }
    }
    Ok(out)
}

/// 集約の書く条件。
fn check_write(
    judge: &Judge<'_>,
    kind: TargetKind,
    before: Option<&Value>,
    after: &Value,
    concrete: Option<&ConcreteSchema>,
    entry: Entry,
) -> Result<(), (&'static str, String)> {
    let r = match kind {
        TargetKind::Concrete => match ConcreteSchema::read(after) {
            Some(c) => {
                // rule の例の related は、その rule の related_fields にある所だけ（組み込みの検査 example-related）
                let broken: Vec<String> = example_related(&c)
                    .into_iter()
                    .filter(|r| r.verdict() == Verdict::Violated)
                    .map(|r| format!("{}：{}", r.pointer(), r.reason()))
                    .collect();
                if !broken.is_empty() {
                    return Err((
                        "invalid_value",
                        format!("書かない（{EXAMPLE_RELATED}）：{}", broken.join(" ／ ")),
                    ));
                }
                c.check_write(before.and_then(ConcreteSchema::read).as_ref())
            }
            None => return Ok(()),
        },
        TargetKind::Instance => {
            match (Instance::read(after), concrete.and_then(ConcreteSchema::id)) {
                (Some(i), Some(id)) => {
                    let values: Vec<&Value> =
                        before.into_iter().chain(std::iter::once(after)).collect();
                    let human = human_fields(judge, id, &values).map_err(|e| ("validator", e))?;
                    i.check_write(before.and_then(Instance::read).as_ref(), &human, entry)
                }
                _ => return Ok(()),
            }
        }
    };
    r.map_err(|v| (v.code(), v.message().to_string()))
}

fn field_rows(diff: &Diff) -> Vec<FieldRow> {
    diff.changes()
        .iter()
        .map(|c| FieldRow {
            pointer: c.pointer().to_string(),
            change: c.change().as_str().to_string(),
            before: c.before().map(Value::to_string),
            after: c.after().map(Value::to_string),
            before_unavailable: c.before_unavailable(),
        })
        .collect()
}

/// 記録にある操作の対象の種類と従う型。記録には持たないので、いまのファイルの $schema か、
/// 記録した欄（作成と削除は根の欄をすべて持つ）の $schema から見分ける。
fn target_of(store: &dyn DocumentStore, path: &str, fields: &[FieldRow]) -> (TargetKind, String) {
    let from_file = store
        .read(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            v.get("$schema")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let declared = from_file
        .or_else(|| {
            fields
                .iter()
                .filter(|f| f.pointer == "/$schema")
                .find_map(|f| f.after.as_deref().or(f.before.as_deref()))
                .and_then(|t| serde_json::from_str::<String>(t).ok())
        })
        .unwrap_or_default();
    let kind = if ConcreteSchema::declares_abstract(&serde_json::json!({ "$schema": declared })) {
        TargetKind::Concrete
    } else {
        TargetKind::Instance
    };
    (kind, declared)
}

/// 記録にある同じ依頼の操作を、結果に戻す。source は依頼の種類（sas ・ outside）。
fn replay(
    stored: StoredChange,
    written: Written,
    source: &str,
    path: &str,
    store: &dyn DocumentStore,
) -> Result<ChangeResult, Refused> {
    let row = stored.row;
    if row.kind != written.kind() || row.source != source || row.target_path != path {
        return Err(refused(
            "request_reused",
            path,
            format!(
                "request_id は {} の {} に使った。別の依頼には別の request_id を使う",
                row.target_path, row.kind
            ),
        ));
    }
    let (kind, concrete) = target_of(store, path, &row.fields);
    let parse = |s: &Option<String>| s.as_deref().and_then(|t| serde_json::from_str(t).ok());
    let changes = row
        .fields
        .iter()
        .filter_map(|f| {
            Some(FieldChange::new(
                &f.pointer,
                Change::parse(&f.change)?,
                parse(&f.before),
                parse(&f.after),
            ))
            .map(|c| match (f.before_unavailable, c.after()) {
                (true, Some(a)) => FieldChange::without_before(&f.pointer, a.clone()),
                _ => c,
            })
        })
        .collect();
    let after = row
        .after
        .as_deref()
        .and_then(ContentHash::parse)
        .ok_or_else(|| {
            log_refused(format!(
                "操作 {} の事後のハッシュを読めない",
                stored.operation
            ))
        })?;
    Ok(ChangeResult {
        session: stored.session,
        operation: stored.operation,
        written,
        kind,
        path: row.target_path,
        schema: concrete,
        before: row.before.as_deref().and_then(ContentHash::parse),
        after,
        diff: Diff::from_changes(changes),
        stale: Vec::new(),
        violated: Vec::new(),
        checked: Vec::new(),
        replayed: true,
        outside: row.source == "outside",
        before_commit: None,
    })
}

fn violations_message(broken: &[(String, String)]) -> String {
    broken
        .iter()
        .map(|(at, why)| format!("{}：{why}", if at.is_empty() { "/" } else { at }))
        .collect::<Vec<_>>()
        .join(" ／ ")
}

/// UC1 作成する（機械）：値を型に照らし、集約の書く条件を確かめてから書き、作成の操作を記録する。
///
/// # Errors
/// 依頼を読めない ・ 従う型が決まらない ・ すでに在る（何も記録しない）。
/// 形に外れる ・ 書く条件に外れる（書かず、記録もしない）。書けない ・ 記録できないとき。
pub fn create(
    request: &CreateRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ChangeLog,
) -> Result<ChangeResult, Refused> {
    let path = request.path.as_str();
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replay(stored, Written::Created, "sas", path, store);
    }
    let mut value = request.value.clone();
    let Some(object) = value.as_object_mut() else {
        return Err(refused(
            "invalid_input",
            path,
            "--value は JSON のオブジェクトで渡す".to_string(),
        ));
    };
    match object.get("$schema").and_then(Value::as_str) {
        None => {
            // $schema を先頭に置く（ほかの欄の並びは渡された順のまま）
            let mut first = serde_json::Map::new();
            first.insert(
                "$schema".to_string(),
                Value::String(request.concrete.clone()),
            );
            first.append(object);
            *object = first;
        }
        Some(s) if s == request.concrete => {}
        Some(s) => {
            return Err(refused(
                "invalid_input",
                path,
                format!(
                    "値の $schema（{s}）が --concrete（{}）と違う",
                    request.concrete
                ),
            ));
        }
    }
    if store.read(path).is_ok() {
        return Err(refused(
            "exists",
            path,
            format!("{path} はすでに在る。直すときは update を使う"),
        ));
    }
    let target = target_of_declared(
        store,
        validator,
        request.schemas.files(),
        path,
        &request.concrete,
    )?;
    let kind = target.kind;
    let row = ChangeRow {
        kind: Written::Created.kind().to_string(),
        source: "sas".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: None,
        after: None,
        reason: request.reason.clone(),
        schema: Some(request.concrete.clone()),
        fields: Vec::new(),
    };
    let broken = form_violations(validator, kind, &request.concrete, &value, target.more())
        .map_err(|e| refused("validator", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!("型に従わない：{}", violations_message(&broken)),
        ));
    }
    if let Err((code, message)) = check_write(
        &Judge {
            validator,
            more: target.more(),
        },
        kind,
        None,
        &value,
        target.concrete.as_ref().map(Resolved::concrete),
        caller.entry(),
    ) {
        return Err(refused(code, path, message));
    }
    let text = to_text(&value);
    store
        .write(path, &text)
        .map_err(|e| refused("store", path, e))?;
    let after = ContentHash::of(text.as_bytes());
    let diff = Diff::between(None, &value);
    let (session, operation) = record_change(
        caller,
        log,
        &ChangeRow {
            after: Some(after.as_str().to_string()),
            fields: field_rows(&diff),
            ..row
        },
    )?;
    Ok(ChangeResult {
        session,
        operation,
        written: Written::Created,
        kind,
        path: path.to_string(),
        schema: request.concrete.clone(),
        before: None,
        after,
        diff,
        stale: Vec::new(),
        violated: Vec::new(),
        checked: Vec::new(),
        replayed: false,
        outside: false,
        before_commit: None,
    })
}

/// UC2 更新する（機械）：いまの中身のハッシュを確かめ、JSON Patch を当て、型と集約の書く条件を確かめてから書き、
/// 前後の中身から求めた差分と一緒に更新の操作を記録する。前の判定のうち入力が変わったものと、
/// Concrete の更新で外れたインスタンスを返す。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// ハッシュが違う ・ 差分を当てられない ・ 形や書く条件に外れる（書かず、記録もしない）。書けない ・ 記録できないとき。
pub fn update(
    request: &UpdateRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ChangeLog,
) -> Result<ChangeResult, Refused> {
    let path = request.path.as_str();
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replay(stored, Written::Updated, "sas", path, store);
    }
    let text = store
        .read(path)
        .map_err(|e| refused("not_found", path, e))?;
    let before: Value = serde_json::from_str(&text)
        .map_err(|e| refused("not_json", path, format!("JSON として読めない：{e}")))?;
    let declared = before
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let target = target_of_declared(store, validator, request.schemas.files(), path, &declared)?;
    let kind = target.kind;
    let range = range_for(kind, path, request.range.as_deref(), "直す")?;
    let current = ContentHash::of(text.as_bytes());
    let row = ChangeRow {
        kind: Written::Updated.kind().to_string(),
        source: "sas".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: Some(current.as_str().to_string()),
        after: None,
        reason: request.reason.clone(),
        schema: Some(declared.clone()),
        fields: Vec::new(),
    };
    // 比べるハッシュ：呼ぶ側が読んだハッシュ（--if-match）か、いまの中身が記録のどの事後とも違うときの最後の事後のハッシュ。
    // SAS が一度も書いていない ・ いまの中身が記録のどれかの事後なら比べるハッシュが無く、そのまま書く
    let (expected, from) = match &request.if_match {
        Some(hash) => (Some(hash.clone()), "--if-match の"),
        None => (
            unrecorded_since(&log.writes(path).map_err(log_refused)?, current.as_str())
                .and_then(|w| w.row.after.clone()),
            "このワークツリーで Semantic Agent Schema が最後に書いた事後の",
        ),
    };
    if let Some(expected) = expected.filter(|e| e != current.as_str()) {
        let writes = log.writes(path).map_err(log_refused)?;
        let between = interleaved(&writes, &expected, current.as_str());
        let mut r = refused(
            "conflict",
            path,
            format!(
                "いまの中身のハッシュは {}。{from}ハッシュ {expected} と違うので直さない。間に入った変更（interleaved）を読んでから直す{}",
                current.as_str(),
                if between.unrecorded {
                    "。SAS を通らない変更がある"
                } else {
                    ""
                }
            ),
        );
        r.hint = Some(Box::new(if between.unrecorded {
            adopt_hint(path, &request.schemas, request.range.as_deref(), caller)
        } else {
            reapply_hint(
                path,
                current.as_str(),
                &request.schemas,
                request.range.as_deref(),
                caller,
            )
        }));
        r.interleaved = Some(Box::new(between));
        return Err(r);
    }
    let after = match patch::apply(&before, &request.patch) {
        Ok(v) => v,
        Err(e) => {
            return Err(refused(
                "patch_failed",
                path,
                format!("JSON Patch を当てられない（どれも当てていない）：{e}"),
            ));
        }
    };
    // Concrete の申告する Abstract の版を、SAS が入れて出す一覧に無い版へ変えるパッチは、形に照らす前に断る
    // （版を移す手続きは、Abstract の新しい版を出すときに作る）
    if kind == TargetKind::Concrete
        && let Some(to) = after.get("$schema").and_then(Value::as_str)
        && to != declared
        && let Err(u) = TargetKind::classify(to)
        && u.code() == "schema_not_listed"
    {
        return Err(refused(
            u.code(),
            path,
            format!("{}。パッチを当てていない（対象は変えない）", u.message()),
        ));
    }
    let broken = form_violations(validator, kind, &declared, &after, target.more())
        .map_err(|e| refused("validator", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!("当てた結果が型に従わない：{}", violations_message(&broken)),
        ));
    }
    if let Err((code, message)) = check_write(
        &Judge {
            validator,
            more: target.more(),
        },
        kind,
        Some(&before),
        &after,
        target.concrete.as_ref().map(Resolved::concrete),
        caller.entry(),
    ) {
        return Err(refused(code, path, message));
    }
    // Concrete を直すときは、渡された範囲と記録の文書を、書く前に読む（新しい中身を同じ $id の Concrete として照らす）
    let followers =
        match ConcreteSchema::read(&before).and_then(|c| c.id().map(ToString::to_string)) {
            Some(id) => Some(Followers::read(
                request.schemas.files(),
                store,
                validator,
                log,
                path,
                &id,
                range,
                &after,
            )?),
            None => None,
        };
    let new_text = to_text(&after);
    store
        .write(path, &new_text)
        .map_err(|e| refused("store", path, e))?;
    let after_hash = ContentHash::of(new_text.as_bytes());
    let diff = Diff::between(Some(&before), &after);
    let (session, operation) = record_change(
        caller,
        log,
        &ChangeRow {
            after: Some(after_hash.as_str().to_string()),
            // 更新で $schema を変えたら、書いたあとの文書が従う schema
            schema: after
                .get("$schema")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .or(row.schema.clone()),
            fields: field_rows(&diff),
            ..row
        },
    )?;

    let stale = stale_judgments(log, path, &after, &target, validator)?;
    let (violated, checked) = match followers {
        Some(f) => f.check(validator)?,
        None => (Vec::new(), Vec::new()),
    };
    Ok(ChangeResult {
        session,
        operation,
        written: Written::Updated,
        kind,
        path: path.to_string(),
        schema: declared,
        before: Some(current),
        after: after_hash,
        diff,
        stale,
        violated,
        checked,
        replayed: false,
        outside: false,
        before_commit: None,
    })
}

/// 前の判定のうち、読んだ所がいまの中身と変わったもの（--stale と同じ基準。rule の related_fields と rule の文を含む）。
/// 形の判定は書く前に照らし直したので除く。
fn stale_judgments(
    log: &dyn ChangeLog,
    path: &str,
    value: &Value,
    target: &Target,
    validator: &dyn SchemaValidator,
) -> Result<Vec<Stale>, Refused> {
    let now = reads_now(path, value, target, validator)?;
    Ok(log
        .last_judgments(path)
        .map_err(log_refused)?
        .into_iter()
        .filter(|j| j.rule.rule != "json-schema")
        .filter(|j| {
            !now.iter()
                .any(|(r, p, h)| *r == j.rule.rule && *p == j.rule.pointer && *h == j.rule.reads)
        })
        .map(|j| Stale {
            rule: j.rule.rule,
            pointer: j.rule.pointer,
            operation: j.operation,
        })
        .collect())
}

/// Concrete を直すときに見る、渡された範囲と記録の文書と、新しい中身で照らす文書（Concrete と $ref の先）。
struct Followers {
    id: String,
    docs: Vec<(Checked, Value)>,
    more: Vec<(String, Value)>,
}

impl Followers {
    /// 範囲と記録の文書を読み、新しい中身（after）の $ref の先を渡された置き場所から引く。書く前に呼ぶ。
    ///
    /// # Errors
    /// 範囲の文書を読めない ・ $ref の先を引けない ・ 記録を読めないとき。
    #[allow(clippy::too_many_arguments)]
    fn read(
        schemas: &[String],
        store: &dyn DocumentStore,
        validator: &dyn SchemaValidator,
        log: &dyn ChangeLog,
        path: &str,
        id: &str,
        range: &[String],
        after: &Value,
    ) -> Result<Self, Refused> {
        let docs = range_docs(store, log, path, id, range)?;
        let more = if docs.iter().any(|(c, _)| c.follows) {
            with_refs(
                &index_of(store, schemas)?,
                store,
                validator,
                path,
                id,
                after.clone(),
            )?
        } else {
            Vec::new()
        };
        Ok(Self {
            id: id.to_string(),
            docs,
            more,
        })
    }

    /// 従うインスタンスを新しい形に照らし、外れたものと、見た文書を返す。
    ///
    /// # Errors
    /// バリデーターが動かないとき。
    fn check(
        self,
        validator: &dyn SchemaValidator,
    ) -> Result<(Vec<Violated>, Vec<Checked>), Refused> {
        let mut violated = Vec::new();
        for (c, value) in &self.docs {
            if !c.follows {
                continue;
            }
            let broken = validator
                .check(&self.id, value, &self.more)
                .map_err(|e| refused("validator", &c.path, e))?;
            if !broken.is_empty() {
                violated.push(Violated {
                    path: c.path.clone(),
                    results: broken
                        .iter()
                        .map(|(at, why)| {
                            RuleResult::new("json-schema", at, By::Machine, Verdict::Violated, why)
                        })
                        .collect(),
                });
            }
        }
        Ok((violated, self.docs.into_iter().map(|(c, _)| c).collect()))
    }
}

/// 返すコマンドの1行。空白や記号を含む引数は単一引用符で囲む。<…> は呼ぶ側が埋める所なので、そのまま出す。
fn command_line(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            let plain = !a.is_empty()
                && a.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:@=+,".contains(c));
            let placeholder = a.starts_with('<') && a.ends_with('>');
            if plain || placeholder {
                a.clone()
            } else {
                format!("'{}'", a.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 返すコマンド：semantic-agent-schema と head の引数に、受け取った --schemas ・ --range と、誰が打ったかを引き継ぐ。
fn carried(
    head: &[&str],
    schemas: Option<&SchemaPlaces>,
    range: Option<&[String]>,
    caller: &Caller,
) -> String {
    let mut args: Vec<String> = std::iter::once("semantic-agent-schema")
        .chain(head.iter().copied())
        .map(ToString::to_string)
        .collect();
    if let Some(s) = schemas {
        args.extend(s.args());
    }
    if let Some(r) = range {
        args.push("--range".to_string());
        args.extend(r.iter().cloned());
    }
    args.extend(caller.args());
    command_line(&args)
}

/// Semantic Agent Schema を通らない変更を取り込むコマンドの案内。
fn adopt_hint(
    path: &str,
    schemas: &SchemaPlaces,
    range: Option<&[String]>,
    caller: &Caller,
) -> String {
    format!(
        "理由を付けて取り込む：{}",
        carried(
            &["update", path, "--adopt", "--reason", "<理由>"],
            Some(schemas),
            range,
            caller
        )
    )
}

/// Semantic Agent Schema を通した変更が間に入ったときに、のせ直して打ち直すコマンドの案内。
fn reapply_hint(
    path: &str,
    current: &str,
    schemas: &SchemaPlaces,
    range: Option<&[String]>,
    caller: &Caller,
) -> String {
    format!(
        "最新を読み直し、間に入った変更（interleaved）の上に変更をのせ直して打ち直す：{}",
        carried(
            &[
                "update",
                path,
                "--patch",
                "<JSON Patch>",
                "--if-match",
                current,
                "--reason",
                "<理由>"
            ],
            Some(schemas),
            range,
            caller
        )
    )
}

/// 取り込みを、作成として行うか更新として行うか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdoptAs {
    /// SAS の記録に無い既存のファイルを取り込む（UC1 の変形「既存のファイルを取り込む」）
    Create,
    /// SAS が書いたあとに外で直されたファイルを取り込む（UC2 の --adopt）
    Update,
}

/// SAS を通らない変更の取り込みの依頼。ファイルは書かない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptRequest {
    /// 取り込む経路
    pub path: String,
    /// 作成か更新か
    pub as_kind: AdoptAs,
    /// 従う Concrete の $id。渡されたら $schema と同じかを確かめる（無ければ $schema から決める）
    pub concrete: Option<String>,
    /// スキーマの置き場所（従う Concrete と $ref の先を $id で引く。返すコマンドにも引き継ぐ）
    pub schemas: SchemaPlaces,
    /// Concrete の更新を取り込むときに、従うインスタンスを確かめる範囲（--range の経路）
    pub range: Option<Vec<String>>,
    /// 理由（なぜ外で直したか）。更新の取り込みでは要る
    pub reason: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 更新の取り込みで、取り込む前の中身があるコミット（--base）。無ければ前の値の無い差分にする
    pub base: Option<String>,
}

/// --base のコミットにある、取り込む前の中身（コミット ・ 中身 ・ そのハッシュ）。中身は、この経路の記録のどれかの事後と
/// 同じでなければならない（記録の事前と事後をつなげる。ワークツリーで記録は1つなので、最後の事後でなくてもよい）。
///
/// # Errors
/// コミットにその経路が無い（not_found）・ JSON でない ・ 記録のどの事後とも違う（invalid_input）・ Git を読めない（versions）とき。
fn base_content(
    committed: &dyn CommitReader,
    base: &str,
    path: &str,
    recorded: &[String],
) -> Result<(String, Value, String), Refused> {
    let bytes = committed
        .read(base, path)
        .map_err(|e| refused("versions", path, e))?
        .ok_or_else(|| {
            refused(
                "not_found",
                path,
                format!("--base のコミット {base} に {path} が無い"),
            )
        })?;
    let hash = ContentHash::of(&bytes);
    if !recorded.iter().any(|r| r == hash.as_str()) {
        return Err(refused(
            "invalid_input",
            path,
            format!(
                "--base のコミット {base} の中身（{}）は、この経路の記録のどの事後とも違う。記録の事後と同じ中身のコミットを渡すか、--base を渡さずに前の値の無い差分で取り込む",
                hash.as_str()
            ),
        ));
    }
    let value = serde_json::from_slice::<Value>(&bytes).map_err(|e| {
        refused(
            "not_json",
            path,
            format!("--base のコミット {base} の中身が JSON として読めない：{e}"),
        )
    })?;
    Ok((base.to_string(), value, hash.as_str().to_string()))
}

/// SAS を通らない変更を取り込む（UC1 の変形 ・ UC2 の --adopt）。ファイルは書かず、いまの中身を
/// 形と集約の書く条件に照らしてから、source = outside の作成か更新として記録する。
/// 更新の取り込みの差分は、--base で渡されたコミット（中身が記録のどれかの事後と同じもの）との比較で、
/// --base が無ければ履歴を探さずに前の値の無い差分（before_unavailable）にし、事前は記録の最後の事後にする。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない ・ 作成の取り込みで記録がすでにある ・ 更新の取り込みで記録が無いか
/// 外の変更が無い ・ 形や書く条件に外れる（何も記録しない）。コミットや記録を読めない ・ 記録できないとき。
pub fn adopt(
    request: &AdoptRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    committed: &dyn CommitReader,
    caller: &Caller,
    log: &mut dyn ChangeLog,
) -> Result<ChangeResult, Refused> {
    let path = request.path.as_str();
    let written = match request.as_kind {
        AdoptAs::Create => Written::Created,
        AdoptAs::Update => Written::Updated,
    };
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replay(stored, written, "outside", path, store);
    }
    let text = store
        .read(path)
        .map_err(|e| refused("not_found", path, e))?;
    let current: Value = serde_json::from_str(&text)
        .map_err(|e| refused("not_json", path, format!("JSON として読めない：{e}")))?;
    let declared = current
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if let Some(c) = &request.concrete
        && *c != declared
    {
        return Err(refused(
            "invalid_input",
            path,
            format!("ファイルの $schema（{declared}）が --concrete（{c}）と違う"),
        ));
    }
    let target = target_of_declared(store, validator, request.schemas.files(), path, &declared)?;
    let kind = target.kind;
    let range = match request.as_kind {
        AdoptAs::Update => range_for(kind, path, request.range.as_deref(), "取り込んで直す")?,
        AdoptAs::Create if request.range.is_some() => {
            return Err(refused(
                "usage",
                path,
                "--range は Concrete を直す ・ 消すときだけ渡す".to_string(),
            ));
        }
        AdoptAs::Create => &[],
    };
    let hash = ContentHash::of(text.as_bytes());
    let writes = log.writes(path).map_err(log_refused)?;
    let recorded: Vec<String> = writes.iter().filter_map(|w| w.row.after.clone()).collect();

    // 前の状態：作成の取り込みは記録が無いこと、更新の取り込みはいまの中身が記録のどの事後とも違うこと
    let last = match (request.as_kind, writes.last()) {
        (AdoptAs::Create, Some(w)) => {
            return Err(refused(
                "recorded",
                path,
                format!(
                    "{path} は操作 {} で記録がある。外で直した変更は {} で取り込む",
                    w.operation,
                    carried(
                        &["update", path, "--adopt", "--reason", "<理由>"],
                        Some(&request.schemas),
                        None,
                        caller
                    )
                ),
            ));
        }
        (AdoptAs::Create, None) => None,
        (AdoptAs::Update, None) => {
            return Err(refused(
                "not_recorded",
                path,
                format!(
                    "{path} は Semantic Agent Schema の記録に無い。既存のファイルは {} で取り込む",
                    carried(
                        &["create", path, "--adopt"],
                        Some(&request.schemas),
                        None,
                        caller
                    )
                ),
            ));
        }
        (AdoptAs::Update, Some(_)) => {
            // 外の変更かは検証と同じ決まりで見る（このワークツリーの最後の事後、まだ書いていなければ記録のどれかの中身）
            let Some(w) = unrecorded_since(&writes, hash.as_str()) else {
                return Err(refused(
                    "nothing_to_adopt",
                    path,
                    "いまの中身は、このワークツリーで Semantic Agent Schema が最後に書いた事後（まだ書いていなければ記録のどれかの中身）と同じで、外の変更が無い".to_string(),
                ));
            };
            if request.reason.is_none() {
                return Err(refused(
                    "invalid_input",
                    path,
                    "更新の取り込みには、外で直した理由（--reason）が要る".to_string(),
                ));
            }
            Some(w.row.after.clone().unwrap_or_default())
        }
    };
    let row = ChangeRow {
        kind: written.kind().to_string(),
        source: "outside".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: None,
        after: None,
        reason: request.reason.clone(),
        schema: Some(declared.clone()),
        fields: Vec::new(),
    };
    let found = match (&last, &request.base) {
        (Some(_), Some(base)) => Some(base_content(committed, base, path, &recorded)?),
        (None, Some(_)) => {
            return Err(refused(
                "usage",
                path,
                "--base は、更新を取り込むとき（update --adopt）だけ渡す".to_string(),
            ));
        }
        (_, None) => None,
    };
    // 事前：--base のコミットの中身のハッシュ（記録のどれかの事後）、無ければ記録の最後の事後
    let expected: Option<String> = match (&found, &last) {
        (Some((_, _, hash)), _) => Some(hash.clone()),
        (None, l) => l.clone(),
    };
    let broken = form_violations(validator, kind, &declared, &current, target.more())
        .map_err(|e| refused("validator", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!(
                "いまの中身が型に従わないので取り込まない：{}",
                violations_message(&broken)
            ),
        ));
    }
    // 書く条件：--base のコミットがあればその中身と比べる。無ければ、前の値を知らない作成と同じに見る
    // （ツールからの取り込みで人の欄が default 以外なら、外で人の欄が変わったかを確かめられないので断る）
    let before_value = found.as_ref().map(|(_, v, _)| v);
    if let Err((code, message)) = check_write(
        &Judge {
            validator,
            more: target.more(),
        },
        kind,
        before_value,
        &current,
        target.concrete.as_ref().map(Resolved::concrete),
        caller.entry(),
    ) {
        let unknown = expected.is_some() && found.is_none();
        let mut r = refused(
            code,
            path,
            if unknown && code == "human_field" {
                format!("{message}。--base が無く、外で人の欄が変わったかを確かめられない")
            } else {
                message
            },
        );
        if code == "human_field" {
            let mut head = vec![written.kind(), path, "--adopt"];
            if expected.is_some() {
                head.extend(["--reason", "<理由>"]);
            }
            r.hint = Some(Box::new(format!(
                "人が --human を付けて取り込む：{}",
                carried(&head, Some(&request.schemas), None, &Caller::Human)
            )));
        }
        return Err(r);
    }
    let own_id = ConcreteSchema::read(&current).and_then(|c| c.id().map(ToString::to_string));
    let followers = match (&own_id, request.as_kind) {
        (Some(id), AdoptAs::Update) => Some(Followers::read(
            request.schemas.files(),
            store,
            validator,
            log,
            path,
            id,
            range,
            &current,
        )?),
        _ => None,
    };
    let diff = match (&expected, &found) {
        (None, _) => Diff::between(None, &current),
        (Some(_), Some((_, before, _))) => Diff::between(Some(before), &current),
        (Some(_), None) => Diff::without_before(&current),
    };
    let (session, operation) = record_change(
        caller,
        log,
        &ChangeRow {
            before: expected.clone(),
            after: Some(hash.as_str().to_string()),
            fields: field_rows(&diff),
            ..row
        },
    )?;
    let stale = stale_judgments(log, path, &current, &target, validator)?;
    let (violated, checked) = match followers {
        Some(f) => f.check(validator)?,
        None => (Vec::new(), Vec::new()),
    };
    Ok(ChangeResult {
        session,
        operation,
        written,
        kind,
        path: path.to_string(),
        schema: declared,
        before: expected.as_deref().and_then(ContentHash::parse),
        after: hash,
        diff,
        stale,
        violated,
        checked,
        replayed: false,
        outside: true,
        before_commit: found.map(|(rev, _, _)| rev),
    })
}
