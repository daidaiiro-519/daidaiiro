//! 集約。

mod concrete;
mod instance;

pub use concrete::{ABSTRACT_VERSIONS, ConcreteSchema, escape, extension_names};
pub use instance::{Entry, Instance};
