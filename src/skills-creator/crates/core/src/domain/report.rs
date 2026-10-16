//! check の項目と結果。どの項目も既定は「判定不能」で、確かめた分岐からだけ合格を置く。
//! 不合格か判定不能が1件でもあれば、終了コードは 1 になる。

use serde_json::{json, Value};

/// 項目の結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
    Unknown,
    Skip,
}

impl Outcome {
    pub fn name(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Fail => "fail",
            Outcome::Unknown => "unknown",
            Outcome::Skip => "skip",
        }
    }
}

/// check の項目。id と、何を確かめるか。
pub const ITEMS: [(&str, &str); 9] = [
    ("skill.frontmatter", "SKILL.md の先頭の欄が在り、YAML として読める"),
    ("skill.type", "SKILL.md が型 skills-creator/skill@v1 に合う（名前 ・ 説明 ・ 欄 ・ 行数 ・ 6つの節 ・ パス ・ リンク）"),
    ("skill.placeholders", "記入欄 {{…}} が残っていない"),
    ("schemas.present", "concrete スキーマが assets/ に1件以上ある"),
    ("schemas.check", "基盤の check-schemas が所見0件で通る"),
    ("engine.runs", "scripts/bin/schema-driven が起動して版を答える"),
    ("engine.abstract", "concrete スキーマの x-abstract の版を、エンジンが扱える"),
    ("entry.delegate", "入口 scripts/<Skill名> が、基盤の操作をエンジンに渡して同じ結果を返す"),
    ("legacy.absent", "古い形のファイル（tool.json ・ tools.json）が無い"),
];

/// 項目ごとの結果の並び。
#[derive(Debug, PartialEq)]
pub struct Report {
    results: Vec<(&'static str, Outcome, String)>,
}

impl Default for Report {
    fn default() -> Self {
        Report {
            results: ITEMS
                .iter()
                .map(|(id, _)| (*id, Outcome::Unknown, "まだ確かめていない".to_owned()))
                .collect(),
        }
    }
}

impl Report {
    /// 項目の結果を置く。知らない id は置かない（項目の一覧は ITEMS が決める）。
    pub fn set(&mut self, id: &str, outcome: Outcome, detail: impl Into<String>) {
        if let Some(slot) = self.results.iter_mut().find(|(known, _, _)| *known == id) {
            slot.1 = outcome;
            slot.2 = detail.into();
        }
    }

    pub fn outcome(&self, id: &str) -> Option<Outcome> {
        self.results
            .iter()
            .find(|(known, _, _)| *known == id)
            .map(|(_, outcome, _)| *outcome)
    }

    /// 不合格か判定不能が1件でもあれば 1、それ以外は 0。
    pub fn code(&self) -> i32 {
        i32::from(
            self.results
                .iter()
                .any(|(_, outcome, _)| matches!(outcome, Outcome::Fail | Outcome::Unknown)),
        )
    }

    pub fn to_json(&self) -> Value {
        let items: Vec<Value> = self
            .results
            .iter()
            .map(|(id, outcome, detail)| json!({"id": id, "result": outcome.name(), "detail": detail}))
            .collect();
        json!({"ok": self.code() == 0, "items": items})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_report_is_unknown_everywhere_and_fails() {
        let report = Report::default();
        let unknown: Vec<Option<Outcome>> =
            ITEMS.iter().map(|(id, _)| report.outcome(id)).collect();
        assert_eq!(unknown, vec![Some(Outcome::Unknown); ITEMS.len()]);
        assert_eq!(report.code(), 1);
    }

    #[test]
    fn only_pass_and_skip_make_the_code_zero() {
        let mut report = Report::default();
        ITEMS
            .iter()
            .for_each(|(id, _)| report.set(id, Outcome::Pass, ""));
        report.set("entry.delegate", Outcome::Skip, "入口が無い");
        assert_eq!(report.code(), 0);
        report.set("legacy.absent", Outcome::Fail, "tool.json が在る");
        assert_eq!(report.code(), 1);
    }

    #[test]
    fn an_unknown_id_is_not_added() {
        let mut report = Report::default();
        report.set("no.such", Outcome::Pass, "");
        assert_eq!(report.outcome("no.such"), None);
    }
}
