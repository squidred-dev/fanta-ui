use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, Modifiers, ParentElement as _, Render,
    Subscription, TestAppContext, VisualTestContext, Window, div,
};
use gpui_component::Root;

use super::*;

struct TestHost {
    timeline: Entity<Timeline>,
    actions: Rc<RefCell<Vec<TimelineAction>>>,
    _subscription: Subscription,
}

impl TestHost {
    fn new(cx: &mut Context<Self>) -> Self {
        let timeline = cx.new(|cx| Timeline::new("test-timeline", TimelineViewData::default(), cx));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&timeline, move |_, _, action: &TimelineAction, _| {
            captured_actions.borrow_mut().push(action.clone());
        });
        Self {
            timeline,
            actions,
            _subscription: subscription,
        }
    }
}

impl Render for TestHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.timeline.clone())
    }
}

fn setup(cx: &mut TestAppContext) -> (Entity<TestHost>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let host_slot = Rc::new(RefCell::new(None));
    let captured_host = host_slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let host = cx.new(TestHost::new);
        *captured_host.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    let host = host_slot
        .borrow_mut()
        .take()
        .expect("timeline test host should be installed");
    (host, cx)
}

#[gpui::test]
fn play_shares_pointer_enter_and_space_activation(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = cx.read(|app| host.read(app).actions.clone());
    let bounds = cx
        .debug_bounds("timeline-play")
        .expect("play control should render");

    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[TimelineAction::PlayStateChangeRequested { playing: true }]
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter space");
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            TimelineAction::PlayStateChangeRequested { playing: true },
            TimelineAction::PlayStateChangeRequested { playing: true },
        ]
    );
}
