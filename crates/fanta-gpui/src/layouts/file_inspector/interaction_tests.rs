use std::{cell::RefCell, rc::Rc};

use gpui::{AppContext as _, Modifiers, TestAppContext, px, size};
use gpui_component::Root;

use super::*;
use crate::{
    layers::{LayersPanelItem, LayersPanelNodeKind},
    pages::PagesPanelItem,
};

#[gpui::test]
fn pages_and_layers_share_one_sidebar_without_overflow(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let sidebar_slot = Rc::new(RefCell::new(None));
    let captured_sidebar = sidebar_slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let pages = cx.new(|cx| {
            PagesPanel::new(
                "file-inspector-pages-test",
                vec![PagesPanelItem::new("page", "Page 1")],
                window,
                cx,
            )
        });
        let layers = cx.new(|cx| {
            LayersPanel::new(
                "file-inspector-layers-test",
                vec![LayersPanelItem::new(
                    "frame",
                    "Frame",
                    LayersPanelNodeKind::Frame,
                )],
                window,
                cx,
            )
        });
        let sidebar = cx.new(|cx| FileInspectorSidebar::new("file-inspector", pages, layers, cx));
        *captured_sidebar.borrow_mut() = Some(sidebar.clone());
        Root::new(sidebar, window, cx)
    });
    cx.simulate_resize(size(
        px(FILE_INSPECTOR_MIN_WIDTH),
        px(FILE_INSPECTOR_MIN_HEIGHT),
    ));

    let sidebar = cx
        .debug_bounds("file-inspector-sidebar")
        .expect("the unified sidebar should render");
    let pages = cx
        .debug_bounds("file-inspector-pages")
        .expect("the Pages section should render");
    let layers = cx
        .debug_bounds("file-inspector-layers")
        .expect("the Layers section should render");
    let pages_header = cx
        .debug_bounds("pages-header")
        .expect("the Pages header should remain visible");
    let layers_header = cx
        .debug_bounds("layers-header")
        .expect("the Layers header should remain visible");

    assert!(pages_header.top() < layers_header.top());
    assert!(pages.bottom() <= layers.top() + px(1.));
    assert!(layers.right() <= sidebar.right() + px(0.5));
    assert!(layers.bottom() <= sidebar.bottom() + px(0.5));

    let entity = sidebar_slot.borrow().clone().unwrap();
    entity.update(cx, |sidebar, cx| {
        sidebar.set_project_name("My design project", cx)
    });
    let toggle = cx.debug_bounds("file-inspector-toggle").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.read(|app| entity.read(app).is_collapsed()));
    assert!(cx.debug_bounds("file-inspector-pages").is_none());
    assert!(cx.debug_bounds("file-inspector-layers").is_none());
    let card = cx.debug_bounds("file-inspector-floating-card").unwrap();
    assert!(card.size.height < sidebar.size.height);
    assert!(card.size.width <= sidebar.size.width);
    assert_eq!(
        cx.read(|app| entity.read(app).project_name.clone()),
        "My design project"
    );

    let toggle = cx.debug_bounds("file-inspector-toggle").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(!cx.read(|app| entity.read(app).is_collapsed()));
    assert!(cx.debug_bounds("file-inspector-pages").is_some());
    assert!(cx.debug_bounds("file-inspector-layers").is_some());
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(cx.read(|app| entity.read(app).is_collapsed()));
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    assert!(!cx.read(|app| entity.read(app).is_collapsed()));
}
