//! Right sidebar composition. Child organisms keep their original intent streams.
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, Tabs, icon_button, render_lucide_icon,
    tokens,
};
use crate::molecules::{ZoomControls, ZoomControlsAction};
use gpui::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, div, px,
};
use gpui_component::{h_flex, tooltip::Tooltip, v_flex};

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
    TabChanged { tab: PropertiesInspectorTab },
    CollapsedChanged { collapsed: bool },
    Zoom(ZoomControlsAction),
}
pub struct PropertiesInspector {
    id: SharedString,
    focus: FocusHandle,
    children: PropertiesInspectorChildren,
    zoom: Entity<ZoomControls>,
    active_tab: PropertiesInspectorTab,
    collapsed: bool,
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
            children,
            zoom,
            active_tab: PropertiesInspectorTab::Design,
            collapsed: false,
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
        cx.notify();
    }
    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.collapsed = collapsed;
        cx.notify();
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
            .h(px(tokens::RowHeight::SECTION_HEADER + tokens::Space::SM))
            .px_2()
            .gap_1()
            .child(self.zoom.clone())
            .child(div().flex_1())
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
                    this.collapsed = !this.collapsed;
                    this.focus.focus(window, cx);
                    cx.emit(PropertiesInspectorAction::CollapsedChanged {
                        collapsed: this.collapsed,
                    });
                    cx.notify();
                }))
                .child(render_lucide_icon(
                    LucideIcon::PanelRight,
                    Color::Text.resolve(cx),
                    tokens::IconSize::MD,
                )),
            )
            .into_any_element()
    }
    fn tabs(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .id(format!("{}-tabs", self.id))
            .debug_selector(|| "properties-inspector-tabs".to_owned())
            .w_full()
            .min_w_0()
            .flex_none()
            .px_2()
            .py_1()
            .gap_1()
            .overflow_x_scroll()
            .border_b_1()
            .border_color(Color::Border.resolve(cx))
            .child(
                Tabs::new(
                    "properties-inspector-tab-list",
                    PropertiesInspectorTab::ALL
                        .iter()
                        .map(|tab| tab.label().into())
                        .collect(),
                )
                .natural_width(true)
                .compact(true)
                .selected_index(
                    PropertiesInspectorTab::ALL
                        .iter()
                        .position(|tab| *tab == self.active_tab)
                        .unwrap_or_default(),
                )
                .on_change(cx.listener(
                    |this, selection: &crate::atoms::TabSelection, _, cx| {
                        let tab = PropertiesInspectorTab::ALL[selection.index];
                        this.set_active_tab(tab, cx);
                        cx.emit(PropertiesInspectorAction::TabChanged { tab });
                    },
                )),
            )
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
                        .bg(Color::BackgroundToolbar.resolve(cx))
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
            .bg(Color::BackgroundToolbar.resolve(cx))
            .text_color(Color::Text.resolve(cx))
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
