use super::*;
use crate::test_support::mount_component;
use gpui::{Modifiers, TestAppContext, point, px};

#[gpui::test]
fn standalone_spectrum_hue_and_alpha_emit_controlled_edits(cx: &mut TestAppContext) {
    let original = PickerColor::rgba(13, 153, 255, 255);
    let (host, actions, cx) = mount_component(cx, move |window, cx| {
        ColorPicker::new("standalone-picker", original, window, cx)
    });
    let picker = cx.read(|app| host.read(app).component.clone());
    for selector in [
        "color-picker-spectrum",
        "color-picker-hue",
        "color-picker-alpha",
    ] {
        actions.borrow_mut().clear();
        let bounds = cx
            .debug_bounds(selector)
            .expect("picker control must be rendered");
        cx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
        let events = actions.borrow();
        assert!(
            events.iter().any(|event| matches!(
                event,
                ColorPickerAction::Edit {
                    phase: ColorPickerPhase::Begin,
                    ..
                }
            )),
            "{selector}: {events:?}"
        );
        assert!(
            events.iter().any(|event| matches!(
                event,
                ColorPickerAction::Edit {
                    phase: ColorPickerPhase::Commit,
                    ..
                }
            )),
            "{selector}: {events:?}"
        );
        picker.read_with(cx, |picker, _| assert_eq!(picker.color(), original));
    }
}

#[gpui::test]
fn standalone_hex_entry_commits_once_and_cancels_drafts(cx: &mut TestAppContext) {
    let original = PickerColor::rgba(13, 153, 255, 255);
    let (_host, actions, cx) = mount_component(cx, move |window, cx| {
        ColorPicker::new("standalone-hex", original, window, cx)
    });
    let field = cx.debug_bounds("color-picker-hex").unwrap();
    cx.simulate_click(field.center(), Modifiers::none());
    cx.run_until_parked();
    for key in ["cmd-a", "1", "2", "3", "4", "5", "6", "enter"] {
        cx.simulate_keystrokes(key);
    }
    cx.run_until_parked();
    let commits = actions
        .borrow()
        .iter()
        .filter_map(|action| match action {
            ColorPickerAction::Edit {
                color,
                phase: ColorPickerPhase::Commit,
            } => Some(*color),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(commits, vec![PickerColor::rgba(0x12, 0x34, 0x56, 255)]);
    actions.borrow_mut().clear();
    for key in ["cmd-a", "a", "b", "c", "d", "e", "f", "escape"] {
        cx.simulate_keystrokes(key);
    }
    cx.run_until_parked();
    assert!(!actions.borrow().iter().any(|action| matches!(
        action,
        ColorPickerAction::Edit {
            phase: ColorPickerPhase::Commit,
            ..
        }
    )));
}

#[gpui::test]
fn picker_text_edits_update_the_visible_preview(cx: &mut TestAppContext) {
    let (_host, actions, cx) = mount_component(cx, |window, cx| {
        ColorPicker::new(
            "text-preview",
            PickerColor::rgba(255, 0, 0, 255),
            window,
            cx,
        )
    });
    let hex = cx.debug_bounds("color-picker-hex").unwrap();
    cx.simulate_click(hex.center(), Modifiers::none());
    cx.run_until_parked();
    let before = cx.debug_bounds("color-picker-hue-thumb").unwrap();
    cx.simulate_keystrokes("cmd-a 0 0 F F 0 0");
    cx.run_until_parked();
    let after = cx.debug_bounds("color-picker-hue-thumb").unwrap();
    assert!(after.left() > before.left());
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(actions.borrow().iter().any(|a|matches!(a,ColorPickerAction::Edit {color,phase:ColorPickerPhase::Commit} if *color==PickerColor::rgba(0,255,0,255))));
    let alpha = cx.debug_bounds("color-picker-opacity").unwrap();
    cx.simulate_click(alpha.center(), Modifiers::none());
    cx.run_until_parked();
    cx.simulate_keystrokes("cmd-a 5 0 enter");
    cx.run_until_parked();
    assert!(actions.borrow().iter().any(|a|matches!(a,ColorPickerAction::Edit {color,phase:ColorPickerPhase::Commit} if color.alpha==128)));
    let format = cx.debug_bounds("color-picker-format").unwrap();
    cx.simulate_click(format.center(), Modifiers::none());
    cx.run_until_parked();
    let menu = cx.debug_bounds("color-picker-format-menu").unwrap();
    assert!(
        menu.bottom() <= format.top(),
        "menu {menu:?}, trigger {format:?}"
    );
    let rgb = cx.debug_bounds("color-picker-format-RGB").unwrap();
    cx.simulate_click(rgb.center(), Modifiers::none());
    cx.run_until_parked();
    let red = cx.debug_bounds("color-picker-channel-0").unwrap();
    cx.simulate_click(red.center(), Modifiers::none());
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("cmd-a 2 5 5 enter");
    cx.run_until_parked();
    assert!(actions.borrow().iter().any(|a|matches!(a,ColorPickerAction::Edit {color,phase:ColorPickerPhase::Commit} if color.red==255 && color.green==255)));
}

#[gpui::test]
fn selectors_stay_inside_all_three_controls_at_extremes(cx: &mut TestAppContext) {
    let (_host, _, cx) = mount_component(cx, |window, cx| {
        ColorPicker::new(
            "bounds-picker",
            PickerColor::rgba(255, 0, 0, 255),
            window,
            cx,
        )
    });
    for (control, thumb) in [
        ("color-picker-spectrum", "color-picker-spectrum-thumb"),
        ("color-picker-hue", "color-picker-hue-thumb"),
        ("color-picker-alpha", "color-picker-alpha-thumb"),
    ] {
        let bounds = cx.debug_bounds(control).unwrap();
        for position in [bounds.origin, bounds.bottom_right()] {
            cx.simulate_click(position, Modifiers::none());
            cx.run_until_parked();
            let thumb = cx.debug_bounds(thumb).unwrap();
            assert!(thumb.left() >= bounds.left() && thumb.right() <= bounds.right());
            assert!(thumb.top() >= bounds.top() && thumb.bottom() <= bounds.bottom());
        }
    }
}

#[gpui::test]
fn opacity_slider_drag_keeps_its_preview_and_commits_the_release_position(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |window, cx| {
        ColorPicker::new(
            "opacity-drag-picker",
            PickerColor::rgba(255, 0, 0, 255),
            window,
            cx,
        )
    });
    let rail = cx.debug_bounds("color-picker-alpha").unwrap();
    let target = point(rail.left() + rail.size.width * 0.25, rail.center().y);

    cx.simulate_mouse_down(rail.center(), gpui::MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(target, Some(gpui::MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    let preview_thumb = cx.debug_bounds("color-picker-alpha-thumb").unwrap();
    assert!((preview_thumb.center().x - target.x).abs() < px(2.));

    cx.simulate_mouse_up(target, gpui::MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let committed_thumb = cx.debug_bounds("color-picker-alpha-thumb").unwrap();
    assert!((committed_thumb.center().x - target.x).abs() < px(2.));
    assert!(
        actions.borrow().iter().any(|action| matches!(
            action,
            ColorPickerAction::Edit {
                color,
                phase: ColorPickerPhase::Commit,
            } if (56..=62).contains(&color.alpha)
        )),
        "{:#?}",
        actions.borrow()
    );
    let picker = cx.read(|app| host.read(app).component.clone());
    let accepted = actions
        .borrow()
        .iter()
        .find_map(|action| match action {
            ColorPickerAction::Edit {
                color,
                phase: ColorPickerPhase::Commit,
            } => Some(*color),
            _ => None,
        })
        .unwrap();
    cx.update(|window, app| {
        picker.update(app, |picker, cx| picker.set_color(accepted, window, cx));
    });
    cx.run_until_parked();
    let echoed_thumb = cx.debug_bounds("color-picker-alpha-thumb").unwrap();
    assert!((echoed_thumb.center().x - target.x).abs() < px(2.));
}

#[gpui::test]
fn spectrum_selector_keeps_pointer_position_near_black(cx: &mut TestAppContext) {
    let (_host, _, cx) = mount_component(cx, |window, cx| {
        ColorPicker::new(
            "spectrum-bottom-picker",
            PickerColor::rgba(0, 255, 255, 255),
            window,
            cx,
        )
    });
    let spectrum = cx.debug_bounds("color-picker-spectrum").unwrap();
    let inset = px(crate::atoms::tokens::SliderGeometry::INSET);
    let bottom = spectrum.bottom() - inset;
    let first = point(spectrum.left() + spectrum.size.width * 0.25, bottom);
    let second = point(spectrum.left() + spectrum.size.width * 0.75, bottom);

    cx.simulate_mouse_down(first, gpui::MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_move(second, Some(gpui::MouseButton::Left), Modifiers::none());
    cx.run_until_parked();

    let thumb = cx.debug_bounds("color-picker-spectrum-thumb").unwrap();
    assert!(
        (thumb.center().x - second.x).abs() < px(2.),
        "spectrum={spectrum:?}, target={second:?}, thumb={thumb:?}"
    );
    assert!((thumb.center().y - bottom).abs() < px(2.));

    cx.simulate_mouse_up(second, gpui::MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let committed_thumb = cx.debug_bounds("color-picker-spectrum-thumb").unwrap();
    assert!((committed_thumb.center().x - second.x).abs() < px(2.));
    assert!((committed_thumb.center().y - bottom).abs() < px(2.));
}
