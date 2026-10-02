//! Zed content navigation shared by inspector panels and their pickers.

use std::rc::Rc;

use gpui::{
    AnyElement, App, FocusHandle, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement as _, Pixels, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::h_flex;
use ui::{LabelCommon as _, TabPosition, Toggleable as _};

use super::{
    CONTROL_KEY_CONTEXT, ControlExt as _, SemanticColor, TabSelection, TypographyExt as _,
    TypographyToken, tokens,
};

type ChangeHandler = Rc<dyn Fn(&TabSelection, &mut Window, &mut App)>;

struct NavigationState {
    focus: Vec<FocusHandle>,
    scroll: ScrollHandle,
    selected: Option<usize>,
    measurement: Option<(Pixels, Pixels)>,
}

/// Content tabs keep canonical Zed geometry when its full theme is installed.
/// Hosts with only gpui-component's theme receive the same rectangular shape.
#[derive(IntoElement)]
pub(crate) struct InspectorTabs {
    id: SharedString,
    items: Vec<(SharedString, SharedString)>,
    selected: usize,
    disabled: bool,
    end_children: Vec<AnyElement>,
    on_change: Option<ChangeHandler>,
}

impl InspectorTabs {
    pub(crate) fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = (SharedString, SharedString)>,
    ) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            selected: 0,
            disabled: false,
            end_children: Vec::new(),
            on_change: None,
        }
    }

    pub(crate) fn selected_index(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn end_child(mut self, child: impl IntoElement) -> Self {
        self.end_children.push(child.into_any_element());
        self
    }

    pub(crate) fn on_change(
        mut self,
        handler: impl Fn(&TabSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for InspectorTabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.items.len();
        let selected = self.selected.min(count.saturating_sub(1));
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| NavigationState {
            focus: (0..count).map(|_| cx.focus_handle()).collect(),
            scroll: ScrollHandle::new(),
            selected: None,
            measurement: None,
        });
        let (focus, scroll) = state.update(cx, |state, cx| {
            while state.focus.len() < count {
                state.focus.push(cx.focus_handle());
            }
            if state.selected != Some(selected) {
                state.selected = Some(selected);
                state.scroll.scroll_to_item(selected);
            }
            (state.focus.clone(), state.scroll.clone())
        });
        let canonical = cx.try_global::<theme::GlobalTheme>().is_some()
            && theme::try_theme_settings(cx).is_some();
        let handler = self.on_change.filter(|_| !self.disabled);
        let tabs = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, (id, label))| {
                let active = index == selected;
                let position = if index == 0 {
                    TabPosition::First
                } else if index + 1 == count {
                    TabPosition::Last
                } else {
                    TabPosition::Middle(index.cmp(&selected))
                };
                let selector = id.to_string();
                let label_selector = format!("{id}-label");
                let focus = focus[index].clone();
                let activation_focus = focus.clone();
                let activation_scroll = scroll.clone();
                let handler = handler.clone();
                let activation =
                    move |event: &super::ActivateEvent, window: &mut Window, cx: &mut App| {
                        activation_focus.focus(window, cx);
                        activation_scroll.scroll_to_item(index);
                        if let Some(handler) = &handler {
                            handler(
                                &TabSelection {
                                    index,
                                    keyboard: event.keyboard,
                                },
                                window,
                                cx,
                            );
                        }
                    };
                if canonical {
                    return div()
                        .debug_selector(move || selector)
                        .flex_none()
                        .opacity(if self.disabled { 0.5 } else { 1. })
                        .child(
                            ui::Tab::new(id)
                                .position(position)
                                .toggle_state(active)
                                .key_context(CONTROL_KEY_CONTEXT)
                                .tab_index(if self.disabled { -1 } else { 0 })
                                .track_focus(&focus)
                                .when(!self.disabled, |tab| tab.on_activate(activation))
                                .child(
                                    div()
                                        .debug_selector(move || label_selector)
                                        .text_left()
                                        .child(ui::Label::new(label).single_line().color(
                                            if active {
                                                ui::Color::Default
                                            } else {
                                                ui::Color::Muted
                                            },
                                        )),
                                ),
                        )
                        .into_any_element();
                }
                let style = super::sidebar_style(cx);
                h_flex()
                    .id(id)
                    .debug_selector(move || selector)
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(if self.disabled { -1 } else { 0 })
                    .track_focus(&focus)
                    .flex_none()
                    .h(px(tokens::InspectorGeometry::SECTION_HEADER))
                    .px(px(tokens::Space::XS))
                    .gap(px(tokens::Space::XS))
                    .border_b_1()
                    .border_r_1()
                    .border_color(style.border)
                    .bg(if active {
                        style.background
                    } else {
                        SemanticColor::BackgroundPanelField.resolve(cx)
                    })
                    .text_color(if active { style.text } else { style.muted_text })
                    .opacity(if self.disabled { 0.5 } else { 1. })
                    .when(!self.disabled, |tab| {
                        tab.cursor_pointer()
                            .hover(|tab| tab.bg(style.hover))
                            .focus(|tab| tab.border_color(style.focused_border))
                            .on_activate(activation)
                    })
                    .child(div().w(px(tokens::IconSize::XS)).flex_none())
                    .child(
                        div()
                            .debug_selector(move || label_selector)
                            .typography(TypographyToken::Panel)
                            .whitespace_nowrap()
                            .text_left()
                            .child(label),
                    )
                    .child(div().w(px(tokens::IconSize::SM)).flex_none())
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let bar = if canonical {
            ui::TabBar::new(format!("{}-zed", self.id))
                .track_scroll(&scroll)
                .children(tabs)
                .end_children(self.end_children)
                .into_any_element()
        } else {
            h_flex()
                .w_full()
                .min_w_0()
                .h(px(tokens::InspectorGeometry::SECTION_HEADER))
                .child(
                    h_flex()
                        .id(format!("{}-scroll", self.id))
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .overflow_x_scroll()
                        .track_scroll(&scroll)
                        .children(tabs),
                )
                .when(!self.end_children.is_empty(), |bar| {
                    bar.child(
                        h_flex()
                            .flex_none()
                            .px(px(tokens::Space::SM))
                            .gap(px(tokens::Space::XS))
                            .border_b_1()
                            .border_l_1()
                            .border_color(SemanticColor::BorderPanel.resolve(cx))
                            .children(self.end_children),
                    )
                })
                .into_any_element()
        };
        let selector = self.id.to_string();
        let measurement_scroll = scroll.clone();
        div()
            .on_children_prepainted(move |bounds, _, app| {
                let Some(bar) = bounds.first() else {
                    return;
                };
                let changed = state.update(app, |state, _| {
                    let measurement = (bar.size.width, measurement_scroll.max_offset().x);
                    let changed = state.measurement != Some(measurement);
                    state.measurement = Some(measurement);
                    changed
                });
                if changed {
                    let state = state.clone();
                    app.defer(move |app| {
                        state.update(app, |state, cx| {
                            state.scroll.scroll_to_item(state.selected.unwrap_or(0));
                            cx.notify();
                        })
                    });
                }
            })
            .id(self.id)
            .debug_selector(move || selector)
            .w_full()
            .min_w_0()
            .flex_none()
            .when_some(handler.filter(|_| count > 0), |bar, handler| {
                bar.on_key_down(move |event: &KeyDownEvent, window: &mut Window, cx| {
                    if !focus.iter().any(|focus| focus.is_focused(window)) {
                        return;
                    }
                    let next = match event.keystroke.key.as_str() {
                        "left" | "up" => (selected + count - 1) % count,
                        "right" | "down" => (selected + 1) % count,
                        "home" => 0,
                        "end" => count - 1,
                        _ => return,
                    };
                    focus[next].focus(window, cx);
                    scroll.scroll_to_item(next);
                    handler(
                        &TabSelection {
                            index: next,
                            keyboard: true,
                        },
                        window,
                        cx,
                    );
                    window.prevent_default();
                    cx.stop_propagation();
                })
            })
            .child(bar)
    }
}
