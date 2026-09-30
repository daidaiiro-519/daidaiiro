// SPDX-License-Identifier: MIT
//! 助言型の受け入れの検査2（複製 ・ 語彙 ・ 出典の頁）を事例で検証する。
//!
//!     cargo test -p sc_business_logic

use std::path::PathBuf;

use sc_business_logic::accept::{concept_units, copied};
use serde_json::{Value, json};

/// 学習ノート1本。**原典に近い言い回しを持つ。**
fn notes() -> Vec<(PathBuf, String)> {
    vec![(
        PathBuf::from("notes/04.md"),
        "## 4.2 主アクター（58頁）\n\n- 主アクターは、システムが動くことで満たされる目的を持つ利害関係者である。（58頁）\n- **アクター-目的リスト**を作ると、優先度付けと分割ができる。（60頁）\n".to_owned(),
    )]
}

/// 判断基準1件。要素の中身と出典の節を差し替えて使う。
fn item(meaning: &str, section: &str) -> Value {
    json!({
        "id": "primary-actor",
        "title": "主アクター",
        "summary": "システムに目的の達成を求める利害関係者。",
        "elements": {"units": [{
            "kind": "definition",
            "term": "主アクター",
            "meaning": meaning,
            "source": {"origin": "原典", "section": section, "location": "書誌", "fetched": "2026-09-29", "method": "学習ノートからまとめた"}
        }]}
    })
}

#[test]
fn a_summary_in_own_words_passes() {
    let items = [item("目的をかなえてもらうために、システムを呼び出す側の利害関係者。", "4.2（58頁）")];
    let check = copied(&items, &notes());
    assert_eq!(check.no, 2);
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[test]
fn a_sentence_copied_from_the_notes_is_found() {
    let items = [item("主アクターは、システムが動くことで満たされる目的を持つ利害関係者である。", "4.2（58頁）")];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("複製している")),
        "ノートと長く一致する文は複製である: {:?}",
        check.findings
    );
}

#[test]
fn a_copy_is_found_even_with_different_spacing_and_markup() {
    let items = [item("主アクター は、システム が 動くことで満たされる **目的** を持つ利害関係者", "4.2（58頁）")];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("複製している")),
        "空白と記法を外して照らす: {:?}",
        check.findings
    );
}

#[test]
fn a_katakana_word_absent_from_the_notes_is_found() {
    let items = [item("システムを呼び出す側の利害関係者で、ペルソナとも呼ぶ。", "4.2（58頁）")];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("ペルソナ")),
        "原典に無い語を使っている: {:?}",
        check.findings
    );
}

#[test]
fn a_vocabulary_word_of_the_notes_passes() {
    let items = [item("一覧からアクター-目的リストを作る。", "4.2（60頁）")];
    let check = copied(&items, &notes());
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[test]
fn a_source_without_a_page_is_found() {
    let items = [item("目的をかなえてもらうために、システムを呼び出す側の利害関係者。", "4.2")];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("頁")),
        "出典には頁を書く: {:?}",
        check.findings
    );
}

#[test]
fn no_notes_is_a_finding() {
    let items = [item("目的をかなえてもらうために、システムを呼び出す側の利害関係者。", "4.2（58頁）")];
    let check = copied(&items, &[]);
    assert!(!check.findings.is_empty(), "学習ノートが無ければ照らせない");
}

/// 題だけを持つ判断基準1件。
fn titled(title: &str) -> Value {
    json!({"id": "c", "title": title})
}

#[test]
fn a_title_in_a_heading_is_a_concept() {
    let check = concept_units(&[titled("主アクター")], &notes());
    assert_eq!(check.no, 4);
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[test]
fn a_title_named_by_a_figure_is_a_concept() {
    let notes = vec![(
        PathBuf::from("notes/01.md"),
        "## 1.3 要求とユースケース（14頁）\n\n- 図1.1 要求のハブ-スポークモデル：中心にユースケースを置く。（17頁）\n".to_owned(),
    )];
    let check = concept_units(&[titled("要求のハブ-スポークモデル")], &notes);
    assert!(
        check.findings.is_empty(),
        "原典が図の題で名付けた概念も単位にできる: {:?}",
        check.findings
    );
}

#[test]
fn a_title_named_nowhere_is_found() {
    let check = concept_units(&[titled("ユースケース駆動の心得")], &notes());
    assert!(
        !check.findings.is_empty(),
        "原典が名付けていない題は概念の単位ではない"
    );
}
