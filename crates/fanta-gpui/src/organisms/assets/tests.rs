use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, Focusable as _, IntoElement, Modifiers, ParentElement as _,
    Render, Subscription, TestAppContext, VisualTestContext, Window, div, px, size,
};
use gpui_component::Root;

use super::*;

struct Host {
    panel: Entity<AssetsPanel>,
    actions: Rc<RefCell<Vec<AssetsPanelAction>>>,
    _subscription: Subscription,
}

impl Host {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut image = AssetRow::new("image", "Coastal House", AssetKind::Image);
        image.detail = "2048 × 2048".into();
        let mut video = AssetRow::new("video", "Product Reveal", AssetKind::Video);
        video.can_place = false;
        video.disabled_reason = Some("Open Motion or Prototype".into());
        let image_two = AssetRow::new("image-two", "Coastal Sunset", AssetKind::Image);
        let svg = AssetRow::new("svg", "Vector Monogram", AssetKind::Svg);
        let panel = cx.new(|cx| {
            AssetsPanel::new(
                "test-assets",
                AssetsViewData {
                    assets: vec![image, video, image_two, svg],
                    pages: vec![
                        AssetPageTarget::new("page-a", "Home"),
                        AssetPageTarget::new("page-b", "Campaign"),
                    ],
                    selected_page_id: Some("page-a".into()),
                },
                window,
                cx,
            )
        });
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured = actions.clone();
        let subscription = cx.subscribe(&panel, move |_, _, action: &AssetsPanelAction, _| {
            captured.borrow_mut().push(action.clone());
        });
        Self {
            panel,
            actions,
            _subscription: subscription,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(280.)).h(px(520.)).child(self.panel.clone())
    }
}

fn setup(cx: &mut TestAppContext) -> (Entity<Host>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let slot = Rc::new(RefCell::new(None));
    let captured_slot = slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let host = cx.new(|cx| Host::new(window, cx));
        *captured_slot.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    cx.simulate_resize(size(px(280.), px(520.)));
    let host = slot.borrow_mut().take().unwrap();
    (host, cx)
}

#[gpui::test]
fn placement_uses_host_selected_page_and_disables_unavailable_media(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let (panel, actions) = cx.read(|cx| {
        let host = host.read(cx);
        (host.panel.clone(), host.actions.clone())
    });

    let image = cx.debug_bounds("asset-place-image").unwrap();
    cx.simulate_click(image.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[AssetsPanelAction::PlaceRequested {
            asset_id: "image".into(),
            page_id: "page-a".into()
        },]
    );

    cx.update(|window, cx| {
        panel.update(cx, |panel, cx| {
            let mut data = panel.view_data().clone();
            data.selected_page_id = Some("page-b".into());
            panel.set_view_data(data, window, cx);
        });
    });
    cx.run_until_parked();
    let image = cx.debug_bounds("asset-place-image").unwrap();
    cx.simulate_click(image.center(), Modifiers::none());
    cx.run_until_parked();
    let placements = actions
        .borrow()
        .iter()
        .filter_map(|action| match action {
            AssetsPanelAction::PlaceRequested { asset_id, page_id } => {
                Some((asset_id.clone(), page_id.clone()))
            }
            AssetsPanelAction::TargetPageSelected { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placements,
        vec![
            ("image".into(), "page-a".into()),
            ("image".into(), "page-b".into())
        ]
    );

    let video = cx.debug_bounds("asset-place-video").unwrap();
    cx.simulate_click(video.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions
            .borrow()
            .iter()
            .filter(|action| matches!(action, AssetsPanelAction::PlaceRequested { .. }))
            .count(),
        2
    );
}

#[gpui::test]
fn find_shortcut_searches_assets_and_navigates_selected_results(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = cx.read(|app| host.read(app).panel.clone());
    assert!(cx.debug_bounds("assets-search-toolbar").is_none());

    cx.update(|window, app| panel.focus_handle(app).focus(window, app));
    cx.simulate_keystrokes("secondary-f");
    cx.run_until_parked();
    assert!(cx.read(|app| panel.read(app).is_search_open()));
    assert!(cx.debug_bounds("assets-search-toolbar").is_some());

    cx.simulate_keystrokes("c o a s t a l");
    cx.run_until_parked();
    assert_eq!(cx.read(|app| panel.read(app).visible_assets(app).len()), 2);
    assert!(cx.debug_bounds("asset-row-image").is_some());
    assert!(cx.debug_bounds("asset-row-image-two").is_some());
    assert!(cx.debug_bounds("asset-row-video").is_none());

    cx.simulate_keystrokes("enter");
    assert_eq!(
        cx.read(|app| panel.read(app).active_result.clone()),
        Some("image".into())
    );
    cx.simulate_keystrokes("enter");
    assert_eq!(
        cx.read(|app| panel.read(app).active_result.clone()),
        Some("image-two".into())
    );
    cx.simulate_keystrokes("shift-enter");
    assert_eq!(
        cx.read(|app| panel.read(app).active_result.clone()),
        Some("image".into())
    );
    let next = cx.debug_bounds("assets-next-result").unwrap();
    cx.simulate_click(next.center(), Modifiers::none());
    assert_eq!(
        cx.read(|app| panel.read(app).active_result.clone()),
        Some("image-two".into())
    );
    let previous = cx.debug_bounds("assets-previous-result").unwrap();
    cx.simulate_click(previous.center(), Modifiers::none());
    assert_eq!(
        cx.read(|app| panel.read(app).active_result.clone()),
        Some("image".into())
    );
    assert_eq!(
        cx.read(|app| panel.read(app).search.read(app).value()),
        "coastal"
    );

    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!cx.read(|app| panel.read(app).is_search_open()));
    assert_eq!(cx.read(|app| panel.read(app).visible_assets(app).len()), 4);
    assert!(cx.debug_bounds("assets-search-toolbar").is_none());
}

#[gpui::test]
fn type_menu_filters_search_and_details_are_searchable(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = cx.read(|app| host.read(app).panel.clone());
    cx.update(|window, app| panel.focus_handle(app).focus(window, app));
    cx.simulate_keystrokes("secondary-f");
    cx.run_until_parked();

    cx.simulate_keystrokes("2 0 4 8");
    assert_eq!(
        cx.read(|app| panel.read(app).visible_assets(app)[0].id.clone()),
        "image"
    );
    cx.simulate_keystrokes("secondary-a delete");
    assert_eq!(cx.read(|app| panel.read(app).visible_assets(app).len()), 4);

    let trigger = cx.debug_bounds("assets-filter-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("assets-filter-menu").is_some());
    let svg = cx.debug_bounds("assets-filter-SVG").unwrap();
    cx.simulate_click(svg.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(cx.read(|app| panel.read(app).visible_assets(app).len()), 1);
    assert!(cx.debug_bounds("asset-row-svg").is_some());
    assert!(cx.debug_bounds("asset-row-image").is_none());
    assert!(cx.debug_bounds("assets-filter-menu").is_none());

    let trigger = cx.debug_bounds("assets-filter-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(cx.read(|app| panel.read(app).is_search_open()));
    assert!(cx.debug_bounds("assets-filter-menu").is_none());
}
