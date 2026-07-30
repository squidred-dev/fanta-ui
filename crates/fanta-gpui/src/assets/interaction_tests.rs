use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, Modifiers, ParentElement as _, Render,
    Subscription, TestAppContext, VisualTestContext, Window, div,
};
use gpui_component::Root;

use super::*;

struct TestHost {
    panel: Entity<AssetsPanel>,
    actions: Rc<RefCell<Vec<AssetsPanelAction>>>,
    _subscription: Subscription,
}

impl TestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panel =
            cx.new(|cx| AssetsPanel::new("test-assets", AssetsViewData::default(), window, cx));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&panel, move |_, _, action: &AssetsPanelAction, _| {
            captured_actions.borrow_mut().push(action.clone());
        });
        Self {
            panel,
            actions,
            _subscription: subscription,
        }
    }
}

impl Render for TestHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panel.clone())
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
        let host = cx.new(|cx| TestHost::new(window, cx));
        *captured_host.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    let host = host_slot
        .borrow_mut()
        .take()
        .expect("assets test host should be installed");
    (host, cx)
}

#[gpui::test]
fn rail_items_share_pointer_enter_and_space_activation(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = cx.read(|app| host.read(app).actions.clone());
    let bounds = cx
        .debug_bounds("assets-rail-file")
        .expect("File rail item should render");

    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[AssetsPanelAction::RailItemSelected {
            item: AssetsRailItem::File,
        }]
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter space");
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            AssetsPanelAction::RailItemSelected {
                item: AssetsRailItem::File,
            },
            AssetsPanelAction::RailItemSelected {
                item: AssetsRailItem::File,
            },
        ]
    );
}
