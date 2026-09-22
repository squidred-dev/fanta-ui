//! Canonical Lucide icon rendering for every Fanta-owned control.
//!
//! The icon catalog and complete SVG geometry come from the pinned
//! `lucide-static-svg` crate. Fanta does not maintain a parallel glyph set.

use std::{cell::RefCell, collections::HashMap};

use gpui::{
    AnyElement, Hsla, IntoElement as _, Path, PathBuilder, PathStyle, Pixels, Point, StrokeOptions,
    Styled as _, canvas, point, px,
};
use lyon::tessellation::{LineCap, LineJoin};
use svgtypes::{PointsParser, SimplePathSegment, SimplifyingPathParser};

pub use lucide_static_svg::Icon as LucideIcon;

const LUCIDE_GRID: f32 = 24.;
const LUCIDE_STROKE_WIDTH: f32 = 2.;
type IconPathCache = HashMap<(usize, u32), Option<Path<Pixels>>>;

thread_local! {
    // Keep tessellated paths at the origin. Parsing SVG XML and tessellating
    // every visible icon on every scroll frame dominates small repeated rows.
    static ICON_PATHS: RefCell<IconPathCache> = RefCell::new(HashMap::new());
}

fn translated_icon_path(
    icon: LucideIcon,
    size: f32,
    origin: Point<Pixels>,
) -> Option<Path<Pixels>> {
    let svg = icon.svg_str();
    let key = (svg.as_ptr() as usize, size.to_bits());
    ICON_PATHS.with(|paths| {
        let mut paths = paths.borrow_mut();
        let mut path = paths
            .entry(key)
            .or_insert_with(|| build_lucide_svg(Point::default(), size, svg))
            .clone()?;
        path.bounds.origin += origin;
        for vertex in &mut path.vertices {
            vertex.xy_position += origin;
        }
        Some(path)
    })
}

/// Renders one icon from the pinned upstream Lucide catalog.
pub fn render_lucide_icon(icon: LucideIcon, color: Hsla, size: f32) -> AnyElement {
    let size = size.max(1.);
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            if let Some(path) = translated_icon_path(icon, size, bounds.origin) {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .into_any_element()
}

fn build_lucide_svg(origin: Point<Pixels>, size: f32, svg: &str) -> Option<Path<Pixels>> {
    let scale = size / LUCIDE_GRID;
    let options = StrokeOptions::default()
        .with_line_width(LUCIDE_STROKE_WIDTH * scale)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round);
    let mut builder = PathBuilder::default().with_style(PathStyle::Stroke(options));
    let to_point = |x: f64, y: f64| {
        point(
            origin.x + px(x as f32 * scale),
            origin.y + px(y as f32 * scale),
        )
    };
    let number =
        |node: roxmltree::Node<'_, '_>, name: &str| node.attribute(name)?.parse::<f64>().ok();
    let document = roxmltree::Document::parse(svg).ok()?;

    for node in document.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "path" => {
                for segment in SimplifyingPathParser::from(node.attribute("d")?) {
                    append_path_segment(&mut builder, &to_point, segment.ok()?);
                }
            }
            "line" => {
                builder.move_to(to_point(number(node, "x1")?, number(node, "y1")?));
                builder.line_to(to_point(number(node, "x2")?, number(node, "y2")?));
            }
            "polyline" | "polygon" => {
                let mut points = PointsParser::from(node.attribute("points")?);
                let first = points.next()?;
                builder.move_to(to_point(first.0, first.1));
                for point in points {
                    builder.line_to(to_point(point.0, point.1));
                }
                if node.tag_name().name() == "polygon" {
                    builder.close();
                }
            }
            "circle" => append_ellipse(
                &mut builder,
                &to_point,
                number(node, "cx")?,
                number(node, "cy")?,
                number(node, "r")?,
                number(node, "r")?,
                scale,
            ),
            "ellipse" => append_ellipse(
                &mut builder,
                &to_point,
                number(node, "cx")?,
                number(node, "cy")?,
                number(node, "rx")?,
                number(node, "ry")?,
                scale,
            ),
            "rect" => append_rect(
                &mut builder,
                &to_point,
                (number(node, "x")?, number(node, "y")?),
                (number(node, "width")?, number(node, "height")?),
                node.attribute("rx")
                    .and_then(|value| value.parse::<f64>().ok())
                    .unwrap_or(0.),
                scale,
            ),
            _ => {}
        }
    }
    builder.build().ok()
}

pub(super) fn append_path_segment(
    builder: &mut PathBuilder,
    to_point: &impl Fn(f64, f64) -> Point<Pixels>,
    segment: SimplePathSegment,
) {
    match segment {
        SimplePathSegment::MoveTo { x, y } => builder.move_to(to_point(x, y)),
        SimplePathSegment::LineTo { x, y } => builder.line_to(to_point(x, y)),
        SimplePathSegment::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => builder.cubic_bezier_to(to_point(x, y), to_point(x1, y1), to_point(x2, y2)),
        SimplePathSegment::Quadratic { x1, y1, x, y } => {
            builder.curve_to(to_point(x, y), to_point(x1, y1));
        }
        SimplePathSegment::ClosePath => builder.close(),
    }
}

fn append_ellipse(
    builder: &mut PathBuilder,
    to_point: &impl Fn(f64, f64) -> Point<Pixels>,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    scale: f32,
) {
    let radii = point(px(rx as f32 * scale), px(ry as f32 * scale));
    builder.move_to(to_point(cx, cy - ry));
    builder.arc_to(radii, px(0.), false, true, to_point(cx, cy + ry));
    builder.arc_to(radii, px(0.), false, true, to_point(cx, cy - ry));
    builder.close();
}

fn append_rect(
    builder: &mut PathBuilder,
    to_point: &impl Fn(f64, f64) -> Point<Pixels>,
    origin: (f64, f64),
    size: (f64, f64),
    radius: f64,
    scale: f32,
) {
    let (x, y) = origin;
    let (width, height) = size;
    let right = x + width;
    let bottom = y + height;
    let radius = radius.min(width / 2.).min(height / 2.).max(0.);
    if radius == 0. {
        builder.move_to(to_point(x, y));
        builder.line_to(to_point(right, y));
        builder.line_to(to_point(right, bottom));
        builder.line_to(to_point(x, bottom));
        builder.close();
        return;
    }
    let radii = point(px(radius as f32 * scale), px(radius as f32 * scale));
    builder.move_to(to_point(x + radius, y));
    builder.line_to(to_point(right - radius, y));
    builder.arc_to(radii, px(0.), false, true, to_point(right, y + radius));
    builder.line_to(to_point(right, bottom - radius));
    builder.arc_to(radii, px(0.), false, true, to_point(right - radius, bottom));
    builder.line_to(to_point(x + radius, bottom));
    builder.arc_to(radii, px(0.), false, true, to_point(x, bottom - radius));
    builder.line_to(to_point(x, y + radius));
    builder.arc_to(radii, px(0.), false, true, to_point(x + radius, y));
    builder.close();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_complete_lucide_svg_builds() {
        for &icon in LucideIcon::all() {
            assert!(
                build_lucide_svg(point(px(0.), px(0.)), 24., icon.svg_str()).is_some(),
                "failed to build official Lucide icon {icon:?}"
            );
        }
    }

    #[test]
    fn catalog_version_is_intentionally_pinned() {
        assert_eq!(lucide_static_svg::LUCIDE_VERSION, "1.45.0");
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn cached_icon_geometry_matches_a_path_built_at_its_screen_position() {
        let origin = point(px(37.), px(91.));
        let icon = LucideIcon::Palette;
        let cached = translated_icon_path(icon, 12., origin).expect("cached palette path");
        let built = build_lucide_svg(origin, 12., icon.svg_str()).expect("built palette path");
        let close = |a: Pixels, b: Pixels| assert!((f32::from(a) - f32::from(b)).abs() < 0.001);
        close(cached.bounds.origin.x, built.bounds.origin.x);
        close(cached.bounds.origin.y, built.bounds.origin.y);
        close(cached.bounds.size.width, built.bounds.size.width);
        close(cached.bounds.size.height, built.bounds.size.height);
        assert_eq!(cached.vertices.len(), built.vertices.len());
        for (actual, expected) in cached.vertices.iter().zip(built.vertices.iter()) {
            close(actual.xy_position.x, expected.xy_position.x);
            close(actual.xy_position.y, expected.xy_position.y);
        }
    }
}
