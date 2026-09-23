use gpui::{TestAppContext, VisualTestContext};

use super::*;
use crate::design::{DesignPageBackground, DesignPanelEditPhase, DesignPanelNodeCapabilities};
use crate::properties_inspector::{PropertiesInspector, PropertiesInspectorChildren};
use crate::test_support::mount_component;

/// Tests keep the controller off the render tree and listen at its single
/// public action boundary, matching a host's composed inspector integration.
struct Harness {
    controller: Entity<DesignPanel>,
    panel: Entity<DesignPropertyPanel>,
    inspector: Option<Entity<DesignInspector>>,
    properties: Option<Entity<PropertiesInspector>>,
    width: f32,
    _subscription: Subscription,
}

impl EventEmitter<DesignPanelAction> for Harness {}

impl Harness {
    fn new(
        kind: DesignPropertyPanelKind,
        full: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let controller = cx.new(|cx| {
            DesignPanel::new(
                "design",
                DesignPanelNode::new(
                    "opaque-node",
                    "Card",
                    if kind == DesignPropertyPanelKind::Typography {
                        DesignPanelNodeKind::Text
                    } else {
                        DesignPanelNodeKind::Rectangle
                    },
                ),
                window,
                cx,
            )
        });
        let panel = cx.new(|cx| DesignPropertyPanel::new("specimen", kind, controller.clone(), cx));
        let inspector =
            full.then(|| cx.new(|cx| DesignInspector::new("composed", controller.clone(), cx)));
        let subscription = cx.subscribe(&controller, |_, _, event: &DesignPanelAction, cx| {
            cx.emit(event.clone());
        });
        Self {
            controller,
            panel,
            inspector,
            properties: None,
            width: 400.,
            _subscription: subscription,
        }
    }

    fn new_with_properties(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut harness = Self::new(DesignPropertyPanelKind::Page, true, window, cx);
        let inspector = harness
            .inspector
            .as_ref()
            .expect("composed inspector")
            .clone();
        let child: gpui::AnyView = inspector.into();
        harness.properties = Some(cx.new(|cx| {
            PropertiesInspector::new(
                "properties-test",
                PropertiesInspectorChildren {
                    design: child.clone(),
                    motion: child.clone(),
                    draw: child.clone(),
                    code: child.clone(),
                    prototype: child.clone(),
                    comments: child,
                },
                100,
                cx,
            )
        }));
        harness
    }
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("panel-viewport")
            .debug_selector(|| "panel-viewport".into())
            .w(px(self.width))
            .h_full()
            .when_some(self.properties.clone(), |root, properties| {
                root.child(
                    div()
                        .absolute()
                        .top(px(80.))
                        .left_0()
                        .w_full()
                        .h(px(640.))
                        .child(properties),
                )
            })
            .when(self.properties.is_none(), |root| {
                root.when_some(self.inspector.clone(), |root, inspector| {
                    root.child(inspector)
                })
            })
            .when(
                self.inspector.is_none() && self.properties.is_none(),
                |root| root.child(self.panel.clone()),
            )
    }
}

fn controller(
    host: &Entity<crate::test_support::ProbeHost<Harness, DesignPanelAction>>,
    cx: &VisualTestContext,
) -> Entity<DesignPanel> {
    cx.read(|app| host.read(app).component.read(app).controller.clone())
}

#[gpui::test]
fn composed_page_background_picker_accepts_spectrum_click(cx: &mut TestAppContext) {
    let (host, actions, visual_cx) = mount_component(cx, |window, cx| {
        Harness::new(DesignPropertyPanelKind::Page, true, window, cx)
    });
    let controller = controller(&host, visual_cx);
    controller.update(visual_cx, |panel, cx| {
        panel.set_inspection_context(
            DesignPanelInspectionContext::page(DesignPanelPermissions::editor()),
            cx,
        );
        panel.set_page_view_data(
            DesignPageViewData::canonical("page", DesignPageBackground::new(DesignColor::WHITE)),
            cx,
        );
    });
    visual_cx.run_until_parked();
    let row = visual_cx
        .debug_bounds("design-page-background-row")
        .unwrap();
    visual_cx.simulate_click(row.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    let popup = visual_cx
        .debug_bounds("design-page-background-popup")
        .unwrap();
    assert!(
        popup.top() >= row.bottom(),
        "Page background picker must open below its row inside the Design inspector"
    );
    let spectrum = visual_cx.debug_bounds("color-picker-spectrum").unwrap();
    visual_cx.simulate_click(spectrum.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    assert!(actions.borrow().iter().any(|action| matches!(
        action,
        DesignPanelAction::PageBackgroundEditRequested { .. }
    )));
}

#[gpui::test]
fn page_background_picker_accepts_click_inside_properties_layout(cx: &mut TestAppContext) {
    let (host, actions, visual_cx) =
        mount_component(cx, |window, cx| Harness::new_with_properties(window, cx));
    let controller = controller(&host, visual_cx);
    controller.update(visual_cx, |panel, cx| {
        panel.set_inspection_context(
            DesignPanelInspectionContext::page(DesignPanelPermissions::editor()),
            cx,
        );
        panel.set_page_view_data(
            DesignPageViewData::canonical("page", DesignPageBackground::new(DesignColor::WHITE)),
            cx,
        );
    });
    visual_cx.run_until_parked();
    let row = visual_cx
        .debug_bounds("design-page-background-row")
        .unwrap();
    visual_cx.simulate_click(row.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    let popup = visual_cx
        .debug_bounds("design-page-background-popup")
        .unwrap();
    assert!(popup.top() >= row.bottom());
    let spectrum = visual_cx.debug_bounds("color-picker-spectrum").unwrap();
    visual_cx.simulate_click(spectrum.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    assert!(actions.borrow().iter().any(|action| matches!(
        action,
        DesignPanelAction::PageBackgroundEditRequested { .. }
    )));
}

#[gpui::test]
fn isolated_panels_mount_only_their_own_live_controls(cx: &mut TestAppContext) {
    for (kind, selector) in [
        (DesignPropertyPanelKind::Alignment, "design-x"),
        (DesignPropertyPanelKind::Layout, "design-width"),
        (DesignPropertyPanelKind::Appearance, "design-opacity-value"),
        (DesignPropertyPanelKind::Fill, "design-add-fill"),
        (DesignPropertyPanelKind::Stroke, "design-add-stroke"),
        (DesignPropertyPanelKind::Effects, "design-add-effect"),
        (DesignPropertyPanelKind::Typography, "design-font-size"),
        (DesignPropertyPanelKind::Export, "design-add-export"),
    ] {
        let (host, _, visual_cx) =
            mount_component(cx, move |window, cx| Harness::new(kind, false, window, cx));
        visual_cx.run_until_parked();
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "{kind:?} has its live controls"
        );
        assert!(visual_cx.debug_bounds("design-tab-design").is_none());
        assert!(visual_cx.debug_bounds("design-tab-prototype").is_none());
        if kind != DesignPropertyPanelKind::Fill {
            assert!(visual_cx.debug_bounds("design-add-fill").is_none());
        }
        for width in [320., 400., 472.] {
            let harness = visual_cx.read(|app| host.read(app).component.clone());
            harness.update(visual_cx, |harness, cx| {
                harness.width = width;
                cx.notify();
            });
            visual_cx.run_until_parked();
            let viewport = visual_cx.debug_bounds("panel-viewport").unwrap();
            let field = visual_cx.debug_bounds(selector).unwrap();
            assert!(field.left() >= viewport.left());
            assert!(field.right() <= viewport.right(), "{kind:?} fits {width}px");
        }
    }
}

#[gpui::test]
fn standalone_fill_preserves_host_intents_and_live_capability_gates(cx: &mut TestAppContext) {
    let (host, actions, visual_cx) = mount_component(cx, |window, cx| {
        Harness::new(DesignPropertyPanelKind::Fill, false, window, cx)
    });
    visual_cx.run_until_parked();
    let controller = controller(&host, visual_cx);
    let before = controller.read_with(visual_cx, |panel, _| panel.node().fills.clone());
    let bounds = visual_cx.debug_bounds("design-add-fill").unwrap();
    visual_cx.simulate_click(bounds.center(), Modifiers::none());
    visual_cx.run_until_parked();
    assert!(
        matches!(actions.borrow().as_slice(), [DesignPanelAction::CollectionItemAddRequested {
        node_id, collection: DesignPanelCollection::Fill, ..
    }] if node_id.as_ref() == "opaque-node")
    );
    assert_eq!(
        controller.read_with(visual_cx, |panel, _| panel.node().fills.clone()),
        before
    );

    controller.update(visual_cx, |panel, cx| {
        panel.set_node(
            DesignPanelNode::new("group", "Group", DesignPanelNodeKind::Group),
            cx,
        );
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.debug_bounds("design-add-fill").is_none());
    assert!(visual_cx.debug_bounds("specimen").is_none());
}

#[gpui::test]
fn composed_inspector_resolves_host_order_and_viewer_context_without_legacy_shell(
    cx: &mut TestAppContext,
) {
    let (host, _, visual_cx) = mount_component(cx, |window, cx| {
        Harness::new(DesignPropertyPanelKind::Alignment, true, window, cx)
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.debug_bounds("composed-alignment").is_some());
    assert!(visual_cx.debug_bounds("composed-layout").is_some());
    assert!(visual_cx.debug_bounds("design-tab-design").is_none());
    let controller = controller(&host, visual_cx);
    controller.update(visual_cx, |panel, cx| {
        panel.set_node(
            DesignPanelNode::new("custom", "Custom", DesignPanelNodeKind::Rectangle)
                .with_capabilities(
                    DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::Rectangle)
                        .with_sections([DesignPanelSection::Stroke, DesignPanelSection::Layer]),
                ),
            cx,
        );
        assert_eq!(
            panel.composed_panel_kinds(),
            vec![
                DesignPropertyPanelKind::Stroke,
                DesignPropertyPanelKind::Appearance,
            ]
        );
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.debug_bounds("composed-alignment").is_none());
    assert!(visual_cx.debug_bounds("composed-stroke").is_some());

    controller.update(visual_cx, |panel, cx| {
        panel.set_inspection_context(
            DesignPanelInspectionContext::single(
                panel.node().clone(),
                DesignPanelParentLayout::Freeform,
                DesignPanelPermissions::viewer(),
            ),
            cx,
        );
        assert_eq!(
            panel.composed_panel_kinds().first(),
            Some(&DesignPropertyPanelKind::Viewer)
        );
        assert!(
            !panel
                .composed_panel_kinds()
                .contains(&DesignPropertyPanelKind::Stroke)
        );
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.debug_bounds("design-add-stroke").is_none());
    assert!(visual_cx.debug_bounds("composed-viewer").is_some());
}

#[gpui::test]
fn standalone_panel_escape_keeps_balanced_property_edit_contract(cx: &mut TestAppContext) {
    let (host, actions, visual_cx) = mount_component(cx, |window, cx| {
        Harness::new(DesignPropertyPanelKind::Alignment, false, window, cx)
    });
    visual_cx.run_until_parked();
    let controller = controller(&host, visual_cx);
    visual_cx.update(|window, app| {
        controller.update(app, |panel, cx| {
            panel.activate_property(
                DesignPanelProperty::X,
                DesignPanelValue::Number(10.),
                window,
                cx,
            );
        });
    });
    visual_cx.run_until_parked();
    visual_cx.simulate_keystrokes("escape");
    visual_cx.run_until_parked();
    let phases: Vec<_> = actions
        .borrow()
        .iter()
        .filter_map(|action| match action {
            DesignPanelAction::PropertyEditRequested {
                property: DesignPanelProperty::X,
                phase,
                ..
            } => Some(*phase),
            _ => None,
        })
        .collect();
    assert_eq!(
        phases,
        [DesignPanelEditPhase::Begin, DesignPanelEditPhase::Cancel]
    );
    assert!(controller.read_with(visual_cx, |panel, _| panel.edit.property.is_none()));
}

#[gpui::test]
fn hiding_composition_cancels_once_and_preserves_host_navigation(cx: &mut TestAppContext) {
    let (host, actions, visual_cx) = mount_component(cx, |window, cx| {
        Harness::new(DesignPropertyPanelKind::Alignment, true, window, cx)
    });
    visual_cx.run_until_parked();
    let controller = controller(&host, visual_cx);
    let before = controller.read_with(visual_cx, |panel, _| panel.view_data());
    visual_cx.update(|window, app| {
        controller.update(app, |panel, cx| {
            panel.activate_property(
                DesignPanelProperty::X,
                DesignPanelValue::Number(10.),
                window,
                cx,
            );
            *panel.overlays.appearance_blend_mode_open_test_slot() = true;
        });
    });
    let inspector = visual_cx.read(|app| {
        host.read(app)
            .component
            .read(app)
            .inspector
            .clone()
            .unwrap()
    });
    visual_cx.update(|window, app| {
        inspector.update(app, |inspector, cx| inspector.deactivate(window, cx));
        inspector.update(app, |inspector, cx| inspector.deactivate(window, cx));
    });
    visual_cx.run_until_parked();
    let phases: Vec<_> = actions
        .borrow()
        .iter()
        .filter_map(|action| match action {
            DesignPanelAction::PropertyEditRequested {
                property: DesignPanelProperty::X,
                phase,
                ..
            } => Some(*phase),
            _ => None,
        })
        .collect();
    assert_eq!(
        phases,
        [DesignPanelEditPhase::Begin, DesignPanelEditPhase::Cancel]
    );
    controller.read_with(visual_cx, |panel, _| {
        assert!(!panel.edit.has_active_session());
        assert!(!panel.overlays.appearance_blend_mode_open());
        assert_eq!(panel.view_data(), before);
    });
}
