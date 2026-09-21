//! UI3 tooltip surfaces and actionable tooltip-link rows.

use std::rc::Rc;

use gpui::{
    App, Bounds, ElementId, FillOptions, InteractiveElement as _, IntoElement, ParentElement as _,
    PathBuilder, PathStyle, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _,
    Styled as _, Window, canvas, div, point, prelude::FluentBuilder as _, px,
};
use gpui_component::{StyledExt as _, h_flex, v_flex};

use super::{
    CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor, TypographyExt as _,
    TypographyToken, render_lucide_icon, tokens,
};

type LinkHandler = Rc<dyn Fn(&TooltipLinkAction, &mut Window, &mut App)>;

/// Caret location from Figma's Tooltip component set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TooltipDirection {
    TopLeft,
    #[default]
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Left,
    Right,
}

impl TooltipDirection {
    pub const ALL: [Self; 8] = [
        Self::TopCenter,
        Self::BottomCenter,
        Self::BottomLeft,
        Self::BottomRight,
        Self::TopLeft,
        Self::TopRight,
        Self::Right,
        Self::Left,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Top left",
            Self::TopCenter => "Top center",
            Self::TopRight => "Top right",
            Self::BottomLeft => "Bottom left",
            Self::BottomCenter => "Bottom center",
            Self::BottomRight => "Bottom right",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

#[derive(Clone, Copy)]
enum CaretDirection {
    Up,
    Down,
    Left,
    Right,
}

fn tooltip_caret(direction: CaretDirection, cx: &App) -> impl IntoElement {
    let color = SemanticColor::BackgroundTooltip.resolve(cx);
    let (width, height) = match direction {
        CaretDirection::Up | CaretDirection::Down => (
            tokens::TooltipGeometry::CARET_WIDTH,
            tokens::TooltipGeometry::CARET_HEIGHT,
        ),
        CaretDirection::Left | CaretDirection::Right => (
            tokens::TooltipGeometry::CARET_HEIGHT,
            tokens::TooltipGeometry::CARET_WIDTH,
        ),
    };
    canvas(
        |_, _, _| {},
        move |bounds: Bounds<Pixels>, _, window, _| {
            let left = bounds.origin.x;
            let top = bounds.origin.y;
            let right = left + bounds.size.width;
            let bottom = top + bounds.size.height;
            let center_x = left + bounds.size.width / 2.;
            let center_y = top + bounds.size.height / 2.;
            let mut path =
                PathBuilder::default().with_style(PathStyle::Fill(FillOptions::default()));
            match direction {
                CaretDirection::Up => {
                    path.move_to(point(center_x, top));
                    path.line_to(point(right, bottom));
                    path.line_to(point(left, bottom));
                }
                CaretDirection::Down => {
                    path.move_to(point(left, top));
                    path.line_to(point(right, top));
                    path.line_to(point(center_x, bottom));
                }
                CaretDirection::Left => {
                    path.move_to(point(left, center_y));
                    path.line_to(point(right, top));
                    path.line_to(point(right, bottom));
                }
                CaretDirection::Right => {
                    path.move_to(point(left, top));
                    path.line_to(point(right, center_y));
                    path.line_to(point(left, bottom));
                }
            }
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .w(px(width))
    .h(px(height))
}

fn tooltip_body(
    id: ElementId,
    label: SharedString,
    hotkey: Option<SharedString>,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .id(id)
        .min_h(px(tokens::RowHeight::PAGE))
        .items_center()
        .gap_2()
        .px_2()
        .rounded(px(tokens::TooltipGeometry::RADIUS))
        .bg(SemanticColor::BackgroundTooltip.resolve(cx))
        .shadow_lg()
        .child(
            div()
                .typography(TypographyToken::BodyMedium)
                .font_medium()
                .text_color(SemanticColor::Text.resolve(cx))
                .child(label),
        )
        .when_some(hotkey, |surface, hotkey| {
            surface.child(
                div()
                    .h(px(tokens::ControlSize::INLINE))
                    .px_1()
                    .flex()
                    .items_center()
                    .rounded(px(tokens::Radius::CONTROL))
                    .bg(SemanticColor::BackgroundSecondary.resolve(cx))
                    .typography(TypographyToken::BodySmall)
                    .text_color(SemanticColor::TextSecondary.resolve(cx))
                    .child(hotkey),
            )
        })
}

/// A tooltip surface. Visibility and placement relative to a trigger remain
/// host-owned; this component renders the eight Figma caret directions.
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    label: SharedString,
    direction: TooltipDirection,
    hotkey: Option<SharedString>,
}

impl Tooltip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            direction: TooltipDirection::TopCenter,
            hotkey: None,
        }
    }

    pub const fn direction(mut self, direction: TooltipDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn hotkey(mut self, hotkey: impl Into<SharedString>) -> Self {
        self.hotkey = Some(hotkey.into());
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = self.direction;
        let body = tooltip_body(self.id, self.label, self.hotkey, cx).into_any_element();
        match direction {
            TooltipDirection::TopLeft
            | TooltipDirection::TopCenter
            | TooltipDirection::TopRight => v_flex()
                .items_start()
                .when(direction == TooltipDirection::TopCenter, |column| {
                    column.items_center()
                })
                .when(direction == TooltipDirection::TopRight, |column| {
                    column.items_end()
                })
                .child(tooltip_caret(CaretDirection::Up, cx))
                .child(body)
                .into_any_element(),
            TooltipDirection::BottomLeft
            | TooltipDirection::BottomCenter
            | TooltipDirection::BottomRight => v_flex()
                .items_start()
                .when(direction == TooltipDirection::BottomCenter, |column| {
                    column.items_center()
                })
                .when(direction == TooltipDirection::BottomRight, |column| {
                    column.items_end()
                })
                .child(body)
                .child(tooltip_caret(CaretDirection::Down, cx))
                .into_any_element(),
            TooltipDirection::Left => h_flex()
                .items_center()
                .child(tooltip_caret(CaretDirection::Left, cx))
                .child(body)
                .into_any_element(),
            TooltipDirection::Right => h_flex()
                .items_center()
                .child(body)
                .child(tooltip_caret(CaretDirection::Right, cx))
                .into_any_element(),
        }
    }
}

/// Variants from Figma's Tooltip link component set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TooltipLinkVariant {
    #[default]
    Url,
    Link,
    Phone,
    Email,
    Page,
    Prototype,
    Frame,
    File,
}

impl TooltipLinkVariant {
    pub const ALL: [Self; 8] = [
        Self::Url,
        Self::Link,
        Self::Phone,
        Self::Email,
        Self::Page,
        Self::Prototype,
        Self::Frame,
        Self::File,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Url => "URL",
            Self::Link => "Link",
            Self::Phone => "Phone",
            Self::Email => "Email",
            Self::Page => "Page",
            Self::Prototype => "Prototype",
            Self::Frame => "Frame",
            Self::File => "File",
        }
    }

    const fn icon(self) -> Option<LucideIcon> {
        match self {
            Self::Url => None,
            Self::Link => Some(LucideIcon::Link),
            Self::Phone => Some(LucideIcon::Phone),
            Self::Email => Some(LucideIcon::Mail),
            Self::Page => Some(LucideIcon::File),
            Self::Prototype => Some(LucideIcon::Play),
            Self::Frame => Some(LucideIcon::Frame),
            Self::File => Some(LucideIcon::Folder),
        }
    }
}

/// Typed activation emitted by a Tooltip link row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TooltipLinkAction {
    pub variant: TooltipLinkVariant,
    pub keyboard: bool,
}

/// An actionable tooltip-link row. The host decides whether activation opens,
/// calls, copies, navigates, or edits a destination.
#[derive(IntoElement)]
pub struct TooltipLink {
    id: SharedString,
    variant: TooltipLinkVariant,
    label: SharedString,
    secondary: Option<SharedString>,
    hotkey: Option<SharedString>,
    on_activate: Option<LinkHandler>,
}

impl TooltipLink {
    pub fn new(
        id: impl Into<SharedString>,
        variant: TooltipLinkVariant,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            variant,
            label: label.into(),
            secondary: None,
            hotkey: None,
            on_activate: None,
        }
    }

    pub fn secondary(mut self, secondary: impl Into<SharedString>) -> Self {
        self.secondary = Some(secondary.into());
        self
    }

    pub fn hotkey(mut self, hotkey: impl Into<SharedString>) -> Self {
        self.hotkey = Some(hotkey.into());
        self
    }

    pub fn on_activate(
        mut self,
        handler: impl Fn(&TooltipLinkAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TooltipLink {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let variant = self.variant;
        let handler = self.on_activate;
        let secondary = self.secondary;
        let selector = self.id.to_string();

        h_flex()
            .id(self.id)
            .debug_selector(move || selector.clone())
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .w(px(tokens::TooltipGeometry::LINK_WIDTH))
            .min_h(px(tokens::RowHeight::PAGE))
            .items_center()
            .gap_2()
            .px_2()
            .rounded(px(tokens::TooltipGeometry::RADIUS))
            .bg(SemanticColor::BackgroundTooltip.resolve(cx))
            .cursor_pointer()
            .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
            .active(|style| style.bg(SemanticColor::BackgroundActive.resolve(cx)))
            .focus(|style| style.border_color(SemanticColor::BorderSelected.resolve(cx)))
            .border_1()
            .border_color(SemanticColor::Border.resolve(cx).opacity(0.))
            .when_some(variant.icon(), |row, icon| {
                row.child(render_lucide_icon(
                    icon,
                    SemanticColor::IconSecondary.resolve(cx),
                    tokens::IconSize::SM,
                ))
            })
            .child(
                div()
                    .min_w(px(tokens::Space::NONE))
                    .flex_1()
                    .truncate()
                    .typography(TypographyToken::BodyMedium)
                    .text_color(if variant == TooltipLinkVariant::Url {
                        SemanticColor::TextTertiary.resolve(cx)
                    } else {
                        SemanticColor::Text.resolve(cx)
                    })
                    .child(self.label),
            )
            .when_some(secondary, |row, secondary| {
                row.child(
                    div()
                        .typography(TypographyToken::BodySmall)
                        .text_color(SemanticColor::TextSecondary.resolve(cx))
                        .child(secondary),
                )
            })
            .when_some(self.hotkey, |row, hotkey| {
                row.child(
                    div()
                        .typography(TypographyToken::BodySmall)
                        .text_color(SemanticColor::TextTertiary.resolve(cx))
                        .child(hotkey),
                )
            })
            .when_some(handler, |row, handler| {
                row.on_activate(move |event, window, cx| {
                    handler(
                        &TooltipLinkAction {
                            variant,
                            keyboard: event.keyboard,
                        },
                        window,
                        cx,
                    );
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_sets_cover_all_linked_figma_variants() {
        assert_eq!(TooltipDirection::ALL.len(), 8);
        assert_eq!(TooltipLinkVariant::ALL.len(), 8);
    }
}
