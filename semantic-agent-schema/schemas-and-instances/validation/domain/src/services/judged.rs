//! 対象に当たる rule（形の検査を除く）と、その判定が読んだ所のハッシュ。
//!
//! 検証は判定を記録するときと、前の判定と比べて判定し直す所を決めるときに、
//! 同じ基準（当たる所の値 ・ rule の related_fields が指す所の値 ・ rule の文）でハッシュを作る。

use crate::services::{builtin_checks, rules_for_concrete, rules_for_instance, unjudged};
use crate::values::RuleResult;
use serde_json::Value;
use writing_domain::aggregates::ConcreteSchema;
use writing_domain::values::ContentHash;

/// 判定と、その判定が読んだ所（JSON Pointer。"" は文書全体）と、判定に使った rule の定義（x-prompt の rules の1件）。
/// 読んだ所は当たる rule の所（と rule の related_fields）で、形の検査と組み込みの検査は文書全体を読む。
pub type Judged = (RuleResult, String, Option<Value>);

/// 文書全体を読む判定。
#[must_use]
pub fn whole(r: RuleResult) -> Judged {
    (r, String::new(), None)
}

/// Concrete に当たる、組み込みの検査の結果と Abstract の rule（判定を渡されていなければ not-judged）。
/// abstract_doc は申告した Abstract の本体、vocabulary は Abstract の語彙。
#[must_use]
pub fn concrete_rules(
    abstract_doc: &Value,
    vocabulary: Option<&Value>,
    concrete: &ConcreteSchema,
) -> Vec<Judged> {
    // JSON Schema で書けない Concrete の決まり3つは、Semantic Agent Schema が組み込みで確かめる
    let mut results: Vec<Judged> = builtin_checks(concrete).into_iter().map(whole).collect();
    for rule in rules_for_concrete(abstract_doc, vocabulary, concrete) {
        results.push((
            unjudged(&rule),
            rule.pointer.clone(),
            Some(rule.definition.clone()),
        ));
    }
    results
}

/// インスタンスに当たる Concrete の rule（判定を渡されていなければ not-judged）。more は渡された置き場所から引いた文書（Concrete と $ref の先。$id と文書）。
#[must_use]
pub fn instance_rules(
    concrete_doc: &Value,
    instance: &Value,
    more: &[(String, Value)],
) -> Vec<Judged> {
    rules_for_instance(concrete_doc, instance, more)
        .into_iter()
        .map(|rule| {
            (
                unjudged(&rule),
                rule.pointer.clone(),
                Some(rule.definition.clone()),
            )
        })
        .collect()
}

/// 判定が読んだ所のハッシュ。
#[must_use]
fn reads_of(document: &Value, at: &str) -> ContentHash {
    ContentHash::of_read(at, document.pointer(at))
}

/// rule の定義の related_fields（判定のときに一緒に読む所。このインスタンスの中の JSON Pointer）。
#[must_use]
pub fn related_fields(definition: Option<&Value>) -> Vec<String> {
    definition
        .and_then(|d| d.get("related_fields"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// 判定1件が読んだもののハッシュ：当たる所の値に、rule の related_fields が指す所の値と rule の文を足す
/// （読む欄が変わる ・ Concrete の rule を書き換えると、前の判定は効かない）。
#[must_use]
pub fn reads_hash(document: &Value, at: &str, definition: Option<&Value>) -> ContentHash {
    let mut reads = reads_of(document, at);
    for p in related_fields(definition) {
        let more = reads_of(document, &p);
        reads = ContentHash::of(format!("{}\u{0}{}", reads.as_str(), more.as_str()).as_bytes());
    }
    match definition {
        Some(def) => ContentHash::of(format!("{}\u{0}{def}", reads.as_str()).as_bytes()),
        None => reads,
    }
}
