//! Independently placeable, host-controlled canvas zoom chrome.
use crate::atoms::{
    ActivateControl, ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _,
    TypographyToken, icon_button, render_lucide_icon, tokens, track_bounds,
};
use crate::molecules::{POPUP_SAFE_MARGIN, menu_item, popup_height, popup_surface, popup_width};
use gpui::{
    Anchor, AnyElement, App, Bounds, Context, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyDownEvent, MouseButton, ParentElement as _, Pixels,
    Render, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    anchored, deferred, div, point, prelude::FluentBuilder as _, px,
};
use gpui_component::{ActiveTheme as _, Icon, IconName, Sizable as _, h_flex, tooltip::Tooltip};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoomControlsAction {
    ZoomChangeRequested { percent: u16 },
    FitToViewRequested,
    FitToSelectionRequested,
}
#[derive(Clone, Copy)]
enum Entry {
    In,
    Out,
    Fit,
    Selection,
    Actual,
    Half,
}
impl Entry {
    const ALL: [Self; 6] = [
        Self::In,
        Self::Out,
        Self::Fit,
        Self::Selection,
        Self::Actual,
        Self::Half,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::In => "Zoom in",
            Self::Out => "Zoom out",
            Self::Fit => "Fit to view",
            Self::Selection => "Fit to selection",
            Self::Actual => "Zoom to 100%",
            Self::Half => "Zoom to 50%",
        }
    }
    fn icon(self) -> LucideIcon {
        match self {
            Self::In => LucideIcon::ZoomIn,
            Self::Out => LucideIcon::ZoomOut,
            Self::Fit => LucideIcon::Scan,
            Self::Selection => LucideIcon::Focus,
            Self::Actual => LucideIcon::Square,
            Self::Half => LucideIcon::Minimize2,
        }
    }
}
pub struct ZoomControls {
    id: SharedString,
    focus: FocusHandle,
    percent: u16,
    framed: bool,
    open: bool,
    cursor: usize,
    bounds: Bounds<Pixels>,
    menu_scroll: ScrollHandle,
}
impl EventEmitter<ZoomControlsAction> for ZoomControls {}
impl Focusable for ZoomControls {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl ZoomControls {
    pub fn new(id: impl Into<SharedString>, percent: u16, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            focus: cx.focus_handle(),
            percent: percent.clamp(1, 3200),
            framed: false,
            open: false,
            cursor: 0,
            bounds: Bounds::default(),
            menu_scroll: ScrollHandle::new(),
        }
    }
    /// Adds standalone floating chrome; headers leave this disabled.
    pub fn set_framed(&mut self, framed: bool, cx: &mut Context<Self>) {
        self.framed = framed;
        cx.notify();
    }
    pub fn percent(&self) -> u16 {
        self.percent
    }
    pub fn set_percent(&mut self, percent: u16, cx: &mut Context<Self>) {
        self.percent = percent.clamp(1, 3200);
        cx.notify();
    }
    fn step(current: u16, up: bool) -> u16 {
        const STEPS: &[u16] = &[1, 2, 3, 6, 12, 25, 50, 100, 200, 400, 800, 1600, 3200];
        if up {
            STEPS.iter().copied().find(|v| *v > current).unwrap_or(3200)
        } else {
            STEPS
                .iter()
                .rev()
                .copied()
                .find(|v| *v < current)
                .unwrap_or(1)
        }
    }
    fn invoke(&mut self, entry: Entry, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.focus.focus(window, cx);
        }
        self.open = false;
        cx.emit(match entry {
            Entry::In => ZoomControlsAction::ZoomChangeRequested {
                percent: Self::step(self.percent, true),
            },
            Entry::Out => ZoomControlsAction::ZoomChangeRequested {
                percent: Self::step(self.percent, false),
            },
            Entry::Fit => ZoomControlsAction::FitToViewRequested,
            Entry::Selection => ZoomControlsAction::FitToSelectionRequested,
            Entry::Actual => ZoomControlsAction::ZoomChangeRequested { percent: 100 },
            Entry::Half => ZoomControlsAction::ZoomChangeRequested { percent: 50 },
        });
        cx.notify();
    }
    fn toggle(&mut self, was_open: bool, cx: &mut Context<Self>) {
        self.open = !was_open;
        self.cursor = 0;
        self.menu_scroll.scroll_to_item(0);
        cx.notify();
    }
    fn button(&self, entry: Entry, selector: &'static str, cx: &mut Context<Self>) -> AnyElement {
        icon_button(
            format!("{}-{selector}", self.id),
            px(tokens::ControlSize::CHROME),
            px(tokens::Radius::CONTROL),
            cx,
        )
        .debug_selector(move || selector.to_owned())
        .tooltip(move |window, cx| Tooltip::new(entry.label()).build(window, cx))
        .on_activate(cx.listener(move |this, _, window, cx| this.invoke(entry, window, cx)))
        .child(render_lucide_icon(
            entry.icon(),
            Color::Text.resolve(cx),
            tokens::IconSize::MD,
        ))
        .into_any_element()
    }
    fn menu(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let width = popup_width(window, 220.);
        let height = popup_height(
            window,
            6. * tokens::RowHeight::MENU + 2. * tokens::Space::XS + 2.,
        );
        let mut menu = popup_surface(format!("{}-menu", self.id), px(tokens::Radius::MENU), cx)
            .debug_selector(|| "zoombar-menu".to_owned())
            .w(width)
            .h(height)
            .p_1()
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .track_scroll(&self.menu_scroll)
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.open = false;
                cx.notify();
            }));
        for (index, entry) in Entry::ALL.into_iter().enumerate() {
            menu = menu.child(
                menu_item(
                    format!("{}-entry-{index}", self.id),
                    px(tokens::RowHeight::MENU),
                    cx,
                )
                .debug_selector(move || format!("zoombar-entry-{index}"))
                .when(index == self.cursor, |row| {
                    row.bg(Color::BackgroundSelected.resolve(cx))
                })
                .on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        this.cursor = index;
                        cx.notify();
                    }
                }))
                .on_activate(cx.listener(move |this, _, window, cx| this.invoke(entry, window, cx)))
                .child(render_lucide_icon(
                    entry.icon(),
                    Color::Text.resolve(cx),
                    tokens::IconSize::SM,
                ))
                .child(entry.label()),
            );
        }
        let above = self.bounds.top() >= height + px(POPUP_SAFE_MARGIN + tokens::Space::XS);
        let x = (self.bounds.right() - width).max(px(POPUP_SAFE_MARGIN));
        let y = if above {
            self.bounds.top() - px(tokens::Space::XS)
        } else {
            self.bounds.bottom() + px(tokens::Space::XS)
        };
        deferred(
            anchored()
                .position(point(x, y))
                .anchor(if above {
                    Anchor::BottomLeft
                } else {
                    Anchor::TopLeft
                })
                .snap_to_window_with_margin(px(POPUP_SAFE_MARGIN))
                .child(menu),
        )
        .with_priority(12)
        .into_any_element()
    }
}
impl Render for ZoomControls {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open;
        h_flex()
            .id(self.id.clone())
            .debug_selector(|| "zoombar".to_owned())
            .track_focus(&self.focus)
            .relative()
            .flex_none()
            .gap_1()
            .when(self.framed, |row| {
                row.p_1()
                    .rounded(px(tokens::Radius::MENU))
                    .border_1()
                    .border_color(Color::Border.resolve(cx))
                    .bg(Color::BackgroundMenu.resolve(cx))
            })
            .text_color(Color::Text.resolve(cx))
            .when(self.framed && cx.theme().shadow, |bar| bar.shadow_md())
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.open {
                    match event.keystroke.key.as_str() {
                        "escape" => {
                            this.open = false;
                            this.focus.focus(window, cx);
                        }
                        "down" => this.cursor = (this.cursor + 1) % Entry::ALL.len(),
                        "up" => {
                            this.cursor = (this.cursor + Entry::ALL.len() - 1) % Entry::ALL.len()
                        }
                        "home" => this.cursor = 0,
                        "end" => this.cursor = Entry::ALL.len() - 1,
                        _ => return,
                    }
                    this.menu_scroll.scroll_to_item(this.cursor);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(self.button(Entry::Out, "zoombar-out", cx))
            .child(
                icon_button(
                    format!("{}-percent", self.id),
                    px(tokens::ControlSize::CHROME),
                    px(tokens::Radius::CONTROL),
                    cx,
                )
                .debug_selector(|| "zoombar-percent".to_owned())
                .w(px(tokens::InputGeometry::NUMERIC_WIDTH))
                .gap_1()
                .typography(TypographyToken::BodyMedium)
                .tooltip(|window, cx| Tooltip::new("Zoom level").build(window, cx))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| this.toggle(open, cx)),
                )
                .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
                    if open {
                        this.invoke(Entry::ALL[this.cursor], window, cx);
                    } else {
                        this.toggle(false, cx);
                    }
                }))
                .child(format!("{}%", self.percent))
                .child(Icon::new(IconName::ChevronDown).xsmall()),
            )
            .child(self.button(Entry::In, "zoombar-in", cx))
            .child(
                div()
                    .h(px(tokens::ControlSize::INLINE))
                    .w(px(tokens::MenuGeometry::DIVIDER_HEIGHT))
                    .bg(Color::Border.resolve(cx)),
            )
            .child(self.button(Entry::Fit, "zoombar-fit", cx))
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.bounds = bounds.dilate(px(tokens::MenuGeometry::DIVIDER_HEIGHT))
            }))
            .when(open, |bar| bar.child(self.menu(window, cx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_ladder_handles_non_steps_and_limits() {
        assert_eq!(ZoomControls::step(75, true), 100);
        assert_eq!(ZoomControls::step(75, false), 50);
        assert_eq!(ZoomControls::step(3200, true), 3200);
        assert_eq!(ZoomControls::step(1, false), 1);
    }
}
