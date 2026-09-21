//! Controlled UI3 radio-button atom.

use std::rc::Rc;

use gpui::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{StyledExt as _, h_flex};

use super::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, SemanticColor, TypographyExt as _,
    TypographyToken, tokens,
};

type SelectionHandler = Rc<dyn Fn(&RadioButtonSelection, &mut Window, &mut App)>;

/// Presentation axis from Figma's Radio button component set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum RadioButtonVariant {
    #[default]
    Input,
    Button,
}

impl RadioButtonVariant {
    pub const ALL: [Self; 2] = [Self::Input, Self::Button];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::Button => "Button",
        }
    }
}

/// State axis from Figma's Radio button component set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum RadioButtonState {
    #[default]
    Default,
    Active,
    Focused,
    Disabled,
}

impl RadioButtonState {
    pub const ALL: [Self; 4] = [Self::Default, Self::Active, Self::Focused, Self::Disabled];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Active => "Active",
            Self::Focused => "Focused",
            Self::Disabled => "Disabled",
        }
    }
}

/// Typed request emitted by a controlled radio button.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RadioButtonSelection {
    pub selected: bool,
    pub keyboard: bool,
}

/// A host-controlled radio button. The host owns group exclusivity and the
/// selected value; activation only requests `selected: true`.
#[derive(IntoElement)]
pub struct RadioButton {
    id: SharedString,
    label: SharedString,
    variant: RadioButtonVariant,
    state: RadioButtonState,
    selected: bool,
    label_visible: bool,
    on_select: Option<SelectionHandler>,
}

impl RadioButton {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: RadioButtonVariant::Input,
            state: RadioButtonState::Default,
            selected: false,
            label_visible: true,
            on_select: None,
        }
    }

    pub const fn variant(mut self, variant: RadioButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub const fn preview_state(mut self, state: RadioButtonState) -> Self {
        self.state = state;
        self
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub const fn label_visible(mut self, visible: bool) -> Self {
        self.label_visible = visible;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&RadioButtonSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RadioButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == RadioButtonState::Disabled;
        let focused = self.state == RadioButtonState::Focused;
        let active = self.state == RadioButtonState::Active;
        let selected = self.selected;
        let variant = self.variant;
        let indicator_border = if disabled {
            SemanticColor::IconDisabled.resolve(cx)
        } else if focused || selected {
            SemanticColor::BorderSelectedStrong.resolve(cx)
        } else {
            SemanticColor::IconSecondary.resolve(cx)
        };
        let label_color = if disabled {
            SemanticColor::TextDisabled.resolve(cx)
        } else {
            SemanticColor::Text.resolve(cx)
        };
        let handler = self.on_select;
        let selector = self.id.to_string();

        h_flex()
            .id(self.id)
            .debug_selector(move || selector.clone())
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(if disabled { -1 } else { 0 })
            .h(px(tokens::RowHeight::FIELD))
            .min_w(if variant == RadioButtonVariant::Button {
                px(tokens::RadioButtonGeometry::BUTTON_MIN_WIDTH)
            } else {
                px(tokens::ControlSize::INLINE)
            })
            .items_center()
            .gap_2()
            .px(if variant == RadioButtonVariant::Button {
                px(tokens::Space::SM)
            } else {
                px(tokens::Space::NONE)
            })
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(if focused {
                SemanticColor::BorderSelected.resolve(cx)
            } else {
                SemanticColor::Border.resolve(cx).opacity(0.)
            })
            .when(variant == RadioButtonVariant::Button, |row| {
                row.bg(if active {
                    SemanticColor::BackgroundActive.resolve(cx)
                } else {
                    SemanticColor::BackgroundSecondary.resolve(cx)
                })
            })
            .when(!disabled, |row| {
                row.cursor_pointer()
                    .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
                    .active(|style| style.bg(SemanticColor::BackgroundActive.resolve(cx)))
                    .focus(|style| style.border_color(SemanticColor::BorderSelected.resolve(cx)))
            })
            .child(
                div()
                    .size(px(tokens::RadioButtonGeometry::INDICATOR_SIZE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .border_1()
                    .border_color(indicator_border)
                    .bg(SemanticColor::Background.resolve(cx))
                    .when(selected, |indicator| {
                        indicator.child(
                            div()
                                .size(px(tokens::RadioButtonGeometry::DOT_SIZE))
                                .rounded_full()
                                .bg(if disabled {
                                    SemanticColor::IconDisabled.resolve(cx)
                                } else {
                                    SemanticColor::IconBrand.resolve(cx)
                                }),
                        )
                    }),
            )
            .when(self.label_visible, |row| {
                row.child(
                    div()
                        .min_w(px(tokens::Space::NONE))
                        .truncate()
                        .typography(TypographyToken::BodyMedium)
                        .font_medium()
                        .text_color(label_color)
                        .child(self.label),
                )
            })
            .when_some(handler.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event: &ActivateEvent, window, cx| {
                    handler(
                        &RadioButtonSelection {
                            selected: true,
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
    fn radio_axes_cover_the_twelve_valid_figma_variants() {
        let input_without_label = 2 * 2;
        let input_with_label = 2 * 2;
        let button = RadioButtonState::ALL.len();
        assert_eq!(input_without_label + input_with_label + button, 12);
        assert_eq!(RadioButtonVariant::ALL.len(), 2);
    }
}
