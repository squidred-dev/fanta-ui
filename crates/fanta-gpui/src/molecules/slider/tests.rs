use super::{Slider, SliderAction, SliderBackground, SliderPhase, SliderVariant};
use crate::test_support::mount_component;
use gpui::{Modifiers, MouseButton, TestAppContext};

#[gpui::test]
fn pointer_keyboard_and_disabled_slider_preserve_controlled_values(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| Slider::new("slider-test", 0.25, cx));
    let slider = cx.read(|cx| host.read(cx).component.clone());
    let bounds = cx.debug_bounds("slider-test").unwrap();
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().iter().map(|a| a.phase).collect::<Vec<_>>(),
        vec![
            SliderPhase::Begin,
            SliderPhase::Preview,
            SliderPhase::Commit
        ]
    );
    assert!((actions.borrow().last().unwrap().value - 0.5).abs() < 0.02);
    slider.read_with(cx, |slider, _| assert_eq!(slider.value(), 0.25));
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&SliderAction {
            value: 1.,
            phase: SliderPhase::Commit
        })
    );
    slider.update(cx, |s, cx| {
        s.configure(SliderVariant::Range, SliderBackground::Default, true, cx)
    });
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn stepped_drag_snaps_and_escape_cancels_without_committing(cx: &mut TestAppContext) {
    let (_, actions, cx) = mount_component(cx, |_, cx| {
        let mut s = Slider::new("stepped-test", 0.25, cx);
        s.configure(
            SliderVariant::Stepper { intervals: 4 },
            SliderBackground::Default,
            false,
            cx,
        );
        s
    });
    let bounds = cx.debug_bounds("stepped-test").unwrap();
    cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&SliderAction {
            value: 0.5,
            phase: SliderPhase::Preview
        })
    );
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&SliderAction {
            value: 0.25,
            phase: SliderPhase::Cancel
        })
    );
    cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(
        !actions
            .borrow()
            .iter()
            .any(|a| a.phase == SliderPhase::Commit)
    );
}

#[gpui::test]
fn range_fill_starts_at_the_rail_edge_while_the_thumb_stays_inset(cx: &mut TestAppContext) {
    let (_, _, cx) = mount_component(cx, |_, cx| Slider::new("edge-fill-test", 0.5, cx));
    let rail = cx.debug_bounds("edge-fill-test").unwrap();
    let fill = cx.debug_bounds("slider-fill").unwrap();
    let thumb = cx.debug_bounds("edge-fill-test-thumb").unwrap();

    assert_eq!(fill.left(), rail.left());
    assert!(thumb.left() > rail.left());
}
