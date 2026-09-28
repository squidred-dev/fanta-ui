use std::{cell::RefCell, rc::Rc};

use gpui::{AppContext as _, Modifiers, TestAppContext, px, size};
use gpui_component::Root;

use super::*;
use crate::{
    assets::{AssetKind, AssetPageTarget, AssetRow, AssetsViewData},
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

#[gpui::test]
fn assets_expand_below_layers_without_replacing_them(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let sidebar_slot = Rc::new(RefCell::new(None));
    let captured_sidebar = sidebar_slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let pages = cx.new(|cx| {
            PagesPanel::new(
                "assets-pages-test",
                vec![PagesPanelItem::new("page", "Page 1")],
                window,
                cx,
            )
        });
        let layers = cx.new(|cx| {
            LayersPanel::new(
                "assets-layers-test",
                vec![LayersPanelItem::new(
                    "frame",
                    "Frame",
                    LayersPanelNodeKind::Frame,
                )],
                window,
                cx,
            )
        });
        let assets = cx.new(|cx| {
            AssetsPanel::new(
                "assets-section-test",
                AssetsViewData {
                    assets: vec![AssetRow::new("image", "Cover image", AssetKind::Image)],
                    pages: vec![AssetPageTarget::new("page", "Page 1")],
                    selected_page_id: Some("page".into()),
                },
                window,
                cx,
            )
        });
        let sidebar =
            cx.new(|cx| FileInspectorSidebar::new("file-inspector-assets", pages, layers, cx));
        sidebar.update(cx, |sidebar, cx| sidebar.set_assets_panel(Some(assets), cx));
        *captured_sidebar.borrow_mut() = Some(sidebar.clone());
        Root::new(sidebar, window, cx)
    });
    cx.simulate_resize(size(px(FILE_INSPECTOR_MIN_WIDTH), px(680.)));

    let layers = cx.debug_bounds("file-inspector-layers").unwrap();
    let header = cx.debug_bounds("file-inspector-assets-header").unwrap();
    assert!(layers.bottom() <= header.top() + px(1.));
    assert!(cx.debug_bounds("file-inspector-assets").is_some());

    let entity = sidebar_slot.borrow().clone().unwrap();
    cx.simulate_click(header.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(!cx.read(|app| entity.read(app).assets_expanded()));
    assert!(cx.debug_bounds("file-inspector-layers").is_some());
    assert!(cx.debug_bounds("file-inspector-assets").is_none());

    let find = cx.debug_bounds("assets-search-trigger").unwrap();
    cx.simulate_click(find.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.read(|app| entity.read(app).assets_expanded()));
    assert!(cx.debug_bounds("file-inspector-layers").is_some());
    assert!(cx.debug_bounds("file-inspector-assets").is_some());
    assert!(cx.debug_bounds("assets-search-toolbar").is_some());
    let assets = cx.read(|app| entity.read(app).assets.clone().unwrap());
    assert!(cx.read(|app| assets.read(app).is_search_open()));
    let find = cx.debug_bounds("assets-search-trigger").unwrap();
    cx.simulate_click(find.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.read(|app| entity.read(app).assets_expanded()));

    let close = cx.debug_bounds("assets-close-search").unwrap();
    cx.simulate_click(close.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(!cx.read(|app| assets.read(app).is_search_open()));
    assert!(cx.read(|app| entity.read(app).assets_expanded()));

    let layers_header = cx.debug_bounds("layers-header").unwrap();
    cx.simulate_click(layers_header.center(), Modifiers::none());
    cx.run_until_parked();
    let layers = cx.debug_bounds("file-inspector-layers").unwrap();
    let assets_header = cx.debug_bounds("file-inspector-assets-header").unwrap();
    assert!(layers.size.height <= layers_header.size.height + px(1.));
    assert!(layers.bottom() <= assets_header.top() + px(1.));
    assert!(cx.debug_bounds("file-inspector-assets").is_some());
}
