use super::*;
use std::{cell::RefCell, rc::Rc};

use gpui::{
    Entity, IntoElement, Keystroke, Render, Subscription, TestAppContext, VisualTestContext,
    Window, div, px,
};
use gpui_component::Root;

struct PickerTestHost {
    picker: Entity<PaintPicker>,
    events: Rc<RefCell<Vec<PaintPickerEvent>>>,
    ancestor_scrolls: Rc<RefCell<usize>>,
    _subscription: Subscription,
}

impl PickerTestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let picker = cx.new(|cx| PaintPicker::new("paint-picker-test", window, cx));
        let events = Rc::new(RefCell::new(Vec::new()));
        let captured_events = events.clone();
        let subscription = cx.subscribe(&picker, move |_, _, event: &PaintPickerEvent, _| {
            captured_events.borrow_mut().push(event.clone());
        });
        Self {
            picker,
            events,
            ancestor_scrolls: Rc::new(RefCell::new(0)),
            _subscription: subscription,
        }
    }
}

impl Render for PickerTestHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let scrolls = self.ancestor_scrolls.clone();
        div()
            .id("picker-test-host")
            .w(px(320.))
            .h(px(720.))
            .on_scroll_wheel(move |_, _, _| *scrolls.borrow_mut() += 1)
            .child(self.picker.clone())
    }
}

fn setup_picker(cx: &mut TestAppContext) -> (Entity<PickerTestHost>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let host_slot = Rc::new(RefCell::new(None));
    let captured_host = host_slot.clone();
    let (_, visual_cx) = cx.add_window_view(move |window, cx| {
        let host = cx.new(|cx| PickerTestHost::new(window, cx));
        *captured_host.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    let host = host_slot
        .borrow_mut()
        .take()
        .expect("Paint picker test host should be installed");
    visual_cx.run_until_parked();
    (host, visual_cx)
}

fn picker(host: &Entity<PickerTestHost>, cx: &VisualTestContext) -> Entity<PaintPicker> {
    cx.read(|app| host.read(app).picker.clone())
}

fn picker_events(
    host: &Entity<PickerTestHost>,
    cx: &VisualTestContext,
) -> Rc<RefCell<Vec<PaintPickerEvent>>> {
    cx.read(|app| host.read(app).events.clone())
}

fn gradient() -> DesignPaint {
    DesignPaint::gradient(
        DesignPaintKind::RadialGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK),
            DesignGradientStop::new(1., DesignColor::WHITE),
        ],
    )
}

fn edit_phases(events: &[PaintPickerEvent]) -> Vec<DesignPanelEditPhase> {
    events
        .iter()
        .filter_map(|event| match event {
            PaintPickerEvent::Edit { phase, .. } => Some(*phase),
            _ => None,
        })
        .collect()
}

#[test]
fn nested_overlay_coordinator_dismisses_only_the_topmost_surface() {
    let mut overlays = PaintPickerOverlayCoordinator::default();
    overlays.open(PaintPickerOverlay::ColorFormat, None);
    overlays.open(PaintPickerOverlay::Creation, None);

    assert_eq!(overlays.topmost(), Some(PaintPickerOverlay::Creation));
    assert!(
        overlays
            .dismissal_intent(
                PaintPickerOverlay::ColorFormat,
                InspectorOverlayDismissCause::OutsideClick,
            )
            .is_none(),
        "a covered surface must not react to the topmost surface's outside click"
    );

    let intent = overlays
        .dismissal_intent(
            PaintPickerOverlay::Creation,
            InspectorOverlayDismissCause::Escape,
        )
        .expect("the topmost surface should accept Escape");
    assert_eq!(intent.cause(), InspectorOverlayDismissCause::Escape);
    overlays.finish_dismissal(&intent);

    assert!(!overlays.is_open(PaintPickerOverlay::Creation));
    assert!(overlays.is_open(PaintPickerOverlay::ColorFormat));
    assert_eq!(overlays.topmost(), Some(PaintPickerOverlay::ColorFormat));
}

#[gpui::test]
fn nested_overlay_dismissal_unwinds_focus_in_stack_order(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
            picker.focus_handle.focus(window, cx);
            picker.open_color_format_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.update(|window, app| {
        picker
            .read(app)
            .color_format_menu_focus_handle
            .is_focused(window)
    }));

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.open_creation_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.update(|window, app| {
        picker
            .read(app)
            .creation_menu_focus_handle
            .is_focused(window)
    }));

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            assert!(!picker.dismiss_nested_overlay(
                PaintPickerOverlay::ColorFormat,
                InspectorOverlayDismissCause::OutsideClick,
                window,
                cx,
            ));
            assert!(picker.dismiss_nested_overlay(
                PaintPickerOverlay::Creation,
                InspectorOverlayDismissCause::Escape,
                window,
                cx,
            ));
        });
    });
    visual_cx.run_until_parked();
    visual_cx.read(|app| {
        let picker = picker.read(app);
        assert!(picker.nested_overlay_is_open(PaintPickerOverlay::ColorFormat));
        assert!(!picker.nested_overlay_is_open(PaintPickerOverlay::Creation));
    });
    assert!(visual_cx.update(|window, app| {
        picker
            .read(app)
            .color_format_menu_focus_handle
            .is_focused(window)
    }));

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            assert!(picker.dismiss_nested_overlay(
                PaintPickerOverlay::ColorFormat,
                InspectorOverlayDismissCause::OutsideClick,
                window,
                cx,
            ));
        });
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.update(|window, app| { picker.read(app).focus_handle.is_focused(window) }));
}

#[test]
fn active_crop_rotation_previews_an_arbitrary_affine_angle() {
    let crop_tool = DesignMediaCropToolState {
        active: true,
        transform: DesignPaintTransform::IDENTITY,
        zoom: 2.5,
        aspect_ratio: crate::design::DesignMediaCropAspectRatio::Square,
    };

    let DesignMediaCropAction::Preview {
        transform,
        zoom,
        aspect_ratio,
    } = rotated_crop_preview(crop_tool)
    else {
        panic!("crop rotation must remain a host-controlled preview");
    };

    let radians = CROP_ROTATION_STEP_DEGREES.to_radians();
    assert!((transform.m11 - radians.cos()).abs() < 0.0001);
    assert!((transform.m12 - radians.sin()).abs() < 0.0001);
    assert!((transform.m21 + radians.sin()).abs() < 0.0001);
    assert!((transform.m22 - radians.cos()).abs() < 0.0001);
    assert_eq!((transform.tx, transform.ty), (0., 0.));
    assert_eq!(zoom, crop_tool.zoom);
    assert_eq!(aspect_ratio, crop_tool.aspect_ratio);
}

#[gpui::test]
fn hex_enter_blur_and_retained_focus_each_have_one_complete_transaction(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
        });
    });
    let hex_input = visual_cx.read(|app| picker.read(app).hex_input.clone());
    visual_cx.update(|window, app| {
        hex_input.focus_handle(app).focus(window, app);
    });
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        hex_input.update(app, |input, cx| {
            input.set_value("112233", window, cx);
        });
    });
    visual_cx.run_until_parked();

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.handle_hex_input(&InputEvent::PressEnter { secondary: false }, window, cx);
            // GPUI keeps the field focused after Enter. This deferred Blur
            // is deliberately delivered and must not create a second
            // terminal.
            picker.handle_hex_input(&InputEvent::Blur, window, cx);
        });
    });
    visual_cx.run_until_parked();

    // Typing again while focus is retained starts a new Begin before the
    // next Preview; Blur then supplies that session's sole terminal.
    visual_cx.update(|window, app| {
        hex_input.update(app, |input, cx| {
            input.set_value("445566", window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.handle_hex_input(&InputEvent::Blur, window, cx);
        });
    });
    visual_cx.run_until_parked();

    assert_eq!(
        edit_phases(events.borrow().as_slice()),
        [
            DesignPanelEditPhase::Begin,
            DesignPanelEditPhase::Preview,
            DesignPanelEditPhase::Commit,
            DesignPanelEditPhase::Begin,
            DesignPanelEditPhase::Preview,
            DesignPanelEditPhase::Commit,
        ]
    );
}

#[gpui::test]
fn unchanged_and_invalid_text_edits_still_terminate_their_begin(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
        });
    });
    let hex_input = visual_cx.read(|app| picker.read(app).hex_input.clone());
    visual_cx.update(|window, app| {
        hex_input.focus_handle(app).focus(window, app);
    });
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.handle_hex_input(&InputEvent::Focus, window, cx);
            picker.handle_hex_input(&InputEvent::Blur, window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert_eq!(
        edit_phases(events.borrow().as_slice()),
        [DesignPanelEditPhase::Begin, DesignPanelEditPhase::Commit,]
    );

    events.borrow_mut().clear();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.handle_hex_input(&InputEvent::Focus, window, cx);
        });
    });
    visual_cx.update(|window, app| {
        hex_input.update(app, |input, cx| {
            input.set_value("invalid", window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.handle_hex_input(&InputEvent::PressEnter { secondary: false }, window, cx);
            picker.handle_hex_input(&InputEvent::Blur, window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert_eq!(
        edit_phases(events.borrow().as_slice()),
        [DesignPanelEditPhase::Begin, DesignPanelEditPhase::Cancel,]
    );
}

#[gpui::test]
fn rgba_text_edits_share_one_transaction_across_solid_color_and_opacity(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
        });
    });
    let input = visual_cx.read(|app| picker.read(app).hex_input.clone());
    visual_cx.update(|window, app| input.focus_handle(app).focus(window, app));
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        input.update(app, |input, cx| {
            input.set_value("FF000080", window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.handle_hex_input(&InputEvent::PressEnter { secondary: false }, window, cx);
            picker.handle_hex_input(&InputEvent::Blur, window, cx);
        });
    });
    visual_cx.run_until_parked();

    let events = events.borrow();
    assert_eq!(
        edit_phases(events.as_slice()),
        [
            DesignPanelEditPhase::Begin,
            DesignPanelEditPhase::Preview,
            DesignPanelEditPhase::Preview,
            DesignPanelEditPhase::Commit,
        ]
    );
    assert!(matches!(
        events.get(1),
        Some(PaintPickerEvent::Edit { edit, .. })
            if edit.property == DesignPaintProperty::Color
    ));
    assert!(matches!(
        events.get(2),
        Some(PaintPickerEvent::Edit { edit, .. })
            if edit.property == DesignPaintProperty::Opacity
    ));
    assert!(matches!(
        events.get(3),
        Some(PaintPickerEvent::Edit { edit, .. })
            if edit.property == DesignPaintProperty::Opacity
    ));
}

#[gpui::test]
fn blend_mode_and_color_style_sample_emit_typed_atomic_intents(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
            picker.set_color_style_sample_view_data(
                DesignColorStyleSampleViewData::new(
                    [DesignColorStyleSample::new(
                        "page-brand",
                        "Brand",
                        DesignColor::PURPLE,
                    )],
                    [],
                ),
                cx,
            );
            picker.select_blend_mode(DesignBlendMode::Multiply, window, cx);
            picker.request_color_style_sample(
                DesignColorStyleSampleSelection::page("page-brand"),
                cx,
            );
        });
    });
    visual_cx.run_until_parked();

    let events = events.borrow();
    assert!(matches!(
        events.first(),
        Some(PaintPickerEvent::Edit {
            edit,
            phase: DesignPanelEditPhase::Commit,
            ..
        }) if edit.property == DesignPaintProperty::BlendMode
            && edit.value == DesignPaintValue::BlendMode(DesignBlendMode::Multiply)
    ));
    assert!(matches!(
        events.get(1),
        Some(PaintPickerEvent::ColorStyleSampleRequested {
            color_target: DesignPaintColorTarget::Solid,
            sample,
            ..
        }) if sample == &DesignColorStyleSampleSelection::page("page-brand")
    ));
}

#[gpui::test]
fn color_only_editability_gates_color_and_opacity_independently(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::LayoutGrid,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("guide"),
                window,
                cx,
            );
            picker.set_color_only(Some("Guide color".into()), cx);
            picker.set_color_only_editability(false, true, cx);
            assert_eq!(picker.color_only_editability(), (false, true));
            assert!(picker.color_editing_disabled());
            assert!(!picker.opacity_editing_disabled());

            picker.set_color_only_editability(true, false, cx);
            assert_eq!(picker.color_only_editability(), (true, false));
            assert!(!picker.color_editing_disabled());
            assert!(picker.opacity_editing_disabled());
        });
    });
}

#[test]
fn picker_recognizes_figma_eyedropper_shortcuts() {
    let i = KeyDownEvent {
        keystroke: Keystroke::parse("i").expect("I shortcut"),
        is_held: false,
        prefer_character_input: false,
    };
    let modified_i = KeyDownEvent {
        keystroke: Keystroke::parse("shift-i").expect("modified I shortcut"),
        is_held: false,
        prefer_character_input: false,
    };
    let control_c = KeyDownEvent {
        keystroke: Keystroke::parse("ctrl-c").expect("macOS Control-C shortcut"),
        is_held: false,
        prefer_character_input: false,
    };
    assert!(is_eyedropper_shortcut(&i));
    assert!(!is_eyedropper_shortcut(&modified_i));
    assert_eq!(
        is_eyedropper_shortcut(&control_c),
        cfg!(target_os = "macos")
    );
}

#[gpui::test]
fn color_format_menu_roves_home_end_wraps_and_restores_focus(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
            picker.focus_handle.focus(window, cx);
            picker.open_color_format_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.update(|window, app| {
        picker
            .read(app)
            .color_format_menu_focus_handle
            .is_focused(window)
    }));

    visual_cx.simulate_keystrokes("down");
    assert_eq!(
        visual_cx.read(|app| picker.read(app).color_format_menu_index),
        ColorFormat::Rgb.index()
    );
    visual_cx.simulate_keystrokes("end");
    assert_eq!(
        visual_cx.read(|app| picker.read(app).color_format_menu_index),
        ColorFormat::Hsb.index()
    );
    visual_cx.simulate_keystrokes("down");
    assert_eq!(
        visual_cx.read(|app| picker.read(app).color_format_menu_index),
        ColorFormat::Hex.index()
    );
    visual_cx.simulate_keystrokes("up");
    assert_eq!(
        visual_cx.read(|app| picker.read(app).color_format_menu_index),
        ColorFormat::Hsb.index()
    );
    visual_cx.simulate_keystrokes("home");
    assert_eq!(
        visual_cx.read(|app| picker.read(app).color_format_menu_index),
        ColorFormat::Hex.index()
    );
    visual_cx.simulate_keystrokes("end enter");
    visual_cx.run_until_parked();
    visual_cx.read(|app| {
        let picker = picker.read(app);
        assert_eq!(picker.color_format, ColorFormat::Hsb);
        assert!(!picker.nested_overlay_is_open(PaintPickerOverlay::ColorFormat));
    });
    assert!(visual_cx.update(|window, app| { picker.read(app).focus_handle.is_focused(window) }));

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.open_color_format_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.simulate_keystrokes("home escape");
    visual_cx.run_until_parked();
    visual_cx.read(|app| {
        let picker = picker.read(app);
        assert_eq!(picker.color_format, ColorFormat::Hsb);
        assert!(!picker.nested_overlay_is_open(PaintPickerOverlay::ColorFormat));
    });
    assert!(visual_cx.update(|window, app| { picker.read(app).focus_handle.is_focused(window) }));
}

#[gpui::test]
fn creation_menu_roves_and_preserves_the_exact_gradient_stop_target(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    let paint = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::PURPLE).with_id("start"),
            DesignGradientStop::new(1., DesignColor::WHITE).with_id("end"),
        ],
    )
    .with_id("gradient");
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target("node", DesignPanelCollection::Fill, 0, paint, window, cx);
            picker.focus_handle.focus(window, cx);
            picker.open_creation_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.update(|window, app| {
        picker
            .read(app)
            .creation_menu_focus_handle
            .is_focused(window)
    }));

    visual_cx.simulate_keystrokes("enter");
    visual_cx.run_until_parked();
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::PaintStyleCreateRequested {
            target,
            color_target: DesignPaintColorTarget::GradientStop {
                stop_id,
                index: 0,
            },
        }] if target.node_id.as_ref() == "node"
            && target.paint_id.as_ref() == "gradient"
            && target.index == 0
            && stop_id.as_ref() == "start"
    ));
    assert!(visual_cx.update(|window, app| { picker.read(app).focus_handle.is_focused(window) }));

    events.borrow_mut().clear();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.open_creation_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.simulate_keystrokes("down enter");
    visual_cx.run_until_parked();
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::ColorVariableCreateRequested {
            target,
            color_target: DesignPaintColorTarget::GradientStop {
                stop_id,
                index: 0,
            },
            color,
        }] if target.node_id.as_ref() == "node"
            && target.paint_id.as_ref() == "gradient"
            && stop_id.as_ref() == "start"
            && *color == DesignColor::PURPLE
    ));
}

#[gpui::test]
fn palette_scope_selector_uses_stable_library_ids_and_resets_stale_sources(
    cx: &mut TestAppContext,
) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
            picker.set_color_style_sample_view_data(
                DesignColorStyleSampleViewData::new(
                    [DesignColorStyleSample::new(
                        "page",
                        "Page",
                        DesignColor::BLACK,
                    )],
                    [crate::design::DesignColorStyleSampleLibrary::new(
                        "shared",
                        "Shared library",
                        [DesignColorStyleSample::new(
                            "brand",
                            "Brand",
                            DesignColor::PURPLE,
                        )],
                    )],
                ),
                cx,
            );
            picker.set_paint_variable_view_data(
                DesignPaintVariableViewData::new([
                    DesignVariable::page(
                        "shared-variable",
                        "Shared",
                        "colors",
                        "Colors",
                        crate::design::DesignVariableResolvedType::Color,
                    )
                    .with_source(DesignVariableSource::library("shared", "Renamed duplicate")),
                    DesignVariable::page(
                        "other-variable",
                        "Other",
                        "colors",
                        "Colors",
                        crate::design::DesignVariableResolvedType::Color,
                    )
                    .with_source(DesignVariableSource::library("other", "Other library")),
                ]),
                cx,
            );
            picker.focus_handle.focus(window, cx);
            picker.open_resource_scope_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.read(|app| {
        let picker = picker.read(app);
        let scopes = picker.resource_scopes();
        assert_eq!(scopes.len(), 3, "library IDs are de-duplicated");
        assert!(matches!(
            &scopes[1],
            PaintResourceScope::Library {
                library_id,
                library_name,
            } if library_id.as_ref() == "shared"
                && library_name.as_ref() == "Shared library"
        ));
    });

    visual_cx.simulate_keystrokes("down enter");
    visual_cx.run_until_parked();
    assert!(visual_cx.read(|app| matches!(
        &picker.read(app).resource_scope,
        PaintResourceScope::Library { library_id, .. }
            if library_id.as_ref() == "shared"
    )));

    picker.update(visual_cx, |picker, cx| {
        picker.set_paint_variable_view_data(
            DesignPaintVariableViewData::new([DesignVariable::page(
                "page-variable",
                "Page",
                "colors",
                "Colors",
                crate::design::DesignVariableResolvedType::Color,
            )]),
            cx,
        );
        picker.set_color_style_sample_view_data(DesignColorStyleSampleViewData::default(), cx);
    });
    visual_cx.run_until_parked();
    visual_cx.read(|app| {
        let picker = picker.read(app);
        assert_eq!(picker.resource_scope, PaintResourceScope::Page);
        assert!(!picker.nested_overlay_is_open(PaintPickerOverlay::ResourceScope));
    });
}

#[test]
fn contrast_checker_uses_wcag_luminance_and_alpha_compositing() {
    assert!(
        (wcag_contrast_ratio(DesignColor::rgb(0, 0, 0), DesignColor::WHITE) - 21.).abs() < 0.001
    );
    assert!((wcag_contrast_ratio(DesignColor::WHITE, DesignColor::WHITE) - 1.).abs() < 0.001);
    let translucent_black =
        wcag_contrast_ratio(DesignColor::rgba(0, 0, 0, 0x80), DesignColor::WHITE);
    assert!(translucent_black > 3.9 && translucent_black < 4.1);
    assert_eq!(
        composite_color(
            DesignColor::rgba(0xff, 0x00, 0x00, 0x80),
            DesignColor::WHITE
        ),
        DesignColor::rgb(0xff, 0x7f, 0x7f)
    );
}

#[gpui::test]
fn contrast_checker_uses_host_ratio_mode_and_correction_without_predicting_paint(
    cx: &mut TestAppContext,
) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    let paint = DesignPaint::solid(DesignColor::rgb(0xa0, 0xae, 0xc0)).with_id("fill");
    let view_data = DesignColorContrastPaintViewData::new(
        crate::design::DesignColorContrastPaintTarget::paint(
            "node",
            DesignPanelCollection::Fill,
            "fill",
            0,
        ),
        [DesignColorContrastLeafViewData::new(
            DesignPaintColorTarget::Solid,
            DesignColor::WHITE,
            2.19,
            DesignColorContrastCategory::Graphics,
        )
        .with_corrections([crate::design::DesignColorContrastCorrection::new(
            DesignColorContrastCategory::Graphics,
            DesignColorContrastLevel::Aa,
            DesignColor::rgb(0, 0, 0),
        )])],
    );

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                paint.clone(),
                window,
                cx,
            );
            picker.set_contrast_view_data(Some(view_data), cx);
            picker.contrast_level = DesignColorContrastLevel::Aaa;
            assert_eq!(
                picker.resolved_contrast_category(),
                Some(DesignColorContrastCategory::Graphics)
            );
            assert_eq!(
                picker.normalized_contrast_level(),
                DesignColorContrastLevel::Aa
            );
            picker.apply_contrast_correction(cx);
            assert_eq!(picker.paint(), Some(&paint));
        });
    });

    let events = events.borrow();
    assert_eq!(events.len(), 1);
    let PaintPickerEvent::Edit {
        target,
        edit,
        phase,
    } = &events[0]
    else {
        panic!("contrast correction should use the ordinary paint edit event");
    };
    assert_eq!(target.node_id.as_ref(), "node");
    assert_eq!(target.paint_id.as_ref(), "fill");
    assert_eq!(*phase, DesignPanelEditPhase::Commit);
    assert_eq!(edit.property, DesignPaintProperty::Color);
    assert_eq!(
        edit.value,
        DesignPaintValue::Color(DesignColor::rgb(0, 0, 0))
    );
}

#[gpui::test]
fn blend_mode_hover_preview_is_balanced_and_commit_cleans_it_up(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    let paint = DesignPaint::solid(DesignColor::BLUE).with_id("stable-fill");

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                paint.clone(),
                window,
                cx,
            );
            picker.set_blend_mode_preview(DesignBlendMode::Multiply, true, cx);
            picker.set_blend_mode_preview(DesignBlendMode::Multiply, true, cx);
            picker.select_blend_mode(DesignBlendMode::Multiply, window, cx);
            picker.cancel_blend_mode_preview(cx);
        });
    });
    visual_cx.run_until_parked();

    let events = events.borrow();
    assert!(matches!(
        events.as_slice(),
        [
            PaintPickerEvent::BlendModePreview {
                phase: DesignMenuPreviewPhase::Begin,
                original: DesignBlendMode::Normal,
                candidate: DesignBlendMode::Multiply,
                ..
            },
            PaintPickerEvent::BlendModePreview {
                phase: DesignMenuPreviewPhase::End,
                original: DesignBlendMode::Normal,
                candidate: DesignBlendMode::Multiply,
                ..
            },
            PaintPickerEvent::Edit {
                edit,
                phase: DesignPanelEditPhase::Commit,
                ..
            }
        ] if edit.property == DesignPaintProperty::BlendMode
            && edit.value == DesignPaintValue::BlendMode(DesignBlendMode::Multiply)
    ));
    assert_eq!(
        visual_cx.read(|app| picker.read(app).paint().map(|paint| paint.blend_mode)),
        Some(DesignBlendMode::Normal),
        "menu previews and commits must not mutate the controlled picker snapshot"
    );
}

#[gpui::test]
fn blend_mode_preview_ends_once_when_exact_paint_index_or_permission_changes(
    cx: &mut TestAppContext,
) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    let paint = DesignPaint::solid(DesignColor::PURPLE).with_id("stable-fill");

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                paint.clone(),
                window,
                cx,
            );
            picker.set_blend_mode_preview(DesignBlendMode::Screen, true, cx);
            picker.set_target("node", DesignPanelCollection::Fill, 1, paint, window, cx);
            picker.set_disabled(true, cx);
            picker.set_blend_mode_preview(DesignBlendMode::Multiply, true, cx);
            picker.prepare_for_dismissal(cx);
        });
    });
    visual_cx.run_until_parked();

    let previews = events
        .borrow()
        .iter()
        .filter_map(|event| match event {
            PaintPickerEvent::BlendModePreview {
                target,
                phase,
                candidate,
                ..
            } => Some((target.index, *phase, *candidate)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        previews,
        vec![
            (0, DesignMenuPreviewPhase::Begin, DesignBlendMode::Screen),
            (0, DesignMenuPreviewPhase::End, DesignBlendMode::Screen),
        ]
    );
}

#[test]
fn contrast_palette_boundary_tracks_both_wcag_crossing_branches() {
    let crossings =
        contrast_value_crossings(210., 0.7, u8::MAX, DesignColor::rgb(0x77, 0x77, 0x77), 3.);
    assert!(crossings.len() <= 2);
    assert!(crossings.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(crossings.iter().all(|value| (0. ..=1.).contains(value)));
}

#[test]
fn hex_parser_accepts_three_four_six_and_eight_digit_forms() {
    assert_eq!(
        parse_hex_color("#0af"),
        Some(DesignColor::rgb(0x00, 0xaa, 0xff))
    );
    assert_eq!(
        parse_hex_color("#0af8"),
        Some(DesignColor::rgba(0x00, 0xaa, 0xff, 0x88))
    );
    assert_eq!(
        parse_hex_color("4C86F7"),
        Some(DesignColor::rgb(0x4c, 0x86, 0xf7))
    );
    assert_eq!(
        parse_hex_color("4c86f780"),
        Some(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80))
    );
    assert!(parse_hex_color_input("4c86f7").is_some_and(|parsed| !parsed.explicit_alpha));
    assert!(parse_hex_color_input("4c86f780").is_some_and(|parsed| parsed.explicit_alpha));
    assert_eq!(parse_hex_color("12"), None);
    assert_eq!(parse_hex_color("12345"), None);
    assert_eq!(parse_hex_color("nope"), None);
}

#[test]
fn hex_and_css_serializers_round_trip_alpha() {
    assert_eq!(rgb_hex(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80)), "4C86F7");
    assert_eq!(
        rgba_hex(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80)),
        "4C86F780"
    );
    assert_eq!(
        parse_css_color_input("rgba(76, 134, 247, 0.502)").map(|parsed| parsed.color),
        Some(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80))
    );
    let css = css_rgba(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80));
    assert_eq!(
        parse_css_color_input(&css).map(|parsed| parsed.color),
        Some(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80))
    );
    assert!(parse_css_color_input("rgb(76, 134, 247)").is_none());
    assert!(parse_css_color_input("rgba(76, 134, 247, 2)").is_none());
}

#[test]
fn color_channel_conversions_cover_rgb_hsl_hsb_and_validation() {
    let source = DesignColor::rgb(0x4c, 0x86, 0xf7);
    for format in [ColorFormat::Rgb, ColorFormat::Hsl, ColorFormat::Hsb] {
        let values = color_channel_values(format, source);
        let round_trip = parse_color_channels(format, [&values[0], &values[1], &values[2]])
            .expect("formatted channels");
        for (actual, expected) in [
            (round_trip.red, source.red),
            (round_trip.green, source.green),
            (round_trip.blue, source.blue),
        ] {
            assert!(actual.abs_diff(expected) <= 1);
        }
    }

    assert_eq!(
        parse_color_channels(ColorFormat::Hsl, ["0", "100", "50"]),
        Some(DesignColor::rgb(0xff, 0x00, 0x00))
    );
    assert_eq!(
        parse_color_channels(ColorFormat::Hsb, ["120", "100", "100"]),
        Some(DesignColor::rgb(0x00, 0xff, 0x00))
    );
    assert!(parse_color_channels(ColorFormat::Rgb, ["256", "0", "0"]).is_none());
    assert!(parse_color_channels(ColorFormat::Hsl, ["361", "50", "50"]).is_none());
    assert!(parse_color_channels(ColorFormat::Hsb, ["0", "101", "50"]).is_none());
    assert!(parse_color_channels(ColorFormat::Rgb, ["NaN", "0", "0"]).is_none());
    assert!(parse_color_channels(ColorFormat::Css, ["0", "0", "0"]).is_none());
}

#[test]
fn formatted_channel_edits_target_the_active_leaf_and_preserve_alpha() {
    let rgb =
        parse_color_channels(ColorFormat::Hsl, ["220", "91", "63"]).expect("valid HSL channels");
    let mut gradient = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK).with_id("start"),
            DesignGradientStop::new(1., DesignColor::rgba(0xff, 0xff, 0xff, 0x70)).with_id("end"),
        ],
    );
    gradient.opacity = 58.;
    let color = rgb_edit_color(&gradient, 1, rgb);
    let edit = color_edit(&gradient, 1, color);
    assert_eq!(
        edit.property,
        DesignPaintProperty::GradientStopColor {
            stop_id: "end".into(),
            index: 1,
        }
    );
    assert_eq!(
        edit.value,
        DesignPaintValue::Color(DesignColor { alpha: 0x70, ..rgb })
    );
    assert!(gradient.apply_edit(&edit));
    assert_eq!(gradient.opacity, 58.);
    assert_eq!(gradient.gradient_stops[0].color, DesignColor::BLACK);
    assert_eq!(gradient.gradient_stops[1].color.alpha, 0x70);

    let mut solid = DesignPaint::solid(DesignColor::BLACK);
    solid.opacity = 42.;
    let color = rgb_edit_color(&solid, 0, rgb);
    let edit = color_edit(&solid, 0, color);
    assert_eq!(edit.property, DesignPaintProperty::Color);
    assert!(solid.apply_edit(&edit));
    assert_eq!(solid.opacity, 42.);
    assert_eq!(solid.color.alpha, u8::MAX);
}

#[test]
fn solid_picker_uses_paint_opacity_for_both_rail_and_percentage() {
    let mut paint = DesignPaint::solid(DesignColor::rgba(0x12, 0x34, 0x56, 0x20));
    paint.opacity = 37.5;

    assert_eq!(picker_opacity(&paint, 0), 37.5);
    let edit = picker_opacity_edit(&paint, 0, 62.);
    assert_eq!(edit.property, DesignPaintProperty::Opacity);
    assert_eq!(edit.value, DesignPaintValue::Number(62.));
    assert!(paint.apply_edit(&edit));
    assert_eq!(paint.opacity, 62.);
    assert_eq!(paint.color.alpha, 0x20);
}

#[test]
fn gradient_picker_uses_only_the_selected_stop_alpha() {
    let mut paint = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::rgba(0x12, 0x34, 0x56, 0x40)).with_id("start"),
            DesignGradientStop::new(1., DesignColor::rgba(0xaa, 0xbb, 0xcc, 0xc0)).with_id("end"),
        ],
    );
    paint.opacity = 33.;

    assert!((picker_opacity(&paint, 1) - f32::from(0xc0_u8) / 255. * 100.).abs() < 0.001);
    let edit = picker_opacity_edit(&paint, 1, 25.);
    assert_eq!(
        edit.property,
        DesignPaintProperty::GradientStopColor {
            stop_id: "end".into(),
            index: 1,
        }
    );
    assert_eq!(
        edit.value,
        DesignPaintValue::Color(DesignColor::rgba(0xaa, 0xbb, 0xcc, 0x40))
    );
    assert!(paint.apply_edit(&edit));
    assert_eq!(paint.opacity, 33.);
    assert_eq!(paint.gradient_stops[0].color.alpha, 0x40);
    assert_eq!(paint.gradient_stops[1].color.alpha, 0x40);
}

#[test]
fn six_digit_rgb_edits_preserve_the_applicable_opacity() {
    let parsed = parse_hex_color("4C86F7").expect("six-digit RGB");

    let mut solid = DesignPaint::solid(DesignColor::rgba(0x12, 0x34, 0x56, 0x20));
    solid.opacity = 42.;
    let solid_color = rgb_edit_color(&solid, 0, parsed);
    assert_eq!(solid_color, DesignColor::rgb(0x4c, 0x86, 0xf7));
    let solid_edit = color_edit(&solid, 0, solid_color);
    assert!(solid.apply_edit(&solid_edit));
    assert_eq!(solid.opacity, 42.);

    let mut gradient = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK).with_id("start"),
            DesignGradientStop::new(1., DesignColor::rgba(0xff, 0xff, 0xff, 0x70)).with_id("end"),
        ],
    );
    gradient.opacity = 58.;
    let stop_color = rgb_edit_color(&gradient, 1, parsed);
    assert_eq!(stop_color, DesignColor::rgba(0x4c, 0x86, 0xf7, 0x70));
    let gradient_edit = color_edit(&gradient, 1, stop_color);
    assert!(gradient.apply_edit(&gradient_edit));
    assert_eq!(gradient.opacity, 58.);
    assert_eq!(gradient.gradient_stops[1].color.alpha, 0x70);
}

#[test]
fn explicit_alpha_maps_to_solid_paint_opacity_but_gradient_leaf_alpha() {
    let parsed = parse_hex_color_input("4C86F780").expect("eight-digit RGBA");

    let mut solid = DesignPaint::solid(DesignColor::BLACK);
    solid.opacity = 72.;
    let color_edit = color_input_edit(&solid, 0, parsed);
    assert_eq!(color_edit.property, DesignPaintProperty::Color);
    assert_eq!(
        color_edit.value,
        DesignPaintValue::Color(DesignColor::rgb(0x4c, 0x86, 0xf7))
    );
    assert!(solid.apply_edit(&color_edit));
    let opacity_edit = picker_opacity_edit(&solid, 0, f32::from(parsed.color.alpha) / 255. * 100.);
    assert_eq!(opacity_edit.property, DesignPaintProperty::Opacity);
    assert!(solid.apply_edit(&opacity_edit));
    assert_eq!(solid.color.alpha, u8::MAX);
    assert!((solid.opacity - f32::from(0x80_u8) / 255. * 100.).abs() < 0.001);

    let mut gradient = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK).with_id("start"),
            DesignGradientStop::new(1., DesignColor::WHITE).with_id("end"),
        ],
    );
    gradient.opacity = 37.;
    let edit = color_input_edit(&gradient, 1, parsed);
    assert_eq!(
        edit.property,
        DesignPaintProperty::GradientStopColor {
            stop_id: "end".into(),
            index: 1,
        }
    );
    assert_eq!(edit.value, DesignPaintValue::Color(parsed.color));
    assert!(gradient.apply_edit(&edit));
    assert_eq!(gradient.opacity, 37.);
    assert_eq!(gradient.gradient_stops[1].color.alpha, 0x80);
}

#[test]
fn color_text_serialization_uses_the_applicable_alpha_leaf() {
    let mut solid = DesignPaint::solid(DesignColor::rgba(0x4c, 0x86, 0xf7, 0x11));
    solid.opacity = f32::from(0x80_u8) / 255. * 100.;
    assert_eq!(color_text_value(ColorFormat::Hex, &solid, 0), "4C86F780");
    assert_eq!(
        color_text_value(ColorFormat::Css, &solid, 0),
        "rgba(76, 134, 247, 0.502)"
    );

    let gradient = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK),
            DesignGradientStop::new(1., DesignColor::rgba(0x4c, 0x86, 0xf7, 0x40)),
        ],
    );
    assert_eq!(color_text_value(ColorFormat::Hex, &gradient, 1), "4C86F740");
}

#[test]
fn hsv_round_trip_preserves_display_channels_and_alpha() {
    for color in [
        DesignColor::rgba(0xff, 0x00, 0x00, 0x33),
        DesignColor::rgba(0x4c, 0x86, 0xf7, 0x80),
        DesignColor::rgba(0x22, 0xcc, 0x88, 0xff),
        DesignColor::rgba(0x77, 0x77, 0x77, 0x01),
    ] {
        assert_eq!(hsv_to_color(rgb_to_hsv(color), color.alpha), color);
    }
}

#[test]
fn top_level_types_preserve_gradient_subtypes_and_create_defaults() {
    assert_eq!(
        DesignPaintType::ALL,
        [
            DesignPaintType::Solid,
            DesignPaintType::Gradient,
            DesignPaintType::Pattern,
            DesignPaintType::Image,
            DesignPaintType::Video,
            DesignPaintType::Shader,
        ]
    );
    assert_eq!(PAINT_HEADER_TRAILING_CONTROL_COUNT, 2);
    assert_eq!(paint_header_min_width(), 272.);
    assert!(
        PICKER_WIDTH >= paint_header_min_width(),
        "six paint types plus compact Blend and Contrast controls must not overflow"
    );
    assert_eq!(
        COLOR_AREA_HEIGHT,
        PICKER_WIDTH - 32.,
        "the widened picker keeps the p-4 color plane square"
    );
    assert_eq!(
        GRADIENT_KINDS,
        [
            DesignPaintKind::LinearGradient,
            DesignPaintKind::RadialGradient,
            DesignPaintKind::AngularGradient,
            DesignPaintKind::DiamondGradient,
        ]
    );
    let radial = gradient();
    assert_eq!(
        paint_for_type(&radial, DesignPaintType::Gradient, 0).kind,
        DesignPaintKind::RadialGradient
    );

    let solid = DesignPaint::solid(DesignColor::BLUE);
    let created = paint_for_type(&solid, DesignPaintType::Gradient, 0);
    assert_eq!(created.kind, DesignPaintKind::LinearGradient);
    assert_eq!(created.gradient_stops.len(), 2);
    assert_eq!(created.gradient_stops[0].color, DesignColor::BLUE);
    assert_eq!(created.gradient_stops[1].color.alpha, 0);

    let pattern = paint_for_type(&solid, DesignPaintType::Pattern, 0);
    assert_eq!(pattern.kind, DesignPaintKind::Pattern);
    let DesignPaintPayload::Pattern(pattern) = pattern.payload else {
        panic!("pattern payload");
    };
    assert!(pattern.source_node_id.is_empty());
    assert_eq!(pattern.tile_type, DesignPatternTileType::Rectangular);
    assert_eq!(pattern.scaling_factor, 1.);
    assert_eq!(pattern.spacing, DesignPatternSpacing::default());
    assert_eq!(
        pattern.horizontal_alignment,
        DesignPatternHorizontalAlignment::Center
    );

    assert_eq!(
        paint_for_kind(&solid, DesignPaintKind::DiamondGradient, 0).kind,
        DesignPaintKind::DiamondGradient
    );
    assert_eq!(
        paint_for_kind(&radial, DesignPaintKind::AngularGradient, 0).kind,
        DesignPaintKind::AngularGradient
    );
}

#[gpui::test]
fn top_level_type_and_gradient_subtype_emit_distinct_typed_edits(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLUE).with_id("fill"),
                window,
                cx,
            );
            picker.select_paint_type(DesignPaintType::Gradient, cx);
        });
    });
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::Edit {
            edit,
            phase: DesignPanelEditPhase::Commit,
            ..
        }] if matches!(
            &edit.value,
            DesignPaintValue::Payload(DesignPaintPayload::Gradient(gradient))
                if gradient.kind == DesignPaintKind::LinearGradient
                    && gradient.stops.len() == 2
        )
    ));

    events.borrow_mut().clear();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                gradient().with_id("fill"),
                window,
                cx,
            );
            picker.select_paint_kind(DesignPaintKind::DiamondGradient, cx);
        });
    });
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::Edit {
            edit,
            phase: DesignPanelEditPhase::Commit,
            ..
        }] if edit.property == DesignPaintProperty::GradientKind
            && edit.value
                == DesignPaintValue::PaintKind(DesignPaintKind::DiamondGradient)
    ));
}

#[gpui::test]
fn host_can_limit_paint_types_and_reject_unsupported_edits(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_supported_paint_types(
                &[DesignPaintType::Solid, DesignPaintType::Gradient],
                cx,
            );
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLUE).with_id("fill"),
                window,
                cx,
            );
            picker.select_paint_type(DesignPaintType::Pattern, cx);
        });
    });
    assert!(events.borrow().is_empty());
    visual_cx.read(|app| {
        assert_eq!(
            picker.read(app).supported_paint_types,
            [DesignPaintType::Solid, DesignPaintType::Gradient]
        );
    });
    visual_cx.update(|_, app| {
        picker.update(app, |picker, cx| {
            picker.select_paint_type(DesignPaintType::Gradient, cx);
        });
    });
    assert_eq!(events.borrow().len(), 1);
}

#[gpui::test]
fn pattern_image_and_video_tabs_emit_payloads(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    for paint_type in [
        DesignPaintType::Pattern,
        DesignPaintType::Image,
        DesignPaintType::Video,
    ] {
        events.borrow_mut().clear();
        visual_cx.update(|window, app| {
            picker.update(app, |picker, cx| {
                picker.set_target(
                    "node",
                    DesignPanelCollection::Fill,
                    0,
                    DesignPaint::solid(DesignColor::BLUE).with_id("fill"),
                    window,
                    cx,
                );
                picker.select_paint_type(paint_type, cx);
            });
        });
        assert!(matches!(
            events.borrow().as_slice(),
            [PaintPickerEvent::Edit {
                edit,
                phase: DesignPanelEditPhase::Commit,
                ..
            }] if matches!(&edit.value, DesignPaintValue::Payload(payload) if payload.kind().paint_type() == paint_type)
        ));
    }
}

#[gpui::test]
fn pattern_source_selector_emits_a_typed_edit_for_an_eligible_layer(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "destination",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::pattern("").with_id("pattern-fill"),
                window,
                cx,
            );
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([]).with_pattern_sources([
                    DesignPatternSource::new("tile-one", "First tile"),
                    DesignPatternSource::new("tile-two", "Second tile"),
                ]),
                cx,
            );
        });
    });
    visual_cx.run_until_parked();

    let trigger = visual_cx
        .debug_bounds("paint-picker-test-pattern-source-trigger")
        .expect("Pattern source selector should be visible");
    visual_cx.simulate_click(trigger.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    let tile = visual_cx
        .debug_bounds("paint-picker-test-pattern-source-tile-two")
        .expect("Host-supplied layer should appear in Pattern source menu");
    visual_cx.simulate_click(tile.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();

    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::Edit {
            target,
            edit,
            phase: DesignPanelEditPhase::Commit,
        }] if target.node_id.as_ref() == "destination"
            && target.paint_id.as_ref() == "pattern-fill"
            && edit.property == DesignPaintProperty::PatternSourceNode
            && edit.value == DesignPaintValue::PatternSourceNode("tile-two".into())
    ));
    assert!(visual_cx.read(|app| {
        !picker
            .read(app)
            .nested_overlay_is_open(PaintPickerOverlay::PatternSource)
    }));
}

#[gpui::test]
fn pattern_source_selector_rejects_stale_and_locked_choices(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "destination",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::pattern("tile-one").with_id("pattern-fill"),
                window,
                cx,
            );
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([]).with_pattern_sources([
                    DesignPatternSource::new("destination", "Destination"),
                    DesignPatternSource::new("tile-one", "First tile"),
                ]),
                cx,
            );
            picker.select_pattern_source("destination".into(), window, cx);
            picker.select_pattern_source("stale-tile".into(), window, cx);
            picker.select_pattern_source("tile-one".into(), window, cx);
            picker.set_disabled(true, cx);
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([])
                    .with_pattern_sources([DesignPatternSource::new("tile-two", "Second tile")]),
                cx,
            );
            picker.select_pattern_source("tile-two".into(), window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(events.borrow().is_empty());
}

#[gpui::test]
fn picker_creation_menu_omits_unavailable_paint_style_action(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLUE).with_id("fill"),
                window,
                cx,
            );
            picker.set_paint_style_creation_enabled(false, cx);
            assert_eq!(picker.creation_kinds(), [PaintCreationKind::Variable]);
            picker.request_paint_style_create(cx);
            picker.activate_creation_kind(PaintCreationKind::Variable, window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::ColorVariableCreateRequested { target, .. }]
            if target.paint_id.as_ref() == "fill"
    ));
}

#[gpui::test]
fn media_property_viewer_can_switch_to_another_supported_paint_type(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::image(DesignPaintSource::new("source", "Photo")).with_id("image-fill"),
                window,
                cx,
            );
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([DesignMediaPaintView::new(
                    DesignPanelCollection::Fill,
                    "image-fill",
                    0,
                )
                .with_capabilities(DesignMediaPaintCapabilities::viewer())]),
                cx,
            );
            assert!(picker.editing_disabled());
            assert!(!picker.base_editing_disabled());
            picker.select_paint_type(DesignPaintType::Solid, cx);
        });
    });
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::Edit {
            edit,
            phase: DesignPanelEditPhase::Commit,
            ..
        }] if matches!(&edit.value, DesignPaintValue::Payload(DesignPaintPayload::Solid(_)))
    ));
}

#[gpui::test]
fn gradient_subtype_menu_roves_wraps_and_commits_the_highlighted_kind(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                gradient().with_id("fill"),
                window,
                cx,
            );
            picker.focus_handle.focus(window, cx);
            picker.open_gradient_kind_menu_from_keyboard(window, cx);
        });
    });
    visual_cx.run_until_parked();
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            assert!(picker.nested_overlay_is_open(PaintPickerOverlay::GradientKind));
            assert_eq!(picker.gradient_kind_menu_index, 1);
            picker.handle_gradient_kind_menu_key(
                &KeyDownEvent {
                    keystroke: Keystroke::parse("home").expect("Home"),
                    is_held: false,
                    prefer_character_input: false,
                },
                window,
                cx,
            );
            picker.handle_gradient_kind_menu_key(
                &KeyDownEvent {
                    keystroke: Keystroke::parse("up").expect("Up"),
                    is_held: false,
                    prefer_character_input: false,
                },
                window,
                cx,
            );
            assert_eq!(picker.gradient_kind_menu_index, GRADIENT_KINDS.len() - 1);
            picker.handle_gradient_kind_menu_key(
                &KeyDownEvent {
                    keystroke: Keystroke::parse("enter").expect("Enter"),
                    is_held: false,
                    prefer_character_input: false,
                },
                window,
                cx,
            );
            assert!(!picker.nested_overlay_is_open(PaintPickerOverlay::GradientKind));
        });
    });
    visual_cx.run_until_parked();

    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::Edit {
            edit,
            phase: DesignPanelEditPhase::Commit,
            ..
        }] if edit.property == DesignPaintProperty::GradientKind
            && edit.value
                == DesignPaintValue::PaintKind(DesignPaintKind::DiamondGradient)
    ));
    assert!(visual_cx.update(|window, app| { picker.read(app).focus_handle.is_focused(window) }));
}

#[test]
fn editing_a_stop_returns_a_candidate_without_mutating_the_source() {
    let source = gradient();
    let color = DesignColor::rgba(0x12, 0x34, 0x56, 0x78);
    let candidate = paint_with_color(&source, 1, color);

    assert_eq!(source.gradient_stops[1].color, DesignColor::WHITE);
    assert_eq!(candidate.gradient_stops[1].color, color);
    assert_eq!(candidate.color, source.gradient_stops[0].color);
}

#[test]
fn gradient_stop_add_and_remove_keep_a_valid_ordered_gradient() {
    let source = gradient();
    let (added, selected) = paint_with_added_stop(&source, 0);
    assert_eq!(added.gradient_stops.len(), 3);
    assert_eq!(added.gradient_stops[selected].position, 0.5);
    assert!(
        added
            .gradient_stops
            .windows(2)
            .all(|pair| pair[0].position <= pair[1].position)
    );

    let (removed, selected) =
        paint_with_removed_stop(&added, selected).expect("three-stop gradient");
    assert_eq!(removed.gradient_stops.len(), 2);
    assert!(selected < removed.gradient_stops.len());
    assert!(paint_with_removed_stop(&removed, selected).is_none());
}

#[test]
fn click_add_interpolates_at_the_exact_pointer_position() {
    let source = gradient();
    let (added, selected) = paint_with_added_stop_at(&source, 0.25);
    assert_eq!(added.gradient_stops[selected].position, 0.25);
    assert_eq!(
        added.gradient_stops[selected].color,
        mix_color(DesignColor::BLACK, DesignColor::WHITE, 0.25)
    );
}

#[test]
fn click_add_selects_the_new_legacy_stop_when_semantics_duplicate_an_existing_stop() {
    let source = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK).with_id("start"),
            DesignGradientStop::new(0.5, DesignColor::rgb(0x80, 0x80, 0x80))
                .with_id("existing-midpoint"),
            DesignGradientStop::new(1., DesignColor::WHITE).with_id("end"),
        ],
    );
    let (added, selected) = paint_with_added_stop_at(&source, 0.5);

    assert_eq!(added.gradient_stops.len(), 4);
    assert_eq!(added.gradient_stops[selected].position, 0.5);
    assert_eq!(
        added.gradient_stops[selected].color,
        DesignColor::rgb(0x80, 0x80, 0x80)
    );
    assert!(added.gradient_stops[selected].id.is_empty());
    assert_ne!(
        added.gradient_stops[selected].id,
        added.gradient_stops[selected - 1].id
    );
}

#[gpui::test]
fn disabled_gradient_preview_add_does_not_drift_transient_selection(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                gradient().with_id("fill"),
                window,
                cx,
            );
            picker.set_disabled(true, cx);
            picker.add_gradient_stop_at(0.5, cx);
            assert_eq!(picker.selected_stop, 0);
            assert!(picker.selected_stop_id.is_empty());
            assert_eq!(
                picker
                    .paint
                    .as_ref()
                    .map(|paint| paint.gradient_stops.len()),
                Some(2)
            );
        });
    });
    assert!(events.borrow().is_empty());
}

#[test]
fn preview_segments_use_host_stop_positions_instead_of_equal_flex_widths() {
    let paint = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK),
            DesignGradientStop::new(0.2, DesignColor::PURPLE),
            DesignGradientStop::new(1., DesignColor::WHITE),
        ],
    );
    let segments = gradient_segments(&paint);
    assert_eq!(segments.len(), 2);
    assert_eq!((segments[0].0, segments[0].1), (0., 0.2));
    assert_eq!((segments[1].0, segments[1].1), (0.2, 1.));

    let inset = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0.2, DesignColor::BLACK),
            DesignGradientStop::new(0.8, DesignColor::WHITE),
        ],
    );
    let inset_segments = gradient_segments(&inset);
    assert_eq!(inset_segments.len(), 3);
    assert_eq!((inset_segments[0].0, inset_segments[0].1), (0., 0.2));
    assert_eq!((inset_segments[2].0, inset_segments[2].1), (0.8, 1.));
}

#[test]
fn stable_picker_target_equality_ignores_reordered_indices() {
    let first = PaintPickerTarget {
        node_id: "node".into(),
        collection: DesignPanelCollection::Fill,
        index: 0,
        paint_id: "stable-paint".into(),
    };
    let moved = PaintPickerTarget {
        index: 3,
        ..first.clone()
    };
    assert_eq!(first, moved);

    let legacy = PaintPickerTarget {
        paint_id: "".into(),
        ..first
    };
    let legacy_moved = PaintPickerTarget {
        index: 3,
        ..legacy.clone()
    };
    assert_ne!(legacy, legacy_moved);
}

#[test]
fn media_drop_path_helper_requires_one_host_accepted_file() {
    let editor = DesignMediaPaintCapabilities::editor();
    assert_eq!(
        media_drop_from_paths(&[PathBuf::from("replacement.PNG")], editor).map(|file| file.kind),
        Some(crate::design::DesignMediaFileKind::Png)
    );
    assert_eq!(
        media_drop_from_paths(
            &[PathBuf::from("one.png"), PathBuf::from("two.png")],
            editor,
        ),
        None
    );
    assert_eq!(
        media_drop_from_paths(&[PathBuf::from("vector.svg")], editor),
        None
    );
    assert_eq!(
        media_drop_from_paths(&[PathBuf::from("scan.tiff")], editor),
        None
    );
    assert_eq!(
        media_drop_from_paths(
            &[PathBuf::from("scan.TIFF")],
            editor.with_accepted_drop_file_kinds(
                editor.accepted_drop_file_kinds | crate::design::DesignMediaFileKinds::TIFF,
            ),
        )
        .map(|file| file.kind),
        Some(crate::design::DesignMediaFileKind::Tiff)
    );
}

#[gpui::test]
fn media_drop_emits_current_source_identity_and_allows_cross_kind_file(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::image(DesignPaintSource::new("source-image", "Cover"))
                    .with_id("stable-media"),
                window,
                cx,
            );
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([DesignMediaPaintView::new(
                    DesignPanelCollection::Fill,
                    "stable-media",
                    0,
                )]),
                cx,
            );
            picker.request_media_source_drop(&[PathBuf::from("clip.WEBM")], cx);
        });
    });
    visual_cx.run_until_parked();
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::MediaSourceDropRequested {
            target,
            expected_source_id,
            expected_media_kind: DesignMediaKind::Image,
            file,
        }] if target.paint_id.as_ref() == "stable-media"
            && expected_source_id.as_ref() == "source-image"
            && file.kind == crate::design::DesignMediaFileKind::Webm
            && file.media_kind() == DesignMediaKind::Video
    ));
}

#[gpui::test]
fn crop_rotation_control_obeys_host_capability(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let events = picker_events(&host, visual_cx);
    let mut paint =
        DesignPaint::image(DesignPaintSource::new("photo", "Photo")).with_id("image-fill");
    if let DesignPaintPayload::Image(image) = &mut paint.payload {
        image.placement = DesignMediaPaintPlacement::Crop {
            transform: DesignPaintTransform::IDENTITY,
        };
    }
    paint.sync_legacy_projection();
    let crop_view = DesignMediaPaintView::new(DesignPanelCollection::Fill, "image-fill", 0)
        .with_crop_tool(DesignMediaCropToolState {
            active: true,
            ..DesignMediaCropToolState::default()
        });
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target("node", DesignPanelCollection::Fill, 0, paint, window, cx);
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([crop_view.clone().with_capabilities(
                    DesignMediaPaintCapabilities::property_editor_only().with_crop_rotation(false),
                )]),
                cx,
            );
        });
    });
    visual_cx.run_until_parked();
    let rotate = visual_cx
        .debug_bounds("paint-picker-test-media-crop-rotate")
        .expect("Rotate crop control should be present");
    visual_cx.simulate_click(rotate.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    assert!(events.borrow().is_empty());

    picker.update(visual_cx, |picker, cx| {
        picker.set_media_view_data(
            DesignMediaPaintViewData::new([crop_view.with_capabilities(
                DesignMediaPaintCapabilities::property_editor_only().with_crop_rotation(true),
            )]),
            cx,
        );
    });
    visual_cx.run_until_parked();
    let rotate = visual_cx
        .debug_bounds("paint-picker-test-media-crop-rotate")
        .expect("Rotate crop control should remain present");
    visual_cx.simulate_click(rotate.center(), gpui::Modifiers::none());
    visual_cx.run_until_parked();
    assert!(matches!(
        events.borrow().as_slice(),
        [PaintPickerEvent::MediaCropActionRequested {
            action: DesignMediaCropAction::Preview { transform, .. },
            ..
        }] if (transform.m11 - 1.).abs() > 0.01
    ));
}

#[test]
fn picker_locks_only_global_read_only_and_opaque_paints() {
    assert!(paint_picker_locked(
        &DesignPaint::solid(DesignColor::BLACK).with_read_only(true)
    ));
    let bound = DesignPaint::from_payload(DesignPaintPayload::Solid(DesignSolidPaint {
        color: DesignColor::PURPLE,
        binding: Some(crate::design::DesignPaintBinding::new("variable", "Brand")),
    }));
    assert!(!paint_picker_locked(&bound));
    assert!(paint_color_locked(&bound, 0));
    assert!(!paint_picker_locked(&DesignPaint::shader(
        "shader",
        "Fractal noise"
    )));
    assert!(!paint_picker_locked(&DesignPaint::solid(
        DesignColor::BLACK
    )));

    let partially_bound = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK),
            DesignGradientStop::new(1., DesignColor::WHITE)
                .with_binding(crate::design::DesignPaintBinding::new("variable", "Brand")),
        ],
    );
    assert!(!paint_color_locked(&partially_bound, 0));
    assert!(paint_color_locked(&partially_bound, 1));
}

#[test]
fn color_style_selection_targets_only_the_active_color_leaf() {
    assert_eq!(
        paint_color_target(&DesignPaint::solid(DesignColor::PURPLE), 0),
        Some(DesignPaintColorTarget::Solid)
    );

    let gradient = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::BLACK).with_id("start"),
            DesignGradientStop::new(1., DesignColor::WHITE).with_id("end"),
        ],
    );
    assert_eq!(
        paint_color_target(&gradient, 1),
        Some(DesignPaintColorTarget::GradientStop {
            stop_id: "end".into(),
            index: 1,
        })
    );
    assert_eq!(
        paint_color_target(&gradient, usize::MAX),
        Some(DesignPaintColorTarget::GradientStop {
            stop_id: "end".into(),
            index: 1,
        })
    );
    assert_eq!(
        paint_color_target(
            &DesignPaint::image(DesignPaintSource::new("image", "Hero")),
            0,
        ),
        None
    );
}

#[test]
fn compact_swatch_and_library_row_share_variable_identity_selection() {
    let selected = DesignPaintBinding::new("brand-primary", "Brand");
    let exact = DesignVariable::page(
        "brand-primary",
        "Brand",
        "colors",
        "Colors",
        crate::design::DesignVariableResolvedType::Color,
    )
    .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::PURPLE));
    let same_color = DesignVariable::page(
        "other",
        "Other",
        "colors",
        "Colors",
        crate::design::DesignVariableResolvedType::Color,
    )
    .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::PURPLE));

    assert!(paint_variable_matches_binding(&exact, Some(&selected)));
    assert!(!paint_variable_matches_binding(
        &same_color,
        Some(&selected)
    ));
    assert!(!paint_variable_matches_binding(&exact, None));
}

#[test]
fn library_resource_search_matches_names_groups_and_library_case_insensitively() {
    assert!(paint_resource_matches(
        "brand",
        ["Brand / Primary", "Semantic colors", "Foundations"]
    ));
    assert!(paint_resource_matches(
        "semantic",
        ["Brand / Primary", "Semantic colors", "Foundations"]
    ));
    assert!(paint_resource_matches(
        "foundations",
        ["Brand / Primary", "Semantic colors", "Product Foundations"]
    ));
    assert!(paint_resource_matches(
        "",
        ["Brand / Primary", "Semantic colors", "Foundations"]
    ));
    assert!(!paint_resource_matches(
        "motion",
        ["Brand / Primary", "Semantic colors", "Foundations"]
    ));
}

#[gpui::test]
fn library_search_is_transient_and_resets_for_a_new_paint_target(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    let search = visual_cx.read(|app| picker.read(app).resource_search_input.clone());
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill-a"),
                window,
                cx,
            );
        });
        search.update(app, |input, cx| {
            input.set_value("brand", window, cx);
        });
    });
    visual_cx.run_until_parked();
    assert_eq!(visual_cx.read(|app| search.read(app).value()), "brand");

    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                1,
                DesignPaint::solid(DesignColor::WHITE).with_id("fill-b"),
                window,
                cx,
            );
        });
    });
    visual_cx.run_until_parked();
    assert!(visual_cx.read(|app| search.read(app).value().is_empty()));
}

#[test]
fn normalization_clamps_host_values_and_repairs_missing_stops() {
    let mut paint = DesignPaint::gradient(
        DesignPaintKind::AngularGradient,
        vec![
            DesignGradientStop::new(2., DesignColor::WHITE),
            DesignGradientStop::new(-1., DesignColor::BLACK),
        ],
    );
    paint.opacity = f32::NAN;
    let normalized = normalize_paint(paint);

    assert_eq!(normalized.opacity, 100.);
    assert_eq!(normalized.gradient_stops[0].position, 0.);
    assert_eq!(normalized.gradient_stops[1].position, 1.);
    assert_eq!(normalized.color, DesignColor::BLACK);
}

#[test]
fn opacity_parser_is_finite_and_clamped() {
    assert_eq!(parse_opacity("42.5%"), Some(42.5));
    assert_eq!(parse_opacity("-10"), Some(0.));
    assert_eq!(parse_opacity("120"), Some(100.));
    assert_eq!(parse_opacity("NaN"), None);
    assert_eq!(parse_opacity(""), None);
    assert_eq!(parse_stop_position("37.5%"), Some(0.375));
}

#[test]
fn pointer_fractions_clamp_to_the_control_bounds() {
    let bounds = Bounds {
        origin: gpui::point(px(10.), px(20.)),
        size: gpui::size(px(100.), px(50.)),
    };
    assert_eq!(fraction_x(bounds, gpui::point(px(-20.), px(0.))), 0.);
    assert_eq!(fraction_x(bounds, gpui::point(px(60.), px(0.))), 0.5);
    assert_eq!(fraction_x(bounds, gpui::point(px(200.), px(0.))), 1.);
    assert_eq!(fraction_y(bounds, gpui::point(px(0.), px(45.))), 0.5);
}

#[gpui::test]
fn picker_header_tab_and_creation_trigger_activate_from_the_keyboard(cx: &mut TestAppContext) {
    let (host, visual_cx) = setup_picker(cx);
    let picker = picker(&host, visual_cx);
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "node",
                DesignPanelCollection::Fill,
                0,
                DesignPaint::solid(DesignColor::BLACK).with_id("fill"),
                window,
                cx,
            );
            picker.focus_handle.focus(window, cx);
        });
    });
    visual_cx.run_until_parked();

    let mut libraries_activated = false;
    for _ in 0..24 {
        visual_cx.update(|window, app| {
            window.focus_next(app);
        });
        visual_cx.simulate_keystrokes("enter");
        visual_cx.run_until_parked();
        if visual_cx.read(|app| picker.read(app).active_tab == PaintPickerTab::Libraries) {
            libraries_activated = true;
            break;
        }
    }
    assert!(
        libraries_activated,
        "the Libraries header tab must be reachable with Tab and activate from Enter"
    );

    // The creation trigger opens its roving menu from the same bound
    // command instead of raw key matching.
    visual_cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.active_tab = PaintPickerTab::Custom;
            cx.notify();
            picker.focus_handle.focus(window, cx);
        });
    });
    visual_cx.run_until_parked();
    let mut creation_opened = false;
    for _ in 0..24 {
        visual_cx.update(|window, app| {
            window.focus_next(app);
        });
        visual_cx.simulate_keystrokes("enter");
        visual_cx.run_until_parked();
        if visual_cx.read(|app| {
            picker
                .read(app)
                .nested_overlay_is_open(PaintPickerOverlay::Creation)
        }) {
            creation_opened = true;
            break;
        }
    }
    assert!(
        creation_opened,
        "the creation-menu trigger must open its menu from Enter while focused"
    );
}

#[gpui::test]
fn standalone_picker_emits_close_and_contains_wheel_events(cx: &mut TestAppContext) {
    let (host, cx) = setup_picker(cx);
    let picker = picker(&host, cx);
    let events = picker_events(&host, cx);
    let paint = gradient().with_id("standalone-gradient");
    cx.update(|window, app| {
        picker.update(app, |picker, cx| {
            picker.set_target(
                "standalone-layer",
                DesignPanelCollection::Fill,
                0,
                paint.clone(),
                window,
                cx,
            );
            picker.focus_handle.focus(window, cx);
        })
    });
    cx.run_until_parked();
    let close = cx.debug_bounds("paint-picker-close").unwrap();
    cx.simulate_event(gpui::ScrollWheelEvent {
        position: close.center(),
        delta: gpui::ScrollDelta::Pixels(point(Pixels::ZERO, px(-80.))),
        ..Default::default()
    });
    cx.run_until_parked();
    assert_eq!(cx.read(|app| *host.read(app).ancestor_scrolls.borrow()), 0);
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert_eq!(
        events.borrow().as_slice(),
        &[PaintPickerAction::CloseRequested]
    );
    assert_eq!(
        cx.read(|app| picker.read(app).paint().cloned()),
        Some(paint)
    );
    events.borrow_mut().clear();
    cx.simulate_click(close.center(), gpui::Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        events.borrow().as_slice(),
        &[PaintPickerAction::CloseRequested]
    );
}
