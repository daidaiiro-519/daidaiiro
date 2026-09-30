// SPDX-License-Identifier: MIT
//! 検査そのものの検査 ── 壊れた図を渡して鳴ること、正しい図で鳴らないこと。
//!
//! 検査は2度、静かに壊れた。1度目は祖先の変換を合成しておらず偽の重なりを出し、2度目は
//! 折れ線の端点しか見ておらず箱を突っ切る線を素通りさせた。**どちらも「破綻0件」と出ていた。**
//! だから、検査が実際に鳴ることを検証する試験を、検査と同じだけ持つ。

use ds_business_logic::verify::{check, check_attachment, check_shapes};

fn svg(inner: &str) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200">{inner}</svg>"#)
}

fn text(x: f64, y: f64, s: &str) -> String {
    format!(r#"<text x="{x}" y="{y}" font-size="14">{s}</text>"#)
}

fn boxed(x: f64, y: f64) -> String {
    format!(
        r#"<g class="svg-box" transform="translate({x},{y})"><rect x="0" y="0" width="80" height="40"/></g>"#
    )
}

fn any(found: &[String], word: &str) -> bool {
    found.iter().any(|f| f.contains(word))
}

#[test]
fn overlapping_text_is_reported() {
    assert!(any(
        &check(&svg(
            &(text(10.0, 50.0, "あいうえお") + &text(12.0, 52.0, "かきくけこ"))
        )),
        "重なる"
    ));
}

#[test]
fn separated_text_is_not_reported() {
    assert!(check(&svg(
        &(text(10.0, 50.0, "あいうえお") + &text(10.0, 120.0, "かきくけこ"))
    ))
    .is_empty());
}

#[test]
fn text_outside_canvas_is_reported() {
    assert!(any(
        &check(&svg(&text(360.0, 50.0, "はみ出す文字列"))),
        "画布の外"
    ));
}

#[test]
fn transformed_position_is_inspected() {
    // 祖先の translate を積まないと、この2つは離れて見えてしまう
    let inner = format!(
        r#"<g transform="translate(0,0)">{}</g><g transform="translate(2,2)">{}</g>"#,
        text(10.0, 50.0, "あいうえお"),
        text(10.0, 50.0, "かきくけこ")
    );
    assert!(any(&check(&svg(&inner)), "重なる"));
}

#[test]
fn text_separated_by_transform_is_not_reported() {
    let inner = format!(
        r#"<g transform="translate(0,0)">{}</g><g transform="translate(0,100)">{}</g>"#,
        text(10.0, 50.0, "あいうえお"),
        text(10.0, 50.0, "かきくけこ")
    );
    assert!(check(&svg(&inner)).is_empty());
}

#[test]
fn missing_viewbox_is_itself_a_defect() {
    assert_eq!(
        check(r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#),
        vec!["viewBoxが無い".to_owned()]
    );
}

#[test]
fn overlapping_boxes_are_reported() {
    assert!(
        check_shapes(&svg(&(boxed(10.0, 10.0) + &boxed(50.0, 20.0))))
            .contains(&"箱どうしが重なる".to_owned())
    );
}

#[test]
fn separated_boxes_are_not_reported() {
    assert!(check_shapes(&svg(&(boxed(10.0, 10.0) + &boxed(200.0, 10.0)))).is_empty());
}

const LINE: &str = r##"fill="none" stroke="#000" stroke-width="2"/>"##;

#[test]
fn line_crossing_a_box_is_reported() {
    // 両端は箱の外にある。端点だけを標本にする検査はこれを見逃した
    let inner = boxed(150.0, 80.0) + r#"<path d="M20,100 L380,100" "# + LINE;
    assert!(check_shapes(&svg(&inner)).contains(&"辺が箱を突っ切る".to_owned()));
}

#[test]
fn line_avoiding_boxes_is_not_reported() {
    let inner = boxed(150.0, 80.0) + r#"<path d="M20,20 L380,20" "# + LINE;
    assert!(check_shapes(&svg(&inner)).is_empty());
}

#[test]
fn line_attached_to_a_box_is_not_a_crossing() {
    // 始点が触れている箱は、その辺の相手なので除く
    let inner = boxed(150.0, 80.0) + r#"<path d="M170,100 L380,100" "# + LINE;
    assert!(check_shapes(&svg(&inner)).is_empty());
}

fn frame(x: f64, y: f64, w: f64, h: f64, extra: &str) -> String {
    format!(
        r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke-dasharray="4"{extra}/>"#
    )
}

#[test]
fn partially_intersecting_group_is_reported() {
    let inner = frame(10.0, 10.0, 120.0, 80.0, "") + &frame(80.0, 40.0, 120.0, 80.0, "");
    assert!(check_shapes(&svg(&inner)).contains(&"囲みが中途半端に交差する".to_owned()));
}

#[test]
fn nested_groups_are_not_reported() {
    let inner = frame(10.0, 10.0, 200.0, 150.0, "") + &frame(30.0, 30.0, 100.0, 80.0, "");
    assert!(check_shapes(&svg(&inner)).is_empty());
}

#[test]
fn group_border_biting_into_content_is_reported() {
    // 余白が0だと、囲みの縁が中の箱の縁に乗る。囲めていない
    let inner = frame(20.0, 20.0, 80.0, 40.0, r#" stroke-width="2""#) + &boxed(20.0, 20.0);
    assert!(check_shapes(&svg(&inner)).contains(&"囲みの線が中身に重なる".to_owned()));
}

#[test]
fn group_with_padding_is_not_reported() {
    let inner = frame(8.0, 8.0, 104.0, 64.0, r#" stroke-width="2""#) + &boxed(20.0, 20.0);
    assert!(check_shapes(&svg(&inner)).is_empty());
}

/// 出どころの節点と、着き先の節点と、その間の辺。**片端だけを節点に着けた図を作ると、
/// もう片端が（正しく）鳴って、試したい側が見えなくなる。**
fn fig(node: &str, d: &str, sw: f64) -> String {
    svg(&format!(
        r##"<g class="wf-node" transform="translate(10,10)"><rect x="0" y="0" width="20" height="20" fill="#eee"/></g><path d="{d}" fill="none" stroke="#000" stroke-width="{sw}"/><g class="wf-node" transform="translate(100,100)">{node}</g>"##
    ))
}

#[test]
fn endpoint_on_ink_is_not_reported() {
    let node = r##"<rect x="0" y="0" width="60" height="30" fill="#eee"/>"##;
    assert!(check_attachment(&fig(node, "M30,20 L100,100", 1.2)).is_empty());
}

#[test]
fn endpoint_on_blank_is_reported() {
    // 節点の外接矩形の中だが、インクは左半分にしか無い
    let node = r##"<rect x="0" y="0" width="20" height="30" fill="#eee"/>"##;
    assert!(any(
        &check_attachment(&fig(node, "M30,20 L160,100", 1.2)),
        "着いていない"
    ));
}

#[test]
fn inside_a_thick_stroke_counts_as_attached() {
    let node = r##"<path d="M0,0 L60,0" fill="none" stroke="#000" stroke-width="24"/>"##;
    assert!(check_attachment(&fig(node, "M30,20 L130,88", 1.2)).is_empty());
}

#[test]
fn sampling_coarseness_causes_no_false_report() {
    // 線分どうしで測らず点で測ると、標本の隙間に入った終端で誤検知する
    let node = r##"<path d="M0,0 L200,0" fill="none" stroke="#000" stroke-width="1"/>"##;
    assert!(check_attachment(&fig(node, "M30,20 L143.7,100", 1.2)).is_empty());
}

#[test]
fn says_nothing_without_node_markers() {
    assert!(check_attachment(&svg(
        r##"<path d="M0,0 L10,10" fill="none" stroke="#000"/>"##
    ))
    .is_empty());
}

#[test]
fn ink_fits_within_the_declared_size() {
    // **検査されない性質は、書いてあっても性質ではない。** 置いた後に描く部品は器を申告しないので対象外
    use ds_business_logic::geometry::sample_ink;
    use ds_business_logic::registry::{known_kinds, render, Origin};
    let st = ds_business_logic::style::resolve("plain", None, None).expect("解決できる");
    let m = st.num("size.stroke-width").expect("在る");
    let ex = ds_business_logic::catalog::examples();
    let mut out = Vec::new();
    for kind in known_kinds() {
        let r = render(kind, ex[kind].as_object().expect("対応表"), &st).expect("描ける");
        if r.origin != Origin::Own {
            continue;
        }
        let pts: Vec<(f64, f64)> = sample_ink(&r.svg, r.width.min(r.height).max(1.0) / 32.0)
            .into_iter()
            .map(|(p, _)| p)
            .collect();
        if pts.is_empty() {
            continue; // 文字だけの部品はこの測り方では見えない
        }
        let x0 = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let y0 = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let x1 = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let y1 = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        if x0 < -m || y0 < -m || x1 > r.width + m || y1 > r.height + m {
            out.push(format!(
                "{kind}: 申告 {:.0}x{:.0} / インク {x0:.1},{y0:.1}〜{x1:.1},{y1:.1}",
                r.width, r.height
            ));
        }
    }
    assert!(
        out.is_empty(),
        "申告した大きさの外へインクが出ている: {}",
        out.join(" / ")
    );
}

#[test]
fn deliberate_overflow_is_measured() {
    // この測り方自身が機能していることを検証する
    let pts = ds_business_logic::geometry::sample_ink(
        r#"<rect x="-9" y="0" width="20" height="10"/>"#,
        10.0 / 32.0,
    );
    assert!(pts.iter().any(|(p, _)| p.0 < -1.0));
}
