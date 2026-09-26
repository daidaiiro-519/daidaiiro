//! 層の外側である。内側の層だけを呼ぶ。
//! 依存の向きは、外から内の1方向だけである。

use core_layer::{Layer, LayerError};

/// 内側の層の名前を返す。
#[must_use]
pub fn name() -> &'static str {
    core_layer::name()
}

/// 内側の層を組んで、その名前を返す。
///
/// # Errors
///
/// 内側の層が名前を受け取れないときは、その理由をそのまま返す。
pub fn label_of(input: &str) -> Result<String, LayerError> {
    Ok(Layer::new(input)?.label().to_owned())
}
