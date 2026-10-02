use super::*;
use crate::test_support::{assert_pointer_and_keyboard_parity, mount_component};
use gpui::{Modifiers, TestAppContext, px, size};

#[gpui::test]
fn motion_controls_request_edits_without_mutating_host_state(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        MotionInspector::new(
            "motion",
            MotionInspectorViewData {
                selection_name: "Hero".into(),
                animated_properties: vec![InspectorChoice::new("opacity", "Opacity")],
                can_preview: true,
                ..Default::default()
            },
            cx,
        )
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "motion-play",
        &actions,
        MotionInspectorAction::PlayingChangeRequested { playing: true },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "motion-auto",
        &actions,
        MotionInspectorAction::AutoKeyframeChangeRequested { enabled: true },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "motion-key-opacity",
        &actions,
        MotionInspectorAction::KeyframeAddRequested {
            property_id: "opacity".into(),
        },
    );
    cx.read(|app| {
        let data = host.read(app).component.read(app).view_data();
        assert!(!data.playing);
        assert!(!data.auto_keyframe);
    });
}

#[gpui::test]
fn code_copy_and_wrap_preserve_the_host_projection(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        CodeInspector::new(
            "code",
            CodeInspectorViewData {
                code: "width: 320px;".into(),
                selection_name: "Card".into(),
                ..Default::default()
            },
            cx,
        )
    });
    cx.simulate_resize(size(px(320.), px(700.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "code-copy",
        &actions,
        CodeInspectorAction::CopyRequested {
            code: "width: 320px;".into(),
        },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "code-wrap",
        &actions,
        CodeInspectorAction::WrapLinesChangeRequested { enabled: true },
    );
    cx.read(|app| assert!(!host.read(app).component.read(app).view_data().wrap_lines));
}

#[gpui::test]
fn draw_pressure_is_controlled_and_read_only_suppresses_edits(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    let options = crate::toolbar::DrawToolbarOptions {
        pressure: false,
        ..Default::default()
    };
    assert_pointer_and_keyboard_parity(
        cx,
        "draw-pressure",
        &actions,
        DrawInspectorAction::OptionsChangeRequested { options },
    );
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        let mut data = view.view_data().clone();
        data.read_only = true;
        view.set_view_data(data, cx);
    });
    cx.run_until_parked();
    let bounds = cx.debug_bounds("draw-pressure").unwrap();
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn draw_checkbox_labels_receive_their_column_width_at_compact_and_wider_sizes(
    cx: &mut TestAppContext,
) {
    let (_host, actions, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    for width in [320., 400.] {
        cx.simulate_resize(size(px(width), px(720.)));
        cx.run_until_parked();
        for (row_selector, content_selector) in [
            ("draw-pressure", "draw-pressure-checkbox-content"),
            ("draw-antialias", "draw-antialias-checkbox-content"),
        ] {
            let row = cx.debug_bounds(row_selector).unwrap();
            let content = cx.debug_bounds(content_selector).unwrap();
            assert!(
                content.size.width >= row.size.width - px(2.),
                "{content_selector} lost its label allocation: {content:?}, column {row:?}"
            );
            assert!(
                content.size.width >= px(120.),
                "{content_selector} cannot fit its checkbox and full label: {content:?}"
            );
        }
    }
    let row = cx.debug_bounds("draw-pressure").unwrap();
    for position in [
        gpui::point(row.left() + px(8.), row.center().y),
        gpui::point(row.left() + px(70.), row.center().y),
        gpui::point(row.right() - px(8.), row.center().y),
    ] {
        actions.borrow_mut().clear();
        cx.simulate_click(position, Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            &[DrawInspectorAction::OptionsChangeRequested {
                options: crate::toolbar::DrawToolbarOptions {
                    pressure: false,
                    ..Default::default()
                }
            }]
        );
    }
}

#[gpui::test]
fn tip_picker_requests_a_soft_brush_by_pointer_and_keyboard_without_accepting_it(
    cx: &mut TestAppContext,
) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    let options = crate::toolbar::DrawToolbarOptions {
        brush_tip: "Soft round".into(),
        hardness: 0,
        ..Default::default()
    };
    let trigger = cx.debug_bounds("draw-tip").unwrap().center();
    cx.simulate_click(trigger, Modifiers::none());
    cx.run_until_parked();
    let soft = cx.debug_bounds("draw-tip-Soft round").unwrap().center();
    cx.simulate_click(soft, Modifiers::none());
    cx.run_until_parked();
    let expected = DrawInspectorAction::OptionsChangeRequested { options };
    assert_eq!(actions.borrow().as_slice(), std::slice::from_ref(&expected));
    assert!(cx.debug_bounds("draw-tip-menu").is_none());
    actions.borrow_mut().clear();
    cx.simulate_click(trigger, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_keystrokes("down enter");
    cx.run_until_parked();
    assert_eq!(actions.borrow().as_slice(), std::slice::from_ref(&expected));
    assert!(cx.debug_bounds("draw-tip-menu").is_none());
    cx.read(|app| {
        let data = host.read(app).component.read(app).view_data();
        assert_eq!(data.options.brush_tip.as_ref(), "Round");
        assert_eq!(data.options.hardness, 100);
    });
    actions.borrow_mut().clear();
    cx.simulate_click(trigger, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_keystrokes("end escape");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    assert!(cx.debug_bounds("draw-tip-menu").is_none());
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        let mut data = view.view_data().clone();
        data.read_only = true;
        view.set_view_data(data, cx);
    });
    cx.run_until_parked();
    cx.simulate_click(trigger, Modifiers::none());
    cx.simulate_keystrokes("down enter");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    assert!(cx.debug_bounds("draw-tip-menu").is_none());
}

#[gpui::test]
fn draw_exact_size_and_flow_fields_preserve_precision_and_reject_out_of_range_values(
    cx: &mut TestAppContext,
) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(320.), px(720.)));
    cx.run_until_parked();
    let size_position = cx.debug_bounds("draw-size").unwrap().center();
    cx.simulate_click(size_position, Modifiers::none());
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("5000");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&DrawInspectorAction::OptionsChangeRequested {
            options: crate::toolbar::DrawToolbarOptions {
                size: 5000,
                ..Default::default()
            }
        })
    );
    let flow_position = cx.debug_bounds("draw-flow").unwrap().center();
    cx.simulate_click(flow_position, Modifiers::none());
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("37");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&DrawInspectorAction::OptionsChangeRequested {
            options: crate::toolbar::DrawToolbarOptions {
                flow: 37,
                ..Default::default()
            }
        })
    );
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("101");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    cx.read(|app| {
        let data = host.read(app).component.read(app).view_data();
        assert_eq!(data.options.size, 24);
        assert_eq!(data.options.flow, 100);
    });
}

#[gpui::test]
fn vector_pencil_capabilities_show_only_supported_draw_controls(cx: &mut TestAppContext) {
    let (host, _, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        view.set_capabilities(crate::toolbar::DrawBrushCapabilities::VECTOR_PENCIL, cx);
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    for selector in [
        "draw-size",
        "draw-color",
        "draw-opacity",
        "draw-smoothing",
        "draw-blend",
    ] {
        assert!(
            cx.debug_bounds(selector).is_some(),
            "{selector} should remain visible"
        );
    }
    for selector in [
        "draw-tip",
        "draw-hardness",
        "draw-flow",
        "draw-pressure",
        "draw-antialias",
        "draw-save-preset",
    ] {
        assert!(
            cx.debug_bounds(selector).is_none(),
            "{selector} should be hidden"
        );
    }
}

#[gpui::test]
fn vector_eraser_capabilities_show_only_size(cx: &mut TestAppContext) {
    let (host, _, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        view.set_capabilities(crate::toolbar::DrawBrushCapabilities::VECTOR_ERASER, cx);
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    assert!(cx.debug_bounds("draw-size").is_some());
    for selector in [
        "draw-tip",
        "draw-hardness",
        "draw-color",
        "draw-blend",
        "draw-opacity",
        "draw-flow",
        "draw-smoothing",
        "draw-pressure",
        "draw-antialias",
        "draw-save-preset",
    ] {
        assert!(
            cx.debug_bounds(selector).is_none(),
            "{selector} should be hidden"
        );
    }
}

#[gpui::test]
fn nonbrush_draw_tools_hide_brush_settings(cx: &mut TestAppContext) {
    let (host, _, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        view.set_capabilities(crate::toolbar::DrawBrushCapabilities::NONE, cx);
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    for selector in ["draw-size", "draw-color", "draw-smoothing"] {
        assert!(
            cx.debug_bounds(selector).is_none(),
            "{selector} should be hidden"
        );
    }
}

#[gpui::test]
fn prototype_operations_keep_connection_identity(cx: &mut TestAppContext) {
    let (_host, actions, cx) = mount_component(cx, |_, cx| {
        PrototypeInspector::new(
            "prototype",
            PrototypeInspectorViewData {
                selection_name: "Frame".into(),
                connections: vec![PrototypeConnection {
                    id: "opaque-id".into(),
                    trigger: "Click".into(),
                    action: "Navigate".into(),
                    destination: "Detail".into(),
                    animation: "Dissolve".into(),
                }],
                ..Default::default()
            },
            cx,
        )
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "prototype-edit-opaque-id",
        &actions,
        PrototypeInspectorAction::ConnectionEditRequested {
            id: "opaque-id".into(),
        },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "prototype-remove-opaque-id",
        &actions,
        PrototypeInspectorAction::ConnectionRemoveRequested {
            id: "opaque-id".into(),
        },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "prototype-add",
        &actions,
        PrototypeInspectorAction::ConnectionAddRequested,
    );
}

#[gpui::test]
fn prototype_play_header_action_respects_host_availability(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        PrototypeInspector::new("prototype", PrototypeInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "prototype-present",
        &actions,
        PrototypeInspectorAction::PresentRequested,
    );
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        let mut data = view.view_data().clone();
        data.can_present = false;
        view.set_view_data(data, cx);
    });
    cx.run_until_parked();
    let bounds = cx
        .debug_bounds("prototype-present")
        .expect("Present button should render");
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn motion_timeline_action_is_only_rendered_when_host_can_open_it(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        MotionInspector::new("motion", MotionInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "motion-timeline",
        &actions,
        MotionInspectorAction::TimelineOpenRequested,
    );
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        let mut data = view.view_data().clone();
        data.timeline_open_available = false;
        view.set_view_data(data, cx);
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("motion-timeline").is_none());
}

#[gpui::test]
fn comments_resolve_and_reply_target_the_supplied_thread(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        CommentsInspector::new(
            "comments",
            CommentsInspectorViewData {
                can_comment: true,
                threads: vec![InspectorCommentThread {
                    id: "thread-7".into(),
                    location: "Frame 2".into(),
                    resolved: false,
                    unread: true,
                    comments: vec![InspectorComment {
                        id: "comment-1".into(),
                        author: "Jane Doe".into(),
                        time_label: "Now".into(),
                        body: "Check this spacing".into(),
                    }],
                }],
                ..Default::default()
            },
            cx,
        )
    });
    cx.simulate_resize(size(px(360.), px(900.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "comments-resolve-thread-7",
        &actions,
        CommentsInspectorAction::ResolveChangeRequested {
            id: "thread-7".into(),
            resolved: true,
        },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "comments-thread-thread-7",
        &actions,
        CommentsInspectorAction::ThreadSelectRequested {
            id: "thread-7".into(),
        },
    );
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| {
        let mut data = view.view_data().clone();
        data.selected_thread = Some("thread-7".into());
        view.set_view_data(data, cx);
    });
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "comments-new-comment",
        &actions,
        CommentsInspectorAction::ThreadClearRequested,
    );
}

#[gpui::test]
fn standalone_tabs_keep_controls_inside_compact_sidebar(cx: &mut TestAppContext) {
    let (_host, _actions, cx) = mount_component(cx, |_, cx| {
        MotionInspector::new(
            "compact",
            MotionInspectorViewData {
                selection_name: "A very long layer name that should truncate in the inspector"
                    .into(),
                ..Default::default()
            },
            cx,
        )
    });
    cx.simulate_resize(size(px(320.), px(900.)));
    cx.run_until_parked();
    for selector in [
        "compact-duration",
        "compact-delay",
        "compact-play",
        "compact-auto",
        "compact-timeline",
    ] {
        let bounds = cx.debug_bounds(selector).unwrap();
        assert!(
            bounds.left() >= px(0.) && bounds.right() <= px(320.),
            "{selector} escaped sidebar: {bounds:?}"
        );
    }
}

#[gpui::test]
fn compact_draw_parameters_and_tip_controls_fit_a_320_pixel_sidebar(cx: &mut TestAppContext) {
    let (_host, _, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(320.), px(720.)));
    cx.run_until_parked();
    for selector in [
        "draw-size",
        "draw-size-slider",
        "draw-hardness",
        "draw-hardness-slider",
        "draw-opacity",
        "draw-opacity-slider",
        "draw-flow",
        "draw-flow-slider",
        "draw-smoothing",
        "draw-smoothing-slider",
        "draw-tip",
        "draw-tip-sample",
        "draw-save-preset",
        "draw-pressure",
        "draw-antialias",
    ] {
        let bounds = cx.debug_bounds(selector).unwrap();
        assert!(
            bounds.left() >= px(0.) && bounds.right() <= px(320.),
            "{selector} escaped sidebar: {bounds:?}"
        );
        assert!(
            bounds.top() >= px(0.) && bounds.bottom() <= px(720.),
            "{selector} escaped sidebar: {bounds:?}"
        );
    }
}

#[gpui::test]
fn draw_opacity_slider_requests_bounded_values_without_accepting_them(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        DrawInspector::new("draw", DrawInspectorViewData::default(), cx)
    });
    cx.simulate_resize(size(px(320.), px(720.)));
    cx.run_until_parked();
    let bounds = cx.debug_bounds("draw-opacity-slider").unwrap();
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&DrawInspectorAction::OptionsChangeRequested {
            options: crate::toolbar::DrawToolbarOptions {
                opacity: 0,
                ..Default::default()
            }
        })
    );
    let component = cx.read(|app| host.read(app).component.clone());
    component.read_with(cx, |view, _| {
        assert_eq!(view.view_data().options.opacity, 100)
    });
    component.update(cx, |view, cx| {
        let mut data = view.view_data().clone();
        data.read_only = true;
        view.set_view_data(data, cx);
    });
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn composed_prototype_can_delegate_play_to_the_shared_header(cx: &mut TestAppContext) {
    let (host, _, cx) = mount_component(cx, |_, cx| {
        PrototypeInspector::new("prototype", PrototypeInspectorViewData::default(), cx)
    });
    let component = cx.read(|app| host.read(app).component.clone());
    component.update(cx, |view, cx| view.set_show_presentation_action(false, cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("prototype-present").is_none());
}
