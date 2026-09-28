use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, Modifiers, ParentElement as _, Render,
    Subscription, TestAppContext, Window, div, px, size,
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
        let panel = cx.new(|cx| {
            AssetsPanel::new(
                "test-assets",
                AssetsViewData {
                    assets: vec![image, video],
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

#[gpui::test]
fn placement_uses_host_selected_page_and_disables_unavailable_media(cx: &mut TestAppContext) {
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
    let host = slot.borrow().clone().unwrap();
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
