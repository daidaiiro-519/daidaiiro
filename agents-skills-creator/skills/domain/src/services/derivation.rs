//! Skill の項目ごとに、作る人の答え（目的と役割）から何を導いて書くか。承認の材料の推測の一覧（validate の、何から推測したか）が
//! この1つの表を使う。
//!
//! 目的（purpose）と役割（role）は、AI が仮説として示し、作る人が承認して決める。ここに無い項目はその2つから入れる。
//!
//! 件ごとの from は「作る人の答え」と、それ以外（推測）の2つで足りる。作る流れは決まった問いを持たず、完成の像を
//! 変える所だけをその場で聞くので、どの問いの答えかは件の値を決める根拠にならない（どの所でも聞かれうる）。

/// 導いて書く項目（skill.json の根の欄の名前、受け持たないことは role/does_not）と、何から導くか。
pub const DERIVED: [(&str, &str); 14] = [
    ("role/does_not", "role の受け持つことの隣の仕事として"),
    (
        "name",
        "最初の返事の前に、作業の動詞と対象から AI が付け、構造の像で見せる（作る人が変えたら --semantic を打ち直す）",
    ),
    ("description", "role の受け持つことと purpose の場面から"),
    (
        "compatibility",
        "role の受け持つことに要る環境から（無ければ書かない）",
    ),
    ("preconditions", "role の受け持つことから"),
    ("inputs", "role の受け持つことから"),
    ("outputs", "目的の得るものから"),
    (
        "steps",
        "role の受け持つことを、purpose の得るものまで並べて",
    ),
    ("edge_cases", "steps と examples から"),
    ("rules", "role の受け持つことと examples から"),
    ("minimal_guarantees", "outputs の置き場所から"),
    ("completion_criteria", "目的の得るものから"),
    (
        "examples",
        "完成の像（頼む文と返る答え）と、作る人が挙げた別の場面から",
    ),
    (
        "references",
        "作る人が示した資料（名前と置き場所）から。中身 ・ 形 ・ 読むステップは steps から導く",
    ),
];

/// 作る人の答えから取った件の from（この from を持つ件は、推測ではなく答え）。
pub const ANSWERED_FROM: [&str; 1] = ["作る人の答え"];

/// 件の from が、作る人の答えを指すか。
#[must_use]
pub fn is_answer(from: &str) -> bool {
    ANSWERED_FROM.iter().any(|a| from.starts_with(a))
}

/// 項目を何から導くか。表に無い項目（purpose ・ role）は None（作る人の答えから入れる）。
#[must_use]
pub fn derived_from(field: &str) -> Option<&'static str> {
    DERIVED
        .iter()
        .find(|(f, _)| *f == field)
        .map(|(_, from)| *from)
}
