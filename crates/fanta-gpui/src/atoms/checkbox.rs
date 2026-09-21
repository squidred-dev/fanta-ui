//! Controlled UI3 checkbox atom.

use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{StyledExt as _, h_flex};

use super::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor,
    TypographyExt as _, TypographyToken, render_lucide_icon, tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CheckboxType {
    Checked,
    #[default]
    Unchecked,
    Mixed,
}

impl CheckboxType {
    pub const ALL: [Self; 3] = [Self::Checked, Self::Unchecked, Self::Mixed];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Checked => "Checked",
            Self::Unchecked => "Unchecked",
            Self::Mixed => "Mixed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CheckboxState {
    #[default]
    Default,
    Focused,
}

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: SharedString,
    value: CheckboxType,
    state: CheckboxState,
    disabled: bool,
    muted: bool,
    ghost: bool,
    on_activate: Option<ActivateHandler>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: CheckboxType::Unchecked,
            state: CheckboxState::Default,
            disabled: false,
            muted: false,
            ghost: false,
            on_activate: None,
        }
    }

    pub const fn value(mut self, value: CheckboxType) -> Self {
        self.value = value;
        self
    }

    pub const fn preview_state(mut self, state: CheckboxState) -> Self {
        self.state = state;
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub const fn ghost(mut self, ghost: bool) -> Self {
        self.ghost = ghost;
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

impl RenderOnce for Checkbox {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let selected = self.value != CheckboxType::Unchecked;
        let foreground = if self.disabled || self.muted {
            SemanticColor::TextDisabled.resolve(cx)
        } else {
            SemanticColor::Text.resolve(cx)
        };
        let mark_color = if self.disabled {
            SemanticColor::IconOnBrand.resolve(cx)
        } else if self.ghost {
            SemanticColor::IconBrand.resolve(cx)
        } else if self.muted {
            SemanticColor::IconDisabled.resolve(cx)
        } else {
            SemanticColor::IconOnBrand.resolve(cx)
        };
        let box_background = if self.disabled {
            SemanticColor::BackgroundDisabled.resolve(cx)
        } else if selected && !self.ghost {
            SemanticColor::BackgroundBrand.resolve(cx)
        } else {
            SemanticColor::Background.resolve(cx)
        };
        let box_border = if self.state == CheckboxState::Focused {
            SemanticColor::BorderSelected.resolve(cx)
        } else if selected && !self.ghost {
            SemanticColor::BackgroundBrand.resolve(cx)
        } else if selected && self.ghost {
            SemanticColor::IconBrand.resolve(cx)
        } else if self.disabled {
            SemanticColor::BackgroundDisabled.resolve(cx)
        } else {
            SemanticColor::Border.resolve(cx)
        };
        let handler = self.on_activate;
        let disabled = self.disabled;

        h_flex()
            .id(self.id)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(if disabled { -1 } else { 0 })
            .h(px(tokens::RowHeight::FIELD))
            .items_center()
            .gap_2()
            .rounded(px(tokens::Radius::CONTROL))
            .when(!disabled, |row| {
                row.cursor_pointer()
                    .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
            })
            .child(
                div()
                    .size(px(tokens::ControlSize::INLINE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(tokens::Radius::CONTROL))
                    .border_1()
                    .border_color(box_border)
                    .bg(box_background)
                    .when(self.value == CheckboxType::Checked, |box_| {
                        box_.child(render_lucide_icon(LucideIcon::Check, mark_color, 12.))
                    })
                    .when(self.value == CheckboxType::Mixed, |box_| {
                        box_.child(render_lucide_icon(LucideIcon::Minus, mark_color, 12.))
                    }),
            )
            .child(
                div()
                    .typography(TypographyToken::BodyMedium)
                    .font_medium()
                    .text_color(foreground)
                    .child(self.label),
            )
            .when_some(handler.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_exposes_the_figma_value_axis() {
        assert_eq!(CheckboxType::ALL.len(), 3);
        assert_eq!(CheckboxType::Checked.label(), "Checked");
        assert_eq!(CheckboxType::Unchecked.label(), "Unchecked");
        assert_eq!(CheckboxType::Mixed.label(), "Mixed");
    }
}
