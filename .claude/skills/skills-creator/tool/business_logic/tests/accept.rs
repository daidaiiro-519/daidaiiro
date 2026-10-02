// SPDX-License-Identifier: MIT
//! 助言型の受け入れの検査2（複製 ・ 語彙 ・ 出典の頁）を事例で検証する。
//!
//!     cargo test -p sc_business_logic

use std::path::PathBuf;

use sc_business_logic::accept::{concept_units, copied};
use serde_json::{json, Value};

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
    let items = [item(
        "目的をかなえてもらうために、システムを呼び出す側の利害関係者。",
        "4.2（58頁）",
    )];
    let check = copied(&items, &notes());
    assert_eq!(check.no, 2);
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[test]
fn a_sentence_copied_from_the_notes_is_found() {
    let items = [item(
        "主アクターは、システムが動くことで満たされる目的を持つ利害関係者である。",
        "4.2（58頁）",
    )];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("複製している")),
        "ノートと長く一致する文は複製である: {:?}",
        check.findings
    );
}

#[test]
fn a_copy_is_found_even_with_different_spacing_and_markup() {
    let items = [item(
        "主アクター は、システム が 動くことで満たされる **目的** を持つ利害関係者",
        "4.2（58頁）",
    )];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("複製している")),
        "空白と記法を外して照らす: {:?}",
        check.findings
    );
}

#[test]
fn a_katakana_word_absent_from_the_notes_is_found() {
    let items = [item(
        "システムを呼び出す側の利害関係者で、ペルソナとも呼ぶ。",
        "4.2（58頁）",
    )];
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
    let items = [item(
        "目的をかなえてもらうために、システムを呼び出す側の利害関係者。",
        "4.2",
    )];
    let check = copied(&items, &notes());
    assert!(
        check.findings.iter().any(|f| f.contains("頁")),
        "出典には頁を書く: {:?}",
        check.findings
    );
}

#[test]
fn no_notes_is_a_finding() {
    let items = [item(
        "目的をかなえてもらうために、システムを呼び出す側の利害関係者。",
        "4.2（58頁）",
    )];
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

#[test]
fn a_number_of_the_original_is_found() {
    // 判断基準の頁に原典の表は無い ── 読み手は「表20.1」を辿れない。番号は出典の欄が持つ
    for text in [
        "合否基準（表20.1）の各項目で確認する。",
        "指針14のとおり、下に置く。",
        "本書の考え方に合わせる。",
    ] {
        let items = [item(text, "4.2（58頁）")];
        let check = copied(&items, &notes());
        assert!(
            check
                .findings
                .iter()
                .any(|f| f.contains("原典の番号") || f.contains("本書")),
            "{text}: {:?}",
            check.findings
        );
    }
}

#[test]
fn a_number_in_the_source_is_not_found() {
    let items = [item(
        "目的をかなえてもらうために、システムを呼び出す側の利害関係者。",
        "メモ11 表20.1（241頁）",
    )];
    let check = copied(&items, &notes());
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[test]
fn writing_one_scenario_is_not_the_book() {
    // 「1本書き」の「本書」は原典を指す語ではない
    let items = [item(
        "成功の流れを1本書き、数本書くだけにする。",
        "4.2（58頁）",
    )];
    let check = copied(&items, &notes());
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

/// 原典が付けた長い名前を持つノート。**名前は25字を超える。**
fn named_notes() -> Vec<(PathBuf, String)> {
    vec![(
        PathBuf::from("notes/05.md"),
        "## テストモニタリング、テストコントロールとテスト完了（5.3）（54頁）\n\n1. テストは欠陥があることは示せるが、欠陥がないことは示せない（18頁）\n".to_owned(),
    )]
}

#[test]
fn a_long_name_in_a_title_is_not_a_copy() {
    // **題は原典の名前をそのまま書く欄である** ── ノートと一致するのが正しい
    // （実測 2026-10-02、qa-advisor の「テストモニタリング、テストコントロールとテスト完了」）
    let mut it = item("自分の言葉でまとめた意味。", "5.3（54頁）");
    it["title"] = json!("テストモニタリング、テストコントロールとテスト完了");
    it["related"] = json!([{"criterion": "x", "title": "テストモニタリング、テストコントロールとテスト完了", "relation": "関係"}]);
    it["elements"]["units"][0]["term"] =
        json!("テストモニタリング、テストコントロールとテスト完了");
    let check = copied(&[it], &named_notes());
    assert!(
        !check.findings.iter().any(|f| f.contains("複製している")),
        "{:?}",
        check.findings
    );
}

#[test]
fn a_name_quoted_in_brackets_is_not_a_copy() {
    // **「」で囲んだ部分は引用である** ── 原典の名前を示すときに使う。地の文の複製は従来どおり検出する
    let quoted = item(
        "原則の1つ目は「テストは欠陥があることは示せるが、欠陥がないことは示せない」である。",
        "1.3（18頁）",
    );
    let check = copied(&[quoted], &named_notes());
    assert!(
        !check.findings.iter().any(|f| f.contains("複製している")),
        "{:?}",
        check.findings
    );
    let bare = item(
        "テストは欠陥があることは示せるが、欠陥がないことは示せないとする原則。",
        "1.3（18頁）",
    );
    let check = copied(&[bare], &named_notes());
    assert!(
        check.findings.iter().any(|f| f.contains("複製している")),
        "{:?}",
        check.findings
    );
}
