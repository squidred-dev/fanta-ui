//! Small, code-native toolbar icons.
//!
//! The toolbar deliberately draws its own monochrome paths so icons inherit the
//! active theme color, remain sharp at the compact 16 px size, and never depend
//! on platform fonts or host-provided assets.

use gpui::{AnyElement, Hsla};

use crate::atoms::vector_icon::{IconPath, render_icon_canvas};

use super::{ToolbarMode, ToolbarTool};

/// Renders a compact semantic icon for a toolbar tool.
pub(crate) fn render_tool_icon(tool: ToolbarTool, color: Hsla, size: f32) -> AnyElement {
    render_icon_canvas(color, size, move |path| draw_tool(path, tool))
}

/// Renders a compact semantic icon for a toolbar mode.
pub(crate) fn render_mode_icon(mode: ToolbarMode, color: Hsla, size: f32) -> AnyElement {
    render_icon_canvas(color, size, move |path| draw_mode(path, mode))
}

fn draw_mode(path: &mut IconPath, mode: ToolbarMode) {
    match mode {
        ToolbarMode::Draw => {
            path.move_to(1.75, 10.75);
            path.cubic_to((13.8, 5.1), (4.25, 3.2), (8.5, 13.25));
            path.poly([(11.9, 3.4), (14.3, 1.8), (13.8, 4.65)], false);
        }
        ToolbarMode::Design => {
            for (x, y, dx, dy) in [
                (2., 2., 3.3, 3.3),
                (14., 2., -3.3, 3.3),
                (2., 14., 3.3, -3.3),
                (14., 14., -3.3, -3.3),
            ] {
                path.move_to(x + dx, y);
                path.line_to(x, y);
                path.line_to(x, y + dy);
            }
            path.diamond(8., 8., 2.25);
        }
        ToolbarMode::Motion => {
            path.poly([(6., 4.), (12., 8.), (6., 12.)], true);
            path.line((1.75, 4.25), (3.75, 4.25));
            path.line((1.75, 8.), (3.75, 8.));
            path.line((1.75, 11.75), (3.75, 11.75));
        }
        ToolbarMode::Dev => {
            path.poly([(6.2, 3.25), (2., 8.), (6.2, 12.75)], false);
            path.poly([(9.8, 3.25), (14., 8.), (9.8, 12.75)], false);
            path.line((9.35, 1.75), (6.65, 14.25));
        }
    }
}

fn draw_tool(path: &mut IconPath, tool: ToolbarTool) {
    match tool {
        ToolbarTool::Move => {
            path.poly(
                [
                    (2.3, 1.75),
                    (12.25, 8.),
                    (7.85, 8.8),
                    (10.5, 13.1),
                    (8.4, 14.25),
                    (5.85, 9.95),
                    (2.3, 12.55),
                ],
                true,
            );
        }
        ToolbarTool::Hand => {
            path.move_to(3.4, 7.6);
            path.line_to(3.4, 5.45);
            path.curve_to((5.1, 5.05), (4.15, 4.65));
            path.line_to(5.1, 3.25);
            path.curve_to((6.8, 3.15), (5.9, 2.35));
            path.line_to(6.8, 2.35);
            path.curve_to((8.5, 2.75), (7.7, 1.65));
            path.line_to(8.5, 2.5);
            path.curve_to((10.2, 3.05), (9.45, 2.1));
            path.line_to(10.2, 3.45);
            path.curve_to((11.9, 4.35), (11.2, 3.45));
            path.line_to(11.9, 8.3);
            path.cubic_to((8.4, 14.), (11.9, 11.6), (10.55, 14.));
            path.cubic_to((4.75, 11.75), (6.25, 14.), (5.2, 13.));
            path.line_to(2.3, 8.55);
            path.curve_to((3.4, 7.6), (2.55, 7.3));
        }
        ToolbarTool::Scale => {
            path.rect(2., 9.75, 5.25, 13.);
            path.line((4.8, 10.2), (13.2, 1.8));
            path.poly([(9.3, 1.8), (13.2, 1.8), (13.2, 5.7)], false);
            path.line((7.1, 12.8), (12.8, 7.1));
        }
        ToolbarTool::Frame => {
            for (x, y, dx, dy) in [
                (2., 2., 4., 4.),
                (14., 2., -4., 4.),
                (2., 14., 4., -4.),
                (14., 14., -4., -4.),
            ] {
                path.move_to(x + dx, y);
                path.line_to(x, y);
                path.line_to(x, y + dy);
            }
        }
        ToolbarTool::Section => {
            path.poly(
                [
                    (2., 4.),
                    (5.4, 4.),
                    (6.5, 2.5),
                    (14., 2.5),
                    (14., 13.5),
                    (2., 13.5),
                ],
                true,
            );
            path.line((5.4, 4.), (14., 4.));
        }
        ToolbarTool::Slice => {
            path.poly([(5.75, 2.), (2., 2.), (2., 5.75)], false);
            path.poly([(10.25, 2.), (14., 2.), (14., 5.75)], false);
            path.poly([(2., 10.25), (2., 14.), (5.75, 14.)], false);
            path.poly([(10.25, 14.), (14., 14.), (14., 10.25)], false);
            path.line((5., 11.), (11., 5.));
        }
        ToolbarTool::Rectangle => path.rect(2.25, 3., 13.75, 13.),
        ToolbarTool::Line => path.line((2.25, 13.25), (13.75, 2.75)),
        ToolbarTool::Arrow => {
            path.line((2.25, 13.25), (13.2, 2.8));
            path.poly([(8.85, 2.8), (13.2, 2.8), (13.2, 7.15)], false);
        }
        ToolbarTool::Ellipse => path.ellipse(8., 8., 5.8, 4.9),
        ToolbarTool::Polygon => path.poly(
            [(8., 1.75), (14., 6.2), (11.7, 13.7), (4.3, 13.7), (2., 6.2)],
            true,
        ),
        ToolbarTool::Star => path.poly(
            [
                (8., 1.5),
                (9.75, 5.8),
                (14.25, 6.1),
                (10.8, 9.),
                (11.9, 13.5),
                (8., 11.),
                (4.1, 13.5),
                (5.2, 9.),
                (1.75, 6.1),
                (6.25, 5.8),
            ],
            true,
        ),
        ToolbarTool::ImageVideo => {
            path.rect(1.75, 3., 14.25, 13.);
            path.circle(5., 6.2, 1.2);
            path.poly(
                [(2.8, 12.), (6.5, 8.3), (9., 10.5), (11., 8.7), (14.1, 12.)],
                false,
            );
            path.poly([(10.7, 4.8), (13., 6.2), (10.7, 7.6)], true);
        }
        ToolbarTool::Pen => {
            path.poly(
                [(8., 1.6), (13.6, 7.2), (9.7, 13.5), (6.3, 13.5), (2.4, 7.2)],
                true,
            );
            path.line((8., 1.6), (8., 9.5));
            path.circle(8., 10.35, 0.85);
        }
        ToolbarTool::Pencil => {
            path.poly(
                [
                    (2.2, 11.1),
                    (10.8, 2.5),
                    (13.5, 5.2),
                    (4.9, 13.8),
                    (1.8, 14.2),
                ],
                true,
            );
            path.line((9.3, 4.), (12., 6.7));
            path.line((2.2, 11.1), (4.9, 13.8));
        }
        ToolbarTool::PathSelect => {
            path.poly(
                [
                    (2.25, 1.8),
                    (10.6, 7.1),
                    (7.1, 7.85),
                    (9.25, 11.4),
                    (7.35, 12.5),
                    (5.2, 8.85),
                    (2.25, 11.),
                ],
                true,
            );
            path.node(12.75, 12.75);
            path.line((9.1, 10.), (12., 12.));
        }
        ToolbarTool::NodeEdit => {
            path.move_to(2.5, 11.8);
            path.cubic_to((13.5, 4.2), (5., 2.8), (10.5, 13.2));
            path.line((2.5, 11.8), (5., 2.8));
            path.line((13.5, 4.2), (10.5, 13.2));
            path.node(2.5, 11.8);
            path.node(13.5, 4.2);
        }
        ToolbarTool::Text => {
            path.line((2.25, 2.5), (13.75, 2.5));
            path.line((8., 2.5), (8., 13.5));
            path.line((5.5, 13.5), (10.5, 13.5));
        }
        ToolbarTool::TextPath => {
            path.line((2.2, 2.5), (8.4, 2.5));
            path.line((5.3, 2.5), (5.3, 8.2));
            path.move_to(2., 12.2);
            path.cubic_to((14., 8.2), (5.5, 6.9), (10.2, 14.));
            path.node(13.8, 8.25);
        }
        ToolbarTool::Comment => {
            path.poly(
                [
                    (2., 2.5),
                    (14., 2.5),
                    (14., 11.),
                    (7.5, 11.),
                    (4., 14.),
                    (4., 11.),
                    (2., 11.),
                ],
                true,
            );
            path.line((5., 6.1), (11., 6.1));
            path.line((5., 8.3), (9., 8.3));
        }
        ToolbarTool::Annotation => {
            path.poly(
                [(3., 1.8), (12.5, 1.8), (12.5, 9.5), (8., 13.8), (3., 13.8)],
                true,
            );
            path.poly([(8., 13.8), (8., 9.5), (12.5, 9.5)], false);
            path.line((5.2, 5.), (10.2, 5.));
            path.line((5.2, 7.4), (9., 7.4));
        }
        ToolbarTool::Measure => {
            path.line((2., 5.), (14., 5.));
            path.poly([(4.5, 2.8), (2., 5.), (4.5, 7.2)], false);
            path.poly([(11.5, 2.8), (14., 5.), (11.5, 7.2)], false);
            path.line((3., 10.), (3., 13.5));
            path.line((6.3, 11.5), (6.3, 13.5));
            path.line((9.7, 10.), (9.7, 13.5));
            path.line((13., 11.5), (13., 13.5));
            path.line((2., 13.5), (14., 13.5));
        }
        ToolbarTool::Resources => {
            for (x, y) in [(5., 5.), (11., 5.), (5., 11.), (11., 11.)] {
                path.diamond(x, y, 2.25);
            }
        }
        ToolbarTool::Actions => {
            path.poly(
                [
                    (8., 1.5),
                    (9.2, 6.8),
                    (14.5, 8.),
                    (9.2, 9.2),
                    (8., 14.5),
                    (6.8, 9.2),
                    (1.5, 8.),
                    (6.8, 6.8),
                ],
                true,
            );
            path.plus(12.5, 3.5, 1.2);
        }
        ToolbarTool::Brush => {
            path.poly([(5.4, 10.2), (10.7, 2.), (14., 4.6), (7.25, 11.6)], true);
            path.cubic_to((2., 13.6), (5.7, 14.3), (3.1, 15.));
            path.cubic_to((5.4, 10.2), (2.15, 11.7), (4., 10.45));
            path.line((10.7, 2.), (14., 4.6));
        }
        ToolbarTool::PaintBucket => {
            path.poly(
                [(3., 5.2), (8., 2.), (13., 7.), (8., 12.), (2.5, 6.5)],
                true,
            );
            path.line((3., 5.2), (10.5, 5.2));
            path.move_to(12.3, 10.1);
            path.curve_to((12.3, 14.2), (15.5, 12.7));
            path.curve_to((12.3, 10.1), (9.1, 12.7));
        }
        ToolbarTool::ShapeBuilder => {
            path.circle(6.1, 8.6, 4.2);
            path.rect(7.1, 3.2, 13.9, 10.);
            path.line((7.3, 4.55), (10.05, 9.95));
        }
        ToolbarTool::Lasso => {
            path.move_to(12.8, 11.2);
            path.cubic_to((2.3, 8.), (11.4, 15.), (1.2, 13.2));
            path.cubic_to((13.2, 5.5), (3.1, 1.2), (12.3, 1.8));
            path.cubic_to((5.6, 11.5), (14.4, 8.5), (10.4, 12.2));
            path.cubic_to((13.9, 13.8), (8.2, 10.9), (11.7, 14.7));
        }
        ToolbarTool::VariableWidth => {
            path.move_to(1.8, 9.8);
            path.cubic_to((14.2, 6.2), (5., 7.8), (10., 2.8));
            path.move_to(1.8, 11.5);
            path.cubic_to((14.2, 4.5), (5., 9.5), (10., 1.1));
            path.line((1.8, 9.8), (1.8, 11.5));
            path.line((14.2, 6.2), (14.2, 4.5));
        }
        ToolbarTool::Inspect => {
            path.circle(6.9, 6.9, 4.3);
            path.line((10.1, 10.1), (14., 14.));
            path.line((6.9, 4.3), (6.9, 9.5));
            path.line((4.3, 6.9), (9.5, 6.9));
        }
        ToolbarTool::ColorPicker => {
            path.poly([(9.8, 1.7), (14.3, 6.2), (11.8, 8.7), (7.3, 4.2)], true);
            path.poly(
                [
                    (8.45, 5.35),
                    (3.4, 10.4),
                    (2.2, 13.8),
                    (5.6, 12.6),
                    (10.65, 7.55),
                ],
                false,
            );
            path.line((2.2, 13.8), (1.8, 14.2));
        }
        ToolbarTool::Code => {
            path.poly([(6.2, 3.), (2., 8.), (6.2, 13.)], false);
            path.poly([(9.8, 3.), (14., 8.), (9.8, 13.)], false);
            path.line((9.4, 1.8), (6.6, 14.2));
        }
        ToolbarTool::Variables => {
            path.diamond(3.3, 8., 1.8);
            path.diamond(12.7, 4., 1.8);
            path.diamond(12.7, 12., 1.8);
            path.line((5.1, 8.), (8., 8.));
            path.line((8., 8.), (10.9, 4.));
            path.line((8., 8.), (10.9, 12.));
        }
        ToolbarTool::ReadyForDev => {
            path.circle(8., 8., 6.1);
            path.check(4.6, 5.2);
        }
        ToolbarTool::MotionSelect => {
            path.poly(
                [
                    (2.2, 1.8),
                    (10.4, 7.),
                    (7., 7.8),
                    (9.25, 11.3),
                    (7.3, 12.5),
                    (5.1, 8.8),
                    (2.2, 11.),
                ],
                true,
            );
            path.diamond(12.5, 12.4, 1.6);
        }
        ToolbarTool::AddKeyframe => {
            path.diamond(6., 8., 4.3);
            path.plus(12.6, 4., 2.1);
        }
        ToolbarTool::MotionPath => {
            path.move_to(2.4, 11.8);
            path.cubic_to((13.6, 4.2), (4.4, 2.3), (11.6, 13.7));
            path.circle(2.4, 11.8, 1.2);
            path.diamond(13.6, 4.2, 1.45);
        }
        ToolbarTool::AnimationStyle => {
            path.poly(
                [
                    (7., 1.5),
                    (8., 5.),
                    (11.5, 6.),
                    (8., 7.),
                    (7., 10.5),
                    (6., 7.),
                    (2.5, 6.),
                    (6., 5.),
                ],
                true,
            );
            path.move_to(3., 13.5);
            path.cubic_to((14., 9.5), (6., 9.5), (10.5, 14.5));
            path.poly([(11.8, 8.5), (14., 9.5), (12.1, 11.)], false);
        }
        ToolbarTool::TimeComment => {
            path.circle(6.8, 6.5, 4.6);
            path.line((6.8, 3.7), (6.8, 6.5));
            path.line((6.8, 6.5), (9., 7.8));
            path.poly(
                [
                    (8.8, 11.),
                    (13.8, 11.),
                    (13.8, 14.2),
                    (12.2, 12.8),
                    (8.8, 12.8),
                ],
                true,
            );
        }
        ToolbarTool::AutoKeyframe => {
            path.move_to(3.15, 5.5);
            path.arc_to(5.1, 5.1, false, true, (12.85, 5.5));
            path.poly([(11., 2.7), (12.85, 5.5), (14.25, 2.4)], false);
            path.move_to(12.85, 10.5);
            path.arc_to(5.1, 5.1, false, true, (3.15, 10.5));
            path.poly([(5., 13.3), (3.15, 10.5), (1.75, 13.6)], false);
            path.diamond(8., 8., 2.2);
        }
        ToolbarTool::PlayPreview => {
            path.circle(8., 8., 6.2);
            path.poly([(6.5, 4.7), (11.2, 8.), (6.5, 11.3)], true);
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::{point, px};

    use crate::atoms::vector_icon::ICON_SIZE;

    use super::*;

    #[test]
    fn every_tool_and_mode_builds_a_valid_path() {
        let origin = point(px(0.), px(0.));

        for &tool in ToolbarTool::ALL {
            let mut path = IconPath::new(origin, ICON_SIZE);
            draw_tool(&mut path, tool);
            assert!(
                path.build().is_some(),
                "failed to build the {} tool icon",
                tool.label()
            );
        }

        for &mode in ToolbarMode::ALL {
            let mut path = IconPath::new(origin, ICON_SIZE);
            draw_mode(&mut path, mode);
            assert!(
                path.build().is_some(),
                "failed to build the {} mode icon",
                mode.label()
            );
        }
    }
}
