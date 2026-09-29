// SPDX-License-Identifier: MIT
//! 部品の登録台帳 ── **新しい描画の型を、核を直さずに足すための場所。**
//!
//! 部品は「種別名」で参照する関数として登録する。核（合成 ・ 配置）はこの台帳越しにしか部品を
//! 呼ばない ── 新しい部品は台帳へ1行足すだけで使えるようになり、核の側は一切変更しない。

use serde_json::{Map, Value};

use crate::theme::Style;

/// 描いた部品の中身と、それが申告すること。
///
/// 描く時点が「置く前」か「置いた後」かで、申告する大きさの意味が違う ── だから印ではなく
/// 2つの種類に分ける。3つ目を作れないので、取り違えが構造として起きない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// 自分の原点 `(0,0)` を基準に描いた断片。呼ぶ側が置き場所を決める。
    ///
    /// **申告した大きさの中に、インクが収まっていなければならない。** 外へ出ると、配置は空で
    /// ない場所を空きと見なし、置いたものどうしが重なる。
    Own,
    /// 既に置かれたものをまたいで、絶対座標で描いた断片。
    Absolute,
}

/// 描いた部品。
#[derive(Debug, Clone)]
pub struct Fragment {
    /// SVG 断片。
    pub svg: String,
    /// 外側から見える幅。
    pub width: f64,
    /// 外側から見える高さ。
    pub height: f64,
    /// 渡された名前を、この部品が自分で描いたか。**名前を描く役目は1か所にしか置けない。**
    pub labels_itself: bool,
    /// どの座標系で描いたか。
    pub origin: Origin,
    /// 幅と高さが、入力の整数のまま来たか。**目録が、見本の大きさを移す前と同じ書き方で出す。**
    pub int_size: (bool, bool),
}

impl Fragment {
    /// 自分の原点で描いた断片。
    #[must_use]
    pub fn own(svg: String, width: f64, height: f64) -> Self {
        Self {
            svg,
            width,
            height,
            labels_itself: false,
            origin: Origin::Own,
            int_size: (false, false),
        }
    }

    /// 絶対座標で描いた断片。
    #[must_use]
    pub fn absolute(svg: String, width: f64, height: f64) -> Self {
        Self {
            svg,
            width,
            height,
            labels_itself: false,
            origin: Origin::Absolute,
            int_size: (false, false),
        }
    }

    /// 幅と高さが、入力の整数のまま来たことを申告する。
    #[must_use]
    pub const fn ints(mut self, width: bool, height: bool) -> Self {
        self.int_size = (width, height);
        self
    }

    /// 名前を自分で描いたことを申告する。
    #[must_use]
    pub const fn labelled(mut self) -> Self {
        self.labels_itself = true;
        self
    }
}

/// 部品の入力。**構造だけを持つ ── 色 ・ 寸法を含まない。**
pub type Props = Map<String, Value>;

/// 部品。
pub type Component = fn(&Props, &Style) -> Result<Fragment, String>;

/// 台帳。**名前の順に並べて返す。**
#[must_use]
pub fn table() -> Vec<(&'static str, Component)> {
    // **台帳は、部品を知らない。** 部品の一覧は crate の根が持つ ── 土台が部品を呼ぶと、
    // 層の向きが逆になる
    let mut all = crate::components();
    all.sort_by_key(|(k, _)| *k);
    all
}

/// 台帳に在る名前。
#[must_use]
pub fn known_kinds() -> Vec<&'static str> {
    table().into_iter().map(|(k, _)| k).collect()
}

/// 種別名から部品を引いて描く。
///
/// # Errors
///
/// 台帳に無い名前のときと、部品が描けないときに返す。
pub fn render(kind: &str, props: &Props, style: &Style) -> Result<Fragment, String> {
    let all = table();
    let Some((_, f)) = all.iter().find(|(k, _)| *k == kind) else {
        let names: Vec<String> = all.iter().map(|(k, _)| crate::py::quote(k)).collect();
        return Err(format!(
            "知らない部品です: {kind}（台帳: [{}]）",
            names.join(", ")
        ));
    };
    f(props, style)
}

/// 節点として置く部品を描く。**自分の原点で描くものでなければならない。**
///
/// # Errors
///
/// 台帳に無い名前のときと、その部品が絶対座標で描くものだったときに返す。
pub fn render_node(kind: &str, props: &Props, style: &Style) -> Result<Fragment, String> {
    let r = render(kind, props, style)?;
    if r.origin != Origin::Own {
        return Err(format!(
            "部品 '{kind}' は絶対座標で描くので、節点としては置けない"
        ));
    }
    Ok(r)
}
