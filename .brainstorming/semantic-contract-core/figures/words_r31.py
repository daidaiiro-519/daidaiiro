# 第31回：Q5 で決めた名前（英語と日本語の対）に、Q0 ・ Q1〜Q4 ・ 図 ・ 案のファイルを揃える。Q5 と、原典の引用 ・ 構想 v0.3 の例のファイルは変えない。
PH1, PH2 = "\x01IP\x01", "\x01SI\x01"
PAIRS = [
 ("Interpreter Protocol（能力表明", "Validator Protocol（構想 v0.3 の " + PH1 + "。能力表明"),
 ("Semantic Interpreter：無い", "Semantic Validator（構想 v0.3 の " + PH2 + "）：無い"),
 ("Semantic Interpreter", "Semantic Validator"),
 ("Interpreter Protocol", "Validator Protocol"),
 ("Interpreter に必要な Capability", "validator に必要な capability"),
 ("Interpreter", "validator"),
 (PH1, "Interpreter Protocol"), (PH2, "Semantic Interpreter"),
 ("必要な Capability", "必要な capability"), ("Capability", "capability"),
 ("pub enum Outcome", "pub enum ValidationResult"), ("Outcome::", "ValidationResult::"), ("(Required, Outcome)", "(Required, ValidationResult)"),
 ("matches!(outcome,", "matches!(result,"), ("|(required, outcome)|", "|(required, result)|"),
 ("proposals/outcome.rs", "proposals/validation_result.rs"), ('"outcome":', '"result":'),
 ("5つの outcome", "検証結果の5つの値"), ("outcome へ投影できないもの", "検証結果の値へ対応させられないもの"),
 ("outcome（enum）", "検証結果（enum）"), ("outcome", "検証結果"),
 ("inbound と outbound のアダプター", "inbound adapter と outbound adapter"),
 ("inbound アダプター", "inbound adapter"), ("outbound アダプター", "outbound adapter"),
]
def fix(s):
    for a, b in PAIRS:
        s = s.replace(a, b)
    return s
