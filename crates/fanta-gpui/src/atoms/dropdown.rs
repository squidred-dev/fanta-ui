//! Controlled UI3 dropdown trigger atom.

use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::h_flex;

use super::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor,
    TypographyExt as _, TypographyToken, render_lucide_icon, sidebar_style, sidebar_text_size,
    tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DropdownSize {
    #[default]
    Default,
    Large,
}

impl DropdownSize {
    pub const ALL: [Self; 2] = [Self::Default, Self::Large];

    pub const fn height(self) -> f32 {
        match self {
            Self::Default => tokens::RowHeight::FIELD,
            Self::Large => tokens::ControlSize::TOOL,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DropdownState {
    #[default]
    Default,
    Focused,
    Active,
}

impl DropdownState {
    pub const ALL: [Self; 3] = [Self::Default, Self::Focused, Self::Active];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Focused => "Focused",
            Self::Active => "Active",
        }
    }
}

#[derive(IntoElement)]
pub struct Dropdown {
    id: ElementId,
    value: SharedString,
    size: DropdownSize,
    state: DropdownState,
    disabled: bool,
    stroke: bool,
    full_width: bool,
    sidebar_style: bool,
    inset: bool,
    leading_icon: Option<LucideIcon>,
    on_activate: Option<ActivateHandler>,
}

impl Dropdown {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            size: DropdownSize::Default,
            state: DropdownState::Default,
            disabled: false,
            stroke: true,
            full_width: false,
            sidebar_style: false,
            inset: false,
            leading_icon: None,
            on_activate: None,
        }
    }

    pub const fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Uses Zed's sidebar palette and text size when its theme is installed.
    pub const fn sidebar_style(mut self, sidebar_style: bool) -> Self {
        self.sidebar_style = sidebar_style;
        self
    }

    /// Quiet filled field surface for dense inspector controls.
    pub const fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    pub const fn size(mut self, size: DropdownSize) -> Self {
        self.size = size;
        self
    }

    pub const fn preview_state(mut self, state: DropdownState) -> Self {
        self.state = state;
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn stroke(mut self, stroke: bool) -> Self {
        self.stroke = stroke;
        self
    }

    pub const fn leading_icon(mut self, icon: LucideIcon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn on_activate(
        mut self,
        handler: impl Fn(&ActivateEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Dropdown {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let focused = matches!(self.state, DropdownState::Focused | DropdownState::Active);
        let sidebar = self.sidebar_style.then(|| sidebar_style(cx));
        let foreground = if let Some(palette) = sidebar {
            if disabled {
                palette.disabled_text
            } else {
                palette.text
            }
        } else if disabled {
            SemanticColor::TextDisabled.resolve(cx)
        } else {
            SemanticColor::Text.resolve(cx)
        };
        let border = if let Some(palette) = sidebar {
            if !self.stroke {
                palette.border.opacity(0.)
            } else if focused {
                palette.focused_border
            } else {
                palette.border
            }
        } else if focused {
            SemanticColor::BorderSelected.resolve(cx)
        } else if self.stroke {
            SemanticColor::Border.resolve(cx)
        } else {
            SemanticColor::Background.resolve(cx).opacity(0.)
        };
        let background = sidebar.map_or_else(
            || {
                if disabled {
                    SemanticColor::BackgroundDisabled.resolve(cx)
                } else {
                    SemanticColor::Background.resolve(cx)
                }
            },
            |palette| {
                if self.inset {
                    SemanticColor::BackgroundPanelField.resolve(cx)
                } else {
                    palette.background
                }
            },
        );
        let hover_background = sidebar.map_or_else(
            || SemanticColor::BackgroundHover.resolve(cx),
            |palette| palette.hover,
        );
        let focused_border = sidebar.map_or_else(
            || SemanticColor::BorderSelected.resolve(cx),
            |palette| palette.focused_border,
        );
        let icon_foreground = sidebar.map_or_else(
            || {
                if disabled {
                    SemanticColor::IconDisabled.resolve(cx)
                } else {
                    SemanticColor::IconSecondary.resolve(cx)
                }
            },
            |palette| {
                if disabled {
                    palette.disabled_text
                } else {
                    palette.muted_icon
                }
            },
        );
        let icon = self.leading_icon;
        let handler = self.on_activate;

        h_flex()
            .id(self.id)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(if disabled { -1 } else { 0 })
            .w(px(tokens::DropdownGeometry::WIDTH))
            .when(self.full_width, |dropdown| dropdown.w_full())
            .h(px(self.size.height()))
            .items_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(border)
            .bg(background)
            .when(!disabled, |dropdown| {
                dropdown
                    .cursor_pointer()
                    .hover(|style| style.bg(hover_background))
                    .focus(|style| style.border_color(focused_border))
            })
            .when_some(icon, |dropdown, icon| {
                dropdown.child(
                    div()
                        .h_full()
                        .w(px(self.size.height()))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(sidebar.map_or_else(
                            || SemanticColor::BackgroundSecondary.resolve(cx),
                            |palette| palette.hover,
                        ))
                        .child(render_lucide_icon(icon, foreground, 14.)),
                )
            })
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .px_2()
                    .when(sidebar.is_some(), |label| {
                        label.text_size(sidebar_text_size())
                    })
                    .when(sidebar.is_none(), |label| {
                        label.typography(TypographyToken::BodyMedium)
                    })
                    .text_color(foreground)
                    .truncate()
                    .child(
                        div()
                            .when(self.state == DropdownState::Active, |label| {
                                label.bg(sidebar.map_or_else(
                                    || SemanticColor::BackgroundSelected.resolve(cx),
                                    |palette| palette.selected,
                                ))
                            })
                            .child(self.value),
                    ),
            )
            .child(
                div()
                    .w(px(tokens::RowHeight::FIELD))
                    .h_full()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(render_lucide_icon(
                        LucideIcon::ChevronDown,
                        icon_foreground,
                        12.,
                    )),
            )
            .when_some(handler.filter(|_| !disabled), |dropdown, handler| {
                dropdown.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropdown_sizes_match_the_figma_component_set() {
        assert_eq!(DropdownSize::Default.height(), 24.);
        assert_eq!(DropdownSize::Large.height(), 32.);
        assert_eq!(DropdownState::ALL.len(), 3);
    }
}
