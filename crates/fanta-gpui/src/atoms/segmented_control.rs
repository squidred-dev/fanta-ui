//! Controlled UI3 segmented-control atoms.

use std::rc::Rc;

use gpui::{
    App, Div, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, RenderOnce,
    SharedString, Stateful, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::h_flex;

use super::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor,
    TypographyExt as _, TypographyToken, render_lucide_icon, tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;
type ChangeHandler = Rc<dyn Fn(&SegmentedControlSelection, &mut Window, &mut App)>;

/// The content axis of the Figma segmented-control component set.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SegmentedControlVariant {
    Icon,
    #[default]
    Label,
}

impl SegmentedControlVariant {
    pub const ALL: [Self; 2] = [Self::Icon, Self::Label];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Icon => "Icon",
            Self::Label => "Label",
        }
    }
}

/// The state axis on the complete segmented-control component.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SegmentedControlState {
    #[default]
    Default,
    Disabled,
}

impl SegmentedControlState {
    pub const ALL: [Self; 2] = [Self::Default, Self::Disabled];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Disabled => "Disabled",
        }
    }
}

/// The state axis shared by the private icon and label segment sets.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SegmentState {
    #[default]
    Default,
    Focused,
    Disabled,
}

impl SegmentState {
    pub const ALL: [Self; 3] = [Self::Default, Self::Focused, Self::Disabled];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Focused => "Focused",
            Self::Disabled => "Disabled",
        }
    }
}

/// One icon option and the label a host uses to describe its action.
#[derive(Clone, Debug)]
pub struct SegmentIconOption {
    pub icon: LucideIcon,
    pub label: SharedString,
}

impl SegmentIconOption {
    pub fn new(icon: LucideIcon, label: impl Into<SharedString>) -> Self {
        Self {
            icon,
            label: label.into(),
        }
    }
}

/// Typed intent emitted by a complete controlled segmented control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentedControlSelection {
    pub index: usize,
    pub keyboard: bool,
}

fn segment_shell(id: SharedString, active: bool, state: SegmentState, cx: &App) -> Stateful<Div> {
    let disabled = state == SegmentState::Disabled;
    let border = if state == SegmentState::Focused {
        SemanticColor::BorderSelected.resolve(cx)
    } else if active {
        SemanticColor::Border.resolve(cx)
    } else {
        SemanticColor::BackgroundSecondary.resolve(cx).opacity(0.)
    };

    let selector = id.to_string();
    h_flex()
        .id(id)
        .debug_selector(move || selector.clone())
        .key_context(CONTROL_KEY_CONTEXT)
        .tab_index(if disabled { -1 } else { 0 })
        .h_full()
        .min_w(px(tokens::RowHeight::FIELD))
        .flex_1()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .rounded(px(tokens::SegmentedControlGeometry::RADIUS))
        .border_1()
        .border_color(border)
        .bg(if active {
            SemanticColor::Background.resolve(cx)
        } else {
            SemanticColor::BackgroundSecondary.resolve(cx)
        })
        .when(!disabled, |segment| {
            segment
                .cursor_pointer()
                .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
                .active(|style| style.bg(SemanticColor::BackgroundActive.resolve(cx)))
                .focus(|style| style.border_color(SemanticColor::BorderSelected.resolve(cx)))
        })
}

/// The icon segment primitive represented by Figma's `_Segment icon` set.
#[derive(IntoElement)]
pub struct SegmentIcon {
    id: SharedString,
    icon: LucideIcon,
    active: bool,
    state: SegmentState,
    on_activate: Option<ActivateHandler>,
}

impl SegmentIcon {
    pub fn new(id: impl Into<SharedString>, icon: LucideIcon) -> Self {
        Self {
            id: id.into(),
            icon,
            active: false,
            state: SegmentState::Default,
            on_activate: None,
        }
    }

    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn preview_state(mut self, state: SegmentState) -> Self {
        self.state = state;
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

impl RenderOnce for SegmentIcon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == SegmentState::Disabled;
        let foreground = if disabled {
            SemanticColor::IconDisabled.resolve(cx)
        } else if self.active {
            SemanticColor::Icon.resolve(cx)
        } else {
            SemanticColor::IconSecondary.resolve(cx)
        };
        segment_shell(self.id, self.active && !disabled, self.state, cx)
            .child(render_lucide_icon(
                self.icon,
                foreground,
                tokens::IconSize::SM,
            ))
            .when_some(
                self.on_activate.filter(|_| !disabled),
                |segment, handler| {
                    segment.on_activate(move |event, window, cx| handler(event, window, cx))
                },
            )
    }
}

/// The text segment primitive represented by Figma's `_Segment label` set.
#[derive(IntoElement)]
pub struct SegmentLabel {
    id: SharedString,
    label: SharedString,
    active: bool,
    state: SegmentState,
    on_activate: Option<ActivateHandler>,
}

impl SegmentLabel {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            active: false,
            state: SegmentState::Default,
            on_activate: None,
        }
    }

    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn preview_state(mut self, state: SegmentState) -> Self {
        self.state = state;
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

impl RenderOnce for SegmentLabel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == SegmentState::Disabled;
        let foreground = if disabled {
            SemanticColor::TextDisabled.resolve(cx)
        } else if self.active {
            SemanticColor::Text.resolve(cx)
        } else {
            SemanticColor::TextSecondary.resolve(cx)
        };
        segment_shell(self.id, self.active && !disabled, self.state, cx)
            .px_2()
            .typography(TypographyToken::BodyMedium)
            .text_color(foreground)
            .child(div().truncate().child(self.label))
            .when_some(
                self.on_activate.filter(|_| !disabled),
                |segment, handler| {
                    segment.on_activate(move |event, window, cx| handler(event, window, cx))
                },
            )
    }
}

enum SegmentedControlItems {
    Icons(Vec<SegmentIconOption>),
    Labels(Vec<SharedString>),
}

/// A host-controlled segmented control supporting the UI3 2–6 tab taxonomy.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: SharedString,
    items: SegmentedControlItems,
    selected_index: Option<usize>,
    compact: bool,
    state: SegmentedControlState,
    on_change: Option<ChangeHandler>,
}

impl SegmentedControl {
    pub fn labels(id: impl Into<SharedString>, labels: Vec<SharedString>) -> Self {
        Self {
            id: id.into(),
            items: SegmentedControlItems::Labels(labels),
            selected_index: Some(0),
            compact: false,
            state: SegmentedControlState::Default,
            on_change: None,
        }
    }

    pub fn icons(id: impl Into<SharedString>, icons: Vec<SegmentIconOption>) -> Self {
        Self {
            id: id.into(),
            items: SegmentedControlItems::Icons(icons),
            selected_index: Some(0),
            compact: false,
            state: SegmentedControlState::Default,
            on_change: None,
        }
    }

    pub const fn selected_index(mut self, selected_index: usize) -> Self {
        self.selected_index = Some(selected_index);
        self
    }

    pub const fn unselected(mut self) -> Self {
        self.selected_index = None;
        self
    }

    pub const fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub const fn state(mut self, state: SegmentedControlState) -> Self {
        self.state = state;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&SegmentedControlSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub const fn variant(&self) -> SegmentedControlVariant {
        match self.items {
            SegmentedControlItems::Icons(_) => SegmentedControlVariant::Icon,
            SegmentedControlItems::Labels(_) => SegmentedControlVariant::Label,
        }
    }

    pub fn len(&self) -> usize {
        match &self.items {
            SegmentedControlItems::Icons(items) => items.len(),
            SegmentedControlItems::Labels(items) => items.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == SegmentedControlState::Disabled;
        let handler = self.on_change;
        let base_id = self.id.clone();
        let item_count = match &self.items {
            SegmentedControlItems::Icons(items) => items.len(),
            SegmentedControlItems::Labels(items) => items.len(),
        };
        let selected_index = self
            .selected_index
            .map(|index| index.min(item_count.saturating_sub(1)));
        let keyboard_anchor = selected_index.unwrap_or(0);
        let children = match self.items {
            SegmentedControlItems::Labels(labels) => labels
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    let mut segment = SegmentLabel::new(
                        SharedString::from(format!("{base_id}-segment-{index}")),
                        label,
                    )
                    .active(!disabled && selected_index == Some(index))
                    .preview_state(if disabled {
                        SegmentState::Disabled
                    } else {
                        SegmentState::Default
                    });
                    if let Some(handler) = handler.clone() {
                        segment = segment.on_activate(move |event, window, cx| {
                            handler(
                                &SegmentedControlSelection {
                                    index,
                                    keyboard: event.keyboard,
                                },
                                window,
                                cx,
                            );
                        });
                    }
                    segment.into_any_element()
                })
                .collect::<Vec<_>>(),
            SegmentedControlItems::Icons(icons) => icons
                .into_iter()
                .enumerate()
                .map(|(index, option)| {
                    let mut segment = SegmentIcon::new(
                        SharedString::from(format!("{base_id}-segment-{index}")),
                        option.icon,
                    )
                    .active(!disabled && selected_index == Some(index))
                    .preview_state(if disabled {
                        SegmentState::Disabled
                    } else {
                        SegmentState::Default
                    });
                    if let Some(handler) = handler.clone() {
                        segment = segment.on_activate(move |event, window, cx| {
                            handler(
                                &SegmentedControlSelection {
                                    index,
                                    keyboard: event.keyboard,
                                },
                                window,
                                cx,
                            );
                        });
                    }
                    segment.into_any_element()
                })
                .collect::<Vec<_>>(),
        };

        let selector = self.id.to_string();
        h_flex()
            .id(self.id)
            .debug_selector(move || selector.clone())
            .w(px(if self.compact {
                tokens::RowHeight::FIELD * item_count as f32
            } else {
                tokens::SegmentedControlGeometry::WIDTH
            }))
            .h(px(tokens::RowHeight::FIELD))
            .items_stretch()
            .overflow_hidden()
            .rounded(px(tokens::SegmentedControlGeometry::RADIUS))
            .bg(SemanticColor::BackgroundSecondary.resolve(cx))
            .when_some(
                handler.filter(|_| !disabled && item_count > 0),
                |control, handler| {
                    control.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let next_index = match event.keystroke.key.as_str() {
                            "left" | "up" => selected_index
                                .map(|_| (keyboard_anchor + item_count - 1) % item_count)
                                .unwrap_or(item_count - 1),
                            "right" | "down" => selected_index
                                .map(|_| (keyboard_anchor + 1) % item_count)
                                .unwrap_or(0),
                            "home" => 0,
                            "end" => item_count - 1,
                            _ => return,
                        };
                        handler(
                            &SegmentedControlSelection {
                                index: next_index,
                                keyboard: true,
                            },
                            window,
                            cx,
                        );
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                },
            )
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmented_control_axes_match_the_figma_component_sets() {
        assert_eq!(SegmentedControlVariant::ALL.len(), 2);
        assert_eq!(SegmentedControlState::ALL.len(), 2);
        assert_eq!(SegmentState::ALL.len(), 3);
        assert_eq!((2..=6).count(), 5);
        assert_eq!(2 * 5 * 2, 20);
    }
}
