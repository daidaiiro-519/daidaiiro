//! ブレストボード。転写した schema-driven の上に、ボードの Design と、
//! ボードに固有のツール（init ・ inspect ・ migrate ・ serve）を置く。

pub mod design;
pub mod inspect;
pub mod migrate;
mod net;
pub mod serve;
pub mod tools;
