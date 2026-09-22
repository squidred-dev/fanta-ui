use gpui::{AnyElement, Hsla, IntoElement as _, PathBuilder, Styled as _, canvas, point, px};
use std::sync::LazyLock;
use svgtypes::{SimplePathSegment, SimplifyingPathParser};

const LOGO_WIDTH: f32 = 78.;
const LOGO_HEIGHT: f32 = 86.;
const LOGO_LEFT: f32 = 8.;

static LOGO_PATH: LazyLock<Vec<SimplePathSegment>> = LazyLock::new(|| {
    let svg = roxmltree::Document::parse(include_str!("fanta_logo.svg"))
        .expect("the bundled Fanta logo is valid SVG");
    svg.descendants()
        .filter_map(|node| node.attribute("d"))
        .flat_map(SimplifyingPathParser::from)
        .map(|segment| segment.expect("the bundled Fanta logo has valid paths"))
        .collect()
});

/// The Fanta brand mark, tinted with a host theme color.
pub fn render_fanta_logo(color: Hsla, height: f32) -> AnyElement {
    let scale = height / LOGO_HEIGHT;
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let mut builder = PathBuilder::fill();
            let to_point = |x: f64, y: f64| {
                point(
                    bounds.origin.x + px((x as f32 - LOGO_LEFT) * scale),
                    bounds.origin.y + px(y as f32 * scale),
                )
            };
            for segment in LOGO_PATH.iter() {
                super::lucide::append_path_segment(&mut builder, &to_point, *segment);
            }
            if let Ok(path) = builder.build() {
                window.paint_path(path, color);
            }
        },
    )
    .w(px(LOGO_WIDTH * scale))
    .h(px(height))
    .into_any_element()
}
