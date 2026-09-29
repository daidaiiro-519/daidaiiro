// SPDX-License-Identifier: MIT
//! 自由配置の合成 ── **配置の解決を経由しない、もう一つの組み立て方。**
//!
//! 関係を自動で置く道 ・ 値から単独で描く道が「意味を持つ図」を組むためのものなら、こちらは
//! 「1枚の絵」を組むための道 ── 背景 ・ 見出し ・ 複数の部品を、好きな位置へ重ねて置ける
//! だけの、薄い合成層である。
//!
//! 層は木にできる ── 子を持つ層は部品ではなく「群」として扱われ、自分の変形を子ぶんまとめて
//! かける。層に切り抜きを持たせると、その中身を指定した形で切り抜く。

use serde_json::{Map, Value};

use crate::props;
use crate::py;
use crate::registry;
use crate::style;

fn map_of(v: Option<&Value>) -> Option<&Map<String, Value>> {
    v.and_then(Value::as_object)
}

/// 層の変形。
fn transform_of(layer: &Map<String, Value>) -> String {
    let mut parts = Vec::new();
    let x = layer.get("x");
    let y = layer.get("y");
    if x.is_some_and(props::truthy) || y.is_some_and(props::truthy) {
        let xv = x.and_then(Value::as_f64).unwrap_or(0.0);
        let yv = y.and_then(Value::as_f64).unwrap_or(0.0);
        parts.push(format!("translate({xv:.1},{yv:.1})"));
    }
    if let Some(r) = layer.get("rotate").filter(|v| props::truthy(v)) {
        parts.push(format!("rotate({})", py::num(r)));
    }
    if let Some(s) = layer.get("scale").filter(|v| props::truthy(v)) {
        parts.push(format!("scale({})", py::num(s)));
    }
    parts.join(" ")
}

fn role_of(layer: &Map<String, Value>) -> String {
    layer
        .get("role")
        .map_or_else(|| "plain".to_owned(), props::text)
}

fn render_layer(layer: &Map<String, Value>, theme: &Map<String, Value>) -> Result<String, String> {
    let inner = if let Some(children) = layer.get("children") {
        let mut out = String::new();
        for child in children.as_array().map_or(&[][..], |x| x.as_slice()) {
            let child = child.as_object().ok_or("層は対応表でなければならない")?;
            out.push_str(&render_layer(child, theme)?);
        }
        out
    } else {
        let st = style::resolve(&role_of(layer), map_of(layer.get("style")), Some(theme))?;
        let kind = layer
            .get("kind")
            .map(props::text)
            .ok_or_else(|| py::quote("kind"))?;
        let empty = Map::new();
        let p = map_of(layer.get("props")).unwrap_or(&empty);
        registry::render(&kind, p, &st)?.svg
    };
    let transform = transform_of(layer);
    let g_attrs = if transform.is_empty() {
        String::new()
    } else {
        format!(" transform=\"{transform}\"")
    };
    if let Some(clip) = map_of(layer.get("clip")).filter(|m| !m.is_empty()) {
        // 実測で見つかった制約 ── 描き手（resvg）は、clipPath の中身が <g> で包まれていると
        // 切り抜きを丸ごと無視する。**位置合わせの変形は、clipPath 要素自身へ置く**
        let null = Value::Null;
        let material = [
            py::repr(clip.get("kind").unwrap_or(&null)),
            py::repr(clip.get("props").unwrap_or(&null)),
            py::repr(clip.get("role").unwrap_or(&null)),
            py::repr(clip.get("style").unwrap_or(&null)),
            py::quote(&transform_of(clip)),
        ];
        let cid = crate::ids::stable("clip", &material);
        let clip_style = style::resolve(&role_of(clip), map_of(clip.get("style")), Some(theme))?;
        let kind = clip
            .get("kind")
            .map(props::text)
            .ok_or_else(|| py::quote("kind"))?;
        let empty = Map::new();
        let clip_r = registry::render(
            &kind,
            map_of(clip.get("props")).unwrap_or(&empty),
            &clip_style,
        )?;
        let clip_transform = transform_of(clip);
        let transform_attr = if clip_transform.is_empty() {
            String::new()
        } else {
            format!(" transform=\"{clip_transform}\"")
        };
        let defs = format!(
            "<clipPath id=\"{cid}\"{transform_attr}>{}</clipPath>",
            clip_r.svg
        );
        return Ok(format!(
            "<defs>{defs}</defs><g{g_attrs} clip-path=\"url(#{cid})\">{inner}</g>"
        ));
    }
    Ok(if g_attrs.is_empty() {
        format!("<g>{inner}</g>")
    } else {
        format!("<g{g_attrs}>{inner}</g>")
    })
}

/// 好きな位置へ部品を重ねて、1枚の SVG に合成する。
///
/// # Errors
///
/// 層を描けないときに返す。
pub fn render_canvas(
    width: f64,
    height: f64,
    layers: &[Value],
    theme: Option<&Map<String, Value>>,
    background: Option<&str>,
) -> Result<String, String> {
    let theme = theme.unwrap_or_else(|| crate::theme::default_theme());
    let mut body = Vec::new();
    if let Some(bg) = background.filter(|b| !b.is_empty()) {
        body.push(format!(
            "<rect x=\"0\" y=\"0\" width=\"{width:.0}\" height=\"{height:.0}\" fill=\"{bg}\"/>"
        ));
    }
    for layer in layers {
        let layer = layer.as_object().ok_or("層は対応表でなければならない")?;
        body.push(render_layer(layer, theme)?);
    }
    Ok(format!(
        "<svg class=\"wf-fig\" viewBox=\"0 0 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\" role=\"img\">{}</svg>",
        body.concat()
    ))
}
