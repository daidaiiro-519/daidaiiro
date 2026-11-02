//! 文脈 skills のドメインモデル。
//!
//! サービス2つ：Skill の項目（skill.json）から SKILL.md を決まった形で作る（skill_md）と、項目ごとに答えから何を導くか
//! （derivation）。SKILL.md の形は作る（build）と確かめる（check）が、何を導くかは作る手順（new）と確認（confirm）が、
//! それぞれ同じ理由で使うので、ここに1つ置く。

pub mod services;
