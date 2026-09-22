use super::*;
use crate::atoms::tokens;
use gpui::{
    Entity, Modifiers, ScrollDelta, ScrollWheelEvent, Subscription, TestAppContext,
    VisualTestContext, size,
};
use gpui::{InteractiveElement as _, ParentElement as _, Styled as _, div, point, px};
use gpui_component::Root;
use std::{cell::RefCell, rc::Rc};
struct Host {
    bar: Entity<ZoomBar>,
    events: Rc<RefCell<Vec<ZoomBarAction>>>,
    scrolls: usize,
    _subscription: Subscription,
}
impl Host {
    fn new(cx: &mut Context<Self>) -> Self {
        let bar = cx.new(|cx| ZoomBar::new("test-zoombar", 100, cx));
        let events = Rc::new(RefCell::new(Vec::new()));
        let captured = events.clone();
        let subscription = cx.subscribe(&bar, move |_, _, event: &ZoomBarAction, _| {
            captured.borrow_mut().push(*event)
        });
        Self {
            bar,
            events,
            scrolls: 0,
            _subscription: subscription,
        }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("zoom-host")
            .size_full()
            .relative()
            .on_scroll_wheel(cx.listener(|this, _, _, _| this.scrolls += 1))
            .child(div().absolute().top_4().right_4().child(self.bar.clone()))
    }
}
fn setup(cx: &mut TestAppContext) -> (Entity<Host>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let slot = Rc::new(RefCell::new(None));
    let captured = slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let host = cx.new(Host::new);
        *captured.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    let host = slot.borrow_mut().take().unwrap();
    (host, cx)
}
fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let point = cx.debug_bounds(selector).unwrap().center();
    cx.simulate_click(point, Modifiers::none());
    cx.run_until_parked();
}
#[gpui::test]
fn zoom_and_fit_controls_emit_and_wait_for_host_echo(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let bar = cx.read(|app| host.read(app).bar.clone());
    let events = cx.read(|app| host.read(app).events.clone());
    click(cx, "zoombar-in");
    assert_eq!(
        events.borrow().last(),
        Some(&ZoomBarAction::ZoomChangeRequested { percent: 200 })
    );
    assert_eq!(cx.read(|app| bar.read(app).percent()), 100);
    cx.update(|_, app| bar.update(app, |bar, cx| bar.set_percent(200, cx)));
    click(cx, "zoombar-out");
    assert_eq!(
        events.borrow().last(),
        Some(&ZoomBarAction::ZoomChangeRequested { percent: 100 })
    );
    events.borrow_mut().clear();
    click(cx, "zoombar-fit");
    cx.simulate_keystrokes("enter space");
    assert_eq!(
        events.borrow().as_slice(),
        &[ZoomBarAction::FitToViewRequested; 3]
    );
}
#[gpui::test]
fn zoom_menu_aligns_to_its_bar_and_supports_keyboard_and_scroll_isolation(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let events = cx.read(|app| host.read(app).events.clone());
    cx.simulate_resize(size(px(500.), px(400.)));
    cx.run_until_parked();
    click(cx, "zoombar-percent");
    let bar = cx.debug_bounds("zoombar").unwrap();
    let menu = cx.debug_bounds("zoombar-menu").unwrap();
    assert_eq!(menu.top(), bar.bottom() + px(tokens::Space::XS));
    assert_eq!(menu.right(), bar.right());
    cx.simulate_keystrokes("down down down enter");
    cx.run_until_parked();
    assert_eq!(
        events.borrow().last(),
        Some(&ZoomBarAction::FitToSelectionRequested)
    );
    assert!(cx.debug_bounds("zoombar-menu").is_none());
    click(cx, "zoombar-percent");
    let menu = cx.debug_bounds("zoombar-menu").unwrap();
    cx.simulate_event(ScrollWheelEvent {
        position: menu.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    assert_eq!(cx.read(|app| host.read(app).scrolls), 0);
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(cx.debug_bounds("zoombar-menu").is_none());
}
