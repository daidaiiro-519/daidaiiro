//! 試験の部品。ワークスペース全体の検査（依存の向き ・ スキーマ ・ 場面と保証の突き合わせ）を tests/ に置く。
//!
//! ポートの代役は、使う application の tests/ に置く（ここに置くと dev-dependencies が循環する。探索の記録 step-01）。
//! この crate は dev-dependencies からだけ使い、製品の crate の [dependencies] に入れない。
