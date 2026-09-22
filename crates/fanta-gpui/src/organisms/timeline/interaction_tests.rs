use super::{
    Timeline, TimelineAction, TimelineEasing, TimelineKeyframe, TimelineKeyframeTime,
    TimelinePlayback, TimelineProperty, TimelineTrack, TimelineViewData,
};
use crate::test_support::{assert_pointer_and_keyboard_parity, mount_component};
use gpui::{
    Focusable as _, Modifiers, MouseButton, ScrollDelta, ScrollWheelEvent, TestAppContext, point,
    px, size,
};

fn data() -> TimelineViewData {
    let mut track = TimelineTrack::new("shape", "Shape");
    track.selected = true;
    track.properties = vec![TimelineProperty {
        id: "opacity".into(),
        name: "Opacity".into(),
        value: "100%".into(),
        keyframes: vec![
            TimelineKeyframe {
                id: "a".into(),
                time_ms: 400,
                value: "0%".into(),
                easing: TimelineEasing::EaseOut,
            },
            TimelineKeyframe {
                id: "b".into(),
                time_ms: 1200,
                value: "100%".into(),
                easing: TimelineEasing::EaseOut,
            },
        ],
    }];
    TimelineViewData {
        tracks: vec![track],
        snapping: false,
        ..Default::default()
    }
}
#[gpui::test]
fn transport_pointer_enter_and_space_emit_exactly_one_request(cx: &mut TestAppContext) {
    let (_, actions, cx) = mount_component(cx, |_, cx| Timeline::new("test-timeline", data(), cx));
    cx.simulate_resize(size(px(1400.), px(360.)));
    cx.run_until_parked();
    assert_pointer_and_keyboard_parity(
        cx,
        "timeline-play",
        &actions,
        TimelineAction::PlayStateChangeRequested { playing: true },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "timeline-auto-keyframe",
        &actions,
        TimelineAction::AutoKeyframeChangeRequested { enabled: true },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "timeline-keyframe",
        &actions,
        TimelineAction::AddKeyframeRequested { time_ms: 0 },
    );
    assert_pointer_and_keyboard_parity(
        cx,
        "timeline-help",
        &actions,
        TimelineAction::HelpRequested,
    );
}
#[gpui::test]
fn ruler_tracks_and_keys_share_coordinates_at_every_width_and_zoom(cx: &mut TestAppContext) {
    let (host, _, cx) = mount_component(cx, |_, cx| Timeline::new("test-timeline", data(), cx));
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    for width in [1400., 800., 400.] {
        for zoom in [0.5, 1., 2.] {
            cx.simulate_resize(size(px(width), px(360.)));
            timeline.update(cx, |t, cx| {
                let mut d = t.view_data().clone();
                d.zoom = zoom;
                t.set_view_data(d, cx);
            });
            cx.run_until_parked();
            cx.run_until_parked();
            let ruler = cx.debug_bounds("timeline-ruler").unwrap();
            let lane = cx.debug_bounds("timeline-property-lane-opacity").unwrap();
            assert_eq!(ruler.left(), lane.left());
            assert_eq!(ruler.size.width, lane.size.width);
            let key = cx.debug_bounds("timeline-key-a").unwrap();
            let x = cx.read(|cx| timeline.read(cx).time_x(400));
            assert!(
                (key.center().x - ruler.left() - px(x)).abs() <= px(1.),
                "{key:?} {ruler:?}"
            );
            assert_eq!(
                key.center().y,
                cx.debug_bounds("timeline-property-row-opacity")
                    .unwrap()
                    .center()
                    .y
            );
        }
    }
}
#[gpui::test]
fn ruler_scrubbing_and_horizontal_pan_use_the_same_time_mapping(cx: &mut TestAppContext) {
    let (host, actions, cx) =
        mount_component(cx, |_, cx| Timeline::new("test-timeline", data(), cx));
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.simulate_resize(size(px(1000.), px(360.)));
    timeline.update(cx, |t, cx| {
        let mut d = t.view_data().clone();
        d.zoom = 2.;
        t.set_view_data(d, cx);
    });
    cx.run_until_parked();
    let ruler = cx.debug_bounds("timeline-ruler").unwrap();
    cx.simulate_event(ScrollWheelEvent {
        position: ruler.center(),
        delta: ScrollDelta::Pixels(point(px(-200.), px(0.))),
        ..Default::default()
    });
    cx.run_until_parked();
    let x = cx.read(|cx| timeline.read(cx).time_x(700));
    cx.simulate_click(
        point(ruler.left() + px(x), ruler.center().y),
        Modifiers::none(),
    );
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::SeekRequested { time_ms: 700 })
    );
    assert_eq!(
        cx.read(|cx| timeline.read(cx).view_data().current_time_ms),
        0
    );
}
#[gpui::test]
fn keyframe_drag_preserves_spacing_and_waits_for_host_echo(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let mut d = data();
        d.selected_keyframes = vec!["a".into(), "b".into()];
        Timeline::new("test-timeline", d, cx)
    });
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.simulate_resize(size(px(1000.), px(360.)));
    cx.run_until_parked();
    let from = cx.debug_bounds("timeline-key-a").unwrap().center();
    let to = point(px(999.), from.y);
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::KeyframesMoveRequested {
            keyframes: vec![
                TimelineKeyframeTime {
                    id: "a".into(),
                    time_ms: 1200
                },
                TimelineKeyframeTime {
                    id: "b".into(),
                    time_ms: 2000
                }
            ]
        })
    );
    assert_eq!(
        cx.read(|cx| timeline.read(cx).view_data().tracks[0].properties[0].keyframes[0].time_ms),
        400
    );
}
#[gpui::test]
fn escape_cancels_a_keyframe_drag(cx: &mut TestAppContext) {
    let (_, actions, cx) = mount_component(cx, |_, cx| Timeline::new("test-timeline", data(), cx));
    cx.run_until_parked();
    let from = cx.debug_bounds("timeline-key-a").unwrap().center();
    let to = point(from.x + px(100.), from.y);
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.simulate_keystrokes("escape");
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(
        !actions
            .borrow()
            .iter()
            .any(|a| matches!(a, TimelineAction::KeyframesMoveRequested { .. }))
    );
}
#[gpui::test]
fn playback_menu_and_time_inputs_request_host_values(cx: &mut TestAppContext) {
    let (_, actions, cx) = mount_component(cx, |_, cx| Timeline::new("test-timeline", data(), cx));
    cx.simulate_resize(size(px(1400.), px(360.)));
    cx.run_until_parked();
    let target = cx.debug_bounds("timeline-playback").unwrap().center();
    cx.simulate_click(target, Modifiers::none());
    cx.run_until_parked();
    let target = cx.debug_bounds("timeline-menu-Ping-pong").unwrap().center();
    cx.simulate_click(target, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::PlaybackChangeRequested {
            playback: TimelinePlayback::PingPong
        })
    );
    actions.borrow_mut().clear();
    let target = cx.debug_bounds("timeline-current-time").unwrap().center();
    cx.simulate_click(target, Modifiers::none());
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("1.25");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::SeekRequested { time_ms: 1250 })
    );
}
#[gpui::test]
fn read_only_tracks_allow_selection_but_never_emit_edits(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let mut d = data();
        d.read_only = true;
        d.selected_keyframes = vec!["a".into()];
        Timeline::new("test-timeline", d, cx)
    });
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.run_until_parked();
    cx.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("backspace secondary-d k");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    let target = cx.debug_bounds("timeline-key-a").unwrap().center();
    cx.simulate_click(target, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::KeyframeSelectionRequested {
            keyframe_ids: vec!["a".into()]
        })
    );
}
#[test]
fn host_time_units_and_zoom_are_normalized() {
    use super::TimelineTimeUnit;
    assert_eq!(TimelineTimeUnit::Seconds.parse("1.25"), Some(1250));
    assert_eq!(TimelineTimeUnit::Milliseconds.parse("1250"), Some(1250));
    assert_eq!(TimelineTimeUnit::Seconds.parse("NaN"), None);
    assert_eq!(TimelineTimeUnit::Seconds.parse("-1"), None);
    let d = TimelineViewData {
        duration_ms: 0,
        current_time_ms: 999,
        zoom: f32::NAN,
        ..Default::default()
    }
    .normalized();
    assert_eq!(d.duration_ms, 1);
    assert_eq!(d.current_time_ms, 1);
    assert_eq!(d.zoom, 1.);
}

#[gpui::test]
fn clips_open_settings_and_locked_keys_cannot_open_easing(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let mut d = data();
        d.selected_keyframes = vec!["a".into()];
        d.tracks[0].clips.push(super::TimelineClip {
            id: "fade".into(),
            name: "Fade in".into(),
            start_ms: 200,
            end_ms: 800,
            easing: TimelineEasing::EaseOut,
        });
        Timeline::new("test-timeline", d, cx)
    });
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.simulate_resize(size(px(1400.), px(800.)));
    cx.run_until_parked();
    let clip = cx.debug_bounds("timeline-span-shape-fade").unwrap();
    cx.simulate_click(clip.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(matches!(
        cx.read(|cx| timeline.read(cx).overlay.clone()),
        Some(super::Overlay::Easing(
            super::TimelineEasingTarget::Clip { .. }
        ))
    ));
    cx.simulate_keystrokes("escape");
    timeline.update(cx, |t, cx| {
        let mut d = t.view_data().clone();
        d.tracks[0].locked = true;
        t.set_view_data(d, cx);
    });
    cx.run_until_parked();
    actions.borrow_mut().clear();
    let button = cx.debug_bounds("timeline-easing").unwrap();
    cx.simulate_click(button.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.read(|cx| timeline.read(cx).overlay.is_none()));
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn header_scroll_and_open_menus_do_not_pan_tracks(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let mut d = data();
        d.zoom = 2.;
        Timeline::new("test-timeline", d, cx)
    });
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.simulate_resize(size(px(1400.), px(800.)));
    cx.run_until_parked();
    let header = cx.debug_bounds("timeline-controls").unwrap();
    cx.simulate_event(ScrollWheelEvent {
        position: header.center(),
        delta: ScrollDelta::Pixels(point(px(-100.), px(0.))),
        ..Default::default()
    });
    assert_eq!(cx.read(|cx| timeline.read(cx).scroll_x), 0.);
    let trigger = cx.debug_bounds("timeline-playback").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    cx.run_until_parked();
    let popup = cx.debug_bounds("timeline-menu").unwrap();
    cx.simulate_event(ScrollWheelEvent {
        position: popup.center(),
        delta: ScrollDelta::Pixels(point(px(-100.), px(-100.))),
        ..Default::default()
    });
    assert_eq!(cx.read(|cx| timeline.read(cx).scroll_x), 0.);
    cx.simulate_keystrokes("escape");
    let ruler = cx.debug_bounds("timeline-ruler").unwrap();
    cx.simulate_mouse_move(ruler.center(), None, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_event(gpui::PinchEvent {
        position: ruler.center(),
        delta: 0.25,
        modifiers: Modifiers::none(),
        phase: gpui::TouchPhase::Moved,
    });
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::ZoomChangeRequested { zoom: 2.5 })
    );
}

#[gpui::test]
fn easing_menu_emits_validated_requests_for_selected_keys(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let mut d = data();
        d.selected_keyframes = vec!["a".into()];
        Timeline::new("test-timeline", d, cx)
    });
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.simulate_resize(size(px(1400.), px(900.)));
    cx.run_until_parked();
    let trigger = cx.debug_bounds("timeline-easing").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    cx.run_until_parked();
    let input = cx.read(|cx| timeline.read(cx).easing_input.clone().unwrap());
    cx.update(|window, cx| {
        input.update(cx, |input, cx| input.set_value("2, 0, 0.5, 1", window, cx))
    });
    let apply = cx.debug_bounds("timeline-apply-bezier").unwrap();
    cx.simulate_click(apply.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    assert!(cx.read(|cx| timeline.read(cx).easing_error.is_some()));
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.set_value("0.2, -0.2, 0.8, 1.2", window, cx)
        })
    });
    cx.simulate_click(apply.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&TimelineAction::EasingChangeRequested {
            target: super::TimelineEasingTarget::Keyframes(vec!["a".into()]),
            easing: TimelineEasing::CubicBezier([0.2, -0.2, 0.8, 1.2]),
        })
    );
    assert_eq!(
        cx.read(
            |cx| timeline.read(cx).view_data().tracks[0].properties[0].keyframes[0]
                .easing
                .clone()
        ),
        TimelineEasing::EaseOut
    );
}

#[gpui::test]
fn layer_rename_is_controlled_and_cancelable(cx: &mut TestAppContext) {
    let (host, actions, cx) =
        mount_component(cx, |_, cx| Timeline::new("rename-timeline", data(), cx));
    let timeline = cx.read(|cx| host.read(cx).component.clone());
    cx.run_until_parked();
    cx.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("f2");
    cx.run_until_parked();
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("Renamed layer");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[TimelineAction::TrackRenameRequested {
            track_id: "shape".into(),
            name: "Renamed layer".into(),
        }]
    );
    assert_eq!(
        timeline.read_with(cx, |t, _| t.view_data.tracks[0].name.clone()),
        "Shape"
    );
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("f2");
    cx.run_until_parked();
    cx.simulate_input("Canceled");
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    assert!(timeline.read_with(cx, |t, _| t.renaming_track.is_none()));
    cx.simulate_keystrokes("f2");
    cx.run_until_parked();
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("   ");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    timeline.update(cx, |t, cx| {
        let mut view = t.view_data.clone();
        view.read_only = true;
        t.set_view_data(view, cx);
    });
    cx.simulate_keystrokes("f2");
    cx.run_until_parked();
    assert!(timeline.read_with(cx, |t, _| t.renaming_track.is_none()));
}
