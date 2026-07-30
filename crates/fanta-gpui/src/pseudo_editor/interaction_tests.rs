use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, Modifiers, ParentElement as _, Render,
    Subscription, TestAppContext, VisualTestContext, Window, div, px, size,
};
use gpui_component::Root;

use super::*;
use crate::{
    assets::{AssetsPanel, AssetsViewData},
    design::{DesignPanel, DesignPanelNode, DesignPanelNodeKind},
    layers::LayersPanel,
    pages::PagesPanel,
    prototype::{PrototypePanel, PrototypeViewData},
    timeline::{Timeline, TimelineViewData},
    toolbar::{EditorToolbar, ToolbarMode, ToolbarTool},
    variables::{VariablesPage, VariablesViewData},
};

struct TestHost {
    editor: Entity<PseudoEditor>,
    actions: Rc<RefCell<Vec<PseudoEditorAction>>>,
    _subscription: Subscription,
}

impl TestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pages = cx.new(|cx| PagesPanel::new("pages", Vec::new(), window, cx));
        let layers = cx.new(|cx| LayersPanel::new("layers", Vec::new(), window, cx));
        let assets = cx.new(|cx| AssetsPanel::new("assets", AssetsViewData::default(), window, cx));
        let design = cx.new(|cx| {
            DesignPanel::new(
                "design",
                DesignPanelNode::new("frame", "Frame", DesignPanelNodeKind::Frame),
                window,
                cx,
            )
        });
        let prototype =
            cx.new(|cx| PrototypePanel::new("prototype", PrototypeViewData::default(), cx));
        let timeline = cx.new(|cx| Timeline::new("timeline", TimelineViewData::default(), cx));
        let toolbar = cx.new(|cx| {
            EditorToolbar::new(
                "toolbar",
                ToolbarMode::Design,
                ToolbarTool::Move,
                100,
                window,
                cx,
            )
        });
        let variables = cx.new(|cx| {
            VariablesPage::new(
                "variables",
                VariablesViewData {
                    document_name: "Test".into(),
                    collections: Vec::new(),
                    selected_collection_id: "".into(),
                    groups: Vec::new(),
                    selected_group_id: "".into(),
                    modes: Vec::new(),
                    variables: Vec::new(),
                },
                window,
                cx,
            )
        });
        let editor = cx.new(|cx| {
            PseudoEditor::new(
                "test-editor",
                PseudoEditorChildren {
                    pages,
                    layers,
                    assets,
                    design,
                    prototype,
                    timeline,
                    toolbar,
                    variables,
                },
                cx,
            )
        });
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&editor, move |_, _, action: &PseudoEditorAction, _| {
            captured_actions.borrow_mut().push(action.clone());
        });
        Self {
            editor,
            actions,
            _subscription: subscription,
        }
    }
}

impl Render for TestHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(1440.)).h(px(900.)).child(self.editor.clone())
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
    cx.simulate_resize(size(px(1440.), px(900.)));
    let host = host_slot
        .borrow_mut()
        .take()
        .expect("pseudo editor test host should be installed");
    (host, cx)
}

#[gpui::test]
fn shell_controls_share_pointer_enter_and_space_activation(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = cx.read(|app| host.read(app).actions.clone());
    let bounds = cx
        .debug_bounds("pseudo-editor-present")
        .expect("Present control should render");

    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[PseudoEditorAction::PresentRequested]
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter space");
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            PseudoEditorAction::PresentRequested,
            PseudoEditorAction::PresentRequested,
        ]
    );
}
