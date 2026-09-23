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
fn soft_round_preset_requests_a_soft_brush_without_mutating_host_state(cx: &mut TestAppContext) {
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
    assert_pointer_and_keyboard_parity(
        cx,
        "draw-preset-Soft round",
        &actions,
        DrawInspectorAction::OptionsChangeRequested { options },
    );
    cx.read(|app| {
        let data = host.read(app).component.read(app).view_data();
        assert_eq!(data.options.brush_tip.as_ref(), "Round");
        assert_eq!(data.options.hardness, 100);
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
