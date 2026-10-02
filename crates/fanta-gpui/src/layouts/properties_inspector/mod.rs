//! Right sidebar composition. Child organisms keep their original intent streams.
use crate::atoms::{
    CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _,
    TypographyToken, icon_button, render_lucide_icon, tokens,
};
use crate::molecules::{ZoomControls, ZoomControlsAction};
use gpui::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, tooltip::Tooltip, v_flex};
use ui::{LabelCommon as _, Tab as ZedTab, TabBar, TabPosition, Toggleable as _};

pub const PROPERTIES_INSPECTOR_MIN_WIDTH: f32 =
    tokens::InputGeometry::TEXT_WIDTH + 8. * tokens::Space::LG;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PropertiesInspectorTab {
    #[default]
    Design,
    Motion,
    Draw,
    Code,
    Prototype,
    Comments,
}
impl PropertiesInspectorTab {
    pub const ALL: [Self; 6] = [
        Self::Design,
        Self::Motion,
        Self::Draw,
        Self::Code,
        Self::Prototype,
        Self::Comments,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Motion => "Motion",
            Self::Draw => "Draw",
            Self::Code => "Code",
            Self::Prototype => "Prototype",
            Self::Comments => "Comments",
        }
    }
    pub const fn id(self) -> &'static str {
        match self {
            Self::Design => "design",
            Self::Motion => "motion",
            Self::Draw => "draw",
            Self::Code => "code",
            Self::Prototype => "prototype",
            Self::Comments => "comments",
        }
    }
}
/// Dedicated child entities; the layout neither interprets nor forwards their edits.
#[derive(Clone)]
pub struct PropertiesInspectorChildren {
    pub design: AnyView,
    pub motion: AnyView,
    pub draw: AnyView,
    pub code: AnyView,
    pub prototype: AnyView,
    pub comments: AnyView,
}
impl PropertiesInspectorChildren {
    fn view(&self, tab: PropertiesInspectorTab) -> AnyView {
        match tab {
            PropertiesInspectorTab::Design => &self.design,
            PropertiesInspectorTab::Motion => &self.motion,
            PropertiesInspectorTab::Draw => &self.draw,
            PropertiesInspectorTab::Code => &self.code,
            PropertiesInspectorTab::Prototype => &self.prototype,
            PropertiesInspectorTab::Comments => &self.comments,
        }
        .clone()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertiesInspectorAction {
    TabChanged {
        tab: PropertiesInspectorTab,
    },
    CollapsedChanged {
        collapsed: bool,
    },
    Zoom(ZoomControlsAction),
    /// The host starts presentation of its accepted prototype state.
    PresentRequested,
}
pub struct PropertiesInspector {
    id: SharedString,
    focus: FocusHandle,
    tab_focus: [FocusHandle; 6],
    tabs_scroll: ScrollHandle,
    tabs_measurement: Option<(Pixels, Pixels)>,
    children: PropertiesInspectorChildren,
    zoom: Entity<ZoomControls>,
    active_tab: PropertiesInspectorTab,
    collapsed: bool,
    can_present: bool,
    _subscription: Subscription,
}
impl EventEmitter<PropertiesInspectorAction> for PropertiesInspector {}
impl Focusable for PropertiesInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl PropertiesInspector {
    pub fn new(
        id: impl Into<SharedString>,
        children: PropertiesInspectorChildren,
        percent: u16,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        let zoom = cx.new(|cx| ZoomControls::new(format!("{id}-zoom"), percent, cx));
        let subscription = cx.subscribe(&zoom, |_, _, event: &ZoomControlsAction, cx| {
            cx.emit(PropertiesInspectorAction::Zoom(*event))
        });
        Self {
            id,
            focus: cx.focus_handle(),
            tab_focus: std::array::from_fn(|_| cx.focus_handle()),
            tabs_scroll: ScrollHandle::new(),
            tabs_measurement: None,
            children,
            zoom,
            active_tab: PropertiesInspectorTab::Design,
            collapsed: false,
            can_present: false,
            _subscription: subscription,
        }
    }
    pub fn active_tab(&self) -> PropertiesInspectorTab {
        self.active_tab
    }
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }
    pub fn set_active_tab(&mut self, tab: PropertiesInspectorTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        if let Some(index) = PropertiesInspectorTab::ALL
            .iter()
            .position(|candidate| *candidate == tab)
        {
            self.tabs_scroll.scroll_to_item(index);
        }
        cx.notify();
    }
    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.collapsed = collapsed;
        cx.notify();
    }
    /// Presentation availability belongs to the prototype host. The play
    /// control is rendered only while the Prototype tab is active.
    pub fn set_can_present(&mut self, can_present: bool, cx: &mut Context<Self>) {
        if self.can_present != can_present {
            self.can_present = can_present;
            cx.notify();
        }
    }
    pub fn set_zoom(&mut self, percent: u16, cx: &mut Context<Self>) {
        self.zoom
            .update(cx, |zoom, cx| zoom.set_percent(percent, cx));
    }
    fn header(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .debug_selector(|| "properties-inspector-header".to_owned())
            .w_full()
            .min_w_0()
            .flex_none()
            .h(px(tokens::InspectorGeometry::SECTION_HEADER))
            .px_2()
            .gap_1()
            .child(self.zoom.clone())
            .child(div().flex_1())
            .when(
                self.active_tab == PropertiesInspectorTab::Prototype,
                |header| {
                    header.child(
                        icon_button(
                            format!("{}-present", self.id),
                            px(tokens::ControlSize::CHROME),
                            px(tokens::Radius::CONTROL),
                            cx,
                        )
                        .debug_selector(|| "properties-inspector-present".to_owned())
                        .tab_index(if self.can_present { 0 } else { -1 })
                        .opacity(if self.can_present { 1. } else { 0.45 })
                        .tooltip(|window, cx| Tooltip::new("Play prototype").build(window, cx))
                        .on_activate(cx.listener(|this, _, _, cx| {
                            if this.can_present
                                && this.active_tab == PropertiesInspectorTab::Prototype
                            {
                                cx.emit(PropertiesInspectorAction::PresentRequested);
                            }
                        }))
                        .child(render_lucide_icon(
                            LucideIcon::Play,
                            crate::atoms::sidebar_style(cx).icon,
                            tokens::IconSize::MD,
                        )),
                    )
                },
            )
            .child(
                icon_button(
                    format!("{}-toggle", self.id),
                    px(tokens::ControlSize::CHROME),
                    px(tokens::Radius::CONTROL),
                    cx,
                )
                .debug_selector(|| "properties-inspector-toggle".to_owned())
                .tooltip(|window, cx| Tooltip::new("Toggle properties sidebar").build(window, cx))
                .on_activate(cx.listener(|this, _, window, cx| {
                    this.set_collapsed(!this.collapsed, cx);
                    this.focus.focus(window, cx);
                    cx.emit(PropertiesInspectorAction::CollapsedChanged {
                        collapsed: this.collapsed,
                    });
                    cx.notify();
                }))
                .child(render_lucide_icon(
                    LucideIcon::PanelRight,
                    crate::atoms::sidebar_style(cx).icon,
                    tokens::IconSize::MD,
                )),
            )
            .into_any_element()
    }
    fn select_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let tab = PropertiesInspectorTab::ALL[index];
        self.set_active_tab(tab, cx);
        self.tab_focus[index].focus(window, cx);
        cx.emit(PropertiesInspectorAction::TabChanged { tab });
    }

    fn adjacent_tab(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let index = PropertiesInspectorTab::ALL
            .iter()
            .position(|tab| *tab == self.active_tab)
            .unwrap_or(0);
        let count = PropertiesInspectorTab::ALL.len();
        self.select_tab(
            (index + if forward { 1 } else { count - 1 }) % count,
            window,
            cx,
        );
    }

    fn tab_navigation(&self, forward: bool, cx: &mut Context<Self>) -> AnyElement {
        let suffix = if forward { "next" } else { "previous" };
        icon_button(
            format!("{}-tab-{suffix}", self.id),
            px(tokens::ControlSize::CHROME),
            px(tokens::Radius::CONTROL),
            cx,
        )
        .debug_selector(move || format!("properties-inspector-tab-{suffix}"))
        .tooltip(move |window, cx| {
            Tooltip::new(if forward {
                "Next inspector tab"
            } else {
                "Previous inspector tab"
            })
            .build(window, cx)
        })
        .on_activate(cx.listener(move |this, _, window, cx| {
            this.adjacent_tab(forward, window, cx);
        }))
        .child(render_lucide_icon(
            if forward {
                LucideIcon::ChevronRight
            } else {
                LucideIcon::ChevronLeft
            },
            crate::atoms::sidebar_style(cx).icon,
            tokens::IconSize::MD,
        ))
        .into_any_element()
    }

    fn has_zed_tab_theme(cx: &App) -> bool {
        cx.try_global::<theme::GlobalTheme>().is_some() && theme::try_theme_settings(cx).is_some()
    }

    fn tab(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let tab = PropertiesInspectorTab::ALL[index];
        let selected = tab == self.active_tab;
        let active_index = PropertiesInspectorTab::ALL
            .iter()
            .position(|tab| *tab == self.active_tab)
            .unwrap_or(0);
        let position = if index == 0 {
            TabPosition::First
        } else if index + 1 == PropertiesInspectorTab::ALL.len() {
            TabPosition::Last
        } else {
            TabPosition::Middle(index.cmp(&active_index))
        };
        let id = SharedString::from(format!("properties-inspector-tab-list-tab-{index}"));
        let selector = id.to_string();
        let activation = cx.listener(move |this, _, window, cx| this.select_tab(index, window, cx));
        if Self::has_zed_tab_theme(cx) {
            return div()
                .debug_selector(move || selector)
                .flex_none()
                .child(
                    ZedTab::new(id)
                        .position(position)
                        .toggle_state(selected)
                        .key_context(CONTROL_KEY_CONTEXT)
                        .tab_index(0)
                        .track_focus(&self.tab_focus[index])
                        .on_activate(activation)
                        .child(
                            ui::Label::new(tab.label())
                                .single_line()
                                .color(if selected {
                                    ui::Color::Default
                                } else {
                                    ui::Color::Muted
                                }),
                        ),
                )
                .into_any_element();
        }

        // Hosts may install only gpui-component's theme. Keep the Zed tab shape
        // and scrolling contract without installing application-global themes.
        let style = crate::atoms::sidebar_style(cx);
        h_flex()
            .id(id)
            .debug_selector(move || selector)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .track_focus(&self.tab_focus[index])
            .flex_none()
            .h(px(tokens::InspectorGeometry::SECTION_HEADER))
            .px(px(tokens::Space::LG))
            .border_b_1()
            .border_r_1()
            .border_color(style.border)
            .bg(if selected {
                style.background
            } else {
                Color::BackgroundPanelField.resolve(cx)
            })
            .text_color(if selected {
                style.text
            } else {
                style.muted_text
            })
            .cursor_pointer()
            .hover(|tab| tab.bg(style.hover))
            .focus(|tab| tab.border_color(style.focused_border))
            .on_activate(activation)
            .child(
                div()
                    .typography(TypographyToken::Panel)
                    .whitespace_nowrap()
                    .child(tab.label()),
            )
            .into_any_element()
    }

    fn tabs(&self, cx: &mut Context<Self>) -> AnyElement {
        let tabs = PropertiesInspectorTab::ALL
            .into_iter()
            .enumerate()
            .map(|(index, _)| self.tab(index, cx))
            .collect::<Vec<_>>();
        let previous = self.tab_navigation(false, cx);
        let next = self.tab_navigation(true, cx);
        let bar = if Self::has_zed_tab_theme(cx) {
            TabBar::new(format!("{}-zed-tabs", self.id))
                .track_scroll(&self.tabs_scroll)
                .children(tabs)
                .end_child(previous)
                .end_child(next)
                .into_any_element()
        } else {
            h_flex()
                .w_full()
                .min_w_0()
                .h(px(tokens::InspectorGeometry::SECTION_HEADER))
                .child(
                    h_flex()
                        .id(format!("{}-tab-scroll", self.id))
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .overflow_x_scroll()
                        .track_scroll(&self.tabs_scroll)
                        .children(tabs),
                )
                .child(
                    h_flex()
                        .h_full()
                        .flex_none()
                        .px_1()
                        .border_l_1()
                        .border_b_1()
                        .border_color(Color::BorderPanel.resolve(cx))
                        .child(previous)
                        .child(next),
                )
                .into_any_element()
        };
        let entity = cx.entity();
        div()
            .on_children_prepainted(move |bounds, _, app| {
                let Some(bar) = bounds.first() else {
                    return;
                };
                let changed = entity.update(app, |this, _| {
                    let measurement = (bar.size.width, this.tabs_scroll.max_offset().x);
                    let changed = this.tabs_measurement != Some(measurement);
                    this.tabs_measurement = Some(measurement);
                    changed
                });
                if changed {
                    let entity = entity.clone();
                    app.defer(move |app| {
                        entity.update(app, |this, cx| {
                            let index = PropertiesInspectorTab::ALL
                                .iter()
                                .position(|tab| *tab == this.active_tab)
                                .unwrap_or(0);
                            this.tabs_scroll.scroll_to_item(index);
                            cx.notify();
                        });
                    });
                }
            })
            .id(format!("{}-tabs", self.id))
            .debug_selector(|| "properties-inspector-tabs".to_owned())
            .w_full()
            .min_w_0()
            .flex_none()
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                let index = PropertiesInspectorTab::ALL
                    .iter()
                    .position(|tab| *tab == this.active_tab)
                    .unwrap_or(0);
                let count = PropertiesInspectorTab::ALL.len();
                let next = match event.keystroke.key.as_str() {
                    "left" | "up" => (index + count - 1) % count,
                    "right" | "down" => (index + 1) % count,
                    "home" => 0,
                    "end" => count - 1,
                    _ => return,
                };
                this.select_tab(next, window, cx);
                window.prevent_default();
                cx.stop_propagation();
            }))
            .child(bar)
            .into_any_element()
    }
}
impl Render for PropertiesInspector {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.collapsed {
            return div()
                .id(self.id.clone())
                .track_focus(&self.focus)
                .max_w_full()
                .p_2()
                .child(
                    div()
                        .id("properties-floating-card")
                        .debug_selector(|| "properties-inspector-floating-card".to_owned())
                        .w(px(PROPERTIES_INSPECTOR_MIN_WIDTH))
                        .max_w_full()
                        .rounded(px(tokens::Radius::MENU))
                        .shadow_md()
                        .bg(Color::BackgroundPanel.resolve(cx))
                        .occlude()
                        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                        .on_pinch(|_, _, cx| cx.stop_propagation())
                        .child(self.header(cx)),
                )
                .into_any_element();
        }
        v_flex()
            .id(self.id.clone())
            .debug_selector(|| "properties-inspector".to_owned())
            .track_focus(&self.focus)
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(Color::BackgroundPanel.resolve(cx))
            .text_color(crate::atoms::sidebar_style(cx).text)
            .overflow_hidden()
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .child(self.header(cx))
            .child(self.tabs(cx))
            .child(
                div()
                    .debug_selector(|| "properties-inspector-content".to_owned())
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(self.children.view(self.active_tab)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod interaction_tests;
