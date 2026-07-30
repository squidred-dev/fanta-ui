use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, Modifiers, ParentElement as _, Render,
    Subscription, TestAppContext, VisualTestContext, Window, div,
};
use gpui_component::Root;

use super::*;

struct TestHost {
    page: Entity<VariablesPage>,
    actions: Rc<RefCell<Vec<VariablesAction>>>,
    _subscription: Subscription,
}

impl TestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let page = cx.new(|cx| VariablesPage::new("test-variables", fixture(), window, cx));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&page, move |_, _, action: &VariablesAction, _| {
            captured_actions.borrow_mut().push(action.clone());
        });
        Self {
            page,
            actions,
            _subscription: subscription,
        }
    }
}

impl Render for TestHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.page.clone())
    }
}

fn fixture() -> VariablesViewData {
    VariablesViewData {
        document_name: "Test document".into(),
        collections: vec![VariablesCollection::new("collection", "Primitives", 1)],
        selected_collection_id: "collection".into(),
        groups: vec![VariablesGroup::new("all", "All variables", 1).aggregate()],
        selected_group_id: "all".into(),
        modes: vec![VariablesMode::new("light", "Light")],
        variables: vec![VariableRow::new(
            "color",
            "Brand",
            "all",
            VariableKind::Color,
            [VariableModeValue::new("light", "#336699").color("336699")],
        )],
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
        .expect("variables test host should be installed");
    (host, cx)
}

#[gpui::test]
fn collection_rows_share_pointer_enter_and_space_activation(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = cx.read(|app| host.read(app).actions.clone());
    let bounds = cx
        .debug_bounds("variables-collection-collection")
        .expect("collection row should render");

    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[VariablesAction::CollectionSelected {
            collection_id: "collection".into(),
        }]
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter space");
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            VariablesAction::CollectionSelected {
                collection_id: "collection".into(),
            },
            VariablesAction::CollectionSelected {
                collection_id: "collection".into(),
            },
        ]
    );
}
