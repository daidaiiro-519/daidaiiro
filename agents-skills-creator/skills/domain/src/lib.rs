//! 文脈 skills のドメインモデル。
//!
//! サービス2つ：Skill の項目（skill.json）から SKILL.md を決まった形で作る（skill_md）と、項目ごとに答えから何を導くか
//! （derivation）。SKILL.md の形は書く（write-skill-md）と確かめる（validate）が、何を導くかは承認の材料（validate）が使う。

pub mod services;
