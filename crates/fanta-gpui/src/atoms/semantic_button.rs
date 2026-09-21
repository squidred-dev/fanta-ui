//! Fanta button components following Figma UI3's button taxonomy.
//!
//! These components keep the Figma component axes while resolving every
//! visual through Fanta semantic colors and typography. `ui_button` remains
//! the compatibility constructor for code that needs a gpui-component
//! `Button`; new component work should use the semantic components below.

use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Sizable as _, Size, StyledExt as _,
    button::{Button, ButtonVariant, ButtonVariants as _},
    h_flex,
};

use super::{
    ActivateEvent, ControlExt as _, LucideIcon, SemanticColor, TypographyExt as _, TypographyToken,
    render_lucide_icon, tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;

/// The ten visual intents in the UI3 text-button component set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticButtonVariant {
    Primary,
    #[default]
    Secondary,
    Destructive,
    SecondaryDestructive,
    Inverse,
    Success,
    Ghost,
    Link,
    LinkDanger,
}

impl SemanticButtonVariant {
    pub const ALL: [Self; 9] = [
        Self::Primary,
        Self::Secondary,
        Self::Destructive,
        Self::SecondaryDestructive,
        Self::Inverse,
        Self::Success,
        Self::Ghost,
        Self::Link,
        Self::LinkDanger,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Destructive => "Destructive",
            Self::SecondaryDestructive => "Secondary destruct",
            Self::Inverse => "Inverse",
            Self::Success => "Success",
            Self::Ghost => "Ghost",
            Self::Link => "Link",
            Self::LinkDanger => "Link danger",
        }
    }
}

/// UI3 text-button sizing: 24 px default, 32 px large, or a 24 px wide row.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticButtonSize {
    #[default]
    Default,
    Large,
    Wide,
}

impl SemanticButtonSize {
    pub const fn height(self) -> f32 {
        match self {
            Self::Default | Self::Wide => 24.,
            Self::Large => 32.,
        }
    }
}

/// Standard interaction states shared by text and icon buttons.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticButtonState {
    #[default]
    Default,
    Hover,
    Active,
    Focused,
    Disabled,
}

impl SemanticButtonState {
    pub const ALL: [Self; 5] = [
        Self::Default,
        Self::Hover,
        Self::Active,
        Self::Focused,
        Self::Disabled,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Hover => "Hover",
            Self::Active => "Active",
            Self::Focused => "Focused",
            Self::Disabled => "Disabled",
        }
    }
}

/// Leading-icon behavior exposed by the UI3 text-button set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticButtonIconAlignment {
    #[default]
    None,
    Left,
    Center,
}

/// A semantic text button with exact UI3 component axes.
#[derive(IntoElement)]
pub struct SemanticButton {
    id: ElementId,
    label: SharedString,
    variant: SemanticButtonVariant,
    size: SemanticButtonSize,
    state: SemanticButtonState,
    icon: Option<LucideIcon>,
    icon_alignment: SemanticButtonIconAlignment,
    on_activate: Option<ActivateHandler>,
}

impl SemanticButton {
    pub fn new(
        id: impl Into<ElementId>,
        variant: SemanticButtonVariant,
        size: SemanticButtonSize,
    ) -> Self {
        Self {
            id: id.into(),
            label: variant.label().into(),
            variant,
            size,
            state: SemanticButtonState::Default,
            icon: None,
            icon_alignment: SemanticButtonIconAlignment::None,
            on_activate: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub const fn icon(mut self, icon: LucideIcon, alignment: SemanticButtonIconAlignment) -> Self {
        self.icon = Some(icon);
        self.icon_alignment = alignment;
        self
    }

    /// Forces one state for documentation. Live controls should leave the
    /// default and receive hover, active, and focus styling from GPUI.
    pub const fn preview_state(mut self, state: SemanticButtonState) -> Self {
        self.state = state;
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.state = SemanticButtonState::Disabled;
        }
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

pub fn semantic_button(
    id: impl Into<ElementId>,
    variant: SemanticButtonVariant,
    size: SemanticButtonSize,
) -> SemanticButton {
    SemanticButton::new(id, variant, size)
}

#[derive(Clone, Copy)]
struct ButtonPalette {
    background: Option<SemanticColor>,
    hover: Option<SemanticColor>,
    active: Option<SemanticColor>,
    foreground: SemanticColor,
    border: Option<SemanticColor>,
    tinted_interaction: bool,
}

fn palette(variant: SemanticButtonVariant) -> ButtonPalette {
    match variant {
        SemanticButtonVariant::Primary => ButtonPalette {
            background: Some(SemanticColor::BackgroundBrand),
            hover: Some(SemanticColor::BackgroundBrandHover),
            active: Some(SemanticColor::BackgroundBrandActive),
            foreground: SemanticColor::TextOnBrand,
            border: None,
            tinted_interaction: false,
        },
        SemanticButtonVariant::Secondary => ButtonPalette {
            background: None,
            hover: Some(SemanticColor::BackgroundHover),
            active: Some(SemanticColor::BackgroundActive),
            foreground: SemanticColor::Text,
            border: Some(SemanticColor::Border),
            tinted_interaction: false,
        },
        SemanticButtonVariant::Destructive => ButtonPalette {
            background: Some(SemanticColor::BackgroundDanger),
            hover: Some(SemanticColor::BackgroundDangerHover),
            active: Some(SemanticColor::BackgroundDangerActive),
            foreground: SemanticColor::TextOnDanger,
            border: None,
            tinted_interaction: false,
        },
        SemanticButtonVariant::SecondaryDestructive => ButtonPalette {
            background: None,
            hover: Some(SemanticColor::BackgroundDangerHover),
            active: Some(SemanticColor::BackgroundDangerActive),
            foreground: SemanticColor::TextDanger,
            border: Some(SemanticColor::IconDanger),
            tinted_interaction: true,
        },
        SemanticButtonVariant::Inverse => ButtonPalette {
            background: Some(SemanticColor::BackgroundToolbar),
            hover: Some(SemanticColor::BackgroundToolbarHover),
            active: Some(SemanticColor::BackgroundToolbarSelected),
            foreground: SemanticColor::IconOnDarkCanvas,
            border: None,
            tinted_interaction: false,
        },
        SemanticButtonVariant::Success => ButtonPalette {
            background: Some(SemanticColor::BackgroundSuccess),
            hover: Some(SemanticColor::BackgroundSuccessHover),
            active: Some(SemanticColor::BackgroundSuccessActive),
            foreground: SemanticColor::TextOnSuccess,
            border: None,
            tinted_interaction: false,
        },
        SemanticButtonVariant::Ghost => ButtonPalette {
            background: None,
            hover: Some(SemanticColor::BackgroundHover),
            active: Some(SemanticColor::BackgroundActive),
            foreground: SemanticColor::Text,
            border: None,
            tinted_interaction: false,
        },
        SemanticButtonVariant::Link => ButtonPalette {
            background: None,
            hover: None,
            active: None,
            foreground: SemanticColor::TextBrand,
            border: None,
            tinted_interaction: false,
        },
        SemanticButtonVariant::LinkDanger => ButtonPalette {
            background: None,
            hover: None,
            active: None,
            foreground: SemanticColor::TextDanger,
            border: None,
            tinted_interaction: false,
        },
    }
}

impl RenderOnce for SemanticButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut colors = palette(self.variant);
        let disabled = self.state == SemanticButtonState::Disabled;
        if disabled {
            colors.background = Some(SemanticColor::BackgroundDisabled);
            colors.foreground = SemanticColor::TextDisabled;
            colors.border = None;
        }
        let background = match self.state {
            SemanticButtonState::Hover => colors.hover.or(colors.background),
            SemanticButtonState::Active => colors.active.or(colors.hover).or(colors.background),
            _ => colors.background,
        };
        let foreground = colors.foreground.resolve(cx);
        let resolve_interaction = |role: SemanticColor| {
            let color = role.resolve(cx);
            if colors.tinted_interaction {
                color.opacity(0.1)
            } else {
                color
            }
        };
        let hover_color = colors.hover.map(resolve_interaction);
        let active_color = colors.active.map(resolve_interaction);
        let focus_color = SemanticColor::BorderSelected.resolve(cx);
        let is_link = matches!(
            self.variant,
            SemanticButtonVariant::Link | SemanticButtonVariant::LinkDanger
        );
        let icon = self.icon;
        let centered_icon = self.icon_alignment == SemanticButtonIconAlignment::Center;
        let content = h_flex()
            .items_center()
            .justify_center()
            .gap_1()
            .when_some(icon, |content, icon| {
                content.child(render_lucide_icon(icon, foreground, 16.))
            })
            .when(!centered_icon, |content| content.child(self.label));
        let handler = self.on_activate;

        div()
            .id(self.id)
            .key_context(super::CONTROL_KEY_CONTEXT)
            .tab_index(if disabled { -1 } else { 0 })
            .h(px(self.size.height()))
            .when(self.size == SemanticButtonSize::Wide, |button| {
                button.w(px(tokens::ButtonGeometry::WIDE_WIDTH))
            })
            .when(self.size != SemanticButtonSize::Wide, |button| {
                button.px(px(if self.size == SemanticButtonSize::Large {
                    12.
                } else if icon.is_some() {
                    6.
                } else {
                    8.
                }))
            })
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(if self.state == SemanticButtonState::Focused {
                SemanticColor::BorderSelected.resolve(cx)
            } else {
                colors.border.map_or_else(
                    || SemanticColor::Background.resolve(cx).opacity(0.),
                    |role| role.resolve(cx),
                )
            })
            .when_some(background, |button, role| {
                button.bg(if colors.tinted_interaction {
                    role.resolve(cx).opacity(0.1)
                } else {
                    role.resolve(cx)
                })
            })
            .typography(TypographyToken::BodyMedium)
            .font_medium()
            .text_color(foreground)
            .when(is_link, |button| button.underline())
            .when(!disabled, |button| {
                button
                    .cursor_pointer()
                    .when_some(hover_color, |button, hover| {
                        button.hover(move |style| style.bg(hover))
                    })
                    .when_some(active_color, |button, active| {
                        button.active(move |style| style.bg(active))
                    })
                    .focus(move |style| style.border_color(focus_color))
            })
            .child(content)
            .when_some(handler.filter(|_| !disabled), |button, handler| {
                button.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// The four square/split icon-button component sets in the Figma source.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticIconButtonKind {
    #[default]
    Button,
    Toggle,
    DialogToggle,
    Split,
}

/// Split buttons distinguish which side is active or focused.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticSplitButtonState {
    #[default]
    Default,
    Hover,
    PrimaryActive,
    SecondaryActive,
    PrimaryFocused,
    SecondaryFocused,
    Disabled,
}

impl SemanticSplitButtonState {
    pub const ALL: [Self; 7] = [
        Self::Default,
        Self::Hover,
        Self::PrimaryActive,
        Self::SecondaryActive,
        Self::PrimaryFocused,
        Self::SecondaryFocused,
        Self::Disabled,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Hover => "Hover",
            Self::PrimaryActive => "Primary active",
            Self::SecondaryActive => "Secondary active",
            Self::PrimaryFocused => "Primary focus",
            Self::SecondaryFocused => "Secondary focus",
            Self::Disabled => "Disabled",
        }
    }
}

#[derive(IntoElement)]
pub struct SemanticIconButton {
    id: ElementId,
    icon: LucideIcon,
    kind: SemanticIconButtonKind,
    state: SemanticButtonState,
    split_state: SemanticSplitButtonState,
    secondary: bool,
    highlighted: bool,
    on: bool,
    large: bool,
    on_activate: Option<ActivateHandler>,
}

impl SemanticIconButton {
    pub fn new(id: impl Into<ElementId>, icon: LucideIcon, kind: SemanticIconButtonKind) -> Self {
        Self {
            id: id.into(),
            icon,
            kind,
            state: SemanticButtonState::Default,
            split_state: SemanticSplitButtonState::Default,
            secondary: false,
            highlighted: false,
            on: false,
            large: false,
            on_activate: None,
        }
    }

    pub const fn preview_state(mut self, state: SemanticButtonState) -> Self {
        self.state = state;
        self
    }

    pub const fn split_state(mut self, state: SemanticSplitButtonState) -> Self {
        self.split_state = state;
        self
    }

    pub const fn secondary(mut self, secondary: bool) -> Self {
        self.secondary = secondary;
        self
    }

    pub const fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    pub const fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub const fn large(mut self, large: bool) -> Self {
        self.large = large;
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

pub fn semantic_icon_button(
    id: impl Into<ElementId>,
    icon: LucideIcon,
    kind: SemanticIconButtonKind,
) -> SemanticIconButton {
    SemanticIconButton::new(id, icon, kind)
}

impl RenderOnce for SemanticIconButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let split = self.kind == SemanticIconButtonKind::Split;
        let disabled = self.state == SemanticButtonState::Disabled
            || self.split_state == SemanticSplitButtonState::Disabled;
        let selected = self.on || self.highlighted;
        let background = if disabled {
            Some(SemanticColor::BackgroundDisabled)
        } else if selected {
            Some(SemanticColor::BackgroundSelected)
        } else if matches!(
            self.state,
            SemanticButtonState::Hover | SemanticButtonState::Active
        ) || matches!(
            self.split_state,
            SemanticSplitButtonState::Hover
                | SemanticSplitButtonState::PrimaryActive
                | SemanticSplitButtonState::SecondaryActive
        ) {
            Some(SemanticColor::BackgroundHover)
        } else {
            None
        };
        let foreground = if disabled {
            SemanticColor::IconDisabled.resolve(cx)
        } else if selected {
            SemanticColor::IconBrand.resolve(cx)
        } else {
            SemanticColor::Icon.resolve(cx)
        };
        let focused = self.state == SemanticButtonState::Focused;
        let primary_focused = self.split_state == SemanticSplitButtonState::PrimaryFocused;
        let secondary_focused = self.split_state == SemanticSplitButtonState::SecondaryFocused;
        let height = if self.large { 32. } else { 24. };
        let handler = self.on_activate;

        let primary = div()
            .h_full()
            .w(px(if split {
                if self.large { 32. } else { 28. }
            } else {
                height
            }))
            .flex()
            .items_center()
            .justify_center()
            .when(primary_focused, |part| {
                part.border_1()
                    .border_color(SemanticColor::BorderSelected.resolve(cx))
            })
            .child(render_lucide_icon(self.icon, foreground, 16.));
        let secondary = div()
            .h_full()
            .w(px(if self.large { 16. } else { 12. }))
            .flex()
            .items_center()
            .justify_center()
            .when(secondary_focused, |part| {
                part.border_1()
                    .border_color(SemanticColor::BorderSelected.resolve(cx))
            })
            .child(render_lucide_icon(LucideIcon::ChevronDown, foreground, 12.));

        h_flex()
            .id(self.id)
            .key_context(super::CONTROL_KEY_CONTEXT)
            .tab_index(if disabled { -1 } else { 0 })
            .h(px(height))
            .when(!split, |button| button.w(px(height)))
            .when(split, |button| {
                button
                    .w(px(if self.large {
                        tokens::ButtonGeometry::SPLIT_LARGE_WIDTH
                    } else {
                        tokens::ButtonGeometry::SPLIT_SMALL_WIDTH
                    }))
                    .gap(px(tokens::ButtonGeometry::SPLIT_GAP))
            })
            .items_center()
            .justify_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(if focused {
                SemanticColor::BorderSelected.resolve(cx)
            } else if self.secondary {
                SemanticColor::Border.resolve(cx)
            } else {
                SemanticColor::Background.resolve(cx).opacity(0.)
            })
            .when_some(background, |button, role| button.bg(role.resolve(cx)))
            .when(!disabled, |button| {
                button
                    .cursor_pointer()
                    .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
                    .active(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
                    .focus(|style| style.border_color(SemanticColor::BorderSelected.resolve(cx)))
            })
            .child(primary)
            .when(split, |button| button.child(secondary))
            .when_some(handler.filter(|_| !disabled), |button, handler| {
                button.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// Compatibility constructor for existing call sites that compose directly
/// with gpui-component Button methods. Its size matches UI3 default (24 px).
pub fn ui_button(id: impl Into<ElementId>) -> Button {
    Button::new(id)
        .with_size(Size::Small)
        .with_variant(ButtonVariant::Secondary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_taxonomy_matches_the_five_figma_component_sets() {
        assert_eq!(SemanticButtonVariant::ALL.len(), 9);
        assert_eq!(SemanticButtonState::ALL.len(), 5);
        assert_eq!(SemanticSplitButtonState::ALL.len(), 7);
        assert_eq!(SemanticButtonSize::Default.height(), 24.);
        assert_eq!(SemanticButtonSize::Large.height(), 32.);
        assert_eq!(SemanticButtonSize::Wide.height(), 24.);
    }
}
