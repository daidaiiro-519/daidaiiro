//! Skill の項目ごとに、作る人の答え（目的と役割）から何を導いて書くか。作る手順（new の値を組む決まり）と、確認の推測の一覧
//! （confirm の何から推測したか）の両方がこの1つの表を使う（同じことを2か所に書かない）。
//!
//! 作る人が答えるのは目的（purpose）と役割（role）だけで、ここに無い項目は答えから入れる。

/// 導いて書く項目（skill.json の根の欄の名前、受け持たないことは role/does_not）と、何から導くか。
pub const DERIVED: [(&str, &str); 14] = [
    ("role/does_not", "role の受け持つことの隣の仕事として"),
    ("name", "purpose と role から AI が付ける（聞かない）"),
    ("description", "role の受け持つことと purpose の場面から"),
    (
        "compatibility",
        "role の受け持つことに要る環境から（無ければ書かない）",
    ),
    ("preconditions", "role の受け持つことから"),
    ("inputs", "role の受け持つことから"),
    ("outputs", "purpose の得るもの（問2）から"),
    (
        "steps",
        "role の受け持つことを、purpose の得るものまで並べて",
    ),
    ("edge_cases", "steps と examples から"),
    ("rules", "role の受け持つことと examples から"),
    ("minimal_guarantees", "outputs の置き場所から"),
    ("completion_criteria", "purpose の得るもの（問2）から"),
    (
        "examples",
        "問1 の場面と、別の場面で「同じ」と答えた場面から",
    ),
    (
        "references",
        "資料の問いの答え（名前と置き場所）から。中身 ・ 形 ・ 読むステップは steps から導く",
    ),
];

/// 作る人の答えから取った件の from（この from を持つ件は、推測ではなく答え）。
pub const ANSWERED_FROM: [&str; 4] = ["問1 の答え", "問2 の答え", "問3 の答え", "資料の問いの答え"];

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
