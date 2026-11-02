//! Skill の項目ごとに、作る人の答え（目的と役割）から何を導いて書くか。作る手順（new の値を組む決まり）と、確認の推測の一覧
//! （confirm の何から推測したか）の両方がこの1つの表を使う（同じことを2か所に書かない）。
//!
//! 作る人が答えるのは目的（purpose）と役割（role）だけで、ここに無い項目は答えから入れる。
//!
//! 項目ごとに、同じ形（目的の種類 ・ 役割 ・ 手順 ・ 決まりが同じで、分野と原典 ・ 知見だけが違う）の Skill の間で
//! 同じになるか（Basis）も持つ。同じ形で作る手順（new --like）と、元との違いの見分け（confirm）がこの表を使う。

/// 項目が、何によって決まるか。同じ形の Skill の間で同じになるのは Shape だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// 目的の種類 ・ 役割 ・ 手順 ・ 決まりから決まる（同じ形なら元の値のまま）
    Shape,
    /// 分野と場面から決まる（同じ形でも、作る人の答えで変わる）
    Domain,
    /// 原典 ・ 知見（資料）から決まる（同じ形でも、資料の問いの答えで変わる）
    Material,
}

/// 作る人の答えから入れる項目（purpose は欄ごと）と、何によって決まるか。
pub const ANSWERS: [(&str, Basis); 4] = [
    ("purpose/who", Basis::Shape),
    ("purpose/when", Basis::Domain),
    ("purpose/gets", Basis::Shape),
    ("role", Basis::Shape),
];

/// 導いて書く項目（skill.json の根の欄の名前、受け持たないことは role/does_not）と、何から導くか。
pub const DERIVED: [(&str, &str, Basis); 14] = [
    (
        "role/does_not",
        "role の受け持つことの隣の仕事として",
        Basis::Shape,
    ),
    (
        "name",
        "purpose と role から AI が付ける（聞かない）",
        Basis::Domain,
    ),
    (
        "description",
        "role の受け持つことと purpose の場面から",
        Basis::Domain,
    ),
    (
        "compatibility",
        "role の受け持つことに要る環境から（無ければ書かない）",
        Basis::Shape,
    ),
    ("preconditions", "role の受け持つことから", Basis::Shape),
    ("inputs", "role の受け持つことから", Basis::Shape),
    ("outputs", "purpose の得るもの（問2）から", Basis::Shape),
    (
        "steps",
        "role の受け持つことを、purpose の得るものまで並べて",
        Basis::Shape,
    ),
    ("edge_cases", "steps と examples から", Basis::Shape),
    ("rules", "role の受け持つことと examples から", Basis::Shape),
    ("minimal_guarantees", "outputs の置き場所から", Basis::Shape),
    (
        "completion_criteria",
        "purpose の得るもの（問2）から",
        Basis::Shape,
    ),
    (
        "examples",
        "問1 の場面と、別の場面で「同じ」と答えた場面から",
        Basis::Domain,
    ),
    (
        "references",
        "資料の問いの答え（名前と置き場所）から。中身 ・ 形 ・ 読むステップは steps から導く",
        Basis::Material,
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
        .find(|(f, _, _)| *f == field)
        .map(|(_, from, _)| *from)
}

/// 項目（purpose は purpose/who のように欄ごと）が何によって決まるか。表に無い項目は None。
#[must_use]
pub fn basis(field: &str) -> Option<Basis> {
    ANSWERS
        .iter()
        .find(|(f, _)| *f == field)
        .map(|(_, b)| *b)
        .or_else(|| {
            DERIVED
                .iter()
                .find(|(f, _, _)| *f == field)
                .map(|(_, _, b)| *b)
        })
}

/// skill.json の中の所（JSON Pointer）が当たる項目の Basis。/purpose/when/… は purpose/when、/steps/2 は steps で引く。
#[must_use]
pub fn basis_at(pointer: &str) -> Option<Basis> {
    let parts: Vec<&str> = pointer.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["purpose", sub, ..] => basis(&format!("purpose/{sub}")),
        ["role", "does_not", ..] => basis("role/does_not"),
        [top, ..] => basis(top),
        [] => None,
    }
}
