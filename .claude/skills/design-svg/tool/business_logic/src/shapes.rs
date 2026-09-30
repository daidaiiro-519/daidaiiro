// SPDX-License-Identifier: MIT
//! 組み込みの部品 ── **どれも構造だけを受け取り、色 ・ 寸法は見た目から引く。**
//!
//! 2系統あるのは、描く時点が置く前か後かで違うから。節点系（箱など）は置かれる前に描くので、
//! 自分の原点を基準に描く。辺 ・ 囲み ・ 囲みのラベルは、既に置かれた節点をまたぐので、置いた
//! 後にしか描けない ── 最初から絶対座標を受け取る。

use serde_json::Value;

use crate::props::{self, esc};
use crate::py::float as f;
use crate::registry::{Component, Fragment, Props};
use crate::style::Style;
use crate::text;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![
        ("box", box_ as Component),
        ("panel", panel),
        ("dot", dot),
        ("edge", edge),
        ("frame_label", frame_label),
        ("frame", frame),
    ]
}

/// 色の濃さの呼び名から、色を引く。**知らない名前は誤りにする。**
///
/// # Errors
///
/// 知らない名前のときと、見た目に無いときに返す。
pub fn tone_color(style: &Style, name: &str) -> Result<String, String> {
    let key = crate::theme::tone(name).ok_or_else(|| crate::py::quote(name))?;
    style.text(key)
}

/// 矢じりを1つ描く。**向きは角度で受け取る。**
///
/// 形はトークンから決まる（幅の比 ・ 長さの比 ・ 開き角）── 部品ごとに書くと、テーマで
/// 矢じりを大きくしても一部だけ変わらない（実測 ── やり取りの図の矢じりだけが追随しなかった）。
///
/// # Errors
///
/// 見た目に要るトークンが無いときに返す。
pub fn arrow_head(
    tip: (f64, f64),
    angle: f64,
    style: &Style,
    color: Option<&str>,
    shape: &str,
) -> Result<String, String> {
    let (x2, y2) = tip;
    let head_w = (style.num("size.stroke-width")? * style.num("size.arrowhead-w-ratio")?)
        .max(style.num("size.arrowhead-min")?);
    let head_len = head_w * style.num("size.arrowhead-len-ratio")?;
    let spread = style.num("size.arrowhead-angle")?.to_radians();
    let hx1 = x2 - head_len * (angle - spread).cos();
    let hy1 = y2 - head_len * (angle - spread).sin();
    let hx2 = x2 - head_len * (angle + spread).cos();
    let hy2 = y2 - head_len * (angle + spread).sin();
    let ink = match color {
        Some(c) if !c.is_empty() => c.to_owned(),
        _ => style.text_or("color.line", &style.text("color.ink-faint")?)?,
    };
    let sw = f(style.num("size.stroke-width")?);
    match shape {
        "open" => Ok(format!(
            "<path class=\"wf-head\" d=\"M{hx1:.1},{hy1:.1} L{x2:.1},{y2:.1} L{hx2:.1},{hy2:.1}\" fill=\"none\" stroke=\"{ink}\" stroke-width=\"{sw}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
        )),
        // **かぎ** ── 線の端を前へ曲げて折り返す半円。UML の図でユースケースの拡張を表す
        // コネクタに使う（コーバーン『ユースケース実践ガイド』図10.1 ・ 付録 A）
        "hook" => {
            let r = head_len * style.num("size.hook-radius-ratio")?;
            let (nx, ny) = (-angle.sin(), angle.cos());
            let (ex, ey) = (x2 + nx * 2.0 * r, y2 + ny * 2.0 * r);
            let (bx, by) = (ex - angle.cos() * r, ey - angle.sin() * r);
            Ok(format!(
                "<path class=\"wf-head\" d=\"M{x2:.1},{y2:.1} A{r:.1},{r:.1} 0 0 1 {ex:.1},{ey:.1} L{bx:.1},{by:.1}\" fill=\"none\" stroke=\"{ink}\" stroke-width=\"{sw}\" stroke-linecap=\"round\"/>"
            ))
        }
        "solid" => Ok(format!(
            "<polygon points=\"{x2:.1},{y2:.1} {hx1:.1},{hy1:.1} {hx2:.1},{hy2:.1}\" fill=\"{ink}\"/>"
        )),
        other => Err(format!("矢じりの形は solid ・ open ・ hook のどれか ── {other}")),
    }
}

/// 名前を1つ持つ、角丸の矩形。**つながり ・ 階層 ・ 包含などの節点に使う。**
fn box_(p: &Props, style: &Style) -> Result<Fragment, String> {
    let label = props::text_or(p, "label", "");
    let font_size = style.num("font.size")?;
    let pad_x = style.num("size.box-pad-x")?;
    let latin = style.num("font.latin-width-ratio")?;
    // **上限を超えるラベルは折り返す** ── 行ごとに1つの text にするので、検査は各行をそのまま測れる
    let lines = text::wrap(
        &label,
        font_size,
        latin,
        style.num("size.box-max-w")? - pad_x * 2.0,
    );
    let widest = lines
        .iter()
        .map(|l| text::width_with(l, font_size, latin))
        .fold(0.0_f64, f64::max);
    let w = style.num("size.box-min-w")?.max(widest + pad_x * 2.0);
    let line_h = font_size * style.num("size.box-line-h")?;
    let extra = line_h * (lines.len().saturating_sub(1)) as f64;
    let h = style.num("size.box-h")? + extra;
    let radius = style.num("size.box-radius")?;
    let fill = style.text("color.box-fill")?;
    let stroke = style.text("color.box-stroke")?;
    let sw = style.num("size.stroke-width")?;
    let dash = style
        .opt_text("stroke-dasharray")
        .filter(|d| !d.is_empty())
        .map_or_else(String::new, |d| format!(" stroke-dasharray=\"{d}\""));
    let text_color = style.text_or("color.text", &style.text("color.ink")?)?;
    let weight = style.text_or("font.weight", "400")?;
    let role = esc(&props::text_or(p, "role", "plain"));
    let first_y = h / 2.0 - extra / 2.0 + font_size * style.num("font.baseline-ratio")?;
    let family = style.text("font.family")?;
    let texts: String = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{weight}\" fill=\"{text_color}\">{}</text>",
                w / 2.0,
                first_y + line_h * i as f64,
                family,
                f(font_size),
                esc(l)
            )
        })
        .collect();
    let svg = format!(
        "<g class=\"svg-box\" role=\"{role}\"><rect x=\"0\" y=\"0\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\"{dash}/>{texts}</g>",
        f(radius),
        f(sw),
    );
    Ok(Fragment::own(svg, w, h).labelled())
}

/// 素の面 ── 塗りと枠だけを持つ矩形。
///
/// **囲み（frame）とは別である** ── 囲みは「ここは領域の内側」を示す破線の注記で、面は意匠
/// そのものである。同じ部品で兼ねると、どちらの意味で置いたかが読めない。
fn panel(p: &Props, style: &Style) -> Result<Fragment, String> {
    let w = props::need_num(p, "width")?;
    let h = props::need_num(p, "height")?;
    let r = match props::num(p, "radius") {
        Some(r) => r,
        None => style.num("size.box-radius")?,
    };
    let tone = |name: &str, fallback: &str| -> Result<String, String> {
        if name == "none" {
            return Ok("none".to_owned());
        }
        let key = crate::theme::tone(name).ok_or_else(|| crate::py::quote(name))?;
        style.text_or(key, &style.text(fallback)?)
    };
    let fill = tone(&props::text_or(p, "fill", "fill"), "color.box-fill")?;
    let stroke = tone(&props::text_or(p, "stroke", "line"), "color.line")?;
    let sw = style.num("size.stroke-width")?;
    let svg = format!(
        "<rect x=\"0\" y=\"0\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{r:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\"/>",
        f(sw)
    );
    Ok(Fragment::own(svg, w, h).ints(props::is_int(p, "width"), props::is_int(p, "height")))
}

/// 始点 ・ 終点の印などに使う、塗りつぶした円。
fn dot(p: &Props, style: &Style) -> Result<Fragment, String> {
    let r = match props::num(p, "radius") {
        Some(r) => r,
        None => style.num("size.dot-radius")?,
    };
    let fill = match p.get("tone").filter(|v| props::truthy(v)) {
        Some(t) => tone_color(style, &props::text(t))?,
        None => style.text_or("color.text", &style.text("color.ink")?)?,
    };
    let line = match p.get("stroke").filter(|v| props::truthy(v)) {
        Some(e) => format!(
            " stroke=\"{}\" stroke-width=\"{}\"",
            tone_color(style, &props::text(e))?,
            f(style.num("size.stroke-width")?)
        ),
        None => String::new(),
    };
    let svg = format!("<circle cx=\"{r:.1}\" cy=\"{r:.1}\" r=\"{r:.1}\" fill=\"{fill}\"{line}/>");
    Ok(Fragment::own(svg, r * 2.0, r * 2.0))
}

/// 点列を、角を丸めたパスへ変換する。**2点なら直線のまま。**
///
/// 3点以上（複数の層をまたぐ辺が仮節点を経由した場合）は、各中間点を二次ベジェで滑らかに
/// つなぐ ── 折れ線のまま出すより、辺だと分かりやすい。
#[must_use]
pub fn smooth_path(points: &[(f64, f64)]) -> String {
    if points.len() < 3 {
        let ((x1, y1), (x2, y2)) = (points[0], points[points.len() - 1]);
        return format!("M{x1:.1},{y1:.1} L{x2:.1},{y2:.1}");
    }
    let mut d = vec![format!("M{:.1},{:.1}", points[0].0, points[0].1)];
    for i in 1..points.len() - 1 {
        let (px, py) = points[i];
        let (nx, ny) = points[i + 1];
        let (mx, my) = ((px + nx) / 2.0, (py + ny) / 2.0);
        d.push(format!("Q{px:.1},{py:.1} {mx:.1},{my:.1}"));
    }
    let (lx, ly) = points[points.len() - 1];
    d.push(format!("L{lx:.1},{ly:.1}"));
    d.join(" ")
}

/// 複数の点を通って結ばれる線。**矢じり ・ ラベル ・ 破線を持てる。**
fn edge(p: &Props, style: &Style) -> Result<Fragment, String> {
    let points = props::points(p.get("points").ok_or_else(|| crate::py::quote("points"))?);
    if points.len() < 2 {
        return Err("辺には点が2つ以上要る".to_owned());
    }
    let (x2, y2) = points[points.len() - 1];
    let color = style.text_or("color.line", &style.text("color.ink-faint")?)?;
    let sw = style.num("size.stroke-width")?;
    let dash = if props::flag(p, "dashed") {
        " stroke-dasharray=\"4 3\""
    } else {
        ""
    };
    let path_d = smooth_path(&points);
    let mut marker = String::new();
    if props::text_or(p, "arrow", "head") != "none" {
        // 矢じりの向きは、実際に終点へ入る最後の線分の傾きから決める
        let (px, py) = if points.len() >= 3 {
            let (a, b) = (points[points.len() - 2], points[points.len() - 1]);
            ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
        } else {
            points[points.len() - 2]
        };
        let ang = (y2 - py).atan2(x2 - px);
        marker = arrow_head(
            (x2, y2),
            ang,
            style,
            Some(&color),
            &props::text_or(p, "arrowhead", "solid"),
        )?;
    }
    let mut label_svg = String::new();
    if props::flag(p, "label") {
        let label = props::text_or(p, "label", "");
        let (mx, my) = match p
            .get("label_at")
            .filter(|v| props::truthy(v))
            .and_then(props::point)
        {
            Some(at) => at,
            None => {
                let mi = points.len() / 2;
                if points.len() % 2 == 1 {
                    points[mi]
                } else {
                    (
                        (points[mi - 1].0 + points[mi].0) / 2.0,
                        (points[mi - 1].1 + points[mi].1) / 2.0,
                    )
                }
            }
        };
        let fs = style.num("font.size-small")?;
        let text_w = text::width_with(&label, fs, style.num("font.latin-width-ratio")?)
            + style.num("size.label-pad-x")?;
        label_svg = format!(
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{text_w:.1}\" height=\"{:.1}\" fill=\"{}\"/><text x=\"{mx:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            mx - text_w / 2.0,
            my - fs,
            style.num("size.label-band-h")?,
            style.text("color.box-fill")?,
            my + fs * style.num("font.baseline-ratio")?,
            style.text("font.family")?,
            f(fs),
            style.text("color.ink-faint")?,
            esc(&label)
        );
    }
    let svg = format!(
        "<path d=\"{path_d}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\"{dash}/>{marker}{label_svg}",
        f(sw)
    );
    // 点が全部整数で書かれていれば、広がりも整数のまま来る
    let all_int = p.get("points").and_then(Value::as_array).is_some_and(|a| {
        a.iter().all(|q| {
            q.as_array()
                .is_some_and(|c| c.iter().all(|v| v.is_i64() || v.is_u64()))
        })
    });
    let xs = points.iter().map(|q| q.0);
    let ys = points.iter().map(|q| q.1);
    let (x_lo, x_hi) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
        (a.min(v), b.max(v))
    });
    let (y_lo, y_hi) = ys.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
        (a.min(v), b.max(v))
    });
    Ok(Fragment::absolute(svg, x_hi - x_lo, y_hi - y_lo).ints(all_int, all_int))
}

/// 囲みのラベルだけを描く。**枠線とは別の層に置くための部品。**
///
/// ラベルは不透明な帯を持つので、線の上に載れば線を断って読める ── 線は曲げない。
fn frame_label(p: &Props, style: &Style) -> Result<Fragment, String> {
    let label = props::need_text(p, "label")?;
    let fs = style.num("font.size-small")?;
    let pad_x = style.num("size.label-pad-x")?;
    let text_w = text::width_with(&label, fs, style.num("font.latin-width-ratio")?) + pad_x;
    let rise = fs * style.num("size.frame-label-rise")?;
    let card_h = fs * style.num("size.frame-label-h")?;
    let base = fs * style.num("size.frame-label-baseline")?;
    let x = props::need_num(p, "x")?;
    let y = props::need_num(p, "y")?;
    let svg = format!(
        "<rect x=\"{x:.1}\" y=\"{:.1}\" width=\"{text_w:.1}\" height=\"{card_h:.1}\" fill=\"{}\"/><text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>",
        y - rise,
        style.text("color.box-fill")?,
        x + text_w / 2.0,
        y - rise + base,
        style.text("font.family")?,
        f(fs),
        style.text("color.accent")?,
        esc(&label)
    );
    Ok(Fragment::absolute(svg, text_w, card_h))
}

/// 区画を示す破線の囲み。**ラベルは描かない** ── ラベルは線より後に描く必要があり、置き場所も
/// 線を避けて決まるので、`frame_label` が受け持つ。
fn frame(p: &Props, style: &Style) -> Result<Fragment, String> {
    let (x, y) = (props::need_num(p, "x")?, props::need_num(p, "y")?);
    let (w, h) = (props::need_num(p, "width")?, props::need_num(p, "height")?);
    let svg = format!(
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-dasharray=\"5 4\" opacity=\"{}\"/>",
        f(style.num("size.radius-large")?),
        style.text("color.accent")?,
        f(style.num("size.stroke-width-thin")?),
        f(style.num("opacity.soft")?)
    );
    Ok(Fragment::absolute(svg, w, h).ints(props::is_int(p, "width"), props::is_int(p, "height")))
}

/// 値を点として読めるか。**呼ぶ側の検査に使う。**
#[must_use]
pub fn is_point(v: &Value) -> bool {
    props::point(v).is_some()
}
