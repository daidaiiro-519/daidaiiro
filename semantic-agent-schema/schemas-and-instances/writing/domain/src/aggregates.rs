//! 集約。

mod concrete;
mod instance;

pub use concrete::{ABSTRACT_VERSIONS, ConcreteSchema, DIALECT, escape};
pub use instance::{Entry, Instance};
