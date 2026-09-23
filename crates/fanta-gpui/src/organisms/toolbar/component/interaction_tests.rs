use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, KeyBinding, Modifiers,
    MouseButton, MouseDownEvent, ParentElement as _, Render, Subscription, TestAppContext,
    VisualTestContext, Window, actions, div, point, px, size,
};
use gpui_component::{IconName, Root};

use super::*;
use crate::toolbar::{ToolbarChromeControl, ToolbarControlValue};

actions!(toolbar_input_regression, [CompetingDeleteSelection]);

struct TestHost {
    toolbar: Entity<EditorToolbar>,
    actions: Rc<RefCell<Vec<ToolbarAction>>>,
    competing_delete_count: usize,
    /// Mouse-down presses that reached the host canvas underneath the dock.
    canvas_presses: usize,
    canvas_scrolls: usize,
    _subscription: Subscription,
}

impl TestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let toolbar = cx.new(|cx| {
            EditorToolbar::new(
                "test-toolbar",
                ToolbarMode::Design,
                ToolbarTool::Move,
                100,
                window,
                cx,
            )
        });
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&toolbar, move |_, _, action: &ToolbarAction, _| {
            captured_actions.borrow_mut().push(action.clone());
        });
        Self {
            toolbar,
            actions,
            competing_delete_count: 0,
            canvas_presses: 0,
            canvas_scrolls: 0,
            _subscription: subscription,
        }
    }
}

impl Render for TestHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("test-canvas")
            .debug_selector(|| "test-canvas".to_owned())
            .relative()
            .size_full()
            .key_context("TestCanvas")
            .on_action(cx.listener(|this, _: &CompetingDeleteSelection, _, cx| {
                this.competing_delete_count += 1;
                cx.stop_propagation();
            }))
            // The canvas probe: a real host canvas listens for presses the
            // same way, so anything the dock fails to occlude lands here.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    this.canvas_presses += 1;
                    cx.notify();
                }),
            )
            .on_scroll_wheel(cx.listener(|this, _, _, _| this.canvas_scrolls += 1))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom(px(18.))
                    .flex()
                    .px_4()
                    .gap(px(8.))
                    .justify_center()
                    .child(self.toolbar.clone())
                    .child(
                        div()
                            .debug_selector(|| "test-host-trailing-controls".to_owned())
                            .w(px(72.))
                            .h(px(40.))
                            .flex_none(),
                    ),
            )
    }
}

fn setup(cx: &mut TestAppContext) -> (Entity<TestHost>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let host_slot = Rc::new(RefCell::new(None));
    let captured_host = host_slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let host = cx.new(|cx| TestHost::new(window, cx));
        *captured_host.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    let host = host_slot
        .borrow_mut()
        .take()
        .expect("toolbar test host should be installed");
    (host, cx)
}

#[gpui::test]
fn host_capability_filters_remove_dead_toolbar_controls(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    toolbar.update(cx, |toolbar, cx| {
        toolbar.set_supported_tools(
            [
                ToolbarTool::Move,
                ToolbarTool::Rectangle,
                ToolbarTool::Actions,
            ],
            cx,
        );
        toolbar.set_supported_secondary_controls([ToolbarSecondaryControl::DevInspect], cx);
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-group-shape-tools-trigger")
            .is_some()
    );
    assert!(
        cx.debug_bounds("toolbar-group-creation-tools-trigger")
            .is_none()
    );

    toolbar.update(cx, |toolbar, cx| {
        toolbar.toggle_tool_group(ToolbarToolGroup::Shape, false, cx);
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("toolbar-flyout-rectangle").is_some());
    assert!(cx.debug_bounds("toolbar-flyout-ellipse").is_none());

    toolbar.update(cx, |toolbar, cx| {
        toolbar.request_tool(ToolbarTool::Ellipse, cx);
        toolbar.set_mode(ToolbarMode::Dev, cx);
    });
    cx.run_until_parked();
    let actions = cx.read(|app| host.read(app).actions.clone());
    assert!(actions.borrow().is_empty());
    assert!(cx.debug_bounds("toolbar-secondary-dev-inspect").is_some());
    assert!(cx.debug_bounds("toolbar-secondary-dev-annotate").is_none());
    assert!(cx.debug_bounds("toolbar-secondary-dev-ready").is_none());
}

#[gpui::test]
fn actions_query_accepts_backward_and_forward_deletion(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());

    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();

    cx.update(|_, app| {
        app.clear_key_bindings();
        app.bind_keys([
            KeyBinding::new("backspace", CompetingDeleteSelection, Some("TestCanvas")),
            KeyBinding::new("delete", CompetingDeleteSelection, Some("TestCanvas")),
        ]);
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("a b c backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "ab"
    );

    cx.simulate_keystrokes("left delete");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "a"
    );

    cx.simulate_keystrokes("b c shift-left shift-left backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "a"
    );

    cx.simulate_keystrokes("secondary-a delete");
    assert!(cx.read(|app| toolbar.read(app).command_input.read(app).value().is_empty()));

    cx.simulate_keystrokes("a b c space d e f");
    #[cfg(target_os = "macos")]
    cx.simulate_keystrokes("alt-backspace");
    #[cfg(not(target_os = "macos"))]
    cx.simulate_keystrokes("ctrl-backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "abc "
    );

    #[cfg(target_os = "macos")]
    {
        cx.simulate_keystrokes("x y z cmd-backspace");
        assert!(cx.read(|app| toolbar.read(app).command_input.read(app).value().is_empty()));
    }

    assert_eq!(
        cx.read(|app| host.read(app).competing_delete_count),
        0,
        "focused toolbar input must suppress a competing canvas delete action"
    );
}

#[gpui::test]
fn intact_input_bindings_do_not_double_dispatch_the_fallback(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("a b backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "a",
        "the fallback must defer to an intact Input binding"
    );
}

#[gpui::test]
fn higher_precedence_global_delete_binding_cannot_steal_intact_input_deletion(
    cx: &mut TestAppContext,
) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();

    cx.update(|_, app| {
        app.bind_keys([KeyBinding::new("backspace", CompetingDeleteSelection, None)]);
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("a b backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "a"
    );
    assert_eq!(cx.read(|app| host.read(app).competing_delete_count), 0);
}

#[gpui::test]
fn intentional_input_delete_remap_keeps_precedence_over_the_fallback(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
        app.bind_keys([KeyBinding::new(
            "backspace",
            CompetingDeleteSelection,
            Some("Input"),
        )]);
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("a b backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "ab"
    );
    assert_eq!(cx.read(|app| host.read(app).competing_delete_count), 1);
}

#[gpui::test]
fn outer_chord_starting_with_backspace_cannot_delay_input_deletion(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
        app.clear_key_bindings();
        app.bind_keys([KeyBinding::new(
            "backspace x",
            CompetingDeleteSelection,
            Some("TestCanvas"),
        )]);
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("a b backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "a"
    );
    cx.update(|window, _| {
        assert!(
            !window.has_pending_keystrokes(),
            "the outer chord must not retain Backspace as a pending prefix"
        );
    });
    assert_eq!(cx.read(|app| host.read(app).competing_delete_count), 0);
}

#[gpui::test]
fn fallback_deletion_discards_an_unmatched_pending_outer_chord(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
        app.clear_key_bindings();
        app.bind_keys([KeyBinding::new(
            "ctrl-k x",
            CompetingDeleteSelection,
            Some("TestCanvas"),
        )]);
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("a b ctrl-k");
    cx.update(|window, _| {
        assert!(
            window.has_pending_keystrokes(),
            "Ctrl+K should start the chord"
        );
    });

    cx.simulate_keystrokes("backspace");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "a"
    );
    cx.update(|window, _| {
        assert!(
            !window.has_pending_keystrokes(),
            "handled deletion must discard the stale chord prefix"
        );
    });

    cx.simulate_keystrokes("c");
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "ac",
        "discarding the chord must preserve text-input focus"
    );
    assert_eq!(cx.read(|app| host.read(app).competing_delete_count), 0);
}

#[gpui::test]
fn opening_a_popup_does_not_reflow_a_contextual_dock(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| toolbar.set_mode(ToolbarMode::Motion, cx));
    });
    cx.run_until_parked();

    let dock_before = cx
        .debug_bounds("editor-toolbar-surface")
        .expect("Motion dock should render");
    let trigger_before = cx
        .debug_bounds("toolbar-tool-actions")
        .expect("Actions trigger should render");

    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();

    assert_eq!(cx.debug_bounds("editor-toolbar-surface"), Some(dock_before));
    assert_eq!(
        cx.debug_bounds("toolbar-tool-actions"),
        Some(trigger_before)
    );
}

#[gpui::test]
fn tool_menu_aligns_with_the_dock_edge_and_a_four_pixel_gap(cx: &mut TestAppContext) {
    let (_, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    cx.run_until_parked();

    let trigger = cx
        .debug_bounds("toolbar-group-move-tools-trigger")
        .expect("Move tools caret should render");
    cx.simulate_click(trigger.center(), Modifiers::none());
    cx.run_until_parked();

    let menu = cx
        .debug_bounds("toolbar-tool-flyout")
        .expect("Move tools flyout should render");
    let dock = cx.debug_bounds("editor-toolbar-surface").unwrap();
    assert_eq!(menu.bottom() + px(POPOVER_GAP), dock.top());
    assert_eq!(menu.left(), dock.left());
    assert!(menu.left() <= trigger.left());
    assert!(menu.right() >= trigger.right());
}

#[gpui::test]
fn disclosure_triggers_close_their_own_open_popups(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    cx.run_until_parked();
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());

    for (trigger, popup) in [
        ("toolbar-group-move-tools-trigger", "toolbar-tool-flyout"),
        ("toolbar-tool-actions", "toolbar-actions-palette"),
    ] {
        let trigger_bounds = cx
            .debug_bounds(trigger)
            .unwrap_or_else(|| panic!("{trigger} should render"));
        cx.simulate_click(trigger_bounds.center(), Modifiers::none());
        cx.run_until_parked();
        assert!(cx.debug_bounds(popup).is_some(), "{popup} should open");

        let trigger_bounds = cx
            .debug_bounds(trigger)
            .unwrap_or_else(|| panic!("{trigger} should remain in place"));
        cx.simulate_click(trigger_bounds.center(), Modifiers::none());
        cx.run_until_parked();
        assert!(
            cx.debug_bounds(popup).is_none(),
            "{trigger} should close its own {popup}"
        );
        if matches!(popup, "toolbar-actions-palette") {
            assert!(
                cx.update(|window, app| toolbar.focus_handle(app).is_focused(window)),
                "{trigger} should restore toolbar focus after removing its text input"
            );
        }
    }
}

#[gpui::test]
fn tall_popups_scroll_and_remain_inside_a_short_window(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(240.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());

    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();
    let palette = cx
        .debug_bounds("toolbar-actions-palette")
        .expect("Actions palette should render in a short window");
    assert!(palette.top() >= px(8.));
    assert!(palette.bottom() <= px(232.));
    assert!(
        (palette.bottom() + px(POPOVER_GAP)
            - cx.debug_bounds("editor-toolbar-surface").unwrap().top())
        .abs()
            <= px(0.5),
        "short windows must shrink the results without covering the dock"
    );
    assert_eq!(
        cx.debug_bounds("toolbar-command-generate-a-design")
            .expect("command row should render")
            .size
            .height,
        px(42.),
        "scrolling must preserve command-row hit targets"
    );

    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.toggle_tool_group(ToolbarToolGroup::Shape, false, cx);
        });
    });
    cx.run_until_parked();
    let tool_menu = cx
        .debug_bounds("toolbar-tool-flyout")
        .expect("large tool menu should render in a short window");
    assert!(tool_menu.top() >= px(8.));
    assert!(tool_menu.bottom() <= px(232.));
    assert_eq!(
        cx.debug_bounds("toolbar-flyout-rectangle")
            .expect("tool row should render")
            .size
            .height,
        px(38.),
        "scrolling must preserve tool-row hit targets"
    );
}

#[gpui::test]
fn shift_zoom_shortcuts_emit_figma_zoom_bindings(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let actions = cx.read(|app| host.read(app).actions.clone());
    cx.update(|window, app| {
        let focus_handle = host.read(app).toolbar.read(app).focus_handle.clone();
        focus_handle.focus(window, app);
    });
    cx.run_until_parked();

    cx.simulate_keystrokes("shift-1");
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::CommandInvoked {
            command: ToolbarCommand::ZoomToFit,
        }]
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("shift-2");
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::CommandInvoked {
            command: ToolbarCommand::ZoomToSelection,
        }]
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("shift-0");
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::ZoomChangeRequested { percent: 100 }]
    );
}

#[gpui::test]
fn motion_style_chip_offers_host_candidates(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());

    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.set_mode(ToolbarMode::Motion, cx);
            toolbar.set_motion_options(
                MotionToolbarOptions {
                    animation_style: "Spring".into(),
                    available_animation_styles: vec!["Fade in".into(), "Spring".into()],
                    ..MotionToolbarOptions::default()
                },
                cx,
            );
        });
    });
    cx.run_until_parked();

    let chip = cx
        .debug_bounds("toolbar-secondary-motion-style")
        .expect("animation style chip should render");
    cx.simulate_click(chip.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-motion-style-editor").is_some(),
        "the style chip should open a candidate menu"
    );

    actions.borrow_mut().clear();
    let candidate = cx
        .debug_bounds("toolbar-motion-style-option-fade-in")
        .expect("host-supplied style should render");
    cx.simulate_click(candidate.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::ControlChangeRequested {
            mode: ToolbarMode::Motion,
            control: ToolbarSecondaryControl::MotionAnimationStyle,
            value: ToolbarControlValue::Choice("Fade in".into()),
        }]
    );
    assert!(
        cx.debug_bounds("toolbar-motion-style-editor").is_none(),
        "choosing a candidate should close the editor"
    );
}

#[gpui::test]
fn motion_style_editor_arrows_move_the_highlight_and_enter_commits(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());

    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| toolbar.set_mode(ToolbarMode::Motion, cx));
    });
    cx.run_until_parked();

    let chip = cx
        .debug_bounds("toolbar-secondary-motion-style")
        .expect("animation style chip should render");
    cx.simulate_click(chip.center(), Modifiers::none());
    cx.run_until_parked();

    // The editor opens highlighted on the accepted value ("Fade in", index
    // 0); Right moves to "Spring" and Enter commits it.
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("right enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::ControlChangeRequested {
            mode: ToolbarMode::Motion,
            control: ToolbarSecondaryControl::MotionAnimationStyle,
            value: ToolbarControlValue::Choice("Spring".into()),
        }],
        "Right should move the highlight and Enter should commit it"
    );
    assert!(cx.debug_bounds("toolbar-motion-style-editor").is_none());
}

#[gpui::test]
fn flyout_arrow_navigation_highlights_and_commits(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let actions = cx.read(|app| host.read(app).actions.clone());

    let caret = cx
        .debug_bounds("toolbar-group-move-tools-trigger")
        .expect("Move tools caret should render");
    cx.simulate_click(caret.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("toolbar-tool-flyout").is_some());

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("down down enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::ToolChangeRequested {
            mode: ToolbarMode::Design,
            tool: ToolbarTool::PathSelect,
        }],
        "arrow navigation should highlight Path selection and Enter should commit it"
    );
    assert!(cx.debug_bounds("toolbar-tool-flyout").is_none());
}

#[gpui::test]
fn fuzzy_actions_query_ranks_the_best_match_first(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());

    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();

    // "zf" is not a substring of any label; the word-boundary subsequence
    // ranks Zoom to fit first and Enter runs the highlighted best match.
    cx.simulate_keystrokes("z f");
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-command-zoom-to-fit").is_some(),
        "a scattered query should still surface its subsequence match"
    );
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::CommandInvoked {
            command: ToolbarCommand::ZoomToFit,
        }]
    );
    // Reopening must not have accumulated a stray newline from the Enter
    // keystroke's key_char falling through the propagated action.
    cx.update(|window, app| {
        toolbar.update(app, |toolbar, cx| toolbar.open_actions(window, cx));
    });
    cx.run_until_parked();
    assert_eq!(
        cx.read(|app| toolbar.read(app).command_input.read(app).value()),
        "zf"
    );
}

#[gpui::test]
fn narrow_primary_row_shows_edge_fade_affordances(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| toolbar.set_mode(ToolbarMode::Motion, cx));
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-primary-fade-end").is_none(),
        "a wide dock should not show an overflow fade"
    );

    cx.simulate_resize(size(px(420.), px(700.)));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-primary-fade-end").is_some(),
        "clipped trailing tools should surface an end fade"
    );
    assert!(
        cx.debug_bounds("toolbar-primary-fade-start").is_none(),
        "an unscrolled row should not fade its start edge"
    );

    cx.update(|_, app| {
        let handle = toolbar.read(app).primary_scroll_handle.clone();
        let max = handle.max_offset().x;
        handle.set_offset(point(-max, px(0.)));
        toolbar.update(app, |_, cx| cx.notify());
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-primary-fade-start").is_some(),
        "scrolling to the end should surface a start fade"
    );
    assert!(
        cx.debug_bounds("toolbar-primary-fade-end").is_none(),
        "the end fade should clear once the row is fully scrolled"
    );
}

fn example_chrome_controls(left_active: bool) -> Vec<ToolbarChromeControl> {
    vec![
        ToolbarChromeControl::new("fit", IconName::Maximize, "Fit to view").shortcut("⇧ 1"),
        ToolbarChromeControl::new(
            "left-sidebar",
            if left_active {
                IconName::PanelLeftClose
            } else {
                IconName::PanelLeftOpen
            },
            "Toggle layers sidebar",
        )
        .active(left_active),
        ToolbarChromeControl::new(
            "right-sidebar",
            IconName::PanelRightOpen,
            "Toggle inspector sidebar",
        ),
    ]
}

#[gpui::test]
fn dock_occludes_the_host_canvas_beneath_it(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.set_chrome_controls(example_chrome_controls(false), cx);
        });
    });
    cx.run_until_parked();
    let presses = |cx: &mut VisualTestContext| cx.read(|app| host.read(app).canvas_presses);

    // A tool press activates the tool and stops at the dock.
    let text = cx
        .debug_bounds("toolbar-tool-text")
        .expect("Text tool should render");
    cx.simulate_click(text.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::ToolChangeRequested {
            mode: ToolbarMode::Design,
            tool: ToolbarTool::Text,
        }]
    );
    assert_eq!(presses(cx), 0, "a tool press must not reach the canvas");

    // Dock padding (no control under the pointer) still stops the press.
    let surface = cx
        .debug_bounds("editor-toolbar-surface")
        .expect("dock surface should render");
    cx.simulate_click(surface.origin + point(px(3.), px(3.)), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(presses(cx), 0, "dock padding must not reach the canvas");

    // The mode tray gap and the host chrome capsule are dock too.
    let tray = cx
        .debug_bounds("toolbar-mode-selector")
        .expect("Design mode tile should render");
    cx.simulate_click(tray.origin + point(px(-3.), px(14.)), Modifiers::none());
    let fit = cx
        .debug_bounds("toolbar-chrome-fit")
        .expect("fit chrome control should render");
    cx.simulate_click(fit.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(presses(cx), 0, "chrome controls must not reach the canvas");
    assert_eq!(
        actions.borrow().last(),
        Some(&ToolbarAction::ChromeControlInvoked { id: "fit".into() })
    );

    // Outside the dock the canvas is live.
    let canvas = cx
        .debug_bounds("test-canvas")
        .expect("host canvas should render");
    cx.simulate_click(canvas.origin + point(px(20.), px(20.)), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        presses(cx),
        1,
        "presses beside the dock must reach the canvas"
    );
    assert!(
        !surface.contains(&(canvas.origin + point(px(20.), px(20.)))),
        "the probe point must lie outside the dock"
    );
}

#[gpui::test]
fn chrome_controls_render_in_the_dock_and_emit_typed_intents(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.simulate_resize(size(px(900.), px(700.)));
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());

    assert!(
        cx.debug_bounds("toolbar-chrome-cluster").is_none(),
        "no capsule renders until the host supplies chrome controls"
    );

    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.set_chrome_controls(example_chrome_controls(true), cx);
        });
    });
    cx.run_until_parked();

    let surface = cx
        .debug_bounds("editor-toolbar-surface")
        .expect("dock surface should render");
    let row = cx
        .debug_bounds("toolbar-main-row")
        .expect("main row should render");
    let cluster = cx
        .debug_bounds("toolbar-chrome-cluster")
        .expect("chrome capsule should render");
    let actions_button = cx
        .debug_bounds("toolbar-tool-actions")
        .expect("Actions button should render");
    assert!(cluster.is_contained_within(&surface));
    assert!(
        cluster.left() >= row.left()
            && cluster.right() <= row.right()
            && cluster.top() >= row.top()
            && cluster.bottom() <= row.bottom()
    );
    assert!(
        cluster.left() > actions_button.right(),
        "the capsule trails the Actions button: {cluster:?} vs {actions_button:?}"
    );
    for selector in [
        "toolbar-chrome-fit",
        "toolbar-chrome-left-sidebar",
        "toolbar-chrome-right-sidebar",
    ] {
        let bounds = cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} should render"));
        assert!(bounds.is_contained_within(&cluster));
        assert_eq!(bounds.size, size(px(28.), px(28.)));
    }
    assert!(
        cx.debug_bounds("toolbar-chrome-left-sidebar-active")
            .is_some()
            && cx.debug_bounds("toolbar-chrome-fit-active").is_none()
            && cx
                .debug_bounds("toolbar-chrome-right-sidebar-active")
                .is_none(),
        "only the active control should carry the active treatment"
    );

    // Pointer activation, then Enter/Space parity on the focused control.
    let left = cx
        .debug_bounds("toolbar-chrome-left-sidebar")
        .expect("left sidebar control should render");
    cx.simulate_click(left.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[ToolbarAction::ChromeControlInvoked {
            id: "left-sidebar".into(),
        }]
    );
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter space");
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            ToolbarAction::ChromeControlInvoked {
                id: "left-sidebar".into(),
            },
            ToolbarAction::ChromeControlInvoked {
                id: "left-sidebar".into(),
            },
        ]
    );

    // The host echoes the toggled state; the toolbar never flips it itself.
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            assert!(toolbar.chrome_controls()[1].active);
            toolbar.set_chrome_controls(example_chrome_controls(false), cx);
        });
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("toolbar-chrome-left-sidebar-active")
            .is_none()
    );
    assert!(
        cx.debug_bounds("toolbar-chrome-left-sidebar-inactive")
            .is_some()
    );

    // Duplicate ids collapse to the first occurrence.
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.set_chrome_controls(
                [
                    ToolbarChromeControl::new("fit", IconName::Maximize, "Fit"),
                    ToolbarChromeControl::new("fit", IconName::Minimize, "Fit again"),
                ],
                cx,
            );
            assert_eq!(toolbar.chrome_controls().len(), 1);
            assert_eq!(toolbar.chrome_controls()[0].label, "Fit");
        });
    });
    cx.run_until_parked();

    // Clearing removes the capsule.
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.set_chrome_controls(std::iter::empty(), cx);
        });
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("toolbar-chrome-cluster").is_none());
}

fn click_control(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing {selector}"));
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
fn mode_dropdown_emits_all_four_modes_and_preserves_controlled_state(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());
    for mode in ToolbarMode::ALL {
        click_control(cx, "toolbar-mode-selector");
        click_control(
            cx,
            match mode {
                ToolbarMode::Design => "toolbar-mode-design",
                ToolbarMode::Motion => "toolbar-mode-motion",
                ToolbarMode::Draw => "toolbar-mode-draw",
                ToolbarMode::Dev => "toolbar-mode-dev",
            },
        );
        assert_eq!(
            actions.borrow().last(),
            Some(&ToolbarAction::ModeChangeRequested { mode: *mode })
        );
        assert_eq!(cx.read(|app| toolbar.read(app).mode()), ToolbarMode::Design);
    }
    actions.borrow_mut().clear();
    click_control(cx, "toolbar-mode-selector");
    cx.simulate_keystrokes("down down enter");
    assert_eq!(
        actions.borrow().last(),
        Some(&ToolbarAction::ModeChangeRequested {
            mode: ToolbarMode::Draw
        })
    );
}

#[gpui::test]
fn dock_has_one_row_in_design_and_two_in_specialist_modes(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.simulate_resize(size(px(1200.), px(800.)));
    for &mode in ToolbarMode::ALL {
        cx.update(|_, app| {
            toolbar.update(app, |t, cx| {
                t.set_mode(mode, cx);
                if mode == ToolbarMode::Draw {
                    t.set_active_tool(ToolbarTool::Brush, cx);
                }
            })
        });
        cx.run_until_parked();
        let surface = cx.debug_bounds("editor-toolbar-surface").unwrap();
        assert!(
            surface.size.height
                <= px(if mode == ToolbarMode::Design {
                    52.
                } else {
                    100.
                }),
            "{mode:?}: {surface:?}"
        );
        assert_eq!(
            cx.debug_bounds("toolbar-context-row").is_some(),
            mode != ToolbarMode::Design
        );
        for selector in ["toolbar-mode-selector", "toolbar-tool-actions"] {
            assert!(
                cx.debug_bounds(selector)
                    .unwrap()
                    .is_contained_within(&surface)
            );
        }
        for removed in [
            "toolbar-agent-launcher",
            "toolbar-zoom-control",
            "toolbar-tool-resources",
            "toolbar-utility-row",
            "toolbar-chrome-toggle-left-sidebar",
            "toolbar-chrome-toggle-right-sidebar",
        ] {
            assert!(cx.debug_bounds(removed).is_none());
        }
    }
    for width in [420., 320., 220.] {
        cx.simulate_resize(size(px(width), px(700.)));
        cx.run_until_parked();
        let root = cx.debug_bounds("editor-toolbar").unwrap();
        assert!(root.size.width <= px(width - 112.), "{root:?}");
        assert!(
            cx.debug_bounds("toolbar-tool-actions")
                .unwrap()
                .is_contained_within(&root)
        );
    }
}

#[gpui::test]
fn actions_center_over_the_dock_and_wheel_never_reaches_canvas(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.simulate_resize(size(px(900.), px(700.)));
    click_control(cx, "toolbar-tool-actions");
    let palette = cx.debug_bounds("toolbar-actions-palette").unwrap();
    let dock = cx.debug_bounds("editor-toolbar-surface").unwrap();
    assert!((dock.center().x - px(450.)).abs() > px(1.));
    assert!((palette.center().x - dock.center().x).abs() < px(1.));
    assert_eq!(palette.bottom() + px(POPOVER_GAP), dock.top());
    assert!(cx.debug_bounds("toolbar-actions-scope-assets").is_none());
    assert!(cx.debug_bounds("toolbar-command-browse-plugins").is_none());
    let results = cx.debug_bounds("toolbar-actions-results").unwrap();
    for _ in 0..30 {
        cx.simulate_event(gpui::ScrollWheelEvent {
            position: results.center(),
            delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-120.))),
            ..Default::default()
        });
        cx.run_until_parked();
    }
    assert!(cx.read(|app| toolbar.read(app).command_scroll_handle.offset().y) < px(0.));
    assert_eq!(cx.read(|app| host.read(app).canvas_scrolls), 0);
    cx.simulate_event(gpui::ScrollWheelEvent {
        position: point(px(20.), px(20.)),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    assert_eq!(cx.read(|app| host.read(app).canvas_scrolls), 1);
}

#[gpui::test]
fn draw_values_and_preset_labels_never_resize_the_dock(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    for width in [1200., 420.] {
        cx.simulate_resize(size(px(width), px(800.)));
        for tool in [
            ToolbarTool::Brush,
            ToolbarTool::MagicWand,
            ToolbarTool::Crop,
        ] {
            cx.update(|_, app| {
                toolbar.update(app, |toolbar, cx| {
                    toolbar.set_mode(ToolbarMode::Draw, cx);
                    toolbar.set_active_tool(tool, cx);
                    toolbar.set_draw_options(crate::toolbar::DrawToolbarOptions::default(), cx);
                })
            });
            cx.run_until_parked();
            let selectors = [
                "editor-toolbar-surface",
                "toolbar-tool-actions",
                "toolbar-mode-selector",
            ];
            let before = selectors.map(|selector| cx.debug_bounds(selector).unwrap());
            for (size, percent, name) in [
                (1, 0, "Ink"),
                (
                    5000,
                    100,
                    "Extra long brush preset or custom crop aspect ratio",
                ),
                (24, 43, "Soft round"),
            ] {
                cx.update(|_, app| {
                    toolbar.update(app, |toolbar, cx| {
                        let mut options = toolbar.draw_options().clone();
                        options.size = size;
                        options.hardness = percent;
                        options.opacity = percent;
                        options.flow = percent;
                        options.smoothing = percent;
                        options.feather = size;
                        options.tolerance = if percent == 100 { 255 } else { percent };
                        options.brush_tip = name.into();
                        options.crop_ratio = name.into();
                        toolbar.set_draw_options(options, cx);
                    })
                });
                cx.run_until_parked();
                assert_eq!(
                    selectors.map(|selector| cx.debug_bounds(selector).unwrap()),
                    before,
                    "value changes moved the {tool:?} dock at width {width}"
                );
            }
        }
    }
}

#[gpui::test]
fn motion_time_and_preset_labels_keep_fixed_control_bounds(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.simulate_resize(size(px(1200.), px(800.)));
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| toolbar.set_mode(ToolbarMode::Motion, cx))
    });
    cx.run_until_parked();
    let selectors = [
        "editor-toolbar-surface",
        "toolbar-motion-time",
        "toolbar-secondary-motion-style",
        "toolbar-tool-actions",
    ];
    let before = selectors.map(|selector| cx.debug_bounds(selector).unwrap());
    for (time, name) in [
        (999, "Pop"),
        (u32::MAX, "Long custom spring animation preset"),
        (0, "Fade in"),
    ] {
        cx.update(|_, app| {
            toolbar.update(app, |toolbar, cx| {
                let mut options = toolbar.motion_options().clone();
                options.current_time_ms = time;
                options.duration_ms = time;
                options.animation_style = name.into();
                toolbar.set_motion_options(options, cx);
            })
        });
        cx.run_until_parked();
        assert_eq!(
            selectors.map(|selector| cx.debug_bounds(selector).unwrap()),
            before
        );
    }
}

#[gpui::test]
fn draw_brush_values_and_crop_actions_emit_controlled_intents(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    let actions = cx.read(|app| host.read(app).actions.clone());
    cx.simulate_resize(size(px(1200.), px(800.)));
    cx.update(|_, app| {
        toolbar.update(app, |t, cx| {
            t.set_mode(ToolbarMode::Draw, cx);
            t.set_active_tool(ToolbarTool::Brush, cx);
        })
    });
    cx.run_until_parked();
    click_control(cx, "toolbar-draw-size");
    assert!(cx.debug_bounds("toolbar-draw-number-popup").is_some());
    assert!(
        cx.update(|window, app| toolbar
            .read(app)
            .draw_input
            .focus_handle(app)
            .is_focused(window)),
        "numeric field must have focus"
    );
    cx.simulate_keystrokes("secondary-a 4 8");
    assert_eq!(
        cx.read(|app| toolbar.read(app).draw_input.read(app).value()),
        "48"
    );
    cx.simulate_keystrokes("enter");
    let options = actions
        .borrow()
        .iter()
        .find_map(|a| {
            if let ToolbarAction::DrawOptionsChangeRequested { options } = a {
                Some(options.clone())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(options.size, 48);
    assert_eq!(
        cx.read(|app| toolbar.read(app).draw_options().size),
        24,
        "component must wait for host echo"
    );
    cx.update(|_, app| toolbar.update(app, |t, cx| t.set_draw_options(options, cx)));
    assert_eq!(cx.read(|app| toolbar.read(app).draw_options().size), 48);
    click_control(cx, "toolbar-group-brush-tools-trigger");
    click_control(cx, "toolbar-flyout-eraser");
    assert_eq!(
        actions.borrow().last(),
        Some(&ToolbarAction::ToolChangeRequested {
            mode: ToolbarMode::Draw,
            tool: ToolbarTool::Eraser
        })
    );
    cx.update(|_, app| toolbar.update(app, |t, cx| t.set_active_tool(ToolbarTool::Crop, cx)));
    cx.run_until_parked();
    click_control(cx, "toolbar-draw-action-ApplyCrop");
    assert_eq!(
        actions.borrow().last(),
        Some(&ToolbarAction::DrawActionInvoked {
            action: crate::toolbar::DrawToolbarAction::ApplyCrop
        })
    );
}

#[gpui::test]
fn empty_host_brush_catalog_has_no_keyboard_candidate(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let toolbar = cx.read(|app| host.read(app).toolbar.clone());
    cx.update(|_, app| {
        toolbar.update(app, |toolbar, cx| {
            toolbar.set_mode(ToolbarMode::Draw, cx);
            toolbar.set_active_tool(ToolbarTool::Brush, cx);
            let mut options = toolbar.draw_options().clone();
            options.brush_tips.clear();
            toolbar.set_draw_options(options, cx);
        })
    });
    cx.run_until_parked();
    click_control(cx, "toolbar-draw-choice-BrushTip");
    cx.simulate_keystrokes("down up enter escape");
    assert!(cx.read(|app| host.read(app).actions.borrow().is_empty()));
}
