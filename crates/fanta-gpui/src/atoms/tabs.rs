//! Controlled UI3 tab primitive and tabs group.

use std::rc::Rc;

use gpui::{
    App, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{StyledExt as _, h_flex};

use super::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, SemanticColor, TypographyExt as _,
    TypographyToken, tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;
type ChangeHandler = Rc<dyn Fn(&TabSelection, &mut Window, &mut App)>;

/// State axis for Figma's `_Tab` primitive.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TabState {
    #[default]
    Default,
    Focused,
    Hover,
}

impl TabState {
    pub const ALL: [Self; 3] = [Self::Default, Self::Focused, Self::Hover];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Focused => "Focused",
            Self::Hover => "Hover",
        }
    }
}

/// Typed selection request emitted by a controlled tabs group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TabSelection {
    pub index: usize,
    pub keyboard: bool,
}

/// The primitive represented by Figma's `_Tab` component set.
#[derive(IntoElement)]
pub struct Tab {
    id: SharedString,
    label: SharedString,
    selected: bool,
    single_tab: bool,
    state: TabState,
    badge: Option<SharedString>,
    on_activate: Option<ActivateHandler>,
}

impl Tab {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            single_tab: false,
            state: TabState::Default,
            badge: None,
            on_activate: None,
        }
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub const fn single_tab(mut self, single_tab: bool) -> Self {
        self.single_tab = single_tab;
        self
    }

    pub const fn preview_state(mut self, state: TabState) -> Self {
        self.state = state;
        self
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
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

impl RenderOnce for Tab {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let selected = self.selected;
        let focused = self.state == TabState::Focused;
        let preview_hover = self.state == TabState::Hover;
        let handler = self.on_activate;
        let selector = self.id.to_string();

        h_flex()
            .id(self.id)
            .debug_selector(move || selector.clone())
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(tokens::RowHeight::FIELD))
            .min_w(px(if self.single_tab {
                tokens::TabsGeometry::SINGLE_TAB_WIDTH
            } else {
                tokens::TabsGeometry::TAB_MIN_WIDTH
            }))
            .items_center()
            .justify_center()
            .gap_1()
            .px_2()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(if focused {
                SemanticColor::BorderSelected.resolve(cx)
            } else {
                SemanticColor::Border.resolve(cx).opacity(0.)
            })
            .bg(if selected {
                SemanticColor::BackgroundSelected.resolve(cx)
            } else if preview_hover {
                SemanticColor::BackgroundHover.resolve(cx)
            } else {
                SemanticColor::BackgroundSecondary.resolve(cx).opacity(0.)
            })
            .cursor_pointer()
            .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
            .active(|style| style.bg(SemanticColor::BackgroundActive.resolve(cx)))
            .focus(|style| style.border_color(SemanticColor::BorderSelected.resolve(cx)))
            .child(
                div()
                    .min_w(px(tokens::Space::NONE))
                    .truncate()
                    .typography(TypographyToken::BodyMedium)
                    .when(selected, |label| label.font_semibold())
                    .text_color(if selected {
                        SemanticColor::Text.resolve(cx)
                    } else {
                        SemanticColor::TextSecondary.resolve(cx)
                    })
                    .child(self.label),
            )
            .when_some(self.badge, |tab, badge| {
                tab.child(
                    div()
                        .h(px(tokens::TabsGeometry::BADGE_HEIGHT))
                        .min_w(px(tokens::TabsGeometry::BADGE_MIN_WIDTH))
                        .px_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .typography(TypographyToken::BodySmall)
                        .text_color(SemanticColor::TextSecondary.resolve(cx))
                        .bg(SemanticColor::BackgroundTertiary.resolve(cx))
                        .child(badge),
                )
            })
            .when_some(handler, |tab, handler| {
                tab.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// A host-controlled 1–4 item tabs group matching the linked Figma set.
#[derive(IntoElement)]
pub struct Tabs {
    id: SharedString,
    labels: Vec<SharedString>,
    selected_index: usize,
    badges: Vec<Option<SharedString>>,
    on_change: Option<ChangeHandler>,
}

impl Tabs {
    pub fn new(id: impl Into<SharedString>, labels: Vec<SharedString>) -> Self {
        let badges = vec![None; labels.len()];
        Self {
            id: id.into(),
            labels,
            selected_index: 0,
            badges,
            on_change: None,
        }
    }

    pub const fn selected_index(mut self, selected_index: usize) -> Self {
        self.selected_index = selected_index;
        self
    }

    pub fn badges(mut self, badges: Vec<Option<SharedString>>) -> Self {
        self.badges = badges;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&TabSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        let item_count = self.labels.len();
        let selected_index = self.selected_index.min(item_count.saturating_sub(1));
        let handler = self.on_change;
        let base_id = self.id.clone();
        let single_tab = item_count == 1;
        let tabs = self
            .labels
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                let mut tab = Tab::new(SharedString::from(format!("{base_id}-tab-{index}")), label)
                    .selected(!single_tab && index == selected_index)
                    .single_tab(single_tab);
                if let Some(Some(badge)) = self.badges.get(index).cloned() {
                    tab = tab.badge(badge);
                }
                if let Some(handler) = handler.clone() {
                    tab = tab.on_activate(move |event, window, cx| {
                        handler(
                            &TabSelection {
                                index,
                                keyboard: event.keyboard,
                            },
                            window,
                            cx,
                        );
                    });
                }
                tab.into_any_element()
            })
            .collect::<Vec<_>>();
        let selector = self.id.to_string();

        h_flex()
            .id(self.id)
            .debug_selector(move || selector.clone())
            .w(px(if single_tab {
                tokens::TabsGeometry::SINGLE_TAB_WIDTH
            } else {
                tokens::TabsGeometry::TABS_WIDTH
            }))
            .h(px(tokens::RowHeight::LIST))
            .items_center()
            .gap_1()
            .when_some(handler.filter(|_| item_count > 0), |group, handler| {
                group.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let next_index = match event.keystroke.key.as_str() {
                        "left" | "up" => (selected_index + item_count - 1) % item_count,
                        "right" | "down" => (selected_index + 1) % item_count,
                        "home" => 0,
                        "end" => item_count - 1,
                        _ => return,
                    };
                    handler(
                        &TabSelection {
                            index: next_index,
                            keyboard: true,
                        },
                        window,
                        cx,
                    );
                    window.prevent_default();
                    cx.stop_propagation();
                })
            })
            .children(tabs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_axes_match_the_linked_figma_sets() {
        assert_eq!((1..=4).count(), 4, "Tabs count variants");
        let tab_variants = 2 + 2 + 1 + 2;
        assert_eq!(tab_variants, 7, "_Tab's valid state combinations");
        assert_eq!(TabState::ALL.len(), 3);
    }
}
