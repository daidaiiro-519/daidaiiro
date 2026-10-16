//! ユースケース。port を通して入出力し、domain の規則で判定する。

pub mod check;
pub mod engine;
pub mod scaffold;

use crate::ports::{Files, Programs, Releases};

/// ユースケースが使う port の組。起動時の結合（adapters の compose）が実装を渡す。
pub struct Env<F: Files, P: Programs, R: Releases> {
    pub files: F,
    pub programs: P,
    pub releases: R,
}
