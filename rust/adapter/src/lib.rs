//! 内側の層を呼ぶ、外側の層である。
//! 依存の向きは、外から内の1方向だけである。

/// 内側の層の名前を返す。
pub fn name() -> &'static str {
    core_layer::name()
}
