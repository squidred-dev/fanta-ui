//! Standalone floating chrome around the reusable header zoom controls.
use crate::molecules::ZoomControls;
pub use crate::molecules::ZoomControlsAction as ZoomBarAction;
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    Render, SharedString, Subscription, Window,
};

pub struct ZoomBar {
    controls: Entity<ZoomControls>,
    percent: u16,
    _subscription: Subscription,
}
impl EventEmitter<ZoomBarAction> for ZoomBar {}
impl ZoomBar {
    pub fn new(id: impl Into<SharedString>, percent: u16, cx: &mut Context<Self>) -> Self {
        let controls = cx.new(|cx| {
            let mut controls = ZoomControls::new(id, percent, cx);
            controls.set_framed(true, cx);
            controls
        });
        let subscription =
            cx.subscribe(&controls, |_, _, event: &ZoomBarAction, cx| cx.emit(*event));
        Self {
            controls,
            percent: percent.clamp(1, 3200),
            _subscription: subscription,
        }
    }
    pub fn percent(&self) -> u16 {
        self.percent
    }
    pub fn set_percent(&mut self, percent: u16, cx: &mut Context<Self>) {
        self.percent = percent.clamp(1, 3200);
        self.controls
            .update(cx, |controls, cx| controls.set_percent(percent, cx));
    }
}
impl Focusable for ZoomBar {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.controls.focus_handle(cx)
    }
}
impl Render for ZoomBar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.controls.clone()
    }
}
#[cfg(test)]
mod tests;
